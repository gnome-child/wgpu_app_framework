use wgpu::util::DeviceExt;

use crate::paint;
use crate::render;

pub struct Batch {
    vertex_buffer: wgpu::Buffer,
    vertex_count: u32,
}

impl Batch {
    pub fn vertex_buffer(&self) -> &wgpu::Buffer {
        &self.vertex_buffer
    }

    pub fn vertex_count(&self) -> u32 {
        self.vertex_count
    }
}

pub fn pipeline(
    render_context: &render::Context,
    format: wgpu::TextureFormat,
) -> wgpu::RenderPipeline {
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
        })
}

pub fn prepare_batch(
    render_context: &render::Context,
    canvas: &render::Canvas,
    quads: &[&paint::Quad],
) -> Option<Batch> {
    let mut vertex_buf = Vec::new();
    for quad in quads {
        push_vertices(&mut vertex_buf, canvas, quad);
    }

    let vertex_count = vertex_buf.len() as u32;
    if vertex_count == 0 {
        return None;
    }

    let vertex_buffer =
        render_context
            .device()
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("Quad Vertex Buffer"),
                contents: bytemuck::cast_slice(&vertex_buf),
                usage: wgpu::BufferUsages::VERTEX,
            });

    Some(Batch {
        vertex_buffer,
        vertex_count,
    })
}

fn push_vertices(
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
