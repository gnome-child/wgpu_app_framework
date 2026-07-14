use crate::text;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TextDrop {
    edit: text::Edit,
    source_cleanup: Option<text::Edit>,
}

impl TextDrop {
    pub fn new(edit: text::Edit) -> Self {
        Self {
            edit,
            source_cleanup: None,
        }
    }

    pub fn with_source_cleanup(mut self, edit: text::Edit) -> Self {
        self.source_cleanup = Some(edit);
        self
    }

    pub(crate) fn into_edits(self) -> (text::Edit, Option<text::Edit>) {
        (self.edit, self.source_cleanup)
    }
}
