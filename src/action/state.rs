#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct State {
    enabled: bool,
    active: bool,
    busy: bool,
}

impl State {
    pub const fn new(enabled: bool, active: bool) -> Self {
        Self {
            enabled,
            active,
            busy: false,
        }
    }

    /// An allowed action that is not currently on, selected, toggled, or busy.
    pub const fn enabled() -> Self {
        Self::new(true, false)
    }

    /// A non-invokable action.
    pub const fn disabled() -> Self {
        Self::new(false, false)
    }

    /// An allowed action that is currently on, selected, or toggled.
    pub const fn active() -> Self {
        Self::new(true, true)
    }

    /// An allowed action whose work is currently in flight.
    pub const fn busy() -> Self {
        Self::new(true, false).with_busy(true)
    }

    /// Whether the action is allowed in the resolved context before busy state is considered.
    pub const fn is_enabled(self) -> bool {
        self.enabled
    }

    /// Whether the action is currently on, selected, or toggled in the resolved context.
    ///
    /// Completed or historical work should stay in application state unless it represents a
    /// persistent current state.
    pub const fn is_active(self) -> bool {
        self.active
    }

    /// Whether action work is currently in flight in the resolved context.
    pub const fn is_busy(self) -> bool {
        self.busy
    }

    pub const fn with_enabled(mut self, enabled: bool) -> Self {
        self.enabled = enabled;
        self
    }

    pub const fn with_active(mut self, active: bool) -> Self {
        self.active = active;
        self
    }

    pub const fn with_busy(mut self, busy: bool) -> Self {
        self.busy = busy;
        self
    }
}

pub fn with_busy_overlay(mut state: State, busy: bool) -> State {
    state.busy |= busy;
    state
}

impl Default for State {
    fn default() -> Self {
        Self::enabled()
    }
}
