mod any;
pub mod effect;

pub use effect::Effect;

pub(crate) use any::AnyResponse;

use super::command::{Error, Result};

pub struct Response<O: Send + 'static> {
    pub(super) output: Result<O>,
    pub(super) effect: Effect,
    changed: bool,
}

impl<O: Send + 'static> Response<O> {
    pub fn output(output: O) -> Self {
        Self {
            output: Ok(output),
            effect: Effect::None,
            changed: false,
        }
    }

    pub fn changed(output: O) -> Self {
        Self {
            output: Ok(output),
            effect: Effect::None,
            changed: true,
        }
    }

    pub fn failed(error: Error) -> Self {
        Self {
            output: Err(error),
            effect: Effect::None,
            changed: false,
        }
    }

    pub fn with_effect(mut self, effect: Effect) -> Self {
        self.effect = effect;
        self
    }

    pub fn is_ok(&self) -> bool {
        self.output.is_ok()
    }

    pub fn changed_state(&self) -> bool {
        self.changed
    }

    pub(super) fn mark_changed(&mut self) {
        self.changed = true;
    }

    pub fn output_ref(&self) -> Option<&O> {
        self.output.as_ref().ok()
    }

    pub fn into_result(self) -> std::result::Result<O, Error> {
        self.output
    }
}
