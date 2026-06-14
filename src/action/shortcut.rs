use crate::ui;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Shortcut {
    key: ui::Key,
    modifiers: ui::Modifiers,
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
}
