use super::Marker;
use super::transaction::{Change, Impact, Transaction};

#[derive(Debug, Clone, Default)]
pub struct Outcome {
    pub(crate) text_changed: bool,
    pub(crate) selection_changed: bool,
    pub(crate) change: Option<Change>,
    pub(crate) impacts: Vec<Impact>,
}

impl Outcome {
    pub(super) fn from_markers(
        before: Marker,
        after: Marker,
        transaction: Transaction,
        impacts: Vec<Impact>,
    ) -> Self {
        let text_changed = !transaction.is_empty();
        let selection_changed =
            before.cursor != after.cursor || before.selection != after.selection;
        Self {
            text_changed,
            selection_changed,
            change: text_changed.then_some(Change {
                before,
                after,
                transaction,
            }),
            impacts: if text_changed {
                impacts
            } else {
                Default::default()
            },
        }
    }

    pub fn text_changed(&self) -> bool {
        self.text_changed
    }

    pub fn selection_changed(&self) -> bool {
        self.selection_changed
    }

    pub fn buffer_changed(&self) -> bool {
        self.text_changed || self.selection_changed
    }
}
