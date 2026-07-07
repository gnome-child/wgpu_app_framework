use crate::{command, context::Source, session, view};

use super::super::{Widget, trigger::TextBoxBinding};

pub struct TextBox {
    text: String,
    placeholder: Option<String>,
    focus: Option<session::Focus>,
    binding: Option<TextBoxBinding>,
}

impl TextBox {
    pub fn new(text: impl Into<String>) -> Self {
        Self {
            text: text.into(),
            placeholder: None,
            focus: None,
            binding: None,
        }
    }

    pub fn placeholder(mut self, placeholder: impl Into<String>) -> Self {
        self.placeholder = Some(placeholder.into());
        self
    }

    pub fn focus(mut self, focus: session::Focus) -> Self {
        self.focus = Some(focus);
        self
    }

    pub fn on_submit<C>(self) -> Self
    where
        C: command::Command,
        C::Args: From<String> + Clone,
    {
        self.submit_with::<C, _>(C::Args::from)
    }

    pub fn submit_with<C, F>(mut self, map: F) -> Self
    where
        C: command::Command,
        C::Args: Clone,
        F: Fn(String) -> C::Args + Send + Sync + 'static,
    {
        self.binding = Some(TextBoxBinding::command::<C>(Source::Input, map));
        self
    }
}

impl Widget for TextBox {
    fn into_node(self) -> view::Node {
        let text = self.text;
        let mut text_box = view::TextBox::new(text.clone());
        if let Some(placeholder) = self.placeholder {
            text_box = text_box.with_placeholder(placeholder);
        }
        if let Some(focus) = self.focus {
            text_box = text_box.with_focus(focus);
        }

        let mut node = view::Node::text_box_state(text_box);
        if let Some(binding) = self.binding {
            node = binding.bind(node, text);
        }

        node
    }
}
