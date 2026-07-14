mod departed;
mod facts;
mod id;
mod options;
mod presentation_epoch;

pub use departed::Departed;
pub use facts::Facts;
pub use id::Id;
pub use options::{Kind, Options};
pub(crate) use presentation_epoch::PresentationEpoch;

use super::{color, theme};

pub const DEFAULT_TITLE: &str = "Window";
pub const DEFAULT_CANVAS_COLOR: color::Color = theme::DEFAULT_CANVAS_COLOR;

pub(crate) const DEFAULT_WIDTH: i32 = 800;
pub(crate) const DEFAULT_HEIGHT: i32 = 600;
