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

    pub(in crate::runtime) fn transact_notification<N: notification::Notification>(
        &mut self,
        focus: Option<crate::session::Focus>,
        window: Option<window::Id>,
        payload: N::Payload,
        source: command_context::Source,
    ) -> notification::Reaction {
        let history = self.prepare_transaction_history(command::History::Ignored);
        let revision_before = self.revision();
        let mut chain = self
            .responders
            .chain_for(&mut self.store, focus.and_then(|focus| focus.target_id()));
        let reaction = chain.notify::<N>(&payload);
        let changed = reaction.changed_state();
        log::debug!(
            "delivered notification {} from {:?}: changed={}, effect={:?}",
            N::NAME,
            source,
            changed,
            reaction.effect()
        );

        drop(chain);

        self.finish_transaction(
            history,
            None,
            window,
            focus,
            revision_before,
            state::Reason::notification(N::NAME),
            changed,
        );

        reaction
    }
}
