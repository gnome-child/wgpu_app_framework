mod action;
mod buffer;
mod clipboard;
mod diagnostics;
mod editor;
mod history;
mod marker;
mod operation;
mod outcome;
mod transaction;

pub use action::{Action, ActionResult};
pub use clipboard::{Clipboard, ClipboardError, ClipboardResult};
pub use diagnostics::Diagnostics;
pub use editor::Editor;
pub use history::{History, HistoryKind, TYPING_UNDO_COALESCE_WINDOW};
pub(crate) use marker::Marker;
pub use operation::{Edit, PointerEditKind};
pub use outcome::Outcome;
#[cfg(test)]
pub(crate) use transaction::Kind as TransactionKind;
