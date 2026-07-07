use std::any::TypeId;

use super::super::super::{Runtime, services::Services};
use super::super::AnyInvocation;
use super::super::outcome::Outcome;
use crate::{
    command, context as command_context, error::Error, responder, response::AnyResponse, state,
    timeline,
};

impl<M: state::State, E: Send + 'static, V> Runtime<M, E, V> {
    pub(in crate::runtime) fn transact_any_command(
        &mut self,
        invocation: AnyInvocation,
        invoke: impl FnOnce(
            &command::Registry,
            &mut responder::Chain<'_, M>,
            &mut command_context::Context,
        ) -> std::result::Result<Option<AnyResponse>, Error>,
    ) -> std::result::Result<Option<Outcome>, Error> {
        let AnyInvocation {
            focus,
            window,
            command_type,
            command_name,
            history_group,
            source,
        } = invocation;
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
            .with_service(services);
        let mut response = match invoke(&self.registry, &mut chain, &mut cx) {
            Ok(Some(response)) => response,
            Ok(None) => {
                drop(chain);
                drop(cx);
                self.finish_transaction(
                    before,
                    history,
                    history_group.clone(),
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
                    history_group.clone(),
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
                    history_group.clone(),
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
            history_group,
            revision_before,
            state::Reason::Command(command_name),
            changed,
        );

        let mut effect = response.effect();
        if changed
            && is_timeline_restore(command_type)
            && let Some(window) = window
            && self.session.clear_text_input(window)
        {
            effect = effect.then(crate::response::Effect::Layout);
        }

        Ok(Some(Outcome::new(response, changed, effect)))
    }
}

fn is_timeline_restore(command_type: TypeId) -> bool {
    command_type == TypeId::of::<timeline::Undo>() || command_type == TypeId::of::<timeline::Redo>()
}
