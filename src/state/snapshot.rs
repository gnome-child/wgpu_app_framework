use super::State;

#[derive(Clone)]
pub struct Snapshot<M: State> {
    model: M,
}

pub(crate) struct PendingSnapshot<M: State> {
    model: M,
}

impl<M: State> Snapshot<M> {
    pub(super) fn new(model: M) -> Self {
        Self { model }
    }

    pub fn from_model(model: M) -> Self {
        Self::new(model)
    }

    pub fn model(&self) -> &M {
        &self.model
    }

    pub(crate) fn into_model(self) -> M {
        self.model
    }
}

impl<M: State> PendingSnapshot<M> {
    pub(super) fn new(model: M) -> Self {
        Self { model }
    }

    pub(crate) fn into_model(self) -> M {
        self.model
    }
}
