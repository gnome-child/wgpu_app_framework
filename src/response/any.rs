use std::any::Any;

use super::{Effect, Response};
use crate::command::{Error, Result};

pub(crate) struct AnyResponse {
    output: Result<Box<dyn Any + Send>>,
    effect: Effect,
    changed: bool,
}

impl AnyResponse {
    pub(crate) fn from_response<O: Send + 'static>(response: Response<O>) -> Self {
        Self {
            output: response
                .output
                .map(|output| Box::new(output) as Box<dyn Any + Send>),
            effect: response.effect,
            changed: response.changed,
        }
    }

    pub(crate) fn failed(error: Error) -> Self {
        Self {
            output: Err(error),
            effect: Effect::None,
            changed: false,
        }
    }

    pub(crate) fn into_response<O: Send + 'static>(self, command: &'static str) -> Response<O> {
        let output = match self.output {
            Ok(output) => output
                .downcast::<O>()
                .map(|output| *output)
                .map_err(|_| Error::OutputMismatch { command }),
            Err(error) => Err(error),
        };

        Response {
            output,
            effect: self.effect,
            changed: self.changed,
        }
    }

    pub(crate) fn effect(&self) -> Effect {
        self.effect.clone()
    }

    pub(crate) fn changed_state(&self) -> bool {
        self.changed
    }

    pub(crate) fn mark_changed(&mut self) {
        self.changed = true;
    }

    pub(crate) fn is_ok(&self) -> bool {
        self.output.is_ok()
    }

    pub(crate) fn output_any(&self) -> Option<&(dyn Any + Send)> {
        self.output.as_ref().ok().map(|output| output.as_ref())
    }

    pub(crate) fn into_result(self) -> Result<()> {
        self.output.map(|_| ())
    }
}
