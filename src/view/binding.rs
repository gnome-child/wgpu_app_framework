use super::super::{
    command,
    context::{Context as CommandContext, Source},
    interaction, responder, response, state,
};
use super::Action;

#[derive(Clone)]
pub struct Binding {
    trigger: Trigger,
    state: command::State,
    description: Option<&'static str>,
    source: Source,
    route: responder::Route,
}

#[derive(Clone)]
enum Trigger {
    Fixed(command::AnyTrigger),
    Slider {
        current: command::AnyTrigger,
        factory: command::AnyValueTrigger<f64>,
    },
}

impl Trigger {
    fn current(&self) -> &command::AnyTrigger {
        match self {
            Self::Fixed(trigger) => trigger,
            Self::Slider { current, .. } => current,
        }
    }

    fn with_slider_value(&self, value: f64) -> Option<Self> {
        let Self::Slider { factory, .. } = self else {
            return None;
        };
        Some(Self::Slider {
            current: factory.trigger(value),
            factory: factory.clone(),
        })
    }
}

impl Binding {
    pub(in crate::view) fn same_scene_state(&self, other: &Self) -> bool {
        self.state == other.state && self.source == other.source
    }

    pub(super) fn new<C>(args: C::Args, source: Source) -> Self
    where
        C: command::Command,
        C::Args: Clone,
    {
        Self::from_trigger(command::AnyTrigger::command::<C>(args), source)
    }

    pub(crate) fn from_trigger(trigger: command::AnyTrigger, source: Source) -> Self {
        Self {
            trigger: Trigger::Fixed(trigger),
            state: command::State::hidden(),
            description: None,
            source,
            route: responder::Route::Chain,
        }
    }

    pub(super) fn slider(
        value: f64,
        source: Source,
        slider_trigger: command::AnyValueTrigger<f64>,
    ) -> Self {
        let current = slider_trigger.trigger(value);
        Self {
            trigger: Trigger::Slider {
                current,
                factory: slider_trigger,
            },
            state: command::State::hidden(),
            description: None,
            source,
            route: responder::Route::Chain,
        }
    }

    pub fn command_name(&self) -> &'static str {
        self.trigger.current().command_name()
    }

    pub fn command_type(&self) -> std::any::TypeId {
        self.trigger.current().command_type()
    }

    pub(crate) fn history_group(&self) -> Option<command::HistoryGroup> {
        self.trigger.current().history_group()
    }

    pub(crate) fn trigger(&self) -> command::AnyTrigger {
        self.trigger.current().clone()
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

    pub fn hint(&self) -> Option<&str> {
        self.state.hint()
    }

    pub fn description(&self) -> Option<&'static str> {
        self.description
    }

    pub fn source(&self) -> Source {
        self.source
    }

    pub(crate) fn from_resolved(action: command::ResolvedAction, source: Source) -> Self {
        Self {
            trigger: Trigger::Fixed(action.trigger()),
            state: action.state().clone(),
            description: action.description(),
            source,
            route: action.route(),
        }
    }

    pub(crate) fn from_bar_action(action: &command::BarAction, show_shortcut: bool) -> Self {
        let mut state = action.state().clone();
        if !show_shortcut {
            state.shortcut = None;
        }
        Self {
            trigger: Trigger::Fixed(action.trigger()),
            state,
            description: action.description(),
            source: Source::Menu,
            route: responder::Route::Chain,
        }
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
        self.state = self
            .trigger
            .current()
            .state_on(self.route, registry, chain, &cx);
        self.description = registry.description(self.trigger.current().command_type());
    }

    pub(crate) fn invoke<M: state::State>(
        &self,
        registry: &command::Registry,
        chain: &mut responder::Chain<'_, M>,
        cx: &mut CommandContext,
    ) -> response::AnyResponse {
        self.trigger
            .current()
            .invoke_on(self.route, registry, chain, cx)
    }

    fn with_slider_value(&self, value: f64) -> Option<Self> {
        Some(Self {
            trigger: self.trigger.with_slider_value(value)?,
            state: self.state.clone(),
            description: self.description,
            source: self.source,
            route: self.route,
        })
    }
}
