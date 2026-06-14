use bytemuck::{Pod, Zeroable};
use wgpu::util::DeviceExt;

use crate::geometry::{Rect, area, point};
use crate::paint;
use crate::render;

pub struct Renderer {
    blur_pipeline: wgpu::RenderPipeline,
    composite_pipeline: wgpu::RenderPipeline,
    bind_group_layout: wgpu::BindGroupLayout,
    sampler: wgpu::Sampler,
    textures: Option<Textures>,
    format: wgpu::TextureFormat,
}

#[derive(Debug, Clone, Copy)]
pub struct Target {
    physical_area: area::Physical,
    logical_area: area::Logical,
    scale_factor: f32,
    surface_usage: wgpu::TextureUsages,
}

struct Textures {
    area: area::Physical,
    source: Texture,
    ping: Texture,
    pong: Texture,
}

struct Texture {
    inner: wgpu::Texture,
    view: wgpu::TextureView,
}

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
struct Params {
    texture_size: [f32; 2],
    canvas_size: [f32; 2],
    direction_radius: [f32; 4],
    rect: [f32; 4],
    radius: [f32; 4],
}

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
struct CompositeVertex {
    position: [f32; 2],
    local_position: [f32; 2],
    rect: [f32; 4],
    radius: [f32; 4],
}

#[derive(Debug, Clone, Copy, PartialEq)]
struct PreparedBackdrop {
    raster_rect: Rect,
    shape_rect: Rect,
    radius: crate::geometry::rect::ResolvedRadius,
    blur_radius: f32,
}

#[derive(Debug, Clone, Copy, PartialEq)]
struct PixelGeometry {
    scale_factor: f32,
    logical_pixel: f32,
}

impl Renderer {
    pub fn new(render_context: &render::Context, format: wgpu::TextureFormat) -> Self {
        let shader = render_context
            .device()
            .create_shader_module(wgpu::include_wgsl!("backdrop.wgsl"));
        let bind_group_layout =
            render_context
                .device()
                .create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                    label: Some("Backdrop Bind Group Layout"),
                    entries: &[
                        wgpu::BindGroupLayoutEntry {
                            binding: 0,
                            visibility: wgpu::ShaderStages::FRAGMENT,
                            ty: wgpu::BindingType::Texture {
                                sample_type: wgpu::TextureSampleType::Float { filterable: true },
                                view_dimension: wgpu::TextureViewDimension::D2,
                                multisampled: false,
                            },
                            count: None,
                        },
                        wgpu::BindGroupLayoutEntry {
                            binding: 1,
                            visibility: wgpu::ShaderStages::FRAGMENT,
                            ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                            count: None,
                        },
                        wgpu::BindGroupLayoutEntry {
                            binding: 2,
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
                    label: Some("Backdrop Pipeline Layout"),
                    bind_group_layouts: &[Some(&bind_group_layout)],
                    immediate_size: 0,
                });
        let blur_pipeline =
            render_context
                .device()
                .create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                    label: Some("Backdrop Blur Pipeline"),
                    layout: Some(&pipeline_layout),
                    vertex: wgpu::VertexState {
                        module: &shader,
                        entry_point: Some("vs_fullscreen"),
                        buffers: &[],
                        compilation_options: Default::default(),
                    },
                    primitive: wgpu::PrimitiveState::default(),
                    depth_stencil: None,
                    multisample: wgpu::MultisampleState::default(),
                    fragment: Some(wgpu::FragmentState {
                        module: &shader,
                        entry_point: Some("fs_blur"),
                        targets: &[Some(wgpu::ColorTargetState {
                            format,
                            blend: None,
                            write_mask: wgpu::ColorWrites::ALL,
                        })],
                        compilation_options: Default::default(),
                    }),
                    multiview_mask: None,
                    cache: None,
                });
        let composite_pipeline =
            render_context
                .device()
                .create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                    label: Some("Backdrop Composite Pipeline"),
                    layout: Some(&pipeline_layout),
                    vertex: wgpu::VertexState {
                        module: &shader,
                        entry_point: Some("vs_composite"),
                        buffers: &[CompositeVertex::layout()],
                        compilation_options: Default::default(),
                    },
                    primitive: wgpu::PrimitiveState::default(),
                    depth_stencil: None,
                    multisample: wgpu::MultisampleState::default(),
                    fragment: Some(wgpu::FragmentState {
                        module: &shader,
                        entry_point: Some("fs_composite"),
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
        let sampler = render_context
            .device()
            .create_sampler(&wgpu::SamplerDescriptor {
                label: Some("Backdrop Sampler"),
                address_mode_u: wgpu::AddressMode::ClampToEdge,
                address_mode_v: wgpu::AddressMode::ClampToEdge,
                mag_filter: wgpu::FilterMode::Linear,
                min_filter: wgpu::FilterMode::Linear,
                ..Default::default()
            });

        Self {
            blur_pipeline,
            composite_pipeline,
            bind_group_layout,
            sampler,
            textures: None,
            format,
        }
    }

    pub fn draw(
        &mut self,
        render_context: &render::Context,
        target: Target,
        encoder: &mut wgpu::CommandEncoder,
        frame: &render::Frame,
        target_view: &wgpu::TextureView,
        backdrop: paint::Backdrop,
    ) {
        if !target.surface_usage.contains(wgpu::TextureUsages::COPY_SRC) {
            log::debug!("skipping backdrop because the surface is not copyable");
            return;
        }

        let paint::BackdropFilter::Blur { radius } = backdrop.filter;
        if radius <= 0.0 {
            return;
        }

        let Some(prepared) = prepare_backdrop(backdrop.rect, radius, target.scale_factor) else {
            return;
        };

        self.ensure_textures(render_context, target.physical_area.clamp_min(1));
        let Some(textures) = self.textures.as_ref() else {
            return;
        };

        encoder.copy_texture_to_texture(
            wgpu::TexelCopyTextureInfo {
                texture: frame.texture(),
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            wgpu::TexelCopyTextureInfo {
                texture: &textures.source.inner,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            wgpu::Extent3d {
                width: textures.area.width(),
                height: textures.area.height(),
                depth_or_array_layers: 1,
            },
        );

        self.blur_pass(
            render_context,
            encoder,
            &textures.source.view,
            &textures.ping.view,
            target,
            prepared,
            [1.0, 0.0],
        );
        self.blur_pass(
            render_context,
            encoder,
            &textures.ping.view,
            &textures.pong.view,
            target,
            prepared,
            [0.0, 1.0],
        );
        self.composite_pass(
            render_context,
            encoder,
            &textures.pong.view,
            target_view,
            target,
            prepared,
        );
    }

    fn ensure_textures(&mut self, render_context: &render::Context, area: area::Physical) {
        if self
            .textures
            .as_ref()
            .is_some_and(|textures| textures.area == area)
        {
            return;
        }

        self.textures = Some(Textures {
            area,
            source: self.create_texture(render_context, area, "Backdrop Source Texture"),
            ping: self.create_texture(render_context, area, "Backdrop Ping Texture"),
            pong: self.create_texture(render_context, area, "Backdrop Pong Texture"),
        });
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
                usage: wgpu::TextureUsages::COPY_DST
                    | wgpu::TextureUsages::RENDER_ATTACHMENT
                    | wgpu::TextureUsages::TEXTURE_BINDING,
                view_formats: &[],
            });
        let view = texture.create_view(&wgpu::TextureViewDescriptor::default());

        Texture {
            inner: texture,
            view,
        }
    }

    fn blur_pass(
        &self,
        render_context: &render::Context,
        encoder: &mut wgpu::CommandEncoder,
        source: &wgpu::TextureView,
        output: &wgpu::TextureView,
        target: Target,
        prepared: PreparedBackdrop,
        direction: [f32; 2],
    ) {
        let params = self.params(target, prepared, direction);
        let bind_group =
            self.bind_group(render_context, source, params, "Backdrop Blur Bind Group");
        let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("Backdrop Blur Pass"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: output,
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

        pass.set_pipeline(&self.blur_pipeline);
        pass.set_bind_group(0, &bind_group, &[]);
        pass.draw(0..6, 0..1);
    }

    fn composite_pass(
        &self,
        render_context: &render::Context,
        encoder: &mut wgpu::CommandEncoder,
        source: &wgpu::TextureView,
        output: &wgpu::TextureView,
        target: Target,
        prepared: PreparedBackdrop,
    ) {
        let params = self.params(target, prepared, [0.0, 0.0]);
        let bind_group = self.bind_group(
            render_context,
            source,
            params,
            "Backdrop Composite Bind Group",
        );
        let vertices = composite_vertices(target.logical_area, prepared);
        let vertex_buffer =
            render_context
                .device()
                .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                    label: Some("Backdrop Composite Vertex Buffer"),
                    contents: bytemuck::cast_slice(&vertices),
                    usage: wgpu::BufferUsages::VERTEX,
                });
        let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("Backdrop Composite Pass"),
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

        pass.set_pipeline(&self.composite_pipeline);
        pass.set_bind_group(0, &bind_group, &[]);
        pass.set_vertex_buffer(0, vertex_buffer.slice(..));
        pass.draw(0..vertices.len() as u32, 0..1);
    }

    fn bind_group(
        &self,
        render_context: &render::Context,
        source: &wgpu::TextureView,
        params: Params,
        label: &'static str,
    ) -> wgpu::BindGroup {
        let buffer =
            render_context
                .device()
                .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                    label: Some("Backdrop Params Buffer"),
                    contents: bytemuck::bytes_of(&params),
                    usage: wgpu::BufferUsages::UNIFORM,
                });

        render_context
            .device()
            .create_bind_group(&wgpu::BindGroupDescriptor {
                label: Some(label),
                layout: &self.bind_group_layout,
                entries: &[
                    wgpu::BindGroupEntry {
                        binding: 0,
                        resource: wgpu::BindingResource::TextureView(source),
                    },
                    wgpu::BindGroupEntry {
                        binding: 1,
                        resource: wgpu::BindingResource::Sampler(&self.sampler),
                    },
                    wgpu::BindGroupEntry {
                        binding: 2,
                        resource: buffer.as_entire_binding(),
                    },
                ],
            })
    }

    fn params(&self, target: Target, prepared: PreparedBackdrop, direction: [f32; 2]) -> Params {
        let physical_area = target.physical_area.clamp_min(1);
        let logical_area = target.logical_area;

        Params {
            texture_size: [physical_area.width() as f32, physical_area.height() as f32],
            canvas_size: [logical_area.width(), logical_area.height()],
            direction_radius: [
                direction[0],
                direction[1],
                prepared.blur_radius * target.scale_factor,
                target.scale_factor,
            ],
            rect: rect_data(prepared.shape_rect),
            radius: radius_data(prepared.radius),
        }
    }
}

impl Target {
    pub fn new(canvas: &render::Canvas) -> Self {
        Self {
            physical_area: canvas.physical_area(),
            logical_area: canvas.logical_area(),
            scale_factor: canvas.scale_factor(),
            surface_usage: canvas.surface().config().usage,
        }
    }
}

impl CompositeVertex {
    fn layout() -> wgpu::VertexBufferLayout<'static> {
        const ATTRIBUTES: [wgpu::VertexAttribute; 4] = wgpu::vertex_attr_array![
            0 => Float32x2,
            1 => Float32x2,
            2 => Float32x4,
            3 => Float32x4
        ];

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

        Self {
            scale_factor,
            logical_pixel: 1.0 / scale_factor,
        }
    }

    fn snap_rect(self, rect: Rect) -> Rect {
        let (left, top, right, bottom) = edges(rect);
        let left = self.snap_position(left);
        let top = self.snap_position(top);
        let mut right = self.snap_position(right);
        let mut bottom = self.snap_position(bottom);

        if right <= left {
            right = left + self.logical_pixel;
        }

        if bottom <= top {
            bottom = top + self.logical_pixel;
        }

        Rect::rounded(
            point::logical(left, top),
            area::logical(right - left, bottom - top),
            rect.radius,
        )
    }

    fn snap_position(self, position: f32) -> f32 {
        (position * self.scale_factor).round() / self.scale_factor
    }
}

fn prepare_backdrop(rect: Rect, blur_radius: f32, scale_factor: f32) -> Option<PreparedBackdrop> {
    if rect.area.width() <= 0.0 || rect.area.height() <= 0.0 {
        return None;
    }

    let pixel_geometry = PixelGeometry::new(scale_factor);
    let shape_rect = pixel_geometry.snap_rect(rect);
    let raster_rect = expand_rect(shape_rect, pixel_geometry.logical_pixel);

    Some(PreparedBackdrop {
        raster_rect,
        shape_rect,
        radius: shape_rect.radius.resolve(shape_rect.area),
        blur_radius: blur_radius.max(0.0),
    })
}

fn composite_vertices(
    canvas_area: area::Logical,
    prepared: PreparedBackdrop,
) -> [CompositeVertex; 6] {
    let to_clip = |x: f32, y: f32| -> [f32; 2] {
        [
            (x / canvas_area.width()) * 2.0 - 1.0,
            1.0 - (y / canvas_area.height()) * 2.0,
        ]
    };
    let (x0, y0, x1, y1) = edges(prepared.raster_rect);
    let rect = rect_data(prepared.shape_rect);
    let radius = radius_data(prepared.radius);
    let vertex = |x: f32, y: f32| CompositeVertex {
        position: to_clip(x, y),
        local_position: [x, y],
        rect,
        radius,
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

fn expand_rect(rect: Rect, amount: f32) -> Rect {
    Rect::new(
        point::logical(rect.origin.x() - amount, rect.origin.y() - amount),
        area::logical(
            rect.area.width() + amount * 2.0,
            rect.area.height() + amount * 2.0,
        ),
    )
}

fn edges(rect: Rect) -> (f32, f32, f32, f32) {
    let x0 = rect.origin.x();
    let y0 = rect.origin.y();

    (x0, y0, x0 + rect.area.width(), y0 + rect.area.height())
}

fn rect_data(rect: Rect) -> [f32; 4] {
    [
        rect.origin.x(),
        rect.origin.y(),
        rect.area.width(),
        rect.area.height(),
    ]
}

fn radius_data(radius: crate::geometry::rect::ResolvedRadius) -> [f32; 4] {
    [
        radius.top_left,
        radius.top_right,
        radius.bottom_right,
        radius.bottom_left,
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn backdrop_shape_snaps_and_raster_bounds_expand_by_one_physical_pixel() {
        let rect = Rect::new(point::logical(10.2, 20.3), area::logical(40.4, 30.8));
        let prepared = prepare_backdrop(rect, 12.0, 2.0).expect("backdrop should prepare");

        assert_eq!(edges(prepared.shape_rect), (10.0, 20.5, 50.5, 51.0));
        assert_eq!(edges(prepared.raster_rect), (9.5, 20.0, 51.0, 51.5));
        assert_eq!(prepared.blur_radius, 12.0);
    }

    #[test]
    fn backdrop_preserves_rounded_shape_metadata_for_composite() {
        let rect = Rect::rounded(
            point::logical(10.0, 20.0),
            area::logical(80.0, 30.0),
            crate::geometry::rect::Radius::splat(1.0),
        );
        let prepared = prepare_backdrop(rect, 18.0, 1.0).expect("backdrop should prepare");
        let vertices = composite_vertices(area::logical(100.0, 100.0), prepared);

        assert_eq!(prepared.radius.top_left, 15.0);
        assert_eq!(vertices.len(), 6);
        assert_eq!(vertices[0].rect, [10.0, 20.0, 80.0, 30.0]);
        assert_eq!(vertices[0].radius, [15.0, 15.0, 15.0, 15.0]);
    }

    #[test]
    fn zero_size_backdrops_do_not_prepare() {
        let rect = Rect::new(point::logical(10.0, 20.0), area::logical(0.0, 30.0));

        assert!(prepare_backdrop(rect, 12.0, 1.0).is_none());
    }
}
