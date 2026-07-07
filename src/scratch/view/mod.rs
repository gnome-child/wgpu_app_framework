use std::any::TypeId;

use super::{
    command, composition, context::Context as CommandContext, interaction, responder, session,
    subject,
};

pub mod action;
mod binding;
mod command_palette;
mod context;
pub mod control;
pub mod node;
mod presentation;
pub mod style;

pub use action::Action;
pub use binding::Binding;
pub(in crate::scratch) use command_palette::{CommandPalette, Entry as CommandPaletteEntry};
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

    pub fn binding<C: command::Command>(&self) -> Option<&Binding> {
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

    pub(in crate::scratch) fn contains_enabled_focus_retained(
        &self,
        tree: &composition::Tree,
        focus: session::Focus,
    ) -> bool {
        self.root
            .contains_enabled_focus_retained(focus, tree.root())
    }

    pub fn focus_order(&self) -> Vec<session::Focus> {
        let mut order = Vec::new();
        if !self.root.collect_floating_panel_focus_order(&mut order) {
            self.root.collect_focus_order(&mut order);
        }
        order
    }

    pub(in crate::scratch) fn focus_order_retained(
        &self,
        tree: &composition::Tree,
    ) -> Vec<session::Focus> {
        let mut order = Vec::new();
        if !self
            .root
            .collect_floating_panel_focus_order_retained(tree.root(), &mut order)
        {
            self.root
                .collect_focus_order_retained(tree.root(), &mut order);
        }
        order
    }

    pub fn next_focus(
        &self,
        current: Option<session::Focus>,
        direction: FocusDirection,
    ) -> Option<session::Focus> {
        next_focus_in_order(self.focus_order(), current, direction)
    }

    pub(in crate::scratch) fn next_focus_retained(
        &self,
        tree: &composition::Tree,
        current: Option<session::Focus>,
        direction: FocusDirection,
    ) -> Option<session::Focus> {
        next_focus_in_order(self.focus_order_retained(tree), current, direction)
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

    pub fn floating_panels(&self) -> Vec<&Node> {
        let mut panels = Vec::new();
        self.root.collect_floating_panels(&mut panels);
        panels
    }

    pub(in crate::scratch) fn project_surfaces(&mut self, interaction: &interaction::Interaction) {
        if let Some(menu) = interaction.open_menu() {
            let panel = self
                .root
                .floating_panel_for_menu(menu)
                .unwrap_or_else(|| Node::floating_panel(menu.id()));
            self.root.push_child(panel);
        }
    }

    pub(in crate::scratch) fn project_interaction_retained(
        &mut self,
        interaction: &interaction::Interaction,
        tree: &composition::Tree,
    ) {
        self.root
            .project_interaction_retained(interaction, tree.root());
    }

    pub(in crate::scratch) fn project_focus_retained(
        &mut self,
        focus: Option<session::Focus>,
        tree: &composition::Tree,
    ) {
        self.root
            .project_focus_retained(focus.as_ref(), tree.root());
    }

    pub(in crate::scratch) fn focus_action_retained(
        &self,
        focus: &session::Focus,
        tree: &composition::Tree,
    ) -> Option<Action> {
        self.root.focus_action_retained(focus, tree.root())
    }

    pub(in crate::scratch) fn subject_path_for_focus_retained(
        &self,
        focus: session::Focus,
        tree: &composition::Tree,
    ) -> Option<subject::Path> {
        self.root
            .subject_path_for_focus_retained(focus, tree.root())
    }

    pub(super) fn resolve_commands(
        &mut self,
        registry: &command::Registry,
        chain: &mut responder::Chain<'_, impl super::state::State>,
        cx: &CommandContext,
    ) {
        self.root.resolve_commands(registry, chain, cx);
        self.root.prune_hidden_commands();
    }
}

fn next_focus_in_order(
    order: Vec<session::Focus>,
    current: Option<session::Focus>,
    direction: FocusDirection,
) -> Option<session::Focus> {
    if order.is_empty() {
        return None;
    }

    let index = current.and_then(|focus| {
        order
            .iter()
            .position(|candidate| candidate.same_target(&focus))
    });
    match (index, direction) {
        (Some(index), FocusDirection::Forward) => Some(order[(index + 1) % order.len()]),
        (Some(0), FocusDirection::Backward) => order.last().copied(),
        (Some(index), FocusDirection::Backward) => Some(order[index - 1]),
        (None, FocusDirection::Forward) => order.first().copied(),
        (None, FocusDirection::Backward) => order.last().copied(),
    }
}
