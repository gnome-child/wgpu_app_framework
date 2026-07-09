use std::cell::RefCell;

use super::Layer;
use super::storage::{ScratchTextures, Texture, Textures};

pub(in crate::render) struct Renderer {
    pub(super) blur_pipeline: wgpu::RenderPipeline,
    pub(super) refraction_pipeline: wgpu::RenderPipeline,
    pub(super) luminosity_pipeline: wgpu::RenderPipeline,
    pub(super) noise_pipeline: wgpu::RenderPipeline,
    pub(super) blit_pipeline: wgpu::RenderPipeline,
    pub(super) composite_pipeline: wgpu::RenderPipeline,
    pub(super) pixel_composite_pipeline: wgpu::RenderPipeline,
    pub(super) bind_group_layout: wgpu::BindGroupLayout,
    pub(super) filtered_sampler: wgpu::Sampler,
    pub(super) pixel_aligned_sampler: wgpu::Sampler,
    pub(super) noise_texture: Texture,
    pub(super) noise_sampler: wgpu::Sampler,
    pub(super) textures: Option<Textures>,
    pub(super) layer_pool: RefCell<Vec<Layer>>,
    pub(super) scratch_pool: RefCell<Vec<ScratchTextures>>,
    pub(super) format: wgpu::TextureFormat,
}
