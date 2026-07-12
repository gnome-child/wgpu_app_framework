mod action;
mod buffer;
mod caret;
mod clipboard;
mod diagnostics;
mod editor;
mod history;
mod marker;
mod motion;
mod operation;
mod outcome;
mod state;
mod surface;
mod transaction;
mod view;

pub use action::{Action, ActionResult};
pub use caret::CaretMap;
pub use clipboard::{Clipboard, ClipboardError, ClipboardResult};
pub use diagnostics::Diagnostics;
pub use editor::Editor;
pub use history::{History, HistoryKind, TYPING_UNDO_COALESCE_WINDOW};
pub(crate) use marker::Marker;
pub use motion::Motion;
pub use operation::{Edit, PointerEditKind};
pub use outcome::Outcome;
pub use state::State;
pub use surface::{Area, AreaWrap, Field, FieldMode, Obscuring, Surface};
pub(crate) use surface::{
    FieldProjection, PositionMap, PreeditProjection, projected_state_for_field,
};
#[cfg(test)]
pub(crate) use transaction::Impact;
#[cfg(test)]
pub(crate) use transaction::Kind as TransactionKind;
pub use view::{
    ObservedArea, Preedit, RevealIntent, ScrollAnchor, View, ViewState, Viewport, Visibility,
};
