use crate::{document, interaction, session, text, view};

use super::super::Widget;

pub struct TextArea {
    buffer: text::Buffer,
    state: text::selection::State,
    wrap: view::Wrap,
    id: Option<interaction::Id>,
    focus: Option<session::Focus>,
    configuration: Option<crate::scroll::Configuration>,
}

impl TextArea {
    pub fn new(text: impl Into<String>) -> Self {
        let buffer = text::Buffer::from_multiline_text(text);
        let state = buffer.initial_state();
        Self::from_buffer(buffer, state)
    }

    pub fn from_buffer(buffer: text::Buffer, state: text::selection::State) -> Self {
        Self {
            buffer,
            state,
            wrap: view::Wrap::Word,
            id: None,
            focus: None,
            configuration: None,
        }
    }

    pub fn from_document(document: &document::Document) -> Self {
        Self::from_buffer(document.buffer().clone(), document.text_state())
    }

    pub fn id(mut self, id: impl Into<interaction::Id>) -> Self {
        self.id = Some(id.into());
        self
    }

    pub fn wrap(mut self, wrap: view::Wrap) -> Self {
        self.wrap = wrap;
        self
    }

    pub fn focus(mut self, focus: session::Focus) -> Self {
        self.focus = Some(focus);
        self
    }

    pub fn configuration(mut self, configuration: crate::scroll::Configuration) -> Self {
        self.configuration = Some(configuration);
        self
    }
}

impl Widget for TextArea {
    fn into_node(self) -> view::Node {
        let id = self
            .id
            .or_else(|| self.focus.and_then(|focus| focus.target_id()));
        let mut text_area =
            view::TextArea::from_buffer(self.buffer, self.state).with_wrap(self.wrap);
        if let Some(focus) = self.focus {
            text_area = text_area.with_focus(focus);
        }

        let mut node = view::Node::text_area_state(text_area);
        if let Some(id) = id {
            node = node.with_interaction_id(id);
        }
        if let Some(configuration) = self.configuration {
            node = node.with_scroll_configuration(configuration);
        }
        node
    }
}
