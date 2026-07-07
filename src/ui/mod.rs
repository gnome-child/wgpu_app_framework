pub mod drag_drop;
pub mod floating;
pub mod focus;
pub mod layout;
pub mod scroll;

mod event;
mod filter;
mod node;
mod painting;
mod popup;
mod snapshot;
mod tree;
mod visual;

pub use crate::action::{
    Binding as ActionBinding, Key as ActionKey, Route as ActionRoute, State as ActionState,
    Subject as ActionSubject, Target as ActionTarget,
};
pub use crate::input::{Key, Modifiers};
pub use crate::path::{Id, Path};
pub use event::Event;
pub use filter::Filter;
pub use node::{Cursor, Interaction, Interactivity, Layout, Node, Shadow, Style};
pub(crate) use node::{CursorOverlay, CursorOverlayText, Intent};
pub(crate) use painting::{ScrollPaintRecord, ScrollPaintRecords};
pub use popup::Popup;
pub use visual::VisualState;
pub type Frame = layout::Frame<Path>;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Metrics {
    Scroll(scroll::Metrics),
}

impl Metrics {
    pub fn scroll(self) -> Option<scroll::Metrics> {
        match self {
            Self::Scroll(metrics) => Some(metrics),
        }
    }
}
#[doc(hidden)]
pub use tree::Composition;
pub use tree::Tree;

#[cfg(test)]
mod tests;
