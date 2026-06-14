#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct State {
    enabled: bool,
    active: bool,
}

impl State {
    pub const fn new(enabled: bool, active: bool) -> Self {
        Self { enabled, active }
    }

    /// An invokable action that is not currently on, selected, or running.
    pub const fn enabled() -> Self {
        Self::new(true, false)
    }

    /// A non-invokable action.
    pub const fn disabled() -> Self {
        Self::new(false, false)
    }

    /// An invokable action that is currently on, selected, or running.
    pub const fn active() -> Self {
        Self::new(true, true)
    }

    /// Whether the action can be invoked in the resolved context.
    pub const fn is_enabled(self) -> bool {
        self.enabled
    }

    /// Whether the action is currently on, selected, or running in the resolved context.
    ///
    /// Completed or historical work should stay in application state unless it represents a
    /// persistent current state.
    pub const fn is_active(self) -> bool {
        self.active
    }

    pub const fn with_enabled(mut self, enabled: bool) -> Self {
        self.enabled = enabled;
        self
    }

    pub const fn with_active(mut self, active: bool) -> Self {
        self.active = active;
        self
    }

    fn with_active_overlay(mut self, active: bool) -> Self {
        self.active |= active;
        self
    }
}

pub fn with_active_overlay(state: State, active: bool) -> State {
    state.with_active_overlay(active)
}

impl Default for State {
    fn default() -> Self {
        Self::enabled()
    }
}
