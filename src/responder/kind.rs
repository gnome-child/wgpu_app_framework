#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Kind {
    Captured,
    Transient,
    Focused,
    Ancestor,
    Window,
    Workspace,
    App,
    Framework,
}

impl Kind {
    pub(crate) fn structural_order(self) -> usize {
        match self {
            Self::Captured => 0,
            Self::Transient => 1,
            Self::Focused => 2,
            Self::Ancestor => 3,
            Self::Window => 4,
            Self::Workspace => 5,
            Self::App => 6,
            Self::Framework => 7,
        }
    }
}
