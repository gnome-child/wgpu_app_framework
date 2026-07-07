use super::super::{diagnostics::Diagnostics, interaction, window};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Context {
    window: window::Id,
    diagnostics: Diagnostics,
    interaction: interaction::Interaction,
}

impl Context {
    pub(crate) fn new(
        window: window::Id,
        diagnostics: Diagnostics,
        interaction: interaction::Interaction,
    ) -> Self {
        Self {
            window,
            diagnostics,
            interaction,
        }
    }

    pub fn window(&self) -> window::Id {
        self.window
    }

    pub fn diagnostics(&self) -> &Diagnostics {
        &self.diagnostics
    }

    pub fn interaction(&self) -> &interaction::Interaction {
        &self.interaction
    }
}
