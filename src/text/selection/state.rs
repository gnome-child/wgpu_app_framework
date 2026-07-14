use super::super::buffer::{Mark, MarkRange};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct State {
    pub(in crate::text) cursor: Mark,
    pub(in crate::text) selection: Option<MarkRange>,
}

impl State {
    pub fn new(cursor: Mark, selection: Option<MarkRange>) -> Self {
        Self { cursor, selection }
    }

    pub fn collapsed(cursor: Mark) -> Self {
        Self::new(cursor, None)
    }

    pub fn cursor(self) -> Mark {
        self.cursor
    }

    pub fn selection(self) -> Option<MarkRange> {
        self.selection
    }
}
