use std::time::Instant;

use super::buffer::{
    self as buffer, Buffer, BufferMarker, Cursor, LineId, Mark, Selection, TextMotion,
    TextPosition, TextRange, collapsed_cursor_for_motion, normalize_for_buffer,
    selection_mark_from_state, text_position_for_motion_in_document_for_state,
};
use super::unicode::word_range_at;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct State {
    pub(crate) cursor: Mark,
    pub(crate) selection: Option<buffer::mark::Range>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Edit {
    Insert(String),
    ImeCommit(String),
    ReplaceRange {
        range: TextRange,
        text: String,
    },
    MoveRange {
        range: TextRange,
        to: TextPosition,
    },
    Backspace,
    Delete,
    InsertLineBreak,
    MovePosition(TextMotion),
    ExtendPosition(TextMotion),
    DeleteWordBackward,
    DeleteWordForward,
    SelectAll,
    SetPosition(TextPosition),
    Pointer {
        kind: PointerEditKind,
        position: TextPosition,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PointerEditKind {
    Click,
    DoubleClick,
    TripleClick,
    Drag,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Command {
    Copy,
    Cut,
    Delete,
    Paste,
    SelectAll,
    Undo,
    Redo,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct CommandResult {
    pub text_changed: bool,
    pub selection_changed: bool,
    pub clipboard_changed: bool,
    pub unavailable: bool,
}

#[derive(Debug, Clone, Default)]
pub(crate) struct Outcome {
    pub text_changed: bool,
    pub selection_changed: bool,
    pub change: Option<Change>,
    pub impacts: Vec<Impact>,
}

#[derive(Debug, Clone)]
pub(crate) struct CommandOutcome {
    pub result: CommandResult,
    pub change: Option<Change>,
    pub impacts: Vec<Impact>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct Impact {
    pub(crate) range: TextRange,
    pub(crate) affected_start_line: usize,
    pub(crate) affected_start_line_id: Option<LineId>,
    pub(crate) removed_line_count: usize,
    pub(crate) inserted_line_count: usize,
    pub(crate) deleted_bytes: usize,
    pub(crate) inserted_bytes: usize,
    pub(crate) caret_mark: Mark,
}

#[derive(Debug, Clone)]
pub(crate) struct Change {
    pub(crate) before: BufferMarker,
    pub(crate) after: BufferMarker,
    pub(crate) transaction: Transaction,
}

#[derive(Debug, Clone, Default)]
pub(crate) struct Transaction {
    pub(crate) deltas: Vec<Delta>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct Delta {
    pub(crate) kind: Kind,
    pub(crate) range: TextRange,
    pub(crate) deleted: String,
    pub(crate) inserted: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Kind {
    Insert,
    Delete,
    Replace,
    Move,
    ImeCommit,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ClipboardError {
    Unavailable,
}

pub type ClipboardResult<T> = Result<T, ClipboardError>;

pub trait Clipboard {
    fn read_text(&mut self) -> ClipboardResult<Option<String>>;
    fn write_text(&mut self, text: &str) -> ClipboardResult<()>;
}

pub trait CaretMap {
    fn position_for_motion(
        &mut self,
        buffer: &Buffer,
        state: State,
        motion: TextMotion,
    ) -> Option<TextPosition>;
}

impl State {
    pub fn new(cursor: Mark, selection: Option<buffer::mark::Range>) -> Self {
        Self { cursor, selection }
    }

    pub fn collapsed(cursor: Mark) -> Self {
        Self::new(cursor, None)
    }

    pub fn cursor(self) -> Mark {
        self.cursor
    }

    pub fn selection(self) -> Option<buffer::mark::Range> {
        self.selection
    }
}

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
        _motion: TextMotion,
    ) -> Option<TextPosition> {
        None
    }
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct Diagnostics {
    pub text_edit_calls: usize,
    pub text_edit_changed_calls: usize,
    pub text_edit_apply_nanos: u128,
    pub text_edit_deleted_bytes: usize,
    pub text_edit_inserted_bytes: usize,
    pub text_edit_impacted_logical_lines: usize,
}

impl CommandResult {
    pub fn buffer_changed(self) -> bool {
        self.text_changed || self.selection_changed
    }
    pub fn changed(self) -> bool {
        self.buffer_changed() || self.clipboard_changed
    }
}

impl Impact {
    pub(crate) fn affected_line_count(&self) -> usize {
        self.removed_line_count.max(self.inserted_line_count).max(1)
    }
}

impl Transaction {
    pub(crate) fn replace(range: TextRange, deleted: String, inserted: String, kind: Kind) -> Self {
        let mut transaction = Self::default();
        transaction.push_replace(range, deleted, inserted, kind);
        transaction
    }

    fn push_replace(&mut self, range: TextRange, deleted: String, inserted: String, kind: Kind) {
        if range.start == range.end && deleted.is_empty() && inserted.is_empty() {
            return;
        }
        self.deltas.push(Delta {
            kind,
            range,
            deleted,
            inserted,
        });
    }

    pub(crate) fn is_empty(&self) -> bool {
        self.deltas.is_empty()
    }

    pub(crate) fn inverse(&self) -> Self {
        let mut inverse = Self::default();
        for delta in self.deltas.iter().rev() {
            inverse.push_replace(
                TextRange::new(delta.range.start, delta.range.start + delta.inserted.len()),
                delta.inserted.clone(),
                delta.deleted.clone(),
                delta.kind,
            );
        }
        inverse
    }

    pub(crate) fn try_coalesce_typing(&mut self, next: &Transaction) -> bool {
        if self.deltas.len() != 1 || next.deltas.len() != 1 {
            return false;
        }
        let current = &mut self.deltas[0];
        let next = &next.deltas[0];
        if current.kind != Kind::Insert || next.kind != Kind::Insert {
            return false;
        }
        if !current.deleted.is_empty() || !next.deleted.is_empty() {
            return false;
        }
        if current.range.start + current.inserted.len() != next.range.start {
            return false;
        }
        current.inserted.push_str(&next.inserted);
        true
    }
}

impl Delta {
    #[allow(dead_code)]
    pub(crate) fn inserted_end(&self) -> usize {
        self.range.start + self.inserted.len()
    }
}

impl Outcome {
    pub(super) fn from_markers(
        before: BufferMarker,
        after: BufferMarker,
        transaction: Transaction,
        impacts: Vec<Impact>,
    ) -> Self {
        let text_changed = !transaction.is_empty();
        let selection_changed =
            before.cursor != after.cursor || before.selection != after.selection;
        Self {
            text_changed,
            selection_changed,
            change: text_changed.then_some(Change {
                before,
                after,
                transaction,
            }),
            impacts: text_changed.then_some(impacts).unwrap_or_default(),
        }
    }
    pub fn buffer_changed(&self) -> bool {
        self.text_changed || self.selection_changed
    }
}

impl Editor {
    pub fn new() -> Self {
        Self {
            diagnostics: Diagnostics::default(),
        }
    }

    #[allow(dead_code)]
    pub(crate) fn apply_edit(
        &mut self,
        buffer: &mut Buffer,
        state: &mut State,
        edit: Edit,
    ) -> Outcome {
        self.apply_edit_with_caret_map(buffer, state, edit, &mut NoCaretMap)
    }

    #[allow(dead_code)]
    pub(crate) fn apply_edit_with_caret_map(
        &mut self,
        buffer: &mut Buffer,
        state: &mut State,
        edit: Edit,
        caret_map: &mut dyn CaretMap,
    ) -> Outcome {
        self.apply_edit_with_caret_map_for_state(buffer, state, edit, caret_map)
    }

    #[allow(dead_code)]
    pub(crate) fn apply_command(
        &mut self,
        buffer: &mut Buffer,
        state: &mut State,
        command: Command,
        clipboard: &mut dyn Clipboard,
    ) -> CommandOutcome {
        self.apply_command_for_state(buffer, state, command, clipboard)
    }

    pub fn apply_text_edit(&mut self, buffer: &mut Buffer, edit: Edit) -> bool {
        self.apply_text_edit_with_result(buffer, edit)
            .buffer_changed()
    }

    pub(crate) fn apply_text_edit_with_result(
        &mut self,
        buffer: &mut Buffer,
        edit: Edit,
    ) -> Outcome {
        self.apply_text_edit_with_caret_map(buffer, edit, &mut NoCaretMap)
    }

    pub(crate) fn apply_text_edit_with_caret_map(
        &mut self,
        buffer: &mut Buffer,
        edit: Edit,
        caret_map: &mut dyn CaretMap,
    ) -> Outcome {
        let mut state = buffer.edit_state();
        let result = self.apply_edit_with_caret_map_for_state(buffer, &mut state, edit, caret_map);
        buffer.set_edit_state(state);
        result
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
                let range = buffer.selected_range_for_state(*state).unwrap_or_else(|| {
                    TextRange::collapsed(buffer.position_for_state(*state).index)
                });
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
                let range = buffer.selected_range_for_state(*state).unwrap_or_else(|| {
                    TextRange::collapsed(buffer.position_for_state(*state).index)
                });
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
                            TextRange::new(start, end),
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
                            TextRange::new(start, end),
                            "",
                            Kind::Delete,
                        ),
                    );
                }
            }
            Edit::InsertLineBreak => {
                if buffer.is_multiline() {
                    let range = buffer.selected_range_for_state(*state).unwrap_or_else(|| {
                        TextRange::collapsed(buffer.position_for_state(*state).index)
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
                            TextRange::new(start, end),
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
                            TextRange::new(start, end),
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
                    Selection::None
                } else {
                    Selection::Normal(buffer.cursor_for_text_index(0))
                };
                buffer.set_cursor_and_selection_for_state(state, cursor, selection);
            }
            Edit::SetPosition(position) => {
                let cursor = buffer.cursor_for_text_index(position.index);
                buffer.set_cursor_and_selection_for_state(
                    state,
                    Cursor::new_with_affinity(cursor.line, cursor.index, position.affinity),
                    Selection::None,
                );
            }
            Edit::Pointer { kind, position } => {
                let cursor = buffer.cursor_for_text_index(position.index);
                match kind {
                    PointerEditKind::Click => {
                        buffer.set_cursor_and_selection_for_state(state, cursor, Selection::None)
                    }
                    PointerEditKind::DoubleClick => {
                        let line_text = buffer.text();
                        let range = word_range_at(&line_text, position.index);
                        buffer.set_cursor_and_selection_for_state(
                            state,
                            buffer.cursor_for_text_index(range.end),
                            Selection::Normal(buffer.cursor_for_text_index(range.start)),
                        );
                    }
                    PointerEditKind::TripleClick => {
                        let end = buffer.len();
                        let cursor = buffer.cursor_for_text_index(end);
                        let selection = if end == 0 {
                            Selection::None
                        } else {
                            Selection::Normal(buffer.cursor_for_text_index(0))
                        };
                        buffer.set_cursor_and_selection_for_state(state, cursor, selection);
                    }
                    PointerEditKind::Drag => {
                        let anchor = selection_mark_from_state(buffer, *state)
                            .unwrap_or_else(|| buffer.cursor_for_state(*state));
                        buffer.set_cursor_and_selection_for_state(
                            state,
                            cursor,
                            Selection::Normal(anchor),
                        );
                    }
                }
            }
        }
        if buffer.selected_range_for_state(*state).is_none() {
            let cursor = buffer.cursor_for_state(*state);
            buffer.set_cursor_and_selection_for_state(state, cursor, Selection::None);
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
        motion: TextMotion,
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
                Selection::None,
            );
            return;
        }
        let next = self
            .motion_position(buffer, *state, motion, caret_map)
            .unwrap_or_else(|| buffer.position_for_state(*state));
        let cursor = buffer.cursor_for_text_index(next.index);
        let cursor = Cursor::new_with_affinity(cursor.line, cursor.index, next.affinity);
        let selection = if extend {
            Selection::Normal(anchor)
        } else {
            Selection::None
        };
        buffer.set_cursor_and_selection_for_state(state, cursor, selection);
    }

    fn motion_position(
        &mut self,
        buffer: &Buffer,
        state: State,
        motion: TextMotion,
        caret_map: &mut dyn CaretMap,
    ) -> Option<TextPosition> {
        if let Some(position) =
            text_position_for_motion_in_document_for_state(buffer, state, motion)
        {
            return Some(position);
        }
        caret_map.position_for_motion(buffer, state, motion)
    }

    pub fn apply_text_command(
        &mut self,
        buffer: &mut Buffer,
        command: Command,
        clipboard: &mut dyn Clipboard,
    ) -> CommandResult {
        self.apply_text_command_with_result(buffer, command, clipboard)
            .result
    }

    pub(crate) fn apply_text_command_with_result(
        &mut self,
        buffer: &mut Buffer,
        command: Command,
        clipboard: &mut dyn Clipboard,
    ) -> CommandOutcome {
        let mut state = buffer.edit_state();
        let outcome = self.apply_command_for_state(buffer, &mut state, command, clipboard);
        buffer.set_edit_state(state);
        outcome
    }

    fn apply_command_for_state(
        &mut self,
        buffer: &mut Buffer,
        state: &mut State,
        command: Command,
        clipboard: &mut dyn Clipboard,
    ) -> CommandOutcome {
        let before = buffer.marker_for_state(*state);
        let mut result = CommandResult::default();
        let mut change = None;
        let mut impacts = Vec::new();
        match command {
            Command::Copy => {
                let Some(selection) = buffer.selected_text_for_state(*state) else {
                    return CommandOutcome {
                        result,
                        change,
                        impacts,
                    };
                };
                match clipboard.write_text(&selection) {
                    Ok(()) => result.clipboard_changed = true,
                    Err(_) => result.unavailable = true,
                }
            }
            Command::Cut => {
                let Some(selection) = buffer.selected_text_for_state(*state) else {
                    return CommandOutcome {
                        result,
                        change,
                        impacts,
                    };
                };
                match clipboard.write_text(&selection) {
                    Ok(()) => {
                        result.clipboard_changed = true;
                        let edit_result = self.apply_edit(buffer, state, Edit::insert(""));
                        change = edit_result.change;
                        impacts = edit_result.impacts;
                    }
                    Err(_) => result.unavailable = true,
                }
            }
            Command::Delete => {
                let edit_result = self.apply_edit(buffer, state, Edit::delete());
                change = edit_result.change;
                impacts = edit_result.impacts;
            }
            Command::Paste => match clipboard.read_text() {
                Ok(Some(text)) if !normalize_for_buffer(buffer, &text).is_empty() => {
                    let edit_result = self.apply_edit(buffer, state, Edit::insert(text));
                    change = edit_result.change;
                    impacts = edit_result.impacts;
                }
                Ok(_) => {}
                Err(_) => result.unavailable = true,
            },
            Command::SelectAll => {
                let edit_result = self.apply_edit(buffer, state, Edit::SelectAll);
                change = edit_result.change;
                impacts = edit_result.impacts;
            }
            Command::Undo | Command::Redo => result.unavailable = true,
        }
        let after = buffer.marker_for_state(*state);
        result.text_changed = before.revision != after.revision;
        result.selection_changed =
            before.cursor != after.cursor || before.selection != after.selection;
        CommandOutcome {
            result,
            change,
            impacts,
        }
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

impl Edit {
    pub fn insert(text: impl Into<String>) -> Self {
        Self::Insert(text.into())
    }

    pub fn ime_commit(text: impl Into<String>) -> Self {
        Self::ImeCommit(text.into())
    }

    pub fn replace_range(range: impl Into<TextRange>, text: impl Into<String>) -> Self {
        Self::ReplaceRange {
            range: range.into(),
            text: text.into(),
        }
    }

    pub fn insert_at(position: impl Into<TextPosition>, text: impl Into<String>) -> Self {
        let position = position.into();
        Self::replace_range(TextRange::collapsed(position.index), text)
    }

    pub fn move_range(range: impl Into<TextRange>, to: impl Into<TextPosition>) -> Self {
        Self::MoveRange {
            range: range.into(),
            to: to.into(),
        }
    }

    pub fn backspace() -> Self {
        Self::Backspace
    }

    pub fn delete() -> Self {
        Self::Delete
    }

    pub fn insert_line_break() -> Self {
        Self::InsertLineBreak
    }

    pub fn move_position(motion: TextMotion) -> Self {
        Self::MovePosition(motion)
    }

    pub fn extend_position(motion: TextMotion) -> Self {
        Self::ExtendPosition(motion)
    }

    #[cfg(test)]
    pub(crate) fn set_cursor(cursor: Cursor) -> Self {
        Self::SetPosition(cursor.into())
    }
    pub fn delete_word_backward() -> Self {
        Self::DeleteWordBackward
    }

    pub fn delete_word_forward() -> Self {
        Self::DeleteWordForward
    }

    pub fn set_position(position: impl Into<TextPosition>) -> Self {
        Self::SetPosition(position.into())
    }

    pub fn pointer(kind: PointerEditKind, position: impl Into<TextPosition>) -> Self {
        Self::Pointer {
            kind,
            position: position.into(),
        }
    }

    pub(crate) fn mutates_text(&self) -> bool {
        matches!(
            self,
            Self::Insert(_)
                | Self::ImeCommit(_)
                | Self::ReplaceRange { .. }
                | Self::MoveRange { .. }
                | Self::Backspace
                | Self::Delete
                | Self::InsertLineBreak
                | Self::DeleteWordBackward
                | Self::DeleteWordForward
        )
    }
}
