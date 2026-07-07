use super::super::buffer::{Mark, MarkRange};
use super::super::{
    buffer::{Buffer, Cursor, CursorSelection, Position, Range, Selection},
    unicode::normalize_for_mode,
};
use super::{
    Marker, State,
    marker::document_end_mark,
    transaction::{Impact, Kind, Transaction},
};

impl Buffer {
    pub fn initial_state(&self) -> State {
        State::collapsed(document_end_mark(self))
    }

    pub(crate) fn marker_for_state(&self, state: State) -> Marker {
        Marker::new(self, state)
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
        let cursor = inner
            .document
            .mark_for_cursor(cursor)
            .unwrap_or_else(|| document_end_mark(self));
        let selection = selection_mark_for_document(self, selection).map(|anchor| MarkRange {
            start: anchor,
            end: cursor,
        });
        *state = State::new(cursor, selection);
    }

    pub(crate) fn replace_text_range_with_kind_and_impact_for_state(
        &mut self,
        state: &mut State,
        range: Range,
        text: &str,
        kind: Kind,
    ) -> (Transaction, Option<Impact>) {
        let inserted = normalize_for_mode(self.is_multiline(), text);
        let range = {
            let inner = &self.inner;
            inner.document.snap_range(range)
        };
        if range.is_empty() && inserted.is_empty() {
            return (Transaction::default(), None);
        }

        let (range, deleted, inserted_text, impact) = {
            let inner = &mut self.inner;
            let (range, removed, inserted, start_line, old_line_count, new_line_count) = inner
                .document
                .replace_range(Range::new(range.start, range.end), &inserted);
            inner.revision = inner.document.revision;
            let cursor = inner
                .document
                .cursor_for_text_index(range.start + inserted.len());
            let cursor = inner.document.mark_for_cursor(cursor).unwrap_or_else(|| {
                inner
                    .document
                    .mark_for_cursor(
                        inner
                            .document
                            .cursor_for_text_index(inner.document.text_len()),
                    )
                    .expect("text documents always contain at least one line")
            });
            *state = State::collapsed(cursor);
            let affected_start_line_id = inner
                .document
                .line_layout_identity(start_line)
                .map(|identity| identity.id);
            let impact = Impact {
                range: Range::new(range.start, range.end),
                affected_start_line: start_line,
                affected_start_line_id,
                removed_line_count: old_line_count,
                inserted_line_count: new_line_count,
                deleted_bytes: removed.len(),
                inserted_bytes: inserted.len(),
                caret_mark: cursor,
            };
            (range, removed, inserted, impact)
        };

        let delta_kind = if deleted.is_empty() && !inserted_text.is_empty() {
            match kind {
                Kind::ImeCommit => Kind::ImeCommit,
                Kind::Move => Kind::Move,
                _ => Kind::Insert,
            }
        } else if inserted_text.is_empty() && !deleted.is_empty() {
            Kind::Delete
        } else {
            kind
        };
        (
            Transaction::replace(
                Range::new(range.start, range.end),
                deleted,
                inserted_text,
                delta_kind,
            ),
            Some(impact),
        )
    }

    pub(crate) fn move_text_range_for_state(
        &mut self,
        state: &mut State,
        range: Range,
        to: Position,
    ) -> Transaction {
        let (range, to, moved) = {
            let inner = &self.inner;
            let range = inner.document.snap_range(range);
            let to = inner.document.floor_grapheme_boundary(to.index);
            let moved = inner.document.text_for_range(range.clone());
            (range, to, moved)
        };
        if range.is_empty() || (range.start..=range.end).contains(&to) {
            let cursor = self.cursor_for_text_index(to);
            self.set_cursor_and_selection_for_state(state, cursor, CursorSelection::None);
            return Transaction::default();
        }
        let adjusted_to = if to > range.end {
            to - (range.end - range.start)
        } else {
            to
        };
        let mut transaction = self.replace_text_range_with_kind_for_state(
            state,
            Range::new(range.start, range.end),
            "",
            Kind::Move,
        );
        let insert = self.replace_text_range_with_kind_for_state(
            state,
            Range::collapsed(adjusted_to),
            &moved,
            Kind::Move,
        );
        transaction.deltas.extend(insert.deltas);
        transaction
    }

    fn replace_text_range_with_kind_for_state(
        &mut self,
        state: &mut State,
        range: Range,
        text: &str,
        kind: Kind,
    ) -> Transaction {
        self.replace_text_range_with_kind_and_impact_for_state(state, range, text, kind)
            .0
    }

    pub(crate) fn restore_marker_for_state(&mut self, state: &mut State, marker: Marker) {
        if self.inner.id == marker.buffer_id {
            self.inner.revision = marker.revision;
            *state = State::new(marker.cursor_for(self), marker.selection_for(self));
        }
    }

    pub(crate) fn apply_transaction_for_state(
        &mut self,
        state: &mut State,
        transaction: &Transaction,
    ) -> bool {
        for delta in &transaction.deltas {
            self.replace_text_range_with_kind_for_state(
                state,
                Range::new(delta.range.start, delta.range.end),
                &delta.inserted,
                delta.kind,
            );
        }
        true
    }
}

fn selection_mark_for_document(buffer: &Buffer, selection: CursorSelection) -> Option<Mark> {
    let inner = &buffer.inner;
    match selection {
        CursorSelection::None => None,
        CursorSelection::Normal(cursor) => inner.document.mark_for_cursor(cursor),
    }
}

pub(crate) fn selection_mark_from_state(buffer: &Buffer, state: State) -> Option<Cursor> {
    let inner = &buffer.inner;
    state
        .selection
        .and_then(|selection| inner.document.cursor_for_mark(selection.start))
}
