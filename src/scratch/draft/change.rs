#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Change {
    text: String,
    text_changed: bool,
    cursor_changed: bool,
    selection_changed: bool,
    changed: bool,
    submit: bool,
}

impl Change {
    pub(super) fn new(
        text: String,
        text_changed: bool,
        cursor_changed: bool,
        selection_changed: bool,
        changed: bool,
        submit: bool,
    ) -> Self {
        Self {
            text,
            text_changed,
            cursor_changed,
            selection_changed,
            changed,
            submit,
        }
    }

    pub fn text(&self) -> &str {
        &self.text
    }

    pub fn text_changed(&self) -> bool {
        self.text_changed
    }

    pub fn cursor_changed(&self) -> bool {
        self.cursor_changed
    }

    pub fn selection_changed(&self) -> bool {
        self.selection_changed
    }

    pub fn changed(&self) -> bool {
        self.changed
    }

    pub fn submit(&self) -> bool {
        self.submit
    }
}
