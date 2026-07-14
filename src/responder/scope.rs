use super::Kind;
use crate::identity;

/// The represented boundary in which commands are resolved.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct Scope {
    responder: Option<identity::Id>,
    kind: Kind,
}

impl Scope {
    pub(crate) fn focused(responder: Option<identity::Id>) -> Self {
        Self {
            responder,
            kind: Kind::Focused,
        }
    }

    pub(crate) fn transient(responder: Option<identity::Id>) -> Self {
        Self {
            responder,
            kind: Kind::Transient,
        }
    }

    pub(crate) fn captured(responder: Option<identity::Id>) -> Self {
        Self {
            responder,
            kind: Kind::Captured,
        }
    }

    pub(crate) fn contextual(responder: Option<identity::Id>) -> Self {
        Self {
            responder,
            kind: Kind::Captured,
        }
    }

    pub(crate) fn kind(self) -> Kind {
        self.kind
    }

    pub(crate) fn responder(self) -> Option<identity::Id> {
        self.responder
    }
}
