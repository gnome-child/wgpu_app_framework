use crate::ui;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Shortcut {
    key: ui::Key,
    modifiers: ui::Modifiers,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn control_shortcut_formats_display_label() {
        assert_eq!(Shortcut::control('a').display_label(), "Ctrl+A");
    }

    #[test]
    fn modifier_shortcut_formats_in_stable_order() {
        let shortcut = Shortcut::new(ui::Key::Escape, ui::Modifiers::new(true, true, true, true));

        assert_eq!(shortcut.display_label(), "Ctrl+Shift+Alt+Super+Esc");
    }
}

impl Shortcut {
    pub const fn new(key: ui::Key, modifiers: ui::Modifiers) -> Self {
        Self { key, modifiers }
    }

    pub const fn control(key: char) -> Self {
        Self::new(
            ui::Key::Character(key.to_ascii_lowercase()),
            ui::Modifiers::new(false, true, false, false),
        )
    }

    pub const fn key(self) -> ui::Key {
        self.key
    }

    pub const fn modifiers(self) -> ui::Modifiers {
        self.modifiers
    }

    pub fn matches(self, key: ui::Key, modifiers: ui::Modifiers) -> bool {
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
            ui::Key::Tab => "Tab".to_owned(),
            ui::Key::Enter => "Enter".to_owned(),
            ui::Key::Space => "Space".to_owned(),
            ui::Key::Escape => "Esc".to_owned(),
            ui::Key::Character(character) => character.to_ascii_uppercase().to_string(),
            ui::Key::Other => "Other".to_owned(),
        });

        parts.join("+")
    }
}
