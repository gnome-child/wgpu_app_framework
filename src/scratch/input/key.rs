use crate::text;

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

pub(in crate::scratch) fn edit_for_key(key: Key, modifiers: Modifiers) -> Option<text::edit::Edit> {
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
                text::edit::Motion::WordPrevious
            } else {
                text::edit::Motion::VisualLeft
            },
            extend,
        )),
        Key::ArrowRight => Some(motion_edit(
            if control {
                text::edit::Motion::WordNext
            } else {
                text::edit::Motion::VisualRight
            },
            extend,
        )),
        Key::ArrowUp if !control => Some(motion_edit(text::edit::Motion::VisualUp, extend)),
        Key::ArrowDown if !control => Some(motion_edit(text::edit::Motion::VisualDown, extend)),
        Key::Home => Some(motion_edit(
            if control {
                text::edit::Motion::DocumentStart
            } else {
                text::edit::Motion::LineStart
            },
            extend,
        )),
        Key::End => Some(motion_edit(
            if control {
                text::edit::Motion::DocumentEnd
            } else {
                text::edit::Motion::LineEnd
            },
            extend,
        )),
        Key::PageUp if !control => Some(motion_edit(text::edit::Motion::PageUp, extend)),
        Key::PageDown if !control => Some(motion_edit(text::edit::Motion::PageDown, extend)),
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

fn motion_edit(motion: text::edit::Motion, extend: bool) -> text::edit::Edit {
    if extend {
        text::edit::Edit::extend_position(motion)
    } else {
        text::edit::Edit::move_position(motion)
    }
}
