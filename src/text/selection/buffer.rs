use super::super::buffer::{
    Buffer, Cursor, CursorSelection, Mark, MarkRange, Position, Range, Selection,
};
use super::State;

impl Buffer {
    pub fn initial_state(&self) -> State {
        State::collapsed(document_end_mark(self))
    }

    pub fn position_for_state(&self, state: State) -> Position {
        let inner = &self.inner;
        inner
            .document
            .position_for_mark(state.cursor)
            .unwrap_or_else(|| Position::new(inner.document.text_len()))
    }

    pub fn selection_for_state(&self, state: State) -> Option<Selection> {
        let inner = &self.inner;
        state
            .selection
            .and_then(|selection| inner.document.selection_for_mark_range(selection))
    }

    pub fn cursor_for_state(&self, state: State) -> Cursor {
        let inner = &self.inner;
        inner
            .document
            .cursor_for_mark(state.cursor)
            .unwrap_or_else(|| {
                inner
                    .document
                    .cursor_for_text_index(inner.document.text_len())
            })
    }

    pub fn selection_bounds_for_state(&self, state: State) -> Option<(Cursor, Cursor)> {
        let inner = &self.inner;
        let selection = state.selection?;
        let (start, end) = inner
            .document
            .ordered_cursor_range_for_mark_range(selection)?;
        (inner.document.text_index_for_cursor(start) < inner.document.text_index_for_cursor(end))
            .then_some((start, end))
    }

    pub fn selected_range_for_state(&self, state: State) -> Option<Range> {
        let inner = &self.inner;
        let selection = state.selection?;
        let start = inner.document.position_for_mark(selection.start)?.index;
        let end = inner.document.position_for_mark(selection.end)?.index;
        let (start, end) = if start <= end {
            (start, end)
        } else {
            (end, start)
        };
        (start < end).then_some(Range::new(start, end))
    }

    pub fn selected_text_for_state(&self, state: State) -> Option<String> {
        let range = self.selected_range_for_state(state)?.as_range();
        Some(self.inner.document.text_for_range(range))
    }

    pub fn has_selection_for_state(&self, state: State) -> bool {
        self.has_non_empty_selection_for_state(state)
    }

    pub(crate) fn has_non_empty_selection_for_state(&self, state: State) -> bool {
        self.selected_range_for_state(state).is_some()
    }

    pub(crate) fn set_cursor_and_selection_for_state(
        &self,
        state: &mut State,
        cursor: Cursor,
        selection: CursorSelection,
    ) {
        let inner = &self.inner;
        let cursor = inner.document.mark_for_cursor(cursor);
        let selection = selection_mark_for_document(self, selection).map(|anchor| MarkRange {
            start: anchor,
            end: cursor,
        });
        *state = State::new(cursor, selection);
    }
}

fn selection_mark_for_document(buffer: &Buffer, selection: CursorSelection) -> Option<Mark> {
    let inner = &buffer.inner;
    match selection {
        CursorSelection::None => None,
        CursorSelection::Normal(cursor) => Some(inner.document.mark_for_cursor(cursor)),
    }
}

pub(crate) fn selection_mark_from_state(buffer: &Buffer, state: State) -> Option<Cursor> {
    let inner = &buffer.inner;
    state
        .selection
        .and_then(|selection| inner.document.cursor_for_mark(selection.start))
}

fn document_end_mark(buffer: &Buffer) -> Mark {
    let inner = &buffer.inner;
    inner.document.mark_for_cursor(
        inner
            .document
            .cursor_for_text_index(inner.document.text_len()),
    )
}
