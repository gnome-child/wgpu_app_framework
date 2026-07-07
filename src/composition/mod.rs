mod store;
mod tree;

pub(crate) use store::Store;
pub use tree::NodeId;
pub(crate) use tree::{Changes, Node, Tree};

use super::{interaction, session, subject, view, window};

pub struct Composition {
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

    pub(crate) fn project_transient_state(
        &mut self,
        interaction: Option<&interaction::Interaction>,
        focus: Option<session::Focus>,
    ) {
        let tree = self.tree.clone();
        if let Some(interaction) = interaction {
            self.view.project_interaction_retained(interaction, &tree);
        }
        self.view.project_focus_retained(focus, &tree);
    }

    pub(crate) fn tree(&self) -> &Tree {
        &self.tree
    }

    #[cfg(test)]
    pub(crate) fn changes(&self) -> &Changes {
        &self.changes
    }

    pub fn contains_focus(&self, focus: session::Focus) -> bool {
        self.view.contains_focus(focus)
    }

    pub(crate) fn next_focus(
        &self,
        current: Option<session::Focus>,
        direction: view::action::FocusDirection,
    ) -> Option<session::Focus> {
        self.view
            .next_focus_retained(&self.tree, current, direction)
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
}
