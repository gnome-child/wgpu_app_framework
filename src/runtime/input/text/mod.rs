use crate::text;
use std::time::Instant;

use super::super::{Runtime, transaction};
use crate::{
    command, context as command_context, document, error::Error, input, interaction, response,
    session, state, window,
};

mod field;
mod focus;
mod transfer;

impl<M: state::State, E: Send + 'static, V> Runtime<M, E, V> {
    pub(in crate::runtime::input) fn handle_text_commit(
        &mut self,
        window: window::Id,
        text: String,
    ) -> std::result::Result<input::Outcome, Error> {
        let Some(focus) = self.session.focused(window) else {
            return Ok(input::Outcome::ignored());
        };

        if self.text_draft_base(window, focus).is_some() {
            if self.text_surface_mode(window, focus) != Some(text::edit::FieldMode::Editable) {
                return Ok(input::Outcome::ignored());
            }
            return self.handle_text_box_edit(window, focus, text::edit::Edit::ime_commit(text));
        }

        if text.is_empty() {
            return if self.session.clear_text_input(window) {
                Ok(self.window_outcome(window, false, response::Effect::Layout))
            } else {
                Ok(input::Outcome::ignored())
            };
        }

        self.handle_text_edit(
            window,
            text::edit::Edit::ime_commit(text),
            command_context::Source::Input,
        )
    }

    pub(in crate::runtime::input) fn text_input_target(
        &self,
        window: window::Id,
        focus: session::Focus,
    ) -> interaction::Target {
        self.composition
            .get(window)
            .and_then(|composition| composition.view().text_input_target(focus))
            .unwrap_or_else(|| interaction::Target::text_area(focus))
    }

    pub(in crate::runtime::input) fn handle_text_edit(
        &mut self,
        window: window::Id,
        edit: text::edit::Edit,
        source: command_context::Source,
    ) -> std::result::Result<input::Outcome, Error> {
        let Some(focus) = self.session.focused(window) else {
            return Ok(input::Outcome::ignored());
        };

        if self.text_draft_base(window, focus).is_some() {
            return self.handle_text_box_edit(window, focus, edit);
        }

        if focus.table_cell_identity().is_some() {
            log::debug!(
                "ignoring text input for a table cell without a current local draft in window {window:?}"
            );
            return Ok(input::Outcome::ignored());
        }

        let reveal_target = self.text_input_target(window, focus);
        let cleared_preedit = self.session.clear_text_input(window);
        let response = self.invoke_focused_with_source(
            window,
            command::Trigger::<document::ApplyEdit>::command(edit),
            source,
        );
        let changed = response.changed_state();
        let reveal = response
            .output_ref()
            .is_some_and(|outcome| outcome.buffer_changed());
        let mut effect = response.effect.clone();
        if reveal && self.session.reveal_scroll(window, reveal_target.clone()) {
            effect = effect.then(response::Effect::Layout);
        }
        if response.output.is_ok()
            && self
                .session
                .reset_text_caret_blink(window, reveal_target, Instant::now())
            && !changed
        {
            effect = effect.then(response::Effect::Layout);
        }
        if cleared_preedit {
            effect = effect.then(response::Effect::Layout);
            self.apply_window_update(window, false, &response::Effect::Layout);
        }

        response
            .output
            .map(|_| input::Outcome::handled(changed, effect))
    }

    pub(in crate::runtime::input) fn handle_shortcut(
        &mut self,
        window: window::Id,
        shortcut: command::KeyChord,
    ) -> std::result::Result<input::Outcome, Error> {
        if let Some(text_box_outcome) = self.handle_text_box_shortcut_for_chord(window, shortcut)? {
            return Ok(text_box_outcome);
        }

        let Some(command) = self.registry.shortcut_command(shortcut, self.keymap)? else {
            return Ok(input::Outcome::ignored());
        };
        let command_type = command.command_type();
        let command_name = command.command_name();
        let history_group = command.history_group(&());
        let focused_text = self.prepare_focused_text_for_command(window, command_type)?;
        if !focused_text.is_accepted() {
            return Ok(focused_text
                .into_committed()
                .unwrap_or_else(input::Outcome::ignored));
        }
        let keymap = self.keymap;

        let source = command_context::Source::Shortcut;
        let Some(transaction) = self.transact_any_command(
            transaction::AnyInvocation {
                focus: self.session.focused(window),
                window: Some(window),
                command_type,
                command_name,
                history_group,
                source,
            },
            |registry, chain, cx| registry.invoke_shortcut(shortcut, keymap, chain, cx),
        )?
        else {
            let owned_by_text = focused_text.is_owned_by_text();

            return Ok(focused_text.into_committed().unwrap_or_else(|| {
                if owned_by_text {
                    self.window_outcome(window, false, response::Effect::None)
                } else {
                    input::Outcome::ignored()
                }
            }));
        };

        let changed = focused_text
            .committed()
            .is_some_and(input::Outcome::changed_state)
            || transaction.changed_state;
        let effect = focused_text
            .committed()
            .map(|outcome| outcome.effect().clone())
            .unwrap_or(response::Effect::None)
            .then(transaction.effect);
        transaction
            .response
            .into_result()
            .map(|_| self.window_outcome(window, changed, effect))
    }
}
