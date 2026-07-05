use super::super::input;
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct KeyChord(&'static str);

impl KeyChord {
    pub fn new(chord: &'static str) -> Self {
        Self(chord)
    }

    pub fn as_str(self) -> &'static str {
        self.0
    }

    pub(in crate::scratch::command) fn matches_key(
        self,
        key: input::Key,
        modifiers: input::Modifiers,
    ) -> bool {
        let Some(chord) = ParsedKeyChord::parse(self.0) else {
            return false;
        };

        chord.matches_key(key, modifiers)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct ParsedKeyChord {
    key: ParsedKey,
    modifiers: input::Modifiers,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ParsedKey {
    Character(char),
    F4,
}

#[derive(Debug, Clone)]
pub struct Spec {
    pub(in crate::scratch::command) display_name: &'static str,
    pub(in crate::scratch::command) shortcut: Option<KeyChord>,
}

impl Spec {
    pub(in crate::scratch) fn new(display_name: &'static str) -> Self {
        Self {
            display_name,
            shortcut: None,
        }
    }

    pub(in crate::scratch) fn shortcut(mut self, shortcut: &'static str) -> Self {
        self.shortcut = Some(KeyChord(shortcut));
        self
    }
}

impl ParsedKeyChord {
    fn parse(chord: &'static str) -> Option<Self> {
        let mut control = false;
        let mut shift = false;
        let mut alt = false;
        let mut super_key = false;
        let mut key = None;

        for part in chord.split('+') {
            if part.eq_ignore_ascii_case("ctrl") || part.eq_ignore_ascii_case("control") {
                control = true;
            } else if part.eq_ignore_ascii_case("shift") {
                shift = true;
            } else if part.eq_ignore_ascii_case("alt") {
                alt = true;
            } else if part.eq_ignore_ascii_case("super")
                || part.eq_ignore_ascii_case("cmd")
                || part.eq_ignore_ascii_case("meta")
            {
                super_key = true;
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
            modifiers: input::Modifiers::new(shift, control, alt, super_key),
        })
    }

    fn matches_key(self, key: input::Key, modifiers: input::Modifiers) -> bool {
        let key = match key.normalized() {
            input::Key::Character(value) => ParsedKey::Character(value),
            input::Key::F4 => ParsedKey::F4,
            _ => return false,
        };

        self.key == key && self.modifiers == modifiers
    }
}
