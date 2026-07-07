use std::any::TypeId;

use super::super::super::Runtime;
use crate::scratch::{error::Error, input, response, session, state, window};

pub(in crate::scratch::runtime) struct FocusedTextCommand {
    owned_by_text: bool,
    committed: Option<input::Outcome>,
}

impl FocusedTextCommand {
    pub(in crate::scratch::runtime) fn is_owned_by_text(&self) -> bool {
        self.owned_by_text
    }

    pub(in crate::scratch::runtime) fn committed(&self) -> Option<&input::Outcome> {
        self.committed.as_ref()
    }

    pub(in crate::scratch::runtime) fn into_committed(self) -> Option<input::Outcome> {
        self.committed
    }
}

impl<M: state::State, E: Send + 'static, V> Runtime<M, E, V> {
    pub(in crate::scratch::runtime) fn prepare_focused_text_for_command(
        &mut self,
        window: window::Id,
        command_type: TypeId,
    ) -> std::result::Result<FocusedTextCommand, Error> {
        let owned_by_text = self.focused_text_owns_command(window, command_type);
        let preserves_focus = command_type == TypeId::of::<session::OpenCommandPalette>();
        let committed = if owned_by_text || preserves_focus {
            None
        } else {
            self.commit_and_deactivate_focused_text_box(window)?
        };

        Ok(FocusedTextCommand {
            owned_by_text,
            committed,
        })
    }

    pub(in crate::scratch::runtime) fn focus_committing_text_box(
        &mut self,
        window: window::Id,
        focus: session::Focus,
    ) -> std::result::Result<input::Outcome, Error> {
        let mut handled = false;
        let mut changed_state = false;
        let mut effect = response::Effect::None;

        if let Some(current) = self.session.focused(window)
            && !current.same_target(&focus)
            && let Some(outcome) = self.commit_and_deactivate_focused_text_box(window)?
        {
            handled |= outcome.is_handled();
            changed_state |= outcome.changed_state();
            effect = effect.then(outcome.effect().clone());
        }

        let focus_changed = self.focus(window, focus);
        if focus_changed {
            effect = effect.then(response::Effect::Layout);
        }

        if handled || focus_changed {
            Ok(input::Outcome::handled(changed_state, effect))
        } else {
            Ok(input::Outcome::ignored())
        }
    }

    pub(in crate::scratch::runtime) fn clear_focus_committing_text_box(
        &mut self,
        window: window::Id,
    ) -> std::result::Result<input::Outcome, Error> {
        let mut handled = false;
        let mut changed_state = false;
        let mut effect = response::Effect::None;

        if let Some(outcome) = self.commit_and_deactivate_focused_text_box(window)? {
            handled |= outcome.is_handled();
            changed_state |= outcome.changed_state();
            effect = effect.then(outcome.effect().clone());
        }

        let focus_changed = self.clear_focus(window);
        if focus_changed {
            effect = effect.then(response::Effect::Layout);
        }

        if handled || focus_changed {
            Ok(input::Outcome::handled(changed_state, effect))
        } else {
            Ok(input::Outcome::ignored())
        }
    }

    pub(in crate::scratch::runtime) fn commit_and_deactivate_focused_text_box(
        &mut self,
        window: window::Id,
    ) -> std::result::Result<Option<input::Outcome>, Error> {
        let Some(current) = self.session.focused(window) else {
            return Ok(None);
        };
        if self.text_box_base_text(window, current).is_none() {
            return Ok(None);
        }

        let mut handled = false;
        let mut changed_state = false;
        let mut effect = response::Effect::None;

        if let Some(outcome) = self.commit_text_box_draft(window, current)? {
            handled |= outcome.is_handled();
            changed_state |= outcome.changed_state();
            effect = effect.then(outcome.effect().clone());
        }

        if self.session.deactivate_text_draft(window, current) {
            handled = true;
            effect = effect.then(response::Effect::Layout);
        }

        if handled {
            self.apply_window_update(window, changed_state, &effect);
            Ok(Some(input::Outcome::handled(changed_state, effect)))
        } else {
            Ok(None)
        }
    }
}
