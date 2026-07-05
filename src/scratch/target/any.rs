use std::any::{Any, TypeId, type_name};

use super::{Selector, Target};
use crate::scratch::{
    command::{Command, State},
    context::Context,
    error::{Error, Result},
    response::{AnyResponse, Response},
    state,
};

pub(in crate::scratch) struct AnyTarget<M: state::State> {
    command_type: TypeId,
    command_name: &'static str,
    concrete_type: TypeId,
    concrete_name: &'static str,
    state: Box<dyn Fn(&mut M, &dyn Any, &Context) -> Result<State>>,
    invoke: Box<dyn Fn(&mut M, Box<dyn Any + Send>, &mut Context) -> AnyResponse>,
}

impl<M: state::State> AnyTarget<M> {
    pub(in crate::scratch) fn new<C, T>(selector: Selector<M, T>) -> Self
    where
        C: Command,
        T: Target<C> + 'static,
    {
        let state_selector = selector.clone();
        let invoke_selector = selector;

        Self {
            command_type: TypeId::of::<C>(),
            command_name: C::NAME,
            concrete_type: TypeId::of::<T>(),
            concrete_name: type_name::<T>(),
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

    pub(in crate::scratch) fn handles<C: Command>(&self) -> bool {
        self.command_type == TypeId::of::<C>()
    }

    pub(in crate::scratch) fn handles_type(&self, command_type: TypeId) -> bool {
        self.command_type == command_type
    }

    pub(in crate::scratch) fn state<C: Command>(
        &self,
        model: &mut M,
        args: &C::Args,
        cx: &Context,
    ) -> Result<State> {
        debug_assert!(self.handles::<C>());
        (self.state)(model, args, cx)
    }

    pub(in crate::scratch) fn state_any(
        &self,
        model: &mut M,
        args: &dyn Any,
        cx: &Context,
    ) -> Result<State> {
        (self.state)(model, args, cx)
    }

    pub(in crate::scratch) fn invoke<C: Command>(
        &self,
        model: &mut M,
        args: C::Args,
        cx: &mut Context,
    ) -> Response<C::Output> {
        debug_assert!(self.handles::<C>());
        (self.invoke)(model, Box::new(args), cx).into_response(C::NAME)
    }

    pub(in crate::scratch) fn invoke_any(
        &self,
        model: &mut M,
        args: Box<dyn Any + Send>,
        cx: &mut Context,
    ) -> AnyResponse {
        (self.invoke)(model, args, cx)
    }
}
