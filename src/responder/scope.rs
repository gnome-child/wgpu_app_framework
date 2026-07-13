use super::Kind;
use crate::{interaction, session};

/// The represented boundary in which commands are resolved.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct Scope {
    focus: Option<session::Focus>,
    responder: Option<interaction::Id>,
    kind: Kind,
    table: Option<interaction::Id>,
}

impl Scope {
    pub(crate) fn focused(focus: Option<session::Focus>) -> Self {
        Self {
            focus,
            responder: focus.and_then(|focus| focus.target_id()),
            kind: Kind::Focused,
            table: focus
                .and_then(session::Focus::table_cell_identity)
                .map(crate::table::Cell::table),
        }
    }

    pub(crate) fn transient(focus: session::Focus) -> Self {
        Self {
            focus: Some(focus),
            responder: focus.target_id(),
            kind: Kind::Transient,
            table: focus.table_cell_identity().map(crate::table::Cell::table),
        }
    }

    pub(crate) fn captured(focus: Option<session::Focus>) -> Self {
        Self {
            focus,
            responder: focus.and_then(|focus| focus.target_id()),
            kind: Kind::Captured,
            table: focus
                .and_then(session::Focus::table_cell_identity)
                .map(crate::table::Cell::table),
        }
    }

    #[cfg(test)]
    pub(crate) fn contextual(
        responder: Option<interaction::Id>,
        focus: Option<session::Focus>,
    ) -> Self {
        Self::contextual_table(responder, focus, None)
    }

    pub(crate) fn contextual_table(
        responder: Option<interaction::Id>,
        focus: Option<session::Focus>,
        table: Option<interaction::Id>,
    ) -> Self {
        Self {
            focus,
            responder,
            kind: Kind::Captured,
            table: table.or_else(|| {
                focus
                    .and_then(session::Focus::table_cell_identity)
                    .map(crate::table::Cell::table)
            }),
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

    pub(crate) fn table(self) -> Option<interaction::Id> {
        self.table
    }
}
