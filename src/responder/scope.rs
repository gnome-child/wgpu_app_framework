use super::Kind;
use crate::session;

/// The represented boundary in which commands are resolved.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct Scope {
    focus: Option<session::Focus>,
    kind: Kind,
}

impl Scope {
    pub(crate) fn focused(focus: Option<session::Focus>) -> Self {
        Self {
            focus,
            kind: Kind::Focused,
        }
    }

    pub(crate) fn transient(focus: session::Focus) -> Self {
        Self {
            focus: Some(focus),
            kind: Kind::Transient,
        }
    }

    pub(crate) fn captured(focus: Option<session::Focus>) -> Self {
        Self {
            focus,
            kind: Kind::Captured,
        }
    }

    pub(crate) fn focus(self) -> Option<session::Focus> {
        self.focus
    }

    pub(crate) fn kind(self) -> Kind {
        self.kind
    }
}
