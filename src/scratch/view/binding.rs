use std::any::TypeId;

use super::super::{
    command as framework_command,
    context::{Context as CommandContext, Source},
    interaction, responder, response, state,
};
use super::Action;

#[derive(Clone)]
pub struct Binding {
    trigger: framework_command::AnyTrigger,
    state: framework_command::State,
    source: Source,
    slider_trigger: Option<framework_command::AnyValueTrigger<f64>>,
    text_trigger: Option<framework_command::AnyValueTrigger<String>>,
}

impl Binding {
    pub(super) fn new<C>(args: C::Args, source: Source) -> Self
    where
        C: framework_command::Command,
        C::Args: Clone,
    {
        Self::from_trigger(framework_command::AnyTrigger::command::<C>(args), source)
    }

    pub(super) fn from_trigger(trigger: framework_command::AnyTrigger, source: Source) -> Self {
        Self {
            trigger,
            state: framework_command::State::hidden(),
            source,
            slider_trigger: None,
            text_trigger: None,
        }
    }

    pub(super) fn slider(
        value: f64,
        source: Source,
        slider_trigger: framework_command::AnyValueTrigger<f64>,
    ) -> Self {
        Self {
            trigger: slider_trigger.trigger(value),
            state: framework_command::State::hidden(),
            source,
            slider_trigger: Some(slider_trigger),
            text_trigger: None,
        }
    }

    pub(super) fn text(
        text: String,
        source: Source,
        text_trigger: framework_command::AnyValueTrigger<String>,
    ) -> Self {
        Self {
            trigger: text_trigger.trigger(text),
            state: framework_command::State::hidden(),
            source,
            slider_trigger: None,
            text_trigger: Some(text_trigger),
        }
    }

    pub fn command_name(&self) -> &'static str {
        self.trigger.command_name()
    }

    pub fn command_type(&self) -> TypeId {
        self.trigger.command_type()
    }

    pub fn state(&self) -> &framework_command::State {
        &self.state
    }

    pub fn label(&self) -> Option<&str> {
        self.state.label.as_deref()
    }

    pub fn source(&self) -> Source {
        self.source
    }

    pub fn action(&self) -> Action {
        Action::activate(self)
    }

    pub(in crate::scratch) fn slider_action(&self, value: f64) -> Option<Action> {
        if !self.is_enabled() {
            return None;
        }

        Some(Action::Activate(self.with_slider_value(value)?))
    }

    pub(super) fn text_action(&self, text: String) -> Option<Action> {
        if !self.is_enabled() {
            return None;
        }

        Some(Action::Activate(self.with_text_value(text)?))
    }

    pub(super) fn element_pointer_target(&self, id: interaction::Id) -> interaction::Target {
        interaction::Target::command_element(id, self.command_name())
    }

    pub(super) fn path_pointer_target(&self, path: &[usize]) -> interaction::Target {
        interaction::Target::command_path(
            self.command_type(),
            self.command_name(),
            self.source,
            path.to_vec(),
        )
    }

    pub(super) fn set_state(&mut self, state: framework_command::State) {
        self.state = state;
    }

    pub fn is_enabled(&self) -> bool {
        self.state.is_enabled()
    }

    pub fn is_hidden(&self) -> bool {
        self.state.is_hidden()
    }

    pub(super) fn resolve<M: state::State>(
        &mut self,
        registry: &framework_command::Registry,
        chain: &mut responder::Chain<'_, M>,
        cx: &CommandContext,
    ) {
        let cx = cx.sourced(self.source);
        self.state = self.trigger.state(registry, chain, &cx);
    }

    pub(in crate::scratch) fn invoke<M: state::State>(
        &self,
        registry: &framework_command::Registry,
        chain: &mut responder::Chain<'_, M>,
        cx: &mut CommandContext,
    ) -> response::AnyResponse {
        self.trigger.invoke(registry, chain, cx)
    }

    fn with_slider_value(&self, value: f64) -> Option<Self> {
        let slider_trigger = self.slider_trigger.clone()?;

        Some(Self {
            trigger: slider_trigger.trigger(value),
            state: self.state.clone(),
            source: self.source,
            slider_trigger: Some(slider_trigger),
            text_trigger: self.text_trigger.clone(),
        })
    }

    fn with_text_value(&self, text: String) -> Option<Self> {
        let text_trigger = self.text_trigger.clone()?;

        Some(Self {
            trigger: text_trigger.trigger(text),
            state: self.state.clone(),
            source: self.source,
            slider_trigger: self.slider_trigger.clone(),
            text_trigger: Some(text_trigger),
        })
    }
}
