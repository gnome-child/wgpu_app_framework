#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Action {
    Copy,
    Cut,
    Delete,
    Paste,
    SelectAll,
    Undo,
    Redo,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct ActionResult {
    pub text_changed: bool,
    pub selection_changed: bool,
    pub clipboard_changed: bool,
    pub unavailable: bool,
}

impl ActionResult {
    pub fn buffer_changed(self) -> bool {
        self.text_changed || self.selection_changed
    }

    pub fn changed(self) -> bool {
        self.buffer_changed() || self.clipboard_changed
    }
}
