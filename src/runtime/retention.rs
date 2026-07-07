use crate::{draft, state, timeline};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Retention {
    changes: usize,
    snapshots: usize,
    drafts: usize,
}

impl Default for Retention {
    fn default() -> Self {
        Self {
            changes: state::DEFAULT_CHANGE_LIMIT,
            snapshots: timeline::DEFAULT_SNAPSHOT_LIMIT,
            drafts: draft::input::DEFAULT_DRAFT_LIMIT,
        }
    }
}

impl Retention {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn changes(mut self, limit: usize) -> Self {
        self.changes = limit;
        self
    }

    pub fn snapshots(mut self, limit: usize) -> Self {
        self.snapshots = limit;
        self
    }

    pub fn drafts(mut self, limit: usize) -> Self {
        self.drafts = limit;
        self
    }

    pub fn change_limit(self) -> usize {
        self.changes
    }

    pub fn snapshot_limit(self) -> usize {
        self.snapshots
    }

    pub fn draft_limit(self) -> usize {
        self.drafts
    }
}
