use crate::text;

use super::super::{interaction, pointer, session};
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
        cursor: pointer::Cursor,
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
    ResizeTableColumn {
        column: crate::table::HeaderCell,
        width: i32,
    },
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

    #[cfg(test)]
    pub(crate) fn pointer_down(target: interaction::Target) -> Self {
        Self::pointer_press(
            target,
            interaction::PressIntent::Activate,
            pointer::Cursor::Default,
        )
    }

    pub(crate) fn pointer_press(
        target: interaction::Target,
        intent: interaction::PressIntent,
        cursor: pointer::Cursor,
    ) -> Self {
        Self::PointerDown {
            target,
            intent,
            cursor,
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

    pub(crate) fn resize_table_column(column: crate::table::HeaderCell, width: i32) -> Self {
        Self::ResizeTableColumn { column, width }
    }

    pub(crate) fn text_focus(focus: Option<session::Focus>) -> Option<Self> {
        focus.map(Self::focus)
    }

    pub(crate) fn text_pointer_focus(focus: Option<session::Focus>) -> Option<Self> {
        focus.map(|focus| Self::focus(focus.pointer()))
    }

    pub(crate) fn text_pointer(
        focus: Option<session::Focus>,
        kind: text::edit::PointerEditKind,
        position: text::buffer::Position,
    ) -> Option<Self> {
        Some(Self::sequence([
            Self::text_pointer_focus(focus)?,
            Self::text_edit(text::edit::Edit::pointer(kind, position)),
        ]))
    }

    pub(crate) fn text_drag(position: text::buffer::Position) -> Self {
        Self::text_edit(text::edit::Edit::pointer(
            text::edit::PointerEditKind::Drag,
            position,
        ))
    }
}
