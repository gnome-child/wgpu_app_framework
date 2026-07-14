mod buffer;
mod caret;
mod motion;
mod state;

pub use caret::CaretMap;
pub use motion::Motion;
pub use state::State;

pub(crate) use buffer::{document_end_mark, selection_mark_from_state};
pub(crate) use motion::{
    collapsed_cursor_for_motion, text_position_for_motion_in_document_for_state,
};
