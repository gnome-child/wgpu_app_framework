use super::{super::Runtime, AnyInvocation};
use crate::{command::Error, response, session, state, view, window};

impl<M: state::State, E: Send + 'static, V> Runtime<M, E, V> {
    pub(in crate::runtime) fn activate_with_focus(
        &mut self,
        focus: Option<session::Focus>,
        window: Option<window::Id>,
        binding: &view::Binding,
    ) -> std::result::Result<response::Effect, Error> {
        let source = binding.source();
        let transaction = self.transact_required_any_command(
            AnyInvocation {
                focus,
                window,
                command_type: binding.command_type(),
                command_name: binding.command_name(),
                history_group: binding.history_group(),
                source,
            },
            |registry, chain, cx| binding.invoke(registry, chain, cx),
        )?;

        transaction
            .response
            .into_result()
            .map(|_| transaction.effect)
    }
}
