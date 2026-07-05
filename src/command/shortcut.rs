use crate::input::{Key, Modifiers};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Shortcut {
    key: Key,
    modifiers: Modifiers,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn control_shortcut_formats_display_label() {
        assert_eq!(Shortcut::ctrl('a').display_label(), "Ctrl+A");
    }

    #[test]
    fn modifier_shortcut_formats_in_stable_order() {
        let shortcut = Shortcut::new(Key::Escape, Modifiers::new(true, true, true, true));

        assert_eq!(shortcut.display_label(), "Ctrl+Shift+Alt+Super+Esc");
    }
}

impl Shortcut {
    pub const fn new(key: Key, modifiers: Modifiers) -> Self {
        Self { key, modifiers }
    }

    pub const fn ctrl(key: char) -> Self {
        Self::new(
            Key::Character(key.to_ascii_lowercase()),
            Modifiers::new(false, true, false, false),
        )
    }

    pub const fn ctrl_shift(key: char) -> Self {
        Self::new(
            Key::Character(key.to_ascii_lowercase()),
            Modifiers::new(true, true, false, false),
        )
    }

    pub const fn key(self) -> Key {
        self.key
    }

    pub const fn modifiers(self) -> Modifiers {
        self.modifiers
    }

    pub fn matches(self, key: Key, modifiers: Modifiers) -> bool {
        self.key == key.normalized() && self.modifiers == modifiers
    }

    pub fn display_label(self) -> String {
        let mut parts = Vec::new();

        if self.modifiers.control() {
            parts.push("Ctrl".to_owned());
        }
        if self.modifiers.shift() {
            parts.push("Shift".to_owned());
        }
        if self.modifiers.alt() {
            parts.push("Alt".to_owned());
        }
        if self.modifiers.super_key() {
            parts.push("Super".to_owned());
        }

        parts.push(match self.key {
            Key::Tab => "Tab".to_owned(),
            Key::Enter => "Enter".to_owned(),
            Key::Space => "Space".to_owned(),
            Key::Escape => "Esc".to_owned(),
            Key::Backspace => "Backspace".to_owned(),
            Key::Delete => "Delete".to_owned(),
            Key::ArrowLeft => "Left".to_owned(),
            Key::ArrowRight => "Right".to_owned(),
            Key::ArrowUp => "Up".to_owned(),
            Key::ArrowDown => "Down".to_owned(),
            Key::Home => "Home".to_owned(),
            Key::End => "End".to_owned(),
            Key::PageUp => "PageUp".to_owned(),
            Key::PageDown => "PageDown".to_owned(),
            Key::F10 => "F10".to_owned(),
            Key::ContextMenu => "Menu".to_owned(),
            Key::Character(character) => character.to_ascii_uppercase().to_string(),
            Key::Other => "Other".to_owned(),
        });

        parts.join("+")
    }
}
