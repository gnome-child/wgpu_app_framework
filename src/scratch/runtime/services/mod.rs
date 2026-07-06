use std::{
    any::{Any, TypeId},
    result,
};

use super::super::{
    command, composition, context, diagnostics, error::Error, responder, response::AnyResponse,
    session, state, timeline::Timeline, window,
};

mod framework;
mod target;
pub(in crate::scratch::runtime) mod text;

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

impl<M: state::State> responder::Service<M> for Services<'_, M> {
    fn state(
        &mut self,
        store: &mut state::Store<M>,
        command_type: TypeId,
        command_name: &'static str,
        args: &dyn Any,
        cx: &context::Context,
    ) -> result::Result<Option<command::State>, Error> {
        if let Some(state) = text::state(
            self.session,
            self.composition,
            self.window,
            command_type,
            args,
            cx,
        )? {
            return Ok(Some(state));
        }

        if let Some(state) = framework::state(self, store, command_type, command_name, args, cx)? {
            return Ok(Some(state));
        }

        Ok(None)
    }

    fn invoke(
        &mut self,
        store: &mut state::Store<M>,
        command_type: TypeId,
        command_name: &'static str,
        args: Box<dyn Any + Send>,
        cx: &mut context::Context,
    ) -> Option<AnyResponse> {
        if text::has_target(self.session, self.composition, self.window, command_type) {
            return text::invoke(
                self.session,
                self.composition,
                self.window,
                command_type,
                args,
                cx,
            );
        }

        framework::invoke(self, store, command_type, command_name, args, cx)
    }
}
