use crate::view;

use super::super::Widget;

pub struct Label {
    content: Content,
}

enum Content {
    Author(String),
    World {
        text: String,
        wrap: view::Wrap,
        overflow: crate::text::Overflow,
    },
}

impl Label {
    pub fn new(text: impl Into<String>) -> Self {
        Self {
            content: Content::Author(text.into()),
        }
    }

    /// Creates a label for text supplied by files, users, providers, or other
    /// sources whose length is not controlled by the program.
    pub fn world(text: impl Into<String>, overflow: crate::text::Overflow) -> Self {
        Self {
            content: Content::World {
                text: text.into(),
                wrap: view::Wrap::None,
                overflow,
            },
        }
    }

    /// Creates word-wrapped text supplied by files, users, or providers.
    pub fn wrapped(text: impl Into<String>) -> Self {
        Self {
            content: Content::World {
                text: text.into(),
                wrap: view::Wrap::Word,
                overflow: crate::text::Overflow::Clip,
            },
        }
    }
}

impl Widget for Label {
    fn into_node(self) -> view::Node {
        match self.content {
            Content::Author(text) => view::Node::label(text),
            Content::World {
                text,
                wrap: view::Wrap::None,
                overflow,
            } => view::Node::world_text(text, overflow),
            Content::World {
                text,
                wrap,
                overflow,
            } => view::Node::world_text_with_policy(text, wrap, overflow),
        }
    }
}
