use crate::text;

use super::super::super::Runtime;
use crate::{
    command, context, draft, error::Error, input, interaction, keymap, response, session, state,
    view, window,
};

pub(in crate::runtime) enum CommitAttempt {
    NotAttempted,
    Accepted(input::Outcome),
    Rejected(input::Outcome),
}

impl CommitAttempt {
    fn into_outcome(self) -> Option<input::Outcome> {
        match self {
            Self::NotAttempted => None,
            Self::Accepted(outcome) | Self::Rejected(outcome) => Some(outcome),
        }
    }
}

impl<M: state::State, E: Send + 'static, V> Runtime<M, E, V> {
    pub(in crate::runtime) fn text_draft_base(
        &self,
        window: window::Id,
        focus: session::Focus,
    ) -> Option<String> {
        self.composition.get(window)?.view().draft_text(focus)
    }

    pub(in crate::runtime) fn text_draft_input(
        &self,
        window: window::Id,
        focus: session::Focus,
    ) -> Option<text::Input> {
        self.composition.get(window)?.view().draft_input(focus)
    }

    pub(in crate::runtime) fn text_surface_mode(
        &self,
        window: window::Id,
        focus: session::Focus,
    ) -> Option<text::edit::FieldMode> {
        self.composition
            .get(window)?
            .view()
            .text_surface_mode(focus)
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
        let Some(base) = self.text_draft_base(window, focus) else {
            return Ok(None);
        };
        let mode = self
            .text_surface_mode(window, focus)
            .unwrap_or(text::edit::FieldMode::Editable);
        if !mode.allows_edit(&edit) {
            return Ok(None);
        }
        let input = self
            .text_draft_input(window, focus)
            .unwrap_or_else(text::Input::unrestricted);
        let had_table_rejection = focus
            .table_cell_identity()
            .is_some_and(|cell| self.session.table_edit_error(window, cell).is_some());
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

        let outcome =
            self.finish_text_box_change(window, focus, change.clone(), had_table_rejection)?;
        Ok(Some((change, outcome)))
    }

    fn finish_text_box_change(
        &mut self,
        window: window::Id,
        focus: session::Focus,
        change: draft::Change,
        had_table_rejection: bool,
    ) -> std::result::Result<input::Outcome, Error> {
        let mut handled = change.changed() || change.submit();
        let mut changed_state = false;
        let mut effect = if change.changed() {
            if change.text_changed()
                && (focus.same_target(&interaction::CommandPalette::query_focus())
                    || had_table_rejection)
            {
                response::Effect::Rebuild
            } else {
                response::Effect::Layout
            }
        } else {
            response::Effect::None
        };

        if change.submit()
            && let Some(outcome) = self.commit_text_box_draft(window, focus)?.into_outcome()
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
        if self.text_draft_base(window, focus).is_none() {
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
    ) -> std::result::Result<CommitAttempt, Error> {
        if self.text_draft_base(window, focus).is_none() {
            return Ok(CommitAttempt::NotAttempted);
        }
        let target = interaction::Target::text_area(focus);
        let Some(draft) = self
            .session
            .interaction(window)
            .and_then(|interaction| interaction.text_input().draft_for(&target).cloned())
        else {
            return Ok(CommitAttempt::NotAttempted);
        };
        let base = draft.base_text().to_owned();
        let text = draft.text().to_owned();
        let Some(commit) = self
            .composition
            .get(window)
            .and_then(|composition| composition.view().text_commit(focus))
        else {
            return Ok(CommitAttempt::NotAttempted);
        };
        let trigger = match commit.build(text.clone()) {
            Ok(trigger) => trigger,
            Err(reason) => {
                if let Some(cell) = focus.table_cell_identity() {
                    self.session.reject_table_edit(window, cell, reason);
                }
                return Ok(CommitAttempt::Rejected(self.window_outcome(
                    window,
                    false,
                    response::Effect::Rebuild,
                )));
            }
        };
        if let Some(cell) = focus.table_cell_identity() {
            self.session.clear_table_edit_error(window, cell);
        }

        if text == base {
            return Ok(CommitAttempt::Accepted(self.window_outcome(
                window,
                false,
                response::Effect::None,
            )));
        }

        let binding = view::Binding::from_trigger(trigger, context::Source::Input);
        let outcome = match self.handle_view(window, view::Action::activate(&binding)) {
            Ok(outcome) => outcome,
            Err(error) => {
                if let Some(cell) = focus.table_cell_identity() {
                    self.session
                        .reject_table_edit(window, cell, error.to_string());
                }
                return Ok(CommitAttempt::Rejected(self.window_outcome(
                    window,
                    false,
                    response::Effect::Rebuild,
                )));
            }
        };
        let sealed = self.session.seal_text_draft(window, focus);
        let effect = outcome.effect().clone().then(if sealed {
            response::Effect::Layout
        } else {
            response::Effect::None
        });

        Ok(CommitAttempt::Accepted(input::Outcome::handled(
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
