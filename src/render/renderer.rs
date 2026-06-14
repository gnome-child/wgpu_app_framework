use crate::paint;
use crate::render;
use crate::render::batch::{ItemBatch, item_batches};

pub struct Renderer {
    quad_pipeline: wgpu::RenderPipeline,
    text_renderer: render::text_renderer::TextRenderer,
}

enum RenderBatch {
    Shapes(render::quad::Batch),
    Text { renderer_index: usize },
}

impl Renderer {
    pub fn new(render_context: &render::Context, format: wgpu::TextureFormat) -> Self {
        Self {
            quad_pipeline: render::quad::pipeline(render_context, format),
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
        let text_renderer = &self.text_renderer;
        let mut text_render_error = None;

        let status = canvas.draw(render_context, |encoder, frame| {
            let view = frame.create_view();

            let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
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

            for batch in &render_batches {
                match batch {
                    RenderBatch::Shapes(batch) => {
                        pass.set_pipeline(quad_pipeline);
                        pass.set_vertex_buffer(0, batch.vertex_buffer().slice(..));
                        pass.draw(0..batch.vertex_count(), 0..1);
                    }
                    RenderBatch::Text { renderer_index } => {
                        if let Err(error) = text_renderer.render(*renderer_index, &mut pass) {
                            text_render_error = Some(error);
                            break;
                        }
                    }
                }
            }
        });

        if let Some(error) = text_render_error {
            return Err(error.into());
        }

        self.text_renderer.trim();

        status
    }
}
