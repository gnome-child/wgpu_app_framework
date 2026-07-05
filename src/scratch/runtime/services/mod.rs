use std::{
    any::{Any, TypeId},
    result,
};

use super::super::{
    command, composition, context, diagnostics, error::Error, responder, response::AnyResponse,
    session, state, timeline::Timeline, window,
};

mod framework;
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

impl<M: state::State> responder::Framework<M> for Services<'_, M> {
    fn state(
        &mut self,
        store: &mut state::Store<M>,
        command_type: TypeId,
        _command_name: &'static str,
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

        if let Some(command) = framework::Command::from_type(command_type) {
            return command.state(self, store, args, cx).map(Some);
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
        if text::handles(self.session, self.composition, self.window, command_type) {
            return text::invoke(
                self.session,
                self.composition,
                self.window,
                command_type,
                args,
                cx,
            );
        }

        framework::Command::from_type(command_type)
            .map(|command| command.invoke(self, store, args, cx))
    }
}
