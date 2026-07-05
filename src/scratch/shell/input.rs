use std::path::PathBuf;

use crate::scratch::{
    Error, geometry, input, interaction, state::State, view, window as app_window,
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

    pub fn pointer_down(
        &mut self,
        window: app_window::Id,
        point: geometry::Point,
    ) -> Result<input::Outcome, Error> {
        let Some(size) = self.window_size(window) else {
            return Ok(input::Outcome::ignored());
        };

        self.runtime.pointer_down_at(window, size, point)
    }

    pub fn pointer_up(
        &mut self,
        window: app_window::Id,
        point: geometry::Point,
    ) -> Result<input::Outcome, Error> {
        let Some(size) = self.window_size(window) else {
            return Ok(input::Outcome::ignored());
        };

        self.runtime.pointer_up_at(window, size, point)
    }

    pub fn pointer_left(&mut self, window: app_window::Id) -> Result<input::Outcome, Error> {
        self.runtime
            .handle_view(window, view::Action::pointer_left())
    }

    pub fn scroll(
        &mut self,
        window: app_window::Id,
        point: geometry::Point,
        delta: interaction::ScrollDelta,
    ) -> Result<input::Outcome, Error> {
        let Some(size) = self.window_size(window) else {
            return Ok(input::Outcome::ignored());
        };

        self.runtime.scroll_at(window, size, point, delta)
    }
}
