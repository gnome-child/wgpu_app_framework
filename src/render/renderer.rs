use crate::paint;
use crate::paint_geometry::Rect;
use crate::render;
use crate::render::batch::{ItemBatch, item_batches};

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct DrawStats {
    pub scene_items: usize,
    pub render_batches: usize,
    pub glyph_batches: usize,
    pub text_surfaces: usize,
    pub inline_text_cache_hits: usize,
    pub inline_text_cache_misses: usize,
    pub inline_text_shape_calls: usize,
    pub inline_icon_cache_hits: usize,
    pub inline_icon_cache_misses: usize,
    pub inline_icon_shape_calls: usize,
    pub quad_vertices: usize,
    pub clip_batches: usize,
    pub filters: usize,
}

pub struct Renderer {
    quad_pipeline: wgpu::RenderPipeline,
    filter_renderer: render::filter::Renderer,
    text_renderer: render::text_renderer::TextRenderer,
}

enum RenderBatch {
    Shapes(render::quad::Batch),
    Filter(paint::Filter),
    Text {
        renderer_index: usize,
        viewport: render::Viewport,
    },
    PushClip(paint::Clip),
    PopClip,
}

struct PreparedScene {
    render_batches: Vec<RenderBatch>,
    stats: DrawStats,
}

#[derive(Debug, Clone, Copy, PartialEq)]
struct ClipFrame {
    clip: paint::Clip,
    layer_index: usize,
}

struct SceneEncoder<'a> {
    render_context: &'a render::Context,
    quad_pipeline: &'a wgpu::RenderPipeline,
    filter_renderer: &'a render::filter::Renderer,
    text_renderer: &'a mut render::text_renderer::TextRenderer,
    filter_target: render::filter::Target,
    output_target: render::filter::Target,
    encoder: &'a mut wgpu::CommandEncoder,
    base_view: &'a wgpu::TextureView,
    clear_color: wgpu::Color,
    viewport: render::Viewport,
    layers: Vec<render::filter::Layer>,
    clip_stack: Vec<ClipFrame>,
    text_render_error: &'a mut Option<render::text_renderer::Error>,
}

struct SceneEncoderInput<'a> {
    render_context: &'a render::Context,
    quad_pipeline: &'a wgpu::RenderPipeline,
    filter_renderer: &'a render::filter::Renderer,
    text_renderer: &'a mut render::text_renderer::TextRenderer,
    filter_target: render::filter::Target,
    encoder: &'a mut wgpu::CommandEncoder,
    base_view: &'a wgpu::TextureView,
    clear_color: wgpu::Color,
    viewport: render::Viewport,
    text_render_error: &'a mut Option<render::text_renderer::Error>,
}

impl Renderer {
    pub fn new(render_context: &render::Context, format: wgpu::TextureFormat) -> Self {
        Self {
            quad_pipeline: render::quad::pipeline(render_context, format),
            filter_renderer: render::filter::Renderer::new(render_context, format),
            text_renderer: render::text_renderer::TextRenderer::new(render_context, format),
        }
    }

    pub fn clear(
        &mut self,
        render_context: &render::Context,
        canvas: &mut render::Canvas,
    ) -> render::Result<()> {
        let clear_color = canvas.color();

        canvas.draw(render_context, |encoder, frame| {
            let view = frame.create_view();

            let _pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Main Render Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    depth_slice: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(clear_color),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                occlusion_query_set: None,
                timestamp_writes: None,
                multiview_mask: None,
            });
        })
    }

    pub fn draw(
        &mut self,
        render_context: &render::Context,
        canvas: &mut render::Canvas,
        scene: &paint::Scene,
    ) -> render::Result<DrawStats> {
        let clear_color = scene
            .clear_color()
            .map(render::color_to_wgpu)
            .unwrap_or_else(|| canvas.color());
        let main_viewport = render::Viewport::from_canvas(canvas);
        let item_batches = item_batches(scene.items());
        let scene_text_batches = item_batches
            .iter()
            .filter(|batch| matches!(batch, ItemBatch::Glyphs(_)))
            .count();
        let text_batch_count = scene_text_batches;

        if text_batch_count > 0 {
            self.text_renderer
                .prepare_frame(render_context, text_batch_count);
        }

        let mut text_renderer_index = 0;
        let mut stats = DrawStats::default();
        let prepared_scene = self.prepare_scene_batches(
            render_context,
            main_viewport,
            &item_batches,
            &mut text_renderer_index,
        )?;
        stats.add(prepared_scene.stats);
        stats.scene_items = scene.items().len();

        let filter_target = self.filter_renderer.prepare(render_context, canvas);
        let quad_pipeline = &self.quad_pipeline;
        let filter_renderer = &self.filter_renderer;
        let text_renderer = &mut self.text_renderer;
        let mut text_render_error = None;

        canvas.draw(render_context, |encoder, frame| {
            let view = frame.create_view();
            filter_renderer.clear_composition(encoder, clear_color);
            let Some(composition_view) = filter_renderer.composition_view() else {
                return;
            };
            let mut scene_encoder = SceneEncoder::new(SceneEncoderInput {
                render_context,
                quad_pipeline,
                filter_renderer,
                text_renderer,
                filter_target,
                encoder,
                base_view: composition_view,
                clear_color,
                viewport: main_viewport,
                text_render_error: &mut text_render_error,
            });
            scene_encoder.encode(&prepared_scene.render_batches);
            if text_render_error.is_some() {
                return;
            }

            filter_renderer.blit_to_view(render_context, encoder, &view, filter_target);
        })?;

        if let Some(error) = text_render_error {
            return Err(error.into());
        }

        self.text_renderer.trim();

        Ok(stats)
    }

    fn prepare_scene_batches(
        &mut self,
        render_context: &render::Context,
        viewport: render::Viewport,
        item_batches: &[ItemBatch<'_>],
        text_renderer_index: &mut usize,
    ) -> render::Result<PreparedScene> {
        let mut render_batches = Vec::with_capacity(item_batches.len());
        let mut stats = DrawStats {
            render_batches: item_batches.len(),
            glyph_batches: item_batches
                .iter()
                .filter(|batch| matches!(batch, ItemBatch::Glyphs(_)))
                .count(),
            text_surfaces: item_batches
                .iter()
                .filter_map(|batch| match batch {
                    ItemBatch::Glyphs(glyphs) => Some(
                        glyphs
                            .iter()
                            .map(|glyph| match glyph {
                                crate::render::batch::Glyph::TextViewport(viewport) => {
                                    viewport.surfaces.len()
                                }
                                _ => 0,
                            })
                            .sum::<usize>(),
                    ),
                    _ => None,
                })
                .sum(),
            clip_batches: item_batches
                .iter()
                .filter(|batch| matches!(batch, ItemBatch::PushClip(_) | ItemBatch::PopClip))
                .count(),
            filters: item_batches
                .iter()
                .filter(|batch| matches!(batch, ItemBatch::Filter(_)))
                .count(),
            ..DrawStats::default()
        };

        for batch in item_batches {
            match batch {
                ItemBatch::Shapes(shapes) => {
                    if let Some(batch) =
                        render::quad::prepare_batch(render_context, viewport, shapes)
                    {
                        stats.quad_vertices += batch.vertex_count() as usize;
                        render_batches.push(RenderBatch::Shapes(batch));
                    }
                }
                ItemBatch::Filter(filter) => {
                    render_batches.push(RenderBatch::Filter((*filter).clone()));
                }
                ItemBatch::PushClip(clip) => {
                    render_batches.push(RenderBatch::PushClip(**clip));
                }
                ItemBatch::PopClip => {
                    render_batches.push(RenderBatch::PopClip);
                }
                ItemBatch::Glyphs(glyphs) => {
                    let report = self.text_renderer.prepare_batch(
                        render_context,
                        viewport,
                        *text_renderer_index,
                        glyphs,
                    )?;
                    stats.inline_text_cache_hits += report.stats.text_cache_hits;
                    stats.inline_text_cache_misses += report.stats.text_cache_misses;
                    stats.inline_text_shape_calls += report.stats.text_shape_calls;
                    stats.inline_icon_cache_hits += report.stats.icon_cache_hits;
                    stats.inline_icon_cache_misses += report.stats.icon_cache_misses;
                    stats.inline_icon_shape_calls += report.stats.icon_shape_calls;
                    if report.has_text {
                        render_batches.push(RenderBatch::Text {
                            renderer_index: *text_renderer_index,
                            viewport,
                        });
                    }

                    *text_renderer_index += 1;
                }
            }
        }
        stats.render_batches = render_batches.len();

        Ok(PreparedScene {
            render_batches,
            stats,
        })
    }
}

impl DrawStats {
    #[cfg(test)]
    fn from_scene(scene: &paint::Scene, batches: &[ItemBatch<'_>]) -> Self {
        Self {
            scene_items: scene.items().len(),
            render_batches: batches.len(),
            glyph_batches: batches
                .iter()
                .filter(|batch| matches!(batch, ItemBatch::Glyphs(_)))
                .count(),
            text_surfaces: scene
                .items()
                .iter()
                .map(|item| match item {
                    paint::Item::TextViewport(viewport) => viewport.surfaces.len(),
                    _ => 0,
                })
                .sum(),
            quad_vertices: 0,
            clip_batches: scene
                .items()
                .iter()
                .filter(|item| matches!(item, paint::Item::Clip(_) | paint::Item::PopClip))
                .count(),
            filters: scene
                .items()
                .iter()
                .filter(|item| matches!(item, paint::Item::Filter(_)))
                .count(),
            ..Self::default()
        }
    }

    fn add(&mut self, other: Self) {
        self.scene_items += other.scene_items;
        self.render_batches += other.render_batches;
        self.glyph_batches += other.glyph_batches;
        self.text_surfaces += other.text_surfaces;
        self.inline_text_cache_hits += other.inline_text_cache_hits;
        self.inline_text_cache_misses += other.inline_text_cache_misses;
        self.inline_text_shape_calls += other.inline_text_shape_calls;
        self.inline_icon_cache_hits += other.inline_icon_cache_hits;
        self.inline_icon_cache_misses += other.inline_icon_cache_misses;
        self.inline_icon_shape_calls += other.inline_icon_shape_calls;
        self.quad_vertices += other.quad_vertices;
        self.clip_batches += other.clip_batches;
        self.filters += other.filters;
    }
}

fn begin_main_pass<'a>(
    encoder: &'a mut wgpu::CommandEncoder,
    view: &'a wgpu::TextureView,
    clear_color: wgpu::Color,
    clear: bool,
) -> wgpu::RenderPass<'a> {
    encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
        label: Some("Main Render Pass"),
        color_attachments: &[Some(wgpu::RenderPassColorAttachment {
            view,
            resolve_target: None,
            depth_slice: None,
            ops: wgpu::Operations {
                load: if clear {
                    wgpu::LoadOp::Clear(clear_color)
                } else {
                    wgpu::LoadOp::Load
                },
                store: wgpu::StoreOp::Store,
            },
        })],
        depth_stencil_attachment: None,
        occlusion_query_set: None,
        timestamp_writes: None,
        multiview_mask: None,
    })
}

impl<'a> SceneEncoder<'a> {
    fn new(input: SceneEncoderInput<'a>) -> Self {
        Self {
            render_context: input.render_context,
            quad_pipeline: input.quad_pipeline,
            filter_renderer: input.filter_renderer,
            text_renderer: input.text_renderer,
            filter_target: input.filter_target,
            output_target: render::filter::Target::from_viewport(input.viewport),
            encoder: input.encoder,
            base_view: input.base_view,
            clear_color: input.clear_color,
            viewport: input.viewport,
            layers: Vec::new(),
            clip_stack: Vec::new(),
            text_render_error: input.text_render_error,
        }
    }

    fn encode(&mut self, render_batches: &[RenderBatch]) {
        for batch in render_batches {
            match batch {
                RenderBatch::Shapes(batch) => self.encode_shapes(batch),
                RenderBatch::Filter(filter) => self.encode_filter(filter),
                RenderBatch::Text {
                    renderer_index,
                    viewport,
                } => self.encode_text(*renderer_index, *viewport),
                RenderBatch::PushClip(clip) => self.push_clip(*clip),
                RenderBatch::PopClip => self.composite_clip_layer(),
            }
        }

        while !self.clip_stack.is_empty() {
            self.composite_clip_layer();
        }
    }

    fn encode_shapes(&mut self, batch: &render::quad::Batch) {
        let Some(scissor) = current_scissor(
            &self.clip_stack,
            self.viewport.physical_area(),
            self.viewport.scale_factor(),
        ) else {
            return;
        };
        let Some(target_view) = current_target_view(self.base_view, &self.layers, &self.clip_stack)
        else {
            return;
        };
        let mut pass = begin_main_pass(self.encoder, target_view, self.clear_color, false);
        pass.set_pipeline(self.quad_pipeline);
        pass.set_scissor_rect(scissor.x(), scissor.y(), scissor.width(), scissor.height());
        pass.set_vertex_buffer(0, batch.vertex_buffer().slice(..));
        pass.draw(0..batch.vertex_count(), 0..1);
    }

    fn encode_filter(&mut self, filter: &paint::Filter) {
        if !self.clip_stack.is_empty() {
            log::debug!("skipping filter draw inside clipped layer");
            return;
        }

        self.filter_renderer.draw(
            self.render_context,
            self.filter_target,
            self.encoder,
            filter.clone(),
            None,
        );
    }

    fn encode_text(&mut self, renderer_index: usize, text_viewport: render::Viewport) {
        let Some(scissor) = current_scissor(
            &self.clip_stack,
            self.viewport.physical_area(),
            self.viewport.scale_factor(),
        ) else {
            return;
        };
        let Some(target_view) = current_target_view(self.base_view, &self.layers, &self.clip_stack)
        else {
            return;
        };
        let mut pass = begin_main_pass(self.encoder, target_view, self.clear_color, false);
        pass.set_scissor_rect(scissor.x(), scissor.y(), scissor.width(), scissor.height());
        if let Err(error) = self.text_renderer.render(
            self.render_context,
            text_viewport,
            renderer_index,
            &mut pass,
        ) {
            *self.text_render_error = Some(error);
        }
    }

    fn push_clip(&mut self, clip: paint::Clip) {
        let layer = self.filter_renderer.create_layer(
            self.render_context,
            self.output_target,
            "Clip Layer Texture",
        );
        self.layers.push(layer);
        let layer_index = self.layers.len() - 1;
        self.filter_renderer
            .clear_layer(self.encoder, &self.layers[layer_index]);
        self.clip_stack.push(ClipFrame { clip, layer_index });
    }

    fn composite_clip_layer(&mut self) {
        let Some(frame) = self.clip_stack.pop() else {
            return;
        };
        let Some(source) = self.layers.get(frame.layer_index) else {
            return;
        };
        let Some(output) = current_target_view(self.base_view, &self.layers, &self.clip_stack)
        else {
            return;
        };

        self.filter_renderer.composite_layer(
            self.render_context,
            self.encoder,
            source,
            output,
            self.output_target,
            frame.clip,
            clip_scissor(
                frame.clip.rect,
                self.viewport.physical_area(),
                self.viewport.scale_factor(),
            ),
        );
    }
}

fn current_target_view<'a>(
    base_view: &'a wgpu::TextureView,
    layers: &'a [render::filter::Layer],
    clip_stack: &[ClipFrame],
) -> Option<&'a wgpu::TextureView> {
    if let Some(frame) = clip_stack.last() {
        Some(layers.get(frame.layer_index)?.view())
    } else {
        Some(base_view)
    }
}

fn current_scissor(
    clips: &[ClipFrame],
    physical_area: crate::paint_geometry::area::Physical,
    scale_factor: f32,
) -> Option<render::Scissor> {
    let mut scissor = render::Scissor::new(
        0,
        0,
        physical_area.width().max(1),
        physical_area.height().max(1),
    )?;

    for clip in clips {
        scissor = intersect_scissor(
            scissor,
            clip_scissor(clip.clip.rect, physical_area, scale_factor)?,
        )?;
    }

    Some(scissor)
}

fn clip_scissor(
    rect: Rect,
    physical_area: crate::paint_geometry::area::Physical,
    scale_factor: f32,
) -> Option<render::Scissor> {
    let scale_factor = scale_factor.max(0.0001);
    let canvas_width = physical_area.width();
    let canvas_height = physical_area.height();
    let left = (rect.origin.x() * scale_factor).floor().max(0.0) as u32;
    let top = (rect.origin.y() * scale_factor).floor().max(0.0) as u32;
    let right = ((rect.origin.x() + rect.area.width()) * scale_factor)
        .ceil()
        .max(0.0)
        .min(canvas_width as f32) as u32;
    let bottom = ((rect.origin.y() + rect.area.height()) * scale_factor)
        .ceil()
        .max(0.0)
        .min(canvas_height as f32) as u32;
    let left = left.min(canvas_width);
    let top = top.min(canvas_height);

    render::Scissor::new(
        left,
        top,
        right.saturating_sub(left),
        bottom.saturating_sub(top),
    )
}

fn intersect_scissor(a: render::Scissor, b: render::Scissor) -> Option<render::Scissor> {
    let left = a.x().max(b.x());
    let top = a.y().max(b.y());
    let right = (a.x() + a.width()).min(b.x() + b.width());
    let bottom = (a.y() + a.height()).min(b.y() + b.height());

    render::Scissor::new(
        left,
        top,
        right.saturating_sub(left),
        bottom.saturating_sub(top),
    )
}

#[cfg(test)]
mod tests {
    use crate::paint_geometry::{Rect, area, point, rect};
    use crate::{paint, text};

    use super::*;

    #[test]
    fn clip_scissor_converts_logical_rect_to_physical_pixels() {
        let scissor = clip_scissor(
            Rect::new(point::logical(2.25, 3.25), area::logical(10.5, 8.5)),
            area::physical(100, 80),
            2.0,
        )
        .expect("clip should produce scissor");

        assert_eq!(scissor.x(), 4);
        assert_eq!(scissor.y(), 6);
        assert_eq!(scissor.width(), 22);
        assert_eq!(scissor.height(), 18);
    }

    #[test]
    fn current_scissor_intersects_nested_clip_stack() {
        let clips = vec![
            ClipFrame {
                layer_index: 0,
                clip: paint::Clip {
                    rect: Rect::rounded(
                        point::logical(2.0, 2.0),
                        area::logical(20.0, 20.0),
                        rect::Rounding::relative(0.5),
                    ),
                },
            },
            ClipFrame {
                layer_index: 1,
                clip: paint::Clip {
                    rect: Rect::new(point::logical(10.0, 0.0), area::logical(20.0, 12.0)),
                },
            },
        ];
        let scissor = current_scissor(&clips, area::physical(100, 80), 1.0)
            .expect("nested clips should intersect");

        assert_eq!(scissor.x(), 10);
        assert_eq!(scissor.y(), 2);
        assert_eq!(scissor.width(), 12);
        assert_eq!(scissor.height(), 10);
    }

    #[test]
    fn clip_frames_track_layer_stack_in_lifo_order() {
        let mut clips = vec![
            ClipFrame {
                layer_index: 0,
                clip: paint::Clip {
                    rect: Rect::rounded(
                        point::logical(2.0, 2.0),
                        area::logical(20.0, 20.0),
                        rect::Rounding::relative(0.5),
                    ),
                },
            },
            ClipFrame {
                layer_index: 1,
                clip: paint::Clip {
                    rect: Rect::new(point::logical(10.0, 0.0), area::logical(20.0, 12.0)),
                },
            },
        ];

        assert_eq!(clips.last().map(|frame| frame.layer_index), Some(1));
        assert_eq!(clips.pop().map(|frame| frame.layer_index), Some(1));
        assert_eq!(clips.last().map(|frame| frame.layer_index), Some(0));
    }

    #[test]
    fn clip_scissor_keeps_rounded_clip_metadata_as_optimization_only() {
        let clip = paint::Clip {
            rect: Rect::rounded(
                point::logical(4.0, 4.0),
                area::logical(18.0, 12.0),
                rect::Rounding::relative(1.0),
            ),
        };
        let scissor = clip_scissor(clip.rect, area::physical(100, 80), 1.0)
            .expect("clip should produce scissor");

        assert_eq!(clip.rect.rounding.resolve(clip.rect.area)[0], 6.0);
        assert_eq!(scissor.x(), 4);
        assert_eq!(scissor.y(), 4);
        assert_eq!(scissor.width(), 18);
        assert_eq!(scissor.height(), 12);
    }

    #[test]
    fn draw_stats_summarize_scene_and_batches() {
        let mut scene = paint::Scene::new();
        scene.push_quad(paint::Quad {
            rect: Rect::new(point::logical(0.0, 0.0), area::logical(10.0, 10.0)),
            rasterization: paint::Rasterization::default(),
            transform: paint::Transform::identity(),
            style: paint::Style {
                fill: Some(paint::Fill::Brush(paint::Brush::solid(paint::Color::BLACK))),
                stroke: None,
                tint: None,
            },
        });
        scene.push_text(paint::Text {
            rect: Rect::new(point::logical(0.0, 0.0), area::logical(10.0, 10.0)),
            document: text::document::Document::plain("hello"),
            wrap: paint::TextWrap::WordOrGlyph,
            vertical_align: paint::TextVerticalAlign::Center,
        });
        scene.push_icon(paint::Icon {
            rect: Rect::new(point::logical(0.0, 0.0), area::logical(10.0, 10.0)),
            icon: crate::icon::Icon::phosphor(crate::icon::Id::new("check")),
            color: paint::Color::BLACK,
            size: 16.0,
        });
        scene.push_filter(paint::Filter::blur(
            Rect::new(point::logical(0.0, 0.0), area::logical(10.0, 10.0)),
            0.5,
        ));
        scene.push_clip(paint::Clip {
            rect: Rect::new(point::logical(0.0, 0.0), area::logical(10.0, 10.0)),
        });
        scene.pop_clip();

        let batches = item_batches(scene.items());
        let stats = DrawStats::from_scene(&scene, &batches);

        assert_eq!(stats.scene_items, 6);
        assert_eq!(stats.render_batches, 5);
        assert_eq!(stats.glyph_batches, 1);
        assert_eq!(stats.text_surfaces, 0);
        assert_eq!(stats.clip_batches, 2);
        assert_eq!(stats.filters, 1);
    }
}
