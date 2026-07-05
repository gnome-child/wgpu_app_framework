use crate::scratch::input;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum Shortcut {
    SelectAll,
    ClearSelection,
    Copy,
    Cut,
    Paste,
    Undo,
    Redo,
}

impl Shortcut {
    pub(super) fn for_key(key: input::Key, modifiers: input::Modifiers) -> Option<Self> {
        if modifiers.alt() || modifiers.super_key() || !modifiers.control() {
            return None;
        }

        match (key.normalized(), modifiers.shift()) {
            (input::Key::Character('a'), false) | (input::Key::Character('/'), false) => {
                Some(Self::SelectAll)
            }
            (input::Key::Character('a'), true) | (input::Key::Character('\\'), false) => {
                Some(Self::ClearSelection)
            }
            (input::Key::Character('c'), false) => Some(Self::Copy),
            (input::Key::Character('x'), false) => Some(Self::Cut),
            (input::Key::Character('v'), false) => Some(Self::Paste),
            (input::Key::Character('z'), false) => Some(Self::Undo),
            (input::Key::Character('z'), true) | (input::Key::Character('y'), false) => {
                Some(Self::Redo)
            }
            _ => None,
        }
    }

    pub(super) fn for_chord(shortcut: &'static str) -> Option<Self> {
        match shortcut.to_ascii_lowercase().as_str() {
            "ctrl+a" | "control+a" | "ctrl+/" | "control+/" => Some(Self::SelectAll),
            "ctrl+shift+a" | "control+shift+a" | "ctrl+\\" | "control+\\" => {
                Some(Self::ClearSelection)
            }
            "ctrl+c" | "control+c" => Some(Self::Copy),
            "ctrl+x" | "control+x" => Some(Self::Cut),
            "ctrl+v" | "control+v" => Some(Self::Paste),
            "ctrl+z" | "control+z" => Some(Self::Undo),
            "ctrl+shift+z" | "control+shift+z" | "ctrl+y" | "control+y" => Some(Self::Redo),
            _ => None,
        }
    }
}
