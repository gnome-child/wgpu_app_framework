use super::super::{
    action::Action,
    binding::Binding,
    control::{Button, Checkbox, Control, Radio, Slider, TextArea, TextBox},
};
use super::{Node, Role};
use crate::scratch::{
    command as framework_command, context::Context as CommandContext, interaction, responder,
    session, state,
};

impl Node {
    pub(in crate::scratch::view) fn collect_bindings<'a>(
        &'a self,
        bindings: &mut Vec<&'a Binding>,
    ) {
        if let Some(binding) = &self.binding {
            bindings.push(binding);
        }

        for child in &self.children {
            child.collect_bindings(bindings);
        }
    }

    pub(in crate::scratch::view) fn collect_text_areas<'a>(
        &'a self,
        text_areas: &mut Vec<&'a TextArea>,
    ) {
        if let Some(text_area) = self.text_area_model() {
            text_areas.push(text_area);
        }

        for child in &self.children {
            child.collect_text_areas(text_areas);
        }
    }

    pub(in crate::scratch::view) fn collect_buttons<'a>(&'a self, buttons: &mut Vec<&'a Button>) {
        if let Some(button) = self.button_model() {
            buttons.push(button);
        }

        for child in &self.children {
            child.collect_buttons(buttons);
        }
    }

    pub(in crate::scratch::view) fn collect_checkboxes<'a>(
        &'a self,
        checkboxes: &mut Vec<&'a Checkbox>,
    ) {
        if let Some(checkbox) = self.checkbox_model() {
            checkboxes.push(checkbox);
        }

        for child in &self.children {
            child.collect_checkboxes(checkboxes);
        }
    }

    pub(in crate::scratch::view) fn collect_radios<'a>(&'a self, radios: &mut Vec<&'a Radio>) {
        if let Some(radio) = self.radio_model() {
            radios.push(radio);
        }

        for child in &self.children {
            child.collect_radios(radios);
        }
    }

    pub(in crate::scratch::view) fn collect_sliders<'a>(&'a self, sliders: &mut Vec<&'a Slider>) {
        if let Some(slider) = self.slider_model() {
            sliders.push(slider);
        }

        for child in &self.children {
            child.collect_sliders(sliders);
        }
    }

    pub(in crate::scratch::view) fn collect_text_boxes<'a>(
        &'a self,
        text_boxes: &mut Vec<&'a TextBox>,
    ) {
        if let Some(text_box) = self.text_box_model() {
            text_boxes.push(text_box);
        }

        for child in &self.children {
            child.collect_text_boxes(text_boxes);
        }
    }

    pub(in crate::scratch::view) fn contains_focus(&self, focus: session::Focus) -> bool {
        self.contains_focus_at(&focus, &[], false)
    }

    pub(in crate::scratch::view) fn contains_enabled_focus(
        &self,
        focus: session::Focus,
    ) -> bool {
        self.contains_focus_at(&focus, &[], true)
    }

    pub(in crate::scratch::view) fn collect_focus_order(&self, order: &mut Vec<session::Focus>) {
        self.collect_focus_order_at(order, &[]);
    }

    pub(in crate::scratch::view) fn collect_popup_focus_order(
        &self,
        order: &mut Vec<session::Focus>,
    ) -> bool {
        self.collect_popup_focus_order_at(order, &[])
    }

    pub(in crate::scratch::view) fn text_commit_action(
        &self,
        focus: session::Focus,
        text: String,
    ) -> Option<Action> {
        if self.text_box_model().and_then(TextBox::focus) == Some(focus.clone()) {
            return self
                .binding
                .as_ref()
                .and_then(|binding| binding.text_action(text));
        }

        self.children
            .iter()
            .find_map(|child| child.text_commit_action(focus.clone(), text.clone()))
    }

    pub(in crate::scratch::view) fn text_box_for_focus(
        &self,
        focus: session::Focus,
    ) -> Option<&TextBox> {
        if let Some(text_box) = self
            .text_box_model()
            .filter(|text_box| text_box.focus() == Some(focus.clone()))
        {
            return Some(text_box);
        }

        self.children
            .iter()
            .find_map(|child| child.text_box_for_focus(focus.clone()))
    }

    pub(in crate::scratch::view) fn text_input_target_for_focus(
        &self,
        focus: session::Focus,
    ) -> Option<interaction::Target> {
        if self.text_area_model().and_then(TextArea::focus) == Some(focus.clone()) {
            return self.text_control_target();
        }

        if self.text_box_model().and_then(TextBox::focus) == Some(focus.clone()) {
            return self.text_control_target();
        }

        self.children
            .iter()
            .find_map(|child| child.text_input_target_for_focus(focus.clone()))
    }

    pub(in crate::scratch::view) fn collect_menus<'a>(&'a self, menus: &mut Vec<&'a Node>) {
        if self.role == Role::Menu {
            menus.push(self);
        }

        for child in &self.children {
            child.collect_menus(menus);
        }
    }

    pub(in crate::scratch::view) fn collect_labels<'a>(&'a self, labels: &mut Vec<&'a str>) {
        if let Some(label) = &self.label {
            labels.push(label);
        }

        for child in &self.children {
            child.collect_labels(labels);
        }
    }

    pub(in crate::scratch::view) fn collect_popups<'a>(&'a self, popups: &mut Vec<&'a Node>) {
        if self.role == Role::Popup {
            popups.push(self);
        }

        for child in &self.children {
            child.collect_popups(popups);
        }
    }

    pub(in crate::scratch::view) fn resolve_commands(
        &mut self,
        registry: &framework_command::Registry,
        chain: &mut responder::Chain<'_, impl state::State>,
        cx: &CommandContext,
    ) {
        if let Some(binding) = &mut self.binding {
            binding.resolve(registry, chain, cx);
        }

        for child in &mut self.children {
            child.resolve_commands(registry, chain, cx);
        }
    }

    pub(in crate::scratch::view) fn prune_hidden_commands(&mut self) {
        for child in &mut self.children {
            child.prune_hidden_commands();
        }

        self.children.retain(|child| !child.is_hidden_binding());
    }

    pub(in crate::scratch::view) fn project_interaction(
        &mut self,
        interaction: &interaction::Interaction,
    ) {
        let text_area_target = if self.role == Role::TextArea {
            self.pointer_target()
        } else {
            None
        };
        if let Some(Control::TextArea(text_area)) = &mut self.control {
            text_area.project_interaction(interaction, text_area_target.as_ref());
        }

        if let Some(Control::TextBox(text_box)) = &mut self.control {
            text_box.project_interaction(interaction);
        }

        for child in &mut self.children {
            child.project_interaction(interaction);
        }
    }

    pub(in crate::scratch::view) fn project_focus(&mut self, focus: Option<&session::Focus>) {
        self.project_focus_at(focus, &[]);
    }

    fn project_focus_at(&mut self, focus: Option<&session::Focus>, path: &[usize]) {
        self.focused = self
            .focus_at(path, true)
            .as_ref()
            .is_some_and(|node_focus| Some(node_focus) == focus);

        if let Some(Control::TextArea(text_area)) = &mut self.control {
            text_area.project_focus(focus);
        }

        if let Some(Control::TextBox(text_box)) = &mut self.control {
            text_box.project_focus(focus);
        }

        for (index, child) in self.children.iter_mut().enumerate() {
            let mut child_path = path.to_vec();
            child_path.push(index);
            child.project_focus_at(focus, &child_path);
        }
    }

    pub(in crate::scratch::view) fn popup_for_menu(
        &self,
        menu: &interaction::Menu,
    ) -> Option<Node> {
        if self.role == Role::Menu && self.id == Some(menu.id()) {
            let mut popup = Node::popup(menu.id(), menu.label());
            popup.children = self.children.clone();
            return Some(popup);
        }

        self.children
            .iter()
            .find_map(|child| child.popup_for_menu(menu))
    }
}

impl Node {
    pub(in crate::scratch::view) fn focus_action(
        &self,
        focus: &session::Focus,
    ) -> Option<Action> {
        self.focus_action_at(focus, &[])
    }

    fn contains_focus_at(
        &self,
        focus: &session::Focus,
        path: &[usize],
        require_enabled: bool,
    ) -> bool {
        self.focus_at(path, require_enabled).as_ref() == Some(focus)
            || self
                .children
                .iter()
                .enumerate()
                .any(|(index, child)| {
                    let mut child_path = path.to_vec();
                    child_path.push(index);
                    child.contains_focus_at(focus, &child_path, require_enabled)
                })
    }

    fn collect_focus_order_at(&self, order: &mut Vec<session::Focus>, path: &[usize]) {
        if let Some(focus) = self.focus_at(path, true) {
            push_focus(order, focus);
        }

        if self.role == Role::Menu {
            return;
        }

        for (index, child) in self.children.iter().enumerate() {
            let mut child_path = path.to_vec();
            child_path.push(index);
            child.collect_focus_order_at(order, &child_path);
        }
    }

    fn collect_popup_focus_order_at(
        &self,
        order: &mut Vec<session::Focus>,
        path: &[usize],
    ) -> bool {
        if self.role == Role::Popup {
            self.collect_focus_order_at(order, path);
            return true;
        }

        let mut found = false;
        for (index, child) in self.children.iter().enumerate() {
            let mut child_path = path.to_vec();
            child_path.push(index);
            found |= child.collect_popup_focus_order_at(order, &child_path);
        }

        found
    }

    fn focus_action_at(&self, focus: &session::Focus, path: &[usize]) -> Option<Action> {
        if self.focus_at(path, true).as_ref() == Some(focus) {
            return self.keyboard_activation_action();
        }

        if self.role == Role::Menu {
            return None;
        }

        self.children.iter().enumerate().find_map(|(index, child)| {
            let mut child_path = path.to_vec();
            child_path.push(index);
            child.focus_action_at(focus, &child_path)
        })
    }

    fn focus_at(&self, path: &[usize], require_enabled: bool) -> Option<session::Focus> {
        if let Some(focus) = self.text_area_model().and_then(TextArea::focus) {
            return Some(focus);
        }

        if let Some(focus) = self.text_box_model().and_then(TextBox::focus) {
            return Some(focus);
        }

        if !self.is_keyboard_focusable(require_enabled) {
            return None;
        }

        self.pointer_target_at_path(path)
            .map(|target| session::Focus::control(&target))
    }

    fn is_keyboard_focusable(&self, require_enabled: bool) -> bool {
        match self.role {
            Role::Menu => true,
            Role::Binding | Role::Button | Role::Checkbox | Role::Radio | Role::Slider => {
                self.binding
                    .as_ref()
                    .is_some_and(|binding| !require_enabled || binding.is_enabled())
            }
            Role::TextArea | Role::TextBox => true,
            Role::Root
            | Role::Stack
            | Role::MenuBar
            | Role::Separator
            | Role::Panel
            | Role::Popup
            | Role::Label => false,
        }
    }
}

fn push_focus(order: &mut Vec<session::Focus>, focus: session::Focus) {
    if !order.contains(&focus) {
        order.push(focus);
    }
}
