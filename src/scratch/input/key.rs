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
