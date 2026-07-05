use crate::text;

use super::super::{Runtime, services};
use crate::scratch::{
    command, context as command_context, document, error::Error, input, interaction, response,
    session, state, window,
};

mod field;
mod focus;
mod shortcut;
mod transfer;

impl<M: state::State, E: Send + 'static, V> Runtime<M, E, V> {
    pub(in crate::scratch::runtime::input) fn handle_text_commit(
        &mut self,
        window: window::Id,
        text: String,
    ) -> std::result::Result<input::Outcome, Error> {
        let Some(focus) = self.session.focused(window) else {
            return Ok(input::Outcome::ignored());
        };

        if self.text_box_base_text(window, focus).is_some() {
            return self.handle_text_box_edit(window, focus, text::edit::Edit::ime_commit(text));
        }

        if text.is_empty() {
            return if self.session.clear_text_input(window) {
                Ok(self.window_outcome(window, false, response::Effect::Repaint))
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

    pub(in crate::scratch::runtime::input) fn text_input_target(
        &self,
        window: window::Id,
        focus: session::Focus,
    ) -> interaction::Target {
        self.composition
            .get(window)
            .and_then(|composition| composition.view().text_input_target(focus))
            .unwrap_or_else(|| interaction::Target::text_area(focus))
    }

    pub(in crate::scratch::runtime::input) fn handle_text_edit(
        &mut self,
        window: window::Id,
        edit: text::edit::Edit,
        source: command_context::Source,
    ) -> std::result::Result<input::Outcome, Error> {
        let Some(focus) = self.session.focused(window) else {
            return Ok(input::Outcome::ignored());
        };

        if self.text_box_base_text(window, focus).is_some() {
            return self.handle_text_box_edit(window, focus, edit);
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
        if reveal && self.session.reveal_scroll(window, reveal_target) {
            effect = effect.then(response::Effect::Repaint);
        }
        if cleared_preedit {
            effect = effect.then(response::Effect::Repaint);
            self.apply_window_update(window, false, &response::Effect::Repaint);
        }

        response
            .output
            .map(|_| input::Outcome::handled(changed, effect))
    }

    pub(in crate::scratch::runtime::input) fn handle_shortcut(
        &mut self,
        window: window::Id,
        shortcut: command::KeyChord,
    ) -> std::result::Result<input::Outcome, Error> {
        if let Some(text_box_outcome) =
            self.handle_text_box_shortcut_for_chord(window, shortcut.as_str())?
        {
            return Ok(text_box_outcome);
        }

        let Some(command) = self.registry.shortcut_command(shortcut)? else {
            return Ok(input::Outcome::ignored());
        };
        let command_type = command.command_type();
        let command_name = command.command_name();
        let history_group = command.history_group(&());
        let text_box_command =
            services::text::handles(&self.session, &self.composition, Some(window), command_type);
        let text_box_commit = if text_box_command {
            None
        } else {
            self.commit_and_deactivate_focused_text_box(window)?
        };

        let source = command_context::Source::Shortcut;
        let Some(transaction) = self.transact_any_command(
            self.session.focused(window),
            Some(window),
            command_type,
            command_name,
            history_group,
            source,
            |registry, chain, cx| registry.invoke_shortcut(shortcut, chain, cx),
        )?
        else {
            return Ok(text_box_commit.unwrap_or_else(|| {
                if text_box_command {
                    self.window_outcome(window, false, response::Effect::None)
                } else {
                    input::Outcome::ignored()
                }
            }));
        };

        let changed = text_box_commit
            .as_ref()
            .is_some_and(input::Outcome::changed_state)
            || transaction.changed_state;
        let effect = text_box_commit
            .as_ref()
            .map(|outcome| outcome.effect().clone())
            .unwrap_or(response::Effect::None)
            .then(transaction.effect);
        transaction
            .response
            .into_result()
            .map(|_| self.window_outcome(window, changed, effect))
    }
}
