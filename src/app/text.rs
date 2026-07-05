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
    before: text::edit::Marker,
    after: text::edit::Marker,
    transaction: text::edit::transaction::Transaction,
    kind: HistoryKind,
    recorded_at: Instant,
}

#[derive(Debug, Clone, Default)]
struct History {
    undo: Vec<HistoryEntry>,
    redo: Vec<HistoryEntry>,
    current: Option<text::edit::Marker>,
}

#[derive(Debug, Default)]
pub(crate) struct Driver {
    states: HashMap<ui::Path, text::edit::ViewState>,
    edit_states: HashMap<ui::Path, text::edit::State>,
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
    fn sync(&mut self, marker: text::edit::Marker) -> bool {
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

    fn record(&mut self, change: text::edit::transaction::Change, kind: HistoryKind, now: Instant) {
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

    fn undo(
        &mut self,
        buffer: &mut text::Buffer,
        state: &mut text::edit::State,
    ) -> text::edit::ActionResult {
        let Some(entry) = self.undo.pop() else {
            return text::edit::ActionResult {
                unavailable: true,
                ..text::edit::ActionResult::default()
            };
        };

        let before = buffer.marker_for_state(*state);
        let reverse = entry.transaction.inverse();

        if !buffer.apply_transaction_for_state(state, &reverse) {
            self.undo.push(entry);
            return text::edit::ActionResult {
                unavailable: true,
                ..text::edit::ActionResult::default()
            };
        }

        buffer.restore_marker_for_state(state, entry.before.clone());
        let after = buffer.marker_for_state(*state);
        self.current = Some(after.clone());
        self.redo.push(entry);
        command_result_from_markers(before, after)
    }

    fn redo(
        &mut self,
        buffer: &mut text::Buffer,
        state: &mut text::edit::State,
    ) -> text::edit::ActionResult {
        let Some(entry) = self.redo.pop() else {
            return text::edit::ActionResult {
                unavailable: true,
                ..text::edit::ActionResult::default()
            };
        };

        let before = buffer.marker_for_state(*state);

        if !buffer.apply_transaction_for_state(state, &entry.transaction) {
            self.redo.push(entry);
            return text::edit::ActionResult {
                unavailable: true,
                ..text::edit::ActionResult::default()
            };
        }

        buffer.restore_marker_for_state(state, entry.after.clone());
        let after = buffer.marker_for_state(*state);
        self.current = Some(after.clone());
        self.undo.push(entry);
        command_result_from_markers(before, after)
    }
}

fn command_result_from_markers(
    before: text::edit::Marker,
    after: text::edit::Marker,
) -> text::edit::ActionResult {
    text::edit::ActionResult {
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
        self.edit_states.clear();
        self.histories.clear();
    }

    pub(crate) fn contains(&self, path: &ui::Path) -> bool {
        self.states.contains_key(path)
    }

    pub(crate) fn states(&self) -> &HashMap<ui::Path, text::edit::ViewState> {
        &self.states
    }

    pub(crate) fn states_mut(&mut self) -> &mut HashMap<ui::Path, text::edit::ViewState> {
        &mut self.states
    }

    pub(crate) fn edit_states(&self) -> &HashMap<ui::Path, text::edit::State> {
        &self.edit_states
    }

    pub(crate) fn edit_state(&self, path: &ui::Path) -> Option<text::edit::State> {
        self.edit_states.get(path).copied()
    }

    pub(crate) fn edit_state_or_initial(
        &self,
        path: &ui::Path,
        buffer: &text::Buffer,
    ) -> text::edit::State {
        self.edit_state(path)
            .unwrap_or_else(|| buffer.initial_state())
    }

    pub(crate) fn store_edit_state(&mut self, path: &ui::Path, state: text::edit::State) -> bool {
        if self.edit_states.get(path).copied() == Some(state) {
            return false;
        }

        self.edit_states.insert(path.clone(), state);
        true
    }

    pub(crate) fn get(&self, path: &ui::Path) -> Option<&text::edit::ViewState> {
        self.states.get(path)
    }

    pub(crate) fn get_cloned_or_default(&self, path: &ui::Path) -> text::edit::ViewState {
        self.states.get(path).cloned().unwrap_or_default()
    }

    pub(crate) fn insert(
        &mut self,
        path: ui::Path,
        state: text::edit::ViewState,
    ) -> Option<text::edit::ViewState> {
        self.states.insert(path, state)
    }

    pub(crate) fn entry(&mut self, path: ui::Path) -> Entry<'_, ui::Path, text::edit::ViewState> {
        self.states.entry(path)
    }

    pub(crate) fn values_mut(&mut self) -> ValuesMut<'_, ui::Path, text::edit::ViewState> {
        self.states.values_mut()
    }

    pub(crate) fn sync_surface(&mut self, path: &ui::Path, surface: &text::edit::Surface) -> bool {
        let edit_changed = self.store_edit_state(path, surface.state());
        let history_changed = self.sync_history_for_state(path, surface.buffer(), surface.state());
        edit_changed || history_changed
    }

    fn sync_history_for_state(
        &mut self,
        path: &ui::Path,
        buffer: &text::Buffer,
        state: text::edit::State,
    ) -> bool {
        self.histories
            .entry(path.clone())
            .or_default()
            .sync(buffer.marker_for_state(state))
    }

    pub(crate) fn record_history_at(
        &mut self,
        path: &ui::Path,
        change: text::edit::transaction::Change,
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
    ) -> text::edit::ActionResult {
        let Some(history) = self.histories.get_mut(path) else {
            return text::edit::ActionResult {
                unavailable: true,
                ..text::edit::ActionResult::default()
            };
        };
        let edit_state = self
            .edit_states
            .entry(path.clone())
            .or_insert_with(|| buffer.initial_state());
        history.undo(buffer, edit_state)
    }

    pub(crate) fn apply_redo(
        &mut self,
        path: &ui::Path,
        buffer: &mut text::Buffer,
    ) -> text::edit::ActionResult {
        let Some(history) = self.histories.get_mut(path) else {
            return text::edit::ActionResult {
                unavailable: true,
                ..text::edit::ActionResult::default()
            };
        };
        let edit_state = self
            .edit_states
            .entry(path.clone())
            .or_insert_with(|| buffer.initial_state());
        history.redo(buffer, edit_state)
    }
}

impl From<HashMap<ui::Path, text::edit::ViewState>> for Driver {
    fn from(states: HashMap<ui::Path, text::edit::ViewState>) -> Self {
        Self {
            states,
            edit_states: HashMap::new(),
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

    fn edit_state(driver: &Driver, path: &ui::Path, buffer: &text::Buffer) -> text::edit::State {
        driver.edit_state_or_initial(path, buffer)
    }

    fn has_selection(driver: &Driver, path: &ui::Path, buffer: &text::Buffer) -> bool {
        buffer.has_selection_for_state(edit_state(driver, path, buffer))
    }

    fn selected_text(driver: &Driver, path: &ui::Path, buffer: &text::Buffer) -> Option<String> {
        buffer.selected_text_for_state(edit_state(driver, path, buffer))
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
        let mut edit_state = driver.edit_state_or_initial(path, buffer);
        driver.sync_history_for_state(path, buffer, edit_state);
        let kind = HistoryKind::for_edit(&edit);
        let result = editor.apply_edit(buffer, &mut edit_state, edit);
        driver.store_edit_state(path, edit_state);
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
        command: text::edit::Action,
        clipboard: &mut dyn text::edit::Clipboard,
    ) -> text::edit::ActionResult {
        let mut edit_state = driver.edit_state_or_initial(path, buffer);
        driver.sync_history_for_state(path, buffer, edit_state);
        let outcome = editor.apply_action(buffer, &mut edit_state, command, clipboard);
        driver.store_edit_state(path, edit_state);
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
        let mut edit_state = edit_state(&driver, &path, &buffer);
        editor.apply_edit(
            &mut buffer,
            &mut edit_state,
            text::edit::Edit::set_position(0),
        );
        driver.store_edit_state(&path, edit_state);
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
            text::edit::Action::Paste,
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

        let mut edit_state = edit_state(&driver, &path, &buffer);
        editor.apply_edit(&mut buffer, &mut edit_state, text::edit::Edit::SelectAll);
        driver.store_edit_state(&path, edit_state);
        let mut clipboard = MockClipboard::default();
        record_command(
            &mut editor,
            &mut driver,
            &path,
            &mut buffer,
            text::edit::Action::Cut,
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

        let mut edit_state = edit_state(&driver, &path, &buffer);
        driver.sync_history_for_state(&path, &buffer, edit_state);
        editor.apply_edit(&mut buffer, &mut edit_state, text::edit::Edit::SelectAll);
        driver.store_edit_state(&path, edit_state);
        record_edit(
            &mut editor,
            &mut driver,
            &path,
            &mut buffer,
            text::edit::Edit::insert("x"),
        );

        assert_eq!(buffer.text(), "x");
        assert!(!has_selection(&driver, &path, &buffer));

        let undo = driver.apply_undo(&path, &mut buffer);
        assert_eq!(buffer.text(), "hello");
        assert_eq!(
            selected_text(&driver, &path, &buffer).as_deref(),
            Some("hello")
        );
        assert!(undo.text_changed);
        assert!(undo.selection_changed);

        let redo = driver.apply_redo(&path, &mut buffer);
        assert_eq!(buffer.text(), "x");
        assert!(!has_selection(&driver, &path, &buffer));
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
        assert!(driver.sync_history_for_state(&path, &external, external.initial_state()));
        assert!(!driver.can_undo(&path));
        assert!(!driver.can_redo(&path));
    }
}
