mod frame;
mod scroll;
mod store;
mod text;

pub use frame::Frame;
pub use scroll::Scroll;
pub use text::Text;

pub(in crate::scratch) use store::Store;

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Diagnostics {
    pub text: Text,
    pub scroll: Scroll,
    pub frame: Frame,
}
