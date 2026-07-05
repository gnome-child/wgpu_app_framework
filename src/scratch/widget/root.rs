use crate::scratch::view;

use super::{Ui, Widget};

pub struct Root {
    node: view::Node,
}

impl Root {
    pub fn new() -> Self {
        Self {
            node: view::Node::root(),
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

impl Default for Root {
    fn default() -> Self {
        Self::new()
    }
}

impl Widget for Root {
    fn into_node(self) -> view::Node {
        self.node
    }
}
