use std::time::{Duration, Instant};

use unicode_segmentation::UnicodeSegmentation;

use super::{
    Action, ActionResult, Edit, Editor, Marker, Outcome, State,
    clipboard::Clipboard,
    transaction::{Change, Transaction},
};
use crate::text::Buffer;

pub const TYPING_UNDO_COALESCE_WINDOW: Duration = Duration::from_millis(1000);

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum HistoryKind {
    Typing(String),
    Boundary,
}

#[derive(Debug, Clone)]
struct Entry {
    before: Marker,
    after: Marker,
    transaction: Transaction,
    kind: HistoryKind,
    recorded_at: Instant,
}

#[derive(Debug, Clone, Default)]
pub struct History {
    undo: Vec<Entry>,
    redo: Vec<Entry>,
    current: Option<Marker>,
}

impl HistoryKind {
    pub fn for_edit(edit: &Edit) -> Self {
        match edit {
            Edit::Insert(text) | Edit::ImeCommit(text) => typing_history_kind(text),
            Edit::ReplaceRange { .. }
            | Edit::MoveRange { .. }
            | Edit::Backspace
            | Edit::Delete
            | Edit::InsertLineBreak
            | Edit::DeleteWordBackward
            | Edit::DeleteWordForward
            | Edit::MovePosition(_)
            | Edit::ExtendPosition(_)
            | Edit::SelectAll
            | Edit::SetPosition(_)
            | Edit::Pointer { .. } => Self::Boundary,
        }
    }

    fn typing_text(&self) -> Option<&str> {
        match self {
            Self::Typing(text) => Some(text),
            Self::Boundary => None,
        }
    }
}

impl History {
    pub fn sync(&mut self, buffer: &Buffer, state: State) -> bool {
        let marker = buffer.marker_for_state(state);
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

    pub fn apply_edit(
        &mut self,
        editor: &mut Editor,
        buffer: &mut Buffer,
        state: &mut State,
        edit: Edit,
    ) -> Outcome {
        self.sync(buffer, *state);
        let kind = HistoryKind::for_edit(&edit);
        let outcome = editor.apply_edit(buffer, state, edit);
        if let Some(change) = outcome.change.clone() {
            self.record(change, kind, Instant::now());
        }
        outcome
    }

    pub fn apply_action(
        &mut self,
        editor: &mut Editor,
        buffer: &mut Buffer,
        state: &mut State,
        action: Action,
        clipboard: &mut dyn Clipboard,
    ) -> ActionResult {
        self.sync(buffer, *state);
        let outcome = editor.apply_action(buffer, state, action, clipboard);
        if let Some(change) = outcome.change.clone() {
            self.record(change, HistoryKind::Boundary, Instant::now());
        }
        outcome.result
    }

    pub(crate) fn record(&mut self, change: Change, kind: HistoryKind, now: Instant) {
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

        self.undo.push(Entry {
            before: change.before,
            after: change.after.clone(),
            transaction: change.transaction,
            kind,
            recorded_at: now,
        });
        self.redo.clear();
        self.current = Some(change.after);
    }

    pub fn can_undo(&self) -> bool {
        !self.undo.is_empty()
    }

    pub fn undo_len(&self) -> usize {
        self.undo.len()
    }

    pub fn can_redo(&self) -> bool {
        !self.redo.is_empty()
    }

    pub fn undo(&mut self, buffer: &mut Buffer, state: &mut State) -> ActionResult {
        let Some(entry) = self.undo.pop() else {
            return ActionResult {
                unavailable: true,
                ..ActionResult::default()
            };
        };

        let before = buffer.marker_for_state(*state);
        let reverse = entry.transaction.inverse();

        if !buffer.apply_transaction_for_state(state, &reverse) {
            self.undo.push(entry);
            return ActionResult {
                unavailable: true,
                ..ActionResult::default()
            };
        }

        buffer.restore_marker_for_state(state, entry.before.clone());
        let after = buffer.marker_for_state(*state);
        self.current = Some(after.clone());
        self.redo.push(entry);
        action_result_from_markers(before, after)
    }

    pub fn redo(&mut self, buffer: &mut Buffer, state: &mut State) -> ActionResult {
        let Some(entry) = self.redo.pop() else {
            return ActionResult {
                unavailable: true,
                ..ActionResult::default()
            };
        };

        let before = buffer.marker_for_state(*state);

        if !buffer.apply_transaction_for_state(state, &entry.transaction) {
            self.redo.push(entry);
            return ActionResult {
                unavailable: true,
                ..ActionResult::default()
            };
        }

        buffer.restore_marker_for_state(state, entry.after.clone());
        let after = buffer.marker_for_state(*state);
        self.current = Some(after.clone());
        self.undo.push(entry);
        action_result_from_markers(before, after)
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

fn action_result_from_markers(before: Marker, after: Marker) -> ActionResult {
    ActionResult {
        text_changed: before.revision != after.revision,
        selection_changed: before.cursor != after.cursor || before.selection != after.selection,
        clipboard_changed: false,
        unavailable: false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn undo_redo_keeps_older_typing_history_available() {
        let mut buffer = Buffer::from_text("");
        let mut state = buffer.initial_state();
        let mut editor = Editor::new();
        let mut history = History::default();

        for character in ["a", "b", "c"] {
            history.apply_edit(
                &mut editor,
                &mut buffer,
                &mut state,
                Edit::ime_commit(character),
            );
        }
        assert_eq!(buffer.text(), "abc");

        history.apply_edit(&mut editor, &mut buffer, &mut state, Edit::Backspace);
        assert_eq!(buffer.text(), "ab");

        assert!(history.undo(&mut buffer, &mut state).buffer_changed());
        assert_eq!(buffer.text(), "abc");

        assert!(history.redo(&mut buffer, &mut state).buffer_changed());
        assert_eq!(buffer.text(), "ab");

        assert!(history.undo(&mut buffer, &mut state).buffer_changed());
        assert_eq!(buffer.text(), "abc");

        assert!(history.undo(&mut buffer, &mut state).buffer_changed());
        assert_eq!(buffer.text(), "");
    }
}
