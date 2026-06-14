use crate::geometry::Rect;

use super::Node;

#[derive(Debug, Clone, PartialEq)]
pub struct Popup {
    rect: Rect,
    root: Node,
}

impl Popup {
    pub fn new(rect: Rect, root: Node) -> Self {
        Self { rect, root }
    }

    pub fn rect(&self) -> Rect {
        self.rect
    }

    pub fn root(&self) -> &Node {
        &self.root
    }
}
