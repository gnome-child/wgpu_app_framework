use std::any::{Any, TypeId};

use super::{Selector, Target};
use crate::{
    command::{Command, State},
    context::Context,
    error::{Error, Result},
    response::AnyResponse,
    state,
};

type StateThunk<M> = dyn Fn(&mut M, &dyn Any, &Context) -> Result<State>;
type InvokeThunk<M> = dyn Fn(&mut M, Box<dyn Any + Send>, &mut Context) -> AnyResponse;

pub(crate) struct AnyTarget<M: state::State> {
    command_type: TypeId,
    state: Box<StateThunk<M>>,
    invoke: Box<InvokeThunk<M>>,
}

impl<M: state::State> AnyTarget<M> {
    pub(crate) fn new<C, T>(selector: Selector<M, T>) -> Self
    where
        C: Command,
        T: Target<C> + 'static,
    {
        let state_selector = selector.clone();
        let invoke_selector = selector;

        Self {
            command_type: TypeId::of::<C>(),
            state: Box::new(move |model, args, cx| {
                let args = args
                    .downcast_ref::<C::Args>()
                    .ok_or(Error::ArgsMismatch { command: C::NAME })?;
                let target = state_selector(model);

                Ok(<T as Target<C>>::state(target, args, cx))
            }),
            invoke: Box::new(move |model, args, cx| {
                let args = match args.downcast::<C::Args>() {
                    Ok(args) => *args,
                    Err(_) => return AnyResponse::failed(Error::ArgsMismatch { command: C::NAME }),
                };
                let target = invoke_selector(model);

                AnyResponse::from_response(<T as Target<C>>::invoke(target, args, cx))
            }),
        }
    }

    pub(crate) fn handles_type(&self, command_type: TypeId) -> bool {
        self.command_type == command_type
    }

    pub(crate) fn state_any(&self, model: &mut M, args: &dyn Any, cx: &Context) -> Result<State> {
        (self.state)(model, args, cx)
    }

    pub(crate) fn invoke_any(
        &self,
        model: &mut M,
        args: Box<dyn Any + Send>,
        cx: &mut Context,
    ) -> AnyResponse {
        (self.invoke)(model, args, cx)
    }
}
