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
        Self::new(Key::of::<C>(), target::Kind::of_type::<TTarget>())
    }

    pub(crate) const fn new(command: Key, target: target::Kind) -> Self {
        Self { command, target }
    }

    pub(crate) const fn command(self) -> Key {
        self.command
    }

    pub(crate) const fn target(self) -> target::Kind {
        self.target
    }
}
