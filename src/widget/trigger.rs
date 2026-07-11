use super::super::{command, context::Source, view};

pub(super) struct TriggerBinding {
    trigger: command::AnyTrigger,
    source: Source,
}

pub(super) enum SliderBinding {
    Fixed(TriggerBinding),
    Change(SliderChangeBinding),
}

pub(super) struct SliderChangeBinding {
    trigger: command::AnyValueTrigger<f64>,
    source: Source,
}

pub(super) struct TextBoxBinding {
    trigger: command::AnyValueTrigger<String>,
    source: Source,
}

impl TriggerBinding {
    pub(super) fn command<C>(args: C::Args, source: Source) -> Self
    where
        C: command::Command,
        C::Args: Clone,
    {
        Self {
            trigger: command::AnyTrigger::command::<C>(args),
            source,
        }
    }

    pub(super) fn bind(self, node: view::Node) -> view::Node {
        node.bind_trigger(self.trigger, self.source)
    }
}

impl SliderBinding {
    pub(super) fn bind(self, node: view::Node, value: f64) -> view::Node {
        match self {
            Self::Fixed(binding) => binding.bind(node),
            Self::Change(binding) => binding.bind(node, value),
        }
    }
}

impl SliderChangeBinding {
    pub(super) fn command<C>(
        source: Source,
        map: impl Fn(f64) -> C::Args + Send + Sync + 'static,
    ) -> Self
    where
        C: command::Command,
        C::Args: Clone,
    {
        Self {
            trigger: command::AnyValueTrigger::command::<C>(map),
            source,
        }
    }

    fn bind(self, node: view::Node, value: f64) -> view::Node {
        node.bind_slider_trigger(value, self.source, self.trigger)
    }
}

impl TextBoxBinding {
    pub(super) fn command<C>(
        source: Source,
        map: impl Fn(String) -> C::Args + Send + Sync + 'static,
    ) -> Self
    where
        C: command::Command,
        C::Args: Clone,
    {
        Self {
            trigger: command::AnyValueTrigger::command::<C>(map),
            source,
        }
    }

    pub(super) fn bind(self, node: view::Node, text: String) -> view::Node {
        node.bind_text_trigger(text, self.source, self.trigger)
    }
}
