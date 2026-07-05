use super::super::Runtime;
use crate::scratch::{error::Error, response, session, state, view, window};

impl<M: state::State, E: Send + 'static, V> Runtime<M, E, V> {
    pub(in crate::scratch::runtime) fn activate_with_focus(
        &mut self,
        focus: Option<session::Focus>,
        window: Option<window::Id>,
        binding: &view::Binding,
    ) -> std::result::Result<response::Effect, Error> {
        let source = binding.source();
        let transaction = self
            .transact_any_command(
                focus,
                window,
                binding.command_type(),
                binding.command_name(),
                source,
                |registry, chain, cx| Ok(Some(binding.invoke(registry, chain, cx))),
            )?
            .expect("view binding activation always invokes a command");

        transaction
            .response
            .into_result()
            .map(|_| transaction.effect)
    }
}
