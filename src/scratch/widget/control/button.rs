use crate::scratch::{command, context::Source, view};

use super::super::{Widget, trigger::TriggerBinding};

pub struct Button {
    label: String,
    reserved_labels: Vec<String>,
    binding: Option<TriggerBinding>,
}

impl Button {
    pub fn new(label: impl Into<String>) -> Self {
        Self {
            label: label.into(),
            reserved_labels: Vec::new(),
            binding: None,
        }
    }

    pub fn reserve_label(mut self, label: impl Into<String>) -> Self {
        self.reserved_labels.push(label.into());
        self
    }

    pub fn reserve_labels(mut self, labels: impl IntoIterator<Item = impl Into<String>>) -> Self {
        self.reserved_labels
            .extend(labels.into_iter().map(Into::into));
        self
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
        let mut node = view::Node::button_state(
            view::control::Button::new(self.label).reserve_labels(self.reserved_labels),
        );
        if let Some(binding) = self.binding {
            node = node.bind_trigger(binding.trigger, binding.source);
        }
        node
    }
}
