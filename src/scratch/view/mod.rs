use std::any::TypeId;

use super::{
    command as framework_command, context::Context as CommandContext, interaction, responder,
    session,
};

pub mod action;
mod binding;
mod context;
pub mod control;
pub mod node;
mod presentation;
pub mod style;

pub use action::Action;
pub use binding::Binding;
pub use context::Context;
pub use node::Node;
pub use presentation::Presentation;
pub use style::Style;

use action::FocusDirection;
use control::{Button, Checkbox, Radio, Slider, TextArea, TextBox};

#[derive(Clone)]
pub struct View {
    root: Node,
}

impl View {
    pub fn new(root: Node) -> Self {
        Self { root }
    }

    pub fn root(&self) -> &Node {
        &self.root
    }

    pub fn bindings(&self) -> Vec<&Binding> {
        let mut bindings = Vec::new();
        self.root.collect_bindings(&mut bindings);
        bindings
    }

    pub fn binding<C: framework_command::Command>(&self) -> Option<&Binding> {
        let command_type = TypeId::of::<C>();
        self.bindings()
            .into_iter()
            .find(|binding| binding.command_type() == command_type)
    }

    pub fn text_areas(&self) -> Vec<&TextArea> {
        let mut text_areas = Vec::new();
        self.root.collect_text_areas(&mut text_areas);
        text_areas
    }

    pub fn buttons(&self) -> Vec<&Button> {
        let mut buttons = Vec::new();
        self.root.collect_buttons(&mut buttons);
        buttons
    }

    pub fn checkboxes(&self) -> Vec<&Checkbox> {
        let mut checkboxes = Vec::new();
        self.root.collect_checkboxes(&mut checkboxes);
        checkboxes
    }

    pub fn radios(&self) -> Vec<&Radio> {
        let mut radios = Vec::new();
        self.root.collect_radios(&mut radios);
        radios
    }

    pub fn sliders(&self) -> Vec<&Slider> {
        let mut sliders = Vec::new();
        self.root.collect_sliders(&mut sliders);
        sliders
    }

    pub fn text_boxes(&self) -> Vec<&TextBox> {
        let mut text_boxes = Vec::new();
        self.root.collect_text_boxes(&mut text_boxes);
        text_boxes
    }

    pub fn contains_focus(&self, focus: session::Focus) -> bool {
        self.root.contains_focus(focus)
    }

    pub fn contains_enabled_focus(&self, focus: session::Focus) -> bool {
        self.root.contains_enabled_focus(focus)
    }

    pub fn focus_order(&self) -> Vec<session::Focus> {
        let mut order = Vec::new();
        if !self.root.collect_popup_focus_order(&mut order) {
            self.root.collect_focus_order(&mut order);
        }
        order
    }

    pub fn next_focus(
        &self,
        current: Option<session::Focus>,
        direction: FocusDirection,
    ) -> Option<session::Focus> {
        let order = self.focus_order();
        if order.is_empty() {
            return None;
        }

        let index =
            current.and_then(|focus| order.iter().position(|candidate| *candidate == focus));
        match (index, direction) {
            (Some(index), FocusDirection::Forward) => Some(order[(index + 1) % order.len()]),
            (Some(0), FocusDirection::Backward) => order.last().copied(),
            (Some(index), FocusDirection::Backward) => Some(order[index - 1]),
            (None, FocusDirection::Forward) => order.first().copied(),
            (None, FocusDirection::Backward) => order.last().copied(),
        }
    }

    pub(super) fn text_commit_action(&self, focus: session::Focus, text: String) -> Option<Action> {
        self.root.text_commit_action(focus, text)
    }

    pub(super) fn text_box_text(&self, focus: session::Focus) -> Option<&str> {
        self.root.text_box_for_focus(focus).map(TextBox::text)
    }

    pub(super) fn text_input_target(&self, focus: session::Focus) -> Option<interaction::Target> {
        self.root.text_input_target_for_focus(focus)
    }

    pub fn menus(&self) -> Vec<&Node> {
        let mut menus = Vec::new();
        self.root.collect_menus(&mut menus);
        menus
    }

    pub fn labels(&self) -> Vec<&str> {
        let mut labels = Vec::new();
        self.root.collect_labels(&mut labels);
        labels
    }

    pub fn popups(&self) -> Vec<&Node> {
        let mut popups = Vec::new();
        self.root.collect_popups(&mut popups);
        popups
    }

    pub(super) fn project_interaction(&mut self, interaction: &interaction::Interaction) {
        self.root.project_interaction(interaction);

        if let Some(menu) = interaction.open_menu() {
            let popup = self
                .root
                .popup_for_menu(menu)
                .unwrap_or_else(|| Node::popup(menu.id(), menu.label()));
            self.root.push_child(popup);
        }
    }

    pub(super) fn project_focus(&mut self, focus: Option<session::Focus>) {
        self.root.project_focus(focus.as_ref());
    }

    pub(in crate::scratch) fn focus_action(&self, focus: &session::Focus) -> Option<Action> {
        self.root.focus_action(focus)
    }

    pub(super) fn resolve_commands(
        &mut self,
        registry: &framework_command::Registry,
        chain: &mut responder::Chain<'_, impl super::state::State>,
        cx: &CommandContext,
    ) {
        self.root.resolve_commands(registry, chain, cx);
        self.root.prune_hidden_commands();
    }
}
