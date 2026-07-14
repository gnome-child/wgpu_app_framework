use std::fmt;

/// The visual priority of a runtime fact.
///
/// Severity does not imply interaction, focus trapping, or lifetime. Those
/// remain policies of the owner retaining the feedback.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Severity {
    Info,
    Warning,
    Error,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct Entry {
    severity: Severity,
    text: String,
}

impl Entry {
    pub(crate) fn new(severity: Severity, message: impl fmt::Display) -> Self {
        Self {
            severity,
            text: message.to_string(),
        }
    }

    pub(crate) fn severity(&self) -> Severity {
        self.severity
    }

    pub(crate) fn from_text(severity: Severity, text: String) -> Self {
        Self { severity, text }
    }

    pub(crate) fn text(&self) -> &str {
        &self.text
    }
}

#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub(crate) struct Stack {
    entries: [Option<Entry>; 3],
}

impl Stack {
    pub(crate) fn report(&mut self, severity: Severity, message: impl fmt::Display) -> bool {
        let entry = Entry::new(severity, message);
        self.replace(entry)
    }

    pub(crate) fn report_text(&mut self, severity: Severity, text: String) -> bool {
        self.replace(Entry::from_text(severity, text))
    }

    fn replace(&mut self, entry: Entry) -> bool {
        let severity = entry.severity();
        let slot = &mut self.entries[severity.index()];
        if slot.as_ref() == Some(&entry) {
            return false;
        }
        *slot = Some(entry);
        true
    }

    pub(crate) fn clear(&mut self, severity: Severity) -> bool {
        self.entries[severity.index()].take().is_some()
    }

    pub(crate) fn clear_all(&mut self) -> bool {
        let changed = self.entries.iter().any(Option::is_some);
        self.entries = Default::default();
        changed
    }

    pub(crate) fn winner(&self) -> Option<&Entry> {
        [Severity::Error, Severity::Warning, Severity::Info]
            .into_iter()
            .find_map(|severity| self.entries[severity.index()].as_ref())
    }
}

impl Severity {
    const fn index(self) -> usize {
        match self {
            Self::Info => 0,
            Self::Warning => 1,
            Self::Error => 2,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reporting_formats_display_once_at_the_boundary() {
        use std::cell::Cell;

        struct CountingDisplay<'a>(&'a Cell<u32>);

        impl fmt::Display for CountingDisplay<'_> {
            fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
                self.0.set(self.0.get() + 1);
                formatter.write_str("current fact")
            }
        }

        let calls = Cell::new(0);
        let entry = Entry::new(Severity::Warning, CountingDisplay(&calls));

        assert_eq!(calls.get(), 1);
        assert_eq!(entry.severity(), Severity::Warning);
        assert_eq!(entry.text(), "current fact");
        assert_eq!(calls.get(), 1);
    }

    #[test]
    fn highest_severity_wins_without_destroying_lower_facts() {
        let mut stack = Stack::default();
        assert!(stack.report(Severity::Info, "saved locally"));
        assert!(stack.report(Severity::Warning, "not synchronized"));
        assert!(stack.report(Severity::Error, "save failed"));

        assert_eq!(stack.winner().map(Entry::text), Some("save failed"));
        assert!(stack.clear(Severity::Error));
        assert_eq!(stack.winner().map(Entry::text), Some("not synchronized"));
        assert!(stack.clear(Severity::Warning));
        assert_eq!(stack.winner().map(Entry::text), Some("saved locally"));
    }
}
