use std::{
    any::{Any, TypeId},
    result,
};

use crate::scratch::{
    command::{self, Command as CommandTrait},
    context,
    error::Error,
    response::AnyResponse,
    session, state,
    target::Target,
    timeline,
};

use super::Services;

#[derive(Clone, Copy)]
pub(super) enum Command {
    Undo,
    Redo,
    CloseWindow,
}

impl Command {
    pub(super) fn from_type(command_type: TypeId) -> Option<Self> {
        if command_type == TypeId::of::<timeline::Undo>() {
            return Some(Self::Undo);
        }

        if command_type == TypeId::of::<timeline::Redo>() {
            return Some(Self::Redo);
        }

        if command_type == TypeId::of::<session::CloseWindow>() {
            return Some(Self::CloseWindow);
        }

        None
    }

    pub(super) fn state<M: state::State>(
        self,
        services: &mut Services<'_, M>,
        store: &mut state::Store<M>,
        args: &dyn Any,
        cx: &context::Context,
    ) -> result::Result<command::State, Error> {
        match self {
            Self::Undo => {
                let args = framework_args::<timeline::Undo>(args)?;
                let service = timeline::Service::new(store, &mut *services.timeline);
                Ok(Target::<timeline::Undo>::state(&service, args, cx))
            }
            Self::Redo => {
                let args = framework_args::<timeline::Redo>(args)?;
                let service = timeline::Service::new(store, &mut *services.timeline);
                Ok(Target::<timeline::Redo>::state(&service, args, cx))
            }
            Self::CloseWindow => {
                let args = framework_args::<session::CloseWindow>(args)?;
                let service = session::Service::new(
                    &mut *services.session,
                    &mut *services.composition,
                    &mut *services.diagnostics,
                    services.window,
                );
                Ok(Target::<session::CloseWindow>::state(&service, args, cx))
            }
        }
    }

    pub(super) fn invoke<M: state::State>(
        self,
        services: &mut Services<'_, M>,
        store: &mut state::Store<M>,
        args: Box<dyn Any + Send>,
        cx: &mut context::Context,
    ) -> AnyResponse {
        match self {
            Self::Undo => match framework_args_box::<timeline::Undo>(args) {
                Ok(()) => {
                    let mut service = timeline::Service::new(store, &mut *services.timeline);
                    AnyResponse::from_response(Target::<timeline::Undo>::invoke(
                        &mut service,
                        (),
                        cx,
                    ))
                }
                Err(error) => AnyResponse::failed(error),
            },
            Self::Redo => match framework_args_box::<timeline::Redo>(args) {
                Ok(()) => {
                    let mut service = timeline::Service::new(store, &mut *services.timeline);
                    AnyResponse::from_response(Target::<timeline::Redo>::invoke(
                        &mut service,
                        (),
                        cx,
                    ))
                }
                Err(error) => AnyResponse::failed(error),
            },
            Self::CloseWindow => match framework_args_box::<session::CloseWindow>(args) {
                Ok(()) => {
                    let mut service = session::Service::new(
                        &mut *services.session,
                        &mut *services.composition,
                        &mut *services.diagnostics,
                        services.window,
                    );
                    AnyResponse::from_response(Target::<session::CloseWindow>::invoke(
                        &mut service,
                        (),
                        cx,
                    ))
                }
                Err(error) => AnyResponse::failed(error),
            },
        }
    }
}

fn framework_args<C: CommandTrait>(args: &dyn Any) -> result::Result<&C::Args, Error> {
    args.downcast_ref::<C::Args>()
        .ok_or(Error::ArgsMismatch { command: C::NAME })
}

fn framework_args_box<C: CommandTrait>(
    args: Box<dyn Any + Send>,
) -> result::Result<C::Args, Error> {
    args.downcast::<C::Args>()
        .map(|args| *args)
        .map_err(|_| Error::ArgsMismatch { command: C::NAME })
}
