use crate::text;

use super::super::{input, interaction, session};
use super::Binding;

#[derive(Clone)]
pub enum Action {
    Sequence(Vec<Action>),
    Command(Binding),
    Focus(session::Focus),
    PointerMove(Option<interaction::Target>),
    PointerDown(interaction::Target),
    PointerDrag {
        hovered: Option<interaction::Target>,
        target: interaction::Target,
        action: Option<Box<Action>>,
    },
    PointerUp {
        target: Option<interaction::Target>,
        action: Option<Box<Action>>,
    },
    PointerLeft,
    Scroll {
        target: interaction::Target,
        delta: interaction::ScrollDelta,
    },
    ToggleMenu(interaction::Menu),
    TextEdit(text::edit::Edit),
    TextDrop(input::TextDrop),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FocusDirection {
    Forward,
    Backward,
}

impl Action {
    pub fn sequence(actions: impl IntoIterator<Item = Action>) -> Self {
        Self::Sequence(actions.into_iter().collect())
    }

    pub fn command(binding: &Binding) -> Self {
        Self::Command(binding.clone())
    }

    pub fn focus(focus: session::Focus) -> Self {
        Self::Focus(focus)
    }

    pub fn pointer_move(target: Option<interaction::Target>) -> Self {
        Self::PointerMove(target)
    }

    pub fn pointer_down(target: interaction::Target) -> Self {
        Self::PointerDown(target)
    }

    pub fn pointer_drag(
        hovered: Option<interaction::Target>,
        target: interaction::Target,
        action: Option<Action>,
    ) -> Self {
        Self::PointerDrag {
            hovered,
            target,
            action: action.map(Box::new),
        }
    }

    pub fn pointer_up(target: Option<interaction::Target>, action: Option<Action>) -> Self {
        Self::PointerUp {
            target,
            action: action.map(Box::new),
        }
    }

    pub fn pointer_left() -> Self {
        Self::PointerLeft
    }

    pub fn scroll(target: interaction::Target, delta: interaction::ScrollDelta) -> Self {
        Self::Scroll { target, delta }
    }

    pub fn toggle_menu(menu: interaction::Menu) -> Self {
        Self::ToggleMenu(menu)
    }

    pub fn text_edit(edit: text::edit::Edit) -> Self {
        Self::TextEdit(edit)
    }

    pub fn text_drop(edit: text::edit::Edit) -> Self {
        Self::TextDrop(input::TextDrop::new(edit))
    }

    pub fn text_drop_with_source_cleanup(
        edit: text::edit::Edit,
        source_cleanup: text::edit::Edit,
    ) -> Self {
        Self::TextDrop(input::TextDrop::new(edit).with_source_cleanup(source_cleanup))
    }
}
