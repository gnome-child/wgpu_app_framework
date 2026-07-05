#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Radio {
    label: String,
    selected: bool,
}

impl Radio {
    pub fn new(label: impl Into<String>, selected: bool) -> Self {
        Self {
            label: label.into(),
            selected,
        }
    }

    pub fn label(&self) -> &str {
        &self.label
    }

    pub fn selected(&self) -> bool {
        self.selected
    }
}
