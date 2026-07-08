use super::super::Runtime;
use crate::{command, context as command_context, notification, state, window};

impl<M: state::State, E: Send + 'static, V> Runtime<M, E, V> {
    pub(crate) fn notify_focused<N: notification::Notification>(
        &mut self,
        window: window::Id,
        payload: N::Payload,
        source: command_context::Source,
    ) -> notification::Reaction {
        let focus = self.session.focused(window);
        self.transact_notification::<N>(focus, Some(window), payload, source)
    }

    fn transact_notification<N: notification::Notification>(
        &mut self,
        focus: Option<crate::session::Focus>,
        _window: Option<window::Id>,
        payload: N::Payload,
        source: command_context::Source,
    ) -> notification::Reaction {
        let revision_before = self.revision();
        let task_sink = self.tasks.sink();
        let mut cx =
            command_context::Context::with_services_source(&mut self.clipboard, task_sink, source)
                .with_text_service(self.layout.text_service());
        let mut chain = self.responders.chain_for(&mut self.store, focus);
        let reaction = chain.notify::<N>(&payload, &mut cx);
        let changed = reaction.changed_state();
        log::debug!(
            "delivered notification {} from {:?}: changed={}, effect={:?}",
            N::NAME,
            source,
            changed,
            reaction.effect()
        );

        drop(chain);
        drop(cx);

        self.finish_transaction(
            None,
            command::History::Ignored,
            None,
            revision_before,
            state::Reason::notification(N::NAME),
            changed,
        );

        reaction
    }
}
