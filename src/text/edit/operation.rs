use super::super::buffer::{Position, Range};
use super::Motion;

#[cfg(test)]
use super::super::buffer::Cursor;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Edit {
    Insert(String),
    ImeCommit(String),
    ReplaceRange {
        range: Range,
        text: String,
    },
    MoveRange {
        range: Range,
        to: Position,
    },
    Backspace,
    Delete,
    InsertLineBreak,
    MovePosition(Motion),
    ExtendPosition(Motion),
    DeleteWordBackward,
    DeleteWordForward,
    SelectAll,
    SetPosition(Position),
    Pointer {
        kind: PointerEditKind,
        position: Position,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PointerEditKind {
    Click,
    DoubleClick,
    TripleClick,
    Drag,
}

impl Edit {
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

    pub fn move_position(motion: Motion) -> Self {
        Self::MovePosition(motion)
    }

    pub fn extend_position(motion: Motion) -> Self {
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

    pub fn set_position(position: impl Into<Position>) -> Self {
        Self::SetPosition(position.into())
    }

    pub fn pointer(kind: PointerEditKind, position: impl Into<Position>) -> Self {
        Self::Pointer {
            kind,
            position: position.into(),
        }
    }
}
