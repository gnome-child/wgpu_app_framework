use crate::geometry::point;
use crate::paint;
use crate::render;

use super::chain::{FilterChainContext, FilterSource};
use super::effects::refraction_effect;
use super::geometry::prepare_filter;
use super::params::AlphaMode;
use super::pass::{BlurLabels, BlurPass, CompositePass, EffectPass, PassLabels, RefractionPass};
use super::state::Renderer;
use super::storage::ScratchTargets;
use super::target::Target;

pub(in crate::render) struct FilterDraw<'a> {
    pub(in crate::render) render_context: &'a render::Context,
    pub(in crate::render) encoder: &'a mut wgpu::CommandEncoder,
    pub(in crate::render) target: Target,
    pub(in crate::render) scratch: FilterScratch,
    pub(in crate::render) owner: FilterOwner,
    pub(in crate::render) source: FilterSource<'a>,
    pub(in crate::render) output: &'a wgpu::TextureView,
    pub(in crate::render) filter: paint::Filter,
    pub(in crate::render) scissor: Option<render::Scissor>,
}

#[derive(Clone, Copy)]
pub(in crate::render) struct FilterScratch {
    target: Target,
    rect: paint::Rect,
    bounds: paint::Rect,
}

#[derive(Clone, Copy, Debug)]
pub(in crate::render) enum FilterOwner {
    PaneBackdrop,
    PaneSurfaceNoise,
}

#[derive(Clone, Copy, Default)]
pub(in crate::render) struct FilterWork {
    pub(in crate::render) intermediate_clears: usize,
    pub(in crate::render) intermediate_clear_bytes: u64,
    pub(in crate::render) intermediate_composites: usize,
    pub(in crate::render) intermediate_composite_bytes: u64,
    pub(in crate::render) largest_intermediate_bytes: u64,
}

impl FilterScratch {
    pub(in crate::render) fn bounded(
        rect: paint::Rect,
        bounds: paint::Rect,
        scale_factor: f32,
    ) -> Self {
        Self {
            target: Target::from_logical_area(bounds.area, scale_factor),
            rect: paint::Rect::rounded(
                point::logical(
                    rect.origin.x() - bounds.origin.x(),
                    rect.origin.y() - bounds.origin.y(),
                ),
                rect.area,
                rect.rounding,
            ),
            bounds,
        }
    }
}

impl FilterWork {
    fn record_op(&mut self, scratch_clears: usize, scratch_bytes: u64, output_bytes: u64) {
        self.intermediate_clears += scratch_clears;
        self.intermediate_clear_bytes = self
            .intermediate_clear_bytes
            .saturating_add(scratch_bytes.saturating_mul(scratch_clears as u64));
        self.intermediate_composites += 1;
        self.intermediate_composite_bytes = self
            .intermediate_composite_bytes
            .saturating_add(output_bytes);
        self.largest_intermediate_bytes = self.largest_intermediate_bytes.max(scratch_bytes);
    }
}

impl Renderer {
    pub(in crate::render) fn draw(&self, pass: FilterDraw<'_>) -> FilterWork {
        let Some(output_prepared) = prepare_filter(pass.filter.rect, pass.target.scale_factor)
        else {
            return FilterWork::default();
        };
        let Some(scratch_prepared) =
            prepare_filter(pass.scratch.rect, pass.scratch.target.scale_factor)
        else {
            return FilterWork::default();
        };
        let Some(textures) = self.textures.as_ref() else {
            return FilterWork::default();
        };
        let scratch_bytes = rgba_bytes(pass.scratch.target.physical_area());
        let output_bytes = rgba_bytes(
            output_prepared
                .raster_rect
                .area
                .to_physical(pass.target.scale_factor)
                .clamp_min(1),
        );
        let mut work = FilterWork::default();
        let scratch = self.scratch_targets(pass.render_context, pass.scratch.target, textures);
        {
            let mut chain = FilterChainContext::new(
                pass.target,
                pass.scratch.target,
                pass.output,
                output_prepared,
                scratch_prepared,
                pass.source,
            );

            for op in pass.filter.ops {
                let alpha_mode = filter_alpha_mode(op);
                match op {
                    paint::FilterOp::Blur { amount } => {
                        if amount <= 0.0 {
                            continue;
                        }
                        work.record_op(2, scratch_bytes, output_bytes);

                        let prepared = chain
                            .scratch_prepared()
                            .with_blur(amount, chain.scratch_target().scale_factor);
                        let sample = chain.current_sample();
                        self.blur_pass(BlurPass {
                            render_context: pass.render_context,
                            encoder: pass.encoder,
                            source: sample.texture,
                            output: scratch.ping_view(),
                            target: chain.scratch_target(),
                            prepared,
                            direction: [1.0, 0.0],
                            source_rect: sample.rect,
                            source_space: sample.space,
                            labels: BlurLabels::new(
                                "Filter Blur Horizontal Bind Group",
                                "Filter Blur Horizontal Pass",
                            ),
                        });
                        let intermediate = chain.local_intermediate(
                            scratch.ping_source(paint::LayerSampling::Filtered),
                        );
                        self.blur_pass(BlurPass {
                            render_context: pass.render_context,
                            encoder: pass.encoder,
                            source: intermediate.texture,
                            output: scratch.pong_view(),
                            target: chain.scratch_target(),
                            prepared,
                            direction: [0.0, 1.0],
                            source_rect: intermediate.rect,
                            source_space: intermediate.space,
                            labels: BlurLabels::new(
                                "Filter Blur Vertical Bind Group",
                                "Filter Blur Vertical Pass",
                            ),
                        });
                        let intermediate = chain.local_intermediate(
                            scratch.pong_source(paint::LayerSampling::Filtered),
                        );
                        self.composite_pass(CompositePass {
                            render_context: pass.render_context,
                            encoder: pass.encoder,
                            source: intermediate.texture,
                            output: chain.output(),
                            target: chain.output_target(),
                            prepared: chain.output_prepared(),
                            source_rect: intermediate.rect,
                            opacity: 1.0,
                            alpha_mode,
                            scissor: pass.scissor,
                            labels: PassLabels::new(
                                "Filter Blur Composite Bind Group",
                                "Filter Blur Composite Vertex Buffer",
                                "Filter Blur Composite Pass",
                            ),
                        });
                        chain.mark_output_as_current();
                    }
                    paint::FilterOp::BackdropBlur(blur) => {
                        if blur.sigma <= 0.0 {
                            continue;
                        }
                        work.record_op(2, scratch_bytes, output_bytes);

                        let prepared = chain
                            .scratch_prepared()
                            .with_blur_sigma(blur.sigma, chain.scratch_target().scale_factor);
                        let sample = chain.current_sample();
                        self.blur_pass(BlurPass {
                            render_context: pass.render_context,
                            encoder: pass.encoder,
                            source: sample.texture,
                            output: scratch.ping_view(),
                            target: chain.scratch_target(),
                            prepared,
                            direction: [1.0, 0.0],
                            source_rect: sample.rect,
                            source_space: sample.space,
                            labels: BlurLabels::new(
                                "Filter Backdrop Blur Horizontal Bind Group",
                                "Filter Backdrop Blur Horizontal Pass",
                            ),
                        });
                        let intermediate = chain.local_intermediate(
                            scratch.ping_source(paint::LayerSampling::Filtered),
                        );
                        self.blur_pass(BlurPass {
                            render_context: pass.render_context,
                            encoder: pass.encoder,
                            source: intermediate.texture,
                            output: scratch.pong_view(),
                            target: chain.scratch_target(),
                            prepared,
                            direction: [0.0, 1.0],
                            source_rect: intermediate.rect,
                            source_space: intermediate.space,
                            labels: BlurLabels::new(
                                "Filter Backdrop Blur Vertical Bind Group",
                                "Filter Backdrop Blur Vertical Pass",
                            ),
                        });
                        let intermediate = chain.local_intermediate(
                            scratch.pong_source(paint::LayerSampling::Filtered),
                        );
                        self.composite_pass(CompositePass {
                            render_context: pass.render_context,
                            encoder: pass.encoder,
                            source: intermediate.texture,
                            output: chain.output(),
                            target: chain.output_target(),
                            prepared: chain.output_prepared(),
                            source_rect: intermediate.rect,
                            opacity: 1.0,
                            alpha_mode,
                            scissor: pass.scissor,
                            labels: PassLabels::new(
                                "Filter Backdrop Blur Composite Bind Group",
                                "Filter Backdrop Blur Composite Vertex Buffer",
                                "Filter Backdrop Blur Composite Pass",
                            ),
                        });
                        chain.mark_output_as_current();
                    }
                    paint::FilterOp::Refraction(refraction) => {
                        if refraction.displacement <= 0.0 {
                            continue;
                        }
                        work.record_op(1, scratch_bytes, output_bytes);

                        let sample = chain.current_sample();
                        self.refraction_pass(RefractionPass {
                            render_context: pass.render_context,
                            encoder: pass.encoder,
                            source: sample.texture.view,
                            source_area: sample.texture.area,
                            source_logical_area: sample.texture.logical_area,
                            source_rect: sample.rect,
                            source_sampling: sample.texture.sampling,
                            output: scratch.ping_view(),
                            target: chain.scratch_target(),
                            prepared: chain.scratch_prepared(),
                            effect: refraction_effect(refraction),
                            alpha_mode,
                            scissor: None,
                        });
                        let intermediate = chain.local_intermediate(
                            scratch.ping_source(paint::LayerSampling::Filtered),
                        );
                        self.composite_pass(CompositePass {
                            render_context: pass.render_context,
                            encoder: pass.encoder,
                            source: intermediate.texture,
                            output: chain.output(),
                            target: chain.output_target(),
                            prepared: chain.output_prepared(),
                            source_rect: intermediate.rect,
                            opacity: 1.0,
                            alpha_mode,
                            scissor: pass.scissor,
                            labels: PassLabels::new(
                                "Filter Refraction Composite Bind Group",
                                "Filter Refraction Composite Vertex Buffer",
                                "Filter Refraction Composite Pass",
                            ),
                        });
                        chain.mark_output_as_current();
                    }
                    paint::FilterOp::Luminosity(luminosity) => {
                        if luminosity.opacity <= 0.0 {
                            continue;
                        }
                        work.record_op(1, scratch_bytes, output_bytes);

                        let sample = chain.current_sample();
                        self.effect_pass(EffectPass {
                            render_context: pass.render_context,
                            encoder: pass.encoder,
                            source: sample.texture.view,
                            source_area: sample.texture.area,
                            source_logical_area: sample.texture.logical_area,
                            source_rect: sample.rect,
                            source_sampling: sample.texture.sampling,
                            output: scratch.ping_view(),
                            target: chain.scratch_target(),
                            prepared: chain.scratch_prepared(),
                            effect: [
                                luminosity.color.r,
                                luminosity.color.g,
                                luminosity.color.b,
                                luminosity.opacity,
                            ],
                            alpha_mode,
                            pipeline: &self.luminosity_pipeline,
                            scissor: None,
                            labels: PassLabels::new(
                                "Filter Luminosity Bind Group",
                                "Filter Luminosity Vertex Buffer",
                                "Filter Luminosity Pass",
                            ),
                        });
                        let intermediate = chain.local_intermediate(
                            scratch.ping_source(paint::LayerSampling::Filtered),
                        );
                        self.composite_pass(CompositePass {
                            render_context: pass.render_context,
                            encoder: pass.encoder,
                            source: intermediate.texture,
                            output: chain.output(),
                            target: chain.output_target(),
                            prepared: chain.output_prepared(),
                            source_rect: intermediate.rect,
                            opacity: 1.0,
                            alpha_mode,
                            scissor: pass.scissor,
                            labels: PassLabels::new(
                                "Filter Luminosity Composite Bind Group",
                                "Filter Luminosity Composite Vertex Buffer",
                                "Filter Luminosity Composite Pass",
                            ),
                        });
                        chain.mark_output_as_current();
                    }
                    paint::FilterOp::Noise(noise) => {
                        if noise.opacity <= 0.0 {
                            continue;
                        }
                        work.record_op(1, scratch_bytes, output_bytes);

                        let sample = chain.current_sample();
                        self.effect_pass(EffectPass {
                            render_context: pass.render_context,
                            encoder: pass.encoder,
                            source: sample.texture.view,
                            source_area: sample.texture.area,
                            source_logical_area: sample.texture.logical_area,
                            source_rect: sample.rect,
                            source_sampling: sample.texture.sampling,
                            output: scratch.ping_view(),
                            target: chain.scratch_target(),
                            prepared: chain.scratch_prepared(),
                            effect: [noise.opacity, 0.0, 0.0, 0.0],
                            alpha_mode,
                            pipeline: &self.noise_pipeline,
                            scissor: None,
                            labels: PassLabels::new(
                                "Filter Noise Bind Group",
                                "Filter Noise Vertex Buffer",
                                "Filter Noise Pass",
                            ),
                        });
                        let intermediate = chain.local_intermediate(
                            scratch.ping_source(paint::LayerSampling::Filtered),
                        );
                        self.composite_pass(CompositePass {
                            render_context: pass.render_context,
                            encoder: pass.encoder,
                            source: intermediate.texture,
                            output: chain.output(),
                            target: chain.output_target(),
                            prepared: chain.output_prepared(),
                            source_rect: intermediate.rect,
                            opacity: 1.0,
                            alpha_mode,
                            scissor: pass.scissor,
                            labels: PassLabels::new(
                                "Filter Noise Composite Bind Group",
                                "Filter Noise Composite Vertex Buffer",
                                "Filter Noise Composite Pass",
                            ),
                        });
                        chain.mark_output_as_current();
                    }
                }
            }
        }

        if let ScratchTargets::Pooled(scratch) = scratch {
            self.recycle_scratch(scratch);
        }

        log::debug!(
            target: "wgpu_l3::render::intermediate",
            "owner={:?} output_bounds={:?} scratch_bounds={:?} clears={} clear_bytes={} composites={} composite_bytes={}",
            pass.owner,
            pass.filter.rect,
            pass.scratch.bounds,
            work.intermediate_clears,
            work.intermediate_clear_bytes,
            work.intermediate_composites,
            work.intermediate_composite_bytes,
        );

        work
    }
}

fn rgba_bytes(area: crate::geometry::area::Physical) -> u64 {
    u64::from(area.width())
        .saturating_mul(u64::from(area.height()))
        .saturating_mul(4)
}

fn filter_alpha_mode(op: paint::FilterOp) -> AlphaMode {
    match op {
        paint::FilterOp::Noise(_) => AlphaMode::Source,
        paint::FilterOp::Blur { .. }
        | paint::FilterOp::BackdropBlur(_)
        | paint::FilterOp::Refraction(_)
        | paint::FilterOp::Luminosity(_) => AlphaMode::Shape,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn noise_transforms_source_pixels_without_claiming_shape_coverage() {
        assert_eq!(
            filter_alpha_mode(paint::FilterOp::noise(paint::Noise { opacity: 0.022 })),
            AlphaMode::Source
        );
        assert_eq!(
            filter_alpha_mode(paint::FilterOp::backdrop_blur(paint::BackdropBlur {
                sigma: 44.55,
                edge_mode: paint::BackdropEdgeMode::Mirror,
            })),
            AlphaMode::Shape
        );
    }

    #[test]
    fn bounded_scratch_keeps_output_shape_local_to_the_effect_envelope() {
        let rect = paint::Rect::new(
            point::logical(40.0, 30.0),
            crate::geometry::area::logical(80.0, 50.0),
        );
        let bounds = paint::Rect::new(
            point::logical(32.0, 22.0),
            crate::geometry::area::logical(96.0, 66.0),
        );

        let scratch = FilterScratch::bounded(rect, bounds, 1.25);

        assert_eq!(scratch.bounds, bounds);
        assert_eq!(scratch.rect.origin, point::logical(8.0, 8.0));
        assert_eq!(scratch.rect.area, rect.area);
        assert_eq!(scratch.target.logical_area(), bounds.area);
        assert_eq!(
            scratch.target.physical_area(),
            crate::geometry::area::physical(120, 83)
        );
    }

    #[test]
    fn filter_work_accounts_each_scratch_clear_and_output_composite() {
        let mut work = FilterWork::default();
        work.record_op(2, 16_000, 8_000);
        work.record_op(1, 4_000, 2_000);

        assert_eq!(work.intermediate_clears, 3);
        assert_eq!(work.intermediate_clear_bytes, 36_000);
        assert_eq!(work.intermediate_composites, 2);
        assert_eq!(work.intermediate_composite_bytes, 10_000);
        assert_eq!(work.largest_intermediate_bytes, 16_000);
    }
}
