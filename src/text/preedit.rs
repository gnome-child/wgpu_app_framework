#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Preedit {
    text: String,
    selection: Option<(usize, usize)>,
}

impl Preedit {
    pub fn new(text: impl Into<String>, selection: Option<(usize, usize)>) -> Self {
        Self {
            text: text.into(),
            selection,
        }
    }

    pub fn text(&self) -> &str {
        &self.text
    }

    pub fn selection(&self) -> Option<(usize, usize)> {
        self.selection
    }
}
