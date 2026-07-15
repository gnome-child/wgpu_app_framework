use std::{
    collections::{HashMap, HashSet},
    sync::Arc,
};

use thiserror::Error;

use super::super::{composition, geometry};
#[cfg(feature = "renderer-debug")]
use super::{Brush, Glass, Material, Offset, Rasterization, Rounding, Style, TextWrap};
use super::{
    Clip, Color, Group, Icon, Outline, Pane, Primitive, Quad, Rule, Scene, Shadow, Text,
    TextViewport, Transform, region::MaterialRegion,
};

macro_rules! revision_currency {
    ($name:ident) => {
        #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
        pub(crate) struct $name(u64);

        impl $name {
            pub(crate) const INITIAL: Self = Self(1);
        }
    };
}

revision_currency!(Revision);
revision_currency!(GeometryRevision);
revision_currency!(TopologyRevision);
revision_currency!(PropertySerial);

impl Revision {
    fn next(self) -> Self {
        Self(self.0.saturating_add(1))
    }

    #[cfg(feature = "renderer-debug")]
    const fn renderer_fixture(value: u64) -> Self {
        Self(if value == 0 { 1 } else { value })
    }
}

impl GeometryRevision {
    fn next(self) -> Self {
        Self(self.0.saturating_add(1))
    }
}

impl TopologyRevision {
    fn next(self) -> Self {
        Self(self.0.saturating_add(1))
    }
}

#[derive(Debug, Clone, PartialEq)]
pub(crate) struct Commit {
    revision: Revision,
    size: geometry::Size,
    clear: Color,
    nodes: Vec<Arc<Node>>,
    property_topology: Vec<PropertyRef>,
    order: Option<Vec<Draw>>,
    material_regions: Vec<MaterialRegion>,
}

#[derive(Debug, Clone, PartialEq)]
pub(crate) struct Node {
    id: composition::tree::NodeId,
    parent: Option<composition::tree::NodeId>,
    content_revision: composition::tree::ContentRevision,
    geometry_revision: GeometryRevision,
    topology_revision: TopologyRevision,
    local_bounds: geometry::Rect,
    content: Vec<Content>,
    properties: Vec<PropertyKind>,
    clip: Option<Clip>,
    opacity: OpacityDeclaration,
    effect: EffectDeclaration,
}

#[derive(Debug, Clone, PartialEq)]
pub(crate) enum Draw {
    Content {
        node: composition::tree::NodeId,
        index: usize,
    },
    PushClip {
        node: Option<composition::tree::NodeId>,
        clip: Clip,
    },
    PopClip,
    PushGroup {
        node: composition::tree::NodeId,
        bounds: geometry::Rect,
        opacity: f32,
    },
    PopGroup,
}

pub(crate) struct Builder {
    size: geometry::Size,
    clear: Color,
    nodes: Vec<NodeDraft>,
    node_indices: HashMap<composition::tree::NodeId, usize>,
    order: Vec<Draw>,
    material_regions: Vec<MaterialRegion>,
}

struct NodeDraft {
    id: composition::tree::NodeId,
    parent: Option<composition::tree::NodeId>,
    content_revision: composition::tree::ContentRevision,
    bounds: geometry::Rect,
    content: Vec<Content>,
}

#[derive(Debug, Clone, PartialEq)]
pub(crate) enum Content {
    Quad(Quad),
    Rule(Rule),
    Text(Text),
    TextViewport(TextViewport),
    Icon(Icon),
    Shadow(Shadow),
    Pane(Pane),
    Outline(Outline),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub(crate) enum PropertyKind {
    Transform,
    ScrollOffset,
    Opacity,
    Clip,
    Blur,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub(crate) struct PropertyRef {
    node: composition::tree::NodeId,
    kind: PropertyKind,
}

#[derive(Debug, Clone, PartialEq)]
pub(crate) struct Properties {
    commit: Revision,
    serial: PropertySerial,
    values: Vec<PropertyValue>,
    changed: Vec<PropertyRef>,
}

#[derive(Debug, Clone, Copy, PartialEq)]
#[allow(
    dead_code,
    reason = "blur values are contract-admitted and enter with retained effect realization"
)]
pub(crate) enum PropertyValue {
    Transform {
        node: composition::tree::NodeId,
        value: Transform,
    },
    ScrollOffset {
        node: composition::tree::NodeId,
        x: f32,
        y: f32,
    },
    Opacity {
        node: composition::tree::NodeId,
        value: f32,
    },
    Clip {
        node: composition::tree::NodeId,
        rect: geometry::Rect,
    },
    Blur {
        node: composition::tree::NodeId,
        sigma: f32,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(not(feature = "renderer-debug"), allow(dead_code))]
pub(crate) enum OpacityDeclaration {
    Opaque,
    Blended,
    Variable,
}

#[derive(Debug, Clone, Copy, PartialEq)]
#[expect(
    dead_code,
    reason = "generic blur realization enters after the retained resource checkpoint"
)]
pub(crate) enum EffectDeclaration {
    None,
    GroupOpacity(EffectEnvelope),
    Blur {
        envelope: EffectEnvelope,
        maximum_sigma: f32,
    },
    Backdrop(EffectEnvelope),
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) struct EffectEnvelope {
    bounds: geometry::Rect,
    maximum_sampling_reach: f32,
}

#[derive(Debug, Clone, PartialEq, Error)]
pub(crate) enum ContractError {
    #[error("scene commit contains duplicate composition identity {0:?}")]
    DuplicateNode(composition::tree::NodeId),
    #[error("scene node {node:?} names missing or later parent {parent:?}")]
    UnknownParent {
        node: composition::tree::NodeId,
        parent: composition::tree::NodeId,
    },
    #[error("scene node {node:?} declares duplicate property {kind:?}")]
    DuplicateProperty {
        node: composition::tree::NodeId,
        kind: PropertyKind,
    },
    #[error("property snapshot targets {actual:?}, expected {expected:?}")]
    IncompatibleCommit {
        expected: Revision,
        actual: Revision,
    },
    #[error("property snapshot contains duplicate value for {0:?}")]
    DuplicateValue(PropertyRef),
    #[error("property snapshot is missing {0:?}")]
    MissingValue(PropertyRef),
    #[error("property snapshot contains undeclared {0:?}")]
    UndeclaredValue(PropertyRef),
    #[error("property snapshot marks undeclared {0:?} as changed")]
    UndeclaredChange(PropertyRef),
    #[error("property snapshot contains a non-finite or out-of-envelope {0:?}")]
    InvalidValue(PropertyRef),
}

#[cfg(feature = "renderer-debug")]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum FixtureCase {
    Empty,
    SolidQuad,
    GradientQuad,
    TransformedQuad,
    Rule,
    Text,
    TextViewport,
    Icon,
    Shadow,
    Outline,
    SolidPane,
    RoundedClip,
    NestedClip,
    GroupOpacity,
    OrderedGroup,
    GlassPane,
    TransparentPopup,
}

impl Commit {
    #[cfg(any(test, feature = "renderer-debug"))]
    pub(crate) fn new(
        revision: Revision,
        size: geometry::Size,
        clear: Color,
        nodes: Vec<Node>,
    ) -> Result<Self, ContractError> {
        Self::from_parts(
            revision,
            size,
            clear,
            nodes.into_iter().map(Arc::new).collect(),
            None,
            Vec::new(),
        )
    }

    fn from_parts(
        revision: Revision,
        size: geometry::Size,
        clear: Color,
        nodes: Vec<Arc<Node>>,
        order: Option<Vec<Draw>>,
        material_regions: Vec<MaterialRegion>,
    ) -> Result<Self, ContractError> {
        let mut seen_nodes = HashSet::new();
        let mut property_topology = Vec::new();
        let mut seen_properties = HashSet::new();

        for node in &nodes {
            if !seen_nodes.insert(node.id) {
                return Err(ContractError::DuplicateNode(node.id));
            }
            if let Some(parent) = node.parent
                && !seen_nodes.contains(&parent)
            {
                return Err(ContractError::UnknownParent {
                    node: node.id,
                    parent,
                });
            }
            for kind in &node.properties {
                let property = PropertyRef::new(node.id, *kind);
                if !seen_properties.insert(property) {
                    return Err(ContractError::DuplicateProperty {
                        node: node.id,
                        kind: *kind,
                    });
                }
                property_topology.push(property);
            }
        }

        Ok(Self {
            revision,
            size: size.sanitized(),
            clear,
            nodes,
            property_topology,
            order,
            material_regions,
        })
    }

    pub(crate) fn size(&self) -> geometry::Size {
        self.size
    }

    pub(crate) fn clear(&self) -> Color {
        self.clear
    }

    pub(crate) fn nodes(&self) -> &[Arc<Node>] {
        &self.nodes
    }

    #[cfg(feature = "renderer-debug")]
    pub(crate) fn property_topology(&self) -> &[PropertyRef] {
        &self.property_topology
    }

    pub(crate) fn order(&self) -> Option<&[Draw]> {
        self.order.as_deref()
    }

    pub(crate) fn compatibility_scene(
        &self,
        properties: &Properties,
    ) -> Result<Scene, ContractError> {
        properties.require_compatible(self)?;
        if let Some(order) = &self.order {
            return Ok(Scene {
                size: self.size,
                clear: self.clear,
                primitives: self.compatibility_order(order),
                material_regions: self.material_regions.clone(),
            });
        }
        let mut primitives = Vec::new();
        for node in self.nodes.iter().filter(|node| node.parent.is_none()) {
            primitives.extend(self.compatibility_node(node, properties));
        }
        Ok(Scene {
            size: self.size,
            clear: self.clear,
            primitives,
            material_regions: self.material_regions.clone(),
        })
    }

    fn compatibility_order(&self, order: &[Draw]) -> Vec<Primitive> {
        let mut primitives = Vec::new();
        let mut index = 0;
        self.compatibility_order_until(order, &mut index, false, &mut primitives);
        primitives
    }

    fn compatibility_order_until(
        &self,
        order: &[Draw],
        index: &mut usize,
        stop_at_group_end: bool,
        target: &mut Vec<Primitive>,
    ) {
        while let Some(draw) = order.get(*index) {
            *index = index.saturating_add(1);
            match draw {
                Draw::Content { node, index } => {
                    let Some(content) = self
                        .nodes
                        .iter()
                        .find(|candidate| candidate.id == *node)
                        .and_then(|node| node.content.get(*index))
                    else {
                        continue;
                    };
                    target.push(content.as_primitive(None));
                }
                Draw::PushClip { clip, .. } => target.push(Primitive::Clip(*clip)),
                Draw::PopClip => target.push(Primitive::PopClip),
                Draw::PushGroup { opacity, .. } => {
                    let mut members = Vec::new();
                    self.compatibility_order_until(order, index, true, &mut members);
                    if let Some(group) = Group::new(members, *opacity) {
                        target.push(Primitive::Group(group));
                    }
                }
                Draw::PopGroup if stop_at_group_end => return,
                Draw::PopGroup => {}
            }
        }
    }

    fn same_projection(&self, other: &Self) -> bool {
        self.size == other.size
            && self.clear == other.clear
            && self.nodes == other.nodes
            && self.property_topology == other.property_topology
            && self.order == other.order
            && self.material_regions == other.material_regions
    }

    #[cfg(test)]
    pub(crate) fn legacy_test_pair(scene: &Scene) -> (Arc<Self>, Properties) {
        let mut next = 1;
        let owner = composition::tree::NodeId::layout(&mut next);
        let mut builder = Builder::new(scene.size(), scene.clear());
        builder.register(
            owner,
            None,
            composition::tree::ContentRevision::INITIAL,
            geometry::Rect::from_size(scene.size()),
        );
        builder.append_fragment(owner, scene);
        let commit = builder
            .finish(None, &mut HashMap::new())
            .expect("test scene must form a valid retained commit");
        let properties =
            Properties::empty(&commit).expect("test scene commit declares no dynamic properties");
        (commit, properties)
    }

    fn compatibility_node(&self, node: &Node, properties: &Properties) -> Vec<Primitive> {
        let transform = properties.value(PropertyRef::new(node.id, PropertyKind::Transform));
        let scroll = properties.value(PropertyRef::new(node.id, PropertyKind::ScrollOffset));
        let mut primitives = node
            .content
            .iter()
            .map(|content| content.as_primitive(transform))
            .collect::<Vec<_>>();
        for child in self
            .nodes
            .iter()
            .filter(|child| child.parent == Some(node.id))
        {
            primitives.extend(self.compatibility_node(child, properties));
        }
        if let Some(PropertyValue::ScrollOffset { x, y, .. }) = scroll {
            let dx = -x.round() as i32;
            let dy = -y.round() as i32;
            primitives = primitives
                .iter()
                .map(|primitive| primitive.translated(dx, dy))
                .collect();
        }
        if matches!(node.effect, EffectDeclaration::GroupOpacity(_)) {
            let opacity = match properties.value(PropertyRef::new(node.id, PropertyKind::Opacity)) {
                Some(PropertyValue::Opacity { value, .. }) => value,
                _ => 1.0,
            };
            primitives = Group::new(primitives, opacity)
                .map(Primitive::Group)
                .into_iter()
                .collect();
        }
        let clip = match properties.value(PropertyRef::new(node.id, PropertyKind::Clip)) {
            Some(PropertyValue::Clip { rect, .. }) => Some(
                Clip::new(rect).with_rounding(node.clip.map(Clip::rounding).unwrap_or_default()),
            ),
            _ => node.clip,
        };
        if let Some(clip) = clip {
            let mut clipped = Vec::with_capacity(primitives.len() + 2);
            clipped.push(Primitive::Clip(clip));
            clipped.extend(primitives);
            clipped.push(Primitive::PopClip);
            primitives = clipped;
        }
        primitives
    }
}

impl Content {
    fn as_primitive(&self, transform: Option<PropertyValue>) -> Primitive {
        match self {
            Self::Quad(quad) => {
                let quad = match transform {
                    Some(PropertyValue::Transform { value, .. }) => quad.with_transform(value),
                    _ => *quad,
                };
                Primitive::Quad(quad)
            }
            Self::Rule(rule) => Primitive::Rule(*rule),
            Self::Text(text) => Primitive::Text(text.clone()),
            Self::TextViewport(text) => Primitive::TextViewport(text.clone()),
            Self::Icon(icon) => Primitive::Icon(*icon),
            Self::Shadow(shadow) => Primitive::Shadow(*shadow),
            Self::Pane(pane) => Primitive::Pane(pane.clone()),
            Self::Outline(outline) => Primitive::Outline(*outline),
        }
    }
}

impl Builder {
    pub(crate) fn new(size: geometry::Size, clear: Color) -> Self {
        Self {
            size: size.sanitized(),
            clear,
            nodes: Vec::new(),
            node_indices: HashMap::new(),
            order: Vec::new(),
            material_regions: Vec::new(),
        }
    }

    pub(crate) fn register(
        &mut self,
        id: composition::tree::NodeId,
        parent: Option<composition::tree::NodeId>,
        content_revision: composition::tree::ContentRevision,
        bounds: geometry::Rect,
    ) {
        if self.node_indices.contains_key(&id) {
            return;
        }
        let index = self.nodes.len();
        self.node_indices.insert(id, index);
        self.nodes.push(NodeDraft {
            id,
            parent,
            content_revision,
            bounds,
            content: Vec::new(),
        });
    }

    pub(crate) fn push_clip(&mut self, clip: Clip) {
        self.order.push(Draw::PushClip { node: None, clip });
    }

    pub(crate) fn pop_clip(&mut self) {
        self.order.push(Draw::PopClip);
    }

    pub(crate) fn append_fragment(&mut self, owner: composition::tree::NodeId, fragment: &Scene) {
        if !self.node_indices.contains_key(&owner) {
            return;
        }
        for primitive in fragment.primitives() {
            self.append_primitive(owner, primitive);
        }
        self.material_regions
            .extend(fragment.material_regions.iter().cloned());
    }

    fn append_primitive(&mut self, owner: composition::tree::NodeId, primitive: &Primitive) {
        match primitive {
            Primitive::Clip(clip) => self.order.push(Draw::PushClip {
                node: Some(owner),
                clip: *clip,
            }),
            Primitive::PopClip => self.order.push(Draw::PopClip),
            Primitive::Group(group) => {
                self.order.push(Draw::PushGroup {
                    node: owner,
                    bounds: self.nodes[self.node_indices[&owner]].bounds,
                    opacity: group.opacity(),
                });
                for primitive in group.primitives() {
                    self.append_primitive(owner, primitive);
                }
                self.order.push(Draw::PopGroup);
            }
            Primitive::Quad(quad) => self.push_content(owner, Content::Quad(*quad)),
            Primitive::Rule(rule) => self.push_content(owner, Content::Rule(*rule)),
            Primitive::Text(text) => self.push_content(owner, Content::Text(text.clone())),
            Primitive::TextViewport(text) => {
                self.push_content(owner, Content::TextViewport(text.clone()));
            }
            Primitive::Icon(icon) => self.push_content(owner, Content::Icon(*icon)),
            Primitive::Shadow(shadow) => self.push_content(owner, Content::Shadow(*shadow)),
            Primitive::Pane(pane) => self.push_content(owner, Content::Pane(pane.clone())),
            Primitive::Outline(outline) => self.push_content(owner, Content::Outline(*outline)),
        }
    }

    fn push_content(&mut self, owner: composition::tree::NodeId, content: Content) {
        let Some(node_index) = self.node_indices.get(&owner).copied() else {
            return;
        };
        let index = self.nodes[node_index].content.len();
        self.nodes[node_index].content.push(content);
        self.order.push(Draw::Content { node: owner, index });
    }

    pub(crate) fn finish(
        self,
        previous: Option<&Arc<Commit>>,
        retained_nodes: &mut HashMap<composition::tree::NodeId, Arc<Node>>,
    ) -> Result<Arc<Commit>, ContractError> {
        let mut nodes = Vec::with_capacity(self.nodes.len());
        for draft in self.nodes {
            let previous = retained_nodes.get(&draft.id);
            let node = Node::retained(previous, draft);
            retained_nodes.insert(node.id, Arc::clone(&node));
            nodes.push(node);
        }
        let revision = previous.map_or(Revision::INITIAL, |commit| commit.revision.next());
        let commit = Commit::from_parts(
            revision,
            self.size,
            self.clear,
            nodes,
            Some(self.order),
            self.material_regions,
        )?;
        if let Some(previous) = previous
            && previous.same_projection(&commit)
        {
            return Ok(Arc::clone(previous));
        }
        Ok(Arc::new(commit))
    }
}

impl Node {
    #[cfg(any(test, feature = "renderer-debug"))]
    pub(crate) fn new(
        id: composition::tree::NodeId,
        parent: Option<composition::tree::NodeId>,
        local_bounds: geometry::Rect,
        content: Vec<Content>,
    ) -> Self {
        Self {
            id,
            parent,
            content_revision: composition::tree::ContentRevision::INITIAL,
            geometry_revision: GeometryRevision::INITIAL,
            topology_revision: TopologyRevision::INITIAL,
            local_bounds,
            content,
            properties: Vec::new(),
            clip: None,
            opacity: OpacityDeclaration::Blended,
            effect: EffectDeclaration::None,
        }
    }

    fn retained(previous: Option<&Arc<Self>>, draft: NodeDraft) -> Arc<Self> {
        let geometry_revision = previous.map_or(GeometryRevision::INITIAL, |previous| {
            if previous.local_bounds == draft.bounds {
                previous.geometry_revision
            } else {
                previous.geometry_revision.next()
            }
        });
        let topology_revision = previous.map_or(TopologyRevision::INITIAL, |previous| {
            if previous.parent == draft.parent {
                previous.topology_revision
            } else {
                previous.topology_revision.next()
            }
        });
        let candidate = Self {
            id: draft.id,
            parent: draft.parent,
            content_revision: draft.content_revision,
            geometry_revision,
            topology_revision,
            local_bounds: draft.bounds,
            content: draft.content,
            properties: Vec::new(),
            clip: None,
            opacity: OpacityDeclaration::Blended,
            effect: EffectDeclaration::None,
        };
        if let Some(previous) = previous
            && previous.as_ref() == &candidate
        {
            Arc::clone(previous)
        } else {
            Arc::new(candidate)
        }
    }

    #[cfg(any(test, feature = "renderer-debug"))]
    pub(crate) fn with_properties(
        mut self,
        properties: impl IntoIterator<Item = PropertyKind>,
    ) -> Self {
        self.properties = properties.into_iter().collect();
        self
    }

    #[cfg(feature = "renderer-debug")]
    pub(crate) fn with_content_revision(mut self, revision: u64) -> Self {
        self.content_revision = composition::tree::ContentRevision::renderer_fixture(revision);
        self
    }

    #[cfg(feature = "renderer-debug")]
    pub(crate) fn with_clip(mut self, clip: Clip) -> Self {
        self.clip = Some(clip);
        self
    }

    #[cfg(any(test, feature = "renderer-debug"))]
    pub(crate) fn with_opacity(mut self, opacity: OpacityDeclaration) -> Self {
        self.opacity = opacity;
        self
    }

    #[cfg(any(test, feature = "renderer-debug"))]
    pub(crate) fn with_effect(mut self, effect: EffectDeclaration) -> Self {
        self.effect = effect;
        self
    }

    pub(crate) fn id(&self) -> composition::tree::NodeId {
        self.id
    }

    pub(crate) fn parent(&self) -> Option<composition::tree::NodeId> {
        self.parent
    }

    pub(crate) fn content(&self) -> &[Content] {
        &self.content
    }

    pub(crate) fn declares(&self, kind: PropertyKind) -> bool {
        self.properties.contains(&kind)
    }

    pub(crate) fn clip(&self) -> Option<Clip> {
        self.clip
    }

    pub(crate) fn effect(&self) -> EffectDeclaration {
        self.effect
    }

    pub(crate) fn content_revision(&self) -> composition::tree::ContentRevision {
        self.content_revision
    }

    pub(crate) fn geometry_revision(&self) -> GeometryRevision {
        self.geometry_revision
    }

    pub(crate) fn topology_revision(&self) -> TopologyRevision {
        self.topology_revision
    }
}

impl PropertyRef {
    pub(crate) const fn new(node: composition::tree::NodeId, kind: PropertyKind) -> Self {
        Self { node, kind }
    }
}

impl Properties {
    pub(crate) fn new(
        commit: &Commit,
        serial: PropertySerial,
        values: Vec<PropertyValue>,
        changed: Vec<PropertyRef>,
    ) -> Result<Self, ContractError> {
        let topology = commit
            .property_topology
            .iter()
            .copied()
            .collect::<HashSet<_>>();
        let mut present = HashSet::new();

        for value in &values {
            let property = value.property_ref();
            if !topology.contains(&property) {
                return Err(ContractError::UndeclaredValue(property));
            }
            if !present.insert(property) {
                return Err(ContractError::DuplicateValue(property));
            }
            if !value.is_valid(commit) {
                return Err(ContractError::InvalidValue(property));
            }
        }
        if let Some(missing) = commit
            .property_topology
            .iter()
            .find(|property| !present.contains(property))
        {
            return Err(ContractError::MissingValue(*missing));
        }
        if let Some(undeclared) = changed.iter().find(|property| !topology.contains(property)) {
            return Err(ContractError::UndeclaredChange(*undeclared));
        }

        Ok(Self {
            commit: commit.revision,
            serial,
            values,
            changed,
        })
    }

    pub(crate) fn empty(commit: &Commit) -> Result<Self, ContractError> {
        Self::new(commit, PropertySerial::INITIAL, Vec::new(), Vec::new())
    }

    pub(crate) fn require_compatible(&self, commit: &Commit) -> Result<(), ContractError> {
        if self.commit == commit.revision {
            Ok(())
        } else {
            Err(ContractError::IncompatibleCommit {
                expected: commit.revision,
                actual: self.commit,
            })
        }
    }

    pub(crate) fn value(&self, property: PropertyRef) -> Option<PropertyValue> {
        self.values
            .iter()
            .copied()
            .find(|value| value.property_ref() == property)
    }

    pub(crate) fn serial(&self) -> PropertySerial {
        self.serial
    }

    #[expect(
        dead_code,
        reason = "Checkpoint 6 consumes the admitted changed-property set for sparse uploads"
    )]
    pub(crate) fn changed(&self) -> &[PropertyRef] {
        &self.changed
    }
}

impl PropertyValue {
    pub(crate) const fn property_ref(self) -> PropertyRef {
        match self {
            Self::Transform { node, .. } => PropertyRef::new(node, PropertyKind::Transform),
            Self::ScrollOffset { node, .. } => PropertyRef::new(node, PropertyKind::ScrollOffset),
            Self::Opacity { node, .. } => PropertyRef::new(node, PropertyKind::Opacity),
            Self::Clip { node, .. } => PropertyRef::new(node, PropertyKind::Clip),
            Self::Blur { node, .. } => PropertyRef::new(node, PropertyKind::Blur),
        }
    }

    fn is_valid(self, commit: &Commit) -> bool {
        let Some(node) = commit
            .nodes
            .iter()
            .find(|node| node.id == self.property_ref().node)
        else {
            return false;
        };
        match self {
            Self::Transform { value, .. } => {
                value.origin_x().is_finite()
                    && value.origin_y().is_finite()
                    && value.translate_x().is_finite()
                    && value.translate_y().is_finite()
                    && value.scale_x().is_finite()
                    && value.scale_y().is_finite()
            }
            Self::ScrollOffset { x, y, .. } => x.is_finite() && y.is_finite(),
            Self::Opacity { value, .. } => value.is_finite() && (0.0..=1.0).contains(&value),
            Self::Clip { rect, .. } => rect_is_within(rect, node.local_bounds),
            Self::Blur { sigma, .. } => match node.effect {
                EffectDeclaration::Blur { maximum_sigma, .. } => {
                    sigma.is_finite() && sigma >= 0.0 && sigma <= maximum_sigma
                }
                _ => false,
            },
        }
    }
}

fn rect_is_within(inner: geometry::Rect, outer: geometry::Rect) -> bool {
    inner.x() >= outer.x()
        && inner.y() >= outer.y()
        && inner.x().saturating_add(inner.width()) <= outer.x().saturating_add(outer.width())
        && inner.y().saturating_add(inner.height()) <= outer.y().saturating_add(outer.height())
}

impl EffectEnvelope {
    #[cfg(any(test, feature = "renderer-debug"))]
    pub(crate) fn new(bounds: geometry::Rect, maximum_sampling_reach: f32) -> Option<Self> {
        (maximum_sampling_reach.is_finite() && maximum_sampling_reach >= 0.0).then_some(Self {
            bounds,
            maximum_sampling_reach,
        })
    }

    pub(crate) fn bounds(self) -> geometry::Rect {
        self.bounds
    }

    pub(crate) fn maximum_sampling_reach(self) -> f32 {
        self.maximum_sampling_reach
    }
}

#[cfg(feature = "renderer-debug")]
pub(crate) fn renderer_fixture(case: FixtureCase) -> Result<(Commit, Properties), ContractError> {
    use crate::icon as icons;

    if matches!(case, FixtureCase::OrderedGroup) {
        return renderer_ordered_group_fixture();
    }

    let root_id = composition::tree::NodeId::renderer_fixture(1);
    let child_id = composition::tree::NodeId::renderer_fixture(2);
    let bounds = geometry::Rect::new(0, 0, 64, 64);
    let mut nodes = Vec::new();
    let mut values = Vec::new();

    let root = match case {
        FixtureCase::Empty => Node::new(root_id, None, bounds, Vec::new()),
        FixtureCase::SolidQuad => Node::new(
            root_id,
            None,
            bounds,
            vec![Content::Quad(Quad::new(
                geometry::Rect::new(8, 8, 40, 40),
                Color::rgba(204, 51, 26, 128),
            ))],
        ),
        FixtureCase::GradientQuad => Node::new(
            root_id,
            None,
            bounds,
            vec![Content::Quad(
                Quad::styled(
                    geometry::Rect::new(8, 8, 40, 40),
                    Style::filled_with(Brush::linear_gradient(
                        Color::rgba(230, 26, 51, 255),
                        Color::rgba(26, 77, 230, 166),
                    )),
                )
                .with_rounding(Rounding::fixed(7.0)),
            )],
        ),
        FixtureCase::TransformedQuad => {
            values.push(PropertyValue::Transform {
                node: root_id,
                value: Transform::scale_about(29.0, 28.0, 0.8, 1.1),
            });
            values.push(PropertyValue::ScrollOffset {
                node: root_id,
                x: 2.0,
                y: -1.0,
            });
            Node::new(
                root_id,
                None,
                bounds,
                vec![Content::Quad(Quad::new(
                    geometry::Rect::new(10, 12, 38, 32),
                    Color::rgba(38, 191, 89, 230),
                ))],
            )
            .with_properties([PropertyKind::Transform, PropertyKind::ScrollOffset])
        }
        FixtureCase::Rule => Node::new(
            root_id,
            None,
            bounds,
            vec![Content::Rule(Rule::horizontal(
                geometry::Rect::new(6, 30, 52, 4),
                Color::rgba(242, 179, 26, 255),
                3,
            ))],
        )
        .with_opacity(OpacityDeclaration::Opaque),
        FixtureCase::Text => Node::new(
            root_id,
            None,
            bounds,
            vec![Content::Text(Text::new(
                geometry::Rect::new(4, 8, 56, 48),
                "Render",
                Color::rgba(230, 230, 235, 255),
                TextWrap::None,
            ))],
        ),
        FixtureCase::TextViewport => {
            let rect = geometry::Rect::new(4, 8, 56, 48);
            let mut engine = crate::text::layout::Engine::new();
            let field = crate::text::surface::Field::new("Viewport");
            let layout = engine.text_field_paint_layout_for_field(
                &field,
                crate::text::document::Style::default().with_size(16.0),
                crate::geometry::area::logical(56.0, 48.0),
                crate::text::view::ViewState::default(),
            );
            let surface = layout
                .surface()
                .expect("nonempty debug text field should shape a surface");
            Node::new(
                root_id,
                None,
                bounds,
                vec![Content::TextViewport(TextViewport::new(
                    rect,
                    vec![super::paint::text_surface::surface(rect, surface)],
                ))],
            )
        }
        FixtureCase::Icon => Node::new(
            root_id,
            None,
            bounds,
            vec![Content::Icon(Icon::new(
                geometry::Rect::new(12, 12, 40, 40),
                icons::Icon::phosphor(icons::Id::new("check")),
                Color::rgba(51, 204, 242, 255),
                32.0,
            ))],
        ),
        FixtureCase::Shadow => Node::new(
            root_id,
            None,
            bounds,
            vec![Content::Shadow(
                Shadow::new(
                    geometry::Rect::new(14, 14, 30, 26),
                    Color::rgba(26, 51, 204, 140),
                    5.0,
                    2.0,
                    Offset::new(3.0, 4.0),
                )
                .with_rounding(Rounding::fixed(5.0)),
            )],
        ),
        FixtureCase::Outline => Node::new(
            root_id,
            None,
            bounds,
            vec![Content::Outline(
                Outline::new(
                    geometry::Rect::new(10, 10, 44, 38),
                    Color::rgba(217, 64, 230, 204),
                )
                .with_width(2.0)
                .with_offset(1.0)
                .with_rounding(Rounding::fixed(8.0)),
            )],
        ),
        FixtureCase::SolidPane => Node::new(
            root_id,
            None,
            bounds,
            vec![Content::Pane(
                Pane::new(
                    geometry::Rect::new(8, 8, 48, 44),
                    Material::solid(Brush::solid(Color::rgba(38, 140, 217, 191))),
                )
                .with_rounding(Rounding::fixed(6.0)),
            )],
        ),
        FixtureCase::RoundedClip => {
            values.push(PropertyValue::Clip {
                node: root_id,
                rect: geometry::Rect::new(12, 12, 36, 36),
            });
            Node::new(
                root_id,
                None,
                bounds,
                vec![Content::Quad(Quad::styled(
                    geometry::Rect::new(4, 20, 56, 22),
                    Style::filled_with(Brush::linear_gradient(
                        Color::rgba(255, 51, 26, 255),
                        Color::rgba(26, 230, 128, 255),
                    )),
                ))],
            )
            .with_properties([PropertyKind::Clip])
            .with_clip(
                Clip::new(geometry::Rect::new(12, 12, 36, 36))
                    .with_rounding(Rounding::relative(1.0)),
            )
        }
        FixtureCase::NestedClip => {
            let root = Node::new(root_id, None, bounds, Vec::new()).with_clip(
                Clip::new(geometry::Rect::new(6, 6, 52, 52)).with_rounding(Rounding::fixed(8.0)),
            );
            nodes.push(root);
            nodes.push(
                Node::new(
                    child_id,
                    Some(root_id),
                    bounds,
                    vec![Content::Quad(Quad::styled(
                        geometry::Rect::new(2, 20, 60, 24),
                        Style::filled_with(Brush::linear_gradient(
                            Color::rgba(242, 64, 26, 230),
                            Color::rgba(26, 102, 242, 191),
                        )),
                    ))],
                )
                .with_clip(
                    Clip::new(geometry::Rect::new(16, 12, 32, 40))
                        .with_rounding(Rounding::relative(0.45)),
                ),
            );
            Node::new(root_id, None, bounds, Vec::new())
        }
        FixtureCase::GroupOpacity => {
            values.push(PropertyValue::Opacity {
                node: root_id,
                value: 0.55,
            });
            Node::new(
                root_id,
                None,
                geometry::Rect::new(12, 12, 40, 40),
                vec![Content::Quad(Quad::new(
                    geometry::Rect::new(14, 14, 36, 36),
                    Color::rgba(230, 38, 64, 204),
                ))],
            )
            .with_properties([PropertyKind::Opacity])
            .with_opacity(OpacityDeclaration::Variable)
            .with_effect(EffectDeclaration::GroupOpacity(
                EffectEnvelope::new(geometry::Rect::new(12, 12, 40, 40), 0.0)
                    .expect("zero-reach group envelope is valid"),
            ))
        }
        FixtureCase::OrderedGroup => {
            unreachable!("ordered fixture returned before node projection")
        }
        FixtureCase::GlassPane => Node::new(
            root_id,
            None,
            bounds,
            vec![
                Content::Quad(Quad::styled(
                    bounds,
                    Style::filled_with(Brush::linear_gradient(
                        Color::rgba(13, 26, 89, 255),
                        Color::rgba(204, 64, 26, 255),
                    )),
                )),
                Content::Pane(
                    Pane::new(
                        geometry::Rect::new(10, 10, 44, 42),
                        Material::glass(
                            Glass::panel_dark()
                                .with_blur_sigma(2.5)
                                .with_tint(Brush::solid(Color::rgba(191, 217, 255, 255)), 0.22)
                                .with_noise_opacity(0.0),
                        ),
                    )
                    .with_rounding(Rounding::fixed(7.0)),
                ),
            ],
        )
        .with_effect(EffectDeclaration::Backdrop(
            EffectEnvelope::new(bounds, 8.0).expect("glass envelope is valid"),
        )),
        FixtureCase::TransparentPopup => Node::new(
            root_id,
            None,
            bounds,
            vec![Content::Quad(
                Quad::new(
                    geometry::Rect::new(8, 8, 48, 48),
                    Color::rgba(128, 128, 128, 128),
                )
                .with_rasterization(Rasterization::new(super::EdgeMode::Hard)),
            )],
        ),
    };

    if !matches!(case, FixtureCase::NestedClip) {
        nodes.push(root);
    }
    let commit = Commit::new(
        Revision::INITIAL,
        geometry::Size::new(64, 64),
        Color::rgba(0, 0, 0, 0),
        nodes,
    )?;
    let properties = Properties::new(
        &commit,
        PropertySerial::INITIAL,
        values,
        commit.property_topology().to_vec(),
    )?;
    Ok((commit, properties))
}

#[cfg(feature = "renderer-debug")]
fn renderer_ordered_group_fixture() -> Result<(Commit, Properties), ContractError> {
    let owner = composition::tree::NodeId::renderer_fixture(31);
    let size = geometry::Size::new(64, 64);
    let mut builder = Builder::new(size, Color::rgba(0, 0, 0, 0));
    builder.register(
        owner,
        None,
        composition::tree::ContentRevision::INITIAL,
        geometry::Rect::new(7, 9, 50, 48),
    );

    let mut nested = Scene::new(size);
    nested.push_quad(Quad::new(
        geometry::Rect::new(25, 17, 26, 30),
        Color::rgba(38, 153, 230, 191),
    ));
    nested.push_text(Text::new(
        geometry::Rect::new(27, 20, 22, 22),
        "G",
        Color::rgba(242, 242, 247, 255),
        TextWrap::None,
    ));
    let mut members = Scene::new(size);
    members.push_quad(Quad::new(
        geometry::Rect::new(11, 13, 40, 36),
        Color::rgba(230, 51, 77, 204),
    ));
    members.push_clip(Clip::new(geometry::Rect::new(17, 15, 32, 32)));
    members.append_scene_with_opacity(&nested, 0.65);
    members.pop_clip();
    let mut fragment = Scene::new(size);
    fragment.append_scene_with_opacity(&members, 0.8);
    builder.append_fragment(owner, &fragment);

    let mut retained = HashMap::new();
    let commit = builder.finish(None, &mut retained)?;
    let properties = Properties::empty(&commit)?;
    drop(retained);
    Ok((Arc::unwrap_or_clone(commit), properties))
}

#[cfg(feature = "renderer-debug")]
pub(crate) fn renderer_partial_update_fixture(
    version: u64,
) -> Result<(Commit, Properties), ContractError> {
    let first_id = composition::tree::NodeId::renderer_fixture(101);
    let second_id = composition::tree::NodeId::renderer_fixture(102);
    let bounds = geometry::Rect::new(0, 0, 64, 64);
    let first = Node::new(
        first_id,
        None,
        bounds,
        vec![Content::Quad(Quad::new(
            geometry::Rect::new(4, 8, 24, 20),
            Color::rgba(51, 102, 204, 255),
        ))],
    );
    let changed = version > 1;
    let second = Node::new(
        second_id,
        None,
        bounds,
        vec![Content::Quad(Quad::new(
            if changed {
                geometry::Rect::new(34, 30, 24, 22)
            } else {
                geometry::Rect::new(34, 8, 24, 20)
            },
            if changed {
                Color::rgba(230, 77, 51, 230)
            } else {
                Color::rgba(51, 204, 102, 230)
            },
        ))],
    )
    .with_content_revision(version);
    let commit = Commit::new(
        Revision::renderer_fixture(version),
        geometry::Size::new(64, 64),
        Color::rgba(0, 0, 0, 0),
        vec![first, second],
    )?;
    let properties = Properties::empty(&commit)?;
    Ok((commit, properties))
}

#[cfg(test)]
mod tests {
    use super::super::{Glass, Material};
    use super::*;

    fn id(value: u64) -> composition::tree::NodeId {
        composition::tree::NodeId::layout(&mut value.clone())
    }

    fn empty_node(value: u64, parent: Option<composition::tree::NodeId>) -> Node {
        Node::new(
            id(value),
            parent,
            geometry::Rect::new(0, 0, 20, 10),
            Vec::new(),
        )
    }

    #[test]
    fn commit_rejects_a_parent_that_has_not_already_entered_the_order() {
        let parent = id(1);
        let child = empty_node(2, Some(parent));

        let error = Commit::new(
            Revision::INITIAL,
            geometry::Size::new(20, 10),
            Color::rgba(0, 0, 0, 0),
            vec![child],
        )
        .expect_err("missing parent must be rejected");

        assert!(matches!(error, ContractError::UnknownParent { .. }));
    }

    #[test]
    fn complete_property_snapshot_accepts_declared_node_derived_values() {
        let node = empty_node(1, None).with_properties([PropertyKind::Opacity]);
        let node_id = node.id();
        let commit = Commit::new(
            Revision::INITIAL,
            geometry::Size::new(20, 10),
            Color::rgba(0, 0, 0, 0),
            vec![node],
        )
        .expect("commit should be valid");
        let property = PropertyRef::new(node_id, PropertyKind::Opacity);
        let properties = Properties::new(
            &commit,
            PropertySerial::INITIAL,
            vec![PropertyValue::Opacity {
                node: node_id,
                value: 0.5,
            }],
            vec![property],
        )
        .expect("complete property state should be valid");

        assert_eq!(
            properties.value(property),
            Some(PropertyValue::Opacity {
                node: node_id,
                value: 0.5
            })
        );
    }

    #[test]
    fn property_snapshot_rejects_missing_and_undeclared_values() {
        let node = empty_node(1, None).with_properties([PropertyKind::Opacity]);
        let node_id = node.id();
        let commit = Commit::new(
            Revision::INITIAL,
            geometry::Size::new(20, 10),
            Color::rgba(0, 0, 0, 0),
            vec![node],
        )
        .expect("commit should be valid");

        assert!(matches!(
            Properties::empty(&commit),
            Err(ContractError::MissingValue(_))
        ));
        assert!(matches!(
            Properties::new(
                &commit,
                PropertySerial::INITIAL,
                vec![PropertyValue::Blur {
                    node: node_id,
                    sigma: 1.0,
                }],
                Vec::new(),
            ),
            Err(ContractError::UndeclaredValue(_))
        ));
    }

    fn ordered_builder(
        order: &[composition::tree::NodeId],
        previous: Option<&Arc<Commit>>,
        retained: &mut HashMap<composition::tree::NodeId, Arc<Node>>,
    ) -> Arc<Commit> {
        let mut builder = Builder::new(geometry::Size::new(40, 20), Color::rgba(0, 0, 0, 0));
        let mut ids = order.to_vec();
        ids.sort_unstable();
        ids.dedup();
        for id in ids.iter().copied() {
            builder.register(
                id,
                None,
                composition::tree::ContentRevision::INITIAL,
                geometry::Rect::new(0, 0, 20, 20),
            );
        }
        for id in order.iter().copied() {
            let index = ids
                .iter()
                .position(|candidate| *candidate == id)
                .expect("ordered node must have been registered");
            let mut fragment = Scene::new(geometry::Size::new(40, 20));
            fragment.push_quad(Quad::new(
                geometry::Rect::new(index as i32 * 20, 0, 20, 20),
                if id == ids[0] {
                    Color::rgb(255, 0, 0)
                } else {
                    Color::rgb(0, 0, 255)
                },
            ));
            builder.append_fragment(id, &fragment);
        }
        builder
            .finish(previous, retained)
            .expect("ordered retained fixture should be valid")
    }

    #[test]
    fn unchanged_commit_reuses_commit_and_node_allocations() {
        let first_id = id(11);
        let second_id = id(12);
        let mut retained = HashMap::new();
        let first = ordered_builder(&[first_id, second_id], None, &mut retained);
        let second = ordered_builder(&[first_id, second_id], Some(&first), &mut retained);

        assert!(Arc::ptr_eq(&first, &second));
        assert!(
            first
                .nodes
                .iter()
                .zip(&second.nodes)
                .all(|(left, right)| Arc::ptr_eq(left, right))
        );
    }

    #[test]
    fn reordered_commit_reuses_nodes_but_changes_structural_order() {
        let first_id = id(21);
        let second_id = id(22);
        let mut retained = HashMap::new();
        let first = ordered_builder(&[first_id, second_id], None, &mut retained);
        let first_nodes = retained.clone();
        let second = ordered_builder(&[second_id, first_id], Some(&first), &mut retained);

        assert!(!Arc::ptr_eq(&first, &second));
        for id in [first_id, second_id] {
            assert!(Arc::ptr_eq(&first_nodes[&id], &retained[&id]));
        }
        assert_ne!(first.order, second.order);
    }

    #[test]
    fn retained_commit_preserves_material_region_identity() {
        let owner = id(31);
        let mut builder = Builder::new(geometry::Size::new(40, 20), Color::rgba(0, 0, 0, 0));
        builder.register(
            owner,
            None,
            composition::tree::ContentRevision::INITIAL,
            geometry::Rect::new(0, 0, 40, 20),
        );
        let mut fragment = Scene::new(geometry::Size::new(40, 20));
        fragment.push_material_pane(
            owner,
            Pane::new(
                geometry::Rect::new(0, 0, 40, 20),
                Material::glass(Glass::panel_dark()),
            ),
            None,
        );
        builder.append_fragment(owner, &fragment);
        let commit = builder
            .finish(None, &mut HashMap::new())
            .expect("material commit should be valid");
        let properties = Properties::empty(&commit).expect("material commit has no properties");
        let compatibility = commit
            .compatibility_scene(&properties)
            .expect("material commit should lower through the legacy adapter");

        assert_eq!(compatibility.material_regions().len(), 1);
        assert_eq!(compatibility.material_regions()[0].id(), owner);
        assert_eq!(compatibility.panes()[0].region_id(), Some(owner));
    }

    #[test]
    fn property_only_opacity_tick_changes_zero_node_revisions() {
        let node = empty_node(41, None)
            .with_properties([PropertyKind::Opacity])
            .with_opacity(OpacityDeclaration::Variable)
            .with_effect(EffectDeclaration::GroupOpacity(
                EffectEnvelope::new(geometry::Rect::new(0, 0, 20, 10), 0.0)
                    .expect("zero-reach opacity envelope is valid"),
            ));
        let node_id = node.id;
        let commit = Commit::new(
            Revision::INITIAL,
            geometry::Size::new(20, 10),
            Color::rgba(0, 0, 0, 0),
            vec![node],
        )
        .expect("property fixture commit should be valid");
        let revision_before = (
            commit.nodes[0].content_revision,
            commit.nodes[0].geometry_revision,
            commit.nodes[0].topology_revision,
        );
        let property = PropertyRef::new(node_id, PropertyKind::Opacity);
        let first = Properties::new(
            &commit,
            PropertySerial::INITIAL,
            vec![PropertyValue::Opacity {
                node: node_id,
                value: 0.25,
            }],
            vec![property],
        )
        .expect("first opacity snapshot should be valid");
        let second = Properties::new(
            &commit,
            PropertySerial(2),
            vec![PropertyValue::Opacity {
                node: node_id,
                value: 0.75,
            }],
            vec![property],
        )
        .expect("second opacity snapshot should be valid");

        assert_ne!(first.values, second.values);
        assert_eq!(
            revision_before,
            (
                commit.nodes[0].content_revision,
                commit.nodes[0].geometry_revision,
                commit.nodes[0].topology_revision,
            )
        );
    }
}
