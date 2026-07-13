use super::Kind;
use crate::{interaction, session};

/// The represented boundary in which commands are resolved.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct Scope {
    focus: Option<session::Focus>,
    responder: Option<interaction::Id>,
    kind: Kind,
}

impl Scope {
    pub(crate) fn focused(focus: Option<session::Focus>) -> Self {
        Self {
            focus,
            responder: focus.and_then(|focus| focus.target_id()),
            kind: Kind::Focused,
        }
    }

    pub(crate) fn transient(focus: session::Focus) -> Self {
        Self {
            focus: Some(focus),
            responder: focus.target_id(),
            kind: Kind::Transient,
        }
    }

    pub(crate) fn captured(focus: Option<session::Focus>) -> Self {
        Self {
            focus,
            responder: focus.and_then(|focus| focus.target_id()),
            kind: Kind::Captured,
        }
    }

    pub(crate) fn contextual(
        responder: Option<interaction::Id>,
        focus: Option<session::Focus>,
    ) -> Self {
        Self {
            focus,
            responder,
            kind: Kind::Captured,
        }
    }

    pub(crate) fn focus(self) -> Option<session::Focus> {
        self.focus
    }

    pub(crate) fn kind(self) -> Kind {
        self.kind
    }

    pub(crate) fn responder(self) -> Option<interaction::Id> {
        self.responder
    }
}
