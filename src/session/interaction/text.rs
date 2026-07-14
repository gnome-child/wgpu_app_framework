use std::time::Instant;

use crate::text;

use crate::{draft, feedback, interaction, window as app_window};

use super::super::{Focus, Session};

impl Session {
    pub(crate) fn activate_text_draft(
        &mut self,
        id: app_window::Id,
        focus: Focus,
        base: impl Into<String>,
    ) -> bool {
        let Some(window) = self.window_mut(id) else {
            return false;
        };
        if !window
            .focus
            .as_ref()
            .is_some_and(|current| current.same_target(&focus))
        {
            return false;
        }
        let Some(target) = focus.text_target() else {
            return false;
        };

        window.interaction.activate_text_draft(target, base)
    }

    pub fn set_text_preedit(&mut self, id: app_window::Id, preedit: text::Preedit) -> Option<bool> {
        let window = self.window_mut(id)?;
        let target = window.focus?.text_target()?;

        Some(window.interaction.set_text_preedit(target, preedit))
    }

    pub fn set_text_preedit_for(
        &mut self,
        id: app_window::Id,
        target: interaction::Target,
        preedit: text::Preedit,
    ) -> bool {
        let Some(window) = self.window_mut(id) else {
            return false;
        };

        window.interaction.set_text_preedit(target, preedit)
    }

    pub fn reset_text_caret_blink(
        &mut self,
        id: app_window::Id,
        target: interaction::Target,
        now: Instant,
    ) -> bool {
        let Some(window) = self.window_mut(id) else {
            return false;
        };

        window.interaction.reset_text_caret_blink(target, now)
    }

    pub(crate) fn edit_text_draft(
        &mut self,
        id: app_window::Id,
        focus: Focus,
        base: impl Into<String>,
        edit: text::Edit,
        input: text::Input,
    ) -> Option<draft::Change> {
        let window = self.window_mut(id)?;
        if !window
            .focus
            .as_ref()
            .is_some_and(|current| current.same_target(&focus))
        {
            return None;
        }
        let target = focus.text_target()?;

        Some(
            window
                .interaction
                .edit_text_draft(target, base, edit, input),
        )
    }

    pub(crate) fn select_text_draft(
        &mut self,
        id: app_window::Id,
        focus: Focus,
        base: impl Into<String>,
        operation: text::selection::Operation,
    ) -> Option<draft::Change> {
        let window = self.window_mut(id)?;
        if !window
            .focus
            .as_ref()
            .is_some_and(|current| current.same_target(&focus))
        {
            return None;
        }
        let target = focus.text_target()?;

        Some(
            window
                .interaction
                .select_text_draft(target, base, operation),
        )
    }

    pub(crate) fn undo_text_draft(
        &mut self,
        id: app_window::Id,
        focus: Focus,
    ) -> Option<draft::Change> {
        let window = self.window_mut(id)?;
        if !window
            .focus
            .as_ref()
            .is_some_and(|current| current.same_target(&focus))
        {
            return None;
        }
        let target = focus.text_target()?;

        window.interaction.undo_text_draft(&target)
    }

    pub(crate) fn redo_text_draft(
        &mut self,
        id: app_window::Id,
        focus: Focus,
    ) -> Option<draft::Change> {
        let window = self.window_mut(id)?;
        if !window
            .focus
            .as_ref()
            .is_some_and(|current| current.same_target(&focus))
        {
            return None;
        }
        let target = focus.text_target()?;

        window.interaction.redo_text_draft(&target)
    }

    pub fn seal_text_draft(&mut self, id: app_window::Id, focus: Focus) -> bool {
        let Some(target) = focus.text_target() else {
            return false;
        };
        let Some(window) = self.window_mut(id) else {
            return false;
        };

        window.interaction.seal_text_draft(&target)
    }

    pub fn text_input_feedback(
        &self,
        id: app_window::Id,
        focus: Focus,
    ) -> Option<(feedback::Severity, &str)> {
        let target = focus.text_target()?;
        self.window(id)?
            .interaction
            .text_input()
            .feedback_for(&target)
    }

    pub(crate) fn reject_text_input(
        &mut self,
        id: app_window::Id,
        focus: Focus,
        reason: String,
    ) -> bool {
        let Some(target) = focus.text_target() else {
            return false;
        };
        self.window_mut(id).is_some_and(|window| {
            window
                .interaction
                .report_text_feedback(&target, feedback::Severity::Error, reason)
        })
    }

    pub fn clear_text_input(&mut self, id: app_window::Id) -> bool {
        let Some(window) = self.window_mut(id) else {
            return false;
        };

        window.interaction.clear_text_input()
    }

    pub fn clear_text_draft(&mut self, id: app_window::Id, focus: Focus) -> bool {
        let Some(target) = focus.text_target() else {
            return false;
        };
        let Some(window) = self.window_mut(id) else {
            return false;
        };

        window.interaction.clear_text_draft(&target)
    }

    pub fn deactivate_text_draft(&mut self, id: app_window::Id, focus: Focus) -> bool {
        let Some(target) = focus.text_target() else {
            return false;
        };
        let Some(window) = self.window_mut(id) else {
            return false;
        };

        window.interaction.deactivate_text_input(&target)
    }
}
