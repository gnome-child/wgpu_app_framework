use std::fmt::Display;

use crate::{command, session, text, view};

use super::super::Widget;

pub struct TextBox {
    text: String,
    placeholder: Option<String>,
    focus: Option<session::Focus>,
    input: text::Input,
    commit: Option<view::TextCommit>,
    inactive_display: Option<(view::Align, view::Wrap, text::Overflow)>,
}

impl TextBox {
    pub fn new(text: impl Into<String>) -> Self {
        Self {
            text: text.into(),
            placeholder: None,
            focus: None,
            input: text::Input::unrestricted(),
            commit: None,
            inactive_display: None,
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

    pub fn input(mut self, input: text::Input) -> Self {
        self.input = input;
        self
    }

    pub fn inactive_display(
        mut self,
        align: view::Align,
        wrap: view::Wrap,
        overflow: text::Overflow,
    ) -> Self {
        self.inactive_display = Some((align, wrap, overflow));
        self
    }

    pub fn on_commit<C>(self) -> Self
    where
        C: command::Command,
        C::Args: From<String> + Clone,
    {
        self.commit_with::<C>(C::Args::from)
    }

    pub fn commit_with<C>(mut self, map: impl Fn(String) -> C::Args + Send + Sync + 'static) -> Self
    where
        C: command::Command,
        C::Args: Clone,
    {
        self.commit = Some(view::TextCommit::infallible::<C>(map));
        self
    }

    pub fn try_commit_with<C, E>(
        mut self,
        map: impl Fn(String) -> Result<C::Args, E> + Send + Sync + 'static,
    ) -> Self
    where
        C: command::Command,
        C::Args: Clone,
        E: Display,
    {
        self.commit = Some(view::TextCommit::fallible::<C, E>(map));
        self
    }

    pub(crate) fn try_commit_with_formatted<C>(
        mut self,
        map: impl Fn(String) -> Result<C::Args, String> + Send + Sync + 'static,
    ) -> Self
    where
        C: command::Command,
        C::Args: Clone,
    {
        self.commit = Some(view::TextCommit::formatted::<C>(map));
        self
    }
}

impl Widget for TextBox {
    fn into_node(self) -> view::Node {
        let text = self.text;
        let inactive_display = self.inactive_display;
        let mut text_box = view::TextBox::new(text.clone()).with_input(self.input);
        if let Some(placeholder) = self.placeholder {
            text_box = text_box.with_placeholder(placeholder);
        }
        if let Some(focus) = self.focus {
            text_box = text_box.with_focus(focus);
        }
        if inactive_display.is_some() {
            text_box = text_box.with_inactive_display();
        }

        let mut node = match self.commit {
            Some(commit) => view::Node::text_box_state_with_commit(text_box, commit),
            None => view::Node::text_box_state(text_box),
        };
        if let Some((align, wrap, overflow)) = inactive_display {
            node = node
                .with_world_text_policy(text.clone(), wrap, overflow)
                .with_world_text_alignment(align);
        }
        node
    }
}
