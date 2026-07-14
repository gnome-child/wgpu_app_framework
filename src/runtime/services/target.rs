use std::any::{Any, TypeId};

use super::super::super::{
    command::{self, Command, Error, Result},
    context::Context,
    response::AnyResponse,
    target::Target,
};

type StateThunk<S> = fn(&mut S, &dyn Any, &Context) -> Result<command::State>;
type InvokeThunk<S> = fn(&mut S, Box<dyn Any + Send>, &mut Context) -> AnyResponse;

pub(super) struct AnyTarget<S> {
    command_type: TypeId,
    state: StateThunk<S>,
    invoke: InvokeThunk<S>,
}

pub(super) struct Claim {
    pub(super) target: usize,
    pub(super) state: command::State,
}

pub(super) trait Provider<C: Command> {
    type Target<'a>: Target<C>
    where
        Self: 'a;

    fn target(&mut self) -> Self::Target<'_>;
}

impl<S> AnyTarget<S> {
    fn new<C: Command>(state: StateThunk<S>, invoke: InvokeThunk<S>) -> Self {
        Self {
            command_type: TypeId::of::<C>(),
            state,
            invoke,
        }
    }

    pub(super) fn for_provider<C>() -> Self
    where
        C: Command,
        S: Provider<C>,
    {
        Self::new::<C>(provider_state::<S, C>, provider_invoke::<S, C>)
    }

    pub(super) fn handles_type(&self, command_type: TypeId) -> bool {
        self.command_type == command_type
    }

    pub(super) fn command_type(&self) -> TypeId {
        self.command_type
    }

    fn state(&self, service: &mut S, args: &dyn Any, cx: &Context) -> Result<command::State> {
        (self.state)(service, args, cx)
    }

    fn invoke(&self, service: &mut S, args: Box<dyn Any + Send>, cx: &mut Context) -> AnyResponse {
        (self.invoke)(service, args, cx)
    }
}

pub(super) fn handles<S>(targets: &[AnyTarget<S>], command_type: TypeId) -> bool {
    targets
        .iter()
        .any(|target| target.handles_type(command_type))
}

pub(super) fn state<S>(
    responder_name: &'static str,
    targets: &[AnyTarget<S>],
    service: &mut S,
    command_type: TypeId,
    command_name: &'static str,
    args: &dyn Any,
    cx: &Context,
) -> Result<Option<command::State>> {
    claim(
        responder_name,
        targets,
        service,
        command_type,
        command_name,
        args,
        cx,
    )
    .map(|claim| claim.map(|claim| claim.state))
}

pub(super) fn claim<S>(
    responder_name: &'static str,
    targets: &[AnyTarget<S>],
    service: &mut S,
    command_type: TypeId,
    command_name: &'static str,
    args: &dyn Any,
    cx: &Context,
) -> Result<Option<Claim>> {
    claim_target(
        responder_name,
        targets,
        service,
        command_type,
        command_name,
        args,
        cx,
    )
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
    let claim = match claim_target(
        responder_name,
        targets,
        service,
        command_type,
        command_name,
        args.as_ref(),
        cx,
    ) {
        Ok(claim) => claim,
        Err(error) => return Some(AnyResponse::failed(error)),
    };
    let claim = claim?;

    match claim.state.availability {
        command::Availability::Hidden => unreachable!("hidden targets are not claims"),
        command::Availability::Disabled => Some(AnyResponse::failed(Error::Disabled {
            command: command_name,
        })),
        command::Availability::Enabled => Some(targets[claim.target].invoke(service, args, cx)),
    }
}

fn claim_target<S>(
    responder_name: &'static str,
    targets: &[AnyTarget<S>],
    service: &mut S,
    command_type: TypeId,
    command_name: &'static str,
    args: &dyn Any,
    cx: &Context,
) -> Result<Option<Claim>> {
    let mut claim = None;

    for (index, target) in targets.iter().enumerate() {
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

        claim = Some(Claim {
            target: index,
            state,
        });
    }

    Ok(claim)
}

fn args<C: Command>(args: &dyn Any) -> Result<&C::Args> {
    args.downcast_ref::<C::Args>()
        .ok_or(Error::ArgsMismatch { command: C::NAME })
}

fn args_box<C: Command>(args: Box<dyn Any + Send>) -> Result<C::Args> {
    args.downcast::<C::Args>()
        .map(|args| *args)
        .map_err(|_| Error::ArgsMismatch { command: C::NAME })
}

fn provider_state<S, C>(service: &mut S, raw_args: &dyn Any, cx: &Context) -> Result<command::State>
where
    C: Command,
    S: Provider<C>,
{
    let args = args::<C>(raw_args)?;
    let target = <S as Provider<C>>::target(service);

    Ok(Target::<C>::state(&target, args, cx))
}

fn provider_invoke<S, C>(
    service: &mut S,
    args: Box<dyn Any + Send>,
    cx: &mut Context,
) -> AnyResponse
where
    C: Command,
    S: Provider<C>,
{
    let args = match args_box::<C>(args) {
        Ok(args) => args,
        Err(error) => return AnyResponse::failed(error),
    };
    let mut target = <S as Provider<C>>::target(service);

    AnyResponse::from_response(Target::<C>::invoke(&mut target, args, cx))
}
