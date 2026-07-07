use crate::{interaction, view};

use super::{Ui, Widget};

pub struct MenuBar {
    node: view::Node,
}

pub struct Menu {
    node: view::Node,
}

impl MenuBar {
    pub fn new() -> Self {
        Self {
            node: view::Node::menu_bar(),
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

impl Default for MenuBar {
    fn default() -> Self {
        Self::new()
    }
}

impl Widget for MenuBar {
    fn into_node(self) -> view::Node {
        self.node
    }
}

impl Menu {
    pub fn new(id: impl Into<interaction::Id>, label: impl Into<String>) -> Self {
        Self {
            node: view::Node::menu(id, label),
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

impl Widget for Menu {
    fn into_node(self) -> view::Node {
        self.node
    }
}
