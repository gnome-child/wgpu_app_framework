#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Button {
    label: String,
    reserved_labels: Vec<String>,
}

impl Button {
    pub fn new(label: impl Into<String>) -> Self {
        Self {
            label: label.into(),
            reserved_labels: Vec::new(),
        }
    }

    pub fn reserve_label(mut self, label: impl Into<String>) -> Self {
        self.reserved_labels.push(label.into());
        self
    }

    pub fn reserve_labels(mut self, labels: impl IntoIterator<Item = impl Into<String>>) -> Self {
        self.reserved_labels
            .extend(labels.into_iter().map(Into::into));
        self
    }

    pub fn label(&self) -> &str {
        &self.label
    }

    pub(crate) fn measurement_labels(&self) -> impl Iterator<Item = &str> {
        std::iter::once(self.label.as_str()).chain(self.reserved_labels.iter().map(String::as_str))
    }
}
