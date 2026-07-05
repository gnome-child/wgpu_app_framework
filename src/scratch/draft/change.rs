use super::State;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Change {
    draft: State,
    text_changed: bool,
    cursor_changed: bool,
    changed: bool,
    submit: bool,
}

impl Change {
    pub(super) fn new(
        draft: State,
        text_changed: bool,
        cursor_changed: bool,
        changed: bool,
        submit: bool,
    ) -> Self {
        Self {
            draft,
            text_changed,
            cursor_changed,
            changed,
            submit,
        }
    }

    pub fn draft(&self) -> &State {
        &self.draft
    }

    pub fn text(&self) -> &str {
        self.draft.text()
    }

    pub fn text_changed(&self) -> bool {
        self.text_changed
    }

    pub fn cursor_changed(&self) -> bool {
        self.cursor_changed
    }

    pub fn changed(&self) -> bool {
        self.changed
    }

    pub fn submit(&self) -> bool {
        self.submit
    }
}
