use crate::paint;
use crate::render;

pub struct Renderer {
    quad_pipeline: wgpu::RenderPipeline,
}

impl Renderer {
    pub fn new(render_context: &render::Context, format: wgpu::TextureFormat) -> Self {
        let shader = render_context
            .device()
            .create_shader_module(wgpu::include_wgsl!("quad.wgsl"));

        let pipeline_layout =
            render_context
                .device()
                .create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                    label: Some("Quad Pipeline Layout"),
                    bind_group_layouts: &[],
                    immediate_size: 0,
                });

        let quad_pipeline =
            render_context
                .device()
                .create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                    label: Some("Quad Pipeline"),
                    layout: Some(&pipeline_layout),
                    vertex: wgpu::VertexState {
                        module: &shader,
                        entry_point: Some("vs_main"),
                        buffers: &[render::primitive::Vertex::layout()],
                        compilation_options: Default::default(),
                    },
                    primitive: wgpu::PrimitiveState::default(),
                    depth_stencil: None,
                    multisample: wgpu::MultisampleState::default(),
                    fragment: Some(wgpu::FragmentState {
                        module: &shader,
                        entry_point: Some("fs_main"),
                        targets: &[Some(wgpu::ColorTargetState {
                            format,
                            blend: Some(wgpu::BlendState::ALPHA_BLENDING),
                            write_mask: wgpu::ColorWrites::ALL,
                        })],
                        compilation_options: Default::default(),
                    }),
                    multiview_mask: None,
                    cache: None,
                });

        Self { quad_pipeline }
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
        quads: &[paint::Quad],
    ) -> render::Result<render::frame::Status> {
        let mut vertex_buf = Vec::new();
        for quad in quads {
            push_quad_vertices(&mut vertex_buf, canvas, quad);
        }

        log::info!("got {} vertices", vertex_buf.len());

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
}

fn push_quad_vertices(
    buffer: &mut Vec<render::primitive::Vertex>,
    canvas: &render::Canvas,
    quad: &paint::Quad,
) {
    let Some(fill) = quad.style.fill else {
        return;
    };

    let paint::Fill::Brush(brush) = fill else {
        return;
    };

    let paint::Brush::Solid(color) = brush else {
        return;
    };

    let rect = quad.rect;
    let origin = rect.origin;
    let area = rect.area;
    let canvas_area = canvas.logical_area();

    let x0 = origin.x();
    let y0 = origin.y();
    let x1 = x0 + area.width();
    let y1 = y0 + area.height();

    let to_clip = |x: f32, y: f32| -> [f32; 2] {
        [
            (x / canvas_area.width()) * 2.0 - 1.0,
            1.0 - (y / canvas_area.height()) * 2.0,
        ]
    };

    let color = color.to_array();

    buffer.extend_from_slice(&[
        render::primitive::Vertex {
            position: to_clip(x0, y0),
            color,
        },
        render::primitive::Vertex {
            position: to_clip(x0, y1),
            color,
        },
        render::primitive::Vertex {
            position: to_clip(x1, y1),
            color,
        },
        render::primitive::Vertex {
            position: to_clip(x0, y0),
            color,
        },
        render::primitive::Vertex {
            position: to_clip(x1, y1),
            color,
        },
        render::primitive::Vertex {
            position: to_clip(x1, y0),
            color,
        },
    ]);
}
