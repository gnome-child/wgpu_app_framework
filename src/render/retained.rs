use bytemuck::{Pod, Zeroable};
use wgpu::util::DeviceExt;

use crate::geometry::{Rect, area, point};
use crate::render;

pub(crate) struct Renderer {
    pipeline: wgpu::RenderPipeline,
    bind_group_layout: wgpu::BindGroupLayout,
    format: wgpu::TextureFormat,
}

pub(crate) struct Layer {
    texture: Texture,
    area: area::Physical,
    logical_area: area::Logical,
}

struct Texture {
    _inner: wgpu::Texture,
    view: wgpu::TextureView,
}

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
struct Params {
    texture_size: [f32; 4],
    source_origin_scale: [f32; 4],
    rect: [f32; 4],
}

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
struct Vertex {
    position: [f32; 2],
    local_position: [f32; 2],
    rect: [f32; 4],
}

#[derive(Debug, Clone, Copy, PartialEq)]
struct PixelGeometry {
    scale_factor: f32,
}

#[derive(Debug, Clone, Copy, PartialEq)]
struct PreparedBlit {
    rect: Rect,
    source_origin_px: point::Physical,
    size_px: area::Physical,
}

impl Renderer {
    pub(crate) fn new(render_context: &render::Context, format: wgpu::TextureFormat) -> Self {
        let shader = render_context
            .device()
            .create_shader_module(wgpu::include_wgsl!("retained.wgsl"));
        let bind_group_layout =
            render_context
                .device()
                .create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                    label: Some("Retained Texture Bind Group Layout"),
                    entries: &[
                        wgpu::BindGroupLayoutEntry {
                            binding: 0,
                            visibility: wgpu::ShaderStages::FRAGMENT,
                            ty: wgpu::BindingType::Texture {
                                sample_type: wgpu::TextureSampleType::Float { filterable: false },
                                view_dimension: wgpu::TextureViewDimension::D2,
                                multisampled: false,
                            },
                            count: None,
                        },
                        wgpu::BindGroupLayoutEntry {
                            binding: 1,
                            visibility: wgpu::ShaderStages::FRAGMENT,
                            ty: wgpu::BindingType::Buffer {
                                ty: wgpu::BufferBindingType::Uniform,
                                has_dynamic_offset: false,
                                min_binding_size: None,
                            },
                            count: None,
                        },
                    ],
                });
        let pipeline_layout =
            render_context
                .device()
                .create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                    label: Some("Retained Texture Pipeline Layout"),
                    bind_group_layouts: &[Some(&bind_group_layout)],
                    immediate_size: 0,
                });
        let pipeline =
            render_context
                .device()
                .create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                    label: Some("Retained Texture Pipeline"),
                    layout: Some(&pipeline_layout),
                    vertex: wgpu::VertexState {
                        module: &shader,
                        entry_point: Some("vs_main"),
                        buffers: &[Vertex::layout()],
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

        Self {
            pipeline,
            bind_group_layout,
            format,
        }
    }

    pub(crate) fn create_layer_for_viewport(
        &self,
        render_context: &render::Context,
        viewport: render::Viewport,
        label: &'static str,
    ) -> Layer {
        let area = viewport.physical_area().clamp_min(1);
        Layer {
            texture: self.create_texture(render_context, area, label),
            area,
            logical_area: viewport.logical_area(),
        }
    }

    pub(crate) fn clear_layer(&self, encoder: &mut wgpu::CommandEncoder, layer: &Layer) {
        let _pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("Retained Texture Clear Pass"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: layer.view(),
                resolve_target: None,
                depth_slice: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Clear(wgpu::Color::TRANSPARENT),
                    store: wgpu::StoreOp::Store,
                },
            })],
            depth_stencil_attachment: None,
            occlusion_query_set: None,
            timestamp_writes: None,
            multiview_mask: None,
        });
    }

    pub(crate) fn draw(
        &self,
        render_context: &render::Context,
        encoder: &mut wgpu::CommandEncoder,
        source: &Layer,
        output: &wgpu::TextureView,
        viewport: render::Viewport,
        rect: Rect,
        source_rect: Rect,
        scissor: Option<render::Scissor>,
    ) {
        let Some(prepared) = prepare_blit(rect, source_rect, viewport.scale_factor()) else {
            return;
        };
        if !source_contains_prepared_blit(source, prepared) {
            trace_retained_blit_miss(viewport, source, rect, source_rect, prepared);
            return;
        }
        trace_retained_blit(viewport, source, rect, source_rect, prepared);

        let params = Params {
            texture_size: [
                source.area().width() as f32,
                source.area().height() as f32,
                0.0,
                0.0,
            ],
            source_origin_scale: [
                prepared.source_origin_px.x(),
                prepared.source_origin_px.y(),
                viewport.scale_factor(),
                viewport.scale_factor(),
            ],
            rect: rect_data(prepared.rect),
        };
        let params_buffer =
            render_context
                .device()
                .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                    label: Some("Retained Texture Params Buffer"),
                    contents: bytemuck::bytes_of(&params),
                    usage: wgpu::BufferUsages::UNIFORM,
                });
        let bind_group = render_context
            .device()
            .create_bind_group(&wgpu::BindGroupDescriptor {
                label: Some("Retained Texture Bind Group"),
                layout: &self.bind_group_layout,
                entries: &[
                    wgpu::BindGroupEntry {
                        binding: 0,
                        resource: wgpu::BindingResource::TextureView(source.view()),
                    },
                    wgpu::BindGroupEntry {
                        binding: 1,
                        resource: params_buffer.as_entire_binding(),
                    },
                ],
            });
        let vertices = vertices(viewport.logical_area(), prepared.rect);
        let vertex_buffer =
            render_context
                .device()
                .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                    label: Some("Retained Texture Vertex Buffer"),
                    contents: bytemuck::cast_slice(&vertices),
                    usage: wgpu::BufferUsages::VERTEX,
                });
        let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("Retained Texture Pass"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: output,
                resolve_target: None,
                depth_slice: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Load,
                    store: wgpu::StoreOp::Store,
                },
            })],
            depth_stencil_attachment: None,
            occlusion_query_set: None,
            timestamp_writes: None,
            multiview_mask: None,
        });

        pass.set_pipeline(&self.pipeline);
        pass.set_bind_group(0, &bind_group, &[]);
        pass.set_vertex_buffer(0, vertex_buffer.slice(..));
        if let Some(scissor) = scissor {
            pass.set_scissor_rect(scissor.x(), scissor.y(), scissor.width(), scissor.height());
        }
        pass.draw(0..vertices.len() as u32, 0..1);
    }

    fn create_texture(
        &self,
        render_context: &render::Context,
        area: area::Physical,
        label: &'static str,
    ) -> Texture {
        let texture = render_context
            .device()
            .create_texture(&wgpu::TextureDescriptor {
                label: Some(label),
                size: wgpu::Extent3d {
                    width: area.width(),
                    height: area.height(),
                    depth_or_array_layers: 1,
                },
                mip_level_count: 1,
                sample_count: 1,
                dimension: wgpu::TextureDimension::D2,
                format: self.format,
                usage: wgpu::TextureUsages::RENDER_ATTACHMENT
                    | wgpu::TextureUsages::TEXTURE_BINDING,
                view_formats: &[],
            });
        let view = texture.create_view(&wgpu::TextureViewDescriptor::default());

        Texture {
            _inner: texture,
            view,
        }
    }
}

impl Layer {
    pub(crate) fn view(&self) -> &wgpu::TextureView {
        &self.texture.view
    }

    pub(crate) fn area(&self) -> area::Physical {
        self.area
    }

    pub(crate) fn logical_area(&self) -> area::Logical {
        self.logical_area
    }
}

impl Vertex {
    fn layout() -> wgpu::VertexBufferLayout<'static> {
        const ATTRIBUTES: [wgpu::VertexAttribute; 3] =
            wgpu::vertex_attr_array![0 => Float32x2, 1 => Float32x2, 2 => Float32x4];

        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<Self>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &ATTRIBUTES,
        }
    }
}

impl PixelGeometry {
    fn new(scale_factor: f32) -> Self {
        let scale_factor = scale_factor.max(0.0001);

        Self { scale_factor }
    }

    fn destination_rect(self, destination: Rect) -> Option<(Rect, area::Physical)> {
        let left = self.snap_position(destination.origin.x());
        let top = self.snap_position(destination.origin.y());
        let right = self.snap_position(destination.origin.x() + destination.area.width());
        let bottom = self.snap_position(destination.origin.y() + destination.area.height());
        let width = right - left;
        let height = bottom - top;
        if width <= 0 || height <= 0 {
            return None;
        }

        let size = area::physical(width as u32, height as u32);
        let rect = Rect::new(
            point::logical(
                left as f32 / self.scale_factor,
                top as f32 / self.scale_factor,
            ),
            area::logical(
                size.width() as f32 / self.scale_factor,
                size.height() as f32 / self.scale_factor,
            ),
        );
        Some((rect, size))
    }

    fn source_origin(self, source: Rect) -> point::Physical {
        point::physical(
            (source.origin.x() * self.scale_factor).round(),
            (source.origin.y() * self.scale_factor).round(),
        )
    }

    fn snap_position(self, position: f32) -> i32 {
        (position * self.scale_factor).round() as i32
    }
}

fn prepare_blit(destination: Rect, source: Rect, scale_factor: f32) -> Option<PreparedBlit> {
    if destination.area.width() <= 0.0 || destination.area.height() <= 0.0 {
        return None;
    }

    let geometry = PixelGeometry::new(scale_factor);
    let (rect, size_px) = geometry.destination_rect(destination)?;
    let source_origin_px = geometry.source_origin(source);

    Some(PreparedBlit {
        rect,
        source_origin_px,
        size_px,
    })
}

fn source_contains_prepared_blit(source: &Layer, prepared: PreparedBlit) -> bool {
    source_area_contains_prepared_blit(source.area(), prepared)
}

fn source_area_contains_prepared_blit(source_area: area::Physical, prepared: PreparedBlit) -> bool {
    let left = prepared.source_origin_px.x();
    let top = prepared.source_origin_px.y();
    let right = left + prepared.size_px.width() as f32;
    let bottom = top + prepared.size_px.height() as f32;

    left >= 0.0
        && top >= 0.0
        && right <= source_area.width() as f32
        && bottom <= source_area.height() as f32
}

fn vertices(canvas_area: area::Logical, rect: Rect) -> [Vertex; 6] {
    let to_clip = |x: f32, y: f32| -> [f32; 2] {
        [
            (x / canvas_area.width()) * 2.0 - 1.0,
            1.0 - (y / canvas_area.height()) * 2.0,
        ]
    };
    let (x0, y0, x1, y1) = edges(rect);
    let rect_data = rect_data(rect);
    let vertex = |x: f32, y: f32| Vertex {
        position: to_clip(x, y),
        local_position: [x, y],
        rect: rect_data,
    };

    [
        vertex(x0, y0),
        vertex(x0, y1),
        vertex(x1, y1),
        vertex(x0, y0),
        vertex(x1, y1),
        vertex(x1, y0),
    ]
}

fn rect_data(rect: Rect) -> [f32; 4] {
    [
        rect.origin.x(),
        rect.origin.y(),
        rect.area.width(),
        rect.area.height(),
    ]
}

fn edges(rect: Rect) -> (f32, f32, f32, f32) {
    let x0 = rect.origin.x();
    let y0 = rect.origin.y();

    (x0, y0, x0 + rect.area.width(), y0 + rect.area.height())
}

fn trace_retained_blit(
    viewport: render::Viewport,
    source: &Layer,
    destination: Rect,
    requested_source: Rect,
    prepared: PreparedBlit,
) {
    if std::env::var_os("WGPU_L3_SCROLL_TRACE").is_none() {
        return;
    }

    let scale = viewport.scale_factor();
    eprintln!(
        "[wgpu_l3 retained] scale={scale} source_area={:?} source_logical={:?} dest={destination:?} snapped={:?} requested_source={requested_source:?} source_origin_px={:?} size_px={:?} target_logical={:?}",
        source.area(),
        source.logical_area(),
        prepared.rect,
        prepared.source_origin_px,
        prepared.size_px,
        viewport.logical_area(),
    );
}

fn trace_retained_blit_miss(
    viewport: render::Viewport,
    source: &Layer,
    destination: Rect,
    requested_source: Rect,
    prepared: PreparedBlit,
) {
    if std::env::var_os("WGPU_L3_SCROLL_TRACE").is_none() {
        return;
    }

    eprintln!(
        "[wgpu_l3 retained] blit bounds miss scale={} source_area={:?} source_logical={:?} dest={destination:?} snapped={:?} requested_source={requested_source:?} source_origin_px={:?} size_px={:?} target_logical={:?}",
        viewport.scale_factor(),
        source.area(),
        source.logical_area(),
        prepared.rect,
        prepared.source_origin_px,
        prepared.size_px,
        viewport.logical_area(),
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn retained_blit_source_origin_tracks_snapped_destination_without_resizing() {
        let destination = Rect::new(point::logical(10.2, 20.3), area::logical(40.4, 30.8));
        let source = Rect::new(point::logical(4.0, 8.0), area::logical(40.4, 30.8));

        let prepared = prepare_blit(destination, source, 2.0).expect("blit should prepare");

        assert_eq!(edges(prepared.rect), (10.0, 20.5, 50.5, 51.0));
        assert_eq!(prepared.rect.area, area::logical(40.5, 30.5));
        assert_eq!(prepared.source_origin_px, point::physical(8.0, 16.0));
        assert_eq!(prepared.size_px, area::physical(81, 61));
    }

    #[test]
    fn retained_blit_destination_size_is_driven_by_destination_pixels() {
        let destination = Rect::new(point::logical(10.0, -2.0), area::logical(23.442871, 32.0));
        let source = Rect::new(point::logical(0.0, 0.0), area::logical(23.442871, 32.0));

        let prepared = prepare_blit(destination, source, 1.25).expect("blit should prepare");

        assert_eq!(edges(prepared.rect), (10.4, -2.4, 33.6, 30.4));
        assert_eq!(prepared.rect.area, area::logical(23.2, 32.8));
        assert_eq!(prepared.source_origin_px, point::physical(0.0, 0.0));
        assert_eq!(prepared.size_px, area::physical(29, 41));
    }

    #[test]
    fn retained_blit_destination_size_is_stable_when_source_origin_changes() {
        let destination = Rect::new(point::logical(10.0, 20.0), area::logical(300.3, 240.6));
        let first_source = Rect::new(point::logical(0.0, 0.0), destination.area);
        let second_source = Rect::new(point::logical(0.0, 137.4), destination.area);

        let first = prepare_blit(destination, first_source, 1.25).expect("first blit");
        let second = prepare_blit(destination, second_source, 1.25).expect("second blit");

        assert_eq!(first.rect, second.rect);
        assert_eq!(first.size_px, second.size_px);
        assert_ne!(first.source_origin_px, second.source_origin_px);
    }

    #[test]
    fn retained_blit_rejects_source_bounds_miss_instead_of_clamping() {
        let prepared = PreparedBlit {
            rect: Rect::new(point::logical(0.0, 0.0), area::logical(24.0, 32.0)),
            source_origin_px: point::physical(1.0, 0.0),
            size_px: area::physical(30, 40),
        };

        assert!(!source_area_contains_prepared_blit(
            area::physical(30, 40),
            prepared
        ));
    }

    #[test]
    fn retained_blit_vertices_use_snapped_destination_rect() {
        let rect = Rect::new(point::logical(10.0, 20.5), area::logical(40.5, 30.5));

        let vertices = vertices(area::logical(100.0, 100.0), rect);

        assert_eq!(vertices[0].local_position, [10.0, 20.5]);
        assert_eq!(vertices[1].local_position, [10.0, 51.0]);
        assert_eq!(vertices[2].local_position, [50.5, 51.0]);
        assert_eq!(vertices[0].rect, [10.0, 20.5, 40.5, 30.5]);
    }
}
