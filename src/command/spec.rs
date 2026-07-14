use super::super::{keyboard, keymap};
use super::menu;
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
    Delete,
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
        key: keyboard::Key,
        modifiers: keyboard::Modifiers,
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
    pub(in crate::command) description: Option<&'static str>,
    pub(in crate::command) shortcut: Option<KeyChord>,
    pub(in crate::command) listing: Listing,
    pub(in crate::command) standard: Option<Standard>,
    pub(in crate::command) menu_placement: Option<menu::Placement>,
    pub(in crate::command) menu_suppressed: bool,
    pub(in crate::command) menu_shortcut_visibility: Option<bool>,
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
            description: None,
            shortcut: None,
            listing: Listing::Included,
            standard: None,
            menu_placement: None,
            menu_suppressed: false,
            menu_shortcut_visibility: None,
        }
    }

    /// Declares a conventional command meaning and derives its default label and chord.
    pub fn standard(standard: Standard) -> Self {
        Self {
            display_name: standard.default_label(),
            description: None,
            shortcut: Some(KeyChord::standard(standard)),
            listing: Listing::Included,
            standard: Some(standard),
            menu_placement: None,
            menu_suppressed: false,
            menu_shortcut_visibility: None,
        }
    }

    /// Attaches conventional meaning to an explicitly labelled command.
    ///
    /// The standard chord is derived unless this spec already declares an override.
    pub fn role(mut self, standard: Standard) -> Self {
        self.standard = Some(standard);
        if self.shortcut.is_none() {
            self.shortcut = Some(KeyChord::standard(standard));
        }
        self
    }

    pub fn shortcut(mut self, shortcut: &'static str) -> Self {
        self.shortcut = Some(KeyChord::new(shortcut));
        self
    }

    /// Describes the stable meaning of this command independently of its current state.
    pub fn description(mut self, description: &'static str) -> Self {
        self.description = Some(description);
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

    /// Places this command in a conventional menu topology.
    pub fn placement(mut self, placement: menu::Placement) -> Self {
        self.menu_placement = Some(placement);
        self.menu_suppressed = false;
        self
    }

    /// Suppresses automatic bar placement while preserving role, label, and chord projections.
    pub fn unplaced(mut self) -> Self {
        self.menu_placement = None;
        self.menu_suppressed = true;
        self
    }

    /// Overrides whether a shortcut is painted beside this command in a menu.
    pub fn show_menu_shortcut(mut self, visible: bool) -> Self {
        self.menu_shortcut_visibility = Some(visible);
        self
    }

    pub fn display_name(&self) -> &'static str {
        self.display_name
    }

    pub fn declared_description(&self) -> Option<&'static str> {
        self.description
    }

    pub fn declared_key_chord(&self) -> Option<KeyChord> {
        self.shortcut
    }

    pub fn standard_role(&self) -> Option<Standard> {
        self.standard
    }

    pub(in crate::command) fn participates_in_standard_menu(&self) -> bool {
        !self.menu_suppressed
            && (self.menu_placement.is_some()
                || self.standard.is_some_and(menu::standard_is_placed))
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
            ParsedKey::Character(value) => keyboard::Key::Character(value),
            ParsedKey::Tab => keyboard::Key::Tab,
            ParsedKey::Enter => keyboard::Key::Enter,
            ParsedKey::Space => keyboard::Key::Space,
            ParsedKey::Escape => keyboard::Key::Escape,
            ParsedKey::Backspace => keyboard::Key::Backspace,
            ParsedKey::Delete => keyboard::Key::Delete,
            ParsedKey::ArrowLeft => keyboard::Key::ArrowLeft,
            ParsedKey::ArrowRight => keyboard::Key::ArrowRight,
            ParsedKey::ArrowUp => keyboard::Key::ArrowUp,
            ParsedKey::ArrowDown => keyboard::Key::ArrowDown,
            ParsedKey::Home => keyboard::Key::Home,
            ParsedKey::End => keyboard::Key::End,
            ParsedKey::PageUp => keyboard::Key::PageUp,
            ParsedKey::PageDown => keyboard::Key::PageDown,
            ParsedKey::F4 => keyboard::Key::F4,
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

    fn resolve(self, platform: keymap::Platform) -> keyboard::Modifiers {
        let primary_is_super = matches!(platform, keymap::Platform::Mac);
        keyboard::Modifiers::new(
            self.shift,
            self.control || (self.primary && !primary_is_super),
            self.alt,
            self.super_key || (self.primary && primary_is_super),
        )
    }
}

impl Standard {
    pub(crate) fn default_label(self) -> &'static str {
        match self {
            Self::Undo => "Undo",
            Self::Redo => "Redo",
            Self::Cut => "Cut",
            Self::Copy => "Copy",
            Self::Paste => "Paste",
            Self::Delete => "Delete",
            Self::SelectAll => "Select All",
            Self::New => "New",
            Self::Open => "Open",
            Self::Save => "Save",
            Self::SaveAs => "Save As",
            Self::CloseWindow => "Close Window",
            Self::CommandPalette => "Command Palette",
        }
    }

    fn declared(self) -> &'static str {
        match self {
            Self::Undo => "Standard::Undo",
            Self::Redo => "Standard::Redo",
            Self::Cut => "Standard::Cut",
            Self::Copy => "Standard::Copy",
            Self::Paste => "Standard::Paste",
            Self::Delete => "Standard::Delete",
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn standard_spec_derives_role_label_and_chord_from_one_meaning() {
        let spec = Spec::standard(Standard::Copy);

        assert_eq!(spec.standard_role(), Some(Standard::Copy));
        assert_eq!(spec.display_name(), "Copy");
        assert_eq!(
            spec.declared_key_chord().map(KeyChord::as_str),
            Some("Standard::Copy")
        );
    }

    #[test]
    fn role_sugar_preserves_explicit_label_and_chord_overrides() {
        let labelled = Spec::new("Copy Frame").role(Standard::Copy);
        assert_eq!(labelled.display_name(), "Copy Frame");
        assert_eq!(labelled.standard_role(), Some(Standard::Copy));
        assert_eq!(
            labelled.declared_key_chord().map(KeyChord::as_str),
            Some("Standard::Copy")
        );

        let chorded = Spec::new("Copy Frame")
            .shortcut("Primary+Shift+C")
            .role(Standard::Copy);
        assert_eq!(chorded.display_name(), "Copy Frame");
        assert_eq!(chorded.standard_role(), Some(Standard::Copy));
        assert_eq!(
            chorded.declared_key_chord().map(KeyChord::as_str),
            Some("Primary+Shift+C")
        );

        let standard_then_override = Spec::standard(Standard::Copy).shortcut("Primary+Shift+C");
        assert_eq!(standard_then_override.standard_role(), Some(Standard::Copy));
        assert_eq!(
            standard_then_override
                .declared_key_chord()
                .map(KeyChord::as_str),
            Some("Primary+Shift+C")
        );
    }

    #[test]
    fn description_is_stable_command_meaning() {
        let spec = Spec::new("Save").description("Writes the current document to disk");

        assert_eq!(
            spec.declared_description(),
            Some("Writes the current document to disk")
        );
    }
}
