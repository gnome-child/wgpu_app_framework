use super::super::buffer::{Buffer, CursorSelection, Position, Range, normalize_for_buffer};
use super::super::selection::State;
use super::{
    Marker,
    transaction::{Impact, Kind, Transaction},
};

impl Buffer {
    pub(crate) fn marker_for_state(&self, state: State) -> Marker {
        Marker::new(self, state)
    }

    pub(crate) fn replace_text_range_with_kind_and_impact_for_state(
        &mut self,
        state: &mut State,
        range: Range,
        text: &str,
        kind: Kind,
    ) -> (Transaction, Option<Impact>) {
        let inserted = normalize_for_buffer(self, text);
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
            let cursor = inner.document.mark_for_cursor(cursor);
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
