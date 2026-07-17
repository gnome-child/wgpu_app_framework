use crate::{interaction, window as app_window};

use super::super::Session;

impl Session {
    pub(crate) fn handle_scroll_session(
        &mut self,
        id: app_window::Id,
        target: &interaction::Target,
        event: interaction::ScrollEvent,
    ) -> interaction::ScrollSessionDisposition {
        let Some(window) = self.window_mut(id) else {
            return interaction::ScrollSessionDisposition::Ignored;
        };
        window.interaction.handle_scroll_session(target, event)
    }

    pub(crate) fn resolve_scroll_edge(
        &mut self,
        id: app_window::Id,
        target: &interaction::Target,
        outcome: interaction::ScrollOutcome,
    ) -> interaction::ScrollOutcome {
        let Some(window) = self.window_mut(id) else {
            return outcome;
        };
        window.interaction.resolve_scroll_edge(target, outcome)
    }

    pub(crate) fn configure_scroll(
        &mut self,
        id: app_window::Id,
        target: interaction::Target,
        maximum: interaction::ScrollOffset,
        page: interaction::ScrollOffset,
    ) -> Option<interaction::ScrollOffset> {
        self.window_mut(id)?
            .interaction
            .configure_scroll(target, maximum, page)
    }

    pub(crate) fn request_scroll(
        &mut self,
        id: app_window::Id,
        target: interaction::Target,
        update: interaction::ScrollUpdate,
    ) -> Option<interaction::ScrollOffset> {
        let Some(window) = self.window_mut(id) else {
            return None;
        };

        let changed = window.interaction.request_scroll(target.clone(), update);
        let reveal_cleared = !matches!(update, interaction::ScrollUpdate::Relative(_))
            && window.interaction.clear_scroll_reveal(&target);
        changed
            .or_else(|| reveal_cleared.then(|| window.interaction.scroll().desired_offset(&target)))
    }

    pub(crate) fn scroll_operation_offset(
        &self,
        id: app_window::Id,
        target: &interaction::Target,
        axis: interaction::ScrollbarAxis,
        operation: interaction::ScrollOperation,
        reversed: bool,
    ) -> Option<interaction::ScrollOffset> {
        self.window(id)?
            .interaction
            .scroll()
            .operation_offset(target, axis, operation, reversed)
    }

    pub(crate) fn accessible_scroll_axis(
        &self,
        id: app_window::Id,
        target: &interaction::Target,
        axis: interaction::ScrollbarAxis,
    ) -> Option<interaction::AccessibleScrollAxis> {
        self.window(id)?
            .interaction
            .scroll()
            .accessible_axis(target, axis)
    }

    pub(crate) fn accept_resident_scroll(
        &mut self,
        id: app_window::Id,
        target: interaction::Target,
        offset: interaction::ScrollOffset,
    ) -> Option<interaction::ScrollOffset> {
        let window = self.window_mut(id)?;
        window.interaction.accept_resident_scroll(target, offset)
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
