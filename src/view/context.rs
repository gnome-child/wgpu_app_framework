use super::super::{diagnostics::Diagnostics, window};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Context {
    window: window::Id,
    diagnostics: Diagnostics,
}

impl Context {
    pub(crate) fn new(window: window::Id, diagnostics: Diagnostics) -> Self {
        Self {
            window,
            diagnostics,
        }
    }

    pub fn window(&self) -> window::Id {
        self.window
    }

    pub fn diagnostics(&self) -> &Diagnostics {
        &self.diagnostics
    }
}
