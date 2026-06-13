use crate::paint;
use crate::render;
use wgpu::util::DeviceExt;

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
        scene: &paint::Scene,
    ) -> render::Result<render::frame::Status> {
        let mut vertex_buf = Vec::new();
        for quad in scene.quads() {
            push_quad_vertices(&mut vertex_buf, canvas, quad);
        }

        let vertex_count = vertex_buf.len() as u32;
        let vertex_buffer =
            if vertex_buf.is_empty() {
                None
            } else {
                Some(render_context.device().create_buffer_init(
                    &wgpu::util::BufferInitDescriptor {
                        label: Some("Quad Vertex Buffer"),
                        contents: bytemuck::cast_slice(&vertex_buf),
                        usage: wgpu::BufferUsages::VERTEX,
                    },
                ))
            };

        let clear_color = scene
            .clear_color()
            .map(render::color_to_wgpu)
            .unwrap_or_else(|| canvas.color());
        let quad_pipeline = &self.quad_pipeline;

        canvas.draw(render_context, move |encoder, frame| {
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

            if let Some(vertex_buffer) = vertex_buffer.as_ref() {
                pass.set_pipeline(quad_pipeline);
                pass.set_vertex_buffer(0, vertex_buffer.slice(..));
                pass.draw(0..vertex_count, 0..1);
            }
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
        log::debug!("skipping unsupported non-brush quad fill");
        return;
    };

    let paint::Brush::Solid(color) = brush else {
        log::debug!("skipping unsupported non-solid quad brush");
        return;
    };

    let rect = quad.rect;

    if quad.style.stroke.is_some() {
        log::debug!("skipping unsupported quad stroke");
    }

    if quad.style.tint.is_some() {
        log::debug!("skipping unsupported quad tint");
    }

    if rect.radius != crate::geometry::rect::Radius::none() {
        log::debug!("skipping unsupported quad radius");
    }

    let origin = rect.origin;
    let area = rect.area;
    let canvas_area = canvas.logical_area();

    if canvas_area.width() <= 0.0 || canvas_area.height() <= 0.0 {
        log::debug!("skipping quad draw for zero-size canvas");
        return;
    }

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
