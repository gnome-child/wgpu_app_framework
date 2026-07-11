use crate::view;

use super::super::Widget;

pub struct Label {
    text: String,
    overflow: Option<crate::text::Overflow>,
    wrap: Option<view::Wrap>,
}

impl Label {
    pub fn new(text: impl Into<String>) -> Self {
        Self {
            text: text.into(),
            overflow: None,
            wrap: None,
        }
    }

    /// Creates a label for text supplied by files, users, providers, or other
    /// sources whose length is not controlled by the program.
    pub fn world(text: impl Into<String>, overflow: crate::text::Overflow) -> Self {
        Self {
            text: text.into(),
            overflow: Some(overflow),
            wrap: None,
        }
    }

    /// Creates word-wrapped text supplied by files, users, or providers.
    pub fn wrapped(text: impl Into<String>) -> Self {
        Self {
            text: text.into(),
            overflow: None,
            wrap: Some(view::Wrap::Word),
        }
    }
}

impl Widget for Label {
    fn into_node(self) -> view::Node {
        match (self.overflow, self.wrap) {
            (Some(overflow), None) => view::Node::world_text(self.text, overflow),
            (None, Some(wrap)) => view::Node::wrapped_world_text(self.text, wrap),
            (None, None) => view::Node::label(self.text),
            (Some(_), Some(_)) => unreachable!("labels cannot ellipsize and wrap together"),
        }
    }
}
