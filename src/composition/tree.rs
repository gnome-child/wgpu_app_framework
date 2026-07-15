use std::collections::HashSet;

use super::super::{interaction, subject, view};

/// Process-transient identity for a composition node.
///
/// The namespace distinguishes retained composition identity from view-only
/// layout identity, so the two cannot collide.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub(crate) struct NodeId {
    space: Space,
    value: u64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub(crate) struct ContentRevision(u64);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
enum Space {
    Retained,
    #[cfg(test)]
    Layout,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct Changes {
    added: Vec<NodeId>,
    changed: Vec<NodeId>,
    removed: Vec<NodeId>,
    removed_elements: Vec<interaction::Id>,
    removed_table_cells: Vec<crate::table::Cell>,
}

/// Retained composition tree for one installed view.
///
/// Reconciliation is deliberately local in v1: explicit ids survive sibling
/// reordering under the same parent, id-less nodes are positional, and moving a
/// node to a different parent is reported as remove plus add.
#[derive(Debug, Clone)]
pub(crate) struct Tree {
    root: Node,
}

#[cfg(test)]
#[derive(Debug, Clone)]
pub(crate) struct Layout {
    root: Node,
}

#[derive(Debug, Clone)]
pub(crate) struct Node {
    id: NodeId,
    content_revision: ContentRevision,
    /// Authored scene state used when reconciling a freshly rebuilt view.
    scene_key: view::node::SceneKey,
    /// The last transiently projected scene state consumed by layout/paint.
    projected_scene_key: view::node::SceneKey,
    key: Key,
    element_id: Option<interaction::Id>,
    subject: Option<subject::Segment>,
    provided_row: Option<view::ProvidedRow>,
    parent: Option<NodeId>,
    children: Vec<Node>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum Key {
    Ordinary {
        role: view::Role,
        axis: Option<view::Axis>,
    },
    ProvidedRow {
        role: view::Role,
        axis: Option<view::Axis>,
        key: crate::virtual_list::Key,
    },
    TableCell {
        role: view::Role,
        axis: Option<view::Axis>,
        cell: crate::table::Cell,
    },
    TableHeaderCell {
        role: view::Role,
        axis: Option<view::Axis>,
        cell: crate::table::HeaderCell,
    },
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

    #[cfg(feature = "renderer-debug")]
    pub(crate) fn renderer_fixture(value: u64) -> Self {
        assert!(
            value > 0,
            "renderer fixtures require nonzero composition identity"
        );
        Self {
            space: Space::Retained,
            value,
        }
    }

    #[cfg(test)]
    pub(crate) fn layout(next: &mut u64) -> Self {
        let id = Self {
            space: Space::Layout,
            value: *next,
        };
        *next = next.saturating_add(1);
        id
    }

    #[cfg(test)]
    pub(crate) fn is_retained(self) -> bool {
        self.space == Space::Retained
    }
}

impl ContentRevision {
    pub(crate) const INITIAL: Self = Self(1);

    #[cfg(feature = "renderer-debug")]
    pub(crate) const fn renderer_fixture(value: u64) -> Self {
        Self(if value == 0 { 1 } else { value })
    }

    fn next(self) -> Self {
        Self(self.0.saturating_add(1))
    }

    #[cfg(test)]
    pub(crate) const fn get(self) -> u64 {
        self.0
    }
}

impl Changes {
    fn empty() -> Self {
        Self {
            added: Vec::new(),
            changed: Vec::new(),
            removed: Vec::new(),
            removed_elements: Vec::new(),
            removed_table_cells: Vec::new(),
        }
    }

    pub(crate) fn added(&self) -> &[NodeId] {
        &self.added
    }

    pub(crate) fn changed(&self) -> &[NodeId] {
        &self.changed
    }

    pub(crate) fn removed(&self) -> &[NodeId] {
        &self.removed
    }

    pub(crate) fn removed_elements(&self) -> &[interaction::Id] {
        &self.removed_elements
    }

    pub(crate) fn removed_table_cells(&self) -> &[crate::table::Cell] {
        &self.removed_table_cells
    }

    pub(crate) fn is_empty(&self) -> bool {
        self.added.is_empty() && self.changed.is_empty() && self.removed.is_empty()
    }

    fn add_added(&mut self, id: NodeId) {
        self.added.push(id);
    }

    fn add_changed(&mut self, id: NodeId) {
        if !self.added.contains(&id) && !self.changed.contains(&id) {
            self.changed.push(id);
        }
    }

    fn add_removed_subtree(&mut self, node: &Node) {
        self.removed.push(node.id);
        if let Some(id) = node.element_id {
            self.removed_elements.push(id);
        }
        if let Some(cell) = node.key.table_cell() {
            self.removed_table_cells.push(cell);
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

    pub(crate) fn reconcile(&self, view: &view::View, next_node_id: &mut u64) -> (Self, Changes) {
        let mut changes = Changes::empty();
        let root = Node::reconcile(
            Some(&self.root),
            view.root(),
            None,
            next_node_id,
            &mut changes,
        );
        changes
            .removed_table_cells
            .retain(|cell| !root.contains_table_cell(*cell));
        (Self { root }, changes)
    }

    pub(crate) fn root(&self) -> &Node {
        &self.root
    }

    pub(crate) fn node(&self, id: NodeId) -> Option<&Node> {
        self.root.find(id)
    }

    /// Advances content revisions for transient state projected into the
    /// installed view without rebuilding its authored structure.
    pub(crate) fn project_scene_state(&mut self, view: &view::View, changes: &mut Changes) {
        self.root.project_scene_state(view.root(), changes);
    }
}

#[cfg(test)]
impl Layout {
    pub(crate) fn new(view: &view::View) -> Self {
        let mut next_id = 1;
        let root = Node::build_layout(view.root(), None, &mut next_id);
        Self { root }
    }

    pub(crate) fn root(&self) -> &Node {
        &self.root
    }
}

impl Node {
    fn build_retained(
        view: &view::Node,
        parent: Option<NodeId>,
        next_node_id: &mut u64,
        changes: &mut Changes,
    ) -> Self {
        let id = NodeId::next(next_node_id);
        changes.add_added(id);
        let mut node = Self::new(id, view, parent, ContentRevision::INITIAL);
        node.children = view
            .children()
            .iter()
            .map(|child| Self::build_retained(child, Some(id), next_node_id, changes))
            .collect();
        node
    }

    #[cfg(test)]
    fn build_layout(view: &view::Node, parent: Option<NodeId>, next_id: &mut u64) -> Self {
        let id = NodeId::layout(next_id);
        let mut node = Self::new(id, view, parent, ContentRevision::INITIAL);
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
        parent: Option<NodeId>,
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
            let id = NodeId::next(next_node_id);
            changes.add_added(id);
            id
        });
        let content_revision = match old {
            Some(old) if old.scene_key == view.scene_key() => old.content_revision,
            Some(old) => {
                changes.add_changed(id);
                old.content_revision.next()
            }
            None => ContentRevision::INITIAL,
        };
        let mut node = Self::new(id, view, parent, content_revision);
        if let Some(old) = old {
            node.projected_scene_key = old.projected_scene_key.clone();
        }
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
            let materialized_keys = view
                .children()
                .iter()
                .filter_map(view::Node::provided_row)
                .map(view::ProvidedRow::key)
                .collect::<HashSet<_>>();
            for child in &old.children {
                if !used_old.contains(&child.id) {
                    let dematerialized = child.key.provided_row().is_some_and(|key| {
                        !materialized_keys.contains(&key)
                            && view
                                .virtual_list_model()
                                .is_some_and(|model| model.contains_key(key))
                    });
                    if dematerialized {
                        continue;
                    }
                    changes.add_removed_subtree(child);
                }
            }
        }

        node
    }

    fn new(
        id: NodeId,
        view: &view::Node,
        parent: Option<NodeId>,
        content_revision: ContentRevision,
    ) -> Self {
        let scene_key = view.scene_key();
        Self {
            id,
            content_revision,
            scene_key: scene_key.clone(),
            projected_scene_key: scene_key,
            key: Key::for_view(view),
            element_id: element_id_for(view),
            subject: subject_for(view),
            provided_row: view.provided_row(),
            parent,
            children: Vec::new(),
        }
    }

    fn project_scene_state(&mut self, view: &view::Node, changes: &mut Changes) {
        debug_assert!(self.matches(view));
        debug_assert_eq!(self.children.len(), view.children().len());

        let projected = view.scene_key();
        if self.projected_scene_key != projected {
            self.projected_scene_key = projected;
            self.content_revision = self.content_revision.next();
            changes.add_changed(self.id);
        }

        for (child, view) in self.children.iter_mut().zip(view.children()) {
            child.project_scene_state(view, changes);
        }
    }

    fn matches(&self, view: &view::Node) -> bool {
        self.key == Key::for_view(view) && self.element_id == element_id_for(view)
    }

    fn match_child<'a>(
        &'a self,
        index: usize,
        view: &view::Node,
        used: &HashSet<NodeId>,
    ) -> Option<&'a Node> {
        if let Some(cell) = view.table_cell() {
            return self
                .children
                .iter()
                .filter(|child| !used.contains(&child.id))
                .find(|child| child.key.table_cell() == Some(cell) && child.matches(view));
        }
        if let Some(cell) = view.table_header_cell() {
            return self
                .children
                .iter()
                .filter(|child| !used.contains(&child.id))
                .find(|child| child.key.table_header_cell() == Some(cell) && child.matches(view));
        }
        if let Some(row) = view.provided_row() {
            return self
                .children
                .iter()
                .filter(|child| !used.contains(&child.id))
                .find(|child| child.key.provided_row() == Some(row.key()) && child.matches(view));
        }

        if let Some(id) = element_id_for(view) {
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

    pub(crate) fn node_id(&self) -> NodeId {
        self.id
    }

    pub(crate) fn content_revision(&self) -> ContentRevision {
        self.content_revision
    }

    pub(crate) fn element_id(&self) -> Option<interaction::Id> {
        self.element_id
    }

    pub(crate) fn parent(&self) -> Option<NodeId> {
        self.parent
    }

    pub(crate) fn subject(&self) -> Option<&subject::Segment> {
        self.subject.as_ref()
    }

    pub(crate) fn provided_row(&self) -> Option<view::ProvidedRow> {
        self.provided_row
    }

    pub(crate) fn children(&self) -> &[Node] {
        &self.children
    }

    fn find(&self, id: NodeId) -> Option<&Node> {
        if self.id == id {
            return Some(self);
        }

        self.children.iter().find_map(|child| child.find(id))
    }

    fn contains_table_cell(&self, cell: crate::table::Cell) -> bool {
        self.key.table_cell() == Some(cell)
            || self
                .children
                .iter()
                .any(|child| child.contains_table_cell(cell))
    }
}

fn element_id_for(view: &view::Node) -> Option<interaction::Id> {
    view.id()
        .or_else(|| {
            view.text_area_model()
                .and_then(view::TextArea::focus)
                .and_then(|focus| focus.target_id())
        })
        .or_else(|| {
            view.text_box_model()
                .and_then(view::TextBox::focus)
                .and_then(|focus| focus.target_id())
        })
}

impl Key {
    fn for_view(view: &view::Node) -> Self {
        let role = view.role();
        let axis = view.axis();
        if let Some(cell) = view.table_cell() {
            Self::TableCell { role, axis, cell }
        } else if let Some(cell) = view.table_header_cell() {
            Self::TableHeaderCell { role, axis, cell }
        } else if let Some(row) = view.provided_row() {
            Self::ProvidedRow {
                role,
                axis,
                key: row.key(),
            }
        } else {
            Self::Ordinary { role, axis }
        }
    }

    fn provided_row(self) -> Option<crate::virtual_list::Key> {
        match self {
            Self::ProvidedRow { key, .. } => Some(key),
            Self::Ordinary { .. } | Self::TableCell { .. } | Self::TableHeaderCell { .. } => None,
        }
    }

    fn table_cell(self) -> Option<crate::table::Cell> {
        match self {
            Self::TableCell { cell, .. } => Some(cell),
            Self::Ordinary { .. } | Self::ProvidedRow { .. } | Self::TableHeaderCell { .. } => None,
        }
    }

    fn table_header_cell(self) -> Option<crate::table::HeaderCell> {
        match self {
            Self::TableHeaderCell { cell, .. } => Some(cell),
            Self::Ordinary { .. } | Self::ProvidedRow { .. } | Self::TableCell { .. } => None,
        }
    }
}

fn subject_for(view: &view::Node) -> Option<subject::Segment> {
    if let Some(subject) = view.subject() {
        return Some(subject.clone());
    }

    match view.role() {
        view::Role::Root => Some(subject::Segment::application()),
        view::Role::TextArea => view
            .id()
            .or_else(|| {
                view.text_area_model()
                    .and_then(view::TextArea::focus)
                    .and_then(|focus| focus.target_id())
            })
            .map(segment_from_id),
        view::Role::TextBox => view
            .text_box_model()
            .and_then(view::TextBox::focus)
            .and_then(|focus| focus.target_id())
            .map(segment_from_id),
        view::Role::Menu | view::Role::FloatingPanel | view::Role::Panel => {
            view.label_text().map(subject::Segment::from_label)
        }
        view::Role::Scroll | view::Role::VirtualList | view::Role::Table => None,
        view::Role::Stack
        | view::Role::MenuBar
        | view::Role::Binding
        | view::Role::Separator
        | view::Role::Button
        | view::Role::Checkbox
        | view::Role::Radio
        | view::Role::Slider
        | view::Role::SectionHeader
        | view::Role::Label => None,
    }
}

fn segment_from_id(id: interaction::Id) -> subject::Segment {
    let name = id.as_str().rsplit('.').next().unwrap_or(id.as_str());
    subject::Segment::from_name(name)
}
