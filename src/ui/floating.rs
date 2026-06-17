use crate::action;
use crate::geometry::{Rect, point};
use crate::widget::menu;

use super::{Id, Path};

#[derive(Debug, Clone, PartialEq)]
pub struct Surface {
    kind: Kind,
    root_id: Id,
    anchor: Anchor,
    command_context: action::Context,
    source: action::Source,
    focus_policy: FocusPolicy,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Kind {
    Menu(menu::Id),
    Submenu(menu::Id),
    ContextMenu { target: Path },
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Anchor {
    Rect(Rect),
    Point(point::Logical),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FocusPolicy {
    PreserveCurrentFocus,
    FocusFirstEnabledRow,
}

impl Surface {
    pub fn new(
        kind: Kind,
        root_id: Id,
        anchor: Anchor,
        command_context: action::Context,
        source: action::Source,
        focus_policy: FocusPolicy,
    ) -> Self {
        Self {
            kind,
            root_id,
            anchor,
            command_context,
            source,
            focus_policy,
        }
    }

    pub fn kind(&self) -> &Kind {
        &self.kind
    }

    pub fn root_id(&self) -> Id {
        self.root_id
    }

    pub fn anchor(&self) -> Anchor {
        self.anchor
    }

    pub fn command_context(&self) -> &action::Context {
        &self.command_context
    }

    pub fn source(&self) -> action::Source {
        self.source
    }

    pub fn focus_policy(&self) -> FocusPolicy {
        self.focus_policy
    }

    pub fn context_menu_target(&self) -> Option<&Path> {
        match &self.kind {
            Kind::ContextMenu { target } => Some(target),
            Kind::Menu(_) | Kind::Submenu(_) => None,
        }
    }

    pub fn menu_id(&self) -> Option<menu::Id> {
        match self.kind {
            Kind::Menu(menu) | Kind::Submenu(menu) => Some(menu),
            Kind::ContextMenu { .. } => None,
        }
    }
}
