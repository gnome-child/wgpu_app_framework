use super::super::Runtime;
use crate::{
    command::Error,
    command::{self, Command},
    context as command_context, response,
    response::Response,
    session, state, window,
};

impl<M: state::State, E: Send + 'static, V> Runtime<M, E, V> {
    pub fn focus(&mut self, window: window::Id, focus: session::Focus) -> bool {
        let changed = self.session.focus(window, focus);
        if changed {
            self.session
                .request_invalidation(window, response::effect::Invalidation::Layout);
        }

        changed
    }

    pub fn clear_focus(&mut self, window: window::Id) -> bool {
        let changed = self.session.clear_focus(window);
        if changed {
            self.session
                .request_invalidation(window, response::effect::Invalidation::Layout);
        }

        changed
    }

    pub fn invoke_focused<C: Command>(
        &mut self,
        window: window::Id,
        trigger: command::Trigger<C>,
    ) -> Response<C::Output> {
        self.invoke_focused_with_source(window, trigger, command_context::Source::Programmatic)
    }

    pub(in crate::runtime::input) fn invoke_focused_with_source<C: Command>(
        &mut self,
        window: window::Id,
        trigger: command::Trigger<C>,
        source: command_context::Source,
    ) -> Response<C::Output> {
        if !self.session.contains(window) {
            return Response::failed(Error::MissingTarget { command: C::NAME });
        }

        let response =
            self.invoke_with_focus(self.session.focused(window), Some(window), trigger, source);
        if response.is_ok() {
            self.apply_window_update(window, response.changed_state(), &response.effect);
        }

        response
    }
}
