use std::collections::HashSet;

use super::super::{interaction, subject, view};

/// Process-transient identity for a composition node.
///
/// The namespace distinguishes retained composition identity from view-only
/// layout identity, so the two cannot collide.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct NodeId {
    space: Space,
    value: u64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
enum Space {
    Retained,
    Layout,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Changes {
    added: Vec<NodeId>,
    removed: Vec<NodeId>,
    removed_elements: Vec<interaction::Id>,
}

/// Retained composition tree for one installed view.
///
/// Reconciliation is deliberately local in v1: explicit ids survive sibling
/// reordering under the same parent, id-less nodes are positional, and moving a
/// node to a different parent is reported as remove plus add.
#[derive(Debug, Clone)]
pub struct Tree {
    root: Node,
}

#[derive(Debug, Clone)]
pub struct Node {
    id: Identity,
    key: Key,
    element_id: Option<interaction::Id>,
    subject: Option<subject::Segment>,
    parent: Option<Identity>,
    children: Vec<Node>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum Identity {
    Retained(NodeId),
    Layout(NodeId),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
struct Key {
    role: view::node::Role,
    axis: Option<view::node::Axis>,
}

impl NodeId {
    pub(in crate::composition) fn next(next: &mut u64) -> Self {
        let id = Self {
            space: Space::Retained,
            value: *next,
        };
        *next = next.saturating_add(1);
        id
    }

    pub(in crate::composition) fn layout(next: &mut u64) -> Self {
        let id = Self {
            space: Space::Layout,
            value: *next,
        };
        *next = next.saturating_add(1);
        id
    }

    pub fn is_retained(self) -> bool {
        self.space == Space::Retained
    }
}

impl Identity {
    fn node_id(self) -> NodeId {
        match self {
            Self::Retained(id) | Self::Layout(id) => id,
        }
    }

    fn retained_id(self) -> Option<NodeId> {
        match self {
            Self::Retained(id) => Some(id),
            Self::Layout(_) => None,
        }
    }
}

impl Changes {
    fn empty() -> Self {
        Self {
            added: Vec::new(),
            removed: Vec::new(),
            removed_elements: Vec::new(),
        }
    }

    pub fn added(&self) -> &[NodeId] {
        &self.added
    }

    pub fn removed(&self) -> &[NodeId] {
        &self.removed
    }

    pub fn removed_elements(&self) -> &[interaction::Id] {
        &self.removed_elements
    }

    pub fn is_empty(&self) -> bool {
        self.added.is_empty() && self.removed.is_empty()
    }

    fn add_added(&mut self, id: NodeId) {
        self.added.push(id);
    }

    fn add_removed_subtree(&mut self, node: &Node) {
        if let Some(id) = node.id.retained_id() {
            self.removed.push(id);
        }
        if let Some(id) = node.element_id {
            self.removed_elements.push(id);
        }
        for child in &node.children {
            self.add_removed_subtree(child);
        }
    }
}

impl Default for Changes {
    fn default() -> Self {
        Self::empty()
    }
}

impl Tree {
    pub(crate) fn new(view: &view::View, next_node_id: &mut u64) -> (Self, Changes) {
        let mut changes = Changes::empty();
        let root = Node::build_retained(view.root(), None, next_node_id, &mut changes);
        (Self { root }, changes)
    }

    pub(crate) fn layout(view: &view::View) -> Self {
        let mut next_id = 1;
        let root = Node::build_layout(view.root(), None, &mut next_id);
        Self { root }
    }

    pub(crate) fn reconcile(&self, view: &view::View, next_node_id: &mut u64) -> (Self, Changes) {
        let mut changes = Changes::empty();
        let root = Node::reconcile(
            Some(&self.root),
            view.root(),
            None,
            next_node_id,
            &mut changes,
        );
        (Self { root }, changes)
    }

    pub fn root(&self) -> &Node {
        &self.root
    }

    pub(crate) fn node(&self, id: NodeId) -> Option<&Node> {
        self.root.find(id)
    }
}

impl Node {
    fn build_retained(
        view: &view::Node,
        parent: Option<Identity>,
        next_node_id: &mut u64,
        changes: &mut Changes,
    ) -> Self {
        let id = Identity::Retained(NodeId::next(next_node_id));
        changes.add_added(
            id.retained_id()
                .expect("retained identity should be retained"),
        );
        let mut node = Self::new(id, view, parent);
        node.children = view
            .children()
            .iter()
            .map(|child| Self::build_retained(child, Some(id), next_node_id, changes))
            .collect();
        node
    }

    fn build_layout(view: &view::Node, parent: Option<Identity>, next_id: &mut u64) -> Self {
        let id = Identity::Layout(NodeId::layout(next_id));
        let mut node = Self::new(id, view, parent);
        node.children = view
            .children()
            .iter()
            .map(|child| Self::build_layout(child, Some(id), next_id))
            .collect();
        node
    }

    fn reconcile(
        old: Option<&Node>,
        view: &view::Node,
        parent: Option<Identity>,
        next_node_id: &mut u64,
        changes: &mut Changes,
    ) -> Self {
        let old = match old {
            Some(old) if old.matches(view) => Some(old),
            Some(old) => {
                changes.add_removed_subtree(old);
                None
            }
            None => None,
        };
        let id = old.map(|old| old.id).unwrap_or_else(|| {
            let id = Identity::Retained(NodeId::next(next_node_id));
            changes.add_added(
                id.retained_id()
                    .expect("retained identity should be retained"),
            );
            id
        });
        let mut node = Self::new(id, view, parent);
        let mut used_old = HashSet::new();

        for (index, child) in view.children().iter().enumerate() {
            let old_child = old
                .and_then(|old| old.match_child(index, child, &used_old))
                .inspect(|child| {
                    used_old.insert(child.id);
                });
            node.children.push(Self::reconcile(
                old_child,
                child,
                Some(id),
                next_node_id,
                changes,
            ));
        }

        if let Some(old) = old {
            for child in &old.children {
                if !used_old.contains(&child.id) {
                    changes.add_removed_subtree(child);
                }
            }
        }

        node
    }

    fn new(id: Identity, view: &view::Node, parent: Option<Identity>) -> Self {
        Self {
            id,
            key: Key::for_view(view),
            element_id: view.id(),
            subject: subject_for(view),
            parent,
            children: Vec::new(),
        }
    }

    fn matches(&self, view: &view::Node) -> bool {
        self.key == Key::for_view(view) && self.element_id == view.id()
    }

    fn match_child<'a>(
        &'a self,
        index: usize,
        view: &view::Node,
        used: &HashSet<Identity>,
    ) -> Option<&'a Node> {
        if let Some(id) = view.id() {
            return self
                .children
                .iter()
                .filter(|child| !used.contains(&child.id))
                .find(|child| child.element_id == Some(id) && child.matches(view));
        }

        self.children
            .get(index)
            .filter(|child| !used.contains(&child.id) && child.element_id.is_none())
            .filter(|child| child.matches(view))
    }

    pub fn retained_id(&self) -> Option<NodeId> {
        self.id.retained_id()
    }

    pub(crate) fn node_id(&self) -> NodeId {
        self.id.node_id()
    }

    pub fn element_id(&self) -> Option<interaction::Id> {
        self.element_id
    }

    pub fn parent(&self) -> Option<NodeId> {
        self.parent.and_then(Identity::retained_id)
    }

    pub fn subject(&self) -> Option<&subject::Segment> {
        self.subject.as_ref()
    }

    pub fn children(&self) -> &[Node] {
        &self.children
    }

    fn find(&self, id: NodeId) -> Option<&Node> {
        if self.id.node_id() == id {
            return Some(self);
        }

        self.children.iter().find_map(|child| child.find(id))
    }
}

impl Key {
    fn for_view(view: &view::Node) -> Self {
        Self {
            role: view.role(),
            axis: view.axis(),
        }
    }
}

fn subject_for(view: &view::Node) -> Option<subject::Segment> {
    if let Some(subject) = view.subject() {
        return Some(subject.clone());
    }

    match view.role() {
        view::node::Role::Root => Some(subject::Segment::application()),
        view::node::Role::TextArea => view
            .id()
            .or_else(|| {
                view.text_area_model()
                    .and_then(view::control::TextArea::focus)
                    .and_then(|focus| focus.target_id())
            })
            .map(segment_from_id),
        view::node::Role::TextBox => view
            .text_box_model()
            .and_then(view::control::TextBox::focus)
            .and_then(|focus| focus.target_id())
            .map(segment_from_id),
        view::node::Role::Menu | view::node::Role::FloatingPanel | view::node::Role::Panel => {
            view.label_text().map(subject::Segment::from_label)
        }
        view::node::Role::Scroll => None,
        view::node::Role::Stack
        | view::node::Role::MenuBar
        | view::node::Role::Binding
        | view::node::Role::Separator
        | view::node::Role::Button
        | view::node::Role::Checkbox
        | view::node::Role::Radio
        | view::node::Role::Slider
        | view::node::Role::SectionHeader
        | view::node::Role::Label => None,
    }
}

fn segment_from_id(id: interaction::Id) -> subject::Segment {
    let name = id.as_str().rsplit('.').next().unwrap_or(id.as_str());
    subject::Segment::from_name(name)
}
