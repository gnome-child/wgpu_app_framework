#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct Change {
    kind: Kind,
    submit: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Kind {
    Unchanged,
    Changed { text: bool, selection: bool },
}

impl Change {
    pub(super) fn new(
        text_changed: bool,
        selection_changed: bool,
        other_changed: bool,
        submit: bool,
    ) -> Self {
        let kind = if text_changed || selection_changed || other_changed {
            Kind::Changed {
                text: text_changed,
                selection: selection_changed,
            }
        } else {
            Kind::Unchanged
        };
        Self { kind, submit }
    }

    pub fn text_changed(&self) -> bool {
        matches!(self.kind, Kind::Changed { text: true, .. })
    }

    pub fn selection_changed(&self) -> bool {
        matches!(
            self.kind,
            Kind::Changed {
                selection: true,
                ..
            }
        )
    }

    pub fn changed(&self) -> bool {
        matches!(self.kind, Kind::Changed { .. })
    }

    pub fn submit(&self) -> bool {
        self.submit
    }
}

#[cfg(test)]
mod tests {
    use super::Change;

    #[test]
    fn text_and_selection_changes_imply_the_broader_change() {
        let text = Change::new(true, false, false, false);
        assert!(text.text_changed());
        assert!(!text.selection_changed());
        assert!(text.changed());

        let selection = Change::new(false, true, false, false);
        assert!(!selection.text_changed());
        assert!(selection.selection_changed());
        assert!(selection.changed());

        let submit_only = Change::new(false, false, false, true);
        assert!(!submit_only.changed());
        assert!(submit_only.submit());
    }
}
