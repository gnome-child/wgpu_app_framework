use super::command::Command;

pub(super) const DEFAULT_CHANGE_LIMIT: usize = 1024;

/// Durable application model state. The framework owns it through `Store`.
pub trait State: Clone + 'static {}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Revision(u64);

impl Revision {
    pub fn initial() -> Self {
        Self(0)
    }

    pub(super) fn next(self) -> Self {
        Self(self.0 + 1)
    }

    pub fn get(self) -> u64 {
        self.0
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Reason {
    Command(&'static str),
    Event(&'static str),
    Load,
    Save,
    Restore,
    Undo,
    Redo,
    Programmatic(&'static str),
}

impl Reason {
    pub fn command<C: Command>() -> Self {
        Self::Command(C::NAME)
    }

    pub fn event(label: &'static str) -> Self {
        Self::Event(label)
    }

    pub fn programmatic(label: &'static str) -> Self {
        Self::Programmatic(label)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Change {
    revision: Revision,
    reason: Reason,
}

#[derive(Clone)]
pub struct Snapshot<M: State> {
    model: M,
}

pub(super) struct PendingSnapshot<M: State> {
    model: M,
}

impl Change {
    fn new(revision: Revision, reason: Reason) -> Self {
        Self { revision, reason }
    }

    pub fn revision(&self) -> Revision {
        self.revision
    }

    pub fn reason(&self) -> &Reason {
        &self.reason
    }
}

impl<M: State> Snapshot<M> {
    fn new(model: M) -> Self {
        Self { model }
    }

    pub fn from_model(model: M) -> Self {
        Self::new(model)
    }

    pub fn model(&self) -> &M {
        &self.model
    }

    fn into_model(self) -> M {
        self.model
    }
}

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

    pub(super) fn model_mut(&mut self) -> &mut M {
        &mut self.model
    }

    pub(super) fn prepare_snapshot(&mut self) -> PendingSnapshot<M> {
        PendingSnapshot::new(
            self.retained_snapshot
                .take()
                .unwrap_or_else(|| self.model.clone()),
        )
    }

    pub(super) fn restore_prepared_snapshot(&mut self, snapshot: PendingSnapshot<M>) {
        self.retained_snapshot = Some(snapshot.into_model());
    }

    pub(super) fn discard_retained_snapshot(&mut self) {
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

    pub(super) fn set_change_limit(&mut self, limit: usize) {
        self.change_limit = limit;
        self.prune_changes();
    }

    pub fn mark_saved(&mut self) {
        self.saved_revision = self.revision;
    }

    pub(super) fn snapshot(&self) -> Snapshot<M> {
        Snapshot::new(self.model.clone())
    }

    pub(super) fn restore(&mut self, snapshot: Snapshot<M>, reason: Reason) -> Change {
        self.model = snapshot.into_model();
        self.commit(reason)
    }

    pub(super) fn commit(&mut self, reason: Reason) -> Change {
        self.discard_retained_snapshot();
        self.revision = self.revision.next();
        let change = Change::new(self.revision, reason);
        self.changes.push(change.clone());
        self.prune_changes();
        change
    }

    pub(super) fn commit_retaining_current(&mut self, reason: Reason) -> Change {
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

impl<M: State> PendingSnapshot<M> {
    fn new(model: M) -> Self {
        Self { model }
    }

    pub(super) fn into_model(self) -> M {
        self.model
    }
}
