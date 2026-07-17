use crate::{interaction, window as app_window};

use super::super::{Session, Window};

impl Session {
    pub fn open_menu(&mut self, id: app_window::Id, menu: interaction::Menu) -> bool {
        let closed_palette = self.close_command_palette(id);
        let Some(window) = self.window_mut(id) else {
            return false;
        };

        if window.interaction.open_menu().is_none() {
            window.menu_restore_focus = window.command_focus();
        }

        window.interaction.open_menu_with(menu) || closed_palette
    }

    pub fn toggle_menu(&mut self, id: app_window::Id, menu: interaction::Menu) -> bool {
        let closed_palette = self.close_command_palette(id);
        let Some(window) = self.window_mut(id) else {
            return false;
        };

        if window.interaction.open_menu() == Some(&menu) {
            let closed = window.interaction.close_menu();
            let restored = restore_menu_focus(window);
            return closed || restored || closed_palette;
        }

        if window.interaction.open_menu().is_none() {
            window.menu_restore_focus = window.command_focus();
        }

        window.interaction.toggle_menu(menu) || closed_palette
    }

    pub fn close_menu(&mut self, id: app_window::Id) -> bool {
        let Some(window) = self.window_mut(id) else {
            return false;
        };

        let closed = window.interaction.close_menu();
        let restored = restore_menu_focus(window);
        closed || restored
    }

    pub fn dismiss_menu_for_target(
        &mut self,
        id: app_window::Id,
        target: Option<&interaction::Target>,
    ) -> bool {
        let Some(window) = self.window_mut(id) else {
            return false;
        };

        dismiss_menu_for_target(window, target)
    }

    pub(crate) fn dismiss_menu_for_surface(
        &mut self,
        id: app_window::Id,
        inside_surface: bool,
    ) -> bool {
        let Some(window) = self.window_mut(id) else {
            return false;
        };

        dismiss_menu_for_surface(window, inside_surface)
    }
}

pub(super) fn dismiss_menu_for_target(
    window: &mut Window,
    target: Option<&interaction::Target>,
) -> bool {
    dismiss_menu_for_surface(
        window,
        target.is_some_and(interaction::Target::is_menu_surface),
    )
}

fn dismiss_menu_for_surface(window: &mut Window, inside_surface: bool) -> bool {
    if window.interaction.open_menu().is_none() {
        return false;
    }

    if inside_surface {
        return false;
    }

    let closed = window.interaction.close_menu();
    if closed {
        window.menu_restore_focus = None;
    }

    closed
}

fn restore_menu_focus(window: &mut Window) -> bool {
    let Some(focus) = window.menu_restore_focus.take() else {
        return false;
    };

    let changed = window.focus.as_ref() != Some(&focus);
    let input_changed = window.interaction.clear_text_preedit();
    if changed {
        window.focus_reveal_pending = focus.is_visible();
    }
    window.focus = Some(focus);

    changed || input_changed
}
