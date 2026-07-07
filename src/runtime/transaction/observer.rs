use std::any::TypeId;

use super::super::Runtime;
use crate::{
    command::Command,
    context::Source,
    error::Error,
    response::{self, Response},
    state,
};

impl<M: state::State, E: Send + 'static, V> Runtime<M, E, V> {
    pub(in crate::runtime) fn observe_response<C: Command>(
        &mut self,
        response: &Response<C::Output>,
        source: Source,
    ) -> std::result::Result<bool, Error> {
        if !response.is_ok() {
            return Ok(false);
        }

        let observers = &mut self.observers;
        let model = self.store.model_mut();
        observers.observe_response::<C>(model, response, source)
    }

    pub(in crate::runtime) fn observe_any_response(
        &mut self,
        command_type: TypeId,
        response: &response::AnyResponse,
        source: Source,
    ) -> std::result::Result<bool, Error> {
        let observers = &mut self.observers;
        let model = self.store.model_mut();
        observers.observe_any(command_type, model, response, source)
    }
}
