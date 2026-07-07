use super::super::{
    command,
    context::{Context as CommandContext, Source},
    interaction, responder, response, state,
};
use super::Action;

#[derive(Clone)]
pub struct Binding {
    trigger: command::AnyTrigger,
    state: command::State,
    source: Source,
    slider_trigger: Option<command::AnyValueTrigger<f64>>,
    text_trigger: Option<command::AnyValueTrigger<String>>,
}

impl Binding {
    pub(super) fn new<C>(args: C::Args, source: Source) -> Self
    where
        C: command::Command,
        C::Args: Clone,
    {
        Self::from_trigger(command::AnyTrigger::command::<C>(args), source)
    }

    pub(crate) fn from_trigger(trigger: command::AnyTrigger, source: Source) -> Self {
        Self {
            trigger,
            state: command::State::hidden(),
            source,
            slider_trigger: None,
            text_trigger: None,
        }
    }

    pub(super) fn slider(
        value: f64,
        source: Source,
        slider_trigger: command::AnyValueTrigger<f64>,
    ) -> Self {
        Self {
            trigger: slider_trigger.trigger(value),
            state: command::State::hidden(),
            source,
            slider_trigger: Some(slider_trigger),
            text_trigger: None,
        }
    }

    pub(super) fn text(
        text: String,
        source: Source,
        text_trigger: command::AnyValueTrigger<String>,
    ) -> Self {
        Self {
            trigger: text_trigger.trigger(text),
            state: command::State::hidden(),
            source,
            slider_trigger: None,
            text_trigger: Some(text_trigger),
        }
    }

    pub fn command_name(&self) -> &'static str {
        self.trigger.command_name()
    }

    pub fn command_type(&self) -> std::any::TypeId {
        self.trigger.command_type()
    }

    pub(crate) fn history_group(&self) -> Option<command::HistoryGroup> {
        self.trigger.history_group()
    }

    pub fn state(&self) -> &command::State {
        &self.state
    }

    pub fn label(&self) -> Option<&str> {
        self.state.label.as_deref()
    }

    pub fn checked(&self) -> Option<bool> {
        self.state.checked
    }

    pub fn shortcut(&self) -> Option<command::KeyChord> {
        self.state.shortcut
    }

    pub fn source(&self) -> Source {
        self.source
    }

    pub(crate) fn action(&self) -> Action {
        Action::activate(self)
    }

    pub(crate) fn slider_action(&self, value: f64) -> Option<Action> {
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
        interaction::Target::command_element(id, self.command_name(), self.source)
    }

    pub fn is_enabled(&self) -> bool {
        self.state.is_enabled()
    }

    pub fn is_hidden(&self) -> bool {
        self.state.is_hidden()
    }

    pub(super) fn resolve<M: state::State>(
        &mut self,
        registry: &command::Registry,
        chain: &mut responder::Chain<'_, M>,
        cx: &CommandContext,
    ) {
        let cx = cx.sourced(self.source);
        self.state = self.trigger.state(registry, chain, &cx);
    }

    pub(crate) fn invoke<M: state::State>(
        &self,
        registry: &command::Registry,
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
