use super::super::{geometry, window as app_window};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Window {
    id: app_window::Id,
    title: String,
    size: geometry::Size,
}

impl Window {
    pub(super) fn new(id: app_window::Id, title: impl Into<String>, size: geometry::Size) -> Self {
        Self {
            id,
            title: title.into(),
            size,
        }
    }

    pub fn id(&self) -> app_window::Id {
        self.id
    }

    pub fn title(&self) -> &str {
        &self.title
    }

    pub fn size(&self) -> geometry::Size {
        self.size
    }

    pub(super) fn update(&mut self, title: &str, size: geometry::Size) {
        self.title = title.to_owned();
        self.size = size;
    }

    pub(super) fn set_size(&mut self, size: geometry::Size) {
        self.size = size;
    }
}
