use super::{Node, View};
use crate::{command, geometry, interaction};

pub(crate) struct ContextMenu {
    anchor: geometry::Point,
    available: geometry::Rect,
    actions: Vec<command::ResolvedAction>,
}

impl ContextMenu {
    pub(crate) fn new(
        anchor: geometry::Point,
        available: geometry::Rect,
        actions: Vec<command::ResolvedAction>,
    ) -> Self {
        Self {
            anchor,
            available,
            actions,
        }
    }
}

impl View {
    pub(crate) fn project_context_menu(&mut self, menu: ContextMenu) {
        let mut panel = Node::floating_panel(interaction::Menu::context_id()).with_menu_placement(
            geometry::PlacementAnchor::Point(menu.anchor),
            menu.available,
        );
        for action in menu.actions {
            panel = panel.child(Node::resolved_menu_action(action));
        }
        self.root.push_child(panel);
    }
}
