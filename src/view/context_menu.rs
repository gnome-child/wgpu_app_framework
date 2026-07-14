use super::{Node, View};
use crate::{command, geometry, interaction};

pub(crate) struct ContextMenu {
    fingerprint: crate::popup::ContextFingerprint,
    anchor: geometry::Point,
    available: geometry::Rect,
    sections: Vec<Vec<command::ResolvedAction>>,
}

impl ContextMenu {
    pub(crate) fn new(
        owner: crate::composition::tree::NodeId,
        anchor: geometry::Point,
        available: geometry::Rect,
        sections: Vec<Vec<command::ResolvedAction>>,
    ) -> Self {
        Self {
            fingerprint: crate::popup::ContextFingerprint::from_owner(owner),
            anchor,
            available,
            sections,
        }
    }
}

impl View {
    pub(crate) fn project_context_menu(&mut self, menu: ContextMenu) {
        let mut panel = Node::floating_panel(interaction::Menu::context_id())
            .with_panel_placement(
                geometry::placement::Anchor::Point(menu.anchor),
                menu.available,
            )
            .with_popup_context(menu.fingerprint);
        for (index, section) in menu.sections.into_iter().enumerate() {
            if index != 0 {
                panel = panel.child(Node::separator());
            }
            for action in section {
                panel = panel.child(Node::resolved_menu_action(action));
            }
        }
        self.push_floating_panel(panel);
    }
}
