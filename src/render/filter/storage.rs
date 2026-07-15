use crate::geometry::area;
use crate::{paint, render};

use super::{Renderer, Target, TextureSource};

pub(in crate::render) struct Layer {
    pub(super) texture: Texture,
    pub(super) area: area::Physical,
    pub(super) logical_area: area::Logical,
}

pub(super) struct Textures {
    pub(super) area: area::Physical,
    pub(super) logical_area: area::Logical,
    pub(super) composition: Texture,
    pub(super) ping: Texture,
    pub(super) pong: Texture,
}

pub(super) struct Texture {
    pub(super) _inner: wgpu::Texture,
    pub(super) view: wgpu::TextureView,
    pub(super) area: area::Physical,
}

pub(super) struct ScratchTextures {
    ping: Texture,
    pong: Texture,
    pub(super) area: area::Physical,
    logical_area: area::Logical,
}

pub(super) enum ScratchTargets<'a> {
    Shared {
        ping: &'a Texture,
        pong: &'a Texture,
        logical_area: area::Logical,
    },
    Pooled(ScratchTextures),
}

pub(in crate::render) struct LayerComposite<'a> {
    pub(in crate::render) render_context: &'a render::Context,
    pub(in crate::render) encoder: &'a mut wgpu::CommandEncoder,
    pub(in crate::render) source: &'a Layer,
    pub(in crate::render) output: &'a wgpu::TextureView,
    pub(in crate::render) target: Target,
    pub(in crate::render) clip: paint::Clip,
    pub(in crate::render) source_rect: Option<paint::Rect>,
    pub(in crate::render) scissor: Option<render::Scissor>,
    pub(in crate::render) opacity: f32,
}

impl Texture {
    pub(super) fn source(
        &self,
        logical_area: area::Logical,
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

impl ScratchTextures {
    pub(super) fn new(
        render_context: &render::Context,
        renderer: &Renderer,
        target: Target,
    ) -> Self {
        let area = target.physical_area.clamp_min(1);

        Self {
            ping: renderer.create_texture(render_context, area, "Filter Scratch Ping Texture"),
            pong: renderer.create_texture(render_context, area, "Filter Scratch Pong Texture"),
            area,
            logical_area: area.to_logical(target.scale_factor),
        }
    }

    pub(super) fn retarget(&mut self, target: Target) {
        debug_assert_eq!(self.area, target.physical_area.clamp_min(1));
        self.logical_area = self.area.to_logical(target.scale_factor);
    }
}

impl<'a> ScratchTargets<'a> {
    pub(super) fn ping_view(&self) -> &wgpu::TextureView {
        match self {
            Self::Shared { ping, .. } => &ping.view,
            Self::Pooled(scratch) => &scratch.ping.view,
        }
    }

    pub(super) fn pong_view(&self) -> &wgpu::TextureView {
        match self {
            Self::Shared { pong, .. } => &pong.view,
            Self::Pooled(scratch) => &scratch.pong.view,
        }
    }

    pub(super) fn ping_source(&self, sampling: paint::LayerSampling) -> TextureSource<'_> {
        match self {
            Self::Shared {
                ping, logical_area, ..
            } => ping.source(*logical_area, sampling),
            Self::Pooled(scratch) => scratch.ping.source(scratch.logical_area, sampling),
        }
    }

    pub(super) fn pong_source(&self, sampling: paint::LayerSampling) -> TextureSource<'_> {
        match self {
            Self::Shared {
                pong, logical_area, ..
            } => pong.source(*logical_area, sampling),
            Self::Pooled(scratch) => scratch.pong.source(scratch.logical_area, sampling),
        }
    }
}

impl Layer {
    pub(in crate::render) fn view(&self) -> &wgpu::TextureView {
        &self.texture.view
    }

    pub(super) fn source(&self, sampling: paint::LayerSampling) -> TextureSource<'_> {
        debug_assert_eq!(self.texture.area, self.area);
        TextureSource {
            view: self.view(),
            area: self.area,
            logical_area: self.logical_area,
            sampling,
        }
    }
}

pub(super) fn take_pooled_layer(pool: &mut Vec<Layer>, area: area::Physical) -> Option<Layer> {
    let position = pool.iter().position(|layer| layer.area == area)?;
    Some(pool.swap_remove(position))
}

pub(super) fn take_pooled_scratch(
    pool: &mut Vec<ScratchTextures>,
    area: area::Physical,
) -> Option<ScratchTextures> {
    let position = pool.iter().position(|scratch| scratch.area == area)?;
    Some(pool.swap_remove(position))
}
