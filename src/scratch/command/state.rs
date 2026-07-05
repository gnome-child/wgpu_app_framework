use super::{registry::AnyCommand, spec::KeyChord};
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(in crate::scratch) enum Availability {
    Enabled,
    Disabled,
    Hidden,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct State {
    pub(in crate::scratch) availability: Availability,
    pub(in crate::scratch) checked: Option<bool>,
    pub(in crate::scratch) label: Option<String>,
    pub(in crate::scratch) shortcut: Option<KeyChord>,
    pub(in crate::scratch) tooltip: Option<String>,
}

impl State {
    pub(in crate::scratch) fn enabled() -> Self {
        Self {
            availability: Availability::Enabled,
            checked: None,
            label: None,
            shortcut: None,
            tooltip: None,
        }
    }

    pub(in crate::scratch) fn disabled() -> Self {
        Self {
            availability: Availability::Disabled,
            checked: None,
            label: None,
            shortcut: None,
            tooltip: None,
        }
    }

    /// Means "this target does not claim the command in this state; keep resolving".
    pub(in crate::scratch) fn hidden() -> Self {
        Self {
            availability: Availability::Hidden,
            checked: None,
            label: None,
            shortcut: None,
            tooltip: None,
        }
    }

    pub(in crate::scratch) fn is_enabled(&self) -> bool {
        self.availability == Availability::Enabled
    }

    pub(in crate::scratch) fn is_hidden(&self) -> bool {
        self.availability == Availability::Hidden
    }

    pub(in crate::scratch) fn with_label(mut self, label: impl Into<String>) -> Self {
        self.label = Some(label.into());
        self
    }

    pub(in crate::scratch) fn with_shortcut(mut self, shortcut: KeyChord) -> Self {
        self.shortcut = Some(shortcut);
        self
    }

    pub(in crate::scratch) fn with_tooltip(mut self, tooltip: impl Into<String>) -> Self {
        self.tooltip = Some(tooltip.into());
        self
    }

    pub(in crate::scratch) fn checked(mut self, checked: bool) -> Self {
        self.checked = Some(checked);
        self
    }

    pub(in crate::scratch::command) fn with_command(mut self, command: &AnyCommand) -> Self {
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
