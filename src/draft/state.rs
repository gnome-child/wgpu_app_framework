use std::ops::Range;

use crate::text;

#[derive(Debug, Clone)]
pub(crate) struct State {
    buffer: text::Buffer,
    edit_state: text::selection::State,
    history: text::edit::History,
    base_text: String,
    text: String,
}

impl State {
    pub(crate) fn new(text: impl Into<String>) -> Self {
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
            base_text: text.clone(),
            text,
        }
    }

    pub fn base_text(&self) -> &str {
        &self.base_text
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

    pub(super) fn apply(&mut self, edit: text::edit::Edit, input: text::Input) -> bool {
        let edit = normalize_single_line_edit(edit);
        if edit == text::edit::Edit::InsertLineBreak {
            return true;
        }

        let mut candidate = self.clone();
        candidate.apply_normalized(edit.clone());
        match input.evaluate(candidate.text()) {
            text::InputDecision::Accept => self.apply_normalized(edit),
            text::InputDecision::Normalize(text) => {
                self.apply_normalized(text::edit::Edit::replace_range(0..self.text.len(), text));
            }
            text::InputDecision::Reject => {}
        }

        false
    }

    fn apply_normalized(&mut self, edit: text::edit::Edit) {
        let mut editor = text::edit::Editor::new();
        self.history
            .apply_edit(&mut editor, &mut self.buffer, &mut self.edit_state, edit);
        self.refresh_text();
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

    pub(super) fn seal(&mut self) -> bool {
        if self.base_text == self.text {
            return false;
        }

        self.base_text.clone_from(&self.text);
        true
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
    let end = text.find(['\r', '\n']).unwrap_or(text.len());
    text[..end].to_owned()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn signed_input_evaluates_whole_candidate_drafts() {
        let mut state = State::new("");
        let input = text::Input::signed_integer();

        assert!(!state.apply(text::edit::Edit::insert("-"), input));
        assert_eq!(state.text(), "-");
        assert!(!state.apply(text::edit::Edit::insert("4"), input));
        assert_eq!(state.text(), "-4");
        assert!(!state.apply(text::edit::Edit::insert("-"), input));
        assert_eq!(
            state.text(),
            "-4",
            "a second minus rejects the whole candidate"
        );
        assert!(!state.apply(text::edit::Edit::Backspace, input));
        assert_eq!(state.text(), "-");
        assert!(!state.apply(text::edit::Edit::Backspace, input));
        assert_eq!(state.text(), "");
    }

    #[test]
    fn normalized_replacement_is_one_undoable_draft_change() {
        let mut state = State::new("123");
        let input = text::Input::signed_integer();

        state.apply(text::edit::Edit::SelectAll, input);
        state.apply(text::edit::Edit::insert(" -9 "), input);
        assert_eq!(state.text(), "-9");
        assert_eq!(state.selection(), None);
        assert!(state.undo());
        assert_eq!(state.text(), "123");
        assert!(state.redo());
        assert_eq!(state.text(), "-9");
    }

    #[test]
    fn unsigned_input_rejects_negative_and_non_digit_candidates() {
        let mut state = State::new("12");
        let input = text::Input::unsigned_integer();

        state.apply(text::edit::Edit::insert("-"), input);
        assert_eq!(state.text(), "12");
        state.apply(text::edit::Edit::SelectAll, input);
        state.apply(text::edit::Edit::ime_commit("x"), input);
        assert_eq!(state.text(), "12");
    }
}
