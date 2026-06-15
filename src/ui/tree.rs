use std::collections::HashMap;

use crate::geometry::area;
use crate::{action, layout, menu, paint, widget, window};

use super::{
    ActionTarget, Intent, Interaction, Interactivity, Node, Path, Popup, layout_engine, painting,
};

#[derive(Debug, Clone, PartialEq)]
pub struct Tree {
    root: Option<Node>,
    popups: Vec<Popup>,
}

impl Tree {
    pub fn new() -> Self {
        Self {
            root: None,
            popups: Vec::new(),
        }
    }

    pub fn set_root(&mut self, root: Node) {
        self.root = Some(root);
    }

    pub fn root(&self) -> Option<&Node> {
        self.root.as_ref()
    }

    pub fn root_mut(&mut self) -> Option<&mut Node> {
        self.root.as_mut()
    }

    pub fn push_popup(&mut self, popup: Popup) {
        self.popups.push(popup);
    }

    pub fn clear_popups(&mut self) {
        self.popups.clear();
    }

    pub fn popups(&self) -> &[Popup] {
        &self.popups
    }

    pub fn is_empty(&self) -> bool {
        self.root.is_none()
    }

    pub fn layout(&self, area: area::Logical) -> Option<layout::Box> {
        let root = self.root.as_ref()?;
        let root_layout = layout_engine::tree(root, area);
        if self.popups.is_empty() {
            return Some(root_layout);
        }

        let mut children = root_layout.children().to_vec();
        let root_path = root_layout.path().clone();
        for popup in &self.popups {
            children.push(layout_engine::subtree_at(
                popup.root(),
                root_path.child(popup.root().id()),
                popup.rect(),
            ));
        }

        Some(root_layout.with_children(children))
    }

    pub fn actions(&self) -> HashMap<Path, action::Id> {
        let mut actions = HashMap::new();

        if let Some(root) = self.root.as_ref() {
            collect_actions(root, &Path::root(root.id()), &mut actions);
            for popup in &self.popups {
                collect_actions(
                    popup.root(),
                    &Path::root(root.id()).child(popup.root().id()),
                    &mut actions,
                );
            }
        }

        actions
    }

    pub fn action_targets(&self) -> HashMap<Path, ActionTarget> {
        let mut targets = HashMap::new();

        if let Some(root) = self.root.as_ref() {
            collect_action_targets(root, &Path::root(root.id()), &mut targets);
            for popup in &self.popups {
                collect_action_targets(
                    popup.root(),
                    &Path::root(root.id()).child(popup.root().id()),
                    &mut targets,
                );
            }
        }

        targets
    }

    pub fn intents(&self) -> HashMap<Path, Intent> {
        let mut intents = HashMap::new();

        if let Some(root) = self.root.as_ref() {
            collect_intents(root, &Path::root(root.id()), &mut intents);
            for popup in &self.popups {
                collect_intents(
                    popup.root(),
                    &Path::root(root.id()).child(popup.root().id()),
                    &mut intents,
                );
            }
        }

        intents
    }

    pub fn menus(&self) -> HashMap<menu::Id, menu::Menu> {
        let mut menus = HashMap::new();

        if let Some(root) = self.root.as_ref() {
            collect_menus(root, &mut menus);
        }

        menus
    }

    pub fn responders(&self) -> HashMap<Path, Vec<action::Id>> {
        let mut responders = HashMap::new();

        if let Some(root) = self.root.as_ref() {
            collect_responders(root, &Path::root(root.id()), &mut responders);
            for popup in &self.popups {
                collect_responders(
                    popup.root(),
                    &Path::root(root.id()).child(popup.root().id()),
                    &mut responders,
                );
            }
        }

        responders
    }

    pub fn command_scopes(&self) -> Vec<Path> {
        let mut scopes = Vec::new();

        if let Some(root) = self.root.as_ref() {
            collect_command_scopes(root, &Path::root(root.id()), &mut scopes);
            for popup in &self.popups {
                collect_command_scopes(
                    popup.root(),
                    &Path::root(root.id()).child(popup.root().id()),
                    &mut scopes,
                );
            }
        }

        scopes
    }

    pub fn interactivity(&self) -> HashMap<Path, Interactivity> {
        let mut interactivity = HashMap::new();

        if let Some(root) = self.root.as_ref() {
            collect_interactivity(root, &Path::root(root.id()), &mut interactivity);
            for popup in &self.popups {
                collect_interactivity(
                    popup.root(),
                    &Path::root(root.id()).child(popup.root().id()),
                    &mut interactivity,
                );
            }
        }

        interactivity
    }

    pub fn widget_metrics(&self, layout: &layout::Box) -> HashMap<Path, widget::Metrics> {
        let mut metrics = HashMap::new();

        if let Some(root) = self.root.as_ref() {
            collect_widget_metrics(root, layout, &mut metrics);
            for popup in &self.popups {
                let path = Path::root(root.id()).child(popup.root().id());
                if let Some(popup_layout) = layout.find_path(&path) {
                    collect_widget_metrics(popup.root(), popup_layout, &mut metrics);
                }
            }
        }

        metrics
    }

    pub fn paint<T>(
        &self,
        layout: &layout::Box,
        actions: &action::Registry<T>,
        window: window::Id,
        interaction: Interaction,
        scene: &mut paint::Scene,
    ) {
        if let Some(root) = self.root.as_ref() {
            painting::tree(root, layout, actions, window, &interaction, scene);
            for popup in &self.popups {
                let path = layout.path().child(popup.root().id());
                if let Some(popup_layout) = layout.find_path(&path) {
                    painting::tree(
                        popup.root(),
                        popup_layout,
                        actions,
                        window,
                        &interaction,
                        scene,
                    );
                }
            }
        }
    }
}

impl Default for Tree {
    fn default() -> Self {
        Self::new()
    }
}

fn collect_actions(node: &Node, path: &Path, actions: &mut HashMap<Path, action::Id>) {
    if let Some(action) = node.action() {
        actions.insert(path.clone(), action);
    }

    for child in node.children() {
        collect_actions(child, &path.child(child.id()), actions);
    }
}

fn collect_action_targets(node: &Node, path: &Path, targets: &mut HashMap<Path, ActionTarget>) {
    if node.action().is_some() {
        targets.insert(path.clone(), node.action_target());
    }

    for child in node.children() {
        collect_action_targets(child, &path.child(child.id()), targets);
    }
}

fn collect_intents(node: &Node, path: &Path, intents: &mut HashMap<Path, Intent>) {
    if let Some(intent) = node.intent() {
        intents.insert(path.clone(), intent);
    }

    for child in node.children() {
        collect_intents(child, &path.child(child.id()), intents);
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

    for node in menu.sections().iter().flat_map(menu::Section::nodes) {
        if let menu::Node::Submenu(submenu) = node {
            collect_menu(submenu, menus);
        }
    }
}

fn collect_responders(node: &Node, path: &Path, responders: &mut HashMap<Path, Vec<action::Id>>) {
    if !node.responders().is_empty() {
        responders.insert(path.clone(), node.responders().to_vec());
    }

    for child in node.children() {
        collect_responders(child, &path.child(child.id()), responders);
    }
}

fn collect_command_scopes(node: &Node, path: &Path, scopes: &mut Vec<Path>) {
    if node.is_command_scope() {
        scopes.push(path.clone());
    }

    for child in node.children() {
        collect_command_scopes(child, &path.child(child.id()), scopes);
    }
}

fn collect_interactivity(
    node: &Node,
    path: &Path,
    interactivity: &mut HashMap<Path, Interactivity>,
) {
    interactivity.insert(path.clone(), node.interactivity());

    for child in node.children() {
        collect_interactivity(child, &path.child(child.id()), interactivity);
    }
}

fn collect_widget_metrics(
    node: &Node,
    layout: &layout::Box,
    metrics: &mut HashMap<Path, widget::Metrics>,
) {
    if let Some(scroll_metrics) = widget::scroll::metrics(node, layout) {
        metrics.insert(
            layout.path().clone(),
            widget::Metrics::Scroll(scroll_metrics),
        );
    }

    for (child, child_layout) in node.children().iter().zip(layout.children()) {
        collect_widget_metrics(child, child_layout, metrics);
    }
}
