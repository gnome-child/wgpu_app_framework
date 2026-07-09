use std::cell::RefCell;

use crate::paint;
use crate::render;

mod chain;
mod draw;
mod effects;
mod encode;
mod geometry;
mod noise;
mod params;
mod pass;
mod setup;
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
use encode::clear_view;
pub(crate) use encode::shader_source;
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
}

#[cfg(test)]
mod tests;
