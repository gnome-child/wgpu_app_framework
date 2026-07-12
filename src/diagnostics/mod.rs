mod draw;
mod frame;
mod pipeline;
mod render;
mod samples;
mod scroll;
mod store;
mod text;

pub use frame::Frame;
pub use pipeline::Pipeline;
pub use render::{Render, Report as RenderReport};
pub use scroll::Scroll;
pub use text::Text;

pub(crate) use draw::DrawStats;
pub(crate) use store::Store;

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Diagnostics {
    pub text: Text,
    pub scroll: Scroll,
    pub frame: Frame,
    pub pipeline: Pipeline,
    pub render: Render,
}
