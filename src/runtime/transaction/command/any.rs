use std::any::TypeId;

use super::super::super::{Runtime, services::Services};
use super::super::outcome::Outcome;
use super::super::{AnyInvocation, history};
use crate::{
    command::{self, Error},
    context as command_context, responder,
    response::AnyResponse,
    session, state, timeline,
};

struct Prepared<M: state::State> {
    invocation: AnyInvocation,
    history: history::Plan<M>,
    revision_before: state::Revision,
}

impl<M: state::State> Prepared<M> {
    fn command_type(&self) -> TypeId {
        self.invocation.command_type
    }

    fn command_name(&self) -> &'static str {
        self.invocation.command_name
    }

    fn source(&self) -> command_context::Source {
        self.invocation.source
    }

    fn window(&self) -> Option<crate::window::Id> {
        self.invocation.window
    }

    fn finish<E: Send + 'static, V>(self, runtime: &mut Runtime<M, E, V>, changed: bool) {
        let Self {
            invocation:
                AnyInvocation {
                    focus,
                    window,
                    command_name,
                    history_group,
                    ..
                },
            history,
            revision_before,
        } = self;
        runtime.finish_transaction(
            history,
            history_group,
            window,
            focus,
            revision_before,
            state::Reason::Command(command_name),
            changed,
        );
    }
}

impl<M: state::State, E: Send + 'static, V> Runtime<M, E, V> {
    pub(in crate::runtime) fn transact_required_any_command(
        &mut self,
        invocation: AnyInvocation,
        invoke: impl FnOnce(
            &command::Registry,
            &mut responder::Chain<'_, M>,
            &mut command_context::Context,
        ) -> AnyResponse,
    ) -> std::result::Result<Outcome, Error> {
        let (transaction, response) = self
            .invoke_any_command(invocation, |registry, chain, cx| {
                Ok(invoke(registry, chain, cx))
            })?;
        self.complete_any_command(transaction, response)
    }

    pub(in crate::runtime) fn transact_optional_any_command(
        &mut self,
        invocation: AnyInvocation,
        invoke: impl FnOnce(
            &command::Registry,
            &mut responder::Chain<'_, M>,
            &mut command_context::Context,
        ) -> std::result::Result<Option<AnyResponse>, Error>,
    ) -> std::result::Result<Option<Outcome>, Error> {
        let (transaction, response) = self.invoke_any_command(invocation, invoke)?;
        let Some(response) = response else {
            log::debug!(
                "command invocation produced no target or command: {} from {:?}",
                transaction.command_name(),
                transaction.source()
            );
            transaction.finish(self, false);
            return Ok(None);
        };

        self.complete_any_command(transaction, response).map(Some)
    }

    fn invoke_any_command<R>(
        &mut self,
        invocation: AnyInvocation,
        invoke: impl FnOnce(
            &command::Registry,
            &mut responder::Chain<'_, M>,
            &mut command_context::Context,
        ) -> std::result::Result<R, Error>,
    ) -> std::result::Result<(Prepared<M>, R), Error> {
        let history = self
            .registry
            .history_for(invocation.command_type)
            .unwrap_or(command::History::Automatic);
        let transaction = Prepared {
            invocation,
            history: self.prepare_transaction_history(history),
            revision_before: self.revision(),
        };
        let focus = transaction.invocation.focus;
        let window = transaction.invocation.window;
        let command_name = transaction.command_name();
        let source = transaction.source();
        let task_sink = self.tasks.sink();
        let scope = window
            .and_then(|window| self.context_menu_scope(window))
            .or_else(|| window.map(|window| self.session.command_scope(window, focus)))
            .unwrap_or_else(|| session::CommandScope::focused(focus));
        let mut cx = command_context::Context::with_clipboard_source(&mut self.clipboard, source)
            .with_tasks(task_sink)
            .with_caret_map(self.layout.text_caret_map());
        let services = Services::new(
            &mut self.timeline,
            &mut self.session,
            &mut self.composition,
            window,
            scope,
        );
        let mut chain = self
            .responders
            .chain_for_scope(&mut self.store, scope.routing())
            .with_service(services);
        let response = match invoke(&self.registry, &mut chain, &mut cx) {
            Ok(response) => response,
            Err(error) => {
                log::warn!(
                    "command invocation failed before dispatch: {command_name} from {source:?}: {error}"
                );
                drop(chain);
                drop(cx);
                transaction.finish(self, false);
                return Err(error);
            }
        };
        drop(chain);
        drop(cx);

        Ok((transaction, response))
    }

    fn complete_any_command(
        &mut self,
        transaction: Prepared<M>,
        mut response: AnyResponse,
    ) -> std::result::Result<Outcome, Error> {
        let command_type = transaction.command_type();
        let command_name = transaction.command_name();
        let source = transaction.source();
        let window = transaction.window();
        let command_changed = response.is_ok() && response.changed_state();

        let observer_changed = match self.observe_any_response(command_type, &response, source) {
            Ok(changed) => changed,
            Err(error) => {
                log::error!("command observer failed for {command_name} from {source:?}: {error}");
                transaction.finish(self, false);
                return Err(error);
            }
        };
        if observer_changed {
            log::debug!("command observer changed state for {command_name}");
            response.mark_changed();
        }
        let changed = response.is_ok() && (command_changed || observer_changed);

        transaction.finish(self, changed);

        let mut effect = response.effect();
        if changed
            && is_timeline_restore(command_type)
            && let Some(window) = window
            && self.session.clear_text_input(window)
        {
            effect = effect.then(crate::response::Effect::Layout);
        }

        Ok(Outcome::new(response, changed, effect))
    }
}

fn is_timeline_restore(command_type: TypeId) -> bool {
    command_type == TypeId::of::<timeline::Undo>() || command_type == TypeId::of::<timeline::Redo>()
}
