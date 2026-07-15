use crate::geometry::area;
use crate::paint;
use crate::render;

use super::encode::clear_view;
use super::params::{AlphaMode, Params};
use super::state::Renderer;
use super::storage::{
    Layer, ScratchTargets, ScratchTextures, Texture, Textures, take_pooled_layer,
    take_pooled_scratch,
};
use super::target::Target;

const LAYER_POOL_LIMIT: usize = 8;
const SCRATCH_POOL_LIMIT: usize = 8;

impl Renderer {
    pub(in crate::render) fn prepare(
        &mut self,
        render_context: &render::Context,
        canvas: &render::Canvas,
    ) -> Target {
        let target = Target::new(canvas);
        self.prepare_target(render_context, target);
        target
    }

    pub(in crate::render) fn prepare_target(
        &mut self,
        render_context: &render::Context,
        target: Target,
    ) {
        self.ensure_textures(
            render_context,
            target.physical_area.clamp_min(1),
            target.logical_area,
        );
    }

    pub(in crate::render) fn composition_view(&self) -> Option<&wgpu::TextureView> {
        Some(&self.textures.as_ref()?.composition.view)
    }

    pub(in crate::render) fn clear_composition(
        &self,
        encoder: &mut wgpu::CommandEncoder,
        clear_color: wgpu::Color,
    ) {
        let Some(view) = self.composition_view() else {
            return;
        };
        clear_view(encoder, view, clear_color, "Composition Clear Pass");
    }

    pub(in crate::render) fn create_layer(
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

    pub(in crate::render) fn recycle_layer(&self, layer: Layer) {
        let mut pool = self.layer_pool.borrow_mut();
        if pool.len() == LAYER_POOL_LIMIT {
            pool.remove(0);
        }
        pool.push(layer);
    }

    pub(in crate::render) fn layer_pool_entries(&self) -> usize {
        self.layer_pool.borrow().len()
    }

    pub(super) fn scratch_targets<'a>(
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

    pub(super) fn recycle_scratch(&self, scratch: ScratchTextures) {
        let mut pool = self.scratch_pool.borrow_mut();
        if pool.len() == SCRATCH_POOL_LIMIT {
            pool.remove(0);
        }
        pool.push(scratch);
    }

    pub(in crate::render) fn scratch_pool_entries(&self) -> usize {
        self.scratch_pool.borrow().len()
    }

    pub(in crate::render) fn clear_layer(&self, encoder: &mut wgpu::CommandEncoder, layer: &Layer) {
        clear_view(
            encoder,
            layer.view(),
            wgpu::Color::TRANSPARENT,
            "Layer Clear Pass",
        );
    }

    pub(in crate::render) fn blit_to_view(
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
        area: area::Physical,
        logical_area: area::Logical,
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

    pub(super) fn create_texture(
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
            area,
        }
    }
}
