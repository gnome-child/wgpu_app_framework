#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct VisualState {
    available: bool,
    active: bool,
    running: bool,
}

impl VisualState {
    const fn new(available: bool, active: bool, running: bool) -> Self {
        Self {
            available,
            active,
            running,
        }
    }

    pub const fn available() -> Self {
        Self::new(true, false, false)
    }

    pub const fn unavailable() -> Self {
        Self::new(false, false, false)
    }

    pub const fn active() -> Self {
        Self::new(true, true, false)
    }

    pub const fn running() -> Self {
        Self::new(true, false, true)
    }

    pub const fn available_if(available: bool) -> Self {
        Self::available().with_available(available)
    }

    pub const fn active_if(active: bool) -> Self {
        Self::available().with_active(active)
    }

    pub const fn running_if(running: bool) -> Self {
        Self::available().with_running(running)
    }

    pub const fn is_available(self) -> bool {
        self.available
    }

    pub const fn is_active(self) -> bool {
        self.active
    }

    pub const fn is_running(self) -> bool {
        self.running
    }

    pub const fn with_available(mut self, available: bool) -> Self {
        self.available = available;
        self
    }

    pub const fn with_active(mut self, active: bool) -> Self {
        self.active = active;
        self
    }

    pub const fn with_running(mut self, running: bool) -> Self {
        self.running = running;
        self
    }
}

impl Default for VisualState {
    fn default() -> Self {
        Self::available()
    }
}
