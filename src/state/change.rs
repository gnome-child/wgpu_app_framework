use super::{Reason, Revision};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Change {
    revision: Revision,
    reason: Reason,
}

impl Change {
    pub(super) fn new(revision: Revision, reason: Reason) -> Self {
        Self { revision, reason }
    }

    pub fn revision(&self) -> Revision {
        self.revision
    }

    pub fn reason(&self) -> &Reason {
        &self.reason
    }
}
