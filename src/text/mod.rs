pub mod buffer;
mod color;
pub mod document;
pub mod edit;
mod input;
pub mod layout;
mod overflow;
pub mod preedit;
pub mod selection;
pub mod surface;
mod unicode;
pub mod view;

pub use buffer::Buffer;
pub use color::Color;
pub use document::Document;
pub use edit::Edit;
pub(crate) use input::Decision as InputDecision;
pub use input::Input;
pub use overflow::Overflow;
pub use preedit::Preedit;
pub use surface::Surface;
pub use view::View;
#[cfg(test)]
mod acceptance;
#[cfg(test)]
mod tests;
