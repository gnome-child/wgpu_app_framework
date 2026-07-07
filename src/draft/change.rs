#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct Change {
    text_changed: bool,
    selection_changed: bool,
    changed: bool,
    submit: bool,
}

impl Change {
    pub(super) fn new(
        text_changed: bool,
        selection_changed: bool,
        changed: bool,
        submit: bool,
    ) -> Self {
        Self {
            text_changed,
            selection_changed,
            changed,
            submit,
        }
    }

    pub fn text_changed(&self) -> bool {
        self.text_changed
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
