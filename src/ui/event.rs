use crate::geometry::{area, point};
use crate::pointer;
use crate::text;

use super::{Path, drag_drop};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
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
    F10,
    ContextMenu,
    Character(char),
    Other,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct Modifiers {
    shift: bool,
    control: bool,
    alt: bool,
    super_key: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Event {
    Resized {
        area: area::Physical,
        scale_factor: f32,
    },
    ScaleFactorChanged {
        scale_factor: f32,
    },
    CloseRequested,
    Focused(bool),
    PointerMoved {
        position: point::Logical,
        delta: point::Logical,
        target: Option<Path>,
    },
    PointerEntered {
        target: Path,
    },
    PointerLeft {
        target: Path,
    },
    PointerDown {
        position: point::Logical,
        delta: point::Logical,
        target: Option<Path>,
        button: pointer::Button,
    },
    PointerUp {
        position: point::Logical,
        delta: point::Logical,
        target: Option<Path>,
        button: pointer::Button,
    },
    ScrollWheel {
        position: point::Logical,
        delta: point::Logical,
        target: Option<Path>,
    },
    ScrollRequested {
        target: Path,
        offset: point::Logical,
    },
    TextEditRequested {
        target: Path,
        edit: text::Edit,
    },
    TextDropRequested {
        source: Option<(Path, text::Edit)>,
        target: Path,
        edit: text::Edit,
        operation: drag_drop::Operation,
    },
    KeyDown {
        key: Key,
        modifiers: Modifiers,
        target: Option<Path>,
        repeat: bool,
    },
    KeyUp {
        key: Key,
        modifiers: Modifiers,
        target: Option<Path>,
    },
    Ignored,
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

impl Key {
    pub const fn normalized(self) -> Self {
        match self {
            Self::Character(value) => Self::Character(value.to_ascii_lowercase()),
            value => value,
        }
    }
}
