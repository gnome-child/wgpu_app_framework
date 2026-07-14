use crate::{
    command, interaction,
    view::{self, node},
};

use super::{Ui, Widget};

pub struct MenuBar {
    node: view::Node,
}

/// An opt-in conventional menu bar with deliberate authored extensions.
///
/// Static commands belong in registration metadata. These methods are for
/// dynamic lists, argument-bearing bindings, submenus, and explicit group or
/// category replacement.
pub struct StandardMenuBar {
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

impl StandardMenuBar {
    pub fn new() -> Self {
        Self {
            node: view::Node::standard_menu_bar(),
        }
    }

    pub fn items_before(
        &mut self,
        anchor: command::Standard,
        children: impl FnOnce(&mut Ui),
    ) -> &mut Self {
        self.extend(node::standard_menu::Extension::items_before(
            anchor,
            extension_nodes(children),
        ))
    }

    pub fn items_after(
        &mut self,
        anchor: command::Standard,
        children: impl FnOnce(&mut Ui),
    ) -> &mut Self {
        self.extend(node::standard_menu::Extension::items_after(
            anchor,
            extension_nodes(children),
        ))
    }

    pub fn section_before(
        &mut self,
        anchor: command::Standard,
        children: impl FnOnce(&mut Ui),
    ) -> &mut Self {
        self.extend(node::standard_menu::Extension::section_before(
            anchor,
            extension_nodes(children),
        ))
    }

    pub fn section_after(
        &mut self,
        anchor: command::Standard,
        children: impl FnOnce(&mut Ui),
    ) -> &mut Self {
        self.extend(node::standard_menu::Extension::section_after(
            anchor,
            extension_nodes(children),
        ))
    }

    pub fn replace_section(
        &mut self,
        anchor: command::Standard,
        children: impl FnOnce(&mut Ui),
    ) -> &mut Self {
        self.extend(node::standard_menu::Extension::replace_section(
            anchor,
            extension_nodes(children),
        ))
    }

    pub fn append_section(
        &mut self,
        category: command::menu::Category,
        children: impl FnOnce(&mut Ui),
    ) -> &mut Self {
        self.extend(node::standard_menu::Extension::append_section(
            category,
            extension_nodes(children),
        ))
    }

    pub fn replace_category(
        &mut self,
        category: command::menu::Category,
        children: impl FnOnce(&mut Ui),
    ) -> &mut Self {
        self.extend(node::standard_menu::Extension::replace_category(
            category,
            extension_nodes(children),
        ))
    }

    fn extend(&mut self, extension: node::standard_menu::Extension) -> &mut Self {
        self.node.push_standard_menu_extension(extension);
        self
    }
}

impl Default for StandardMenuBar {
    fn default() -> Self {
        Self::new()
    }
}

impl Widget for MenuBar {
    fn into_node(self) -> view::Node {
        self.node
    }
}

impl Widget for StandardMenuBar {
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

fn extension_nodes(children: impl FnOnce(&mut Ui)) -> Vec<view::Node> {
    let mut ui = Ui::new();
    children(&mut ui);
    ui.into_nodes()
}
