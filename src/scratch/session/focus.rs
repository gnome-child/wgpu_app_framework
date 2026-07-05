use super::super::{interaction, window as app_window};
use super::{Session, Window};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Focus {
    kind: Kind,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum Kind {
    Text(interaction::Id),
    Control(u64),
}

impl Focus {
    pub fn text(target: impl Into<interaction::Id>) -> Self {
        Self {
            kind: Kind::Text(target.into()),
        }
    }

    pub fn control(target: &interaction::Target) -> Self {
        Self {
            kind: Kind::Control(target.focus_key()),
        }
    }

    pub fn target(self) -> interaction::Id {
        match self.kind {
            Kind::Text(target) => target,
            Kind::Control(_) => {
                panic!("control focus does not have a text target id")
            }
        }
    }

    pub fn into_target(self) -> interaction::Target {
        self.text_target()
            .expect("control focus does not have a text target")
    }

    pub fn target_id(&self) -> Option<interaction::Id> {
        match self.kind {
            Kind::Text(target) => Some(target),
            Kind::Control(_) => None,
        }
    }

    pub fn text_target(self) -> Option<interaction::Target> {
        match self.kind {
            Kind::Text(target) => Some(interaction::Target::text_area_id(target)),
            Kind::Control(_) => None,
        }
    }

    pub fn matches_target(self, target: &interaction::Target) -> bool {
        match self.kind {
            Kind::Text(id) => target.kind() == interaction::target::Kind::TextArea
                && target.element_id() == Some(id),
            Kind::Control(key) => target.focus_key() == key,
        }
    }
}

impl Session {
    pub fn focus(&mut self, id: app_window::Id, focus: Focus) -> bool {
        let Some(window) = self.window_mut(id) else {
            return false;
        };
        let changed = window.focus.as_ref() != Some(&focus);
        let input_changed = if let Some(target) = focus.text_target() {
            window.interaction.clear_text_input_unless(&target)
        } else {
            window.interaction.clear_text_input()
        };
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
