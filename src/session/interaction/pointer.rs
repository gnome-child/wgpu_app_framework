use crate::{interaction, window as app_window};

use super::super::Session;

impl Session {
    pub(crate) fn hover_tip_visible(&self, id: app_window::Id) -> bool {
        self.interaction(id)
            .is_some_and(|interaction| interaction.pointer().hover_tip_visible())
    }

    pub(crate) fn hover_tip_deadline(
        &self,
        id: app_window::Id,
        delay: std::time::Duration,
    ) -> Option<std::time::Instant> {
        self.interaction(id)?.pointer().hover_tip_deadline(delay)
    }

    pub(crate) fn promote_hover_tip(
        &mut self,
        id: app_window::Id,
        now: std::time::Instant,
        delay: std::time::Duration,
    ) -> bool {
        self.window_mut(id)
            .is_some_and(|window| window.interaction.promote_hover_tip(now, delay))
    }

    pub(crate) fn set_pointer_position(
        &mut self,
        id: app_window::Id,
        position: Option<crate::geometry::Point>,
        surface: crate::popup::Surface,
    ) -> bool {
        self.window_mut(id)
            .is_some_and(|window| window.interaction.set_pointer_position(position, surface))
    }

    pub(crate) fn project_pointer_hover(
        &mut self,
        id: app_window::Id,
        target: Option<interaction::Target>,
        tip_eligible: bool,
    ) -> bool {
        self.window_mut(id).is_some_and(|window| {
            window
                .interaction
                .project_pointer_hover(target, tip_eligible)
        })
    }

    pub(crate) fn classify_click(
        &mut self,
        id: app_window::Id,
        target: &interaction::Target,
        point: crate::geometry::Point,
        at: std::time::Instant,
    ) -> interaction::ClickCount {
        self.window_mut(id)
            .map(|window| window.interaction.classify_click(target, point, at))
            .unwrap_or(interaction::ClickCount::Single)
    }

    pub(crate) fn cancel_click_sequence(&mut self, id: app_window::Id) -> bool {
        self.window_mut(id)
            .is_some_and(|window| window.interaction.cancel_click_sequence())
    }

    pub fn pointer_move(
        &mut self,
        id: app_window::Id,
        target: Option<interaction::Target>,
    ) -> bool {
        let Some(window) = self.window_mut(id) else {
            return false;
        };

        let open_menu = target
            .as_ref()
            .and_then(interaction::Target::as_menu)
            .filter(|menu| {
                window
                    .interaction
                    .open_menu()
                    .is_some_and(|open| open != menu)
            })
            .map(|menu| window.interaction.open_menu_with(menu))
            .unwrap_or(false);

        window.interaction.pointer_move(target) || open_menu
    }

    pub(crate) fn pointer_down(
        &mut self,
        id: app_window::Id,
        target: interaction::Target,
        intent: interaction::PressIntent,
    ) -> bool {
        let Some(window) = self.window_mut(id) else {
            return false;
        };

        window.interaction.pointer_down(target, intent)
    }

    pub fn pointer_up(&mut self, id: app_window::Id, target: Option<interaction::Target>) -> bool {
        let Some(window) = self.window_mut(id) else {
            return false;
        };

        window.interaction.pointer_up(target)
    }

    pub(crate) fn set_pointer_press_intent(
        &mut self,
        id: app_window::Id,
        target: &interaction::Target,
        intent: interaction::PressIntent,
    ) -> bool {
        let Some(window) = self.window_mut(id) else {
            return false;
        };

        window.interaction.set_pointer_press_intent(target, intent)
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
}
