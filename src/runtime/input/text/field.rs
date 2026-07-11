use crate::text;

use super::super::super::Runtime;
use crate::{
    command, draft, error::Error, input, interaction, keymap, response, session, state, window,
};

impl<M: state::State, E: Send + 'static, V> Runtime<M, E, V> {
    pub(in crate::runtime) fn text_box_base_text(
        &self,
        window: window::Id,
        focus: session::Focus,
    ) -> Option<String> {
        self.composition
            .get(window)?
            .view()
            .text_box_text(focus)
            .map(str::to_owned)
    }

    pub(in crate::runtime) fn text_box_input(
        &self,
        window: window::Id,
        focus: session::Focus,
    ) -> Option<text::Input> {
        self.composition.get(window)?.view().text_box_input(focus)
    }

    pub(super) fn handle_text_box_edit(
        &mut self,
        window: window::Id,
        focus: session::Focus,
        edit: text::edit::Edit,
    ) -> std::result::Result<input::Outcome, Error> {
        let Some((_, outcome)) = self.handle_text_box_edit_with_change(window, focus, edit)? else {
            return Ok(input::Outcome::ignored());
        };

        Ok(outcome)
    }

    fn handle_text_box_edit_with_change(
        &mut self,
        window: window::Id,
        focus: session::Focus,
        edit: text::edit::Edit,
    ) -> std::result::Result<Option<(draft::Change, input::Outcome)>, Error> {
        let Some(base) = self.text_box_base_text(window, focus) else {
            return Ok(None);
        };
        let input = self
            .text_box_input(window, focus)
            .unwrap_or_else(text::Input::unrestricted);
        let Some(change) = self
            .session
            .edit_text_draft(window, focus, base, edit, input)
        else {
            return Ok(None);
        };
        if focus.same_target(&interaction::CommandPalette::query_focus())
            && change.text_changed()
            && self.session.reset_command_palette_selection(window)
        {
            self.session
                .request_invalidation(window, response::Invalidation::Rebuild);
        }

        let outcome = self.finish_text_box_change(window, focus, change.clone())?;
        Ok(Some((change, outcome)))
    }

    fn finish_text_box_change(
        &mut self,
        window: window::Id,
        focus: session::Focus,
        change: draft::Change,
    ) -> std::result::Result<input::Outcome, Error> {
        let mut handled = change.changed() || change.submit();
        let mut changed_state = false;
        let mut effect = if change.changed() {
            if focus.same_target(&interaction::CommandPalette::query_focus())
                && change.text_changed()
            {
                response::Effect::Rebuild
            } else {
                response::Effect::Layout
            }
        } else {
            response::Effect::None
        };

        if change.submit()
            && let Some(outcome) = self.commit_text_box_draft(window, focus)?
        {
            handled |= outcome.is_handled();
            changed_state |= outcome.changed_state();
            effect = effect.then(outcome.effect().clone());
        }

        if handled {
            Ok(self.window_outcome(window, changed_state, effect))
        } else {
            Ok(input::Outcome::ignored())
        }
    }

    pub(in crate::runtime::input) fn handle_text_box_key_shortcut(
        &mut self,
        window: window::Id,
        key: input::Key,
        modifiers: input::Modifiers,
    ) -> std::result::Result<Option<input::Outcome>, Error> {
        if self.keymap.text_box_shortcut_for_key(key, modifiers)
            != Some(keymap::TextBoxShortcut::ClearSelection)
        {
            return Ok(None);
        }

        self.handle_text_box_clear_selection_shortcut(window)
    }

    fn handle_text_box_clear_selection_shortcut(
        &mut self,
        window: window::Id,
    ) -> std::result::Result<Option<input::Outcome>, Error> {
        let Some(focus) = self.session.command_focus(window) else {
            return Ok(None);
        };
        if self.text_box_base_text(window, focus).is_none() {
            return Ok(None);
        }
        if !self
            .session
            .focused(window)
            .is_some_and(|current| current.same_target(&focus))
        {
            self.focus(window, focus);
        }

        let outcome = self.handle_text_box_edit(
            window,
            focus,
            text::edit::Edit::MovePosition(text::edit::Motion::VisualRight),
        )?;

        Ok(Some(outcome))
    }

    pub(in crate::runtime) fn commit_text_box_draft(
        &mut self,
        window: window::Id,
        focus: session::Focus,
    ) -> std::result::Result<Option<input::Outcome>, Error> {
        if self.text_box_base_text(window, focus).is_none() {
            return Ok(None);
        }
        let target = interaction::Target::text_area(focus);
        let Some(draft) = self
            .session
            .interaction(window)
            .and_then(|interaction| interaction.text_input().draft_for(&target).cloned())
        else {
            return Ok(None);
        };
        let base = draft.base_text().to_owned();
        let text = draft.text().to_owned();
        let table_action = self
            .composition
            .get(window)
            .and_then(|composition| composition.view().table_edit_action(focus, text.to_owned()));
        let action = match table_action {
            Some(Err((cell, reason))) => {
                self.session.reject_table_edit(window, cell, reason);
                return Ok(Some(self.window_outcome(
                    window,
                    false,
                    response::Effect::Layout,
                )));
            }
            Some(Ok((cell, action))) => {
                self.session.clear_table_edit_error(window, cell);
                Some(action)
            }
            None => self.composition.get(window).and_then(|composition| {
                composition
                    .view()
                    .text_commit_action(focus, text.to_owned())
            }),
        };
        let Some(action) = action else {
            return Ok(None);
        };

        if text == base {
            return Ok(Some(self.window_outcome(
                window,
                false,
                response::Effect::None,
            )));
        }

        let outcome = self.handle_view(window, action)?;
        let sealed = self.session.seal_text_draft(window, focus);
        let effect = outcome.effect().clone().then(if sealed {
            response::Effect::Layout
        } else {
            response::Effect::None
        });

        Ok(Some(input::Outcome::handled(
            outcome.changed_state(),
            effect,
        )))
    }

    pub(super) fn handle_text_box_shortcut_for_chord(
        &mut self,
        window: window::Id,
        shortcut: command::KeyChord,
    ) -> std::result::Result<Option<input::Outcome>, Error> {
        if self.keymap.text_box_shortcut_for_chord(shortcut)
            != Some(keymap::TextBoxShortcut::ClearSelection)
        {
            return Ok(None);
        }

        self.handle_text_box_clear_selection_shortcut(window)
    }
}
