use super::super::{
    action::Action,
    binding::Binding,
    control::{Button, Checkbox, Control, Radio, Slider, TextArea, TextBox},
};
use super::{Node, Role};
use crate::{
    command, composition, context::Context as CommandContext, interaction, responder, session,
    state, subject,
};

impl Node {
    pub(in crate::view) fn collect_bindings<'a>(&'a self, bindings: &mut Vec<&'a Binding>) {
        if let Some(binding) = &self.binding {
            bindings.push(binding);
        }

        for child in &self.children {
            child.collect_bindings(bindings);
        }
    }

    pub(in crate::view) fn collect_text_areas<'a>(&'a self, text_areas: &mut Vec<&'a TextArea>) {
        if let Some(text_area) = self.text_area_model() {
            text_areas.push(text_area);
        }

        for child in &self.children {
            child.collect_text_areas(text_areas);
        }
    }

    pub(in crate::view) fn collect_buttons<'a>(&'a self, buttons: &mut Vec<&'a Button>) {
        if let Some(button) = self.button_model() {
            buttons.push(button);
        }

        for child in &self.children {
            child.collect_buttons(buttons);
        }
    }

    pub(in crate::view) fn collect_checkboxes<'a>(&'a self, checkboxes: &mut Vec<&'a Checkbox>) {
        if let Some(checkbox) = self.checkbox_model() {
            checkboxes.push(checkbox);
        }

        for child in &self.children {
            child.collect_checkboxes(checkboxes);
        }
    }

    pub(in crate::view) fn collect_radios<'a>(&'a self, radios: &mut Vec<&'a Radio>) {
        if let Some(radio) = self.radio_model() {
            radios.push(radio);
        }

        for child in &self.children {
            child.collect_radios(radios);
        }
    }

    pub(in crate::view) fn collect_sliders<'a>(&'a self, sliders: &mut Vec<&'a Slider>) {
        if let Some(slider) = self.slider_model() {
            sliders.push(slider);
        }

        for child in &self.children {
            child.collect_sliders(sliders);
        }
    }

    pub(in crate::view) fn collect_text_boxes<'a>(&'a self, text_boxes: &mut Vec<&'a TextBox>) {
        if let Some(text_box) = self.text_box_model() {
            text_boxes.push(text_box);
        }

        for child in &self.children {
            child.collect_text_boxes(text_boxes);
        }
    }

    pub(in crate::view) fn contains_focus(&self, focus: session::Focus) -> bool {
        self.contains_focus_at(&focus, false)
    }

    pub(in crate::view) fn contains_enabled_focus_retained(
        &self,
        focus: session::Focus,
        retained: &composition::Node,
    ) -> bool {
        self.contains_focus_retained_at(&focus, retained, true)
    }

    pub(in crate::view) fn collect_focus_order_retained(
        &self,
        retained: &composition::Node,
        order: &mut Vec<session::Focus>,
    ) {
        self.collect_focus_order_retained_at(retained, order);
    }

    pub(in crate::view) fn collect_floating_panel_focus_order_retained(
        &self,
        retained: &composition::Node,
        order: &mut Vec<session::Focus>,
    ) -> bool {
        self.collect_floating_panel_focus_order_retained_at(retained, order)
    }

    pub(in crate::view) fn text_commit_action(
        &self,
        focus: session::Focus,
        text: String,
    ) -> Option<Action> {
        if self
            .text_box_model()
            .and_then(TextBox::focus)
            .is_some_and(|text_focus| text_focus.same_target(&focus))
        {
            return self
                .binding
                .as_ref()
                .and_then(|binding| binding.text_action(text));
        }

        self.children
            .iter()
            .find_map(|child| child.text_commit_action(focus.clone(), text.clone()))
    }

    pub(in crate::view) fn text_box_for_focus(&self, focus: session::Focus) -> Option<&TextBox> {
        if let Some(text_box) = self.text_box_model().filter(|text_box| {
            text_box
                .focus()
                .is_some_and(|text_focus| text_focus.same_target(&focus))
        }) {
            return Some(text_box);
        }

        self.children
            .iter()
            .find_map(|child| child.text_box_for_focus(focus.clone()))
    }

    pub(in crate::view) fn text_input_target_for_focus(
        &self,
        focus: session::Focus,
    ) -> Option<interaction::Target> {
        if self
            .text_area_model()
            .and_then(TextArea::focus)
            .is_some_and(|text_focus| text_focus.same_target(&focus))
        {
            return self.text_control_target();
        }

        if self
            .text_box_model()
            .and_then(TextBox::focus)
            .is_some_and(|text_focus| text_focus.same_target(&focus))
        {
            return self.text_control_target();
        }

        self.children
            .iter()
            .find_map(|child| child.text_input_target_for_focus(focus.clone()))
    }

    pub(in crate::view) fn collect_menus<'a>(&'a self, menus: &mut Vec<&'a Node>) {
        if self.role == Role::Menu {
            menus.push(self);
        }

        for child in &self.children {
            child.collect_menus(menus);
        }
    }

    pub(in crate::view) fn collect_labels<'a>(&'a self, labels: &mut Vec<&'a str>) {
        if let Some(label) = &self.label {
            labels.push(label);
        }

        for child in &self.children {
            child.collect_labels(labels);
        }
    }

    pub(in crate::view) fn collect_floating_panels<'a>(&'a self, panels: &mut Vec<&'a Node>) {
        if self.role == Role::FloatingPanel {
            panels.push(self);
        }

        for child in &self.children {
            child.collect_floating_panels(panels);
        }
    }

    pub(in crate::view) fn resolve_commands(
        &mut self,
        registry: &command::Registry,
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

    pub(in crate::view) fn prune_hidden_commands(&mut self) {
        for child in &mut self.children {
            child.prune_hidden_commands();
        }

        self.children.retain(|child| !child.is_hidden_binding());
    }

    pub(in crate::view) fn project_interaction_retained(
        &mut self,
        interaction: &interaction::Interaction,
        retained: &composition::Node,
    ) {
        self.project_interaction_retained_at(interaction, retained);
    }

    fn project_interaction_retained_at(
        &mut self,
        interaction: &interaction::Interaction,
        retained: &composition::Node,
    ) {
        let pointer_target = self.node_pointer_target(require_retained_id(retained));
        self.hovered = pointer_target
            .as_ref()
            .is_some_and(|target| interaction.pointer().hovered() == Some(target));
        self.pressed = pointer_target
            .as_ref()
            .is_some_and(|target| interaction.pointer().pressed() == Some(target));
        let pointer_active = pointer_target
            .as_ref()
            .is_some_and(|target| interaction.pointer().activation_target() == Some(target));
        self.active = match self.role {
            Role::MenuBar => interaction.open_menu().is_some(),
            Role::Menu => interaction
                .open_menu()
                .is_some_and(|menu| self.id == Some(menu.id())),
            _ => pointer_active,
        };
        self.scroll_offset = if self.role == Role::Scroll {
            pointer_target
                .as_ref()
                .map(|target| interaction.scroll().offset(target))
                .unwrap_or_default()
        } else {
            interaction::ScrollOffset::default()
        };

        let text_area_target = if self.role == Role::TextArea {
            pointer_target.clone()
        } else {
            None
        };
        if let Some(Control::TextArea(text_area)) = &mut self.control {
            text_area.project_interaction(interaction, text_area_target.as_ref());
        }

        if let Some(Control::TextBox(text_box)) = &mut self.control {
            text_box.project_interaction(interaction, self.binding.is_some());
        }

        for (index, child) in self.children.iter_mut().enumerate() {
            child.project_interaction_retained_at(interaction, retained_child(retained, index));
        }
    }

    pub(in crate::view) fn project_focus_retained(
        &mut self,
        focus: Option<&session::Focus>,
        retained: &composition::Node,
    ) {
        self.project_focus_retained_at(focus, retained);
    }

    fn project_focus_retained_at(
        &mut self,
        focus: Option<&session::Focus>,
        retained: &composition::Node,
    ) {
        self.focused = self
            .focus_at_retained(retained, true)
            .as_ref()
            .is_some_and(|node_focus| focus.is_some_and(|focus| node_focus.same_target(focus)));
        self.focus_visible =
            self.focused && focus.is_some_and(|focus| focus.shows_focus_indicator());

        if let Some(Control::TextArea(text_area)) = &mut self.control {
            text_area.project_focus(focus);
        }

        if let Some(Control::TextBox(text_box)) = &mut self.control {
            text_box.project_focus(focus);
        }

        for (index, child) in self.children.iter_mut().enumerate() {
            child.project_focus_retained_at(focus, retained_child(retained, index));
        }
    }

    pub(in crate::view) fn floating_panel_for_menu(
        &self,
        menu: &interaction::Menu,
    ) -> Option<Node> {
        if self.role == Role::Menu && self.id == Some(menu.id()) {
            let mut panel = Node::floating_panel(menu.id());
            panel.children = self.children.clone();
            return Some(panel);
        }

        self.children
            .iter()
            .find_map(|child| child.floating_panel_for_menu(menu))
    }
}

impl Node {
    pub(in crate::view) fn focus_action_retained(
        &self,
        focus: &session::Focus,
        retained: &composition::Node,
    ) -> Option<Action> {
        self.focus_action_retained_at(focus, retained)
    }

    pub(in crate::view) fn subject_path_for_focus_retained(
        &self,
        focus: session::Focus,
        retained: &composition::Node,
    ) -> Option<subject::Path> {
        self.subject_path_for_focus_retained_at(&focus, retained, &mut Vec::new())
    }

    fn contains_focus_at(&self, focus: &session::Focus, require_enabled: bool) -> bool {
        self.focus_at(require_enabled)
            .as_ref()
            .is_some_and(|node_focus| node_focus.same_target(focus))
            || self
                .children
                .iter()
                .any(|child| child.contains_focus_at(focus, require_enabled))
    }

    fn contains_focus_retained_at(
        &self,
        focus: &session::Focus,
        retained: &composition::Node,
        require_enabled: bool,
    ) -> bool {
        self.focus_at_retained(retained, require_enabled)
            .as_ref()
            .is_some_and(|node_focus| node_focus.same_target(focus))
            || self.children.iter().enumerate().any(|(index, child)| {
                child.contains_focus_retained_at(
                    focus,
                    retained_child(retained, index),
                    require_enabled,
                )
            })
    }

    fn collect_focus_order_retained_at(
        &self,
        retained: &composition::Node,
        order: &mut Vec<session::Focus>,
    ) {
        if let Some(focus) = self.focus_at_retained(retained, true) {
            push_focus(order, focus.keyboard());
        }

        if self.role == Role::Menu {
            return;
        }

        for (index, child) in self.children.iter().enumerate() {
            child.collect_focus_order_retained_at(retained_child(retained, index), order);
        }
    }

    fn collect_floating_panel_focus_order_retained_at(
        &self,
        retained: &composition::Node,
        order: &mut Vec<session::Focus>,
    ) -> bool {
        if self.role == Role::FloatingPanel {
            self.collect_focus_order_retained_at(retained, order);
            return true;
        }

        let mut found = false;
        for (index, child) in self.children.iter().enumerate() {
            found |= child.collect_floating_panel_focus_order_retained_at(
                retained_child(retained, index),
                order,
            );
        }

        found
    }

    fn focus_action_retained_at(
        &self,
        focus: &session::Focus,
        retained: &composition::Node,
    ) -> Option<Action> {
        if self
            .focus_at_retained(retained, true)
            .as_ref()
            .is_some_and(|node_focus| node_focus.same_target(focus))
        {
            return self.keyboard_activation_action();
        }

        if self.role == Role::Menu {
            return None;
        }

        self.children.iter().enumerate().find_map(|(index, child)| {
            child.focus_action_retained_at(focus, retained_child(retained, index))
        })
    }

    fn subject_path_for_focus_retained_at(
        &self,
        focus: &session::Focus,
        retained: &composition::Node,
        segments: &mut Vec<subject::Segment>,
    ) -> Option<subject::Path> {
        let pushed = retained.subject().cloned().inspect(|segment| {
            segments.push(segment.clone());
        });

        if self
            .focus_at_retained(retained, false)
            .as_ref()
            .is_some_and(|node_focus| node_focus.same_target(focus))
        {
            return Some(subject::Path::new(segments.clone()));
        }

        for (index, child) in self.children.iter().enumerate() {
            if let Some(path) = child.subject_path_for_focus_retained_at(
                focus,
                retained_child(retained, index),
                segments,
            ) {
                return Some(path);
            }
        }

        if pushed.is_some() {
            segments.pop();
        }
        None
    }

    fn focus_at(&self, require_enabled: bool) -> Option<session::Focus> {
        if let Some(focus) = self.text_area_model().and_then(TextArea::focus) {
            return Some(focus);
        }

        if let Some(focus) = self.text_box_model().and_then(TextBox::focus) {
            return Some(focus);
        }

        if !self.is_keyboard_focusable(require_enabled) {
            return None;
        }

        self.pointer_target()
            .map(|target| session::Focus::control(&target))
    }

    fn focus_at_retained(
        &self,
        retained: &composition::Node,
        require_enabled: bool,
    ) -> Option<session::Focus> {
        if let Some(focus) = self.text_area_model().and_then(TextArea::focus) {
            return Some(focus);
        }

        if let Some(focus) = self.text_box_model().and_then(TextBox::focus) {
            return Some(focus);
        }

        if !self.is_keyboard_focusable(require_enabled) {
            return None;
        }

        self.node_pointer_target(require_retained_id(retained))
            .map(|target| session::Focus::control(&target))
    }

    fn is_keyboard_focusable(&self, require_enabled: bool) -> bool {
        match self.role {
            Role::Menu => true,
            Role::Binding | Role::Button | Role::Checkbox | Role::Radio | Role::Slider => self
                .binding
                .as_ref()
                .is_some_and(|binding| !require_enabled || binding.is_enabled()),
            Role::TextArea | Role::TextBox => true,
            Role::Root
            | Role::Stack
            | Role::MenuBar
            | Role::Separator
            | Role::Scroll
            | Role::Panel
            | Role::FloatingPanel
            | Role::SectionHeader
            | Role::Label => false,
        }
    }
}

fn push_focus(order: &mut Vec<session::Focus>, focus: session::Focus) {
    if !order.iter().any(|existing| existing.same_target(&focus)) {
        order.push(focus);
    }
}

fn retained_child(parent: &composition::Node, index: usize) -> &composition::Node {
    parent
        .children()
        .get(index)
        .expect("composition tree must match view child order")
}

fn require_retained_id(node: &composition::Node) -> composition::NodeId {
    node.retained_id()
        .expect("retained view traversal requires retained composition identity")
}
