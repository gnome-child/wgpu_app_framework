use crate::action;

use super::{Command, Key, Target, state::State, target};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Route {
    command: Key,
    target: target::Kind,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Binding {
    command: Key,
    state: Option<State>,
}

pub trait Responder {
    fn bind_targets(&self, _targets: &mut Vec<target::Kind>) {}

    fn bind_commands(&self, _bindings: &mut Vec<Binding>) {}

    fn command_targets(&self) -> Vec<target::Kind> {
        let mut targets = Vec::new();
        self.bind_targets(&mut targets);
        targets
    }

    fn command_bindings(&self) -> Vec<Binding> {
        let mut bindings = Vec::new();
        self.bind_commands(&mut bindings);
        bindings
    }
}

impl Binding {
    pub(crate) const fn new(command: Key) -> Self {
        Self {
            command,
            state: None,
        }
    }

    pub fn of<C: Command>() -> Self {
        Self::new(Key::of::<C>())
    }

    pub(crate) const fn command(&self) -> Key {
        self.command
    }

    pub(crate) fn from_action(binding: action::Binding) -> Self {
        let mut command = Self {
            command: Key::from_action(binding.key()),
            state: None,
        };

        if let Some(state) = binding.state() {
            command.state = Some(command_state(*state));
        }

        command
    }

    pub fn action(&self) -> action::Binding {
        let mut binding = action::Binding::new(self.command.action());

        if let Some(state) = self.state.as_ref() {
            binding = binding
                .available(state.is_available())
                .active(state.is_active())
                .running(state.is_running());
        }

        binding
    }

    pub fn state(&self) -> Option<&State> {
        self.state.as_ref()
    }

    pub fn available(mut self, available: bool) -> Self {
        self.state = Some(self.current_state().with_available(available));
        self
    }

    pub fn active(mut self, active: bool) -> Self {
        self.state = Some(self.current_state().with_active(active));
        self
    }

    pub fn running(mut self, running: bool) -> Self {
        self.state = Some(self.current_state().with_running(running));
        self
    }

    fn current_state(&self) -> State {
        self.state.clone().unwrap_or_else(State::available)
    }
}

impl Route {
    pub fn invokes<C, TTarget>() -> Self
    where
        C: Command,
        TTarget: Target<C> + 'static,
    {
        let _ = std::any::TypeId::of::<TTarget>();
        Self::new(Key::of::<C>(), C::target())
    }

    pub(crate) const fn new(command: Key, target: target::Kind) -> Self {
        Self { command, target }
    }

    pub(crate) const fn from_action(route: action::Route) -> Self {
        Self {
            command: Key::from_action(route.key()),
            target: target::Kind::from_action(route.target()),
        }
    }

    pub const fn action(self) -> action::Route {
        action::Route::new(self.command.action(), self.target.action())
    }

    pub(crate) const fn command(self) -> Key {
        self.command
    }

    pub(crate) const fn target(self) -> target::Kind {
        self.target
    }
}

fn command_state(state: action::State) -> State {
    State::available_if(state.is_available())
        .with_active(state.is_active())
        .with_running(state.is_running())
}
