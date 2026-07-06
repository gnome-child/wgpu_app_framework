use std::any::{Any, TypeId};

use super::super::super::{
    command::{self, Command},
    context::Context,
    error::{Error, Result},
    response::AnyResponse,
};

pub(super) type StateThunk<S> = fn(&mut S, &dyn Any, &Context) -> Result<command::State>;
pub(super) type InvokeThunk<S> = fn(&mut S, Box<dyn Any + Send>, &mut Context) -> AnyResponse;

pub(super) struct AnyTarget<S> {
    command_type: TypeId,
    state: StateThunk<S>,
    invoke: InvokeThunk<S>,
}

impl<S> AnyTarget<S> {
    pub(super) fn new<C: Command>(state: StateThunk<S>, invoke: InvokeThunk<S>) -> Self {
        Self {
            command_type: TypeId::of::<C>(),
            state,
            invoke,
        }
    }

    pub(super) fn handles_type(&self, command_type: TypeId) -> bool {
        self.command_type == command_type
    }

    fn state(&self, service: &mut S, args: &dyn Any, cx: &Context) -> Result<command::State> {
        (self.state)(service, args, cx)
    }

    fn invoke(&self, service: &mut S, args: Box<dyn Any + Send>, cx: &mut Context) -> AnyResponse {
        (self.invoke)(service, args, cx)
    }
}

pub(super) fn state<S>(
    responder_name: &'static str,
    targets: impl IntoIterator<Item = AnyTarget<S>>,
    service: &mut S,
    command_type: TypeId,
    command_name: &'static str,
    args: &dyn Any,
    cx: &Context,
) -> Result<Option<command::State>> {
    let mut claim = None;

    for target in targets {
        if !target.handles_type(command_type) {
            continue;
        }

        let state = target.state(service, args, cx)?;
        if state.is_hidden() {
            continue;
        }

        if claim.is_some() {
            return Err(Error::AmbiguousTarget {
                command: command_name,
                responder: responder_name,
            });
        }

        claim = Some(state);
    }

    Ok(claim)
}

pub(super) fn invoke<S>(
    responder_name: &'static str,
    targets: &[AnyTarget<S>],
    service: &mut S,
    command_type: TypeId,
    command_name: &'static str,
    args: Box<dyn Any + Send>,
    cx: &mut Context,
) -> Option<AnyResponse> {
    let mut claim = None;

    for (index, target) in targets.iter().enumerate() {
        if !target.handles_type(command_type) {
            continue;
        }

        match target.state(service, args.as_ref(), cx) {
            Ok(state) if state.is_hidden() => {}
            Ok(state) => {
                if claim.is_some() {
                    return Some(AnyResponse::failed(Error::AmbiguousTarget {
                        command: command_name,
                        responder: responder_name,
                    }));
                }

                claim = Some((index, state));
            }
            Err(error) => return Some(AnyResponse::failed(error)),
        }
    }

    let Some((index, state)) = claim else {
        return None;
    };

    match state.availability {
        command::Availability::Hidden => unreachable!("hidden targets are not claims"),
        command::Availability::Disabled => Some(AnyResponse::failed(Error::Disabled {
            command: command_name,
        })),
        command::Availability::Enabled => Some(targets[index].invoke(service, args, cx)),
    }
}

pub(super) fn args<C: Command>(args: &dyn Any) -> Result<&C::Args> {
    args.downcast_ref::<C::Args>()
        .ok_or(Error::ArgsMismatch { command: C::NAME })
}

pub(super) fn args_box<C: Command>(args: Box<dyn Any + Send>) -> Result<C::Args> {
    args.downcast::<C::Args>()
        .map(|args| *args)
        .map_err(|_| Error::ArgsMismatch { command: C::NAME })
}
