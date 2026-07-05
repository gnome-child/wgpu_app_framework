use std::collections::HashMap;
use std::collections::hash_map::{Entry, ValuesMut};
use std::time::{Duration, Instant};

use crate::{text, ui};
use unicode_segmentation::UnicodeSegmentation;

pub(crate) const TYPING_UNDO_COALESCE_WINDOW: Duration = Duration::from_millis(1000);

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum HistoryKind {
    Typing(String),
    Boundary,
}

#[derive(Debug, Clone)]
struct HistoryEntry {
    before: text::buffer::BufferMarker,
    after: text::buffer::BufferMarker,
    transaction: text::edit::Transaction,
    kind: HistoryKind,
    recorded_at: Instant,
}

#[derive(Debug, Clone, Default)]
struct History {
    undo: Vec<HistoryEntry>,
    redo: Vec<HistoryEntry>,
    current: Option<text::buffer::BufferMarker>,
}

#[derive(Debug, Default)]
pub(crate) struct Driver {
    states: HashMap<ui::Path, text::view::TextViewState>,
    histories: HashMap<ui::Path, History>,
}

impl HistoryKind {
    pub(crate) fn for_edit(edit: &text::edit::Edit) -> Self {
        match edit {
            text::edit::Edit::Insert(text) => typing_history_kind(text),
            text::edit::Edit::ImeCommit(_)
            | text::edit::Edit::ReplaceRange { .. }
            | text::edit::Edit::MoveRange { .. }
            | text::edit::Edit::Backspace
            | text::edit::Edit::Delete
            | text::edit::Edit::InsertLineBreak
            | text::edit::Edit::DeleteWordBackward
            | text::edit::Edit::DeleteWordForward => Self::Boundary,
            text::edit::Edit::MovePosition(_)
            | text::edit::Edit::ExtendPosition(_)
            | text::edit::Edit::SelectAll
            | text::edit::Edit::SetPosition(_)
            | text::edit::Edit::Pointer { .. } => Self::Boundary,
        }
    }

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

impl History {
    fn sync(&mut self, marker: text::buffer::BufferMarker) -> bool {
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

    fn record(&mut self, change: text::edit::Change, kind: HistoryKind, now: Instant) {
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

    fn can_undo(&self) -> bool {
        !self.undo.is_empty()
    }

    #[cfg(test)]
    fn undo_len(&self) -> usize {
        self.undo.len()
    }

    fn can_redo(&self) -> bool {
        !self.redo.is_empty()
    }

    fn undo(&mut self, buffer: &mut text::Buffer) -> text::edit::CommandResult {
        let Some(entry) = self.undo.pop() else {
            return text::edit::CommandResult {
                unavailable: true,
                ..text::edit::CommandResult::default()
            };
        };

        let before = buffer.marker();
        let reverse = entry.transaction.inverse();

        if !buffer.apply_transaction(&reverse) {
            self.undo.push(entry);
            return text::edit::CommandResult {
                unavailable: true,
                ..text::edit::CommandResult::default()
            };
        }

        buffer.restore_marker(entry.before.clone());
        let after = buffer.marker();
        self.current = Some(after.clone());
        self.redo.push(entry);
        command_result_from_markers(before, after)
    }

    fn redo(&mut self, buffer: &mut text::Buffer) -> text::edit::CommandResult {
        let Some(entry) = self.redo.pop() else {
            return text::edit::CommandResult {
                unavailable: true,
                ..text::edit::CommandResult::default()
            };
        };

        let before = buffer.marker();

        if !buffer.apply_transaction(&entry.transaction) {
            self.redo.push(entry);
            return text::edit::CommandResult {
                unavailable: true,
                ..text::edit::CommandResult::default()
            };
        }

        buffer.restore_marker(entry.after.clone());
        let after = buffer.marker();
        self.current = Some(after.clone());
        self.undo.push(entry);
        command_result_from_markers(before, after)
    }
}

fn command_result_from_markers(
    before: text::buffer::BufferMarker,
    after: text::buffer::BufferMarker,
) -> text::edit::CommandResult {
    text::edit::CommandResult {
        text_changed: before.revision != after.revision,
        selection_changed: before.cursor != after.cursor || before.selection != after.selection,
        clipboard_changed: false,
        unavailable: false,
    }
}

impl Driver {
    pub(crate) fn is_empty(&self) -> bool {
        self.states.is_empty()
    }

    pub(crate) fn clear(&mut self) {
        self.states.clear();
        self.histories.clear();
    }

    pub(crate) fn contains(&self, path: &ui::Path) -> bool {
        self.states.contains_key(path)
    }

    pub(crate) fn states(&self) -> &HashMap<ui::Path, text::view::TextViewState> {
        &self.states
    }

    pub(crate) fn states_mut(&mut self) -> &mut HashMap<ui::Path, text::view::TextViewState> {
        &mut self.states
    }

    pub(crate) fn get(&self, path: &ui::Path) -> Option<&text::view::TextViewState> {
        self.states.get(path)
    }

    pub(crate) fn get_cloned_or_default(&self, path: &ui::Path) -> text::view::TextViewState {
        self.states.get(path).cloned().unwrap_or_default()
    }

    pub(crate) fn insert(
        &mut self,
        path: ui::Path,
        state: text::view::TextViewState,
    ) -> Option<text::view::TextViewState> {
        self.states.insert(path, state)
    }

    pub(crate) fn entry(
        &mut self,
        path: ui::Path,
    ) -> Entry<'_, ui::Path, text::view::TextViewState> {
        self.states.entry(path)
    }

    pub(crate) fn values_mut(&mut self) -> ValuesMut<'_, ui::Path, text::view::TextViewState> {
        self.states.values_mut()
    }

    pub(crate) fn sync_history(&mut self, path: &ui::Path, buffer: &text::Buffer) -> bool {
        self.histories
            .entry(path.clone())
            .or_default()
            .sync(buffer.marker())
    }

    pub(crate) fn record_history_at(
        &mut self,
        path: &ui::Path,
        change: text::edit::Change,
        kind: HistoryKind,
        now: Instant,
    ) {
        self.histories
            .entry(path.clone())
            .or_default()
            .record(change, kind, now);
    }

    pub(crate) fn can_undo(&self, path: &ui::Path) -> bool {
        self.histories.get(path).is_some_and(History::can_undo)
    }

    #[cfg(test)]
    pub(crate) fn history_undo_len(&self, path: &ui::Path) -> usize {
        self.histories.get(path).map_or(0, History::undo_len)
    }

    pub(crate) fn can_redo(&self, path: &ui::Path) -> bool {
        self.histories.get(path).is_some_and(History::can_redo)
    }

    pub(crate) fn apply_undo(
        &mut self,
        path: &ui::Path,
        buffer: &mut text::Buffer,
    ) -> text::edit::CommandResult {
        self.histories.get_mut(path).map_or_else(
            || text::edit::CommandResult {
                unavailable: true,
                ..text::edit::CommandResult::default()
            },
            |history| history.undo(buffer),
        )
    }

    pub(crate) fn apply_redo(
        &mut self,
        path: &ui::Path,
        buffer: &mut text::Buffer,
    ) -> text::edit::CommandResult {
        self.histories.get_mut(path).map_or_else(
            || text::edit::CommandResult {
                unavailable: true,
                ..text::edit::CommandResult::default()
            },
            |history| history.redo(buffer),
        )
    }
}

impl From<HashMap<ui::Path, text::view::TextViewState>> for Driver {
    fn from(states: HashMap<ui::Path, text::view::TextViewState>) -> Self {
        Self {
            states,
            histories: HashMap::new(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const FIELD: ui::Id = ui::Id::new("field");

    #[derive(Default)]
    struct MockClipboard {
        text: Option<String>,
    }

    impl MockClipboard {
        fn with_text(text: impl Into<String>) -> Self {
            Self {
                text: Some(text.into()),
            }
        }
    }

    impl text::edit::Clipboard for MockClipboard {
        fn read_text(&mut self) -> text::edit::ClipboardResult<Option<String>> {
            Ok(self.text.clone())
        }

        fn write_text(&mut self, text: &str) -> text::edit::ClipboardResult<()> {
            self.text = Some(text.to_owned());
            Ok(())
        }
    }

    fn path() -> ui::Path {
        ui::Path::from(FIELD)
    }

    fn record_edit(
        editor: &mut text::edit::Editor,
        driver: &mut Driver,
        path: &ui::Path,
        buffer: &mut text::Buffer,
        edit: text::edit::Edit,
    ) -> text::edit::Outcome {
        record_edit_at(editor, driver, path, buffer, edit, Instant::now())
    }

    fn record_edit_at(
        editor: &mut text::edit::Editor,
        driver: &mut Driver,
        path: &ui::Path,
        buffer: &mut text::Buffer,
        edit: text::edit::Edit,
        now: Instant,
    ) -> text::edit::Outcome {
        driver.sync_history(path, buffer);
        let kind = HistoryKind::for_edit(&edit);
        let result = editor.apply_text_edit_with_result(buffer, edit);
        if let Some(change) = result.change.clone() {
            driver.record_history_at(path, change, kind, now);
        }
        result
    }

    fn record_command(
        editor: &mut text::edit::Editor,
        driver: &mut Driver,
        path: &ui::Path,
        buffer: &mut text::Buffer,
        command: text::edit::Command,
        clipboard: &mut dyn text::edit::Clipboard,
    ) -> text::edit::CommandResult {
        driver.sync_history(path, buffer);
        let outcome = editor.apply_text_command_with_result(buffer, command, clipboard);
        if let Some(change) = outcome.change.clone() {
            driver.record_history_at(path, change, HistoryKind::Boundary, Instant::now());
        }
        outcome.result
    }

    #[test]
    fn history_coalesces_typing_into_one_undo_step() {
        let mut editor = text::edit::Editor::new();
        let mut driver = Driver::default();
        let path = path();
        let mut buffer = text::Buffer::new();

        record_edit(
            &mut editor,
            &mut driver,
            &path,
            &mut buffer,
            text::edit::Edit::insert("a"),
        );
        record_edit(
            &mut editor,
            &mut driver,
            &path,
            &mut buffer,
            text::edit::Edit::insert("b"),
        );
        record_edit(
            &mut editor,
            &mut driver,
            &path,
            &mut buffer,
            text::edit::Edit::insert("c"),
        );

        assert_eq!(buffer.text(), "abc");
        assert_eq!(driver.history_undo_len(&path), 1);
        assert!(driver.can_undo(&path));

        let undo = driver.apply_undo(&path, &mut buffer);
        assert_eq!(buffer.text(), "");
        assert!(undo.text_changed);
        assert!(driver.can_redo(&path));

        let redo = driver.apply_redo(&path, &mut buffer);
        assert_eq!(buffer.text(), "abc");
        assert!(redo.text_changed);
    }

    #[test]
    fn history_splits_typing_after_coalesce_timeout() {
        let mut editor = text::edit::Editor::new();
        let mut driver = Driver::default();
        let path = path();
        let mut buffer = text::Buffer::new();
        let start = Instant::now();

        record_edit_at(
            &mut editor,
            &mut driver,
            &path,
            &mut buffer,
            text::edit::Edit::insert("a"),
            start,
        );
        record_edit_at(
            &mut editor,
            &mut driver,
            &path,
            &mut buffer,
            text::edit::Edit::insert("b"),
            start + TYPING_UNDO_COALESCE_WINDOW + Duration::from_millis(1),
        );

        assert_eq!(buffer.text(), "ab");
        assert_eq!(driver.history_undo_len(&path), 2);
    }

    #[test]
    fn history_splits_typing_at_whitespace_and_punctuation() {
        let mut editor = text::edit::Editor::new();
        let mut driver = Driver::default();
        let path = path();
        let mut buffer = text::Buffer::new();

        record_edit(
            &mut editor,
            &mut driver,
            &path,
            &mut buffer,
            text::edit::Edit::insert("a"),
        );
        record_edit(
            &mut editor,
            &mut driver,
            &path,
            &mut buffer,
            text::edit::Edit::insert(" "),
        );
        record_edit(
            &mut editor,
            &mut driver,
            &path,
            &mut buffer,
            text::edit::Edit::insert("b"),
        );
        record_edit(
            &mut editor,
            &mut driver,
            &path,
            &mut buffer,
            text::edit::Edit::insert("."),
        );

        assert_eq!(buffer.text(), "a b.");
        assert_eq!(driver.history_undo_len(&path), 4);
    }

    #[test]
    fn history_splits_typing_after_cursor_movement() {
        let mut editor = text::edit::Editor::new();
        let mut driver = Driver::default();
        let path = path();
        let mut buffer = text::Buffer::new();

        record_edit(
            &mut editor,
            &mut driver,
            &path,
            &mut buffer,
            text::edit::Edit::insert("a"),
        );
        editor.apply_text_edit(&mut buffer, text::edit::Edit::set_position(0));
        record_edit(
            &mut editor,
            &mut driver,
            &path,
            &mut buffer,
            text::edit::Edit::insert("b"),
        );

        assert_eq!(buffer.text(), "ba");
        assert_eq!(driver.history_undo_len(&path), 2);
        driver.apply_undo(&path, &mut buffer);
        assert_eq!(buffer.text(), "a");
    }

    #[test]
    fn history_keeps_paste_cut_delete_word_delete_and_ime_as_separate_steps() {
        let mut editor = text::edit::Editor::new();
        let mut driver = Driver::default();
        let path = path();
        let mut buffer = text::Buffer::from_text("hello");
        let mut clipboard = MockClipboard::with_text(" pasted");

        record_command(
            &mut editor,
            &mut driver,
            &path,
            &mut buffer,
            text::edit::Command::Paste,
            &mut clipboard,
        );
        record_edit(
            &mut editor,
            &mut driver,
            &path,
            &mut buffer,
            text::edit::Edit::backspace(),
        );
        record_edit(
            &mut editor,
            &mut driver,
            &path,
            &mut buffer,
            text::edit::Edit::delete_word_backward(),
        );
        record_edit(
            &mut editor,
            &mut driver,
            &path,
            &mut buffer,
            text::edit::Edit::ime_commit("x"),
        );

        editor.apply_text_edit(&mut buffer, text::edit::Edit::SelectAll);
        let mut clipboard = MockClipboard::default();
        record_command(
            &mut editor,
            &mut driver,
            &path,
            &mut buffer,
            text::edit::Command::Cut,
            &mut clipboard,
        );

        assert_eq!(driver.history_undo_len(&path), 5);
    }

    #[test]
    fn history_undo_restores_text_cursor_and_selection() {
        let mut editor = text::edit::Editor::new();
        let mut driver = Driver::default();
        let path = path();
        let mut buffer = text::Buffer::from_text("hello");

        driver.sync_history(&path, &buffer);
        editor.apply_text_edit(&mut buffer, text::edit::Edit::SelectAll);
        record_edit(
            &mut editor,
            &mut driver,
            &path,
            &mut buffer,
            text::edit::Edit::insert("x"),
        );

        assert_eq!(buffer.text(), "x");
        assert!(!buffer.has_selection());

        let undo = driver.apply_undo(&path, &mut buffer);
        assert_eq!(buffer.text(), "hello");
        assert_eq!(buffer.selected_text().as_deref(), Some("hello"));
        assert!(undo.text_changed);
        assert!(undo.selection_changed);

        let redo = driver.apply_redo(&path, &mut buffer);
        assert_eq!(buffer.text(), "x");
        assert!(!buffer.has_selection());
        assert!(redo.text_changed);
    }

    #[test]
    fn history_new_edit_after_undo_clears_redo() {
        let mut editor = text::edit::Editor::new();
        let mut driver = Driver::default();
        let path = path();
        let mut buffer = text::Buffer::new();

        record_edit(
            &mut editor,
            &mut driver,
            &path,
            &mut buffer,
            text::edit::Edit::insert("a"),
        );
        driver.apply_undo(&path, &mut buffer);
        assert!(driver.can_redo(&path));

        record_edit(
            &mut editor,
            &mut driver,
            &path,
            &mut buffer,
            text::edit::Edit::insert("b"),
        );

        assert_eq!(buffer.text(), "b");
        assert!(!driver.can_redo(&path));
        assert!(driver.can_undo(&path));
    }

    #[test]
    fn history_external_buffer_replacement_clears_stale_history() {
        let mut editor = text::edit::Editor::new();
        let mut driver = Driver::default();
        let path = path();
        let mut buffer = text::Buffer::new();

        record_edit(
            &mut editor,
            &mut driver,
            &path,
            &mut buffer,
            text::edit::Edit::insert("a"),
        );
        assert!(driver.can_undo(&path));

        let external = text::Buffer::from_text("external");
        assert!(driver.sync_history(&path, &external));
        assert!(!driver.can_undo(&path));
        assert!(!driver.can_redo(&path));
    }
}
