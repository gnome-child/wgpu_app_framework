pub mod buffer;
mod color;
pub mod document;
pub mod edit;
pub mod layout;
mod unicode;

pub use buffer::Buffer;
pub use color::Color;
pub use document::Document;
#[cfg(test)]
mod tests;
