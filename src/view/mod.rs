#[cfg(test)]
use std::any::TypeId;

use super::{
    command, composition, context::Context as CommandContext, interaction, responder, session,
    subject,
};

mod action;
mod binding;
mod command_palette;
mod context;
mod control;
mod node;
mod presentation;
mod style;

pub use action::Action;
pub(crate) use action::FocusDirection;
pub use binding::Binding;
pub(crate) use command_palette::{CommandPalette, Entry as CommandPaletteEntry};
pub use context::Context;
pub use control::{Button, Checkbox, Radio, Slider, TextArea, TextBox, Wrap};
pub use node::{Axis, FloatingPlacement, Node, Role};
pub use presentation::Presentation;
pub use style::{Align, Dimension, Padding, Style};

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

    #[cfg(test)]
    pub(crate) fn bindings(&self) -> Vec<&Binding> {
        let mut bindings = Vec::new();
        self.root.collect_bindings(&mut bindings);
        bindings
    }

    #[cfg(test)]
    pub(crate) fn binding<C: command::Command>(&self) -> Option<&Binding> {
        let command_type = TypeId::of::<C>();
        self.bindings()
            .into_iter()
            .find(|binding| binding.command_type() == command_type)
    }

    #[cfg(test)]
    pub(crate) fn text_areas(&self) -> Vec<&TextArea> {
        let mut text_areas = Vec::new();
        self.root.collect_text_areas(&mut text_areas);
        text_areas
    }

    #[cfg(test)]
    pub(crate) fn buttons(&self) -> Vec<&Button> {
        let mut buttons = Vec::new();
        self.root.collect_buttons(&mut buttons);
        buttons
    }

    #[cfg(test)]
    pub(crate) fn checkboxes(&self) -> Vec<&Checkbox> {
        let mut checkboxes = Vec::new();
        self.root.collect_checkboxes(&mut checkboxes);
        checkboxes
    }

    #[cfg(test)]
    pub(crate) fn radios(&self) -> Vec<&Radio> {
        let mut radios = Vec::new();
        self.root.collect_radios(&mut radios);
        radios
    }

    #[cfg(test)]
    pub(crate) fn sliders(&self) -> Vec<&Slider> {
        let mut sliders = Vec::new();
        self.root.collect_sliders(&mut sliders);
        sliders
    }

    #[cfg(test)]
    pub(crate) fn text_boxes(&self) -> Vec<&TextBox> {
        let mut text_boxes = Vec::new();
        self.root.collect_text_boxes(&mut text_boxes);
        text_boxes
    }

    pub fn contains_focus(&self, focus: session::Focus) -> bool {
        self.root.contains_focus(focus)
    }

    pub(crate) fn contains_enabled_focus_retained(
        &self,
        tree: &composition::Tree,
        focus: session::Focus,
    ) -> bool {
        self.root
            .contains_enabled_focus_retained(focus, tree.root())
    }

    pub(crate) fn focus_order_retained(&self, tree: &composition::Tree) -> Vec<session::Focus> {
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

    pub(crate) fn next_focus_retained(
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

    #[cfg(test)]
    pub(crate) fn menus(&self) -> Vec<&Node> {
        let mut menus = Vec::new();
        self.root.collect_menus(&mut menus);
        menus
    }

    #[cfg(test)]
    pub(crate) fn labels(&self) -> Vec<&str> {
        let mut labels = Vec::new();
        self.root.collect_labels(&mut labels);
        labels
    }

    #[cfg(test)]
    pub(crate) fn floating_panels(&self) -> Vec<&Node> {
        let mut panels = Vec::new();
        self.root.collect_floating_panels(&mut panels);
        panels
    }

    pub(crate) fn project_surfaces(&mut self, interaction: &interaction::Interaction) {
        if let Some(menu) = interaction.open_menu() {
            let panel = self
                .root
                .floating_panel_for_menu(menu)
                .unwrap_or_else(|| Node::floating_panel(menu.id()));
            self.root.push_child(panel);
        }
    }

    pub(crate) fn project_interaction_retained(
        &mut self,
        interaction: &interaction::Interaction,
        tree: &composition::Tree,
    ) {
        self.root
            .project_interaction_retained(interaction, tree.root());
    }

    pub(crate) fn project_focus_retained(
        &mut self,
        focus: Option<session::Focus>,
        tree: &composition::Tree,
    ) {
        self.root
            .project_focus_retained(focus.as_ref(), tree.root());
    }

    pub(crate) fn focus_action_retained(
        &self,
        focus: &session::Focus,
        tree: &composition::Tree,
    ) -> Option<Action> {
        self.root.focus_action_retained(focus, tree.root())
    }

    pub(crate) fn subject_path_for_focus_retained(
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
