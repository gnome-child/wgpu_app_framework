use super::super::super::{Runtime, services::Services};
use crate::{
    command::{self, Command},
    context as command_context,
    response::Response,
    session, state, window,
};

impl<M: state::State, E: Send + 'static, V> Runtime<M, E, V> {
    pub(in crate::runtime) fn invoke_with_focus<C: Command>(
        &mut self,
        focus: Option<session::Focus>,
        window: Option<window::Id>,
        trigger: command::Trigger<C>,
        source: command_context::Source,
    ) -> Response<C::Output> {
        self.transact_command::<C>(focus, window, trigger.into_args(), source, false)
    }

    pub(in crate::runtime) fn transact_command<C: Command>(
        &mut self,
        focus: Option<session::Focus>,
        window: Option<window::Id>,
        args: C::Args,
        source: command_context::Source,
        request_all_redraws: bool,
    ) -> Response<C::Output> {
        let history = C::HISTORY;
        let history_group = C::history_group(&args);
        let before = self.snapshot_before_transaction(history);
        let revision_before = self.revision();
        let task_sink = self.tasks.sink();
        let mut cx =
            command_context::Context::with_services_source(&mut self.clipboard, task_sink, source)
                .with_text_service(self.layout.text_service());
        let services = Services::new(
            &mut self.timeline,
            &mut self.session,
            &mut self.composition,
            &mut self.diagnostics,
            window,
        );
        let mut chain = self
            .responders
            .chain_for(&mut self.store, focus)
            .with_service(services);
        let mut response = self.registry.invoke::<C>(&mut chain, args, &mut cx);
        let command_changed = response.is_ok() && response.changed_state();

        drop(chain);
        drop(cx);

        let observer_changed = match self.observe_response::<C>(&response, source) {
            Ok(changed) => changed,
            Err(error) => {
                log::error!(
                    "command observer failed for {} from {:?}: {error}",
                    C::NAME,
                    source
                );
                self.finish_transaction(
                    before,
                    history,
                    history_group,
                    revision_before,
                    state::Reason::command(C::NAME),
                    false,
                );
                return Response::failed(error);
            }
        };
        if observer_changed {
            log::debug!("command observer changed state for {}", C::NAME);
            response.mark_changed();
        }
        let changed = response.is_ok() && (command_changed || observer_changed);

        self.finish_transaction(
            before,
            history,
            history_group,
            revision_before,
            state::Reason::command(C::NAME),
            changed,
        );
        if changed && request_all_redraws {
            self.request_all_redraws();
        }

        response
    }
}
