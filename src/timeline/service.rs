use super::Timeline;
use crate::state::{self, Store};

pub(crate) struct Service<'a, M: state::State> {
    pub(super) store: &'a mut Store<M>,
    pub(super) timeline: &'a mut Timeline<M>,
}

impl<'a, M: state::State> Service<'a, M> {
    pub(crate) fn new(store: &'a mut Store<M>, timeline: &'a mut Timeline<M>) -> Self {
        Self { store, timeline }
    }

    pub(super) fn timeline(&self) -> &Timeline<M> {
        &*self.timeline
    }
}
