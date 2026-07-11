pub mod buffer;
mod color;
pub mod document;
pub mod edit;
pub mod layout;
mod overflow;
mod unicode;

pub use buffer::Buffer;
pub use color::Color;
pub use document::Document;
pub use overflow::Overflow;
#[cfg(test)]
mod acceptance;
#[cfg(test)]
mod tests;
