mod store;
pub(crate) mod tree;

pub(crate) use store::Store;
pub(crate) use tree::Tree;

use tree::{Changes, NodeId};

use super::{interaction, session, subject, view, window};

pub(crate) struct Composition {
    window: window::Id,
    view: view::View,
    tree: Tree,
    changes: Changes,
}

impl Composition {
    pub(super) fn new_prepared(
        window: window::Id,
        view: view::View,
        tree: Tree,
        changes: Changes,
    ) -> Self {
        Self {
            window,
            view,
            tree,
            changes,
        }
    }

    pub(super) fn install_prepared(&mut self, view: view::View, tree: Tree, changes: Changes) {
        self.view = view;
        self.tree = tree;
        self.changes = changes;
    }

    pub fn window(&self) -> window::Id {
        self.window
    }

    pub fn view(&self) -> &view::View {
        &self.view
    }

    pub(super) fn view_mut(&mut self) -> &mut view::View {
        &mut self.view
    }

    pub(super) fn reconcile_residency(
        &mut self,
        deltas: &[crate::list::AppliedResidencyDelta],
        next_node_id: &mut u64,
    ) -> Changes {
        let changes = self
            .tree
            .reconcile_residency(&self.view, deltas, next_node_id);
        self.changes = changes.clone();
        changes
    }

    pub(super) fn project_residency_state(
        &mut self,
        deltas: &[crate::list::AppliedResidencyDelta],
        interaction: Option<&interaction::Interaction>,
        focus: Option<session::Focus>,
    ) -> Changes {
        self.view
            .project_residency_retained(deltas, interaction, focus, &self.tree);
        self.tree
            .project_residency_scene_state(&self.view, deltas, &mut self.changes);
        self.changes.clone()
    }

    pub(crate) fn project_transient_state(
        &mut self,
        interaction: Option<&interaction::Interaction>,
        focus: Option<session::Focus>,
    ) {
        let tree = self.tree.clone();
        if let Some(interaction) = interaction {
            self.view.project_table_widths(interaction.tables());
            let selections = interaction.selections().snapshot();
            self.view.project_virtual_selections(&selections);
            self.view
                .project_layout_interaction_retained(interaction, &tree);
        }
        self.view.project_focus_retained(focus, &tree);
        self.tree.project_scene_state(&self.view, &mut self.changes);
    }

    pub(crate) fn tree(&self) -> &Tree {
        &self.tree
    }

    pub(crate) fn changes(&self) -> &Changes {
        &self.changes
    }

    pub(crate) fn next_focus(
        &self,
        current: Option<session::Focus>,
        direction: view::FocusDirection,
    ) -> Option<session::Focus> {
        self.view
            .next_focus_retained(&self.tree, current, direction)
    }

    pub(crate) fn next_focus_outside_table(
        &self,
        current: session::Focus,
        direction: view::FocusDirection,
        table: interaction::Id,
    ) -> Option<session::Focus> {
        self.view
            .next_focus_outside_table_retained(&self.tree, current, direction, table)
    }

    pub(crate) fn virtual_list_pins(
        &self,
        focus: Option<session::Focus>,
        targets: &[interaction::Target],
    ) -> std::collections::HashMap<interaction::Id, Vec<crate::list::Key>> {
        self.view
            .virtual_list_pins_retained(&self.tree, focus, targets)
    }

    pub(crate) fn virtual_list_model(&self, id: interaction::Id) -> Option<&crate::list::State> {
        self.view.virtual_list_model(id)
    }

    pub(crate) fn selectable_virtual_list_for_focus(
        &self,
        focus: session::Focus,
    ) -> Option<&crate::list::State> {
        self.view
            .selectable_virtual_list_for_focus(&self.tree, focus)
    }

    pub(crate) fn focus_action(&self, focus: &session::Focus) -> Option<view::Action> {
        self.view.focus_action_retained(focus, &self.tree)
    }

    pub(crate) fn subject_path_for_focus(&self, focus: Option<session::Focus>) -> subject::Path {
        focus
            .and_then(|focus| self.view.subject_path_for_focus_retained(focus, &self.tree))
            .unwrap_or_else(subject::Path::application)
    }

    pub(crate) fn node_is_self_or_descendant_of_element(
        &self,
        node_id: NodeId,
        element_id: interaction::Id,
    ) -> bool {
        let mut current = Some(node_id);
        while let Some(id) = current {
            let Some(node) = self.tree.node(id) else {
                return false;
            };
            if node.element_id() == Some(element_id) {
                return true;
            }
            current = node.parent();
        }

        false
    }

    pub(crate) fn context_path_for_node(&self, node_id: NodeId) -> Vec<view::ContextOwner> {
        self.view.context_path_retained(&self.tree, node_id)
    }

    pub(crate) fn provided_row_for_node(&self, node_id: NodeId) -> Option<view::ProvidedRow> {
        let mut current = Some(node_id);
        while let Some(id) = current {
            let node = self.tree.node(id)?;
            if let Some(row) = node.provided_row() {
                return Some(row);
            }
            current = node.parent();
        }
        None
    }
}
