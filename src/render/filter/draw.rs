use crate::paint;
use crate::render;

use super::chain::{FilterChainContext, FilterSource};
use super::effects::refraction_effect;
use super::geometry::prepare_filter;
use super::params::AlphaMode;
use super::pass::{BlurLabels, BlurPass, CompositePass, EffectPass, LiquidPass, PassLabels};
use super::state::Renderer;
use super::storage::ScratchTargets;
use super::target::Target;

pub(crate) struct FilterDraw<'a> {
    pub(crate) render_context: &'a render::Context,
    pub(crate) encoder: &'a mut wgpu::CommandEncoder,
    pub(crate) target: Target,
    pub(crate) source: FilterSource<'a>,
    pub(crate) output: &'a wgpu::TextureView,
    pub(crate) filter: paint::Filter,
    pub(crate) scissor: Option<render::Scissor>,
}

impl Renderer {
    pub(crate) fn draw(&self, pass: FilterDraw<'_>) {
        let Some(prepared) = prepare_filter(pass.filter.rect, pass.target.scale_factor) else {
            return;
        };
        let Some(textures) = self.textures.as_ref() else {
            return;
        };
        let scratch = self.scratch_targets(pass.render_context, pass.target, textures);
        {
            let mut chain =
                FilterChainContext::new(pass.target, pass.output, prepared, pass.source);

            for op in pass.filter.ops {
                match op {
                    paint::FilterOp::Blur { amount } => {
                        if amount <= 0.0 {
                            continue;
                        }

                        let prepared = chain
                            .base_prepared()
                            .with_blur(amount, chain.target().scale_factor);
                        let sample = chain.current_sample();
                        self.blur_pass(BlurPass {
                            render_context: pass.render_context,
                            encoder: pass.encoder,
                            source: sample.texture,
                            output: scratch.ping_view(),
                            target: chain.target(),
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
                            target: chain.target(),
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
                            target: chain.target(),
                            prepared,
                            source_rect: intermediate.rect,
                            opacity: 1.0,
                            alpha_mode: AlphaMode::Shape,
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

                        let prepared = chain
                            .base_prepared()
                            .with_blur_sigma(blur.sigma, chain.target().scale_factor);
                        let sample = chain.current_sample();
                        self.blur_pass(BlurPass {
                            render_context: pass.render_context,
                            encoder: pass.encoder,
                            source: sample.texture,
                            output: scratch.ping_view(),
                            target: chain.target(),
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
                            target: chain.target(),
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
                            target: chain.target(),
                            prepared,
                            source_rect: intermediate.rect,
                            opacity: 1.0,
                            alpha_mode: AlphaMode::Shape,
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

                        let sample = chain.current_sample();
                        self.liquid_pass(LiquidPass {
                            render_context: pass.render_context,
                            encoder: pass.encoder,
                            source: sample.texture.view,
                            source_area: sample.texture.area,
                            source_logical_area: sample.texture.logical_area,
                            source_rect: sample.rect,
                            source_sampling: sample.texture.sampling,
                            output: scratch.ping_view(),
                            target: chain.target(),
                            prepared: chain.base_prepared(),
                            effect: refraction_effect(refraction),
                            alpha_mode: AlphaMode::Shape,
                            scissor: pass.scissor,
                        });
                        let intermediate = chain.local_intermediate(
                            scratch.ping_source(paint::LayerSampling::Filtered),
                        );
                        self.composite_pass(CompositePass {
                            render_context: pass.render_context,
                            encoder: pass.encoder,
                            source: intermediate.texture,
                            output: chain.output(),
                            target: chain.target(),
                            prepared: chain.base_prepared(),
                            source_rect: intermediate.rect,
                            opacity: 1.0,
                            alpha_mode: AlphaMode::Shape,
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
                            target: chain.target(),
                            prepared: chain.base_prepared(),
                            effect: [
                                luminosity.color.r,
                                luminosity.color.g,
                                luminosity.color.b,
                                luminosity.opacity,
                            ],
                            alpha_mode: AlphaMode::Shape,
                            pipeline: &self.luminosity_pipeline,
                            scissor: pass.scissor,
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
                            target: chain.target(),
                            prepared: chain.base_prepared(),
                            source_rect: intermediate.rect,
                            opacity: 1.0,
                            alpha_mode: AlphaMode::Shape,
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
                            target: chain.target(),
                            prepared: chain.base_prepared(),
                            effect: [noise.opacity, 0.0, 0.0, 0.0],
                            alpha_mode: AlphaMode::Shape,
                            pipeline: &self.noise_pipeline,
                            scissor: pass.scissor,
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
                            target: chain.target(),
                            prepared: chain.base_prepared(),
                            source_rect: intermediate.rect,
                            opacity: 1.0,
                            alpha_mode: AlphaMode::Shape,
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
    }
}
