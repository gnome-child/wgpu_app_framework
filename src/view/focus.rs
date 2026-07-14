#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub(crate) enum Presentation {
    #[default]
    Unfocused,
    Focused,
    Visible,
}

impl Presentation {
    pub(in crate::view) fn focused(visible: bool) -> Self {
        if visible {
            Self::Visible
        } else {
            Self::Focused
        }
    }

    pub(crate) fn is_focused(self) -> bool {
        !matches!(self, Self::Unfocused)
    }

    pub(crate) fn is_visible(self) -> bool {
        matches!(self, Self::Visible)
    }
}
