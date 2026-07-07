use std::ops::RangeInclusive;

use crate::{command, context::Source, view};

use super::super::{
    Widget,
    trigger::{SliderBinding, SliderChangeBinding, TriggerBinding},
};

pub struct Slider {
    label: String,
    value: f64,
    range: RangeInclusive<f64>,
    binding: Option<SliderBinding>,
}

impl Slider {
    pub fn new(label: impl Into<String>, value: f64, range: RangeInclusive<f64>) -> Self {
        Self {
            label: label.into(),
            value,
            range,
            binding: None,
        }
    }

    pub fn trigger<C>(mut self, args: C::Args) -> Self
    where
        C: command::Command,
        C::Args: Clone,
    {
        self.binding = Some(SliderBinding::Fixed(TriggerBinding::command::<C>(
            args,
            Source::Button,
        )));
        self
    }

    pub fn on_change<C>(self) -> Self
    where
        C: command::Command,
        C::Args: From<f64> + Clone,
    {
        self.trigger_with::<C, _>(C::Args::from)
    }

    pub fn trigger_with<C, F>(mut self, map: F) -> Self
    where
        C: command::Command,
        C::Args: Clone,
        F: Fn(f64) -> C::Args + Send + Sync + 'static,
    {
        self.binding = Some(SliderBinding::Change(SliderChangeBinding::command::<C>(
            Source::Button,
            map,
        )));
        self
    }
}

impl Widget for Slider {
    fn into_node(self) -> view::Node {
        let start = *self.range.start();
        let end = *self.range.end();
        let mut node = view::Node::slider(self.label, self.value, start, end);
        if let Some(binding) = self.binding {
            node = binding.bind(node, self.value);
        }
        node
    }
}
