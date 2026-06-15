use crate::geometry::Rect;
use crate::ui;

#[derive(Debug, Clone, PartialEq)]
pub struct Popup {
    rect: Rect,
    root: ui::Node,
}

impl Popup {
    pub fn new(rect: Rect, root: ui::Node) -> Self {
        Self { rect, root }
    }

    pub fn rect(&self) -> Rect {
        self.rect
    }

    pub fn root(&self) -> &ui::Node {
        &self.root
    }
}
