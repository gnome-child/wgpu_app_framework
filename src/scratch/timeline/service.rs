use super::Timeline;
use crate::scratch::state::{self, Store};

pub(in crate::scratch) struct Service<'a, M: state::State> {
    pub(super) store: &'a mut Store<M>,
    pub(super) timeline: &'a mut Timeline<M>,
}

impl<'a, M: state::State> Service<'a, M> {
    pub(in crate::scratch) fn new(store: &'a mut Store<M>, timeline: &'a mut Timeline<M>) -> Self {
        Self { store, timeline }
    }

    pub(super) fn timeline(&self) -> &Timeline<M> {
        &*self.timeline
    }
}
