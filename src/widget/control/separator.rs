use crate::view;

use super::super::Widget;

pub struct Separator;

impl Separator {
    pub fn new() -> Self {
        Self
    }
}

impl Default for Separator {
    fn default() -> Self {
        Self::new()
    }
}

impl Widget for Separator {
    fn into_node(self) -> view::Node {
        view::Node::separator()
    }
}
