use crate::text;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Outcome {
    text_changed: bool,
    selection_changed: bool,
    clipboard_changed: bool,
    unavailable: bool,
}

impl Outcome {
    pub(in crate::scratch::document) fn unavailable_result() -> Self {
        Self {
            unavailable: true,
            text_changed: false,
            selection_changed: false,
            clipboard_changed: false,
        }
    }

    pub(in crate::scratch::document) fn from_edit_result(result: text::edit::Outcome) -> Self {
        Self {
            text_changed: result.text_changed,
            selection_changed: result.selection_changed,
            clipboard_changed: false,
            unavailable: false,
        }
    }

    pub(in crate::scratch::document) fn from_command_result(
        result: text::edit::ActionResult,
    ) -> Self {
        Self {
            text_changed: result.text_changed,
            selection_changed: result.selection_changed,
            clipboard_changed: result.clipboard_changed,
            unavailable: result.unavailable,
        }
    }

    pub fn text_changed(self) -> bool {
        self.text_changed
    }

    pub fn selection_changed(self) -> bool {
        self.selection_changed
    }

    pub fn clipboard_changed(self) -> bool {
        self.clipboard_changed
    }

    pub fn unavailable(self) -> bool {
        self.unavailable
    }

    pub fn buffer_changed(self) -> bool {
        self.text_changed || self.selection_changed
    }
}
