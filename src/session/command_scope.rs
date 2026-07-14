use super::Focus;
use crate::{identity, responder};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct CommandScope {
    routing: responder::Scope,
    focus: Option<Focus>,
    table: Option<identity::Id>,
}

impl CommandScope {
    pub(crate) fn focused(focus: Option<Focus>) -> Self {
        Self::new(
            responder::Scope::focused(focus.and_then(|focus| focus.target_id())),
            focus,
            None,
        )
    }

    pub(crate) fn transient(focus: Focus) -> Self {
        Self::new(
            responder::Scope::transient(focus.target_id()),
            Some(focus),
            None,
        )
    }

    pub(crate) fn captured(focus: Option<Focus>) -> Self {
        Self::new(
            responder::Scope::captured(focus.and_then(|focus| focus.target_id())),
            focus,
            None,
        )
    }

    pub(crate) fn contextual(
        responder: Option<identity::Id>,
        focus: Option<Focus>,
        table: Option<identity::Id>,
    ) -> Self {
        Self::new(responder::Scope::contextual(responder), focus, table)
    }

    fn new(routing: responder::Scope, focus: Option<Focus>, table: Option<identity::Id>) -> Self {
        let table = table.or_else(|| {
            focus
                .and_then(Focus::table_cell_identity)
                .map(crate::table::Cell::table)
        });
        Self {
            routing,
            focus,
            table,
        }
    }

    pub(crate) fn routing(self) -> responder::Scope {
        self.routing
    }

    pub(crate) fn focus(self) -> Option<Focus> {
        self.focus
    }

    pub(crate) fn table(self) -> Option<identity::Id> {
        self.table
    }

    pub(crate) fn kind(self) -> responder::Kind {
        self.routing.kind()
    }
}
