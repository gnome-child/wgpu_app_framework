#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Checkbox {
    label: String,
    checked: bool,
}

impl Checkbox {
    pub fn new(label: impl Into<String>, checked: bool) -> Self {
        Self {
            label: label.into(),
            checked,
        }
    }

    pub fn label(&self) -> &str {
        &self.label
    }

    pub fn checked(&self) -> bool {
        self.checked
    }
}
