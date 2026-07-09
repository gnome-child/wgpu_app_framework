use crate::paint;
use crate::render;

use super::params::{self, AlphaMode, ParamInput, Params};
use super::pass::{BlurPass, CompositePass, EffectPass, LiquidPass, composite_vertices};
use super::shader;
use super::state::Renderer;
use wgpu::util::DeviceExt;

impl Renderer {
    pub(super) fn blur_pass(&self, pass: BlurPass<'_>) {
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

    pub(super) fn liquid_pass(&self, pass: LiquidPass<'_>) {
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

    pub(super) fn effect_pass(&self, pass: EffectPass<'_>) {
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

    pub(super) fn composite_pass(&self, pass: CompositePass<'_>) {
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

    pub(super) fn bind_group(
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

pub(super) fn clear_view(
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
