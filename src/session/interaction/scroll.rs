use crate::{interaction, window as app_window};

use super::super::Session;

impl Session {
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

    pub fn reveal_active_descendant(
        &mut self,
        id: app_window::Id,
        viewport: interaction::Target,
    ) -> bool {
        let Some(window) = self.window_mut(id) else {
            return false;
        };

        window.interaction.reveal_active_descendant(viewport)
    }

    pub fn clear_scroll_reveal(
        &mut self,
        id: app_window::Id,
        target: &interaction::Target,
    ) -> bool {
        let Some(window) = self.window_mut(id) else {
            return false;
        };

        window.interaction.clear_scroll_reveal(target)
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
}
