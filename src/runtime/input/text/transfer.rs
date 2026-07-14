use super::super::super::Runtime;
use crate::{command::Error, context as command_context, document, input, response, state, window};
use std::time::Instant;

impl<M: state::State, E: Send + 'static, V> Runtime<M, E, V> {
    pub(in crate::runtime::input) fn handle_text_drop(
        &mut self,
        window: window::Id,
        text_drop: input::TextDrop,
    ) -> std::result::Result<input::Outcome, Error> {
        let before = self.store.prepare_snapshot();
        let focus = self.session.focused(window);
        let reveal_target = focus.map(|focus| self.text_input_target(window, focus));
        let (edit, source_cleanup) = text_drop.into_edits();
        let task_sink = self.tasks.sink();
        let mut cx = command_context::Context::with_clipboard_source(
            &mut self.clipboard,
            command_context::Source::Input,
        )
        .with_tasks(task_sink)
        .with_caret_map(self.layout.text_caret_map());
        let mut chain = self.responders.chain_for(&mut self.store, focus);

        let response = self
            .registry
            .invoke::<document::ApplyEdit>(&mut chain, edit, &mut cx);
        let mut changed = response.changed_state();
        let mut effect = response.effect.clone();

        if let Err(error) = response.output {
            drop(chain);
            if changed {
                self.store.discard_retained_snapshot();
            } else {
                self.store.restore_prepared_snapshot(before);
            }
            return Err(error);
        }

        if changed && let Some(source_cleanup) = source_cleanup {
            let cleanup_response =
                self.registry
                    .invoke::<document::ApplyEdit>(&mut chain, source_cleanup, &mut cx);

            changed |= cleanup_response.changed_state();
            effect = effect.then(cleanup_response.effect.clone());

            if let Err(error) = cleanup_response.output {
                drop(chain);
                if changed {
                    self.store.discard_retained_snapshot();
                } else {
                    self.store.restore_prepared_snapshot(before);
                }
                return Err(error);
            }
        }

        drop(chain);

        if changed {
            if let Some(target) = reveal_target
                && self.session.reveal_scroll(window, target)
            {
                effect = effect.then(response::Effect::Layout);
            }
            if let Some(focus) = focus {
                let target = self.text_input_target(window, focus);
                if self
                    .session
                    .reset_text_caret_blink(window, target, Instant::now())
                    && !changed
                {
                    effect = effect.then(response::Effect::Layout);
                }
            }
            self.timeline.record(before.into_model());
            self.store
                .commit_retaining_current(state::Reason::event("text_drop"));
        } else {
            self.store.restore_prepared_snapshot(before);
        }

        Ok(self.window_outcome(window, changed, effect))
    }
}
