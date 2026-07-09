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
mod resources;
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

#[cfg(test)]
mod tests;
