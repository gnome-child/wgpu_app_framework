use bytemuck::{Pod, Zeroable};
use wgpu::util::DeviceExt;

use crate::paint::{self, Rect};
use crate::render;
use crate::render::silhouette::{self, PreparedSilhouette, edges, rect_data, rounding_data};

const FILTER_WGSL: &str = include_str!("filter.wgsl");

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
    composition: Texture,
    ping: Texture,
    pong: Texture,
}

struct Texture {
    _inner: wgpu::Texture,
    view: wgpu::TextureView,
}

pub(crate) struct LayerComposite<'a> {
    pub(crate) render_context: &'a render::Context,
    pub(crate) encoder: &'a mut wgpu::CommandEncoder,
    pub(crate) source: &'a Layer,
    pub(crate) output: &'a wgpu::TextureView,
    pub(crate) target: Target,
    pub(crate) clip: paint::Clip,
    pub(crate) scissor: Option<render::Scissor>,
}

struct TextureSource<'a> {
    view: &'a wgpu::TextureView,
    area: paint::area::Physical,
    logical_area: paint::area::Logical,
    sampling: paint::LayerSampling,
}

struct BlurPass<'a> {
    render_context: &'a render::Context,
    encoder: &'a mut wgpu::CommandEncoder,
    source: &'a wgpu::TextureView,
    output: &'a wgpu::TextureView,
    target: Target,
    prepared: PreparedFilter,
    direction: [f32; 2],
}

struct LiquidPass<'a> {
    render_context: &'a render::Context,
    encoder: &'a mut wgpu::CommandEncoder,
    source: &'a wgpu::TextureView,
    output: &'a wgpu::TextureView,
    target: Target,
    prepared: PreparedFilter,
    effect: [f32; 4],
    scissor: Option<render::Scissor>,
}

struct EffectPass<'a> {
    render_context: &'a render::Context,
    encoder: &'a mut wgpu::CommandEncoder,
    source: &'a wgpu::TextureView,
    output: &'a wgpu::TextureView,
    target: Target,
    prepared: PreparedFilter,
    effect: [f32; 4],
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
    scissor: Option<render::Scissor>,
    labels: PassLabels,
}

#[derive(Clone, Copy)]
struct PassLabels {
    bind_group: &'static str,
    vertex_buffer: &'static str,
    pass: &'static str,
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

struct ParamInput {
    target_scale_factor: f32,
    texture_area: paint::area::Physical,
    texture_logical_area: paint::area::Logical,
    prepared: PreparedFilter,
    source_rect: Rect,
    direction: [f32; 2],
    effect: [f32; 4],
    sampling: paint::LayerSampling,
}

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
struct Params {
    texture_size: [f32; 2],
    source_scale: [f32; 2],
    direction_radius: [f32; 4],
    effect: [f32; 4],
    rect: [f32; 4],
    source_rect: [f32; 4],
    rounding: [f32; 4],
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
    const NOISE_TEXTURE_SIZE: u32 = 128;
    const NOISE_SEED: u32 = 0x8f73_d4a9;
    const MAX_BLUR_RADIUS_PX: f32 = 256.0;
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
        let noise_texture = create_noise_texture(render_context);
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
            format,
        }
    }

    pub fn prepare(&mut self, render_context: &render::Context, canvas: &render::Canvas) -> Target {
        let target = Target::new(canvas);
        self.ensure_textures(render_context, target.physical_area.clamp_min(1));
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
        Layer {
            texture: self.create_texture(render_context, area, label),
            area,
            logical_area: area.to_logical(target.scale_factor),
        }
    }

    pub fn clear_layer(&self, encoder: &mut wgpu::CommandEncoder, layer: &Layer) {
        clear_view(
            encoder,
            layer.view(),
            wgpu::Color::TRANSPARENT,
            "Layer Clear Pass",
        );
    }

    pub fn draw(
        &self,
        render_context: &render::Context,
        target: Target,
        encoder: &mut wgpu::CommandEncoder,
        filter: paint::Filter,
        scissor: Option<render::Scissor>,
    ) {
        let Some(prepared) = prepare_filter(filter.rect, target.scale_factor) else {
            return;
        };
        let Some(textures) = self.textures.as_ref() else {
            return;
        };

        for op in filter.ops {
            match op {
                paint::FilterOp::Blur { amount } => {
                    if amount <= 0.0 {
                        continue;
                    }

                    let prepared = prepared.with_blur(amount, target.scale_factor);
                    self.blur_pass(BlurPass {
                        render_context,
                        encoder,
                        source: &textures.composition.view,
                        output: &textures.ping.view,
                        target,
                        prepared,
                        direction: [1.0, 0.0],
                    });
                    self.blur_pass(BlurPass {
                        render_context,
                        encoder,
                        source: &textures.ping.view,
                        output: &textures.pong.view,
                        target,
                        prepared,
                        direction: [0.0, 1.0],
                    });
                    self.composite_pass(CompositePass {
                        render_context,
                        encoder,
                        source: TextureSource {
                            view: &textures.pong.view,
                            area: target.physical_area.clamp_min(1),
                            logical_area: target.logical_area,
                            sampling: paint::LayerSampling::Filtered,
                        },
                        output: &textures.composition.view,
                        target,
                        prepared,
                        source_rect: prepared.shape_rect,
                        scissor,
                        labels: PassLabels::new(
                            "Filter Blur Composite Bind Group",
                            "Filter Blur Composite Vertex Buffer",
                            "Filter Blur Composite Pass",
                        ),
                    });
                }
                paint::FilterOp::BackdropBlur(blur) => {
                    if blur.sigma <= 0.0 {
                        continue;
                    }

                    let prepared = prepared.with_blur_sigma(blur.sigma, target.scale_factor);
                    self.blur_pass(BlurPass {
                        render_context,
                        encoder,
                        source: &textures.composition.view,
                        output: &textures.ping.view,
                        target,
                        prepared,
                        direction: [1.0, 0.0],
                    });
                    self.blur_pass(BlurPass {
                        render_context,
                        encoder,
                        source: &textures.ping.view,
                        output: &textures.pong.view,
                        target,
                        prepared,
                        direction: [0.0, 1.0],
                    });
                    self.composite_pass(CompositePass {
                        render_context,
                        encoder,
                        source: TextureSource {
                            view: &textures.pong.view,
                            area: target.physical_area.clamp_min(1),
                            logical_area: target.logical_area,
                            sampling: paint::LayerSampling::Filtered,
                        },
                        output: &textures.composition.view,
                        target,
                        prepared,
                        source_rect: prepared.shape_rect,
                        scissor,
                        labels: PassLabels::new(
                            "Filter Backdrop Blur Composite Bind Group",
                            "Filter Backdrop Blur Composite Vertex Buffer",
                            "Filter Backdrop Blur Composite Pass",
                        ),
                    });
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

                    self.liquid_pass(LiquidPass {
                        render_context,
                        encoder,
                        source: &textures.composition.view,
                        output: &textures.ping.view,
                        target,
                        prepared,
                        effect: liquid_effect(depth, splay, feather, curve),
                        scissor,
                    });
                    self.composite_pass(CompositePass {
                        render_context,
                        encoder,
                        source: TextureSource {
                            view: &textures.ping.view,
                            area: target.physical_area.clamp_min(1),
                            logical_area: target.logical_area,
                            sampling: paint::LayerSampling::Filtered,
                        },
                        output: &textures.composition.view,
                        target,
                        prepared,
                        source_rect: prepared.shape_rect,
                        scissor,
                        labels: PassLabels::new(
                            "Filter Liquid Composite Bind Group",
                            "Filter Liquid Composite Vertex Buffer",
                            "Filter Liquid Composite Pass",
                        ),
                    });
                }
                paint::FilterOp::Refraction(refraction) => {
                    if refraction.displacement <= 0.0 {
                        continue;
                    }

                    self.liquid_pass(LiquidPass {
                        render_context,
                        encoder,
                        source: &textures.composition.view,
                        output: &textures.ping.view,
                        target,
                        prepared,
                        effect: refraction_effect(refraction),
                        scissor,
                    });
                    self.composite_pass(CompositePass {
                        render_context,
                        encoder,
                        source: TextureSource {
                            view: &textures.ping.view,
                            area: target.physical_area.clamp_min(1),
                            logical_area: target.logical_area,
                            sampling: paint::LayerSampling::Filtered,
                        },
                        output: &textures.composition.view,
                        target,
                        prepared,
                        source_rect: prepared.shape_rect,
                        scissor,
                        labels: PassLabels::new(
                            "Filter Refraction Composite Bind Group",
                            "Filter Refraction Composite Vertex Buffer",
                            "Filter Refraction Composite Pass",
                        ),
                    });
                }
                paint::FilterOp::Luminosity(luminosity) => {
                    if luminosity.opacity <= 0.0 {
                        continue;
                    }

                    self.effect_pass(EffectPass {
                        render_context,
                        encoder,
                        source: &textures.composition.view,
                        output: &textures.ping.view,
                        target,
                        prepared,
                        effect: [
                            luminosity.color.r,
                            luminosity.color.g,
                            luminosity.color.b,
                            luminosity.opacity,
                        ],
                        pipeline: &self.luminosity_pipeline,
                        scissor,
                        labels: PassLabels::new(
                            "Filter Luminosity Bind Group",
                            "Filter Luminosity Vertex Buffer",
                            "Filter Luminosity Pass",
                        ),
                    });
                    self.composite_pass(CompositePass {
                        render_context,
                        encoder,
                        source: TextureSource {
                            view: &textures.ping.view,
                            area: target.physical_area.clamp_min(1),
                            logical_area: target.logical_area,
                            sampling: paint::LayerSampling::Filtered,
                        },
                        output: &textures.composition.view,
                        target,
                        prepared,
                        source_rect: prepared.shape_rect,
                        scissor,
                        labels: PassLabels::new(
                            "Filter Luminosity Composite Bind Group",
                            "Filter Luminosity Composite Vertex Buffer",
                            "Filter Luminosity Composite Pass",
                        ),
                    });
                }
                paint::FilterOp::Noise(noise) => {
                    if noise.opacity <= 0.0 {
                        continue;
                    }

                    self.effect_pass(EffectPass {
                        render_context,
                        encoder,
                        source: &textures.composition.view,
                        output: &textures.ping.view,
                        target,
                        prepared,
                        effect: [noise.opacity, 0.0, 0.0, 0.0],
                        pipeline: &self.noise_pipeline,
                        scissor,
                        labels: PassLabels::new(
                            "Filter Noise Bind Group",
                            "Filter Noise Vertex Buffer",
                            "Filter Noise Pass",
                        ),
                    });
                    self.composite_pass(CompositePass {
                        render_context,
                        encoder,
                        source: TextureSource {
                            view: &textures.ping.view,
                            area: target.physical_area.clamp_min(1),
                            logical_area: target.logical_area,
                            sampling: paint::LayerSampling::Filtered,
                        },
                        output: &textures.composition.view,
                        target,
                        prepared,
                        source_rect: prepared.shape_rect,
                        scissor,
                        labels: PassLabels::new(
                            "Filter Noise Composite Bind Group",
                            "Filter Noise Composite Vertex Buffer",
                            "Filter Noise Composite Pass",
                        ),
                    });
                }
            }
        }
    }

    pub(crate) fn composite_layer(&self, pass: LayerComposite<'_>) {
        let Some(prepared) = prepare_clip(pass.clip.rect, pass.target.scale_factor) else {
            return;
        };
        let source_rect =
            source_rect_for_prepared_destination(pass.clip.rect, prepared, pass.clip.rect);

        self.composite_pass(CompositePass {
            render_context: pass.render_context,
            encoder: pass.encoder,
            source: TextureSource {
                view: pass.source.view(),
                area: pass.source.area(),
                logical_area: pass.source.logical_area(),
                sampling: paint::LayerSampling::PixelAligned,
            },
            output: pass.output,
            target: pass.target,
            prepared,
            source_rect,
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
            effect: [0.0; 4],
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
            rounding: [0.0; 4],
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

    fn ensure_textures(&mut self, render_context: &render::Context, area: paint::area::Physical) {
        if self
            .textures
            .as_ref()
            .is_some_and(|textures| textures.area == area)
        {
            return;
        }

        self.textures = Some(Textures {
            area,
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
        }
    }

    fn blur_pass(&self, pass: BlurPass<'_>) {
        let params = self.params(pass.target, pass.prepared, pass.direction);
        let bind_group = self.bind_group(
            pass.render_context,
            pass.source,
            params,
            paint::LayerSampling::Filtered,
            "Filter Blur Bind Group",
        );
        let mut render_pass = pass.encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("Filter Blur Pass"),
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
            texture_area: pass.target.physical_area.clamp_min(1),
            texture_logical_area: pass.target.logical_area,
            prepared: pass.prepared,
            source_rect: pass.prepared.shape_rect,
            direction: [0.0, 0.0],
            effect: pass.effect,
            sampling: paint::LayerSampling::Filtered,
        });
        let bind_group = self.bind_group(
            pass.render_context,
            pass.source,
            params,
            paint::LayerSampling::Filtered,
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
            texture_area: pass.target.physical_area.clamp_min(1),
            texture_logical_area: pass.target.logical_area,
            prepared: pass.prepared,
            source_rect: pass.prepared.shape_rect,
            direction: [0.0, 0.0],
            effect: pass.effect,
            sampling: paint::LayerSampling::Filtered,
        });
        let bind_group = self.bind_group(
            pass.render_context,
            pass.source,
            params,
            paint::LayerSampling::Filtered,
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
            effect: [0.0; 4],
            sampling: pass.source.sampling,
        });
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

    fn params(&self, target: Target, prepared: PreparedFilter, direction: [f32; 2]) -> Params {
        self.params_with_texture_area(ParamInput {
            target_scale_factor: target.scale_factor,
            texture_area: target.physical_area.clamp_min(1),
            texture_logical_area: target.logical_area,
            prepared,
            source_rect: prepared.shape_rect,
            direction,
            effect: [prepared.blur_sigma_px, 0.0, 0.0, 0.0],
            sampling: paint::LayerSampling::Filtered,
        })
    }

    fn params_with_texture_area(&self, input: ParamInput) -> Params {
        let physical_area = input.texture_area.clamp_min(1);

        let source_rect_data = physical_source_rect_data(
            input.source_rect,
            input.texture_logical_area,
            physical_area,
            input.target_scale_factor,
            input.sampling,
        );
        let source_scale = source_step_data(
            source_rect_data,
            input.prepared.shape_rect.area,
            input.target_scale_factor,
            input.sampling,
        );

        Params {
            texture_size: [physical_area.width() as f32, physical_area.height() as f32],
            source_scale,
            direction_radius: [
                input.direction[0],
                input.direction[1],
                input.prepared.blur_radius_px,
                0.0,
            ],
            effect: input.effect,
            rect: rect_data(input.prepared.shape_rect),
            source_rect: source_rect_data,
            rounding: rounding_data(input.prepared.rounding),
        }
    }
}

pub(crate) fn shader_source() -> String {
    silhouette::wgsl_module_source(FILTER_WGSL)
}

impl Layer {
    pub fn view(&self) -> &wgpu::TextureView {
        &self.texture.view
    }

    pub fn area(&self) -> paint::area::Physical {
        self.area
    }

    pub fn logical_area(&self) -> paint::area::Logical {
        self.logical_area
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
    (amount.clamp(0.0, 1.0) * Renderer::MAX_BLUR_RADIUS_PX * scale_factor)
        .clamp(0.0, Renderer::MAX_BLUR_RADIUS_PX)
}

fn blur_sigma_px(sigma: f32, scale_factor: f32) -> f32 {
    sigma.max(0.0) * scale_factor.max(0.0001)
}

fn blur_kernel_radius_px(sigma: f32, scale_factor: f32) -> f32 {
    (blur_sigma_px(sigma, scale_factor) * 3.0)
        .ceil()
        .clamp(0.0, Renderer::MAX_BLUR_RADIUS_PX)
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

fn physical_source_rect_data(
    source_rect: Rect,
    texture_logical_area: paint::area::Logical,
    texture_physical_area: paint::area::Physical,
    target_scale_factor: f32,
    sampling: paint::LayerSampling,
) -> [f32; 4] {
    let [x_scale, y_scale] = match sampling {
        paint::LayerSampling::PixelAligned => [target_scale_factor, target_scale_factor],
        paint::LayerSampling::Filtered => {
            source_scale_data(texture_logical_area, texture_physical_area)
        }
    };

    [
        (source_rect.origin.x() * x_scale).round(),
        (source_rect.origin.y() * y_scale).round(),
        (source_rect.area.width() * x_scale).round().max(1.0),
        (source_rect.area.height() * y_scale).round().max(1.0),
    ]
}

fn source_scale_data(
    texture_logical_area: paint::area::Logical,
    texture_physical_area: paint::area::Physical,
) -> [f32; 2] {
    [
        texture_physical_area.width() as f32 / texture_logical_area.width().max(1.0),
        texture_physical_area.height() as f32 / texture_logical_area.height().max(1.0),
    ]
}

fn source_step_data(
    source_rect_data: [f32; 4],
    destination_area: paint::area::Logical,
    target_scale_factor: f32,
    sampling: paint::LayerSampling,
) -> [f32; 2] {
    match sampling {
        paint::LayerSampling::PixelAligned => [target_scale_factor, target_scale_factor],
        paint::LayerSampling::Filtered => [
            source_rect_data[2] / destination_area.width().max(1.0),
            source_rect_data[3] / destination_area.height().max(1.0),
        ],
    }
}

fn create_noise_texture(render_context: &render::Context) -> Texture {
    let extent = wgpu::Extent3d {
        width: Renderer::NOISE_TEXTURE_SIZE,
        height: Renderer::NOISE_TEXTURE_SIZE,
        depth_or_array_layers: 1,
    };
    let texture = render_context
        .device()
        .create_texture(&wgpu::TextureDescriptor {
            label: Some("Filter Noise Texture"),
            size: extent,
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8Unorm,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
            view_formats: &[],
        });
    let bytes = noise_texture_bytes();
    render_context.queue().write_texture(
        wgpu::TexelCopyTextureInfo {
            texture: &texture,
            mip_level: 0,
            origin: wgpu::Origin3d::ZERO,
            aspect: wgpu::TextureAspect::All,
        },
        &bytes,
        wgpu::TexelCopyBufferLayout {
            offset: 0,
            bytes_per_row: Some(Renderer::NOISE_TEXTURE_SIZE * 4),
            rows_per_image: Some(Renderer::NOISE_TEXTURE_SIZE),
        },
        extent,
    );
    let view = texture.create_view(&wgpu::TextureViewDescriptor::default());

    Texture {
        _inner: texture,
        view,
    }
}

fn noise_texture_bytes() -> Vec<u8> {
    let size = Renderer::NOISE_TEXTURE_SIZE as usize;
    let mut bytes = Vec::with_capacity(size * size * 4);

    for y in 0..Renderer::NOISE_TEXTURE_SIZE {
        for x in 0..Renderer::NOISE_TEXTURE_SIZE {
            let value = noise_texel(x, y);
            bytes.extend_from_slice(&[value, value, value, u8::MAX]);
        }
    }

    bytes
}

fn noise_texel(x: u32, y: u32) -> u8 {
    let hash = hash_noise_texel(x, y);
    let centered = hash as i16 - 128;
    let value = 128 + centered * 35 / 100;

    value.clamp(0, u8::MAX as i16) as u8
}

fn hash_noise_texel(x: u32, y: u32) -> u8 {
    let mut value = x.wrapping_mul(0x9e37_79b9).rotate_left(7)
        ^ y.wrapping_mul(0x85eb_ca6b).rotate_left(13)
        ^ Renderer::NOISE_SEED;

    value ^= value >> 16;
    value = value.wrapping_mul(0x7feb_352d);
    value ^= value >> 15;
    value = value.wrapping_mul(0x846c_a68b);
    value ^= value >> 16;

    (value >> 24) as u8
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
        let bytes = noise_texture_bytes();

        assert_eq!(
            bytes.len(),
            (Renderer::NOISE_TEXTURE_SIZE * Renderer::NOISE_TEXTURE_SIZE * 4) as usize
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
        let bytes = noise_texture_bytes();
        let values = bytes.chunks_exact(4).map(|rgba| rgba[0]);
        let min = values.clone().min().expect("noise should have texels");
        let max = values.max().expect("noise should have texels");

        assert_eq!(min, 84);
        assert_eq!(max, 172);
    }

    #[test]
    fn noise_texel_values_are_seed_stable() {
        assert_eq!(noise_texel(0, 0), 152);
        assert_eq!(noise_texel(1, 0), 166);
        assert_eq!(noise_texel(0, 1), 106);
        assert_eq!(noise_texel(17, 31), 167);
        assert_eq!(noise_texel(127, 127), 143);
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
}
