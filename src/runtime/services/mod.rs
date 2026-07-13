use std::{
    any::{Any, TypeId},
    result,
};

use super::super::{
    composition, context, error::Error, responder, response::AnyResponse, session, state,
    timeline::Timeline, window,
};
use super::Runtime;

mod system;
mod target;
mod text;

pub(super) struct Services<'a, M: state::State> {
    timeline: &'a mut Timeline<M>,
    session: &'a mut session::Session,
    composition: &'a mut composition::Store,
    window: Option<window::Id>,
    scope: responder::Scope,
}

impl<'a, M: state::State> Services<'a, M> {
    pub(super) fn new(
        timeline: &'a mut Timeline<M>,
        session: &'a mut session::Session,
        composition: &'a mut composition::Store,
        window: Option<window::Id>,
        scope: responder::Scope,
    ) -> Self {
        Self {
            timeline,
            session,
            composition,
            window,
            scope,
        }
    }
}

impl<M: state::State, E: Send + 'static, V> Runtime<M, E, V> {
    pub(in crate::runtime) fn focused_text_owns_command(
        &self,
        window: window::Id,
        command_type: TypeId,
    ) -> bool {
        let scope = self
            .session
            .command_scope(window, self.session.focused(window));
        text::owns_command(&self.composition, Some(window), scope.focus(), command_type)
    }
}

impl<M: state::State> responder::Service<M> for Services<'_, M> {
    fn claim(
        &mut self,
        store: &mut state::Store<M>,
        command_type: TypeId,
        command_name: &'static str,
        args: &dyn Any,
        cx: &context::Context,
    ) -> result::Result<Option<responder::Claim>, Error> {
        if let Some(claim) = text::claim(
            self.session,
            self.composition,
            self.window,
            self.scope.focus(),
            self.scope.kind(),
            command_type,
            command_name,
            args,
            cx,
        )? {
            return Ok(Some(claim));
        }

        if let Some(claim) = system::claim(self, store, command_type, command_name, args, cx)? {
            return Ok(Some(claim));
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
        let text_claimed = match text::state(
            self.session,
            self.composition,
            self.window,
            self.scope.focus(),
            command_type,
            command_name,
            args.as_ref(),
            cx,
        ) {
            Ok(Some(_)) => true,
            Ok(None) => false,
            Err(error) => return Some(AnyResponse::failed(error)),
        };

        if text_claimed {
            return text::invoke(
                self.session,
                self.composition,
                self.window,
                self.scope.focus(),
                command_type,
                command_name,
                args,
                cx,
            );
        }

        system::invoke(self, store, command_type, command_name, args, cx)
    }

    fn claim_exact(
        &mut self,
        _store: &mut state::Store<M>,
        service: &'static str,
        command_type: TypeId,
        command_name: &'static str,
        args: &dyn Any,
        cx: &context::Context,
    ) -> result::Result<Option<responder::Claim>, Error> {
        if service != text::RESPONDER_NAME {
            return Ok(None);
        }
        text::claim(
            self.session,
            self.composition,
            self.window,
            self.scope.focus(),
            self.scope.kind(),
            command_type,
            command_name,
            args,
            cx,
        )
    }

    fn invoke_exact(
        &mut self,
        _store: &mut state::Store<M>,
        service: &'static str,
        command_type: TypeId,
        command_name: &'static str,
        args: Box<dyn Any + Send>,
        cx: &mut context::Context,
    ) -> Option<AnyResponse> {
        if service != text::RESPONDER_NAME {
            return None;
        }
        text::invoke(
            self.session,
            self.composition,
            self.window,
            self.scope.focus(),
            command_type,
            command_name,
            args,
            cx,
        )
    }
}

pub(super) fn contextual_targets(
    composition: &composition::Store,
    window: window::Id,
    focus: Option<session::Focus>,
) -> Vec<(TypeId, responder::Route)> {
    text::contextual_target_types(composition, Some(window), focus)
        .into_iter()
        .map(|command_type| {
            (
                command_type,
                responder::Route::Service(text::RESPONDER_NAME),
            )
        })
        .collect()
}
