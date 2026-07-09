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

pub(in crate::render) use chain::FilterSource;
pub(in crate::render) use draw::FilterDraw;
#[cfg(test)]
pub(in crate::render) use encode::shader_source;
#[cfg(test)]
pub(in crate::render) use geometry::{
    prepared_clip_silhouette_for_test, prepared_filter_silhouette_for_test,
};
pub(in crate::render) use source::TextureSource;
pub(in crate::render) use state::Renderer;
pub(in crate::render) use storage::Layer;
pub(in crate::render) use storage::LayerComposite;
pub(in crate::render) use target::Target;

#[cfg(test)]
mod tests;
