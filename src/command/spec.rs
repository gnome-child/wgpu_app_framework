use super::super::{input, keymap};
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct KeyChord {
    declared: &'static str,
    kind: KeyChordKind,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Standard {
    Undo,
    Redo,
    Cut,
    Copy,
    Paste,
    SelectAll,
    New,
    Open,
    Save,
    SaveAs,
    CloseWindow,
    CommandPalette,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub(crate) enum KeyChordKind {
    Chord(ParsedKeyChord),
    Standard(Standard),
}

impl KeyChord {
    pub fn new(chord: &'static str) -> Self {
        let kind = ParsedKeyChord::parse(chord)
            .map(KeyChordKind::Chord)
            .unwrap_or_else(|| KeyChordKind::Chord(ParsedKeyChord::invalid()));
        Self {
            declared: chord,
            kind,
        }
    }

    pub fn primary(key: char) -> Self {
        let parsed = ParsedKeyChord {
            key: ParsedKey::Character(key.to_ascii_lowercase()),
            modifiers: ParsedModifiers::primary(),
        };
        Self {
            declared: "Primary",
            kind: KeyChordKind::Chord(parsed),
        }
    }

    pub fn standard(standard: Standard) -> Self {
        Self {
            declared: standard.declared(),
            kind: KeyChordKind::Standard(standard),
        }
    }

    pub fn as_str(self) -> &'static str {
        self.declared
    }

    pub(in crate::command) fn matches_key(
        self,
        key: input::Key,
        modifiers: input::Modifiers,
        profile: keymap::Profile,
    ) -> bool {
        profile.matches(self, key, modifiers)
    }

    pub(crate) fn display_parts(
        self,
        profile: keymap::Profile,
        style: keymap::DisplayStyle,
    ) -> keymap::ShortcutDisplay {
        profile.display_parts(self, style)
    }

    pub(crate) fn kind(self) -> KeyChordKind {
        self.kind
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub(crate) struct ParsedKeyChord {
    key: ParsedKey,
    modifiers: ParsedModifiers,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum ParsedKey {
    Character(char),
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
    Invalid,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash)]
struct ParsedModifiers {
    shift: bool,
    control: bool,
    alt: bool,
    super_key: bool,
    primary: bool,
}

#[derive(Debug, Clone)]
pub struct Spec {
    pub(in crate::command) display_name: &'static str,
    pub(in crate::command) shortcut: Option<KeyChord>,
    pub(in crate::command) listing: Listing,
}

/// Whether a command may appear in surfaces that describe the current command world.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum Listing {
    #[default]
    Included,
    /// The command opens the describing surface and is not part of its description.
    Describer,
}

impl Spec {
    pub fn new(display_name: &'static str) -> Self {
        Self {
            display_name,
            shortcut: None,
            listing: Listing::Included,
        }
    }

    pub fn shortcut(mut self, shortcut: &'static str) -> Self {
        self.shortcut = Some(KeyChord::new(shortcut));
        self
    }

    pub fn key_chord(mut self, shortcut: KeyChord) -> Self {
        self.shortcut = Some(shortcut);
        self
    }

    pub fn listing(mut self, listing: Listing) -> Self {
        self.listing = listing;
        self
    }
}

impl ParsedKeyChord {
    fn parse(chord: &'static str) -> Option<Self> {
        let mut control = false;
        let mut shift = false;
        let mut alt = false;
        let mut super_key = false;
        let mut primary = false;
        let mut key = None;

        for part in chord.split('+') {
            let part = part.trim();
            if part.eq_ignore_ascii_case("primary") || part.eq_ignore_ascii_case("cmdorctrl") {
                primary = true;
            } else if part.eq_ignore_ascii_case("ctrl") || part.eq_ignore_ascii_case("control") {
                control = true;
            } else if part.eq_ignore_ascii_case("shift") {
                shift = true;
            } else if part.eq_ignore_ascii_case("alt") || part.eq_ignore_ascii_case("option") {
                alt = true;
            } else if part.eq_ignore_ascii_case("super")
                || part.eq_ignore_ascii_case("cmd")
                || part.eq_ignore_ascii_case("command")
                || part.eq_ignore_ascii_case("meta")
            {
                super_key = true;
            } else if part.eq_ignore_ascii_case("tab") {
                key = Some(ParsedKey::Tab);
            } else if part.eq_ignore_ascii_case("enter") || part.eq_ignore_ascii_case("return") {
                key = Some(ParsedKey::Enter);
            } else if part.eq_ignore_ascii_case("space") {
                key = Some(ParsedKey::Space);
            } else if part.eq_ignore_ascii_case("esc") || part.eq_ignore_ascii_case("escape") {
                key = Some(ParsedKey::Escape);
            } else if part.eq_ignore_ascii_case("backspace") {
                key = Some(ParsedKey::Backspace);
            } else if part.eq_ignore_ascii_case("delete") || part.eq_ignore_ascii_case("del") {
                key = Some(ParsedKey::Delete);
            } else if part.eq_ignore_ascii_case("left") || part.eq_ignore_ascii_case("arrowleft") {
                key = Some(ParsedKey::ArrowLeft);
            } else if part.eq_ignore_ascii_case("right") || part.eq_ignore_ascii_case("arrowright")
            {
                key = Some(ParsedKey::ArrowRight);
            } else if part.eq_ignore_ascii_case("up") || part.eq_ignore_ascii_case("arrowup") {
                key = Some(ParsedKey::ArrowUp);
            } else if part.eq_ignore_ascii_case("down") || part.eq_ignore_ascii_case("arrowdown") {
                key = Some(ParsedKey::ArrowDown);
            } else if part.eq_ignore_ascii_case("home") {
                key = Some(ParsedKey::Home);
            } else if part.eq_ignore_ascii_case("end") {
                key = Some(ParsedKey::End);
            } else if part.eq_ignore_ascii_case("pageup") || part.eq_ignore_ascii_case("pgup") {
                key = Some(ParsedKey::PageUp);
            } else if part.eq_ignore_ascii_case("pagedown") || part.eq_ignore_ascii_case("pgdn") {
                key = Some(ParsedKey::PageDown);
            } else if part.eq_ignore_ascii_case("f4") {
                key = Some(ParsedKey::F4);
            } else {
                let mut chars = part.chars();
                let value = chars.next()?;
                if chars.next().is_some() {
                    return None;
                }

                key = Some(ParsedKey::Character(value.to_ascii_lowercase()));
            }
        }

        Some(Self {
            key: key?,
            modifiers: ParsedModifiers {
                shift,
                control,
                alt,
                super_key,
                primary,
            },
        })
    }

    pub(crate) fn resolve(self, platform: keymap::Platform) -> Option<keymap::ConcreteChord> {
        let key = match self.key {
            ParsedKey::Character(value) => input::Key::Character(value),
            ParsedKey::Tab => input::Key::Tab,
            ParsedKey::Enter => input::Key::Enter,
            ParsedKey::Space => input::Key::Space,
            ParsedKey::Escape => input::Key::Escape,
            ParsedKey::Backspace => input::Key::Backspace,
            ParsedKey::Delete => input::Key::Delete,
            ParsedKey::ArrowLeft => input::Key::ArrowLeft,
            ParsedKey::ArrowRight => input::Key::ArrowRight,
            ParsedKey::ArrowUp => input::Key::ArrowUp,
            ParsedKey::ArrowDown => input::Key::ArrowDown,
            ParsedKey::Home => input::Key::Home,
            ParsedKey::End => input::Key::End,
            ParsedKey::PageUp => input::Key::PageUp,
            ParsedKey::PageDown => input::Key::PageDown,
            ParsedKey::F4 => input::Key::F4,
            ParsedKey::Invalid => return None,
        };
        Some(keymap::ConcreteChord::new(
            key,
            self.modifiers.resolve(platform),
        ))
    }

    fn invalid() -> Self {
        Self {
            key: ParsedKey::Invalid,
            modifiers: ParsedModifiers::default(),
        }
    }
}

impl ParsedModifiers {
    const fn primary() -> Self {
        Self {
            shift: false,
            control: false,
            alt: false,
            super_key: false,
            primary: true,
        }
    }

    fn resolve(self, platform: keymap::Platform) -> input::Modifiers {
        let primary_is_super = matches!(platform, keymap::Platform::Mac);
        input::Modifiers::new(
            self.shift,
            self.control || (self.primary && !primary_is_super),
            self.alt,
            self.super_key || (self.primary && primary_is_super),
        )
    }
}

impl Standard {
    fn declared(self) -> &'static str {
        match self {
            Self::Undo => "Standard::Undo",
            Self::Redo => "Standard::Redo",
            Self::Cut => "Standard::Cut",
            Self::Copy => "Standard::Copy",
            Self::Paste => "Standard::Paste",
            Self::SelectAll => "Standard::SelectAll",
            Self::New => "Standard::New",
            Self::Open => "Standard::Open",
            Self::Save => "Standard::Save",
            Self::SaveAs => "Standard::SaveAs",
            Self::CloseWindow => "Standard::CloseWindow",
            Self::CommandPalette => "Standard::CommandPalette",
        }
    }
}
