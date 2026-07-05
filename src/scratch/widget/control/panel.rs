use crate::scratch::view;

use super::super::{Ui, Widget};

pub struct Panel {
    node: view::Node,
}

impl Panel {
    pub fn new() -> Self {
        Self {
            node: view::Node::panel(),
        }
    }

    pub fn child(mut self, child: impl Widget) -> Self {
        self.node = self.node.child(child.into_node());
        self
    }

    pub fn children(mut self, children: impl FnOnce(&mut Ui)) -> Self {
        let mut ui = Ui::new();
        children(&mut ui);
        for child in ui.into_nodes() {
            self.node = self.node.child(child);
        }
        self
    }
}

impl Default for Panel {
    fn default() -> Self {
        Self::new()
    }
}

impl Widget for Panel {
    fn into_node(self) -> view::Node {
        self.node
    }
}
