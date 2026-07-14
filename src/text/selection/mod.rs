mod buffer;
mod caret;
mod motion;
mod operation;
mod state;

pub use caret::CaretMap;
pub use motion::Motion;
pub use operation::{Operation, PointerKind, apply};
pub use state::State;

pub(crate) use buffer::selection_mark_from_state;
pub(crate) use motion::{
    collapsed_cursor_for_motion, text_position_for_motion_in_document_for_state,
};
pub(crate) use operation::apply_with_caret_map;
