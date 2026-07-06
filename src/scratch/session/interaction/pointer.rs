use crate::scratch::{interaction, window as app_window};

use super::{super::Session, menu};

impl Session {
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

    pub fn pointer_down(&mut self, id: app_window::Id, target: interaction::Target) -> bool {
        let Some(window) = self.window_mut(id) else {
            return false;
        };

        let dismissed_menu = menu::dismiss_menu_for_target(window, Some(&target));
        window.interaction.pointer_down(target) || dismissed_menu
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
}
