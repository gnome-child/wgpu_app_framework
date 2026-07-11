use crate::view;

use super::super::Widget;

pub struct Label {
    text: String,
    overflow: Option<crate::text::Overflow>,
}

impl Label {
    pub fn new(text: impl Into<String>) -> Self {
        Self {
            text: text.into(),
            overflow: None,
        }
    }

    /// Creates a label for text supplied by files, users, providers, or other
    /// sources whose length is not controlled by the program.
    pub fn world(text: impl Into<String>, overflow: crate::text::Overflow) -> Self {
        Self {
            text: text.into(),
            overflow: Some(overflow),
        }
    }
}

impl Widget for Label {
    fn into_node(self) -> view::Node {
        match self.overflow {
            Some(overflow) => view::Node::world_text(self.text, overflow),
            None => view::Node::label(self.text),
        }
    }
}
