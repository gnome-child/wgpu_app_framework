use crate::geometry::Rect;
use crate::paint;
use crate::render;
use crate::render::batch::{ItemBatch, item_batches};

pub struct Renderer {
    quad_pipeline: wgpu::RenderPipeline,
    backdrop_renderer: render::backdrop::Renderer,
    text_renderer: render::text_renderer::TextRenderer,
}

enum RenderBatch {
    Shapes(render::quad::Batch),
    Backdrop(paint::Backdrop),
    Text { renderer_index: usize },
    PushClip(paint::Clip),
    PopClip,
}

#[derive(Debug, Clone, Copy, PartialEq)]
struct ClipFrame {
    clip: paint::Clip,
    layer_index: usize,
}

impl Renderer {
    pub fn new(render_context: &render::Context, format: wgpu::TextureFormat) -> Self {
        Self {
            quad_pipeline: render::quad::pipeline(render_context, format),
            backdrop_renderer: render::backdrop::Renderer::new(render_context, format),
            text_renderer: render::text_renderer::TextRenderer::new(render_context, format),
        }
    }

    pub fn clear(
        &mut self,
        render_context: &render::Context,
        canvas: &mut render::Canvas,
    ) -> render::Result<render::frame::Status> {
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
    ) -> render::Result<render::frame::Status> {
        let clear_color = scene
            .clear_color()
            .map(render::color_to_wgpu)
            .unwrap_or_else(|| canvas.color());
        let item_batches = item_batches(scene.items());
        let text_batch_count = item_batches
            .iter()
            .filter(|batch| matches!(batch, ItemBatch::Glyphs(_)))
            .count();

        if text_batch_count > 0 {
            self.text_renderer
                .prepare_frame(render_context, canvas, text_batch_count);
        }

        let mut render_batches = Vec::with_capacity(item_batches.len());
        let mut text_renderer_index = 0;

        for batch in item_batches {
            match batch {
                ItemBatch::Shapes(shapes) => {
                    if let Some(batch) =
                        render::quad::prepare_batch(render_context, canvas, &shapes)
                    {
                        render_batches.push(RenderBatch::Shapes(batch));
                    }
                }
                ItemBatch::Backdrop(backdrop) => {
                    render_batches.push(RenderBatch::Backdrop(*backdrop));
                }
                ItemBatch::PushClip(clip) => {
                    render_batches.push(RenderBatch::PushClip(*clip));
                }
                ItemBatch::PopClip => {
                    render_batches.push(RenderBatch::PopClip);
                }
                ItemBatch::Glyphs(glyphs) => {
                    if self.text_renderer.prepare_batch(
                        render_context,
                        canvas,
                        text_renderer_index,
                        &glyphs,
                    )? {
                        render_batches.push(RenderBatch::Text {
                            renderer_index: text_renderer_index,
                        });
                    }

                    text_renderer_index += 1;
                }
            }
        }

        let quad_pipeline = &self.quad_pipeline;
        let backdrop_renderer = &mut self.backdrop_renderer;
        let backdrop_target = backdrop_renderer.prepare(render_context, canvas);
        let text_renderer = &self.text_renderer;
        let mut text_render_error = None;
        let physical_area = canvas.physical_area();
        let scale_factor = canvas.scale_factor();

        let status = canvas.draw(render_context, |encoder, frame| {
            let view = frame.create_view();
            let mut layers = Vec::new();
            let mut clip_stack = Vec::new();

            backdrop_renderer.clear_composition(encoder, clear_color);

            for batch in &render_batches {
                match batch {
                    RenderBatch::Shapes(batch) => {
                        let Some(scissor) =
                            current_scissor(&clip_stack, physical_area, scale_factor)
                        else {
                            continue;
                        };
                        let Some(target_view) =
                            current_target_view(backdrop_renderer, &layers, &clip_stack)
                        else {
                            return;
                        };
                        let mut pass = begin_main_pass(encoder, target_view, clear_color, false);
                        pass.set_pipeline(quad_pipeline);
                        pass.set_scissor_rect(
                            scissor.x(),
                            scissor.y(),
                            scissor.width(),
                            scissor.height(),
                        );
                        pass.set_vertex_buffer(0, batch.vertex_buffer().slice(..));
                        pass.draw(0..batch.vertex_count(), 0..1);
                    }
                    RenderBatch::Backdrop(backdrop) => {
                        if !clip_stack.is_empty() {
                            log::debug!("skipping backdrop draw inside clipped layer");
                            continue;
                        }

                        backdrop_renderer.draw(
                            render_context,
                            backdrop_target,
                            encoder,
                            *backdrop,
                            None,
                        );
                    }
                    RenderBatch::Text { renderer_index } => {
                        let Some(scissor) =
                            current_scissor(&clip_stack, physical_area, scale_factor)
                        else {
                            continue;
                        };
                        let Some(target_view) =
                            current_target_view(backdrop_renderer, &layers, &clip_stack)
                        else {
                            return;
                        };
                        let mut pass = begin_main_pass(encoder, target_view, clear_color, false);
                        pass.set_scissor_rect(
                            scissor.x(),
                            scissor.y(),
                            scissor.width(),
                            scissor.height(),
                        );
                        if let Err(error) = text_renderer.render(*renderer_index, &mut pass) {
                            text_render_error = Some(error);
                            break;
                        }
                    }
                    RenderBatch::PushClip(clip) => {
                        let layer = backdrop_renderer.create_layer(
                            render_context,
                            backdrop_target,
                            "Clip Layer Texture",
                        );
                        layers.push(layer);
                        let layer_index = layers.len() - 1;
                        backdrop_renderer.clear_layer(encoder, &layers[layer_index]);
                        clip_stack.push(ClipFrame {
                            clip: *clip,
                            layer_index,
                        });
                    }
                    RenderBatch::PopClip => {
                        composite_clip_layer(
                            render_context,
                            backdrop_renderer,
                            backdrop_target,
                            encoder,
                            physical_area,
                            scale_factor,
                            &layers,
                            &mut clip_stack,
                        );
                    }
                }
            }

            while !clip_stack.is_empty() {
                composite_clip_layer(
                    render_context,
                    backdrop_renderer,
                    backdrop_target,
                    encoder,
                    physical_area,
                    scale_factor,
                    &layers,
                    &mut clip_stack,
                );
            }

            backdrop_renderer.blit_to_view(render_context, encoder, &view, backdrop_target);
        });

        if let Some(error) = text_render_error {
            return Err(error.into());
        }

        self.text_renderer.trim();

        status
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

fn composite_clip_layer(
    render_context: &render::Context,
    backdrop_renderer: &render::backdrop::Renderer,
    target: render::backdrop::Target,
    encoder: &mut wgpu::CommandEncoder,
    physical_area: crate::geometry::area::Physical,
    scale_factor: f32,
    layers: &[render::backdrop::Layer],
    clip_stack: &mut Vec<ClipFrame>,
) {
    let Some(frame) = clip_stack.pop() else {
        return;
    };
    let Some(source) = layers.get(frame.layer_index) else {
        return;
    };
    let Some(output) = current_target_view(backdrop_renderer, layers, clip_stack) else {
        return;
    };

    backdrop_renderer.composite_layer(
        render_context,
        encoder,
        source,
        output,
        target,
        frame.clip,
        clip_scissor(frame.clip.rect, physical_area, scale_factor),
    );
}

fn current_target_view<'a>(
    backdrop_renderer: &'a render::backdrop::Renderer,
    layers: &'a [render::backdrop::Layer],
    clip_stack: &[ClipFrame],
) -> Option<&'a wgpu::TextureView> {
    if let Some(frame) = clip_stack.last() {
        Some(layers.get(frame.layer_index)?.view())
    } else {
        backdrop_renderer.composition_view()
    }
}

fn current_scissor(
    clips: &[ClipFrame],
    physical_area: crate::geometry::area::Physical,
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
    physical_area: crate::geometry::area::Physical,
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
    use crate::geometry::{Rect, area, point, rect};
    use crate::paint;

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
}
