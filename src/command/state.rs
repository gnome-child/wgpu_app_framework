use super::{registry::AnyCommand, spec::KeyChord};
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Availability {
    Enabled,
    Disabled,
    Hidden,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct State {
    pub(crate) availability: Availability,
    pub(crate) checked: Option<bool>,
    pub(crate) label: Option<String>,
    pub(crate) shortcut: Option<KeyChord>,
    pub(crate) tooltip: Option<String>,
}

impl State {
    pub(crate) fn enabled() -> Self {
        Self {
            availability: Availability::Enabled,
            checked: None,
            label: None,
            shortcut: None,
            tooltip: None,
        }
    }

    pub(crate) fn disabled() -> Self {
        Self {
            availability: Availability::Disabled,
            checked: None,
            label: None,
            shortcut: None,
            tooltip: None,
        }
    }

    /// Means "this target does not claim the command in this state; keep resolving".
    pub(crate) fn hidden() -> Self {
        Self {
            availability: Availability::Hidden,
            checked: None,
            label: None,
            shortcut: None,
            tooltip: None,
        }
    }

    pub(crate) fn is_enabled(&self) -> bool {
        self.availability == Availability::Enabled
    }

    pub(crate) fn is_hidden(&self) -> bool {
        self.availability == Availability::Hidden
    }

    pub(crate) fn with_label(mut self, label: impl Into<String>) -> Self {
        self.label = Some(label.into());
        self
    }

    pub(crate) fn with_shortcut(mut self, shortcut: KeyChord) -> Self {
        self.shortcut = Some(shortcut);
        self
    }

    pub(crate) fn with_tooltip(mut self, tooltip: impl Into<String>) -> Self {
        self.tooltip = Some(tooltip.into());
        self
    }

    pub(crate) fn checked(mut self, checked: bool) -> Self {
        self.checked = Some(checked);
        self
    }

    pub(in crate::command) fn with_command(mut self, command: &AnyCommand) -> Self {
        if self.label.is_none() {
            self = self.with_label(command.spec.display_name);
        }

        if let Some(shortcut) = command.shortcut()
            && self.shortcut.is_none()
        {
            self = self.with_shortcut(shortcut);
        }

        self
    }
}
