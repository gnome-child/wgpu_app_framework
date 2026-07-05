use super::super::buffer::{Mark, mark};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct State {
    pub(in crate::text) cursor: Mark,
    pub(in crate::text) selection: Option<mark::Range>,
}

impl State {
    pub fn new(cursor: Mark, selection: Option<mark::Range>) -> Self {
        Self { cursor, selection }
    }

    pub fn collapsed(cursor: Mark) -> Self {
        Self::new(cursor, None)
    }

    pub fn cursor(self) -> Mark {
        self.cursor
    }

    pub fn selection(self) -> Option<mark::Range> {
        self.selection
    }
}
