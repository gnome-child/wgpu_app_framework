use crate::{command, context::Source, view};

use super::super::{Widget, trigger::TriggerBinding};

pub struct Radio {
    label: String,
    selected: bool,
    binding: Option<TriggerBinding>,
}

impl Radio {
    pub fn new(label: impl Into<String>, selected: bool) -> Self {
        Self {
            label: label.into(),
            selected,
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

impl Widget for Radio {
    fn into_node(self) -> view::Node {
        let mut node = view::Node::radio(self.label, self.selected);
        if let Some(binding) = self.binding {
            node = node.bind_trigger(binding.trigger, binding.source);
        }
        node
    }
}
