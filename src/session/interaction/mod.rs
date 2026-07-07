mod command_palette;
mod menu;
mod pointer;
mod scroll;
mod text;

use crate::window as app_window;

use super::{Session, Window};

impl Session {
    pub(crate) fn interaction(
        &self,
        id: app_window::Id,
    ) -> Option<&crate::interaction::Interaction> {
        self.window(id).map(Window::interaction)
    }
}
