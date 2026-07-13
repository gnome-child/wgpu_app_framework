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
    pub(crate) hint: Option<String>,
}

impl State {
    pub fn enabled() -> Self {
        Self {
            availability: Availability::Enabled,
            checked: None,
            label: None,
            shortcut: None,
            hint: None,
        }
    }

    pub fn disabled() -> Self {
        Self {
            availability: Availability::Disabled,
            checked: None,
            label: None,
            shortcut: None,
            hint: None,
        }
    }

    /// Means "this target does not claim the command in this state; keep resolving".
    pub fn hidden() -> Self {
        Self {
            availability: Availability::Hidden,
            checked: None,
            label: None,
            shortcut: None,
            hint: None,
        }
    }

    pub fn is_enabled(&self) -> bool {
        self.availability == Availability::Enabled
    }

    pub fn is_hidden(&self) -> bool {
        self.availability == Availability::Hidden
    }

    pub fn label(&self) -> Option<&str> {
        self.label.as_deref()
    }

    pub fn shortcut(&self) -> Option<KeyChord> {
        self.shortcut
    }

    pub fn hint(&self) -> Option<&str> {
        self.hint.as_deref()
    }

    pub fn checked_state(&self) -> Option<bool> {
        self.checked
    }

    pub fn with_label(mut self, label: impl Into<String>) -> Self {
        self.label = Some(label.into());
        self
    }

    pub fn with_shortcut(mut self, shortcut: KeyChord) -> Self {
        self.shortcut = Some(shortcut);
        self
    }

    pub fn with_hint(mut self, hint: impl Into<String>) -> Self {
        self.hint = Some(hint.into());
        self
    }

    pub fn checked(mut self, checked: bool) -> Self {
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hint_is_contextual_state() {
        let state = State::disabled().with_hint("Choose a document first");

        assert_eq!(state.hint(), Some("Choose a document first"));
    }
}
