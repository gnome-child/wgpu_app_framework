use std::path::PathBuf;

use crate::text;

use super::{command, interaction, response, session};

pub enum Input {
    Cancel,
    Focus(session::Focus),
    FilePathSelected(Option<PathBuf>),
    PointerMove(Option<interaction::Target>),
    PointerDown(interaction::Target),
    PointerDrag(Option<interaction::Target>),
    PointerUp(Option<interaction::Target>),
    PointerLeft,
    Scroll {
        target: interaction::Target,
        delta: interaction::ScrollDelta,
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
    TextPreedit(text::Preedit),
    TextDrop(TextDrop),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TextDrop {
    edit: text::edit::Edit,
    source_cleanup: Option<text::edit::Edit>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Key {
    Tab,
    Enter,
    Space,
    Escape,
    Backspace,
    Delete,
    ArrowLeft,
    ArrowRight,
    ArrowUp,
    ArrowDown,
    Home,
    End,
    PageUp,
    PageDown,
    F4,
    Character(char),
    Other,
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Modifiers {
    shift: bool,
    control: bool,
    alt: bool,
    super_key: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Outcome {
    handled: bool,
    changed_state: bool,
    effect: response::Effect,
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

    pub fn text_preedit(preedit: text::Preedit) -> Self {
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
        Self::PointerDown(target)
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

impl Key {
    pub const fn normalized(self) -> Self {
        match self {
            Self::Character(value) => Self::Character(value.to_ascii_lowercase()),
            value => value,
        }
    }
}

impl Modifiers {
    pub const fn new(shift: bool, control: bool, alt: bool, super_key: bool) -> Self {
        Self {
            shift,
            control,
            alt,
            super_key,
        }
    }

    pub const fn shift(self) -> bool {
        self.shift
    }

    pub const fn control(self) -> bool {
        self.control
    }

    pub const fn alt(self) -> bool {
        self.alt
    }

    pub const fn super_key(self) -> bool {
        self.super_key
    }
}

pub(super) fn edit_for_key(key: Key, modifiers: Modifiers) -> Option<text::edit::Edit> {
    if modifiers.alt() || modifiers.super_key() {
        return None;
    }

    let key = key.normalized();
    let control = modifiers.control();
    let extend = modifiers.shift();

    match key {
        Key::Backspace if control => Some(text::edit::Edit::delete_word_backward()),
        Key::Backspace => Some(text::edit::Edit::backspace()),
        Key::Delete if control => Some(text::edit::Edit::delete_word_forward()),
        Key::Delete => Some(text::edit::Edit::delete()),
        Key::Enter if !control => Some(text::edit::Edit::insert_line_break()),
        Key::ArrowLeft => Some(motion_edit(
            if control {
                text::TextMotion::WordPrevious
            } else {
                text::TextMotion::VisualLeft
            },
            extend,
        )),
        Key::ArrowRight => Some(motion_edit(
            if control {
                text::TextMotion::WordNext
            } else {
                text::TextMotion::VisualRight
            },
            extend,
        )),
        Key::ArrowUp if !control => Some(motion_edit(text::TextMotion::VisualUp, extend)),
        Key::ArrowDown if !control => Some(motion_edit(text::TextMotion::VisualDown, extend)),
        Key::Home => Some(motion_edit(
            if control {
                text::TextMotion::DocumentStart
            } else {
                text::TextMotion::LineStart
            },
            extend,
        )),
        Key::End => Some(motion_edit(
            if control {
                text::TextMotion::DocumentEnd
            } else {
                text::TextMotion::LineEnd
            },
            extend,
        )),
        Key::PageUp if !control => Some(motion_edit(text::TextMotion::PageUp, extend)),
        Key::PageDown if !control => Some(motion_edit(text::TextMotion::PageDown, extend)),
        Key::Tab
        | Key::Space
        | Key::Escape
        | Key::Enter
        | Key::ArrowUp
        | Key::ArrowDown
        | Key::PageUp
        | Key::PageDown
        | Key::F4
        | Key::Character(_)
        | Key::Other => None,
    }
}

fn motion_edit(motion: text::TextMotion, extend: bool) -> text::edit::Edit {
    if extend {
        text::edit::Edit::extend_position(motion)
    } else {
        text::edit::Edit::move_position(motion)
    }
}

impl TextDrop {
    pub fn new(edit: text::edit::Edit) -> Self {
        Self {
            edit,
            source_cleanup: None,
        }
    }

    pub fn with_source_cleanup(mut self, edit: text::edit::Edit) -> Self {
        self.source_cleanup = Some(edit);
        self
    }

    pub(super) fn into_edits(self) -> (text::edit::Edit, Option<text::edit::Edit>) {
        (self.edit, self.source_cleanup)
    }
}

impl Outcome {
    pub(super) fn handled(changed_state: bool, effect: response::Effect) -> Self {
        Self {
            handled: true,
            changed_state,
            effect,
        }
    }

    pub(super) fn ignored() -> Self {
        Self {
            handled: false,
            changed_state: false,
            effect: response::Effect::None,
        }
    }

    pub fn is_handled(&self) -> bool {
        self.handled
    }

    pub fn changed_state(&self) -> bool {
        self.changed_state
    }

    pub fn effect(&self) -> &response::Effect {
        &self.effect
    }
}
