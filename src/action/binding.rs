use super::{Id, State};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Binding {
    action: Id,
    state: Option<State>,
}

impl Binding {
    pub const fn new(action: Id) -> Self {
        Self {
            action,
            state: None,
        }
    }

    pub const fn action(self) -> Id {
        self.action
    }

    pub const fn state(self) -> Option<State> {
        self.state
    }

    pub const fn enabled(mut self, enabled: bool) -> Self {
        self.state = Some(self.current_state().with_enabled(enabled));
        self
    }

    pub const fn active(mut self, active: bool) -> Self {
        self.state = Some(self.current_state().with_active(active));
        self
    }

    pub const fn busy(mut self, busy: bool) -> Self {
        self.state = Some(self.current_state().with_busy(busy));
        self
    }

    const fn current_state(self) -> State {
        match self.state {
            Some(state) => state,
            None => State::enabled(),
        }
    }
}
