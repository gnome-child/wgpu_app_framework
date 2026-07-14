#[cfg(test)]
use std::any::TypeId;

use super::{
    command,
    composition::{Tree, tree},
    context::Context as CommandContext,
    interaction, responder, session, subject,
};
use std::collections::HashMap;

mod action;
mod binding;
mod command_palette;
mod commit;
mod context;
mod context_menu;
mod control;
mod feedback;
pub(crate) mod focus;
mod hint;
pub(crate) mod node;
#[cfg(test)]
mod presentation;
mod style;

pub(crate) use action::{Action, FocusDirection};
pub use binding::Binding;
pub(crate) use command_palette::{CommandPalette, Entry as CommandPaletteEntry};
pub(crate) use commit::TextCommit;
pub use context::Context;
pub(crate) use context_menu::ContextMenu;
pub use control::{Button, Checkbox, Radio, Slider, TextArea, TextBox, Wrap};
pub(crate) use hint::{Hint, Tone};
pub(crate) use node::StandardMenuExtension;
pub use node::{Axis, FloatingPlacement, NativePopupMaterialPreference, Node};
pub(crate) use node::{PanelAttachment, PanelPolicy, Participation, ProvidedRow, Role, TablePart};
#[cfg(test)]
pub(crate) use presentation::Presentation;
pub use style::{Align, Dimension, Padding, Style};

#[derive(Clone)]
pub struct View {
    root: Node,
}

#[derive(Clone)]
pub(crate) struct ContextOwner {
    responder: Option<interaction::Id>,
    focus: Option<session::Focus>,
    binding: Option<Binding>,
    application: bool,
    location: Location,
    service: ContextService,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Location {
    None,
    Table(interaction::Id),
    Row(crate::table::Row),
    Cell(crate::table::Cell),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum ContextService {
    None,
    Table,
    Text,
}

impl ContextOwner {
    fn new(
        responder: Option<interaction::Id>,
        focus: Option<session::Focus>,
        binding: Option<Binding>,
        application: bool,
        location: Location,
        service: ContextService,
    ) -> Self {
        Self {
            responder,
            focus,
            binding,
            application,
            location,
            service,
        }
    }

    pub(crate) fn responder(&self) -> Option<interaction::Id> {
        self.responder
    }

    pub(crate) fn focus(&self) -> Option<session::Focus> {
        self.focus
    }

    pub(crate) fn binding(&self) -> Option<&Binding> {
        self.binding.as_ref()
    }

    pub(crate) fn is_application(&self) -> bool {
        self.application
    }

    pub(crate) fn table(&self) -> Option<interaction::Id> {
        match self.location {
            Location::None => None,
            Location::Table(table) => Some(table),
            Location::Row(row) => Some(row.table()),
            Location::Cell(cell) => Some(cell.table()),
        }
    }

    pub(crate) fn row(&self) -> Option<crate::table::Row> {
        match self.location {
            Location::Row(row) => Some(row),
            Location::None | Location::Table(_) | Location::Cell(_) => None,
        }
    }

    #[cfg(test)]
    pub(crate) fn cell(&self) -> Option<crate::table::Cell> {
        match self.location {
            Location::Cell(cell) => Some(cell),
            Location::None | Location::Table(_) | Location::Row(_) => None,
        }
    }

    pub(crate) fn service(&self) -> ContextService {
        self.service
    }
}

impl View {
    pub fn new(root: Node) -> Self {
        let root = if root.role() == Role::Root {
            root
        } else {
            Node::root().child(root)
        };
        Self { root }
    }

    pub fn root(&self) -> &Node {
        &self.root
    }

    pub(in crate::view) fn push_floating_panel(&mut self, panel: Node) {
        debug_assert_eq!(panel.role(), Role::FloatingPanel);
        debug_assert_eq!(self.root.role(), Role::Root);
        self.root.push_child(panel);
    }

    pub(crate) fn has_standard_menu_bar(&self) -> bool {
        self.root.has_standard_menu_bar()
    }

    pub(crate) fn project_standard_menu_bar(&mut self, projection: &command::BarProjection) {
        self.root.project_standard_menu_bar(projection);
    }

    pub(crate) fn resolve_standard_menu_extensions(
        &mut self,
        registry: &command::Registry,
        chain: &mut responder::Chain<'_, impl super::state::State>,
        cx: &CommandContext,
    ) {
        self.root
            .resolve_standard_menu_extensions(registry, chain, cx);
    }

    pub(crate) fn materialize_virtual_lists(
        &mut self,
        requests: &HashMap<interaction::Id, crate::virtual_list::Materialization>,
        measurements: &HashMap<interaction::Id, crate::virtual_list::Measurements>,
    ) {
        self.root.materialize_virtual_lists(requests, measurements);
    }

    pub(crate) fn project_table_widths(&mut self, tables: &interaction::Tables) {
        self.root.project_table_widths(tables);
    }

    pub(crate) fn selectable_virtual_lists(&self) -> Vec<crate::virtual_list::Model> {
        let mut models = Vec::new();
        self.root.collect_selectable_virtual_lists(&mut models);
        models
    }

    pub(crate) fn virtual_list_model(
        &self,
        id: interaction::Id,
    ) -> Option<&crate::virtual_list::Model> {
        self.root.virtual_list_model_for_id(id)
    }

    pub(crate) fn table_columns(&self, id: interaction::Id) -> Vec<interaction::Id> {
        self.root
            .table_model_for_id(id)
            .map(crate::table::Model::column_ids)
            .unwrap_or_default()
    }

    pub(crate) fn table_cell_focus(&self, cell: crate::table::Cell) -> Option<session::Focus> {
        self.root.table_cell_focus(cell)
    }

    pub(crate) fn selectable_virtual_list_for_focus(
        &self,
        tree: &Tree,
        focus: session::Focus,
    ) -> Option<&crate::virtual_list::Model> {
        self.root
            .selectable_virtual_list_for_focus_retained(focus, tree.root())
    }

    pub(crate) fn project_virtual_selections(
        &mut self,
        selections: &[(interaction::Id, crate::selection::Selection)],
    ) {
        self.root.project_virtual_selections(selections);
    }

    pub(crate) fn project_active_table_cells(
        &mut self,
        interaction: &interaction::Interaction,
        selections: &[(interaction::Id, crate::selection::Selection)],
    ) {
        self.root
            .project_active_table_cells(interaction, selections);
    }

    pub(crate) fn project_input_feedback(&mut self, interaction: &interaction::Interaction) {
        self.root.project_input_feedback(interaction);
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
        tree: &Tree,
        focus: session::Focus,
    ) -> bool {
        self.root
            .contains_enabled_focus_retained(focus, tree.root())
    }

    pub(crate) fn virtual_list_pins_retained(
        &self,
        tree: &Tree,
        focus: Option<session::Focus>,
        targets: &[interaction::Target],
    ) -> HashMap<interaction::Id, Vec<crate::virtual_list::Key>> {
        let mut pins = HashMap::new();
        self.root
            .collect_virtual_list_pins_retained(tree.root(), focus, targets, &mut pins);
        pins
    }

    pub(crate) fn focus_order_retained(&self, tree: &Tree) -> Vec<session::Focus> {
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
        tree: &Tree,
        current: Option<session::Focus>,
        direction: FocusDirection,
    ) -> Option<session::Focus> {
        next_focus_in_order(self.focus_order_retained(tree), current, direction)
    }

    pub(crate) fn next_focus_outside_table_retained(
        &self,
        tree: &Tree,
        current: session::Focus,
        direction: FocusDirection,
        table: interaction::Id,
    ) -> Option<session::Focus> {
        let order = self.focus_order_retained(tree);
        let index = order
            .iter()
            .position(|candidate| candidate.same_target(&current))?;
        let outside = |candidate: &&session::Focus| {
            candidate
                .table_cell_identity()
                .is_none_or(|cell| cell.table() != table)
        };
        match direction {
            FocusDirection::Forward => order.iter().skip(index + 1).find(outside).copied(),
            FocusDirection::Backward => order.iter().take(index).rev().find(outside).copied(),
        }
    }

    pub(super) fn text_commit(&self, focus: session::Focus) -> Option<TextCommit> {
        self.root.text_commit_for_focus(focus)
    }

    pub(super) fn draft_text(&self, focus: session::Focus) -> Option<String> {
        self.root.draft_text_for_focus(focus)
    }

    pub(super) fn draft_input(&self, focus: session::Focus) -> Option<crate::text::Input> {
        self.root.text_box_for_focus(focus).map(TextBox::input)
    }

    pub(crate) fn text_surface_mode(
        &self,
        focus: session::Focus,
    ) -> Option<crate::text::surface::FieldMode> {
        self.root.text_surface_mode_for_focus(focus)
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
            if menu.is_context() {
                return;
            }
            let panel = self
                .root
                .floating_panel_for_menu(menu)
                .unwrap_or_else(|| Node::floating_panel(menu.id()));
            self.push_floating_panel(panel);
        }
    }

    pub(crate) fn project_layout_interaction_retained(
        &mut self,
        interaction: &interaction::Interaction,
        tree: &Tree,
    ) {
        self.root
            .project_layout_interaction_retained(interaction, tree.root());
    }

    pub(crate) fn project_focus_retained(&mut self, focus: Option<session::Focus>, tree: &Tree) {
        self.root
            .project_focus_retained(focus.as_ref(), tree.root());
    }

    pub(crate) fn focus_action_retained(
        &self,
        focus: &session::Focus,
        tree: &Tree,
    ) -> Option<Action> {
        self.root.focus_action_retained(focus, tree.root())
    }

    pub(crate) fn subject_path_for_focus_retained(
        &self,
        focus: session::Focus,
        tree: &Tree,
    ) -> Option<subject::Path> {
        self.root
            .subject_path_for_focus_retained(focus, tree.root())
    }

    pub(crate) fn context_path_retained(
        &self,
        tree: &Tree,
        target: tree::NodeId,
    ) -> Vec<ContextOwner> {
        let mut path = Vec::new();
        self.root
            .context_path_retained(tree.root(), target, &mut path);
        path
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
