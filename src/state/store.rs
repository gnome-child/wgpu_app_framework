use super::{Change, Reason, Revision, Snapshot, State, snapshot::PendingSnapshot};

pub(crate) const DEFAULT_CHANGE_LIMIT: usize = 1024;

pub struct Store<M: State> {
    model: M,
    retained_snapshot: Option<M>,
    revision: Revision,
    saved_revision: Revision,
    changes: Vec<Change>,
    change_limit: usize,
}

impl<M: State> Store<M> {
    pub fn new(model: M) -> Self {
        let revision = Revision::initial();

        Self {
            model,
            retained_snapshot: None,
            revision,
            saved_revision: revision,
            changes: Vec::new(),
            change_limit: DEFAULT_CHANGE_LIMIT,
        }
    }

    pub fn model(&self) -> &M {
        &self.model
    }

    pub(crate) fn model_mut(&mut self) -> &mut M {
        &mut self.model
    }

    pub(crate) fn prepare_snapshot(&mut self) -> PendingSnapshot<M> {
        PendingSnapshot::new(
            self.retained_snapshot
                .take()
                .unwrap_or_else(|| self.model.clone()),
        )
    }

    pub(crate) fn restore_prepared_snapshot(&mut self, snapshot: PendingSnapshot<M>) {
        self.retained_snapshot = Some(snapshot.into_model());
    }

    pub(crate) fn discard_retained_snapshot(&mut self) {
        self.retained_snapshot = None;
    }

    pub fn revision(&self) -> Revision {
        self.revision
    }

    pub fn saved_revision(&self) -> Revision {
        self.saved_revision
    }

    pub fn is_dirty(&self) -> bool {
        self.revision != self.saved_revision
    }

    pub fn changes(&self) -> &[Change] {
        &self.changes
    }

    pub fn change_limit(&self) -> usize {
        self.change_limit
    }

    pub(crate) fn set_change_limit(&mut self, limit: usize) {
        self.change_limit = limit;
        self.prune_changes();
    }

    pub fn mark_saved(&mut self) {
        self.saved_revision = self.revision;
    }

    pub(crate) fn snapshot(&self) -> Snapshot<M> {
        Snapshot::new(self.model.clone())
    }

    pub(crate) fn restore(&mut self, snapshot: Snapshot<M>, reason: Reason) -> Change {
        self.model = snapshot.into_model();
        self.commit(reason)
    }

    pub(crate) fn commit(&mut self, reason: Reason) -> Change {
        self.discard_retained_snapshot();
        self.revision = self.revision.next();
        let change = Change::new(self.revision, reason);
        self.changes.push(change.clone());
        self.prune_changes();
        change
    }

    pub(crate) fn commit_retaining_current(&mut self, reason: Reason) -> Change {
        let change = self.commit(reason);
        self.retained_snapshot = Some(self.model.clone());
        change
    }

    fn prune_changes(&mut self) {
        if self.changes.len() > self.change_limit {
            let drop_count = self.changes.len() - self.change_limit;
            self.changes.drain(0..drop_count);
        }
    }
}
