use super::window;

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Cursor {
    #[default]
    Default,
    Text,
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Button {
    #[default]
    Primary,
    Secondary,
    Middle,
    Back,
    Forward,
    Other(u16),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Update {
    window: window::Id,
    cursor: Cursor,
}

impl Update {
    pub(crate) fn new(window: window::Id, cursor: Cursor) -> Self {
        Self { window, cursor }
    }

    pub fn window(self) -> window::Id {
        self.window
    }

    pub fn cursor(self) -> Cursor {
        self.cursor
    }
}
