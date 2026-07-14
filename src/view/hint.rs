use crate::{feedback, icon};

/// One resolved explanation payload shared by inline indicators and the
/// ordinary hover-panel presenter.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct Hint {
    description: String,
    icon: Option<icon::Icon>,
    tone: Tone,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Tone {
    Neutral,
    Warning,
    Error,
}

impl Hint {
    pub(crate) fn plain(description: impl Into<String>) -> Self {
        Self {
            description: description.into(),
            icon: None,
            tone: Tone::Neutral,
        }
    }

    pub(crate) fn information(description: impl Into<String>) -> Self {
        Self {
            description: description.into(),
            icon: Some(icon::Icon::phosphor(icon::Id::new("info"))),
            tone: Tone::Neutral,
        }
    }

    pub(crate) fn from_feedback(
        severity: feedback::Severity,
        description: impl Into<String>,
    ) -> Self {
        let (name, tone) = match severity {
            feedback::Severity::Info => ("info", Tone::Neutral),
            feedback::Severity::Warning => ("warning", Tone::Warning),
            feedback::Severity::Error => ("x-circle", Tone::Error),
        };
        Self {
            description: description.into(),
            icon: Some(icon::Icon::phosphor(icon::Id::new(name))),
            tone,
        }
    }

    pub(crate) fn description(&self) -> &str {
        &self.description
    }

    pub(crate) fn icon(&self) -> Option<icon::Icon> {
        self.icon
    }

    pub(crate) fn tone(&self) -> Tone {
        self.tone
    }
}
