use super::{Node, View};
use crate::{command, geometry, interaction, view};

pub(crate) struct ContextMenu {
    anchor: geometry::Point,
    actions: Vec<command::ResolvedAction>,
}

impl ContextMenu {
    pub(crate) fn new(anchor: geometry::Point, actions: Vec<command::ResolvedAction>) -> Self {
        Self { anchor, actions }
    }
}

impl View {
    pub(crate) fn project_context_menu(&mut self, menu: ContextMenu) {
        let mut panel = Node::floating_panel(interaction::Menu::context_id())
            .with_floating_placement(view::FloatingPlacement::Offset {
                x: menu.anchor.x(),
                y: menu.anchor.y(),
            });
        for action in menu.actions {
            panel = panel.child(Node::resolved_menu_action(action));
        }
        self.root.push_child(panel);
    }
}
