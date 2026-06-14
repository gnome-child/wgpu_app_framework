use crate::geometry::{area, point};

use super::Path;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Button {
    Left,
    Right,
    Middle,
    Other(u16),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Key {
    Tab,
    Enter,
    Space,
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
        target: Option<Path>,
        button: Button,
    },
    PointerUp {
        position: point::Logical,
        target: Option<Path>,
        button: Button,
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
