use std::any::Any;

use super::{Effect, Response};
use crate::scratch::error::{Error, Result};

pub(in crate::scratch) struct AnyResponse {
    output: Result<Box<dyn Any + Send>>,
    effect: Effect,
    changed: bool,
}

impl AnyResponse {
    pub(in crate::scratch) fn from_response<O: Send + 'static>(response: Response<O>) -> Self {
        Self {
            output: response
                .output
                .map(|output| Box::new(output) as Box<dyn Any + Send>),
            effect: response.effect,
            changed: response.changed,
        }
    }

    pub(in crate::scratch) fn failed(error: Error) -> Self {
        Self {
            output: Err(error),
            effect: Effect::None,
            changed: false,
        }
    }

    pub(in crate::scratch) fn into_response<O: Send + 'static>(
        self,
        command: &'static str,
    ) -> Response<O> {
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

    pub(in crate::scratch) fn effect(&self) -> Effect {
        self.effect.clone()
    }

    pub(in crate::scratch) fn changed_state(&self) -> bool {
        self.changed
    }

    pub(in crate::scratch) fn mark_changed(&mut self) {
        self.changed = true;
    }

    pub(in crate::scratch) fn is_ok(&self) -> bool {
        self.output.is_ok()
    }

    pub(in crate::scratch) fn output_any(&self) -> Option<&(dyn Any + Send)> {
        self.output.as_ref().ok().map(|output| output.as_ref())
    }

    pub(in crate::scratch) fn into_result(self) -> Result<()> {
        self.output.map(|_| ())
    }
}
