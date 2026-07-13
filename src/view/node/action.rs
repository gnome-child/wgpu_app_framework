#[cfg(test)]
use crate::text;

use super::super::{
    action::Action,
    control::{TextArea, TextBox},
};
use super::{Node, Role};
use crate::{composition, interaction};

impl Node {
    pub(crate) fn pointer_target(&self) -> Option<interaction::Target> {
        self.explicit_pointer_target()
    }

    pub(crate) fn node_pointer_target(
        &self,
        node_id: composition::NodeId,
    ) -> Option<interaction::Target> {
        if self.role() == Role::FloatingPanel && !self.panel_policy().accepts_input() {
            return None;
        }

        if let Some(target) = self.text_control_target() {
            return Some(target);
        }

        if let Some(binding) = &self.binding {
            let target = self.id.map_or_else(
                || {
                    interaction::Target::command_node(
                        node_id,
                        None,
                        binding.command_name(),
                        binding.source(),
                    )
                },
                |id| binding.element_pointer_target(id),
            );

            return Some(if self.role == Role::Slider {
                target.with_capture()
            } else {
                target
            });
        }

        if matches!(self.role, Role::Scroll | Role::VirtualList) {
            let label = self.label.as_deref().unwrap_or("Scroll");
            return Some(self.id.map_or_else(
                || interaction::Target::scroll_node(node_id, None, label),
                |id| interaction::Target::scroll(id, label),
            ));
        }

        self.pointer_target()
    }

    fn explicit_pointer_target(&self) -> Option<interaction::Target> {
        if let Some(target) = self.text_control_target() {
            return Some(target);
        }

        if let Some(binding) = &self.binding {
            let target = self.id.map(|id| binding.element_pointer_target(id))?;

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
            Role::FloatingPanel => self.id.map(|id| {
                interaction::Target::floating_panel(
                    id,
                    self.label.as_deref().unwrap_or_else(|| id.as_str()),
                )
            }),
            Role::Label => self
                .id
                .zip(self.label.as_ref())
                .map(|(id, label)| interaction::Target::label(id, label.clone())),
            Role::Root
            | Role::Stack
            | Role::Table
            | Role::MenuBar
            | Role::Binding
            | Role::Separator
            | Role::Button
            | Role::Checkbox
            | Role::Radio
            | Role::Slider
            | Role::Scroll
            | Role::VirtualList
            | Role::Panel
            | Role::SectionHeader => None,
        }
    }

    pub(in crate::view) fn text_control_target(&self) -> Option<interaction::Target> {
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

    #[cfg(test)]
    pub(crate) fn pointer_move_action(&self) -> Option<Action> {
        Some(Action::pointer_move(Some(self.pointer_target()?)))
    }

    #[cfg(test)]
    pub(crate) fn pointer_down_action(&self) -> Option<Action> {
        Some(Action::pointer_down(self.pointer_target()?))
    }

    #[cfg(test)]
    pub(crate) fn pointer_up_action(&self) -> Option<Action> {
        Some(Action::pointer_up(
            Some(self.pointer_target()?),
            self.pointer_activation_action(),
        ))
    }

    #[cfg(test)]
    pub(crate) fn scroll_action(&self, delta: interaction::ScrollDelta) -> Option<Action> {
        Some(Action::scroll(self.pointer_target()?, delta))
    }

    #[cfg(test)]
    pub(crate) fn text_pointer_down_action(
        &self,
        position: text::buffer::Position,
    ) -> Option<Action> {
        if self.role != Role::TextArea {
            return None;
        }

        let text_area = self.text_area_model()?;
        let target = self.pointer_target()?;

        Some(Action::sequence([
            Action::pointer_down(target),
            Action::text_pointer_focus(text_area.focus())?,
            Action::text_edit(text::edit::Edit::pointer(
                text::edit::PointerEditKind::Click,
                position,
            )),
        ]))
    }

    #[cfg(test)]
    pub(crate) fn text_pointer_drag_action(
        &self,
        position: text::buffer::Position,
    ) -> Option<Action> {
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

    pub(crate) fn menu_action(&self) -> Option<Action> {
        if self.role != Role::Menu {
            return None;
        }

        Some(Action::toggle_menu(interaction::Menu::new(
            self.id?,
            self.label.as_deref()?,
        )))
    }

    #[cfg(test)]
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
            | Role::Table
            | Role::MenuBar
            | Role::Binding
            | Role::Separator
            | Role::TextArea
            | Role::Button
            | Role::Checkbox
            | Role::Radio
            | Role::Slider
            | Role::Scroll
            | Role::VirtualList
            | Role::Panel
            | Role::FloatingPanel
            | Role::TextBox
            | Role::SectionHeader
            | Role::Label => None,
        }
    }

    pub(in crate::view) fn keyboard_activation_action(&self) -> Option<Action> {
        match self.role {
            Role::Menu => self.menu_action(),
            Role::Binding | Role::Button | Role::Checkbox | Role::Radio | Role::Slider => self
                .binding
                .as_ref()
                .and_then(|binding| binding.is_enabled().then(|| Action::activate(binding))),
            Role::Root
            | Role::Stack
            | Role::Table
            | Role::MenuBar
            | Role::Separator
            | Role::TextArea
            | Role::TextBox
            | Role::Scroll
            | Role::VirtualList
            | Role::Panel
            | Role::FloatingPanel
            | Role::SectionHeader
            | Role::Label => None,
        }
    }
}
