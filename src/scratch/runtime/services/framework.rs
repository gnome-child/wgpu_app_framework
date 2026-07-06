use std::any::{Any, TypeId};

use crate::scratch::{
    command, context, error::Result, response::AnyResponse, session, state, target::Target,
    timeline,
};

use super::{Services, target};

const RESPONDER_NAME: &str = "framework";

struct ServiceContext<'a, 'services, M: state::State> {
    services: &'a mut Services<'services, M>,
    store: &'a mut state::Store<M>,
}

pub(super) fn state<M: state::State>(
    services: &mut Services<'_, M>,
    store: &mut state::Store<M>,
    command_type: TypeId,
    command_name: &'static str,
    args: &dyn Any,
    cx: &context::Context,
) -> Result<Option<command::State>> {
    let mut service = ServiceContext { services, store };

    target::state(
        RESPONDER_NAME,
        targets(),
        &mut service,
        command_type,
        command_name,
        args,
        cx,
    )
}

pub(super) fn invoke<M: state::State>(
    services: &mut Services<'_, M>,
    store: &mut state::Store<M>,
    command_type: TypeId,
    command_name: &'static str,
    args: Box<dyn Any + Send>,
    cx: &mut context::Context,
) -> Option<AnyResponse> {
    let targets = targets();
    let mut service = ServiceContext { services, store };

    target::invoke(
        RESPONDER_NAME,
        &targets,
        &mut service,
        command_type,
        command_name,
        args,
        cx,
    )
}

fn targets<'a, 'services, M: state::State>()
-> [target::AnyTarget<ServiceContext<'a, 'services, M>>; 3] {
    [
        target::AnyTarget::new::<timeline::Undo>(
            timeline_undo_state::<M>,
            timeline_undo_invoke::<M>,
        ),
        target::AnyTarget::new::<timeline::Redo>(
            timeline_redo_state::<M>,
            timeline_redo_invoke::<M>,
        ),
        target::AnyTarget::new::<session::CloseWindow>(
            close_window_state::<M>,
            close_window_invoke::<M>,
        ),
    ]
}

fn timeline_undo_state<M: state::State>(
    context: &mut ServiceContext<'_, '_, M>,
    args: &dyn Any,
    cx: &context::Context,
) -> Result<command::State> {
    let args = target::args::<timeline::Undo>(args)?;
    let service = timeline::Service::new(context.store, &mut *context.services.timeline);

    Ok(Target::<timeline::Undo>::state(&service, args, cx))
}

fn timeline_undo_invoke<M: state::State>(
    context: &mut ServiceContext<'_, '_, M>,
    args: Box<dyn Any + Send>,
    cx: &mut context::Context,
) -> AnyResponse {
    let args = match target::args_box::<timeline::Undo>(args) {
        Ok(args) => args,
        Err(error) => return AnyResponse::failed(error),
    };
    let mut service = timeline::Service::new(context.store, &mut *context.services.timeline);

    AnyResponse::from_response(Target::<timeline::Undo>::invoke(&mut service, args, cx))
}

fn timeline_redo_state<M: state::State>(
    context: &mut ServiceContext<'_, '_, M>,
    args: &dyn Any,
    cx: &context::Context,
) -> Result<command::State> {
    let args = target::args::<timeline::Redo>(args)?;
    let service = timeline::Service::new(context.store, &mut *context.services.timeline);

    Ok(Target::<timeline::Redo>::state(&service, args, cx))
}

fn timeline_redo_invoke<M: state::State>(
    context: &mut ServiceContext<'_, '_, M>,
    args: Box<dyn Any + Send>,
    cx: &mut context::Context,
) -> AnyResponse {
    let args = match target::args_box::<timeline::Redo>(args) {
        Ok(args) => args,
        Err(error) => return AnyResponse::failed(error),
    };
    let mut service = timeline::Service::new(context.store, &mut *context.services.timeline);

    AnyResponse::from_response(Target::<timeline::Redo>::invoke(&mut service, args, cx))
}

fn close_window_state<M: state::State>(
    context: &mut ServiceContext<'_, '_, M>,
    args: &dyn Any,
    cx: &context::Context,
) -> Result<command::State> {
    let args = target::args::<session::CloseWindow>(args)?;
    let service = session::Service::new(
        &mut *context.services.session,
        &mut *context.services.composition,
        &mut *context.services.diagnostics,
        context.services.window,
    );

    Ok(Target::<session::CloseWindow>::state(&service, args, cx))
}

fn close_window_invoke<M: state::State>(
    context: &mut ServiceContext<'_, '_, M>,
    args: Box<dyn Any + Send>,
    cx: &mut context::Context,
) -> AnyResponse {
    let args = match target::args_box::<session::CloseWindow>(args) {
        Ok(args) => args,
        Err(error) => return AnyResponse::failed(error),
    };
    let mut service = session::Service::new(
        &mut *context.services.session,
        &mut *context.services.composition,
        &mut *context.services.diagnostics,
        context.services.window,
    );

    AnyResponse::from_response(Target::<session::CloseWindow>::invoke(
        &mut service,
        args,
        cx,
    ))
}
