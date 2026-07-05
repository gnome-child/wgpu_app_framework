use crate::text;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TextDrop {
    edit: text::edit::Edit,
    source_cleanup: Option<text::edit::Edit>,
}

impl TextDrop {
    pub fn new(edit: text::edit::Edit) -> Self {
        Self {
            edit,
            source_cleanup: None,
        }
    }

    pub fn with_source_cleanup(mut self, edit: text::edit::Edit) -> Self {
        self.source_cleanup = Some(edit);
        self
    }

    pub(in crate::scratch) fn into_edits(self) -> (text::edit::Edit, Option<text::edit::Edit>) {
        (self.edit, self.source_cleanup)
    }
}
