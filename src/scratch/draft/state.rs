use crate::text::{self, unicode};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct State {
    text: String,
    cursor: usize,
}

impl State {
    pub(super) fn new(text: impl Into<String>) -> Self {
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

    pub(super) fn apply(&mut self, edit: text::edit::Edit) -> bool {
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

    fn replace_range(&mut self, range: text::buffer::Range, value: &str) {
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

    fn move_cursor(&mut self, motion: text::edit::Motion) {
        match motion {
            text::edit::Motion::VisualLeft
            | text::edit::Motion::LogicalPrevious
            | text::edit::Motion::WordPrevious => {
                self.cursor = if motion == text::edit::Motion::WordPrevious {
                    unicode::previous_word_boundary(&self.text, self.cursor)
                } else {
                    unicode::previous_grapheme_boundary(&self.text, self.cursor)
                };
            }
            text::edit::Motion::VisualRight
            | text::edit::Motion::LogicalNext
            | text::edit::Motion::WordNext => {
                self.cursor = if motion == text::edit::Motion::WordNext {
                    unicode::next_word_boundary(&self.text, self.cursor)
                } else {
                    unicode::next_grapheme_boundary(&self.text, self.cursor)
                };
            }
            text::edit::Motion::LineStart
            | text::edit::Motion::ParagraphStart
            | text::edit::Motion::DocumentStart => self.cursor = 0,
            text::edit::Motion::LineEnd
            | text::edit::Motion::ParagraphEnd
            | text::edit::Motion::DocumentEnd => self.cursor = self.text.len(),
            text::edit::Motion::VisualUp
            | text::edit::Motion::VisualDown
            | text::edit::Motion::PageUp
            | text::edit::Motion::PageDown => {}
        }
    }

    fn set_cursor(&mut self, cursor: usize) {
        self.cursor = unicode::floor_boundary(&self.text, cursor.min(self.text.len()));
    }
}
