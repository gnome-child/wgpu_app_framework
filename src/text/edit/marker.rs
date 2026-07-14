use super::super::buffer::{self, Buffer, Mark, Position, Selection};
use super::super::selection::State;

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct Marker {
    pub(crate) buffer_id: u64,
    pub(crate) revision: u64,
    pub(crate) cursor: Mark,
    pub(crate) selection: Option<buffer::MarkRange>,
    cursor_position: Position,
    selection_positions: Option<Selection>,
}

impl Marker {
    pub(super) fn new(buffer: &Buffer, state: State) -> Self {
        let inner = &buffer.inner;
        Self {
            buffer_id: inner.id,
            revision: inner.revision,
            cursor: state.cursor,
            selection: state.selection,
            cursor_position: inner
                .document
                .position_for_mark(state.cursor)
                .unwrap_or_else(|| Position::new(inner.document.text_len())),
            selection_positions: state
                .selection
                .and_then(|selection| inner.document.selection_for_mark_range(selection)),
        }
    }

    pub(super) fn cursor_for(&self, buffer: &Buffer) -> Mark {
        let inner = &buffer.inner;
        if inner.document.position_for_mark(self.cursor).is_some() {
            self.cursor
        } else {
            inner.document.mark_for_position(self.cursor_position)
        }
    }

    pub(super) fn selection_for(&self, buffer: &Buffer) -> Option<buffer::MarkRange> {
        let inner = &buffer.inner;
        if let Some(selection) = self.selection
            && inner.document.position_for_mark(selection.start).is_some()
            && inner.document.position_for_mark(selection.end).is_some()
        {
            return Some(selection);
        }
        self.selection_positions
            .map(|selection| inner.document.mark_range_for_selection(selection))
    }
}
