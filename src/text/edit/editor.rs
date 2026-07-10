use std::time::Instant;

use super::super::{
    buffer::{Buffer, Cursor, CursorSelection, Position, Range, normalize_for_buffer},
    unicode::word_range_at,
};
use super::{
    action::{Action, ActionResult},
    buffer::selection_mark_from_state,
    caret::CaretMap,
    clipboard::Clipboard,
    diagnostics::Diagnostics,
    motion::{Motion, collapsed_cursor_for_motion, text_position_for_motion_in_document_for_state},
    operation::{Edit, PointerEditKind},
    outcome::{self, Outcome},
    state::State,
    transaction::{Impact, Kind, Transaction},
};

#[derive(Debug)]
pub struct Editor {
    diagnostics: Diagnostics,
}

struct NoCaretMap;

impl CaretMap for NoCaretMap {
    fn position_for_motion(
        &mut self,
        _buffer: &Buffer,
        _state: State,
        _motion: Motion,
    ) -> Option<Position> {
        None
    }
}

impl Editor {
    pub fn new() -> Self {
        Self {
            diagnostics: Diagnostics::default(),
        }
    }

    pub fn apply_edit(&mut self, buffer: &mut Buffer, state: &mut State, edit: Edit) -> Outcome {
        self.apply_edit_with_caret_map(buffer, state, edit, &mut NoCaretMap)
    }

    pub(crate) fn apply_edit_with_caret_map(
        &mut self,
        buffer: &mut Buffer,
        state: &mut State,
        edit: Edit,
        caret_map: &mut dyn CaretMap,
    ) -> Outcome {
        self.apply_edit_with_caret_map_for_state(buffer, state, edit, caret_map)
    }

    pub(crate) fn apply_action(
        &mut self,
        buffer: &mut Buffer,
        state: &mut State,
        action: Action,
        clipboard: &mut dyn Clipboard,
    ) -> outcome::ActionOutcome {
        self.apply_action_for_state(buffer, state, action, clipboard)
    }

    fn apply_edit_with_caret_map_for_state(
        &mut self,
        buffer: &mut Buffer,
        state: &mut State,
        edit: Edit,
        caret_map: &mut dyn CaretMap,
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
                    let range = buffer.selected_range_for_state(*state).unwrap_or_else(|| {
                        Range::collapsed(buffer.position_for_state(*state).index)
                    });
                    transaction = record_text_edit_impact(
                        &mut impacts,
                        buffer.replace_text_range_with_kind_and_impact_for_state(
                            state,
                            range,
                            "\n",
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
            Edit::MovePosition(motion) => {
                self.move_position(buffer, state, motion, false, caret_map)
            }
            Edit::ExtendPosition(motion) => {
                self.move_position(buffer, state, motion, true, caret_map)
            }
            Edit::SelectAll => {
                let end = buffer.len();
                let cursor = buffer.cursor_for_text_index(end);
                let selection = if end == 0 {
                    CursorSelection::None
                } else {
                    CursorSelection::Normal(buffer.cursor_for_text_index(0))
                };
                buffer.set_cursor_and_selection_for_state(state, cursor, selection);
            }
            Edit::SetPosition(position) => {
                buffer.set_cursor_and_selection_for_state(
                    state,
                    buffer.cursor_for_position(position),
                    CursorSelection::None,
                );
            }
            Edit::Pointer { kind, position } => {
                let cursor = buffer.cursor_for_position(position);
                match kind {
                    PointerEditKind::Click => buffer.set_cursor_and_selection_for_state(
                        state,
                        cursor,
                        CursorSelection::None,
                    ),
                    PointerEditKind::DoubleClick => {
                        let line_text = buffer.text();
                        let range = word_range_at(&line_text, position.index);
                        buffer.set_cursor_and_selection_for_state(
                            state,
                            buffer.cursor_for_text_index(range.end),
                            CursorSelection::Normal(buffer.cursor_for_text_index(range.start)),
                        );
                    }
                    PointerEditKind::TripleClick => {
                        let end = buffer.len();
                        let cursor = buffer.cursor_for_text_index(end);
                        let selection = if end == 0 {
                            CursorSelection::None
                        } else {
                            CursorSelection::Normal(buffer.cursor_for_text_index(0))
                        };
                        buffer.set_cursor_and_selection_for_state(state, cursor, selection);
                    }
                    PointerEditKind::Drag => {
                        let anchor = selection_mark_from_state(buffer, *state)
                            .unwrap_or_else(|| buffer.cursor_for_state(*state));
                        buffer.set_cursor_and_selection_for_state(
                            state,
                            cursor,
                            CursorSelection::Normal(anchor),
                        );
                    }
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

    fn move_position(
        &mut self,
        buffer: &mut Buffer,
        state: &mut State,
        motion: Motion,
        extend: bool,
        caret_map: &mut dyn CaretMap,
    ) {
        let anchor = if extend {
            selection_mark_from_state(buffer, *state)
                .unwrap_or_else(|| buffer.cursor_for_state(*state))
        } else {
            buffer.cursor_for_state(*state)
        };
        if !extend && let Some((start, end)) = buffer.selection_bounds_for_state(*state) {
            buffer.set_cursor_and_selection_for_state(
                state,
                collapsed_cursor_for_motion(motion, start, end),
                CursorSelection::None,
            );
            return;
        }
        let next = self
            .motion_position(buffer, *state, motion, caret_map)
            .unwrap_or_else(|| buffer.position_for_state(*state));
        let cursor = buffer.cursor_for_text_index(next.index);
        let cursor = Cursor::new_with_affinity(cursor.line, cursor.index, next.affinity);
        let selection = if extend {
            CursorSelection::Normal(anchor)
        } else {
            CursorSelection::None
        };
        buffer.set_cursor_and_selection_for_state(state, cursor, selection);
    }

    fn motion_position(
        &mut self,
        buffer: &Buffer,
        state: State,
        motion: Motion,
        caret_map: &mut dyn CaretMap,
    ) -> Option<Position> {
        if let Some(position) =
            text_position_for_motion_in_document_for_state(buffer, state, motion)
        {
            return Some(position);
        }
        caret_map.position_for_motion(buffer, state, motion)
    }

    fn apply_action_for_state(
        &mut self,
        buffer: &mut Buffer,
        state: &mut State,
        action: Action,
        clipboard: &mut dyn Clipboard,
    ) -> outcome::ActionOutcome {
        let before = buffer.marker_for_state(*state);
        let mut result = ActionResult::default();
        let mut change = None;
        match action {
            Action::Copy => {
                let Some(selection) = buffer.selected_text_for_state(*state) else {
                    return outcome::ActionOutcome { result, change };
                };
                match clipboard.write_text(&selection) {
                    Ok(()) => result.clipboard_changed = true,
                    Err(_) => result.unavailable = true,
                }
            }
            Action::Cut => {
                let Some(selection) = buffer.selected_text_for_state(*state) else {
                    return outcome::ActionOutcome { result, change };
                };
                match clipboard.write_text(&selection) {
                    Ok(()) => {
                        result.clipboard_changed = true;
                        let edit_result = self.apply_edit(buffer, state, Edit::insert(""));
                        change = edit_result.change;
                    }
                    Err(_) => result.unavailable = true,
                }
            }
            Action::Delete => {
                let edit_result = self.apply_edit(buffer, state, Edit::delete());
                change = edit_result.change;
            }
            Action::Paste => match clipboard.read_text() {
                Ok(Some(text)) if !normalize_for_buffer(buffer, &text).is_empty() => {
                    let edit_result = self.apply_edit(buffer, state, Edit::insert(text));
                    change = edit_result.change;
                }
                Ok(_) => {}
                Err(_) => result.unavailable = true,
            },
            Action::SelectAll => {
                let edit_result = self.apply_edit(buffer, state, Edit::SelectAll);
                change = edit_result.change;
            }
            Action::Undo | Action::Redo => result.unavailable = true,
        }
        let after = buffer.marker_for_state(*state);
        result.text_changed = before.revision != after.revision;
        result.selection_changed =
            before.cursor != after.cursor || before.selection != after.selection;
        outcome::ActionOutcome { result, change }
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
