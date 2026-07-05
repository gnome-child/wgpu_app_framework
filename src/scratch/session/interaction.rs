use crate::text;

use super::super::{draft, interaction, window as app_window};
use super::{Focus, Session, Window};

impl Session {
    pub fn open_menu(&mut self, id: app_window::Id, menu: interaction::Menu) -> bool {
        let Some(window) = self.window_mut(id) else {
            return false;
        };

        window.interaction.open_menu_with(menu)
    }

    pub fn toggle_menu(&mut self, id: app_window::Id, menu: interaction::Menu) -> bool {
        let Some(window) = self.window_mut(id) else {
            return false;
        };

        window.interaction.toggle_menu(menu)
    }

    pub fn close_menu(&mut self, id: app_window::Id) -> bool {
        let Some(window) = self.window_mut(id) else {
            return false;
        };

        window.interaction.close_menu()
    }

    pub fn pointer_move(
        &mut self,
        id: app_window::Id,
        target: Option<interaction::Target>,
    ) -> bool {
        let Some(window) = self.window_mut(id) else {
            return false;
        };

        window.interaction.pointer_move(target)
    }

    pub fn pointer_down(&mut self, id: app_window::Id, target: interaction::Target) -> bool {
        let Some(window) = self.window_mut(id) else {
            return false;
        };

        window.interaction.pointer_down(target)
    }

    pub fn pointer_up(&mut self, id: app_window::Id, target: Option<interaction::Target>) -> bool {
        let Some(window) = self.window_mut(id) else {
            return false;
        };

        window.interaction.pointer_up(target)
    }

    pub fn pointer_left(&mut self, id: app_window::Id) -> bool {
        let Some(window) = self.window_mut(id) else {
            return false;
        };

        window.interaction.pointer_left()
    }

    pub fn cancel_pointer(&mut self, id: app_window::Id) -> bool {
        let Some(window) = self.window_mut(id) else {
            return false;
        };

        window.interaction.cancel_pointer()
    }

    pub fn scroll_by(
        &mut self,
        id: app_window::Id,
        target: interaction::Target,
        delta: interaction::ScrollDelta,
    ) -> bool {
        let Some(window) = self.window_mut(id) else {
            return false;
        };

        window.interaction.scroll_by(target, delta)
    }

    pub fn reveal_scroll(&mut self, id: app_window::Id, target: interaction::Target) -> bool {
        let Some(window) = self.window_mut(id) else {
            return false;
        };

        window.interaction.reveal_scroll(target)
    }

    pub fn resolve_scroll(
        &mut self,
        id: app_window::Id,
        target: interaction::Target,
        offset: interaction::ScrollOffset,
    ) -> bool {
        let Some(window) = self.window_mut(id) else {
            return false;
        };

        let scrolled = window.interaction.scroll_to(target.clone(), offset);
        let revealed = window.interaction.clear_scroll_reveal(&target);
        scrolled || revealed
    }

    pub fn set_text_preedit(
        &mut self,
        id: app_window::Id,
        preedit: text::edit::Preedit,
    ) -> Option<bool> {
        let window = self.window_mut(id)?;
        let target = interaction::Target::text_area(window.focus?);

        Some(window.interaction.set_text_preedit(target, preedit))
    }

    pub fn set_text_preedit_for(
        &mut self,
        id: app_window::Id,
        target: interaction::Target,
        preedit: text::edit::Preedit,
    ) -> bool {
        let Some(window) = self.window_mut(id) else {
            return false;
        };

        window.interaction.set_text_preedit(target, preedit)
    }

    pub fn edit_text_draft(
        &mut self,
        id: app_window::Id,
        focus: Focus,
        base: impl Into<String>,
        edit: text::edit::Edit,
    ) -> Option<draft::Change> {
        let window = self.window_mut(id)?;
        if window.focus != Some(focus) {
            return None;
        }

        Some(
            window
                .interaction
                .edit_text_draft(interaction::Target::text_area(focus), base, edit),
        )
    }

    pub fn undo_text_draft(
        &mut self,
        id: app_window::Id,
        focus: Focus,
    ) -> Option<draft::Change> {
        let window = self.window_mut(id)?;
        if window.focus != Some(focus) {
            return None;
        }

        window
            .interaction
            .undo_text_draft(&interaction::Target::text_area(focus))
    }

    pub fn redo_text_draft(
        &mut self,
        id: app_window::Id,
        focus: Focus,
    ) -> Option<draft::Change> {
        let window = self.window_mut(id)?;
        if window.focus != Some(focus) {
            return None;
        }

        window
            .interaction
            .redo_text_draft(&interaction::Target::text_area(focus))
    }

    pub fn clear_text_input(&mut self, id: app_window::Id) -> bool {
        let Some(window) = self.window_mut(id) else {
            return false;
        };

        window.interaction.clear_text_input()
    }

    pub fn interaction(&self, id: app_window::Id) -> Option<&interaction::Interaction> {
        self.window(id).map(Window::interaction)
    }
}
