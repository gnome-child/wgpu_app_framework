use super::super::{
    command::{self, Command},
    context as command_context, responder,
    response::Response,
    state, window,
};
use super::{Runtime, services::Services};

impl<M: state::State, E: Send + 'static, V> Runtime<M, E, V> {
    pub fn trigger<C: Command>(&self, args: C::Args) -> command::Trigger<C> {
        command::Trigger::command(args)
    }

    pub fn state_for<C: Command>(&mut self, trigger: &command::Trigger<C>) -> command::State {
        let cx = command_context::Context::with_clipboard_source(
            &mut self.clipboard,
            command_context::Source::Programmatic,
        );
        let services = Services::new(
            &mut self.timeline,
            &mut self.session,
            &mut self.composition,
            None,
            responder::Scope::focused(None),
        );
        let mut chain = self
            .responders
            .chain(&mut self.store)
            .with_service(services);

        self.registry.state::<C>(&mut chain, trigger.args(), &cx)
    }

    pub fn state_for_focused<C: Command>(
        &mut self,
        window: window::Id,
        trigger: &command::Trigger<C>,
    ) -> command::State {
        if !self.session.contains(window) {
            return self
                .registry
                .apply_spec::<C>(command::State::disabled().with_hint("window is not open"));
        }

        let focus = self.session.focused(window);
        let scope = self.session.command_scope(window, focus);
        let cx = command_context::Context::with_clipboard_source(
            &mut self.clipboard,
            command_context::Source::Programmatic,
        );
        let services = Services::new(
            &mut self.timeline,
            &mut self.session,
            &mut self.composition,
            Some(window),
            scope,
        );
        let mut chain = self
            .responders
            .chain_for(&mut self.store, focus)
            .with_service(services);

        self.registry.state::<C>(&mut chain, trigger.args(), &cx)
    }

    pub fn invoke<C: Command>(&mut self, trigger: command::Trigger<C>) -> Response<C::Output> {
        self.transact_command::<C>(
            None,
            None,
            trigger.into_args(),
            command_context::Source::Programmatic,
            true,
        )
    }
}
