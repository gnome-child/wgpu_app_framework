use std::time::Instant;

use super::super::{
    buffer::{Buffer, CursorSelection, Range},
    selection::State,
};
use super::{
    diagnostics::Diagnostics,
    operation::Edit,
    outcome::Outcome,
    transaction::{Impact, Kind, Transaction},
};

#[derive(Debug)]
pub struct Editor {
    diagnostics: Diagnostics,
}

impl Editor {
    pub fn new() -> Self {
        Self {
            diagnostics: Diagnostics::default(),
        }
    }

    pub fn apply_edit(&mut self, buffer: &mut Buffer, state: &mut State, edit: Edit) -> Outcome {
        self.apply_edit_for_state(buffer, state, edit)
    }

    fn apply_edit_for_state(
        &mut self,
        buffer: &mut Buffer,
        state: &mut State,
        edit: Edit,
    ) -> Outcome {
        let edit_started = Instant::now();
        let before = buffer.marker_for_state(*state);
        let mut transaction = Transaction::default();
        let mut impacts = Vec::new();
        match edit {
            Edit::Insert(text) => {
                let range = buffer
                    .selected_range_for_state(*state)
                    .unwrap_or_else(|| Range::collapsed(buffer.position_for_state(*state).index));
                transaction = record_text_edit_impact(
                    &mut impacts,
                    buffer.replace_text_range_with_kind_and_impact_for_state(
                        state,
                        range,
                        &text,
                        Kind::Insert,
                    ),
                );
            }
            Edit::ImeCommit(text) => {
                let range = buffer
                    .selected_range_for_state(*state)
                    .unwrap_or_else(|| Range::collapsed(buffer.position_for_state(*state).index));
                transaction = record_text_edit_impact(
                    &mut impacts,
                    buffer.replace_text_range_with_kind_and_impact_for_state(
                        state,
                        range,
                        &text,
                        Kind::ImeCommit,
                    ),
                );
            }
            Edit::ReplaceRange { range, text } => {
                transaction = record_text_edit_impact(
                    &mut impacts,
                    buffer.replace_text_range_with_kind_and_impact_for_state(
                        state,
                        range,
                        &text,
                        Kind::Replace,
                    ),
                );
            }
            Edit::MoveRange { range, to } => {
                transaction = buffer.move_text_range_for_state(state, range, to);
            }
            Edit::Backspace => {
                if let Some(range) = buffer.selected_range_for_state(*state) {
                    transaction = record_text_edit_impact(
                        &mut impacts,
                        buffer.replace_text_range_with_kind_and_impact_for_state(
                            state,
                            range,
                            "",
                            Kind::Delete,
                        ),
                    );
                } else {
                    let end = buffer.position_for_state(*state).index;
                    let start = buffer.inner.document.previous_grapheme_boundary_index(end);
                    transaction = record_text_edit_impact(
                        &mut impacts,
                        buffer.replace_text_range_with_kind_and_impact_for_state(
                            state,
                            Range::new(start, end),
                            "",
                            Kind::Delete,
                        ),
                    );
                }
            }
            Edit::Delete => {
                if let Some(range) = buffer.selected_range_for_state(*state) {
                    transaction = record_text_edit_impact(
                        &mut impacts,
                        buffer.replace_text_range_with_kind_and_impact_for_state(
                            state,
                            range,
                            "",
                            Kind::Delete,
                        ),
                    );
                } else {
                    let start = buffer.position_for_state(*state).index;
                    let end = buffer.inner.document.next_grapheme_boundary_index(start);
                    transaction = record_text_edit_impact(
                        &mut impacts,
                        buffer.replace_text_range_with_kind_and_impact_for_state(
                            state,
                            Range::new(start, end),
                            "",
                            Kind::Delete,
                        ),
                    );
                }
            }
            Edit::InsertLineBreak => {
                if buffer.is_multiline() {
                    let line_ending = buffer.line_ending();
                    let range = buffer.selected_range_for_state(*state).unwrap_or_else(|| {
                        Range::collapsed(buffer.position_for_state(*state).index)
                    });
                    transaction = record_text_edit_impact(
                        &mut impacts,
                        buffer.replace_text_range_with_kind_and_impact_for_state(
                            state,
                            range,
                            line_ending,
                            Kind::Insert,
                        ),
                    );
                }
            }
            Edit::DeleteWordBackward => {
                if let Some(range) = buffer.selected_range_for_state(*state) {
                    transaction = record_text_edit_impact(
                        &mut impacts,
                        buffer.replace_text_range_with_kind_and_impact_for_state(
                            state,
                            range,
                            "",
                            Kind::Delete,
                        ),
                    );
                } else {
                    let end = buffer.position_for_state(*state).index;
                    let start = buffer.inner.document.previous_word_boundary_index(end);
                    transaction = record_text_edit_impact(
                        &mut impacts,
                        buffer.replace_text_range_with_kind_and_impact_for_state(
                            state,
                            Range::new(start, end),
                            "",
                            Kind::Delete,
                        ),
                    );
                }
            }
            Edit::DeleteWordForward => {
                if let Some(range) = buffer.selected_range_for_state(*state) {
                    transaction = record_text_edit_impact(
                        &mut impacts,
                        buffer.replace_text_range_with_kind_and_impact_for_state(
                            state,
                            range,
                            "",
                            Kind::Delete,
                        ),
                    );
                } else {
                    let start = buffer.position_for_state(*state).index;
                    let end = buffer.inner.document.next_word_boundary_index(start);
                    transaction = record_text_edit_impact(
                        &mut impacts,
                        buffer.replace_text_range_with_kind_and_impact_for_state(
                            state,
                            Range::new(start, end),
                            "",
                            Kind::Delete,
                        ),
                    );
                }
            }
        }
        if buffer.selected_range_for_state(*state).is_none() {
            let cursor = buffer.cursor_for_state(*state);
            buffer.set_cursor_and_selection_for_state(state, cursor, CursorSelection::None);
        }
        let after = buffer.marker_for_state(*state);
        let result = Outcome::from_markers(before, after, transaction, impacts);
        self.diagnostics.text_edit_calls += 1;
        self.diagnostics.text_edit_apply_nanos += edit_started.elapsed().as_nanos();
        if result.text_changed {
            self.diagnostics.text_edit_changed_calls += 1;
            self.diagnostics.text_edit_deleted_bytes += result
                .impacts
                .iter()
                .map(|impact| impact.deleted_bytes)
                .sum::<usize>();
            self.diagnostics.text_edit_inserted_bytes += result
                .impacts
                .iter()
                .map(|impact| impact.inserted_bytes)
                .sum::<usize>();
            self.diagnostics.text_edit_impacted_logical_lines += result
                .impacts
                .iter()
                .map(Impact::affected_line_count)
                .sum::<usize>();
        }
        result
    }

    pub fn diagnostics(&self) -> Diagnostics {
        self.diagnostics
    }

    pub fn reset_diagnostics(&mut self) {
        self.diagnostics = Diagnostics::default();
    }
}

impl Default for Editor {
    fn default() -> Self {
        Self::new()
    }
}

fn record_text_edit_impact(
    impacts: &mut Vec<Impact>,
    edit: (Transaction, Option<Impact>),
) -> Transaction {
    let (transaction, impact) = edit;
    if let Some(impact) = impact {
        impacts.push(impact);
    }
    transaction
}
