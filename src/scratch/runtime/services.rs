use std::{
    any::{Any, TypeId},
    result,
};

use super::super::{
    command::{self, Command},
    composition, context, diagnostics,
    error::Error,
    responder,
    response::AnyResponse,
    session, state,
    target::Target,
    timeline::{self, Timeline},
    window,
};

pub(super) struct Services<'a, M: state::State> {
    timeline: &'a mut Timeline<M>,
    session: &'a mut session::Session,
    composition: &'a mut composition::Store,
    diagnostics: &'a mut diagnostics::Store,
    window: Option<window::Id>,
}

impl<'a, M: state::State> Services<'a, M> {
    pub(super) fn new(
        timeline: &'a mut Timeline<M>,
        session: &'a mut session::Session,
        composition: &'a mut composition::Store,
        diagnostics: &'a mut diagnostics::Store,
        window: Option<window::Id>,
    ) -> Self {
        Self {
            timeline,
            session,
            composition,
            diagnostics,
            window,
        }
    }
}

impl<M: state::State> responder::Framework<M> for Services<'_, M> {
    fn state(
        &mut self,
        store: &mut state::Store<M>,
        command_type: TypeId,
        _command_name: &'static str,
        args: &dyn Any,
        cx: &context::Context,
    ) -> result::Result<Option<command::State>, Error> {
        if command_type == TypeId::of::<timeline::Undo>() {
            let args = framework_args::<timeline::Undo>(args)?;
            let service = timeline::Service::new(store, &mut *self.timeline);
            return Ok(Some(Target::<timeline::Undo>::state(&service, args, cx)));
        }

        if command_type == TypeId::of::<timeline::Redo>() {
            let args = framework_args::<timeline::Redo>(args)?;
            let service = timeline::Service::new(store, &mut *self.timeline);
            return Ok(Some(Target::<timeline::Redo>::state(&service, args, cx)));
        }

        if command_type == TypeId::of::<session::CloseWindow>() {
            let args = framework_args::<session::CloseWindow>(args)?;
            let service = session::Service::new(
                &mut *self.session,
                &mut *self.composition,
                &mut *self.diagnostics,
                self.window,
            );
            return Ok(Some(Target::<session::CloseWindow>::state(
                &service, args, cx,
            )));
        }

        Ok(None)
    }

    fn invoke(
        &mut self,
        store: &mut state::Store<M>,
        command_type: TypeId,
        _command_name: &'static str,
        args: Box<dyn Any + Send>,
        cx: &mut context::Context,
    ) -> Option<AnyResponse> {
        if command_type == TypeId::of::<timeline::Undo>() {
            match framework_args_box::<timeline::Undo>(args) {
                Ok(()) => {}
                Err(error) => return Some(AnyResponse::failed(error)),
            }
            let mut service = timeline::Service::new(store, &mut *self.timeline);
            return Some(AnyResponse::from_response(
                Target::<timeline::Undo>::invoke(&mut service, (), cx),
            ));
        }

        if command_type == TypeId::of::<timeline::Redo>() {
            match framework_args_box::<timeline::Redo>(args) {
                Ok(()) => {}
                Err(error) => return Some(AnyResponse::failed(error)),
            }
            let mut service = timeline::Service::new(store, &mut *self.timeline);
            return Some(AnyResponse::from_response(
                Target::<timeline::Redo>::invoke(&mut service, (), cx),
            ));
        }

        if command_type == TypeId::of::<session::CloseWindow>() {
            match framework_args_box::<session::CloseWindow>(args) {
                Ok(()) => {}
                Err(error) => return Some(AnyResponse::failed(error)),
            }
            let mut service = session::Service::new(
                &mut *self.session,
                &mut *self.composition,
                &mut *self.diagnostics,
                self.window,
            );
            return Some(AnyResponse::from_response(
                Target::<session::CloseWindow>::invoke(&mut service, (), cx),
            ));
        }

        None
    }
}

fn framework_args<C: Command>(args: &dyn Any) -> result::Result<&C::Args, Error> {
    args.downcast_ref::<C::Args>()
        .ok_or(Error::ArgsMismatch { command: C::NAME })
}

fn framework_args_box<C: Command>(args: Box<dyn Any + Send>) -> result::Result<C::Args, Error> {
    args.downcast::<C::Args>()
        .map(|args| *args)
        .map_err(|_| Error::ArgsMismatch { command: C::NAME })
}
