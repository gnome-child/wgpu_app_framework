mod frame;
mod pipeline;
mod render;
mod samples;
mod scroll;
mod store;

pub use crate::layout::Text;
pub use crate::render::RenderReport;
pub use frame::Frame;
pub use pipeline::Pipeline;
pub use render::Render;
pub use scroll::Scroll;

pub(crate) use store::Store;

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Diagnostics {
    pub text: Text,
    pub scroll: Scroll,
    pub frame: Frame,
    pub pipeline: Pipeline,
    pub render: Render,
}
