use std::any::TypeId;

use super::super::{Runtime, services::Services};
use crate::scratch::{
    command::{self, Command},
    context as command_context,
    error::Error,
    responder,
    response::{AnyResponse, Response},
    session, state, window,
};

impl<M: state::State, E: Send + 'static, V> Runtime<M, E, V> {
    pub(in crate::scratch::runtime) fn invoke_with_focus<C: Command>(
        &mut self,
        focus: Option<session::Focus>,
        window: Option<window::Id>,
        trigger: command::Trigger<C>,
        source: command_context::Source,
    ) -> Response<C::Output> {
        self.transact_command::<C>(focus, window, trigger.into_args(), source, false)
    }

    pub(in crate::scratch::runtime) fn transact_command<C: Command>(
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
            .with_framework(services);
        let mut response = self.registry.invoke::<C>(&mut chain, args, &mut cx);
        let command_changed = response.is_ok() && response.changed_state();

        drop(chain);
        drop(cx);

        let observer_changed = match self.observe_response::<C>(&response, source) {
            Ok(changed) => changed,
            Err(error) => {
                self.finish_transaction(
                    before,
                    history,
                    history_group,
                    revision_before,
                    state::Reason::command::<C>(),
                    false,
                );
                return Response::failed(error);
            }
        };
        if observer_changed {
            response.mark_changed();
        }
        let changed = response.is_ok() && (command_changed || observer_changed);

        self.finish_transaction(
            before,
            history,
            history_group,
            revision_before,
            state::Reason::command::<C>(),
            changed,
        );
        if changed && request_all_redraws {
            self.request_all_redraws();
        }

        response
    }

    pub(in crate::scratch::runtime) fn transact_any_command(
        &mut self,
        focus: Option<session::Focus>,
        window: Option<window::Id>,
        command_type: TypeId,
        command_name: &'static str,
        source: command_context::Source,
        invoke: impl FnOnce(
            &command::Registry,
            &mut responder::Chain<'_, M>,
            &mut command_context::Context,
        ) -> std::result::Result<Option<AnyResponse>, Error>,
    ) -> std::result::Result<Option<super::outcome::Outcome>, Error> {
        let history = self
            .registry
            .history_for(command_type)
            .unwrap_or(command::History::Automatic);
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
            .with_framework(services);
        let mut response = match invoke(&self.registry, &mut chain, &mut cx) {
            Ok(Some(response)) => response,
            Ok(None) => {
                drop(chain);
                drop(cx);
                self.finish_transaction(
                    before,
                    history,
                    None,
                    revision_before,
                    state::Reason::Command(command_name),
                    false,
                );
                return Ok(None);
            }
            Err(error) => {
                drop(chain);
                drop(cx);
                self.finish_transaction(
                    before,
                    history,
                    None,
                    revision_before,
                    state::Reason::Command(command_name),
                    false,
                );
                return Err(error);
            }
        };
        let command_changed = response.is_ok() && response.changed_state();

        drop(chain);
        drop(cx);

        let observer_changed = match self.observe_any_response(command_type, &response, source) {
            Ok(changed) => changed,
            Err(error) => {
                self.finish_transaction(
                    before,
                    history,
                    None,
                    revision_before,
                    state::Reason::Command(command_name),
                    false,
                );
                return Err(error);
            }
        };
        if observer_changed {
            response.mark_changed();
        }
        let changed = response.is_ok() && (command_changed || observer_changed);

        self.finish_transaction(
            before,
            history,
            None,
            revision_before,
            state::Reason::Command(command_name),
            changed,
        );

        let effect = response.effect();

        Ok(Some(super::outcome::Outcome::new(
            response, changed, effect,
        )))
    }
}
