use crate::paint;
use crate::render;

mod chain;
mod draw;
mod effects;
mod encode;
mod geometry;
mod layer;
mod noise;
mod params;
mod pass;
mod resources;
mod setup;
mod shader;
mod source;
mod state;
mod storage;
mod target;

use chain::FilterChainContext;
pub(crate) use chain::FilterSource;
pub(crate) use draw::FilterDraw;
use effects::{liquid_effect, liquid_is_identity, refraction_effect};
pub(crate) use encode::shader_source;
use geometry::PreparedFilter;
#[cfg(test)]
pub(crate) use geometry::{prepared_clip_silhouette_for_test, prepared_filter_silhouette_for_test};
use params::{AlphaMode, ParamInput, Params};
use pass::{
    BlurLabels, BlurPass, CompositePass, CompositeVertex, EffectPass, LiquidPass, PassLabels,
    composite_vertices,
};
pub(crate) use source::TextureSource;
pub(crate) use state::Renderer;
pub(crate) use storage::Layer;
pub(crate) use storage::LayerComposite;
use storage::{ScratchTargets, Textures, take_pooled_layer, take_pooled_scratch};
pub(crate) use target::Target;

#[cfg(test)]
mod tests;
