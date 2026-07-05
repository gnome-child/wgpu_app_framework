use std::collections::HashMap;

use crate::widget::menu;
use crate::{action, text};

use super::tree::Tree;
use super::{Cursor, Intent, Interactivity, Node, Path};

#[derive(Debug, Clone, Default, PartialEq)]
pub(crate) struct Snapshot {
    pub(crate) menus: HashMap<menu::Id, menu::Menu>,
    pub(crate) actions: HashMap<Path, action::Route>,
    pub(crate) action_subjects: HashMap<Path, action::Subject>,
    pub(crate) intents: HashMap<Path, Intent>,
    pub(crate) responders: HashMap<Path, Vec<action::Key>>,
    pub(crate) responder_bindings: HashMap<Path, Vec<action::Binding>>,
    pub(crate) action_targets: HashMap<Path, Vec<action::Target>>,
    pub(crate) action_scopes: Vec<Path>,
    pub(crate) text_fields: HashMap<Path, text::Field>,
    pub(crate) text_surfaces: HashMap<Path, text::Surface>,
    pub(crate) interactivity: HashMap<Path, Interactivity>,
    pub(crate) cursors: HashMap<Path, Cursor>,
}

impl Snapshot {
    pub(crate) fn from_tree(tree: &Tree) -> Self {
        let mut snapshot = Self::default();

        if let Some(root) = tree.root() {
            collect_menus(root, &mut snapshot.menus);
            snapshot.collect_node(root, &Path::root(root.path_id(0)));
            for (popup_index, popup) in tree.popups().iter().enumerate() {
                snapshot.collect_node(
                    popup.root(),
                    &Path::root(root.path_id(0)).child(popup.root().path_id(popup_index)),
                );
            }
        }

        snapshot
    }

    fn collect_node(&mut self, node: &Node, path: &Path) {
        if let Some(route) = node.action_route() {
            self.actions.insert(path.clone(), route);
            self.action_subjects
                .insert(path.clone(), node.action_subject());
        }

        if let Some(intent) = node.intent() {
            self.intents.insert(path.clone(), intent);
        }

        if !node.responders().is_empty() {
            self.responders
                .insert(path.clone(), node.responders().to_vec());
        }

        if !node.responder_bindings().is_empty() {
            self.responder_bindings
                .insert(path.clone(), node.responder_bindings().to_vec());
        }

        if !node.action_targets().is_empty() {
            self.action_targets
                .insert(path.clone(), node.action_targets().to_vec());
        }

        if node.is_action_scope() {
            self.action_scopes.push(path.clone());
        }

        if let Some(surface) = node.text_surface() {
            if let Some(field) = surface.as_field() {
                self.text_fields.insert(path.clone(), field.clone());
            }
            self.text_surfaces.insert(path.clone(), surface.clone());
        }

        self.interactivity
            .insert(path.clone(), node.interactivity());
        self.cursors.insert(path.clone(), node.cursor());

        for (index, child) in node.children().iter().enumerate() {
            self.collect_node(child, &path.child(child.path_id(index)));
        }
    }
}

fn collect_menus(node: &Node, menus: &mut HashMap<menu::Id, menu::Menu>) {
    if let Some(bar) = node.menu_bar() {
        for menu in bar.menus() {
            collect_menu(menu, menus);
        }
    }

    for child in node.children() {
        collect_menus(child, menus);
    }
}

fn collect_menu(menu: &menu::Menu, menus: &mut HashMap<menu::Id, menu::Menu>) {
    menus.insert(menu.id(), menu.clone());
    for section in menu.sections() {
        for node in section.nodes() {
            if let menu::Node::Submenu(menu) = node {
                collect_menu(menu, menus);
            }
        }
    }
}
