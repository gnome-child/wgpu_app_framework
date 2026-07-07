mod command;
mod service;
mod undoable;

pub use command::{Redo, Undo};
pub use undoable::Undoable;

pub(crate) use command::register;
pub(crate) use service::Service;

use super::state;

pub(crate) const DEFAULT_SNAPSHOT_LIMIT: usize = 256;

pub struct Timeline<M: state::State> {
    past: Vec<M>,
    future: Vec<M>,
    snapshot_limit: usize,
}

impl<M: state::State> Default for Timeline<M> {
    fn default() -> Self {
        Self {
            past: Vec::new(),
            future: Vec::new(),
            snapshot_limit: DEFAULT_SNAPSHOT_LIMIT,
        }
    }
}

impl<M: state::State> Timeline<M> {
    pub(super) fn record(&mut self, previous: M) {
        self.past.push(previous);
        self.prune_past();
        self.future.clear();
    }

    pub(super) fn undo(&mut self, current: &mut M) -> bool {
        let Some(previous) = self.past.pop() else {
            return false;
        };

        self.future.push(current.clone());
        self.prune_future();
        *current = previous;

        true
    }

    pub(super) fn redo(&mut self, current: &mut M) -> bool {
        let Some(next) = self.future.pop() else {
            return false;
        };

        self.past.push(current.clone());
        self.prune_past();
        *current = next;

        true
    }

    pub fn can_undo(&self) -> bool {
        !self.past.is_empty()
    }

    pub fn can_redo(&self) -> bool {
        !self.future.is_empty()
    }

    pub fn undo_depth(&self) -> usize {
        self.past.len()
    }

    pub fn redo_depth(&self) -> usize {
        self.future.len()
    }

    pub fn snapshot_limit(&self) -> usize {
        self.snapshot_limit
    }

    pub(super) fn set_snapshot_limit(&mut self, limit: usize) {
        self.snapshot_limit = limit;
        self.prune_past();
        self.prune_future();
    }

    pub(super) fn clear(&mut self) {
        self.past.clear();
        self.future.clear();
    }

    fn prune_past(&mut self) {
        if self.past.len() > self.snapshot_limit {
            let drop_count = self.past.len() - self.snapshot_limit;
            self.past.drain(0..drop_count);
        }
    }

    fn prune_future(&mut self) {
        if self.future.len() > self.snapshot_limit {
            let drop_count = self.future.len() - self.snapshot_limit;
            self.future.drain(0..drop_count);
        }
    }
}
