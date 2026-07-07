#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct Representations {
    text: Option<String>,
}

impl Representations {
    pub(super) fn set_text(&mut self, text: String) {
        self.text = Some(text);
    }

    pub(super) fn clear_text(&mut self) {
        self.text = None;
    }

    pub(super) fn text(&self) -> Option<&str> {
        self.text.as_deref()
    }

    pub(super) fn non_empty_text(&self) -> Option<String> {
        self.text.as_ref().filter(|text| !text.is_empty()).cloned()
    }
}
