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

        window
            .interaction
            .activate_text_draft(interaction::Target::text_area(focus), base)
    }

    pub fn set_text_preedit(
        &mut self,
        id: app_window::Id,
        preedit: text::view::Preedit,
    ) -> Option<bool> {
        let window = self.window_mut(id)?;
        let target = interaction::Target::text_area(window.focus?);

        Some(window.interaction.set_text_preedit(target, preedit))
    }

    pub fn set_text_preedit_for(
        &mut self,
        id: app_window::Id,
        target: interaction::Target,
        preedit: text::view::Preedit,
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
        edit: text::edit::Edit,
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

        Some(window.interaction.edit_text_draft(
            interaction::Target::text_area(focus),
            base,
            edit,
            input,
        ))
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

        window
            .interaction
            .undo_text_draft(&interaction::Target::text_area(focus))
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

        window
            .interaction
            .redo_text_draft(&interaction::Target::text_area(focus))
    }

    pub fn seal_text_draft(&mut self, id: app_window::Id, focus: Focus) -> bool {
        let Some(window) = self.window_mut(id) else {
            return false;
        };

        window
            .interaction
            .seal_text_draft(&interaction::Target::text_area(focus))
    }

    pub fn text_input_feedback(
        &self,
        id: app_window::Id,
        focus: Focus,
    ) -> Option<(feedback::Severity, &str)> {
        self.window(id)?
            .interaction
            .text_input()
            .feedback_for(&interaction::Target::text_area(focus))
    }

    pub(crate) fn reject_text_input(
        &mut self,
        id: app_window::Id,
        focus: Focus,
        reason: String,
    ) -> bool {
        self.window_mut(id).is_some_and(|window| {
            window.interaction.report_text_feedback(
                &interaction::Target::text_area(focus),
                feedback::Severity::Error,
                reason,
            )
        })
    }

    pub fn clear_text_input(&mut self, id: app_window::Id) -> bool {
        let Some(window) = self.window_mut(id) else {
            return false;
        };

        window.interaction.clear_text_input()
    }

    pub fn clear_text_draft(&mut self, id: app_window::Id, focus: Focus) -> bool {
        let Some(window) = self.window_mut(id) else {
            return false;
        };

        window
            .interaction
            .clear_text_draft(&interaction::Target::text_area(focus))
    }

    pub fn deactivate_text_draft(&mut self, id: app_window::Id, focus: Focus) -> bool {
        let Some(window) = self.window_mut(id) else {
            return false;
        };

        window
            .interaction
            .deactivate_text_input(&interaction::Target::text_area(focus))
    }
}
