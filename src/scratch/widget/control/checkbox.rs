use crate::scratch::{command, context::Source, view};

use super::super::{Widget, trigger::TriggerBinding};

pub struct Checkbox {
    label: String,
    checked: bool,
    binding: Option<TriggerBinding>,
}

impl Checkbox {
    pub fn new(label: impl Into<String>, checked: bool) -> Self {
        Self {
            label: label.into(),
            checked,
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

impl Widget for Checkbox {
    fn into_node(self) -> view::Node {
        let mut node = view::Node::checkbox(self.label, self.checked);
        if let Some(binding) = self.binding {
            node = node.bind_trigger(binding.trigger, binding.source);
        }
        node
    }
}
