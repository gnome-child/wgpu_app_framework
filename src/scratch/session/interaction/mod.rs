mod menu;
mod pointer;
mod scroll;
mod text;

use crate::scratch::window as app_window;

use super::{Session, Window};

impl Session {
    pub fn interaction(
        &self,
        id: app_window::Id,
    ) -> Option<&crate::scratch::interaction::Interaction> {
        self.window(id).map(Window::interaction)
    }
}
