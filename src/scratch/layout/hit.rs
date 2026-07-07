use super::super::{geometry::Point, interaction, view};
use super::{chrome, engine, frame::Frame};

#[derive(Clone)]
pub struct Hit {
    frame: Frame,
    chrome: Option<chrome::Chrome>,
}

impl Hit {
    pub(super) fn new(frame: Frame) -> Self {
        Self {
            frame,
            chrome: None,
        }
    }

    pub(super) fn chrome(frame: Frame, chrome: chrome::Chrome) -> Self {
        Self {
            frame,
            chrome: Some(chrome),
        }
    }

    pub fn frame(&self) -> &Frame {
        &self.frame
    }

    pub(in crate::scratch) fn is_chrome(&self) -> bool {
        self.chrome.is_some()
    }

    pub fn target(&self) -> Option<&interaction::Target> {
        if let Some(chrome) = &self.chrome {
            return Some(chrome.target());
        }

        self.frame.target()
    }

    pub fn action(&self) -> Option<&view::Action> {
        if self.chrome.is_some() {
            return None;
        }

        self.frame.action()
    }

    pub fn action_at(&self, point: Point) -> Option<view::Action> {
        if self.chrome.is_some() {
            return None;
        }

        self.frame.action_at(point)
    }

    pub fn action_at_with_engine(
        &self,
        point: Point,
        engine: &mut engine::Engine,
    ) -> Option<view::Action> {
        if self.chrome.is_some() {
            return None;
        }

        self.frame.action_at_with_engine(point, engine)
    }

    pub fn drag_action_at_with_engine(
        &self,
        point: Point,
        engine: &mut engine::Engine,
    ) -> Option<view::Action> {
        if let Some(chrome) = &self.chrome {
            return Some(view::Action::scroll_to(
                chrome.scroll_target().clone(),
                chrome.scroll_offset_at(point),
            ));
        }

        self.frame.drag_action_at_with_engine(point, engine)
    }
}
