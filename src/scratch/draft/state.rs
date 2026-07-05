use std::ops::Range;

use crate::text;

#[derive(Debug, Clone)]
pub struct State {
    buffer: text::Buffer,
    edit_state: text::edit::State,
    history: text::edit::History,
    text: String,
}

impl State {
    pub(in crate::scratch) fn new(text: impl Into<String>) -> Self {
        let mut state = Self::from_buffer(text::Buffer::from_text(text.into()));
        state.history.sync(&state.buffer, state.edit_state);
        state
    }

    fn from_buffer(buffer: text::Buffer) -> Self {
        let edit_state = buffer.initial_state();
        let text = buffer.text();
        Self {
            buffer,
            edit_state,
            history: text::edit::History::default(),
            text,
        }
    }

    pub fn text(&self) -> &str {
        &self.text
    }

    pub fn cursor(&self) -> usize {
        self.buffer.position_for_state(self.edit_state).index
    }

    pub fn selection(&self) -> Option<Range<usize>> {
        self.buffer
            .selected_range_for_state(self.edit_state)
            .map(text::buffer::Range::as_range)
    }

    pub fn selected_text(&self) -> Option<String> {
        self.buffer.selected_text_for_state(self.edit_state)
    }

    pub fn can_undo(&self) -> bool {
        self.history.can_undo()
    }

    pub fn can_redo(&self) -> bool {
        self.history.can_redo()
    }

    pub(super) fn apply(&mut self, edit: text::edit::Edit) -> bool {
        let edit = normalize_single_line_edit(edit);
        if edit == text::edit::Edit::InsertLineBreak {
            return true;
        }

        let mut editor = text::edit::Editor::new();
        self.history
            .apply_edit(&mut editor, &mut self.buffer, &mut self.edit_state, edit);
        self.refresh_text();
        false
    }

    pub(super) fn undo(&mut self) -> bool {
        let result = self.history.undo(&mut self.buffer, &mut self.edit_state);
        self.refresh_text();
        result.buffer_changed()
    }

    pub(super) fn redo(&mut self) -> bool {
        let result = self.history.redo(&mut self.buffer, &mut self.edit_state);
        self.refresh_text();
        result.buffer_changed()
    }

    fn refresh_text(&mut self) {
        self.text = self.buffer.text();
    }
}

impl PartialEq for State {
    fn eq(&self, other: &Self) -> bool {
        self.text() == other.text()
            && self.cursor() == other.cursor()
            && self.selection() == other.selection()
    }
}

impl Eq for State {}

fn normalize_single_line_edit(edit: text::edit::Edit) -> text::edit::Edit {
    match edit {
        text::edit::Edit::Insert(text) => text::edit::Edit::Insert(first_line(text)),
        text::edit::Edit::ImeCommit(text) => text::edit::Edit::ImeCommit(first_line(text)),
        text::edit::Edit::ReplaceRange { range, text } => text::edit::Edit::ReplaceRange {
            range,
            text: first_line(text),
        },
        edit => edit,
    }
}

fn first_line(text: String) -> String {
    let end = text
        .find(['\r', '\n'])
        .unwrap_or(text.len());
    text[..end].to_owned()
}
