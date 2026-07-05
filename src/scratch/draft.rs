use crate::text::{self, unicode};

use super::interaction::Target;

#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct Input {
    target: Option<Target>,
    draft: Option<State>,
    preedit: Option<text::Preedit>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct State {
    text: String,
    cursor: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Change {
    draft: State,
    text_changed: bool,
    cursor_changed: bool,
    changed: bool,
    submit: bool,
}

impl Input {
    pub fn target(&self) -> Option<&Target> {
        self.target.as_ref()
    }

    pub fn draft(&self) -> Option<&State> {
        self.draft.as_ref()
    }

    pub fn draft_for(&self, target: &Target) -> Option<&State> {
        (self.target.as_ref() == Some(target))
            .then_some(self.draft.as_ref())
            .flatten()
    }

    pub fn preedit(&self) -> Option<&text::Preedit> {
        self.preedit.as_ref()
    }

    pub fn preedit_for(&self, target: &Target) -> Option<&text::Preedit> {
        (self.target.as_ref() == Some(target))
            .then_some(self.preedit.as_ref())
            .flatten()
    }

    pub(super) fn set_preedit(&mut self, target: Target, preedit: text::Preedit) -> bool {
        if preedit.text().is_empty() {
            if self.target.as_ref() == Some(&target) && self.draft.is_some() {
                let changed = self.preedit.is_some();
                self.preedit = None;
                return changed;
            }

            return if self.target.as_ref() == Some(&target) {
                self.clear()
            } else {
                false
            };
        }

        let target_changed = self.target.as_ref() != Some(&target);
        let changed = target_changed || self.preedit.as_ref() != Some(&preedit);
        self.target = Some(target);
        if target_changed {
            self.draft = None;
        }
        self.preedit = Some(preedit);
        changed
    }

    pub(super) fn edit(
        &mut self,
        target: Target,
        base: impl Into<String>,
        edit: text::edit::Edit,
    ) -> Change {
        let before = self
            .draft_for(&target)
            .cloned()
            .unwrap_or_else(|| State::new(base.into()));
        let mut draft = before.clone();
        let submit = draft.apply(edit);
        let text_changed = before.text != draft.text;
        let cursor_changed = before.cursor != draft.cursor;
        let target_changed = self.target.as_ref() != Some(&target);
        let preedit_cleared = self.preedit.is_some();

        self.target = Some(target);
        self.draft = Some(draft.clone());
        self.preedit = None;

        Change {
            draft,
            text_changed,
            cursor_changed,
            changed: target_changed || text_changed || cursor_changed || preedit_cleared,
            submit,
        }
    }

    pub(super) fn clear(&mut self) -> bool {
        let changed = self.target.is_some() || self.draft.is_some() || self.preedit.is_some();
        self.target = None;
        self.draft = None;
        self.preedit = None;
        changed
    }

    pub(super) fn clear_unless(&mut self, target: &Target) -> bool {
        if self.target.as_ref() == Some(target) {
            return false;
        }

        self.clear()
    }
}

impl State {
    fn new(text: impl Into<String>) -> Self {
        let text = text.into();
        let cursor = text.len();
        Self { text, cursor }
    }

    pub fn text(&self) -> &str {
        &self.text
    }

    pub fn cursor(&self) -> usize {
        self.cursor
    }

    fn apply(&mut self, edit: text::edit::Edit) -> bool {
        match edit {
            text::edit::Edit::Insert(text) | text::edit::Edit::ImeCommit(text) => {
                self.insert(&text);
                false
            }
            text::edit::Edit::ReplaceRange { range, text } => {
                self.replace_range(range, &text);
                false
            }
            text::edit::Edit::Backspace => {
                self.backspace();
                false
            }
            text::edit::Edit::Delete => {
                self.delete();
                false
            }
            text::edit::Edit::InsertLineBreak => true,
            text::edit::Edit::MovePosition(motion) | text::edit::Edit::ExtendPosition(motion) => {
                self.move_cursor(motion);
                false
            }
            text::edit::Edit::DeleteWordBackward => {
                self.delete_word_backward();
                false
            }
            text::edit::Edit::DeleteWordForward => {
                self.delete_word_forward();
                false
            }
            text::edit::Edit::SelectAll => false,
            text::edit::Edit::SetPosition(position) => {
                self.set_cursor(position.index);
                false
            }
            text::edit::Edit::Pointer { position, .. } => {
                self.set_cursor(position.index);
                false
            }
            text::edit::Edit::MoveRange { .. } => false,
        }
    }

    fn insert(&mut self, text: &str) {
        if text.is_empty() {
            return;
        }

        self.cursor = unicode::floor_boundary(&self.text, self.cursor);
        self.text.insert_str(self.cursor, text);
        self.cursor += text.len();
    }

    fn replace_range(&mut self, range: text::TextRange, value: &str) {
        let start = unicode::floor_boundary(&self.text, range.start.min(self.text.len()));
        let end = unicode::floor_boundary(&self.text, range.end.min(self.text.len()));
        let range = start.min(end)..start.max(end);
        self.text.replace_range(range.clone(), value);
        self.cursor = range.start + value.len();
    }

    fn backspace(&mut self) {
        let cursor = unicode::floor_boundary(&self.text, self.cursor);
        let previous = unicode::previous_grapheme_boundary(&self.text, cursor);
        if previous == cursor {
            return;
        }

        self.text.replace_range(previous..cursor, "");
        self.cursor = previous;
    }

    fn delete(&mut self) {
        let cursor = unicode::floor_boundary(&self.text, self.cursor);
        let next = unicode::next_grapheme_boundary(&self.text, cursor);
        if next == cursor {
            return;
        }

        self.text.replace_range(cursor..next, "");
        self.cursor = cursor;
    }

    fn delete_word_backward(&mut self) {
        let cursor = unicode::floor_boundary(&self.text, self.cursor);
        let previous = unicode::previous_word_boundary(&self.text, cursor);
        if previous == cursor {
            return;
        }

        self.text.replace_range(previous..cursor, "");
        self.cursor = previous;
    }

    fn delete_word_forward(&mut self) {
        let cursor = unicode::floor_boundary(&self.text, self.cursor);
        let next = unicode::next_word_boundary(&self.text, cursor);
        if next == cursor {
            return;
        }

        self.text.replace_range(cursor..next, "");
        self.cursor = cursor;
    }

    fn move_cursor(&mut self, motion: text::TextMotion) {
        match motion {
            text::TextMotion::VisualLeft
            | text::TextMotion::LogicalPrevious
            | text::TextMotion::WordPrevious => {
                self.cursor = if motion == text::TextMotion::WordPrevious {
                    unicode::previous_word_boundary(&self.text, self.cursor)
                } else {
                    unicode::previous_grapheme_boundary(&self.text, self.cursor)
                };
            }
            text::TextMotion::VisualRight
            | text::TextMotion::LogicalNext
            | text::TextMotion::WordNext => {
                self.cursor = if motion == text::TextMotion::WordNext {
                    unicode::next_word_boundary(&self.text, self.cursor)
                } else {
                    unicode::next_grapheme_boundary(&self.text, self.cursor)
                };
            }
            text::TextMotion::LineStart
            | text::TextMotion::ParagraphStart
            | text::TextMotion::DocumentStart => self.cursor = 0,
            text::TextMotion::LineEnd
            | text::TextMotion::ParagraphEnd
            | text::TextMotion::DocumentEnd => self.cursor = self.text.len(),
            text::TextMotion::VisualUp
            | text::TextMotion::VisualDown
            | text::TextMotion::PageUp
            | text::TextMotion::PageDown => {}
        }
    }

    fn set_cursor(&mut self, cursor: usize) {
        self.cursor = unicode::floor_boundary(&self.text, cursor.min(self.text.len()));
    }
}

impl Change {
    pub fn draft(&self) -> &State {
        &self.draft
    }

    pub fn text(&self) -> &str {
        self.draft.text()
    }

    pub fn text_changed(&self) -> bool {
        self.text_changed
    }

    pub fn cursor_changed(&self) -> bool {
        self.cursor_changed
    }

    pub fn changed(&self) -> bool {
        self.changed
    }

    pub fn submit(&self) -> bool {
        self.submit
    }
}
