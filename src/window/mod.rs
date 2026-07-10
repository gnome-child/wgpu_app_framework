mod departed;
mod facts;
mod id;
mod options;

pub use departed::Departed;
pub use facts::Facts;
pub use id::Id;
pub use options::{Kind, Options};

use super::{scene, theme};

pub const DEFAULT_TITLE: &str = "Window";
pub const DEFAULT_CANVAS_COLOR: scene::Color = theme::DEFAULT_CANVAS_COLOR;

pub(crate) const DEFAULT_WIDTH: i32 = 800;
pub(crate) const DEFAULT_HEIGHT: i32 = 600;
