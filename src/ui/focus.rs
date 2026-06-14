#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Reason {
    Pointer,
    Keyboard,
    Programmatic,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Visibility {
    Visible,
    Hidden,
}

impl Default for Visibility {
    fn default() -> Self {
        Self::Hidden
    }
}

impl Visibility {
    pub const fn is_visible(self) -> bool {
        matches!(self, Self::Visible)
    }
}
