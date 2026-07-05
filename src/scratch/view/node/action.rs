use crate::text;

use super::super::{
    action::Action,
    control::{TextArea, TextBox},
};
use super::{Node, Role};
use crate::scratch::interaction;

impl Node {
    pub fn pointer_target(&self) -> Option<interaction::Target> {
        self.pointer_target_with_path(None)
    }

    pub fn pointer_target_at_path(&self, path: &[usize]) -> Option<interaction::Target> {
        self.pointer_target_with_path(Some(path))
    }

    fn pointer_target_with_path(&self, path: Option<&[usize]>) -> Option<interaction::Target> {
        if let Some(target) = self.text_control_target() {
            return Some(target);
        }

        if let Some(binding) = &self.binding {
            let target = self
                .id
                .map(|id| binding.element_pointer_target(id))
                .or_else(|| path.map(|path| binding.path_pointer_target(path)))?;

            return Some(if self.role == Role::Slider {
                target.with_capture()
            } else {
                target
            });
        }

        match self.role {
            Role::Menu => self
                .id
                .zip(self.label.as_ref())
                .map(|(id, label)| interaction::Target::menu(id, label.clone())),
            Role::TextArea | Role::TextBox => None,
            Role::Popup => self.id.map(|id| {
                interaction::Target::popup(id, self.label.as_deref().unwrap_or_else(|| id.as_str()))
            }),
            Role::Label => self
                .id
                .zip(self.label.as_ref())
                .map(|(id, label)| interaction::Target::label(id, label.clone())),
            Role::Root
            | Role::Stack
            | Role::MenuBar
            | Role::Binding
            | Role::Separator
            | Role::Button
            | Role::Checkbox
            | Role::Radio
            | Role::Slider
            | Role::Panel => None,
        }
    }

    pub(in crate::scratch::view) fn text_control_target(&self) -> Option<interaction::Target> {
        match self.role {
            Role::TextArea => self.id.map(interaction::Target::text_area_id).or_else(|| {
                self.text_area_model()
                    .and_then(TextArea::focus)
                    .map(interaction::Target::text_area)
            }),
            Role::TextBox => self
                .text_box_model()
                .and_then(TextBox::focus)
                .map(interaction::Target::text_area),
            _ => None,
        }
    }

    pub fn label_target(&self) -> Option<interaction::Target> {
        if self.role != Role::Label {
            return None;
        }

        self.id
            .zip(self.label.as_ref())
            .map(|(id, label)| interaction::Target::label(id, label.clone()))
    }

    pub fn pointer_move_action(&self) -> Option<Action> {
        Some(Action::pointer_move(Some(self.pointer_target()?)))
    }

    pub fn pointer_down_action(&self) -> Option<Action> {
        Some(Action::pointer_down(self.pointer_target()?))
    }

    pub fn pointer_up_action(&self) -> Option<Action> {
        Some(Action::pointer_up(
            Some(self.pointer_target()?),
            self.pointer_activation_action(),
        ))
    }

    pub fn scroll_action(&self, delta: interaction::ScrollDelta) -> Option<Action> {
        Some(Action::scroll(self.pointer_target()?, delta))
    }

    pub fn text_pointer_down_action(&self, position: text::buffer::Position) -> Option<Action> {
        if self.role != Role::TextArea {
            return None;
        }

        let text_area = self.text_area_model()?;
        let target = self.pointer_target()?;

        Some(Action::sequence([
            Action::pointer_down(target),
            text_area.pointer_focus_action()?,
            Action::text_edit(text::edit::Edit::pointer(
                text::edit::PointerEditKind::Click,
                position,
            )),
        ]))
    }

    pub fn text_pointer_drag_action(&self, position: text::buffer::Position) -> Option<Action> {
        if self.role != Role::TextArea {
            return None;
        }

        let target = self.pointer_target()?;

        Some(Action::pointer_drag(
            Some(target.clone()),
            target,
            Some(Action::text_edit(text::edit::Edit::pointer(
                text::edit::PointerEditKind::Drag,
                position,
            ))),
        ))
    }

    pub fn menu_action(&self) -> Option<Action> {
        if self.role != Role::Menu {
            return None;
        }

        Some(Action::toggle_menu(interaction::Menu::new(
            self.id?,
            self.label.as_deref()?,
        )))
    }

    fn pointer_activation_action(&self) -> Option<Action> {
        match self.role {
            Role::TextArea => return self.text_area_model().and_then(TextArea::focus_action),
            Role::TextBox => return self.text_box_model().and_then(TextBox::focus_action),
            _ => {}
        }

        if let Some(binding) = &self.binding {
            return binding.is_enabled().then(|| Action::activate(binding));
        }

        match self.role {
            Role::Menu => self.menu_action(),
            Role::Root
            | Role::Stack
            | Role::MenuBar
            | Role::Binding
            | Role::Separator
            | Role::TextArea
            | Role::Button
            | Role::Checkbox
            | Role::Radio
            | Role::Slider
            | Role::Panel
            | Role::TextBox
            | Role::Popup
            | Role::Label => None,
        }
    }

    pub(in crate::scratch::view) fn keyboard_activation_action(&self) -> Option<Action> {
        match self.role {
            Role::Menu => self.menu_action(),
            Role::Binding | Role::Button | Role::Checkbox | Role::Radio | Role::Slider => self
                .binding
                .as_ref()
                .and_then(|binding| binding.is_enabled().then(|| Action::activate(binding))),
            Role::Root
            | Role::Stack
            | Role::MenuBar
            | Role::Separator
            | Role::TextArea
            | Role::TextBox
            | Role::Panel
            | Role::Popup
            | Role::Label => None,
        }
    }
}
