use super::super::{interaction, window as app_window};
use super::{Session, Window};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Focus {
    target: interaction::Id,
}

impl Focus {
    pub fn text(target: impl Into<interaction::Id>) -> Self {
        Self {
            target: target.into(),
        }
    }

    pub fn target(self) -> interaction::Id {
        self.target
    }
}

impl Session {
    pub fn focus(&mut self, id: app_window::Id, focus: Focus) -> bool {
        let Some(window) = self.window_mut(id) else {
            return false;
        };
        let target = interaction::Target::text_area(focus);
        let changed = window.focus != Some(focus);
        let input_changed = window.interaction.clear_text_input_unless(&target);
        window.focus = Some(focus);
        changed || input_changed
    }

    pub fn clear_focus(&mut self, id: app_window::Id) -> bool {
        let Some(window) = self.window_mut(id) else {
            return false;
        };
        let changed = window.focus.is_some();
        let input_changed = window.interaction.clear_text_input();
        window.focus = None;
        changed || input_changed
    }

    pub fn focused(&self, id: app_window::Id) -> Option<Focus> {
        self.window(id).and_then(Window::focus)
    }
}
