use crate::{interaction, window as app_window};

use super::super::Session;

impl Session {
    pub(crate) fn apply_scroll(
        &mut self,
        id: app_window::Id,
        target: interaction::Target,
        update: interaction::ScrollUpdate,
    ) -> Option<interaction::ScrollOffset> {
        let Some(window) = self.window_mut(id) else {
            return None;
        };

        let changed = window.interaction.apply_scroll(target.clone(), update);
        let reveal_cleared = !matches!(update, interaction::ScrollUpdate::Relative(_))
            && window.interaction.clear_scroll_reveal(&target);
        changed.or_else(|| reveal_cleared.then(|| window.interaction.scroll().offset(&target)))
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
}
