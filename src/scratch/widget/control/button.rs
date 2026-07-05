use crate::scratch::{command, context::Source, view};

use super::super::{Widget, trigger::TriggerBinding};

pub struct Button {
    label: String,
    binding: Option<TriggerBinding>,
}

impl Button {
    pub fn new(label: impl Into<String>) -> Self {
        Self {
            label: label.into(),
            binding: None,
        }
    }

    pub fn trigger<C>(mut self, args: C::Args) -> Self
    where
        C: command::Command,
        C::Args: Clone,
    {
        self.binding = Some(TriggerBinding::command::<C>(args, Source::Button));
        self
    }
}

impl Widget for Button {
    fn into_node(self) -> view::Node {
        let mut node = view::Node::button(self.label);
        if let Some(binding) = self.binding {
            node = node.bind_trigger(binding.trigger, binding.source);
        }
        node
    }
}
