#[derive(Debug, Clone, PartialEq, Eq)]
pub struct State {
    available: bool,
    active: bool,
    running: bool,
    display: Option<String>,
    hint: Option<String>,
}

impl State {
    fn new(available: bool, active: bool) -> Self {
        Self {
            available,
            active,
            running: false,
            display: None,
            hint: None,
        }
    }

    /// A command that can run in the resolved context.
    pub fn available() -> Self {
        Self::new(true, false)
    }

    /// A command that cannot run in the resolved context.
    pub fn unavailable() -> Self {
        Self::new(false, false)
    }

    /// An allowed command that is currently on, selected, or toggled.
    pub fn active() -> Self {
        Self::new(true, true)
    }

    /// An allowed command whose work is currently in flight.
    pub fn running() -> Self {
        Self::new(true, false).with_running(true)
    }

    pub fn available_if(available: bool) -> Self {
        Self::available().with_available(available)
    }

    pub fn active_if(active: bool) -> Self {
        Self::available().with_active(active)
    }

    pub fn running_if(running: bool) -> Self {
        Self::available().with_running(running)
    }

    /// Whether the command is allowed in the resolved context before running state is considered.
    pub fn is_available(&self) -> bool {
        self.available
    }

    /// Whether the command is currently on, selected, or toggled in the resolved context.
    ///
    /// Completed or historical work should stay in application state unless it represents a
    /// persistent current state.
    pub fn is_active(&self) -> bool {
        self.active
    }

    /// Whether command work is currently in flight in the resolved context.
    pub fn is_running(&self) -> bool {
        self.running
    }

    /// Runtime display override for this command in the resolved context.
    pub fn display(&self) -> Option<&str> {
        self.display.as_deref()
    }

    /// Runtime hint override for this command in the resolved context.
    pub fn hint(&self) -> Option<&str> {
        self.hint.as_deref()
    }

    pub fn with_available(mut self, available: bool) -> Self {
        self.available = available;
        self
    }

    pub fn with_active(mut self, active: bool) -> Self {
        self.active = active;
        self
    }

    pub fn with_running(mut self, running: bool) -> Self {
        self.running = running;
        self
    }

    pub fn with_display(mut self, display: impl Into<String>) -> Self {
        self.display = Some(display.into());
        self
    }

    pub fn with_hint(mut self, hint: impl Into<String>) -> Self {
        self.hint = Some(hint.into());
        self
    }

    pub fn clear_display(mut self) -> Self {
        self.display = None;
        self
    }

    pub fn clear_hint(mut self) -> Self {
        self.hint = None;
        self
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Presentation {
    display: String,
    hint: Option<String>,
    state: State,
}

impl Presentation {
    pub fn new(display: impl Into<String>, hint: Option<String>, state: State) -> Self {
        Self {
            display: display.into(),
            hint,
            state,
        }
    }

    pub fn display(&self) -> &str {
        &self.display
    }

    pub fn hint(&self) -> Option<&str> {
        self.hint.as_deref()
    }

    pub fn state(&self) -> &State {
        &self.state
    }
}

pub fn with_running_overlay(mut state: State, running: bool) -> State {
    state.running |= running;
    state
}

impl Default for State {
    fn default() -> Self {
        Self::available()
    }
}
