use std::collections::HashMap;

use crate::geometry::area;
use crate::{action, layout, paint, window};

use super::{ActionTarget, Interaction, Interactivity, Node, Path, layout_engine, painting};

#[derive(Debug, Clone, PartialEq)]
pub struct Tree {
    root: Option<Node>,
}

impl Tree {
    pub fn new() -> Self {
        Self { root: None }
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

    pub fn is_empty(&self) -> bool {
        self.root.is_none()
    }

    pub fn layout(&self, area: area::Logical) -> Option<layout::Box> {
        self.root
            .as_ref()
            .map(|root| layout_engine::tree(root, area))
    }

    pub fn actions(&self) -> HashMap<Path, action::Id> {
        let mut actions = HashMap::new();

        if let Some(root) = self.root.as_ref() {
            collect_actions(root, &Path::root(root.id()), &mut actions);
        }

        actions
    }

    pub fn action_targets(&self) -> HashMap<Path, ActionTarget> {
        let mut targets = HashMap::new();

        if let Some(root) = self.root.as_ref() {
            collect_action_targets(root, &Path::root(root.id()), &mut targets);
        }

        targets
    }

    pub fn responders(&self) -> HashMap<Path, Vec<action::Id>> {
        let mut responders = HashMap::new();

        if let Some(root) = self.root.as_ref() {
            collect_responders(root, &Path::root(root.id()), &mut responders);
        }

        responders
    }

    pub fn interactivity(&self) -> HashMap<Path, Interactivity> {
        let mut interactivity = HashMap::new();

        if let Some(root) = self.root.as_ref() {
            collect_interactivity(root, &Path::root(root.id()), &mut interactivity);
        }

        interactivity
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
            painting::tree(root, layout, actions, window, interaction, scene);
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

fn collect_responders(node: &Node, path: &Path, responders: &mut HashMap<Path, Vec<action::Id>>) {
    if !node.responders().is_empty() {
        responders.insert(path.clone(), node.responders().to_vec());
    }

    for child in node.children() {
        collect_responders(child, &path.child(child.id()), responders);
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
