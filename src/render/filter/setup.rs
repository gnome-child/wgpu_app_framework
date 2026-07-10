use std::cell::RefCell;

use crate::render;

use super::encode::shader_source;
use super::noise;
use super::pass::CompositeVertex;
use super::state::Renderer;

impl Renderer {
    pub(in crate::render) fn new(
        render_context: &render::Context,
        format: wgpu::TextureFormat,
    ) -> Self {
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
                        targets: &[Some(render::alpha::color_target(
                            format,
                            render::alpha::FragmentOutput::Replace,
                        ))],
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
                        targets: &[Some(render::alpha::color_target(
                            format,
                            render::alpha::FragmentOutput::Replace,
                        ))],
                        compilation_options: Default::default(),
                    }),
                    multiview_mask: None,
                    cache: None,
                });
        let refraction_pipeline =
            render_context
                .device()
                .create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                    label: Some("Filter Refraction Pipeline"),
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
                        entry_point: Some("fs_refraction"),
                        targets: &[Some(render::alpha::color_target(
                            format,
                            render::alpha::FragmentOutput::Replace,
                        ))],
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
                        targets: &[Some(render::alpha::color_target(
                            format,
                            render::alpha::FragmentOutput::Replace,
                        ))],
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
                        targets: &[Some(render::alpha::color_target(
                            format,
                            render::alpha::FragmentOutput::Replace,
                        ))],
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
                        targets: &[Some(render::alpha::color_target(
                            format,
                            render::alpha::FragmentOutput::Premultiplied,
                        ))],
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
                        targets: &[Some(render::alpha::color_target(
                            format,
                            render::alpha::FragmentOutput::Premultiplied,
                        ))],
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
            refraction_pipeline,
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
}
