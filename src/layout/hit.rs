use super::super::{geometry::Point, interaction, view};
use super::{chrome, engine, frame::Frame};

#[derive(Clone)]
pub(crate) struct Hit {
    frame: Frame,
    chrome: Option<chrome::Chrome>,
    target: Option<interaction::Target>,
    table_cell: Option<crate::table::Cell>,
}

impl Hit {
    pub(super) fn new(frame: Frame) -> Self {
        Self {
            frame,
            chrome: None,
            target: None,
            table_cell: None,
        }
    }

    pub(super) fn chrome(frame: Frame, chrome: chrome::Chrome) -> Self {
        Self {
            frame,
            chrome: Some(chrome),
            target: None,
            table_cell: None,
        }
    }

    pub(super) fn table_divider(frame: Frame, target: interaction::Target) -> Self {
        Self {
            frame,
            chrome: None,
            target: Some(target),
            table_cell: None,
        }
    }

    pub(super) fn with_table_cell(mut self, cell: Option<crate::table::Cell>) -> Self {
        self.table_cell = cell;
        self
    }

    pub(crate) fn table_cell(&self) -> Option<crate::table::Cell> {
        self.table_cell
    }

    pub(crate) fn frame(&self) -> &Frame {
        &self.frame
    }

    pub(crate) fn is_chrome(&self) -> bool {
        self.chrome.is_some()
    }

    pub(crate) fn target(&self) -> Option<&interaction::Target> {
        if let Some(chrome) = &self.chrome {
            return Some(chrome.target());
        }

        if let Some(target) = &self.target {
            return Some(target);
        }

        self.frame.target()
    }

    #[cfg(test)]
    pub(crate) fn action(&self) -> Option<&view::Action> {
        if self.chrome.is_some() || self.target.is_some() {
            return None;
        }

        self.frame.action()
    }

    pub(crate) fn action_at(&self, point: Point) -> Option<view::Action> {
        if self.chrome.is_some() || self.target.is_some() {
            return None;
        }

        self.frame.action_at(point)
    }

    pub(crate) fn action_at_with_engine(
        &self,
        point: Point,
        engine: &mut engine::Engine,
    ) -> Option<view::Action> {
        if self.chrome.is_some() || self.target.is_some() {
            return None;
        }

        self.frame.action_at_with_engine(point, engine)
    }
}
