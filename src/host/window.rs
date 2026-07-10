use super::super::{geometry, window as app_window};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Window {
    facts: app_window::Facts,
}

impl Window {
    pub(super) fn new(facts: app_window::Facts) -> Self {
        Self { facts }
    }

    pub fn id(&self) -> app_window::Id {
        self.facts.id()
    }

    pub fn title(&self) -> &str {
        self.facts.title()
    }

    pub fn size(&self) -> geometry::Size {
        self.facts.inner_size()
    }

    pub(super) fn update(&mut self, facts: &app_window::Facts) {
        self.facts.clone_from(facts);
    }

    pub(super) fn set_size(&mut self, size: geometry::Size) {
        self.facts.set_inner_size(size);
    }
}
