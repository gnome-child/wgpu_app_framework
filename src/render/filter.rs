use bytemuck::{Pod, Zeroable};
use std::cell::RefCell;

use wgpu::util::DeviceExt;

use crate::paint::{self, Rect};
use crate::render;
use crate::render::silhouette::{PreparedSilhouette, edges, rect_data, rounding_data};

mod noise;
mod params;
mod shader;

use params::{AlphaMode, ParamInput, Params};

#[cfg(test)]
use params::{
    noise_material_position_data, physical_rect_data, physical_source_rect_data, source_scale_data,
    source_step_data, with_texture_area as params_with_texture_area,
};

const LAYER_POOL_LIMIT: usize = 8;
const SCRATCH_POOL_LIMIT: usize = 8;

pub struct Renderer {
    blur_pipeline: wgpu::RenderPipeline,
    liquid_pipeline: wgpu::RenderPipeline,
    luminosity_pipeline: wgpu::RenderPipeline,
    noise_pipeline: wgpu::RenderPipeline,
    blit_pipeline: wgpu::RenderPipeline,
    composite_pipeline: wgpu::RenderPipeline,
    pixel_composite_pipeline: wgpu::RenderPipeline,
    bind_group_layout: wgpu::BindGroupLayout,
    filtered_sampler: wgpu::Sampler,
    pixel_aligned_sampler: wgpu::Sampler,
    noise_texture: Texture,
    noise_sampler: wgpu::Sampler,
    textures: Option<Textures>,
    layer_pool: RefCell<Vec<Layer>>,
    scratch_pool: RefCell<Vec<ScratchTextures>>,
    format: wgpu::TextureFormat,
}

pub struct Layer {
    texture: Texture,
    area: paint::area::Physical,
    logical_area: paint::area::Logical,
}

#[derive(Debug, Clone, Copy)]
pub struct Target {
    physical_area: paint::area::Physical,
    logical_area: paint::area::Logical,
    scale_factor: f32,
}

struct Textures {
    area: paint::area::Physical,
    logical_area: paint::area::Logical,
    composition: Texture,
    ping: Texture,
    pong: Texture,
}

struct Texture {
    _inner: wgpu::Texture,
    view: wgpu::TextureView,
    area: paint::area::Physical,
}

struct ScratchTextures {
    ping: Texture,
    pong: Texture,
    area: paint::area::Physical,
    logical_area: paint::area::Logical,
}

enum ScratchTargets<'a> {
    Shared {
        ping: &'a Texture,
        pong: &'a Texture,
        logical_area: paint::area::Logical,
    },
    Pooled(ScratchTextures),
}

pub(crate) struct LayerComposite<'a> {
    pub(crate) render_context: &'a render::Context,
    pub(crate) encoder: &'a mut wgpu::CommandEncoder,
    pub(crate) source: &'a Layer,
    pub(crate) output: &'a wgpu::TextureView,
    pub(crate) target: Target,
    pub(crate) clip: paint::Clip,
    pub(crate) source_rect: Option<Rect>,
    pub(crate) scissor: Option<render::Scissor>,
    pub(crate) opacity: f32,
}

#[derive(Clone, Copy)]
pub(crate) struct TextureSource<'a> {
    view: &'a wgpu::TextureView,
    area: paint::area::Physical,
    logical_area: paint::area::Logical,
    sampling: paint::LayerSampling,
}

impl<'a> TextureSource<'a> {
    pub(crate) fn new(
        view: &'a wgpu::TextureView,
        area: paint::area::Physical,
        logical_area: paint::area::Logical,
        sampling: paint::LayerSampling,
    ) -> Self {
        debug_assert!(area.width() > 0 && area.height() > 0);
        debug_assert!(logical_area.width() > 0.0 && logical_area.height() > 0.0);
        Self {
            view,
            area,
            logical_area,
            sampling,
        }
    }

    fn for_target_view(
        view: &'a wgpu::TextureView,
        target: Target,
        sampling: paint::LayerSampling,
    ) -> Self {
        let area = target.physical_area.clamp_min(1);

        Self::new(view, area, area.to_logical(target.scale_factor), sampling)
    }
}

impl Texture {
    fn source(
        &self,
        logical_area: paint::area::Logical,
        sampling: paint::LayerSampling,
    ) -> TextureSource<'_> {
        debug_assert!(self.area.width() > 0 && self.area.height() > 0);
        debug_assert!(logical_area.width() > 0.0 && logical_area.height() > 0.0);
        TextureSource {
            view: &self.view,
            area: self.area,
            logical_area,
            sampling,
        }
    }
}

impl<'a> FilterSource<'a> {
    fn initial_sample(self) -> FilterSample<'a> {
        match self {
            Self::Backdrop {
                texture,
                global_rect,
            } => FilterSample {
                texture,
                rect: global_rect,
                space: FilterSourceSpace::Backdrop,
            },
            Self::Local {
                texture,
                local_rect,
            } => FilterSample {
                texture,
                rect: local_rect,
                space: FilterSourceSpace::Local,
            },
        }
    }
}

impl<'a> FilterChainContext<'a> {
    fn new(
        target: Target,
        output: &'a wgpu::TextureView,
        prepared: PreparedFilter,
        source: FilterSource<'a>,
    ) -> Self {
        let sample = source.initial_sample();

        Self {
            target,
            output,
            prepared,
            current_source: sample.texture,
            current_rect: sample.rect,
            current_space: sample.space,
        }
    }

    fn target(&self) -> Target {
        self.target
    }

    fn output(&self) -> &'a wgpu::TextureView {
        self.output
    }

    fn base_prepared(&self) -> PreparedFilter {
        self.prepared
    }

    fn current_sample(&self) -> FilterSample<'a> {
        FilterSample {
            texture: self.current_source,
            rect: self.current_rect,
            space: self.current_space,
        }
    }

    fn local_rect(&self) -> Rect {
        self.prepared.shape_rect
    }

    fn local_intermediate<'b>(&self, texture: TextureSource<'b>) -> FilterSample<'b> {
        FilterSample {
            texture,
            rect: self.local_rect(),
            space: FilterSourceSpace::Local,
        }
    }

    fn mark_output_as_current(&mut self) {
        self.current_source = TextureSource::for_target_view(
            self.output,
            self.target,
            paint::LayerSampling::PixelAligned,
        );
        self.current_rect = self.local_rect();
        self.current_space = FilterSourceSpace::Local;
    }
}

impl ScratchTextures {
    fn new(render_context: &render::Context, renderer: &Renderer, target: Target) -> Self {
        let area = target.physical_area.clamp_min(1);

        Self {
            ping: renderer.create_texture(render_context, area, "Filter Scratch Ping Texture"),
            pong: renderer.create_texture(render_context, area, "Filter Scratch Pong Texture"),
            area,
            logical_area: area.to_logical(target.scale_factor),
        }
    }

    fn retarget(&mut self, target: Target) {
        debug_assert_eq!(self.area, target.physical_area.clamp_min(1));
        self.logical_area = self.area.to_logical(target.scale_factor);
    }
}

impl<'a> ScratchTargets<'a> {
    fn ping_view(&self) -> &wgpu::TextureView {
        match self {
            Self::Shared { ping, .. } => &ping.view,
            Self::Pooled(scratch) => &scratch.ping.view,
        }
    }

    fn pong_view(&self) -> &wgpu::TextureView {
        match self {
            Self::Shared { pong, .. } => &pong.view,
            Self::Pooled(scratch) => &scratch.pong.view,
        }
    }

    fn ping_source(&self, sampling: paint::LayerSampling) -> TextureSource<'_> {
        match self {
            Self::Shared {
                ping, logical_area, ..
            } => ping.source(*logical_area, sampling),
            Self::Pooled(scratch) => scratch.ping.source(scratch.logical_area, sampling),
        }
    }

    fn pong_source(&self, sampling: paint::LayerSampling) -> TextureSource<'_> {
        match self {
            Self::Shared {
                pong, logical_area, ..
            } => pong.source(*logical_area, sampling),
            Self::Pooled(scratch) => scratch.pong.source(scratch.logical_area, sampling),
        }
    }
}

pub(crate) struct FilterDraw<'a> {
    pub(crate) render_context: &'a render::Context,
    pub(crate) encoder: &'a mut wgpu::CommandEncoder,
    pub(crate) target: Target,
    pub(crate) source: FilterSource<'a>,
    pub(crate) output: &'a wgpu::TextureView,
    pub(crate) filter: paint::Filter,
    pub(crate) scissor: Option<render::Scissor>,
}

#[derive(Clone, Copy)]
pub(crate) enum FilterSource<'a> {
    Backdrop {
        texture: TextureSource<'a>,
        global_rect: Rect,
    },
    Local {
        texture: TextureSource<'a>,
        local_rect: Rect,
    },
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum FilterSourceSpace {
    Backdrop,
    Local,
}

#[derive(Clone, Copy)]
struct FilterSample<'a> {
    texture: TextureSource<'a>,
    rect: Rect,
    space: FilterSourceSpace,
}

struct FilterChainContext<'a> {
    target: Target,
    output: &'a wgpu::TextureView,
    prepared: PreparedFilter,
    current_source: TextureSource<'a>,
    current_rect: Rect,
    current_space: FilterSourceSpace,
}

struct BlurPass<'a> {
    render_context: &'a render::Context,
    encoder: &'a mut wgpu::CommandEncoder,
    output: &'a wgpu::TextureView,
    target: Target,
    prepared: PreparedFilter,
    direction: [f32; 2],
    source: TextureSource<'a>,
    source_rect: Rect,
    source_space: FilterSourceSpace,
    labels: BlurLabels,
}

struct LiquidPass<'a> {
    render_context: &'a render::Context,
    encoder: &'a mut wgpu::CommandEncoder,
    source: &'a wgpu::TextureView,
    source_area: paint::area::Physical,
    source_logical_area: paint::area::Logical,
    source_rect: Rect,
    source_sampling: paint::LayerSampling,
    output: &'a wgpu::TextureView,
    target: Target,
    prepared: PreparedFilter,
    effect: [f32; 4],
    alpha_mode: AlphaMode,
    scissor: Option<render::Scissor>,
}

struct EffectPass<'a> {
    render_context: &'a render::Context,
    encoder: &'a mut wgpu::CommandEncoder,
    source: &'a wgpu::TextureView,
    source_area: paint::area::Physical,
    source_logical_area: paint::area::Logical,
    source_rect: Rect,
    source_sampling: paint::LayerSampling,
    output: &'a wgpu::TextureView,
    target: Target,
    prepared: PreparedFilter,
    effect: [f32; 4],
    alpha_mode: AlphaMode,
    pipeline: &'a wgpu::RenderPipeline,
    scissor: Option<render::Scissor>,
    labels: PassLabels,
}

struct CompositePass<'a> {
    render_context: &'a render::Context,
    encoder: &'a mut wgpu::CommandEncoder,
    source: TextureSource<'a>,
    output: &'a wgpu::TextureView,
    target: Target,
    prepared: PreparedFilter,
    source_rect: Rect,
    opacity: f32,
    alpha_mode: AlphaMode,
    scissor: Option<render::Scissor>,
    labels: PassLabels,
}

#[derive(Clone, Copy)]
struct PassLabels {
    bind_group: &'static str,
    vertex_buffer: &'static str,
    pass: &'static str,
}

#[derive(Clone, Copy)]
struct BlurLabels {
    bind_group: &'static str,
    pass: &'static str,
}

impl BlurLabels {
    const fn new(bind_group: &'static str, pass: &'static str) -> Self {
        Self { bind_group, pass }
    }
}

impl PassLabels {
    const fn new(
        bind_group: &'static str,
        vertex_buffer: &'static str,
        pass: &'static str,
    ) -> Self {
        Self {
            bind_group,
            vertex_buffer,
            pass,
        }
    }
}

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
struct CompositeVertex {
    position: [f32; 2],
    local_position: [f32; 2],
    rect: [f32; 4],
    rounding: [f32; 4],
}

#[derive(Debug, Clone, Copy, PartialEq)]
struct PreparedFilter {
    raster_rect: Rect,
    shape_rect: Rect,
    rounding: [f32; 4],
    blur_amount: f32,
    blur_sigma_px: f32,
    blur_radius_px: f32,
}

impl Renderer {
    const MAX_LIQUID_REFRACTION: f32 = 48.0;

    pub fn new(render_context: &render::Context, format: wgpu::TextureFormat) -> Self {
        let shader_source = shader_source();
        let shader = render_context
            .device()
            .create_shader_module(wgpu::ShaderModuleDescriptor {
                label: Some("filter.wgsl"),
                source: wgpu::ShaderSource::Wgsl(shader_source.into()),
            });
        let bind_group_layout =
            render_context
                .device()
                .create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                    label: Some("Filter Bind Group Layout"),
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
                        wgpu::BindGroupLayoutEntry {
                            binding: 3,
                            visibility: wgpu::ShaderStages::FRAGMENT,
                            ty: wgpu::BindingType::Texture {
                                sample_type: wgpu::TextureSampleType::Float { filterable: true },
                                view_dimension: wgpu::TextureViewDimension::D2,
                                multisampled: false,
                            },
                            count: None,
                        },
                        wgpu::BindGroupLayoutEntry {
                            binding: 4,
                            visibility: wgpu::ShaderStages::FRAGMENT,
                            ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                            count: None,
                        },
                    ],
                });
        let pipeline_layout =
            render_context
                .device()
                .create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                    label: Some("Filter Pipeline Layout"),
                    bind_group_layouts: &[Some(&bind_group_layout)],
                    immediate_size: 0,
                });
        let blur_pipeline =
            render_context
                .device()
                .create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                    label: Some("Filter Blur Pipeline"),
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
        let blit_pipeline =
            render_context
                .device()
                .create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                    label: Some("Filter Blit Pipeline"),
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
                        entry_point: Some("fs_blit"),
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
        let liquid_pipeline =
            render_context
                .device()
                .create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                    label: Some("Filter Liquid Pipeline"),
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
                        entry_point: Some("fs_liquid"),
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
        let luminosity_pipeline =
            render_context
                .device()
                .create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                    label: Some("Filter Luminosity Pipeline"),
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
                        entry_point: Some("fs_luminosity"),
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
        let noise_pipeline =
            render_context
                .device()
                .create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                    label: Some("Filter Noise Pipeline"),
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
                        entry_point: Some("fs_noise"),
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
        let composite_pipeline =
            render_context
                .device()
                .create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                    label: Some("Filter Composite Pipeline"),
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
        let pixel_composite_pipeline =
            render_context
                .device()
                .create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                    label: Some("Pixel Aligned Layer Composite Pipeline"),
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
                        entry_point: Some("fs_composite_pixel"),
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
        let filtered_sampler = render_context
            .device()
            .create_sampler(&wgpu::SamplerDescriptor {
                label: Some("Filtered Layer Sampler"),
                address_mode_u: wgpu::AddressMode::ClampToEdge,
                address_mode_v: wgpu::AddressMode::ClampToEdge,
                mag_filter: wgpu::FilterMode::Linear,
                min_filter: wgpu::FilterMode::Linear,
                ..Default::default()
            });
        let pixel_aligned_sampler =
            render_context
                .device()
                .create_sampler(&wgpu::SamplerDescriptor {
                    label: Some("Pixel Aligned Layer Sampler"),
                    address_mode_u: wgpu::AddressMode::ClampToEdge,
                    address_mode_v: wgpu::AddressMode::ClampToEdge,
                    mag_filter: wgpu::FilterMode::Nearest,
                    min_filter: wgpu::FilterMode::Nearest,
                    ..Default::default()
                });
        let noise_texture = noise::create_texture(render_context);
        let noise_sampler = render_context
            .device()
            .create_sampler(&wgpu::SamplerDescriptor {
                label: Some("Filter Noise Sampler"),
                address_mode_u: wgpu::AddressMode::Repeat,
                address_mode_v: wgpu::AddressMode::Repeat,
                address_mode_w: wgpu::AddressMode::Repeat,
                mag_filter: wgpu::FilterMode::Nearest,
                min_filter: wgpu::FilterMode::Nearest,
                mipmap_filter: wgpu::MipmapFilterMode::Nearest,
                ..Default::default()
            });

        Self {
            blur_pipeline,
            liquid_pipeline,
            luminosity_pipeline,
            noise_pipeline,
            blit_pipeline,
            composite_pipeline,
            pixel_composite_pipeline,
            bind_group_layout,
            filtered_sampler,
            pixel_aligned_sampler,
            noise_texture,
            noise_sampler,
            textures: None,
            layer_pool: RefCell::new(Vec::new()),
            scratch_pool: RefCell::new(Vec::new()),
            format,
        }
    }

    pub fn prepare(&mut self, render_context: &render::Context, canvas: &render::Canvas) -> Target {
        let target = Target::new(canvas);
        self.ensure_textures(
            render_context,
            target.physical_area.clamp_min(1),
            target.logical_area,
        );
        target
    }

    pub fn composition_view(&self) -> Option<&wgpu::TextureView> {
        Some(&self.textures.as_ref()?.composition.view)
    }

    pub fn clear_composition(&self, encoder: &mut wgpu::CommandEncoder, clear_color: wgpu::Color) {
        let Some(view) = self.composition_view() else {
            return;
        };
        clear_view(encoder, view, clear_color, "Composition Clear Pass");
    }

    pub fn create_layer(
        &self,
        render_context: &render::Context,
        target: Target,
        label: &'static str,
    ) -> Layer {
        let area = target.physical_area.clamp_min(1);
        if let Some(mut layer) = take_pooled_layer(&mut self.layer_pool.borrow_mut(), area) {
            layer.logical_area = area.to_logical(target.scale_factor);
            return layer;
        }

        Layer {
            texture: self.create_texture(render_context, area, label),
            area,
            logical_area: area.to_logical(target.scale_factor),
        }
    }

    pub fn recycle_layer(&self, layer: Layer) {
        let mut pool = self.layer_pool.borrow_mut();
        if pool.len() == LAYER_POOL_LIMIT {
            pool.remove(0);
        }
        pool.push(layer);
    }

    fn scratch_targets<'a>(
        &'a self,
        render_context: &render::Context,
        target: Target,
        textures: &'a Textures,
    ) -> ScratchTargets<'a> {
        let area = target.physical_area.clamp_min(1);
        if area == textures.area && target.logical_area == textures.logical_area {
            return ScratchTargets::Shared {
                ping: &textures.ping,
                pong: &textures.pong,
                logical_area: textures.logical_area,
            };
        }

        ScratchTargets::Pooled(self.take_scratch(render_context, target))
    }

    fn take_scratch(&self, render_context: &render::Context, target: Target) -> ScratchTextures {
        let area = target.physical_area.clamp_min(1);
        if let Some(mut scratch) = take_pooled_scratch(&mut self.scratch_pool.borrow_mut(), area) {
            scratch.retarget(target);
            return scratch;
        }

        ScratchTextures::new(render_context, self, target)
    }

    fn recycle_scratch(&self, scratch: ScratchTextures) {
        let mut pool = self.scratch_pool.borrow_mut();
        if pool.len() == SCRATCH_POOL_LIMIT {
            pool.remove(0);
        }
        pool.push(scratch);
    }

    pub fn clear_layer(&self, encoder: &mut wgpu::CommandEncoder, layer: &Layer) {
        clear_view(
            encoder,
            layer.view(),
            wgpu::Color::TRANSPARENT,
            "Layer Clear Pass",
        );
    }

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
                    paint::FilterOp::Liquid {
                        depth,
                        splay,
                        feather,
                        curve,
                    } => {
                        if liquid_is_identity(depth) {
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
                            effect: liquid_effect(depth, splay, feather, curve),
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
                                "Filter Liquid Composite Bind Group",
                                "Filter Liquid Composite Vertex Buffer",
                                "Filter Liquid Composite Pass",
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

    pub(crate) fn composite_layer(&self, pass: LayerComposite<'_>) {
        let Some(prepared) = prepare_clip(pass.clip.rect, pass.target.scale_factor) else {
            return;
        };
        let source_rect = pass.source_rect.unwrap_or_else(|| {
            source_rect_for_prepared_destination(pass.clip.rect, prepared, pass.clip.rect)
        });

        self.composite_pass(CompositePass {
            render_context: pass.render_context,
            encoder: pass.encoder,
            source: pass.source.source(paint::LayerSampling::PixelAligned),
            output: pass.output,
            target: pass.target,
            prepared,
            source_rect,
            opacity: pass.opacity,
            alpha_mode: AlphaMode::Source,
            scissor: pass.scissor,
            labels: PassLabels {
                bind_group: "Layer Composite Bind Group",
                vertex_buffer: "Layer Composite Vertex Buffer",
                pass: "Layer Composite Pass",
            },
        });
    }

    pub fn blit_to_view(
        &self,
        render_context: &render::Context,
        encoder: &mut wgpu::CommandEncoder,
        output: &wgpu::TextureView,
        target: Target,
    ) {
        let Some(textures) = self.textures.as_ref() else {
            return;
        };
        let physical_area = target.physical_area.clamp_min(1);
        let params = Params {
            texture_size: [physical_area.width() as f32, physical_area.height() as f32],
            source_scale: [target.scale_factor, target.scale_factor],
            direction_radius: [0.0, 0.0, 0.0, 0.0],
            effect: [1.0, 0.0, 0.0, 0.0],
            rect: [
                0.0,
                0.0,
                target.logical_area.width(),
                target.logical_area.height(),
            ],
            source_rect: [
                0.0,
                0.0,
                target.logical_area.width(),
                target.logical_area.height(),
            ],
            target_rect: [
                0.0,
                0.0,
                target.physical_area.width() as f32,
                target.physical_area.height() as f32,
            ],
            rounding: [0.0; 4],
            alpha_mode: [AlphaMode::Source.shader_value(), 0.0, 0.0, 0.0],
        };
        let bind_group = self.bind_group(
            render_context,
            &textures.composition.view,
            params,
            paint::LayerSampling::Filtered,
            "Filter Blit Bind Group",
        );
        let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("Filter Blit Pass"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: output,
                resolve_target: None,
                depth_slice: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Clear(wgpu::Color::BLACK),
                    store: wgpu::StoreOp::Store,
                },
            })],
            depth_stencil_attachment: None,
            occlusion_query_set: None,
            timestamp_writes: None,
            multiview_mask: None,
        });

        pass.set_pipeline(&self.blit_pipeline);
        pass.set_bind_group(0, &bind_group, &[]);
        pass.draw(0..3, 0..1);
    }

    fn ensure_textures(
        &mut self,
        render_context: &render::Context,
        area: paint::area::Physical,
        logical_area: paint::area::Logical,
    ) {
        if self
            .textures
            .as_ref()
            .is_some_and(|textures| textures.area == area && textures.logical_area == logical_area)
        {
            return;
        }

        self.textures = Some(Textures {
            area,
            logical_area,
            composition: self.create_texture(render_context, area, "Filter Composition Texture"),
            ping: self.create_texture(render_context, area, "Filter Ping Texture"),
            pong: self.create_texture(render_context, area, "Filter Pong Texture"),
        });
    }

    fn create_texture(
        &self,
        render_context: &render::Context,
        area: paint::area::Physical,
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
            area,
        }
    }

    fn blur_pass(&self, pass: BlurPass<'_>) {
        let params = self.params_with_texture_area(ParamInput {
            target_scale_factor: pass.target.scale_factor,
            texture_area: pass.source.area,
            texture_logical_area: pass.source.logical_area,
            prepared: pass.prepared,
            source_rect: pass.source_rect,
            direction: pass.direction,
            effect: [pass.prepared.blur_sigma_px, 0.0, 0.0, 0.0],
            alpha_mode: AlphaMode::Source,
            sampling: pass.source.sampling,
        });
        log::debug!(
            target: "wgpu_l3::render::filter_params",
            "{} source={:?} source_rect={:?} target_rect={:?} texture_size={:?} target_area={:?}",
            pass.labels.pass,
            pass.source_space,
            params.source_rect,
            params.target_rect,
            params.texture_size,
            pass.target.physical_area,
        );
        let bind_group = self.bind_group(
            pass.render_context,
            pass.source.view,
            params,
            pass.source.sampling,
            pass.labels.bind_group,
        );
        let mut render_pass = pass.encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some(pass.labels.pass),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: pass.output,
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

        render_pass.set_pipeline(&self.blur_pipeline);
        render_pass.set_bind_group(0, &bind_group, &[]);
        render_pass.draw(0..3, 0..1);
    }

    fn liquid_pass(&self, pass: LiquidPass<'_>) {
        let params = self.params_with_texture_area(ParamInput {
            target_scale_factor: pass.target.scale_factor,
            texture_area: pass.source_area,
            texture_logical_area: pass.source_logical_area,
            prepared: pass.prepared,
            source_rect: pass.source_rect,
            direction: [0.0, 0.0],
            effect: pass.effect,
            alpha_mode: pass.alpha_mode,
            sampling: pass.source_sampling,
        });
        let bind_group = self.bind_group(
            pass.render_context,
            pass.source,
            params,
            pass.source_sampling,
            "Filter Liquid Bind Group",
        );
        let vertices = composite_vertices(pass.target.logical_area, pass.prepared);
        let vertex_buffer =
            pass.render_context
                .device()
                .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                    label: Some("Filter Liquid Vertex Buffer"),
                    contents: bytemuck::cast_slice(&vertices),
                    usage: wgpu::BufferUsages::VERTEX,
                });
        let mut render_pass = pass.encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("Filter Liquid Pass"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: pass.output,
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

        render_pass.set_pipeline(&self.liquid_pipeline);
        render_pass.set_bind_group(0, &bind_group, &[]);
        render_pass.set_vertex_buffer(0, vertex_buffer.slice(..));
        if let Some(scissor) = pass.scissor {
            render_pass.set_scissor_rect(
                scissor.x(),
                scissor.y(),
                scissor.width(),
                scissor.height(),
            );
        }
        render_pass.draw(0..vertices.len() as u32, 0..1);
    }

    fn effect_pass(&self, pass: EffectPass<'_>) {
        let params = self.params_with_texture_area(ParamInput {
            target_scale_factor: pass.target.scale_factor,
            texture_area: pass.source_area,
            texture_logical_area: pass.source_logical_area,
            prepared: pass.prepared,
            source_rect: pass.source_rect,
            direction: [0.0, 0.0],
            effect: pass.effect,
            alpha_mode: pass.alpha_mode,
            sampling: pass.source_sampling,
        });
        let bind_group = self.bind_group(
            pass.render_context,
            pass.source,
            params,
            pass.source_sampling,
            pass.labels.bind_group,
        );
        let vertices = composite_vertices(pass.target.logical_area, pass.prepared);
        let vertex_buffer =
            pass.render_context
                .device()
                .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                    label: Some(pass.labels.vertex_buffer),
                    contents: bytemuck::cast_slice(&vertices),
                    usage: wgpu::BufferUsages::VERTEX,
                });
        let mut render_pass = pass.encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some(pass.labels.pass),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: pass.output,
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

        render_pass.set_pipeline(pass.pipeline);
        render_pass.set_bind_group(0, &bind_group, &[]);
        render_pass.set_vertex_buffer(0, vertex_buffer.slice(..));
        if let Some(scissor) = pass.scissor {
            render_pass.set_scissor_rect(
                scissor.x(),
                scissor.y(),
                scissor.width(),
                scissor.height(),
            );
        }
        render_pass.draw(0..vertices.len() as u32, 0..1);
    }

    fn composite_pass(&self, pass: CompositePass<'_>) {
        let params = self.params_with_texture_area(ParamInput {
            target_scale_factor: pass.target.scale_factor,
            texture_area: pass.source.area,
            texture_logical_area: pass.source.logical_area,
            prepared: pass.prepared,
            source_rect: pass.source_rect,
            direction: [0.0, 0.0],
            effect: [pass.opacity.clamp(0.0, 1.0), 0.0, 0.0, 0.0],
            alpha_mode: pass.alpha_mode,
            sampling: pass.source.sampling,
        });
        log::debug!(
            target: "wgpu_l3::render::filter_params",
            "{} source_rect={:?} target_rect={:?} coverage_rect={:?} texture_size={:?} target_area={:?} alpha_mode={:?} alpha_flags={:?} opacity={:.4}",
            pass.labels.pass,
            params.source_rect,
            params.target_rect,
            params.rect,
            params.texture_size,
            pass.target.physical_area,
            pass.alpha_mode,
            params.alpha_mode,
            pass.opacity,
        );
        let bind_group = self.bind_group(
            pass.render_context,
            pass.source.view,
            params,
            pass.source.sampling,
            pass.labels.bind_group,
        );
        let vertices = composite_vertices(pass.target.logical_area, pass.prepared);
        let vertex_buffer =
            pass.render_context
                .device()
                .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                    label: Some(pass.labels.vertex_buffer),
                    contents: bytemuck::cast_slice(&vertices),
                    usage: wgpu::BufferUsages::VERTEX,
                });
        let mut render_pass = pass.encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some(pass.labels.pass),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: pass.output,
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

        render_pass.set_pipeline(match pass.source.sampling {
            paint::LayerSampling::Filtered => &self.composite_pipeline,
            paint::LayerSampling::PixelAligned => &self.pixel_composite_pipeline,
        });
        render_pass.set_bind_group(0, &bind_group, &[]);
        render_pass.set_vertex_buffer(0, vertex_buffer.slice(..));
        if let Some(scissor) = pass.scissor {
            render_pass.set_scissor_rect(
                scissor.x(),
                scissor.y(),
                scissor.width(),
                scissor.height(),
            );
        }
        render_pass.draw(0..vertices.len() as u32, 0..1);
    }

    fn bind_group(
        &self,
        render_context: &render::Context,
        source: &wgpu::TextureView,
        params: Params,
        sampling: paint::LayerSampling,
        label: &'static str,
    ) -> wgpu::BindGroup {
        let buffer =
            render_context
                .device()
                .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                    label: Some("Filter Params Buffer"),
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
                        resource: wgpu::BindingResource::Sampler(self.sampler(sampling)),
                    },
                    wgpu::BindGroupEntry {
                        binding: 2,
                        resource: buffer.as_entire_binding(),
                    },
                    wgpu::BindGroupEntry {
                        binding: 3,
                        resource: wgpu::BindingResource::TextureView(&self.noise_texture.view),
                    },
                    wgpu::BindGroupEntry {
                        binding: 4,
                        resource: wgpu::BindingResource::Sampler(&self.noise_sampler),
                    },
                ],
            })
    }

    fn sampler(&self, sampling: paint::LayerSampling) -> &wgpu::Sampler {
        match sampling {
            paint::LayerSampling::Filtered => &self.filtered_sampler,
            paint::LayerSampling::PixelAligned => &self.pixel_aligned_sampler,
        }
    }

    fn params_with_texture_area(&self, input: ParamInput) -> Params {
        params::with_texture_area(input)
    }
}

fn take_pooled_layer(pool: &mut Vec<Layer>, area: paint::area::Physical) -> Option<Layer> {
    let position = pool.iter().position(|layer| layer.area == area)?;
    Some(pool.swap_remove(position))
}

fn take_pooled_scratch(
    pool: &mut Vec<ScratchTextures>,
    area: paint::area::Physical,
) -> Option<ScratchTextures> {
    let position = pool.iter().position(|scratch| scratch.area == area)?;
    Some(pool.swap_remove(position))
}

pub(crate) fn shader_source() -> String {
    shader::module_source()
}

impl Layer {
    pub fn view(&self) -> &wgpu::TextureView {
        &self.texture.view
    }

    fn source(&self, sampling: paint::LayerSampling) -> TextureSource<'_> {
        debug_assert_eq!(self.texture.area, self.area);
        TextureSource {
            view: self.view(),
            area: self.area,
            logical_area: self.logical_area,
            sampling,
        }
    }
}

impl Target {
    pub fn new(canvas: &render::Canvas) -> Self {
        Self {
            physical_area: canvas.physical_area(),
            logical_area: canvas.logical_area(),
            scale_factor: canvas.scale_factor(),
        }
    }

    pub fn from_viewport(viewport: render::Viewport) -> Self {
        Self {
            physical_area: viewport.physical_area(),
            logical_area: viewport.logical_area(),
            scale_factor: viewport.scale_factor(),
        }
    }

    pub fn from_logical_area(logical_area: paint::area::Logical, scale_factor: f32) -> Self {
        Self {
            physical_area: logical_area.to_physical(scale_factor).clamp_min(1),
            logical_area,
            scale_factor,
        }
    }

    pub fn physical_area(self) -> paint::area::Physical {
        self.physical_area
    }

    pub fn logical_area(self) -> paint::area::Logical {
        self.logical_area
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

impl PreparedFilter {
    fn with_blur(mut self, blur_amount: f32, scale_factor: f32) -> Self {
        let blur_amount = blur_amount.clamp(0.0, 1.0);

        self.blur_amount = blur_amount;
        self.blur_radius_px = blur_radius_px(blur_amount, scale_factor);
        self.blur_sigma_px = self.blur_radius_px * 0.42;
        self
    }

    fn with_blur_sigma(mut self, blur_sigma: f32, scale_factor: f32) -> Self {
        let blur_sigma = blur_sigma.max(0.0);

        self.blur_amount = 0.0;
        self.blur_sigma_px = blur_sigma_px(blur_sigma, scale_factor);
        self.blur_radius_px = blur_kernel_radius_px(blur_sigma, scale_factor);
        self
    }
}

fn prepare_filter(rect: Rect, scale_factor: f32) -> Option<PreparedFilter> {
    prepare_clip(rect, scale_factor)
}

fn prepare_clip(rect: Rect, scale_factor: f32) -> Option<PreparedFilter> {
    let silhouette = PreparedSilhouette::for_filter_rect(rect, scale_factor)?;

    Some(PreparedFilter {
        raster_rect: silhouette.raster_rect,
        shape_rect: silhouette.shape_rect,
        rounding: silhouette.rounding,
        blur_amount: 0.0,
        blur_sigma_px: 0.0,
        blur_radius_px: 0.0,
    })
}

fn source_rect_for_prepared_destination(
    destination: Rect,
    prepared: PreparedFilter,
    source: Rect,
) -> Rect {
    let origin_delta = paint::point::logical(
        prepared.shape_rect.origin.x() - destination.origin.x(),
        prepared.shape_rect.origin.y() - destination.origin.y(),
    );

    Rect::new(
        paint::point::logical(
            source.origin.x() + origin_delta.x(),
            source.origin.y() + origin_delta.y(),
        ),
        prepared.shape_rect.area,
    )
}

fn blur_radius_px(amount: f32, scale_factor: f32) -> f32 {
    paint::filter_blur_radius_px(amount, scale_factor)
}

fn blur_sigma_px(sigma: f32, scale_factor: f32) -> f32 {
    paint::filter_blur_sigma_px(sigma, scale_factor)
}

fn blur_kernel_radius_px(sigma: f32, scale_factor: f32) -> f32 {
    paint::filter_blur_kernel_radius_px(sigma, scale_factor)
}

fn liquid_depth_displacement(depth: f32) -> f32 {
    depth.clamp(0.0, 1.0) * Renderer::MAX_LIQUID_REFRACTION
}

fn liquid_effect(depth: f32, splay: f32, feather: f32, curve: f32) -> [f32; 4] {
    [
        liquid_depth_displacement(depth),
        splay.max(0.0),
        feather.max(0.0),
        curve.max(0.1),
    ]
}

fn refraction_effect(refraction: paint::Refraction) -> [f32; 4] {
    [
        refraction.displacement.clamp(0.0, 4.0),
        refraction.splay.max(0.0),
        refraction.feather.max(0.0),
        refraction.curve.max(0.1),
    ]
}

fn liquid_is_identity(depth: f32) -> bool {
    depth <= 0.0
}

fn composite_vertices(
    canvas_area: paint::area::Logical,
    prepared: PreparedFilter,
) -> [CompositeVertex; 6] {
    let to_clip = |x: f32, y: f32| -> [f32; 2] {
        [
            (x / canvas_area.width()) * 2.0 - 1.0,
            1.0 - (y / canvas_area.height()) * 2.0,
        ]
    };
    let (x0, y0, x1, y1) = edges(prepared.raster_rect);
    let rect = rect_data(prepared.shape_rect);
    let rounding = rounding_data(prepared.rounding);
    let vertex = |x: f32, y: f32| CompositeVertex {
        position: to_clip(x, y),
        local_position: [x, y],
        rect,
        rounding,
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

fn clear_view(
    encoder: &mut wgpu::CommandEncoder,
    view: &wgpu::TextureView,
    color: wgpu::Color,
    label: &'static str,
) {
    let _pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
        label: Some(label),
        color_attachments: &[Some(wgpu::RenderPassColorAttachment {
            view,
            resolve_target: None,
            depth_slice: None,
            ops: wgpu::Operations {
                load: wgpu::LoadOp::Clear(color),
                store: wgpu::StoreOp::Store,
            },
        })],
        depth_stencil_attachment: None,
        occlusion_query_set: None,
        timestamp_writes: None,
        multiview_mask: None,
    });
}

#[cfg(test)]
pub(crate) fn prepared_filter_silhouette_for_test(
    rect: Rect,
    scale_factor: f32,
) -> Option<PreparedSilhouette> {
    let prepared = prepare_filter(rect, scale_factor)?;

    Some(
        PreparedSilhouette::from_parts(prepared.shape_rect, prepared.raster_rect)
            .with_rounding(prepared.rounding),
    )
}

#[cfg(test)]
pub(crate) fn prepared_clip_silhouette_for_test(
    rect: Rect,
    scale_factor: f32,
) -> Option<PreparedSilhouette> {
    let prepared = prepare_clip(rect, scale_factor)?;

    Some(
        PreparedSilhouette::from_parts(prepared.shape_rect, prepared.raster_rect)
            .with_rounding(prepared.rounding),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn filter_shape_snaps_and_raster_bounds_expand_by_one_physical_pixel() {
        let rect = Rect::new(
            paint::point::logical(10.2, 20.3),
            paint::area::logical(40.4, 30.8),
        );
        let prepared = prepare_filter(rect, 2.0)
            .expect("filter should prepare")
            .with_blur_sigma(30.0, 2.0);

        assert_eq!(edges(prepared.shape_rect), (10.0, 20.5, 50.5, 51.0));
        assert_eq!(edges(prepared.raster_rect), (9.5, 20.0, 51.0, 51.5));
        assert_eq!(prepared.blur_amount, 0.0);
        assert_eq!(prepared.blur_sigma_px, 60.0);
        assert_eq!(prepared.blur_radius_px, 180.0);
    }

    #[test]
    fn filter_preserves_rounded_shape_metadata_for_composite() {
        let rect = Rect::rounded(
            paint::point::logical(10.0, 20.0),
            paint::area::logical(80.0, 30.0),
            crate::paint::Rounding::relative(1.0),
        );
        let prepared = prepare_filter(rect, 1.0).expect("filter should prepare");
        let vertices = composite_vertices(paint::area::logical(100.0, 100.0), prepared);

        assert_eq!(prepared.rounding[0], 15.0);
        assert_eq!(vertices.len(), 6);
        assert_eq!(vertices[0].rect, [10.0, 20.0, 80.0, 30.0]);
        assert_eq!(vertices[0].rounding, [15.0, 15.0, 15.0, 15.0]);
    }

    #[test]
    fn noise_texture_bytes_are_deterministic_rgba_grayscale() {
        let bytes = noise::bytes();

        assert_eq!(
            bytes.len(),
            (noise::texture_size() * noise::texture_size() * 4) as usize
        );
        assert_eq!(
            &bytes[..16],
            &[
                152, 152, 152, 255, 166, 166, 166, 255, 110, 110, 110, 255, 160, 160, 160, 255
            ]
        );
        assert!(
            bytes
                .chunks_exact(4)
                .all(|rgba| { rgba[0] == rgba[1] && rgba[1] == rgba[2] && rgba[3] == u8::MAX })
        );
    }

    #[test]
    fn noise_texel_range_stays_low_contrast() {
        let bytes = noise::bytes();
        let values = bytes.chunks_exact(4).map(|rgba| rgba[0]);
        let min = values.clone().min().expect("noise should have texels");
        let max = values.max().expect("noise should have texels");

        assert_eq!(min, 84);
        assert_eq!(max, 172);
    }

    #[test]
    fn noise_texel_values_are_seed_stable() {
        assert_eq!(noise::texel(0, 0), 152);
        assert_eq!(noise::texel(1, 0), 166);
        assert_eq!(noise::texel(0, 1), 106);
        assert_eq!(noise::texel(17, 31), 167);
        assert_eq!(noise::texel(127, 127), 143);
    }

    #[test]
    fn clip_preserves_rounded_shape_metadata_for_layer_composite() {
        let rect = Rect::rounded(
            paint::point::logical(8.0, 12.0),
            paint::area::logical(48.0, 20.0),
            crate::paint::Rounding::relative(1.0),
        );
        let prepared = prepare_clip(rect, 1.0).expect("clip should prepare");
        let vertices = composite_vertices(paint::area::logical(100.0, 100.0), prepared);

        assert_eq!(prepared.blur_amount, 0.0);
        assert_eq!(prepared.blur_sigma_px, 0.0);
        assert_eq!(prepared.blur_radius_px, 0.0);
        assert_eq!(vertices[0].rect, [8.0, 12.0, 48.0, 20.0]);
        assert_eq!(vertices[0].rounding, [10.0, 10.0, 10.0, 10.0]);
    }

    #[test]
    fn layer_source_rect_tracks_snapped_destination_without_scaling() {
        let destination = Rect::new(
            paint::point::logical(10.2, 20.3),
            paint::area::logical(40.4, 30.8),
        );
        let source = Rect::new(
            paint::point::logical(4.0, 8.0),
            paint::area::logical(40.4, 30.8),
        );
        let prepared = prepare_clip(destination, 2.0).expect("clip should prepare");

        let source = source_rect_for_prepared_destination(destination, prepared, source);

        assert_eq!(edges(prepared.shape_rect), (10.0, 20.5, 50.5, 51.0));
        assert_edges_close(edges(source), (3.8, 8.2, 44.3, 38.7));
        assert_eq!(source.area, prepared.shape_rect.area);
    }

    #[test]
    fn layer_source_rect_data_uses_source_texture_scale() {
        let destination = Rect::new(
            paint::point::logical(10.2, 20.3),
            paint::area::logical(40.4, 30.8),
        );
        let source = Rect::new(
            paint::point::logical(4.0, 8.0),
            paint::area::logical(40.4, 30.8),
        );
        let prepared = prepare_clip(destination, 2.0).expect("clip should prepare");
        let source = source_rect_for_prepared_destination(destination, prepared, source);

        let source_data = physical_source_rect_data(
            source,
            paint::area::logical(100.0, 80.0),
            paint::area::physical(200, 160),
            2.0,
            paint::LayerSampling::Filtered,
        );
        let source_size = source.area.to_physical(2.0);

        assert_eq!(source_data[0], 8.0);
        assert_eq!(source_data[1], 16.0);
        assert_eq!(source_data[2], source_size.width() as f32);
        assert_eq!(source_data[3], source_size.height() as f32);
    }

    #[test]
    fn layer_source_rect_data_does_not_assume_destination_scale() {
        let source = Rect::new(
            paint::point::logical(4.0, 8.0),
            paint::area::logical(40.0, 30.0),
        );

        let source_data = physical_source_rect_data(
            source,
            paint::area::logical(80.0, 100.0),
            paint::area::physical(100, 150),
            2.0,
            paint::LayerSampling::Filtered,
        );

        assert_eq!(source_data, [5.0, 12.0, 50.0, 45.0]);
    }

    #[test]
    fn pixel_aligned_source_rect_data_uses_target_scale() {
        let source = Rect::new(
            paint::point::logical(2.0, 4.0),
            paint::area::logical(801.2, 1047.2),
        );

        let source_data = physical_source_rect_data(
            source,
            paint::area::logical(805.2, 2138.4),
            paint::area::physical(1007, 2673),
            1.25,
            paint::LayerSampling::PixelAligned,
        );

        assert_eq!(source_data, [3.0, 5.0, 1002.0, 1309.0]);
    }

    #[test]
    fn target_rect_data_is_physical_destination_space() {
        let target = Rect::new(
            paint::point::logical(12.0, 8.0),
            paint::area::logical(160.0, 80.0),
        );

        assert_eq!(physical_rect_data(target, 1.25), [15.0, 10.0, 200.0, 100.0]);
    }

    #[test]
    fn alpha_mode_params_encode_shape_and_source_modes() {
        let prepared = prepare_filter(
            Rect::new(
                paint::point::logical(0.0, 0.0),
                paint::area::logical(160.0, 80.0),
            ),
            1.0,
        )
        .expect("filter should prepare");
        let input = |alpha_mode| ParamInput {
            target_scale_factor: 1.0,
            texture_area: paint::area::physical(160, 80),
            texture_logical_area: paint::area::logical(160.0, 80.0),
            prepared,
            source_rect: prepared.shape_rect,
            direction: [0.0, 0.0],
            effect: [1.0, 0.0, 0.0, 0.0],
            alpha_mode,
            sampling: paint::LayerSampling::Filtered,
        };

        let source = params_with_texture_area(input(AlphaMode::Source));
        let shape = params_with_texture_area(input(AlphaMode::Shape));

        assert_eq!(source.alpha_mode, [0.0, 0.0, 0.0, 0.0]);
        assert_eq!(shape.alpha_mode, [1.0, 0.0, 0.0, 0.0]);
    }

    #[test]
    fn filter_shader_uses_named_alpha_mode_helper() {
        let source = shader::raw_source();

        assert!(source.contains("alpha_mode: vec4<f32>"));
        assert!(source.contains("fn filter_alpha"));
        assert!(source.contains("fn filter_source_rgb"));
        assert!(source.contains("params.alpha_mode.x"));
    }

    #[test]
    fn shape_alpha_mode_reads_source_rgb_without_unpremultiply() {
        let source = shader::raw_source();
        let helper = source
            .split("fn filter_source_rgb")
            .nth(1)
            .expect("source rgb helper should exist")
            .split("fn filter_alpha")
            .next()
            .expect("source rgb helper should precede alpha helper");

        assert!(helper.contains("return color.rgb;"));
        assert!(helper.contains("return unpremultiply(color);"));

        for fragment in [
            "fs_composite",
            "fs_liquid",
            "fs_luminosity",
            "fs_noise",
            "fs_composite_pixel",
        ] {
            let body = source
                .split(&format!("fn {fragment}"))
                .nth(1)
                .unwrap_or_else(|| panic!("{fragment} fragment should exist"))
                .split("@fragment")
                .next()
                .expect("fragment body should be bounded");

            assert!(body.contains("filter_source_rgb(color)"));
            assert!(!body.contains("unpremultiply(color)"));
        }
    }

    #[test]
    fn blur_intermediate_source_rect_is_local_after_first_pass() {
        let local_filter_rect = Rect::new(
            paint::point::logical(0.0, 0.0),
            paint::area::logical(160.0, 80.0),
        );
        let global_backdrop_rect = Rect::new(
            paint::point::logical(240.0, 120.0),
            paint::area::logical(160.0, 80.0),
        );
        let prepared = prepare_filter(local_filter_rect, 1.25).expect("filter should prepare");

        assert_eq!(
            physical_source_rect_data(
                global_backdrop_rect,
                paint::area::logical(800.0, 600.0),
                paint::area::physical(1000, 750),
                1.25,
                paint::LayerSampling::PixelAligned,
            ),
            [300.0, 150.0, 200.0, 100.0]
        );
        assert_eq!(
            physical_source_rect_data(
                prepared.shape_rect,
                paint::area::logical(160.0, 80.0),
                paint::area::physical(200, 100),
                1.25,
                paint::LayerSampling::PixelAligned,
            ),
            [0.0, 0.0, 200.0, 100.0]
        );
    }

    #[test]
    fn first_backdrop_sample_can_use_global_texture_extent_for_local_target() {
        let prepared = prepare_filter(
            Rect::new(
                paint::point::logical(0.0, 0.0),
                paint::area::logical(160.0, 80.0),
            ),
            1.5,
        )
        .expect("filter should prepare");
        let params = params_with_texture_area(ParamInput {
            target_scale_factor: 1.5,
            texture_area: paint::area::physical(1200, 900),
            texture_logical_area: paint::area::logical(800.0, 600.0),
            prepared,
            source_rect: Rect::new(
                paint::point::logical(240.0, 120.0),
                paint::area::logical(160.0, 80.0),
            ),
            direction: [1.0, 0.0],
            effect: [1.0, 0.0, 0.0, 0.0],
            alpha_mode: AlphaMode::Shape,
            sampling: paint::LayerSampling::PixelAligned,
        });

        assert_eq!(params.texture_size, [1200.0, 900.0]);
        assert_eq!(params.source_rect, [360.0, 180.0, 240.0, 120.0]);
        assert_eq!(params.target_rect, [0.0, 0.0, 240.0, 120.0]);
    }

    #[test]
    fn group_intermediate_params_use_target_local_texture_extent() {
        let prepared = prepare_filter(
            Rect::new(
                paint::point::logical(0.0, 0.0),
                paint::area::logical(160.0, 80.0),
            ),
            1.5,
        )
        .expect("filter should prepare");
        let params = params_with_texture_area(ParamInput {
            target_scale_factor: 1.5,
            texture_area: paint::area::physical(240, 120),
            texture_logical_area: paint::area::logical(160.0, 80.0),
            prepared,
            source_rect: prepared.shape_rect,
            direction: [0.0, 1.0],
            effect: [1.0, 0.0, 0.0, 0.0],
            alpha_mode: AlphaMode::Shape,
            sampling: paint::LayerSampling::Filtered,
        });

        assert_eq!(params.texture_size, [240.0, 120.0]);
        assert_eq!(params.source_rect, [0.0, 0.0, 240.0, 120.0]);
        assert_eq!(params.target_rect, [0.0, 0.0, 240.0, 120.0]);
    }

    #[test]
    fn promoted_blur_first_pass_samples_global_and_writes_local() {
        let pane = paint::Pane::new(
            Rect::new(
                paint::point::logical(20.0, 30.0),
                paint::area::logical(50.0, 40.0),
            ),
            paint::Material::Glass(paint::Glass {
                fallback: paint::Brush::solid(paint::Color::BLACK),
                backdrop_layers: vec![paint::BackdropLayer::Blur(paint::BackdropBlur {
                    sigma: 44.55,
                    edge_mode: paint::BackdropEdgeMode::Mirror,
                })],
                surface_layers: Vec::new(),
            }),
        );
        let group = paint::group_from_items(&[paint::Item::Pane(pane)], 0.5, paint::Grid::new(1.5))
            .expect("pane should produce group");
        let [paint::Item::Pane(local)] = group.items.as_slice() else {
            panic!("expected translated pane");
        };
        let prepared = prepare_filter(local.rect, 1.5)
            .expect("filter should prepare")
            .with_blur_sigma(44.55, 1.5);
        let params = params_with_texture_area(ParamInput {
            target_scale_factor: 1.5,
            texture_area: paint::area::physical(1200, 900),
            texture_logical_area: paint::area::logical(800.0, 600.0),
            prepared,
            source_rect: local
                .source_rect
                .expect("group pane keeps global source rect"),
            direction: [1.0, 0.0],
            effect: [prepared.blur_sigma_px, 0.0, 0.0, 0.0],
            alpha_mode: AlphaMode::Source,
            sampling: paint::LayerSampling::PixelAligned,
        });

        assert_eq!(params.source_rect, [30.0, 45.0, 75.0, 60.0]);
        assert_eq!(params.target_rect, [201.0, 201.0, 75.0, 60.0]);
        assert!(rect_fits_in_area(prepared.raster_rect, group.bounds.area));
    }

    #[test]
    fn high_sigma_blur_write_rect_fits_in_inflated_group_target() {
        let pane = paint::Pane::new(
            Rect::new(
                paint::point::logical(20.0, 30.0),
                paint::area::logical(50.0, 40.0),
            ),
            paint::Material::Glass(paint::Glass {
                fallback: paint::Brush::solid(paint::Color::BLACK),
                backdrop_layers: vec![paint::BackdropLayer::Blur(paint::BackdropBlur {
                    sigma: 44.55,
                    edge_mode: paint::BackdropEdgeMode::Mirror,
                })],
                surface_layers: Vec::new(),
            }),
        );

        for scale in [1.0, 1.5] {
            let group = paint::group_from_items(
                &[paint::Item::Pane(pane.clone())],
                0.5,
                paint::Grid::new(scale),
            )
            .expect("pane should produce group");
            let [paint::Item::Pane(local)] = group.items.as_slice() else {
                panic!("expected translated pane");
            };
            let prepared = prepare_filter(local.rect, scale)
                .expect("filter should prepare")
                .with_blur_sigma(44.55, scale);

            assert!(
                rect_fits_in_area(prepared.raster_rect, group.bounds.area),
                "scale {scale} should keep the blur write rect inside target-local scratch"
            );
        }
    }

    #[test]
    fn blur_pass_labels_distinguish_horizontal_and_vertical_stages() {
        let source = std::fs::read_to_string(file!()).expect("filter source should be readable");

        for label in [
            "Filter Backdrop Blur Horizontal Pass",
            "Filter Backdrop Blur Vertical Pass",
            "Filter Blur Horizontal Pass",
            "Filter Blur Vertical Pass",
        ] {
            assert!(source.contains(label), "missing blur pass label {label}");
        }
    }

    #[test]
    fn composite_pass_logs_coverage_and_alpha_params() {
        let source = std::fs::read_to_string(file!()).expect("filter source should be readable");
        let log = source
            .split("target: \"wgpu_l3::render::filter_params\"")
            .find(|entry| entry.contains("coverage_rect") && entry.contains("alpha_flags"))
            .expect("composite pass should log coverage and alpha params");

        assert!(log.contains("source_rect"));
        assert!(log.contains("target_rect"));
        assert!(log.contains("texture_size"));
        assert!(log.contains("target_area"));
    }

    #[test]
    fn filter_op_composites_use_context_output() {
        let source = std::fs::read_to_string(file!()).expect("filter source should be readable");
        let draw_body = source
            .split("pub(crate) fn draw")
            .nth(1)
            .expect("draw function should exist")
            .split("pub(crate) fn composite_layer")
            .next()
            .expect("draw body should precede layer composite");

        assert!(
            !draw_body.contains("output: pass.output"),
            "filter op composites must use the chain output, not raw draw-pass output"
        );
        assert!(draw_body.contains("output: chain.output()"));
        assert!(draw_body.contains("chain.local_intermediate"));
        assert!(draw_body.contains("mark_output_as_current"));
    }

    #[test]
    fn noise_material_coordinates_are_panel_local_across_group_translation() {
        let scale = 1.5;
        let inline_rect = Rect::new(
            paint::point::logical(240.0, 120.0),
            paint::area::logical(160.0, 80.0),
        );
        let promoted_rect = Rect::new(
            paint::point::logical(0.0, 0.0),
            paint::area::logical(160.0, 80.0),
        );
        let inline_prepared = prepare_filter(inline_rect, scale).expect("filter should prepare");
        let promoted_prepared =
            prepare_filter(promoted_rect, scale).expect("filter should prepare");
        let inline_params = params_with_texture_area(ParamInput {
            target_scale_factor: scale,
            texture_area: paint::area::physical(1200, 900),
            texture_logical_area: paint::area::logical(800.0, 600.0),
            prepared: inline_prepared,
            source_rect: inline_prepared.shape_rect,
            direction: [0.0, 0.0],
            effect: [1.0, 0.0, 0.0, 0.0],
            alpha_mode: AlphaMode::Shape,
            sampling: paint::LayerSampling::PixelAligned,
        });
        let promoted_params = params_with_texture_area(ParamInput {
            target_scale_factor: scale,
            texture_area: paint::area::physical(1200, 900),
            texture_logical_area: paint::area::logical(800.0, 600.0),
            prepared: promoted_prepared,
            source_rect: inline_prepared.shape_rect,
            direction: [0.0, 0.0],
            effect: [1.0, 0.0, 0.0, 0.0],
            alpha_mode: AlphaMode::Shape,
            sampling: paint::LayerSampling::PixelAligned,
        });

        assert_eq!(
            noise_material_position_data([264.0, 138.0], inline_params),
            noise_material_position_data([24.0, 18.0], promoted_params)
        );
    }

    #[test]
    fn noise_material_coordinates_ignore_source_rect() {
        let prepared = prepare_filter(
            Rect::new(
                paint::point::logical(0.0, 0.0),
                paint::area::logical(160.0, 80.0),
            ),
            1.25,
        )
        .expect("filter should prepare");
        let params_a = params_with_texture_area(ParamInput {
            target_scale_factor: 1.25,
            texture_area: paint::area::physical(1000, 750),
            texture_logical_area: paint::area::logical(800.0, 600.0),
            prepared,
            source_rect: prepared.shape_rect,
            direction: [0.0, 0.0],
            effect: [1.0, 0.0, 0.0, 0.0],
            alpha_mode: AlphaMode::Shape,
            sampling: paint::LayerSampling::PixelAligned,
        });
        let params_b = params_with_texture_area(ParamInput {
            target_scale_factor: 1.25,
            texture_area: paint::area::physical(1000, 750),
            texture_logical_area: paint::area::logical(800.0, 600.0),
            prepared,
            source_rect: Rect::new(
                paint::point::logical(300.0, 150.0),
                paint::area::logical(160.0, 80.0),
            ),
            direction: [0.0, 0.0],
            effect: [1.0, 0.0, 0.0, 0.0],
            alpha_mode: AlphaMode::Shape,
            sampling: paint::LayerSampling::PixelAligned,
        });

        assert_ne!(params_a.source_rect, params_b.source_rect);
        assert_eq!(
            noise_material_position_data([32.0, 24.0], params_a),
            noise_material_position_data([32.0, 24.0], params_b)
        );
    }

    #[test]
    fn noise_material_coordinates_use_target_physical_scale() {
        let prepared = prepare_filter(
            Rect::new(
                paint::point::logical(0.0, 0.0),
                paint::area::logical(80.0, 40.0),
            ),
            1.25,
        )
        .expect("filter should prepare");
        let params = params_with_texture_area(ParamInput {
            target_scale_factor: 1.25,
            texture_area: paint::area::physical(100, 50),
            texture_logical_area: paint::area::logical(80.0, 40.0),
            prepared,
            source_rect: prepared.shape_rect,
            direction: [0.0, 0.0],
            effect: [1.0, 0.0, 0.0, 0.0],
            alpha_mode: AlphaMode::Shape,
            sampling: paint::LayerSampling::PixelAligned,
        });

        assert_eq!(
            noise_material_position_data([16.0, 8.0], params),
            [20.0, 10.0]
        );
    }

    #[test]
    fn noise_shader_uses_material_local_coordinates() {
        let noise_body = shader::raw_source()
            .split("fn fs_noise")
            .nth(1)
            .expect("noise fragment should exist")
            .split("fn fs_composite_pixel")
            .next()
            .expect("noise fragment should precede pixel composite");

        assert!(noise_body.contains("material_position_for_local"));
        assert!(
            !noise_body.contains("source_position_for_local(in.local_position) / vec2<f32>(128.0)")
        );
    }

    #[test]
    fn retained_layer_composite_uses_source_rect_step_by_axis() {
        let source_rect_data = [5.0, 12.0, 48.0, 33.0];

        assert_eq!(
            source_step_data(
                source_rect_data,
                paint::area::logical(32.0, 22.0),
                2.0,
                paint::LayerSampling::Filtered,
            ),
            [1.5, 1.5]
        );
    }

    #[test]
    fn retained_layer_source_step_does_not_assume_whole_texture_scale() {
        let source_rect_data = [10.0, 20.0, 96.0, 72.0];

        assert_eq!(
            source_scale_data(
                paint::area::logical(200.0, 100.0),
                paint::area::physical(300, 300)
            ),
            [1.5, 3.0]
        );
        assert_eq!(
            source_step_data(
                source_rect_data,
                paint::area::logical(64.0, 48.0),
                2.0,
                paint::LayerSampling::Filtered,
            ),
            [1.5, 1.5]
        );
    }

    #[test]
    fn pixel_aligned_layer_source_step_uses_target_scale_not_source_size() {
        let source_rect_data = [10.0, 20.0, 97.0, 73.0];

        assert_eq!(
            source_step_data(
                source_rect_data,
                paint::area::logical(64.0, 48.0),
                1.25,
                paint::LayerSampling::PixelAligned,
            ),
            [1.25, 1.25]
        );
    }

    #[test]
    fn zero_size_filters_do_not_prepare() {
        let rect = Rect::new(
            paint::point::logical(10.0, 20.0),
            paint::area::logical(0.0, 30.0),
        );

        assert!(prepare_filter(rect, 1.0).is_none());
    }

    #[test]
    fn normalized_blur_amount_maps_to_internal_physical_cap() {
        assert_eq!(blur_radius_px(-1.0, 1.0), 0.0);
        assert_eq!(blur_radius_px(0.5, 1.0), 128.0);
        assert_eq!(blur_radius_px(1.0, 1.0), 256.0);
        assert_eq!(blur_radius_px(1.0, 2.0), 256.0);
    }

    #[test]
    fn blur_sigma_maps_to_dip_kernel_radius() {
        assert_eq!(blur_sigma_px(30.0, 1.0), 30.0);
        assert_eq!(blur_sigma_px(30.0, 2.0), 60.0);
        assert_eq!(blur_kernel_radius_px(-1.0, 1.0), 0.0);
        assert_eq!(blur_kernel_radius_px(30.0, 1.0), 90.0);
        assert_eq!(blur_kernel_radius_px(30.0, 2.0), 180.0);
    }

    #[test]
    fn normalized_liquid_depth_maps_to_logical_cap() {
        assert_eq!(liquid_depth_displacement(-1.0), 0.0);
        assert_eq!(liquid_depth_displacement(0.5), 24.0);
        assert_eq!(liquid_depth_displacement(1.0), 48.0);
        assert_eq!(liquid_depth_displacement(2.0), 48.0);
    }

    #[test]
    fn liquid_effect_clamps_and_preserves_shape_parameters() {
        assert_eq!(liquid_effect(0.5, 2.0, 18.0, 2.0), [24.0, 2.0, 18.0, 2.0]);
        assert_eq!(liquid_effect(2.0, -1.0, -4.0, 0.0), [48.0, 0.0, 0.0, 0.1]);
    }

    #[test]
    fn zero_depth_liquid_is_identity() {
        assert!(liquid_is_identity(0.0));
        assert!(liquid_is_identity(-1.0));
        assert!(!liquid_is_identity(0.01));
    }

    fn assert_edges_close(left: (f32, f32, f32, f32), right: (f32, f32, f32, f32)) {
        const EPSILON: f32 = 0.0001;
        assert!((left.0 - right.0).abs() <= EPSILON, "{left:?} != {right:?}");
        assert!((left.1 - right.1).abs() <= EPSILON, "{left:?} != {right:?}");
        assert!((left.2 - right.2).abs() <= EPSILON, "{left:?} != {right:?}");
        assert!((left.3 - right.3).abs() <= EPSILON, "{left:?} != {right:?}");
    }

    fn rect_fits_in_area(rect: Rect, area: paint::area::Logical) -> bool {
        const EPSILON: f32 = 0.0001;
        let (left, top, right, bottom) = edges(rect);

        left >= -EPSILON
            && top >= -EPSILON
            && right <= area.width() + EPSILON
            && bottom <= area.height() + EPSILON
    }
}
