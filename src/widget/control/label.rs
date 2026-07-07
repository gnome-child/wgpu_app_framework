use crate::view;

use super::super::Widget;

pub struct Label {
    text: String,
}

impl Label {
    pub fn new(text: impl Into<String>) -> Self {
        Self { text: text.into() }
    }
}

impl Widget for Label {
    fn into_node(self) -> view::Node {
        view::Node::label(self.text)
    }
}
