use crate::text;

use super::super::{interaction, session};
use super::Binding;

#[derive(Clone)]
pub(crate) enum Action {
    Sequence(Vec<Action>),
    Activate(Binding),
    Focus(session::Focus),
    PointerMove(Option<interaction::Target>),
    PointerDown {
        target: interaction::Target,
        intent: interaction::PressIntent,
    },
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
    ScrollTo {
        target: interaction::Target,
        offset: interaction::ScrollOffset,
    },
    ToggleMenu(interaction::Menu),
    TextEdit(text::edit::Edit),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FocusDirection {
    Forward,
    Backward,
}

impl Action {
    pub(crate) fn sequence(actions: impl IntoIterator<Item = Action>) -> Self {
        Self::Sequence(actions.into_iter().collect())
    }

    pub(crate) fn activate(binding: &Binding) -> Self {
        Self::Activate(binding.clone())
    }

    pub(crate) fn focus(focus: session::Focus) -> Self {
        Self::Focus(focus)
    }

    pub(crate) fn pointer_move(target: Option<interaction::Target>) -> Self {
        Self::PointerMove(target)
    }

    pub(crate) fn pointer_down(target: interaction::Target) -> Self {
        Self::PointerDown {
            target,
            intent: interaction::PressIntent::Activate,
        }
    }

    pub(crate) fn pointer_manipulate(target: interaction::Target) -> Self {
        Self::PointerDown {
            target,
            intent: interaction::PressIntent::Manipulate,
        }
    }

    pub(crate) fn pointer_drag(
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

    pub(crate) fn pointer_up(target: Option<interaction::Target>, action: Option<Action>) -> Self {
        Self::PointerUp {
            target,
            action: action.map(Box::new),
        }
    }

    pub(crate) fn pointer_left() -> Self {
        Self::PointerLeft
    }

    pub(crate) fn scroll(target: interaction::Target, delta: interaction::ScrollDelta) -> Self {
        Self::Scroll { target, delta }
    }

    pub(crate) fn scroll_to(
        target: interaction::Target,
        offset: interaction::ScrollOffset,
    ) -> Self {
        Self::ScrollTo { target, offset }
    }

    pub(crate) fn toggle_menu(menu: interaction::Menu) -> Self {
        Self::ToggleMenu(menu)
    }

    pub(crate) fn text_edit(edit: text::edit::Edit) -> Self {
        Self::TextEdit(edit)
    }

    pub(crate) fn text_focus(focus: Option<session::Focus>) -> Option<Self> {
        focus.map(Self::focus)
    }

    pub(crate) fn text_pointer_focus(focus: Option<session::Focus>) -> Option<Self> {
        focus.map(|focus| Self::focus(focus.pointer()))
    }

    pub(crate) fn text_click(
        focus: Option<session::Focus>,
        position: text::buffer::Position,
    ) -> Option<Self> {
        Some(Self::sequence([
            Self::text_pointer_focus(focus)?,
            Self::text_edit(text::edit::Edit::pointer(
                text::edit::PointerEditKind::Click,
                position,
            )),
        ]))
    }

    pub(crate) fn text_drag(position: text::buffer::Position) -> Self {
        Self::text_edit(text::edit::Edit::pointer(
            text::edit::PointerEditKind::Drag,
            position,
        ))
    }
}
