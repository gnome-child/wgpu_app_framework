use std::path::PathBuf;

use crate::text;

use super::{command, interaction, session};

mod key;
mod outcome;
mod text_drop;

pub use key::{Key, Modifiers};
pub use outcome::Outcome;
pub use text_drop::TextDrop;

pub enum Input {
    Cancel,
    Focus(session::Focus),
    FilePathSelected(Option<PathBuf>),
    PointerMove(Option<interaction::Target>),
    PointerDown {
        target: interaction::Target,
        intent: interaction::PressIntent,
    },
    PointerDrag(Option<interaction::Target>),
    PointerUp(Option<interaction::Target>),
    PointerLeft,
    Scroll {
        target: interaction::Target,
        delta: interaction::ScrollDelta,
    },
    ScrollTo {
        target: interaction::Target,
        offset: interaction::ScrollOffset,
    },
    Shortcut(command::KeyChord),
    KeyDown {
        key: Key,
        modifiers: Modifiers,
        text: Option<String>,
    },
    ToggleMenu(interaction::Menu),
    TextEdit(text::edit::Edit),
    TextCommit(String),
    TextPreedit(text::edit::Preedit),
    TextDrop(TextDrop),
}

impl Input {
    pub fn cancel() -> Self {
        Self::Cancel
    }

    pub fn focus(focus: session::Focus) -> Self {
        Self::Focus(focus)
    }

    pub fn text_edit(edit: text::edit::Edit) -> Self {
        Self::TextEdit(edit)
    }

    pub fn text_commit(text: impl Into<String>) -> Self {
        Self::TextCommit(text.into())
    }

    pub fn text_preedit(preedit: text::edit::Preedit) -> Self {
        Self::TextPreedit(preedit)
    }

    pub fn text_drop(edit: text::edit::Edit) -> Self {
        Self::TextDrop(TextDrop::new(edit))
    }

    pub fn text_drop_with_source_cleanup(
        edit: text::edit::Edit,
        source_cleanup: text::edit::Edit,
    ) -> Self {
        Self::TextDrop(TextDrop::new(edit).with_source_cleanup(source_cleanup))
    }

    pub fn file_path_selected(path: Option<PathBuf>) -> Self {
        Self::FilePathSelected(path)
    }

    pub fn pointer_move(target: Option<interaction::Target>) -> Self {
        Self::PointerMove(target)
    }

    pub fn pointer_down(target: interaction::Target) -> Self {
        Self::pointer_down_with_intent(target, interaction::PressIntent::Activate)
    }

    pub fn pointer_manipulate(target: interaction::Target) -> Self {
        Self::pointer_down_with_intent(target, interaction::PressIntent::Manipulate)
    }

    pub fn pointer_down_with_intent(
        target: interaction::Target,
        intent: interaction::PressIntent,
    ) -> Self {
        Self::PointerDown { target, intent }
    }

    pub fn pointer_drag(hovered: Option<interaction::Target>) -> Self {
        Self::PointerDrag(hovered)
    }

    pub fn pointer_up(target: Option<interaction::Target>) -> Self {
        Self::PointerUp(target)
    }

    pub fn pointer_left() -> Self {
        Self::PointerLeft
    }

    pub fn scroll(target: interaction::Target, delta: interaction::ScrollDelta) -> Self {
        Self::Scroll { target, delta }
    }

    pub fn scroll_to(target: interaction::Target, offset: interaction::ScrollOffset) -> Self {
        Self::ScrollTo { target, offset }
    }

    pub fn shortcut(shortcut: &'static str) -> Self {
        Self::Shortcut(command::KeyChord::new(shortcut))
    }

    pub fn key_down(key: Key, modifiers: Modifiers) -> Self {
        Self::KeyDown {
            key,
            modifiers,
            text: None,
        }
    }

    pub fn key_down_with_text(
        key: Key,
        modifiers: Modifiers,
        text: impl Into<Option<String>>,
    ) -> Self {
        Self::KeyDown {
            key,
            modifiers,
            text: text.into(),
        }
    }

    pub fn toggle_menu(menu: interaction::Menu) -> Self {
        Self::ToggleMenu(menu)
    }
}
