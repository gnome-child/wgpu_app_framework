mod any;
mod effect;

pub use effect::{Effect, Invalidation};

pub(in crate::scratch) use any::AnyResponse;

use super::error::{Error, Result};

pub struct Response<O: Send + 'static> {
    pub(super) output: Result<O>,
    pub(super) effect: Effect,
    changed: bool,
}

impl<O: Send + 'static> Response<O> {
    pub(super) fn output(output: O) -> Self {
        Self {
            output: Ok(output),
            effect: Effect::None,
            changed: false,
        }
    }

    pub(super) fn changed(output: O) -> Self {
        Self {
            output: Ok(output),
            effect: Effect::None,
            changed: true,
        }
    }

    pub(super) fn failed(error: Error) -> Self {
        Self {
            output: Err(error),
            effect: Effect::None,
            changed: false,
        }
    }

    pub(super) fn with_effect(mut self, effect: Effect) -> Self {
        self.effect = effect;
        self
    }

    pub(super) fn is_ok(&self) -> bool {
        self.output.is_ok()
    }

    pub(super) fn changed_state(&self) -> bool {
        self.changed
    }

    pub(super) fn mark_changed(&mut self) {
        self.changed = true;
    }

    pub(super) fn output_ref(&self) -> Option<&O> {
        self.output.as_ref().ok()
    }
}
