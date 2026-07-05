use super::Id;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Menu {
    id: Id,
    label: String,
}

impl Menu {
    pub fn new(id: impl Into<Id>, label: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            label: label.into(),
        }
    }

    pub fn id(&self) -> Id {
        self.id
    }

    pub fn label(&self) -> &str {
        &self.label
    }
}
