pub mod buffer;
mod color;
pub mod document;
pub mod edit;
mod input;
pub mod layout;
mod overflow;
mod unicode;

pub use buffer::Buffer;
pub use color::Color;
pub use document::Document;
pub(crate) use input::Decision as InputDecision;
pub use input::Input;
pub use overflow::Overflow;
#[cfg(test)]
mod acceptance;
#[cfg(test)]
mod tests;
