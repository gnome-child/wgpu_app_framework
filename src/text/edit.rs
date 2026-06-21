use std::time::Instant;

use unicode_segmentation::UnicodeSegmentation;

use crate::text_system;

pub(super) const TYPING_UNDO_COALESCE_WINDOW: std::time::Duration =
    std::time::Duration::from_millis(1000);

use super::buffer::{
    Buffer, BufferMarker, Cursor, Selection, TextChange, TextEditImpact, TextEditKind, TextMotion,
    TextPosition, TextRange, TextTransaction, collapsed_cursor_for_motion,
    cosmic_motion_for_text_motion, glyph_affinity, normalize_for_buffer,
    selection_mark_from_buffer, text_position_for_cursor_in_buffer,
    text_position_for_motion_in_document,
};
use super::unicode::word_range_at;

#[cfg(test)]
fn text_motion_from_cosmic_motion(motion: glyphon::cosmic_text::Motion) -> TextMotion {
    match motion {
        glyphon::cosmic_text::Motion::Left => TextMotion::VisualLeft,
        glyphon::cosmic_text::Motion::Right => TextMotion::VisualRight,
        glyphon::cosmic_text::Motion::Up => TextMotion::VisualUp,
        glyphon::cosmic_text::Motion::Down => TextMotion::VisualDown,
        glyphon::cosmic_text::Motion::PageUp => TextMotion::PageUp,
        glyphon::cosmic_text::Motion::PageDown => TextMotion::PageDown,
        glyphon::cosmic_text::Motion::Previous => TextMotion::LogicalPrevious,
        glyphon::cosmic_text::Motion::Next => TextMotion::LogicalNext,
        glyphon::cosmic_text::Motion::LeftWord | glyphon::cosmic_text::Motion::PreviousWord => {
            TextMotion::WordPrevious
        }
        glyphon::cosmic_text::Motion::RightWord | glyphon::cosmic_text::Motion::NextWord => {
            TextMotion::WordNext
        }
        glyphon::cosmic_text::Motion::Home | glyphon::cosmic_text::Motion::SoftHome => {
            TextMotion::LineStart
        }
        glyphon::cosmic_text::Motion::End => TextMotion::LineEnd,
        glyphon::cosmic_text::Motion::ParagraphStart => TextMotion::ParagraphStart,
        glyphon::cosmic_text::Motion::ParagraphEnd => TextMotion::ParagraphEnd,
        glyphon::cosmic_text::Motion::BufferStart => TextMotion::DocumentStart,
        glyphon::cosmic_text::Motion::BufferEnd => TextMotion::DocumentEnd,
        _ => TextMotion::LogicalNext,
    }
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
pub(crate) struct TextEditResult {
    pub text_changed: bool,
    pub selection_changed: bool,
    pub change: Option<TextChange>,
    pub impacts: Vec<TextEditImpact>,
}
#[derive(Debug, Clone)]
pub(crate) struct TextCommandOutcome {
    pub result: CommandResult,
    pub change: Option<TextChange>,
    pub impacts: Vec<TextEditImpact>,
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum HistoryKind {
    Typing(String),
    Boundary,
}
#[derive(Debug, Clone)]
struct HistoryEntry {
    before: BufferMarker,
    after: BufferMarker,
    transaction: TextTransaction,
    kind: HistoryKind,
    recorded_at: Instant,
}
#[derive(Debug, Clone, Default)]
pub(super) struct EditHistory {
    undo: Vec<HistoryEntry>,
    redo: Vec<HistoryEntry>,
    current: Option<BufferMarker>,
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

#[derive(Debug)]
pub struct Editor {
    font_system: Option<glyphon::FontSystem>,
    diagnostics: Diagnostics,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct Diagnostics {
    pub text_edit_calls: usize,
    pub text_edit_changed_calls: usize,
    pub text_edit_apply_nanos: u128,
    pub text_edit_deleted_bytes: usize,
    pub text_edit_inserted_bytes: usize,
    pub text_edit_impacted_logical_lines: usize,
    pub aggregate_buffer_fallbacks: usize,
}

impl CommandResult {
    pub fn buffer_changed(self) -> bool {
        self.text_changed || self.selection_changed
    }
    pub fn changed(self) -> bool {
        self.buffer_changed() || self.clipboard_changed
    }
}

impl TextEditResult {
    pub(super) fn from_markers(
        before: BufferMarker,
        after: BufferMarker,
        transaction: TextTransaction,
        impacts: Vec<TextEditImpact>,
    ) -> Self {
        let text_changed = !transaction.is_empty();
        let selection_changed =
            before.cursor != after.cursor || before.selection != after.selection;
        Self {
            text_changed,
            selection_changed,
            change: text_changed.then_some(TextChange {
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

impl HistoryKind {
    fn typing_text(&self) -> Option<&str> {
        match self {
            Self::Typing(text) => Some(text),
            Self::Boundary => None,
        }
    }
}

fn typing_history_kind(text: &str) -> HistoryKind {
    let mut graphemes = text.graphemes(true);
    let Some(first) = graphemes.next() else {
        return HistoryKind::Boundary;
    };
    if graphemes.next().is_some() {
        return HistoryKind::Boundary;
    }
    if first
        .chars()
        .any(|ch| ch.is_whitespace() || ch.is_ascii_punctuation())
    {
        return HistoryKind::Boundary;
    }
    HistoryKind::Typing(first.to_owned())
}

impl Editor {
    pub fn new() -> Self {
        Self {
            font_system: None,
            diagnostics: Diagnostics::default(),
        }
    }

    pub fn apply_text_edit(&mut self, buffer: &mut Buffer, edit: Edit) -> bool {
        self.apply_text_edit_with_result(buffer, edit)
            .buffer_changed()
    }

    pub(crate) fn apply_text_edit_with_result(
        &mut self,
        buffer: &mut Buffer,
        edit: Edit,
    ) -> TextEditResult {
        let edit_started = Instant::now();
        let before = buffer.marker();
        let mut transaction = TextTransaction::default();
        let mut impacts = Vec::new();
        match edit {
            Edit::Insert(text) => {
                let range = buffer
                    .selected_range()
                    .unwrap_or_else(|| TextRange::collapsed(buffer.position().index));
                transaction = record_text_edit_impact(
                    &mut impacts,
                    buffer.replace_text_range_with_kind_and_impact(
                        range,
                        &text,
                        TextEditKind::Insert,
                    ),
                );
            }
            Edit::ImeCommit(text) => {
                let range = buffer
                    .selected_range()
                    .unwrap_or_else(|| TextRange::collapsed(buffer.position().index));
                transaction = record_text_edit_impact(
                    &mut impacts,
                    buffer.replace_text_range_with_kind_and_impact(
                        range,
                        &text,
                        TextEditKind::ImeCommit,
                    ),
                );
            }
            Edit::ReplaceRange { range, text } => {
                transaction = record_text_edit_impact(
                    &mut impacts,
                    buffer.replace_text_range_with_kind_and_impact(
                        range,
                        &text,
                        TextEditKind::Replace,
                    ),
                );
            }
            Edit::MoveRange { range, to } => transaction = buffer.move_text_range(range, to),
            Edit::Backspace => {
                if let Some(range) = buffer.selected_range() {
                    transaction = record_text_edit_impact(
                        &mut impacts,
                        buffer.replace_text_range_with_kind_and_impact(
                            range,
                            "",
                            TextEditKind::Delete,
                        ),
                    );
                } else {
                    let end = buffer.position().index;
                    let start = buffer
                        .inner
                        .borrow()
                        .document
                        .previous_grapheme_boundary_index(end);
                    transaction = record_text_edit_impact(
                        &mut impacts,
                        buffer.replace_text_range_with_kind_and_impact(
                            TextRange::new(start, end),
                            "",
                            TextEditKind::Delete,
                        ),
                    );
                }
            }
            Edit::Delete => {
                if let Some(range) = buffer.selected_range() {
                    transaction = record_text_edit_impact(
                        &mut impacts,
                        buffer.replace_text_range_with_kind_and_impact(
                            range,
                            "",
                            TextEditKind::Delete,
                        ),
                    );
                } else {
                    let start = buffer.position().index;
                    let end = buffer
                        .inner
                        .borrow()
                        .document
                        .next_grapheme_boundary_index(start);
                    transaction = record_text_edit_impact(
                        &mut impacts,
                        buffer.replace_text_range_with_kind_and_impact(
                            TextRange::new(start, end),
                            "",
                            TextEditKind::Delete,
                        ),
                    );
                }
            }
            Edit::InsertLineBreak => {
                if buffer.is_multiline() {
                    let range = buffer
                        .selected_range()
                        .unwrap_or_else(|| TextRange::collapsed(buffer.position().index));
                    transaction = record_text_edit_impact(
                        &mut impacts,
                        buffer.replace_text_range_with_kind_and_impact(
                            range,
                            "\n",
                            TextEditKind::Insert,
                        ),
                    );
                }
            }
            Edit::DeleteWordBackward => {
                if let Some(range) = buffer.selected_range() {
                    transaction = record_text_edit_impact(
                        &mut impacts,
                        buffer.replace_text_range_with_kind_and_impact(
                            range,
                            "",
                            TextEditKind::Delete,
                        ),
                    );
                } else {
                    let end = buffer.position().index;
                    let start = buffer
                        .inner
                        .borrow()
                        .document
                        .previous_word_boundary_index(end);
                    transaction = record_text_edit_impact(
                        &mut impacts,
                        buffer.replace_text_range_with_kind_and_impact(
                            TextRange::new(start, end),
                            "",
                            TextEditKind::Delete,
                        ),
                    );
                }
            }
            Edit::DeleteWordForward => {
                if let Some(range) = buffer.selected_range() {
                    transaction = record_text_edit_impact(
                        &mut impacts,
                        buffer.replace_text_range_with_kind_and_impact(
                            range,
                            "",
                            TextEditKind::Delete,
                        ),
                    );
                } else {
                    let start = buffer.position().index;
                    let end = buffer
                        .inner
                        .borrow()
                        .document
                        .next_word_boundary_index(start);
                    transaction = record_text_edit_impact(
                        &mut impacts,
                        buffer.replace_text_range_with_kind_and_impact(
                            TextRange::new(start, end),
                            "",
                            TextEditKind::Delete,
                        ),
                    );
                }
            }
            Edit::MovePosition(motion) => self.move_position(buffer, motion, false),
            Edit::ExtendPosition(motion) => self.move_position(buffer, motion, true),
            Edit::SelectAll => {
                let end = buffer.len();
                let cursor = buffer.cursor_for_text_index(end);
                let selection = if end == 0 {
                    Selection::None
                } else {
                    Selection::Normal(buffer.cursor_for_text_index(0))
                };
                buffer.set_cursor_and_selection(cursor, selection);
            }
            Edit::SetPosition(position) => {
                let cursor = buffer.cursor_for_text_index(position.index);
                buffer.set_cursor_and_selection(
                    Cursor::new_with_affinity(
                        cursor.line,
                        cursor.index,
                        glyph_affinity(position.affinity),
                    ),
                    Selection::None,
                );
            }
            Edit::Pointer { kind, position } => {
                let cursor = buffer.cursor_for_text_index(position.index);
                match kind {
                    PointerEditKind::Click => {
                        buffer.set_cursor_and_selection(cursor, Selection::None)
                    }
                    PointerEditKind::DoubleClick => {
                        let line_text = buffer.text();
                        let range = word_range_at(&line_text, position.index);
                        buffer.set_cursor_and_selection(
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
                        buffer.set_cursor_and_selection(cursor, selection);
                    }
                    PointerEditKind::Drag => {
                        let anchor =
                            selection_mark_from_buffer(buffer).unwrap_or_else(|| buffer.cursor());
                        buffer.set_cursor_and_selection(cursor, Selection::Normal(anchor));
                    }
                }
            }
        }
        if buffer.selected_range().is_none() {
            let cursor = buffer.cursor();
            buffer.set_cursor_and_selection(cursor, Selection::None);
        }
        let after = buffer.marker();
        let result = TextEditResult::from_markers(before, after, transaction, impacts);
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
                .map(TextEditImpact::affected_line_count)
                .sum::<usize>();
        }
        result
    }

    fn move_position(&mut self, buffer: &mut Buffer, motion: TextMotion, extend: bool) {
        let anchor = if extend {
            selection_mark_from_buffer(buffer).unwrap_or_else(|| buffer.cursor())
        } else {
            buffer.cursor()
        };
        if !extend && let Some((start, end)) = buffer.selection_bounds() {
            buffer.set_cursor_and_selection(
                collapsed_cursor_for_motion(motion, start, end),
                Selection::None,
            );
            return;
        }
        let next = self
            .motion_position(buffer, motion)
            .unwrap_or_else(|| buffer.position());
        let cursor = buffer.cursor_for_text_index(next.index);
        let cursor =
            Cursor::new_with_affinity(cursor.line, cursor.index, glyph_affinity(next.affinity));
        let selection = if extend {
            Selection::Normal(anchor)
        } else {
            Selection::None
        };
        buffer.set_cursor_and_selection(cursor, selection);
    }

    fn motion_position(&mut self, buffer: &Buffer, motion: TextMotion) -> Option<TextPosition> {
        if let Some(position) = text_position_for_motion_in_document(buffer, motion) {
            return Some(position);
        }
        let cosmic_motion = cosmic_motion_for_text_motion(motion)?;
        self.diagnostics.aggregate_buffer_fallbacks += 1;
        let font_system = self
            .font_system
            .get_or_insert_with(text_system::font_system);
        let mut prepared = buffer.cloned_cosmic_buffer();
        prepared.set_wrap(font_system, glyphon::Wrap::None);
        prepared.shape_until_scroll(font_system, false);
        let mut editor = glyphon::Editor::new(&mut prepared);
        glyphon::Edit::set_cursor(&mut editor, buffer.cursor());
        glyphon::Edit::set_selection(&mut editor, Selection::None);
        glyphon::Edit::action(
            &mut editor,
            font_system,
            glyphon::Action::Motion(cosmic_motion),
        );
        let cursor = glyphon::Edit::cursor(&editor);
        drop(editor);
        Some(text_position_for_cursor_in_buffer(&prepared, cursor))
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
    ) -> TextCommandOutcome {
        let before = buffer.marker();
        let mut result = CommandResult::default();
        let mut change = None;
        let mut impacts = Vec::new();
        match command {
            Command::Copy => {
                let Some(selection) = buffer.selected_text() else {
                    return TextCommandOutcome {
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
                let Some(selection) = buffer.selected_text() else {
                    return TextCommandOutcome {
                        result,
                        change,
                        impacts,
                    };
                };
                match clipboard.write_text(&selection) {
                    Ok(()) => {
                        result.clipboard_changed = true;
                        let edit_result =
                            self.apply_text_edit_with_result(buffer, Edit::insert(""));
                        change = edit_result.change;
                        impacts = edit_result.impacts;
                    }
                    Err(_) => result.unavailable = true,
                }
            }
            Command::Paste => match clipboard.read_text() {
                Ok(Some(text)) if !normalize_for_buffer(buffer, &text).is_empty() => {
                    let edit_result = self.apply_text_edit_with_result(buffer, Edit::insert(text));
                    change = edit_result.change;
                    impacts = edit_result.impacts;
                }
                Ok(_) => {}
                Err(_) => result.unavailable = true,
            },
            Command::SelectAll => {
                let edit_result = self.apply_text_edit_with_result(buffer, Edit::SelectAll);
                change = edit_result.change;
                impacts = edit_result.impacts;
            }
            Command::Undo | Command::Redo => result.unavailable = true,
        }
        let after = buffer.marker();
        result.text_changed = before.revision != after.revision;
        result.selection_changed =
            before.cursor != after.cursor || before.selection != after.selection;
        TextCommandOutcome {
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
    impacts: &mut Vec<TextEditImpact>,
    edit: (TextTransaction, Option<TextEditImpact>),
) -> TextTransaction {
    let (transaction, impact) = edit;
    if let Some(impact) = impact {
        impacts.push(impact);
    }
    transaction
}

impl EditHistory {
    pub(super) fn sync(&mut self, marker: BufferMarker) -> bool {
        if self.current.as_ref() == Some(&marker) {
            return false;
        }

        if self.current.as_ref().is_some_and(|current| {
            current.buffer_id == marker.buffer_id && current.revision == marker.revision
        }) {
            self.current = Some(marker);
            return false;
        }

        let changed = self.current.is_some() || !self.undo.is_empty() || !self.redo.is_empty();
        self.undo.clear();
        self.redo.clear();
        self.current = Some(marker);
        changed
    }

    pub(super) fn record(&mut self, change: TextChange, kind: HistoryKind, now: Instant) {
        if change.before == change.after {
            self.current = Some(change.after);
            return;
        }

        if let Some(current) = self.current.as_ref()
            && current != &change.before
            && (current.buffer_id != change.before.buffer_id
                || current.revision != change.before.revision)
        {
            self.undo.clear();
            self.redo.clear();
        }

        if kind.typing_text().is_some()
            && let Some(last) = self.undo.last_mut()
            && last.kind.typing_text().is_some()
            && last.after == change.before
            && now.saturating_duration_since(last.recorded_at) <= TYPING_UNDO_COALESCE_WINDOW
            && last.transaction.try_coalesce_typing(&change.transaction)
        {
            last.after = change.after.clone();
            last.kind = kind;
            last.recorded_at = now;
            self.redo.clear();
            self.current = Some(change.after);
            return;
        }

        self.undo.push(HistoryEntry {
            before: change.before,
            after: change.after.clone(),
            transaction: change.transaction,
            kind,
            recorded_at: now,
        });
        self.redo.clear();
        self.current = Some(change.after);
    }

    pub(super) fn can_undo(&self) -> bool {
        !self.undo.is_empty()
    }

    #[cfg(test)]
    pub(super) fn undo_len(&self) -> usize {
        self.undo.len()
    }

    pub(super) fn can_redo(&self) -> bool {
        !self.redo.is_empty()
    }

    pub(super) fn undo(&mut self, buffer: &mut Buffer) -> CommandResult {
        let Some(entry) = self.undo.pop() else {
            return CommandResult {
                unavailable: true,
                ..CommandResult::default()
            };
        };

        let before = buffer.marker();
        let reverse = entry.transaction.inverse();

        if !buffer.apply_transaction(&reverse) {
            self.undo.push(entry);
            return CommandResult {
                unavailable: true,
                ..CommandResult::default()
            };
        }

        buffer.restore_marker(entry.before.clone());
        let after = buffer.marker();
        self.current = Some(after.clone());
        self.redo.push(entry);
        command_result_from_markers(before, after)
    }

    pub(super) fn redo(&mut self, buffer: &mut Buffer) -> CommandResult {
        let Some(entry) = self.redo.pop() else {
            return CommandResult {
                unavailable: true,
                ..CommandResult::default()
            };
        };

        let before = buffer.marker();

        if !buffer.apply_transaction(&entry.transaction) {
            self.redo.push(entry);
            return CommandResult {
                unavailable: true,
                ..CommandResult::default()
            };
        }

        buffer.restore_marker(entry.after.clone());
        let after = buffer.marker();
        self.current = Some(after.clone());
        self.undo.push(entry);
        command_result_from_markers(before, after)
    }
}

fn command_result_from_markers(before: BufferMarker, after: BufferMarker) -> CommandResult {
    CommandResult {
        text_changed: before.revision != after.revision,
        selection_changed: before.cursor != after.cursor || before.selection != after.selection,
        clipboard_changed: false,
        unavailable: false,
    }
}

impl PartialEq for EditHistory {
    fn eq(&self, other: &Self) -> bool {
        self.current == other.current
            && self.undo.len() == other.undo.len()
            && self.redo.len() == other.redo.len()
            && self
                .undo
                .last()
                .map(|entry| (&entry.before, &entry.after, &entry.kind))
                == other
                    .undo
                    .last()
                    .map(|entry| (&entry.before, &entry.after, &entry.kind))
            && self
                .redo
                .last()
                .map(|entry| (&entry.before, &entry.after, &entry.kind))
                == other
                    .redo
                    .last()
                    .map(|entry| (&entry.before, &entry.after, &entry.kind))
    }
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
    pub(crate) fn action(action: glyphon::Action) -> Self {
        match action {
            glyphon::Action::Backspace => Self::Backspace,
            glyphon::Action::Delete => Self::Delete,
            glyphon::Action::Enter => Self::InsertLineBreak,
            glyphon::Action::Insert(character) => Self::insert(character.to_string()),
            glyphon::Action::Motion(motion) => Self::motion(motion),
            _ => Self::MovePosition(TextMotion::LogicalNext),
        }
    }

    #[cfg(test)]
    pub(crate) fn motion(motion: glyphon::cosmic_text::Motion) -> Self {
        Self::MovePosition(text_motion_from_cosmic_motion(motion))
    }

    #[cfg(test)]
    pub(crate) fn extend_motion(motion: glyphon::cosmic_text::Motion) -> Self {
        Self::ExtendPosition(text_motion_from_cosmic_motion(motion))
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

    pub(crate) fn history_kind(&self) -> HistoryKind {
        match self {
            Self::Insert(text) => typing_history_kind(text),
            Self::ImeCommit(_)
            | Self::ReplaceRange { .. }
            | Self::MoveRange { .. }
            | Self::Backspace
            | Self::Delete
            | Self::InsertLineBreak
            | Self::DeleteWordBackward
            | Self::DeleteWordForward => HistoryKind::Boundary,
            Self::MovePosition(_)
            | Self::ExtendPosition(_)
            | Self::SelectAll
            | Self::SetPosition(_)
            | Self::Pointer { .. } => HistoryKind::Boundary,
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
