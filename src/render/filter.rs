use std::cell::RefCell;

use wgpu::util::DeviceExt;

use crate::paint;
use crate::render;

mod chain;
mod draw;
mod effects;
mod geometry;
mod noise;
mod params;
mod pass;
mod shader;
mod source;
mod storage;
mod target;

use chain::FilterChainContext;
pub(crate) use chain::FilterSource;
pub(crate) use draw::FilterDraw;
#[cfg(test)]
use effects::liquid_depth_displacement;
use effects::{liquid_effect, liquid_is_identity, refraction_effect};
use geometry::{
    PreparedFilter, prepare_clip, prepare_filter, source_rect_for_prepared_destination,
};
#[cfg(test)]
use geometry::{blur_kernel_radius_px, blur_radius_px, blur_sigma_px};
#[cfg(test)]
pub(crate) use geometry::{prepared_clip_silhouette_for_test, prepared_filter_silhouette_for_test};
use params::{AlphaMode, ParamInput, Params};
use pass::{
    BlurLabels, BlurPass, CompositePass, CompositeVertex, EffectPass, LiquidPass, PassLabels,
    composite_vertices,
};
pub(crate) use source::TextureSource;
pub use storage::Layer;
pub(crate) use storage::LayerComposite;
use storage::{
    ScratchTargets, ScratchTextures, Texture, Textures, take_pooled_layer, take_pooled_scratch,
};
pub use target::Target;

#[cfg(test)]
use crate::paint::Rect;
#[cfg(test)]
use crate::render::silhouette::edges;
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

impl Renderer {
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

    pub(crate) fn layer_pool_entries(&self) -> usize {
        self.layer_pool.borrow().len()
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

    pub(crate) fn scratch_pool_entries(&self) -> usize {
        self.scratch_pool.borrow().len()
    }

    pub fn clear_layer(&self, encoder: &mut wgpu::CommandEncoder, layer: &Layer) {
        clear_view(
            encoder,
            layer.view(),
            wgpu::Color::TRANSPARENT,
            "Layer Clear Pass",
        );
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

pub(crate) fn shader_source() -> String {
    shader::module_source()
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
mod tests;
