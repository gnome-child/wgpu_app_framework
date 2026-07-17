use std::path::PathBuf;

use crate::{
    command::Error, geometry, input, interaction, pointer, state::State, window as app_window,
};

use super::Shell;

impl<M: State, E: Send + 'static> Shell<M, E> {
    pub fn handle_input(
        &mut self,
        window: app_window::Id,
        input: input::Input,
    ) -> Result<input::Outcome, Error> {
        self.runtime.handle_input(window, input)
    }

    pub fn file_path_selected(
        &mut self,
        window: app_window::Id,
        path: Option<PathBuf>,
    ) -> Result<input::Outcome, Error> {
        self.handle_input(window, input::Input::file_path_selected(path))
    }

    pub fn pointer_move(
        &mut self,
        window: app_window::Id,
        point: geometry::Point,
    ) -> Result<input::Outcome, Error> {
        let Some(size) = self.window_size(window) else {
            return Ok(input::Outcome::ignored());
        };

        self.runtime.pointer_move_at(window, size, point)
    }

    pub(crate) fn pointer_move_on_popup(
        &mut self,
        window: app_window::Id,
        popup: interaction::Id,
        point: geometry::Point,
    ) -> Result<input::Outcome, Error> {
        let Some(size) = self.window_size(window) else {
            return Ok(input::Outcome::ignored());
        };

        self.runtime.pointer_move_on_surface(
            window,
            size,
            point,
            crate::popup::Surface::Native(popup),
        )
    }

    pub fn pointer_down(
        &mut self,
        window: app_window::Id,
        point: geometry::Point,
        button: pointer::Button,
    ) -> Result<input::Outcome, Error> {
        if button != pointer::Button::Primary {
            return Ok(input::Outcome::ignored());
        }

        let Some(size) = self.window_size(window) else {
            return Ok(input::Outcome::ignored());
        };

        self.runtime.pointer_down_at(window, size, point)
    }

    pub fn pointer_down_with_modifiers(
        &mut self,
        window: app_window::Id,
        point: geometry::Point,
        button: pointer::Button,
        modifiers: input::Modifiers,
    ) -> Result<input::Outcome, Error> {
        if button != pointer::Button::Primary {
            return Ok(input::Outcome::ignored());
        }

        let Some(size) = self.window_size(window) else {
            return Ok(input::Outcome::ignored());
        };

        self.runtime
            .pointer_down_at_with_modifiers(window, size, point, modifiers)
    }

    pub(crate) fn pointer_down_on_popup(
        &mut self,
        window: app_window::Id,
        popup: interaction::Id,
        point: geometry::Point,
        button: pointer::Button,
        modifiers: input::Modifiers,
    ) -> Result<input::Outcome, Error> {
        if button != pointer::Button::Primary {
            return Ok(input::Outcome::ignored());
        }
        let Some(size) = self.window_size(window) else {
            return Ok(input::Outcome::ignored());
        };

        self.runtime.pointer_down_on_surface(
            window,
            size,
            point,
            modifiers,
            crate::popup::Surface::Native(popup),
        )
    }

    pub fn pointer_up(
        &mut self,
        window: app_window::Id,
        point: geometry::Point,
        button: pointer::Button,
    ) -> Result<input::Outcome, Error> {
        let Some(size) = self.window_size(window) else {
            return Ok(input::Outcome::ignored());
        };

        if button == pointer::Button::Secondary {
            return self.runtime.open_context_menu_at(window, size, point);
        }
        if button != pointer::Button::Primary {
            return Ok(input::Outcome::ignored());
        }

        self.runtime.pointer_up_at(window, size, point)
    }

    pub(crate) fn pointer_up_on_popup(
        &mut self,
        window: app_window::Id,
        popup: interaction::Id,
        point: geometry::Point,
        button: pointer::Button,
    ) -> Result<input::Outcome, Error> {
        let Some(size) = self.window_size(window) else {
            return Ok(input::Outcome::ignored());
        };
        let surface = crate::popup::Surface::Native(popup);

        if button == pointer::Button::Secondary {
            return self
                .runtime
                .open_context_menu_on_surface(window, size, point, surface);
        }
        if button != pointer::Button::Primary {
            return Ok(input::Outcome::ignored());
        }

        self.runtime
            .pointer_up_on_surface(window, size, point, surface)
    }

    pub fn pointer_left(&mut self, window: app_window::Id) -> Result<input::Outcome, Error> {
        self.runtime.pointer_left_at(window)
    }

    pub(crate) fn pointer_left_popup(
        &mut self,
        window: app_window::Id,
        _popup: interaction::Id,
    ) -> Result<input::Outcome, Error> {
        self.runtime.pointer_left_at(window)
    }

    pub(crate) fn pointer_modifiers_changed(
        &mut self,
        window: app_window::Id,
        modifiers: input::Modifiers,
    ) -> Result<input::Outcome, Error> {
        self.runtime.pointer_modifiers_changed(window, modifiers)
    }

    pub fn scroll(
        &mut self,
        window: app_window::Id,
        point: geometry::Point,
        delta: interaction::Delta,
    ) -> Result<input::Outcome, Error> {
        let Some(size) = self.window_size(window) else {
            return Ok(input::Outcome::ignored());
        };

        self.runtime.scroll_at(window, size, point, delta)
    }

    pub(crate) fn scroll_popup(
        &mut self,
        window: app_window::Id,
        popup: interaction::Id,
        point: geometry::Point,
        delta: interaction::Delta,
    ) -> Result<input::Outcome, Error> {
        let Some(size) = self.window_size(window) else {
            return Ok(input::Outcome::ignored());
        };

        self.runtime.scroll_on_surface(
            window,
            size,
            point,
            delta,
            crate::popup::Surface::Native(popup),
        )
    }
}
