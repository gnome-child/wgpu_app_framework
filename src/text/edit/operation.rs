use super::super::buffer::{Position, Range};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Edit {
    Insert(String),
    ImeCommit(String),
    ReplaceRange { range: Range, text: String },
    MoveRange { range: Range, to: Position },
    Backspace,
    Delete,
    InsertLineBreak,
    DeleteWordBackward,
    DeleteWordForward,
}

impl Edit {
    pub fn insert(text: impl Into<String>) -> Self {
        Self::Insert(text.into())
    }

    pub fn ime_commit(text: impl Into<String>) -> Self {
        Self::ImeCommit(text.into())
    }

    pub fn replace_range(range: impl Into<Range>, text: impl Into<String>) -> Self {
        Self::ReplaceRange {
            range: range.into(),
            text: text.into(),
        }
    }

    pub fn insert_at(position: impl Into<Position>, text: impl Into<String>) -> Self {
        let position = position.into();
        Self::replace_range(Range::collapsed(position.index), text)
    }

    pub fn move_range(range: impl Into<Range>, to: impl Into<Position>) -> Self {
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

    pub fn delete_word_backward() -> Self {
        Self::DeleteWordBackward
    }

    pub fn delete_word_forward() -> Self {
        Self::DeleteWordForward
    }
}
