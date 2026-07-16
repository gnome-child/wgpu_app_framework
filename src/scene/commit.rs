use std::{
    collections::{HashMap, HashSet},
    sync::Arc,
};

use thiserror::Error;

use super::super::{composition, geometry, interaction};
#[cfg(feature = "renderer-debug")]
use super::{Brush, Glass, Material, Offset, Rasterization, Rounding, Style, TextStyle, TextWrap};
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

impl PropertySerial {
    pub(crate) fn next(self) -> Self {
        Self(self.0.saturating_add(1))
    }

    pub(crate) fn value(self) -> u64 {
        self.0
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
    scroll: Option<ScrollDeclaration>,
    clip: Option<Clip>,
    opacity: OpacityDeclaration,
    effect: EffectDeclaration,
}

#[derive(Debug, Clone, PartialEq)]
pub(crate) enum Draw {
    Content {
        node: composition::tree::NodeId,
        index: usize,
        projection: ContentProjection,
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
    PushScroll {
        node: composition::tree::NodeId,
    },
    PopScroll,
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
    properties: Vec<PropertyKind>,
    scroll: Option<ScrollDeclaration>,
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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub(crate) enum ContentProjection {
    Normal,
    ScrollbarTrack {
        axis: interaction::ScrollbarAxis,
        edge: i32,
        base_thickness: i32,
        maximum_thickness: i32,
    },
    ScrollbarThumb {
        axis: interaction::ScrollbarAxis,
        edge: i32,
        base_thickness: i32,
        maximum_thickness: i32,
        baseline_start: i32,
        baseline_extent: i32,
        baseline_position: i32,
        travel: i32,
        maximum_offset: i32,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub(crate) enum PropertyKind {
    Transform,
    ScrollOffset,
    Opacity,
    Clip,
    Blur,
    Scrollbar,
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
        value: interaction::ScrollOffset,
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
    Scrollbar {
        node: composition::tree::NodeId,
        opacity: f32,
        thickness: f32,
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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct ScrollDeclaration {
    viewport: geometry::Rect,
    content_bounds: geometry::Rect,
    resident_bounds: geometry::Rect,
    baseline: interaction::ScrollOffset,
    maximum: interaction::ScrollOffset,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum OrderStop {
    Group,
    Scroll,
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
    Scroll,
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

    pub(crate) fn revision(&self) -> Revision {
        self.revision
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

    pub(crate) fn semantic_projection(
        candidate: &Arc<Self>,
        previous: Option<&Arc<Self>>,
        resident_nodes: &HashSet<composition::tree::NodeId>,
    ) -> Result<Arc<Self>, ContractError> {
        if resident_nodes.is_empty() {
            return Ok(Arc::clone(candidate));
        }
        let nodes = candidate
            .nodes
            .iter()
            .filter(|node| !resident_nodes.contains(&node.id))
            .map(|node| {
                Node::semantic(
                    previous.and_then(|previous| {
                        previous
                            .nodes
                            .iter()
                            .find(|candidate| candidate.id == node.id)
                    }),
                    node,
                )
            })
            .collect::<Vec<_>>();
        let order = candidate
            .order
            .as_ref()
            .map(|order| semantic_order(order, resident_nodes));
        let material_regions = candidate
            .material_regions
            .iter()
            .filter(|region| !resident_nodes.contains(&region.id()))
            .cloned()
            .collect();
        let revision = previous.map_or(Revision::INITIAL, |commit| commit.revision.next());
        let semantic = Self::from_parts(
            revision,
            candidate.size,
            candidate.clear,
            nodes,
            order,
            material_regions,
        )?;
        if let Some(previous) = previous
            && previous.same_projection(&semantic)
        {
            Ok(Arc::clone(previous))
        } else {
            Ok(Arc::new(semantic))
        }
    }

    pub(crate) fn with_revision(&self, revision: Revision) -> Arc<Self> {
        let mut commit = self.clone();
        commit.revision = revision;
        Arc::new(commit)
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
                primitives: self.compatibility_order(order, properties),
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

    fn compatibility_order(&self, order: &[Draw], properties: &Properties) -> Vec<Primitive> {
        let mut primitives = Vec::new();
        let mut index = 0;
        self.compatibility_order_until(order, &mut index, None, properties, &mut primitives);
        primitives
    }

    fn compatibility_order_until(
        &self,
        order: &[Draw],
        index: &mut usize,
        stop: Option<OrderStop>,
        properties: &Properties,
        target: &mut Vec<Primitive>,
    ) {
        while let Some(draw) = order.get(*index) {
            *index = index.saturating_add(1);
            match draw {
                Draw::Content {
                    node,
                    index,
                    projection,
                } => {
                    let Some(content) = self
                        .nodes
                        .iter()
                        .find(|candidate| candidate.id == *node)
                        .and_then(|node| node.content.get(*index))
                    else {
                        continue;
                    };
                    target.push(projection.project_primitive(
                        content.as_primitive(None),
                        *node,
                        properties,
                    ));
                }
                Draw::PushClip { clip, .. } => target.push(Primitive::Clip(*clip)),
                Draw::PopClip => target.push(Primitive::PopClip),
                Draw::PushGroup { opacity, .. } => {
                    let mut members = Vec::new();
                    self.compatibility_order_until(
                        order,
                        index,
                        Some(OrderStop::Group),
                        properties,
                        &mut members,
                    );
                    if let Some(group) = Group::new(members, *opacity) {
                        target.push(Primitive::Group(group));
                    }
                }
                Draw::PushScroll { node } => {
                    let mut members = Vec::new();
                    self.compatibility_order_until(
                        order,
                        index,
                        Some(OrderStop::Scroll),
                        properties,
                        &mut members,
                    );
                    let declaration = self
                        .nodes
                        .iter()
                        .find(|candidate| candidate.id == *node)
                        .and_then(|node| node.scroll);
                    if let Some(declaration) = declaration {
                        let current = properties
                            .scroll_offset(*node)
                            .unwrap_or(declaration.baseline);
                        let dx = declaration.baseline.x().saturating_sub(current.x());
                        let dy = declaration.baseline.y().saturating_sub(current.y());
                        let mut clipped = Vec::with_capacity(members.len().saturating_add(2));
                        clipped.push(Primitive::Clip(Clip::new(declaration.viewport)));
                        clipped
                            .extend(members.iter().map(|primitive| primitive.translated(dx, dy)));
                        clipped.push(Primitive::PopClip);
                        target.extend(clipped);
                    } else {
                        target.extend(members);
                    }
                }
                Draw::PopGroup if stop == Some(OrderStop::Group) => return,
                Draw::PopGroup => {}
                Draw::PopScroll if stop == Some(OrderStop::Scroll) => return,
                Draw::PopScroll => {}
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

    #[cfg(any(test, feature = "renderer-debug"))]
    pub(crate) fn projection_difference(&self, other: &Self) -> String {
        let changed_nodes = self
            .nodes
            .iter()
            .filter_map(|node| {
                other
                    .nodes
                    .iter()
                    .find(|candidate| candidate.id == node.id)
                    .filter(|candidate| candidate.as_ref() != node.as_ref())
                    .map(|_| node.id)
            })
            .collect::<Vec<_>>();
        let first_order = self
            .order
            .iter()
            .flatten()
            .zip(other.order.iter().flatten())
            .position(|(left, right)| left != right);
        format!(
            "size={} clear={} changed_nodes={changed_nodes:?} properties={} order_lengths={:?}/{:?} first_order_difference={first_order:?} materials={}",
            self.size == other.size,
            self.clear == other.clear,
            self.property_topology == other.property_topology,
            self.order.as_ref().map(Vec::len),
            other.order.as_ref().map(Vec::len),
            self.material_regions == other.material_regions,
        )
    }

    #[cfg(test)]
    pub(crate) fn test_pair(scene: &Scene) -> (Arc<Self>, Properties) {
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
        if let Some(PropertyValue::ScrollOffset { value, .. }) = scroll {
            let baseline = node.scroll.map_or(
                interaction::ScrollOffset::default(),
                ScrollDeclaration::baseline,
            );
            let dx = baseline.x().saturating_sub(value.x());
            let dy = baseline.y().saturating_sub(value.y());
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

fn semantic_order(
    order: &[Draw],
    resident_nodes: &HashSet<composition::tree::NodeId>,
) -> Vec<Draw> {
    let mut projected = Vec::with_capacity(order.len());
    let mut scopes = Vec::new();
    for draw in order {
        match draw {
            Draw::PushClip { node, .. } => {
                let retained = node.is_none_or(|node| !resident_nodes.contains(&node));
                let start = retained.then(|| {
                    let start = projected.len();
                    projected.push(draw.clone());
                    start
                });
                scopes.push((start, Draw::PopClip));
            }
            Draw::PushGroup { node, .. } => {
                let start = (!resident_nodes.contains(node)).then(|| {
                    let start = projected.len();
                    projected.push(draw.clone());
                    start
                });
                scopes.push((start, Draw::PopGroup));
            }
            Draw::PushScroll { node } => {
                let start = (!resident_nodes.contains(node)).then(|| {
                    let start = projected.len();
                    projected.push(draw.clone());
                    start
                });
                scopes.push((start, Draw::PopScroll));
            }
            Draw::PopClip | Draw::PopGroup | Draw::PopScroll => {
                let Some((start, pop)) = scopes.pop() else {
                    continue;
                };
                debug_assert_eq!(std::mem::discriminant(draw), std::mem::discriminant(&pop));
                if let Some(start) = start {
                    if projected.len() == start + 1 {
                        projected.truncate(start);
                    } else {
                        projected.push(pop);
                    }
                }
            }
            Draw::Content { node, .. } if !resident_nodes.contains(node) => {
                projected.push(draw.clone());
            }
            Draw::Content { .. } => {}
        }
    }
    projected
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

impl ContentProjection {
    fn maximum_thickness(self) -> Option<i32> {
        match self {
            Self::Normal => None,
            Self::ScrollbarTrack {
                maximum_thickness, ..
            }
            | Self::ScrollbarThumb {
                maximum_thickness, ..
            } => Some(maximum_thickness),
        }
    }

    pub(crate) fn scrollbar_position(self, offset: interaction::ScrollOffset) -> Option<i32> {
        let Self::ScrollbarThumb {
            axis,
            travel,
            maximum_offset,
            ..
        } = self
        else {
            return None;
        };
        if maximum_offset <= 0 {
            return Some(0);
        }

        let offset = match axis {
            interaction::ScrollbarAxis::Vertical => offset.y(),
            interaction::ScrollbarAxis::Horizontal => offset.x(),
        }
        .clamp(0, maximum_offset);
        let numerator = i64::from(travel) * i64::from(offset);
        let denominator = i64::from(maximum_offset);
        let half = denominator / 2;
        let rounded = if numerator >= 0 {
            (numerator + half) / denominator
        } else {
            (numerator - half) / denominator
        };
        Some(rounded.clamp(i64::from(i32::MIN), i64::from(i32::MAX)) as i32)
    }

    fn project_primitive(
        self,
        primitive: Primitive,
        node: composition::tree::NodeId,
        properties: &Properties,
    ) -> Primitive {
        let (axis, edge, base_thickness, maximum_thickness) = match self {
            Self::Normal => return primitive,
            Self::ScrollbarTrack {
                axis,
                edge,
                base_thickness,
                maximum_thickness,
            }
            | Self::ScrollbarThumb {
                axis,
                edge,
                base_thickness,
                maximum_thickness,
                ..
            } => (axis, edge, base_thickness, maximum_thickness),
        };
        let Some(PropertyValue::Scrollbar {
            opacity, thickness, ..
        }) = properties.value(PropertyRef::new(node, PropertyKind::Scrollbar))
        else {
            return primitive;
        };
        let Primitive::Quad(quad) = primitive else {
            return primitive;
        };
        let thickness = (thickness.round() as i32).clamp(
            base_thickness.max(1),
            maximum_thickness.max(base_thickness).max(1),
        );
        let mut rect = quad.rect();
        rect = match axis {
            interaction::ScrollbarAxis::Vertical => geometry::Rect::new(
                edge.saturating_sub(thickness),
                rect.y(),
                thickness,
                rect.height(),
            ),
            interaction::ScrollbarAxis::Horizontal => geometry::Rect::new(
                rect.x(),
                edge.saturating_sub(thickness),
                rect.width(),
                thickness,
            ),
        };
        if let Self::ScrollbarThumb {
            baseline_position, ..
        } = self
            && let Some(offset) = properties.scroll_offset(node)
        {
            let position = self
                .scrollbar_position(offset)
                .expect("scrollbar thumb projection must have one integral position");
            let delta = position.saturating_sub(baseline_position);
            rect = geometry::Rect::new(
                rect.x()
                    .saturating_add(if axis == interaction::ScrollbarAxis::Horizontal {
                        delta
                    } else {
                        0
                    }),
                rect.y()
                    .saturating_add(if axis == interaction::ScrollbarAxis::Vertical {
                        delta
                    } else {
                        0
                    }),
                rect.width(),
                rect.height(),
            );
        }
        Primitive::Quad(quad.with_rect(rect).with_opacity(opacity))
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
            properties: Vec::new(),
            scroll: None,
        });
    }

    pub(crate) fn declare_scroll(
        &mut self,
        id: composition::tree::NodeId,
        declaration: ScrollDeclaration,
    ) {
        let Some(index) = self.node_indices.get(&id).copied() else {
            return;
        };
        let node = &mut self.nodes[index];
        if !node.properties.contains(&PropertyKind::ScrollOffset) {
            node.properties.push(PropertyKind::ScrollOffset);
        }
        node.scroll = Some(declaration);
    }

    pub(crate) fn push_scroll(&mut self, node: composition::tree::NodeId) {
        self.order.push(Draw::PushScroll { node });
    }

    pub(crate) fn pop_scroll(&mut self) {
        self.order.push(Draw::PopScroll);
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
        self.push_projected_content(owner, content, ContentProjection::Normal);
    }

    pub(crate) fn push_projected_content(
        &mut self,
        owner: composition::tree::NodeId,
        content: Content,
        projection: ContentProjection,
    ) {
        let Some(node_index) = self.node_indices.get(&owner).copied() else {
            return;
        };
        if projection != ContentProjection::Normal
            && !self.nodes[node_index]
                .properties
                .contains(&PropertyKind::Scrollbar)
        {
            self.nodes[node_index]
                .properties
                .push(PropertyKind::Scrollbar);
        }
        let index = self.nodes[node_index].content.len();
        self.nodes[node_index].content.push(content);
        self.order.push(Draw::Content {
            node: owner,
            index,
            projection,
        });
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
            scroll: None,
            clip: None,
            opacity: OpacityDeclaration::Blended,
            effect: EffectDeclaration::None,
        }
    }

    fn retained(previous: Option<&Arc<Self>>, draft: NodeDraft) -> Arc<Self> {
        let content_revision = previous.map_or(draft.content_revision, |previous| {
            if previous.content == draft.content {
                previous.content_revision
            } else {
                previous.content_revision.next()
            }
        });
        let geometry_revision = previous.map_or(GeometryRevision::INITIAL, |previous| {
            if previous.local_bounds == draft.bounds {
                previous.geometry_revision
            } else {
                previous.geometry_revision.next()
            }
        });
        let topology_revision = previous.map_or(TopologyRevision::INITIAL, |previous| {
            if previous.parent == draft.parent
                && previous.properties == draft.properties
                && previous.scroll == draft.scroll
            {
                previous.topology_revision
            } else {
                previous.topology_revision.next()
            }
        });
        let candidate = Self {
            id: draft.id,
            parent: draft.parent,
            content_revision,
            geometry_revision,
            topology_revision,
            local_bounds: draft.bounds,
            content: draft.content,
            properties: draft.properties,
            scroll: draft.scroll,
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

    fn semantic(previous: Option<&Arc<Self>>, source: &Arc<Self>) -> Arc<Self> {
        let scroll = source.scroll.map(ScrollDeclaration::semantic);
        let content_revision = previous.map_or(source.content_revision, |previous| {
            if previous.content == source.content {
                previous.content_revision
            } else {
                previous.content_revision.next()
            }
        });
        let geometry_revision = previous.map_or(GeometryRevision::INITIAL, |previous| {
            if previous.local_bounds == source.local_bounds {
                previous.geometry_revision
            } else {
                previous.geometry_revision.next()
            }
        });
        let topology_revision = previous.map_or(TopologyRevision::INITIAL, |previous| {
            if previous.parent == source.parent
                && previous.properties == source.properties
                && previous.scroll == scroll
                && previous.clip == source.clip
                && previous.opacity == source.opacity
                && previous.effect == source.effect
            {
                previous.topology_revision
            } else {
                previous.topology_revision.next()
            }
        });
        let candidate = Self {
            id: source.id,
            parent: source.parent,
            content_revision,
            geometry_revision,
            topology_revision,
            local_bounds: source.local_bounds,
            content: source.content.clone(),
            properties: source.properties.clone(),
            scroll,
            clip: source.clip,
            opacity: source.opacity,
            effect: source.effect,
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
    pub(crate) fn with_scroll(mut self, scroll: ScrollDeclaration) -> Self {
        self.scroll = Some(scroll);
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

    pub(crate) fn scroll(&self) -> Option<ScrollDeclaration> {
        self.scroll
    }

    pub(crate) fn clip(&self) -> Option<Clip> {
        self.clip
    }

    pub(crate) fn opacity(&self) -> OpacityDeclaration {
        self.opacity
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

    #[cfg(any(test, feature = "renderer-debug"))]
    pub(crate) fn empty(commit: &Commit) -> Result<Self, ContractError> {
        Self::new(commit, PropertySerial::INITIAL, Vec::new(), Vec::new())
    }

    pub(crate) fn snapshot(
        commit: &Commit,
        previous: Option<&Self>,
        serial: PropertySerial,
        values: Vec<PropertyValue>,
    ) -> Result<(Self, bool), ContractError> {
        if let Some(previous) = previous
            && previous.commit == commit.revision
            && previous.values == values
        {
            return Ok((previous.clone(), false));
        }
        let changed =
            if let Some(previous) = previous.filter(|value| value.commit == commit.revision) {
                values
                    .iter()
                    .filter_map(|value| {
                        let property = value.property_ref();
                        (previous.value(property) != Some(*value)).then_some(property)
                    })
                    .collect()
            } else {
                values.iter().map(|value| value.property_ref()).collect()
            };
        Self::new(commit, serial, values, changed).map(|snapshot| (snapshot, true))
    }

    pub(crate) fn project_onto(
        &self,
        commit: &Commit,
        current: &Self,
    ) -> Result<(Self, bool), ContractError> {
        self.project_onto_with_scroll(commit, current, |_, offset| offset)
    }

    pub(super) fn project_onto_with_scroll(
        &self,
        commit: &Commit,
        current: &Self,
        mut project_scroll: impl FnMut(
            composition::tree::NodeId,
            interaction::ScrollOffset,
        ) -> interaction::ScrollOffset,
    ) -> Result<(Self, bool), ContractError> {
        current.require_compatible(commit)?;
        let values = commit
            .property_topology
            .iter()
            .filter_map(|property| {
                self.value(*property)
                    .map(|value| match value {
                        PropertyValue::ScrollOffset { node, value } => {
                            PropertyValue::ScrollOffset {
                                node,
                                value: project_scroll(node, value),
                            }
                        }
                        value => value,
                    })
                    .filter(|value| value.is_valid(commit) && value.is_projectable_onto(commit))
                    .or_else(|| current.value(*property))
            })
            .collect::<Vec<_>>();
        let changed = values
            .iter()
            .filter_map(|value| {
                let property = value.property_ref();
                (current.value(property) != Some(*value)).then_some(property)
            })
            .collect::<Vec<_>>();
        if changed.is_empty() {
            return Ok((current.clone(), false));
        }
        Self::new(commit, self.serial, values, changed).map(|properties| (properties, true))
    }

    pub(crate) fn rebase_onto_for_activation(
        &self,
        commit: &Commit,
        current: &Self,
    ) -> Result<Self, ContractError> {
        current.require_compatible(commit)?;
        let mut values = Vec::with_capacity(commit.property_topology.len());

        for property in commit.property_topology.iter().copied() {
            let candidate = self.value(property);
            let value = if property.kind == PropertyKind::ScrollOffset {
                let value = candidate.ok_or(ContractError::MissingValue(property))?;
                if !value.is_valid(commit) || !value.is_projectable_onto(commit) {
                    return Err(ContractError::InvalidValue(property));
                }
                value
            } else {
                candidate
                    .filter(|value| value.is_valid(commit) && value.is_projectable_onto(commit))
                    .or_else(|| current.value(property))
                    .ok_or(ContractError::MissingValue(property))?
            };
            values.push(value);
        }

        let changed = values
            .iter()
            .filter_map(|value| {
                let property = value.property_ref();
                (current.value(property) != Some(*value)).then_some(property)
            })
            .collect();
        Self::new(commit, self.serial, values, changed)
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

    pub(crate) fn with_commit_revision(mut self, commit: &Commit) -> Self {
        self.commit = commit.revision;
        self
    }

    pub(crate) fn value(&self, property: PropertyRef) -> Option<PropertyValue> {
        self.values
            .iter()
            .copied()
            .find(|value| value.property_ref() == property)
    }

    pub(crate) fn scroll_offset(
        &self,
        node: composition::tree::NodeId,
    ) -> Option<interaction::ScrollOffset> {
        match self.value(PropertyRef::new(node, PropertyKind::ScrollOffset)) {
            Some(PropertyValue::ScrollOffset { value, .. }) => Some(value),
            _ => None,
        }
    }

    pub(crate) fn scrollbar(&self, node: composition::tree::NodeId) -> Option<(f32, f32)> {
        match self.value(PropertyRef::new(node, PropertyKind::Scrollbar)) {
            Some(PropertyValue::Scrollbar {
                opacity, thickness, ..
            }) => Some((opacity, thickness)),
            _ => None,
        }
    }

    pub(crate) fn serial(&self) -> PropertySerial {
        self.serial
    }

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
            Self::Scrollbar { node, .. } => PropertyRef::new(node, PropertyKind::Scrollbar),
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
            Self::ScrollOffset { value, .. } => {
                node.scroll.is_some_and(|scroll| scroll.accepts(value))
            }
            Self::Opacity { value, .. } => value.is_finite() && (0.0..=1.0).contains(&value),
            Self::Clip { rect, .. } => rect_is_within(rect, node.local_bounds),
            Self::Blur { sigma, .. } => match node.effect {
                EffectDeclaration::Blur { maximum_sigma, .. } => {
                    sigma.is_finite() && sigma >= 0.0 && sigma <= maximum_sigma
                }
                _ => false,
            },
            Self::Scrollbar {
                opacity, thickness, ..
            } => {
                opacity.is_finite()
                    && (0.0..=1.0).contains(&opacity)
                    && thickness.is_finite()
                    && thickness >= 1.0
                    && commit.order.as_deref().is_some_and(|order| {
                        order.iter().any(|draw| match draw {
                            Draw::Content {
                                node: owner,
                                projection,
                                ..
                            } if *owner == node.id => projection
                                .maximum_thickness()
                                .is_some_and(|maximum| thickness <= maximum as f32),
                            _ => false,
                        })
                    })
            }
        }
    }

    fn is_projectable_onto(self, commit: &Commit) -> bool {
        let Self::ScrollOffset { node, value } = self else {
            return true;
        };
        commit
            .nodes
            .iter()
            .find(|candidate| candidate.id == node)
            .and_then(|node| node.scroll)
            .is_some_and(|scroll| scroll.accepts(value))
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

impl ScrollDeclaration {
    pub(crate) fn new(
        viewport: geometry::Rect,
        content_bounds: geometry::Rect,
        resident_bounds: geometry::Rect,
        baseline: interaction::ScrollOffset,
        maximum: interaction::ScrollOffset,
    ) -> Option<Self> {
        (viewport.width() > 0
            && viewport.height() > 0
            && content_bounds.width() >= 0
            && content_bounds.height() >= 0
            && resident_bounds.width() > 0
            && resident_bounds.height() > 0
            && maximum.x() >= 0
            && maximum.y() >= 0
            && baseline.x() >= 0
            && baseline.y() >= 0
            && baseline.x() <= maximum.x()
            && baseline.y() <= maximum.y())
        .then_some(Self {
            viewport,
            content_bounds,
            resident_bounds,
            baseline,
            maximum,
        })
        .filter(|declaration| declaration.accepts(baseline))
    }

    pub(crate) fn viewport(self) -> geometry::Rect {
        self.viewport
    }

    pub(crate) fn resident_bounds(self) -> geometry::Rect {
        self.resident_bounds
    }

    pub(crate) fn baseline(self) -> interaction::ScrollOffset {
        self.baseline
    }

    fn semantic(self) -> Self {
        Self {
            resident_bounds: self.content_bounds,
            baseline: interaction::ScrollOffset::default(),
            ..self
        }
    }

    fn accepts(self, offset: interaction::ScrollOffset) -> bool {
        if offset.x() < 0
            || offset.y() < 0
            || offset.x() > self.maximum.x()
            || offset.y() > self.maximum.y()
        {
            return false;
        }
        let content = geometry::Rect::new(
            self.content_bounds.x().saturating_sub(offset.x()),
            self.content_bounds.y().saturating_sub(offset.y()),
            self.content_bounds.width(),
            self.content_bounds.height(),
        );
        let required_x = self.viewport.x().max(content.x());
        let required_y = self.viewport.y().max(content.y());
        let required_right = self.viewport.right().min(content.right());
        let required_bottom = self.viewport.bottom().min(content.bottom());
        if required_right <= required_x || required_bottom <= required_y {
            return true;
        }

        let dx = self.baseline.x().saturating_sub(offset.x());
        let dy = self.baseline.y().saturating_sub(offset.y());
        let resident_x = self.resident_bounds.x().saturating_add(dx);
        let resident_y = self.resident_bounds.y().saturating_add(dy);
        resident_x <= required_x
            && resident_y <= required_y
            && resident_x.saturating_add(self.resident_bounds.width()) >= required_right
            && resident_y.saturating_add(self.resident_bounds.height()) >= required_bottom
    }
}

#[cfg(feature = "renderer-debug")]
pub(crate) fn renderer_fixture(case: FixtureCase) -> Result<(Commit, Properties), ContractError> {
    use crate::icon as icons;

    if matches!(case, FixtureCase::OrderedGroup) {
        return renderer_ordered_group_fixture();
    }
    if matches!(case, FixtureCase::Scroll) {
        return renderer_scroll_fixture();
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
                value: interaction::ScrollOffset::new(2, -1),
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
        FixtureCase::Scroll => unreachable!("scroll fixture returned before node projection"),
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
fn renderer_scroll_fixture() -> Result<(Commit, Properties), ContractError> {
    let before = composition::tree::NodeId::renderer_fixture(40);
    let outer = composition::tree::NodeId::renderer_fixture(41);
    let inner = composition::tree::NodeId::renderer_fixture(42);
    let after = composition::tree::NodeId::renderer_fixture(43);
    let size = geometry::Size::new(64, 64);
    let viewport = geometry::Rect::new(8, 10, 48, 40);
    let outer_bounds = geometry::Rect::new(8, 10, 80, 40);
    let inner_bounds = geometry::Rect::new(8, 10, 48, 80);
    let outer_declaration = ScrollDeclaration::new(
        viewport,
        geometry::Rect::new(viewport.x(), viewport.y(), 80, 40),
        outer_bounds,
        interaction::ScrollOffset::new(4, 0),
        interaction::ScrollOffset::new(32, 0),
    )
    .expect("renderer outer-scroll fixture has a nonempty envelope");
    let inner_declaration = ScrollDeclaration::new(
        viewport,
        geometry::Rect::new(viewport.x(), viewport.y(), 48, 80),
        inner_bounds,
        interaction::ScrollOffset::new(0, 12),
        interaction::ScrollOffset::new(0, 40),
    )
    .expect("renderer inner-scroll fixture has a nonempty envelope");
    let before_node = Node::new(
        before,
        None,
        geometry::Rect::new(2, 2, 60, 6),
        vec![Content::Quad(Quad::new(
            geometry::Rect::new(2, 2, 60, 6),
            Color::rgba(26, 217, 115, 255),
        ))],
    );
    let outer_node = Node::new(
        outer,
        None,
        outer_bounds,
        vec![
            Content::Quad(Quad::new(
                geometry::Rect::new(8, 10, 48, 5),
                Color::rgba(217, 230, 242, 255),
            )),
            Content::Quad(Quad::new(
                geometry::Rect::new(8, 48, 48, 2),
                Color::rgba(179, 191, 204, 255),
            )),
        ],
    )
    .with_properties([PropertyKind::ScrollOffset])
    .with_scroll(outer_declaration);
    let inner_node = Node::new(
        inner,
        Some(outer),
        inner_bounds,
        vec![
            Content::Quad(Quad::new(
                geometry::Rect::new(8, 16, 48, 28),
                Color::rgba(230, 51, 38, 255),
            )),
            Content::Quad(Quad::new(
                geometry::Rect::new(8, 50, 48, 32),
                Color::rgba(38, 102, 230, 255),
            )),
        ],
    )
    .with_properties([PropertyKind::ScrollOffset])
    .with_scroll(inner_declaration);
    let after_node = Node::new(
        after,
        None,
        geometry::Rect::new(2, 56, 60, 6),
        vec![Content::Quad(Quad::new(
            geometry::Rect::new(2, 56, 60, 6),
            Color::rgba(230, 204, 38, 255),
        ))],
    );
    let commit = Commit::from_parts(
        Revision::INITIAL,
        size,
        Color::rgba(0, 0, 0, 0),
        vec![
            Arc::new(before_node),
            Arc::new(outer_node),
            Arc::new(inner_node),
            Arc::new(after_node),
        ],
        Some(vec![
            Draw::PushClip {
                node: None,
                clip: Clip::new(geometry::Rect::new(1, 1, 62, 62))
                    .with_rounding(Rounding::fixed(3.0)),
            },
            Draw::Content {
                node: before,
                index: 0,
                projection: ContentProjection::Normal,
            },
            Draw::PushGroup {
                node: outer,
                bounds: geometry::Rect::from_size(size),
                opacity: 0.82,
            },
            Draw::PushScroll { node: outer },
            Draw::Content {
                node: outer,
                index: 0,
                projection: ContentProjection::Normal,
            },
            Draw::PushScroll { node: inner },
            Draw::Content {
                node: inner,
                index: 0,
                projection: ContentProjection::Normal,
            },
            Draw::PopScroll,
            Draw::PopScroll,
            Draw::PushScroll { node: outer },
            Draw::Content {
                node: outer,
                index: 1,
                projection: ContentProjection::Normal,
            },
            Draw::PushScroll { node: inner },
            Draw::Content {
                node: inner,
                index: 1,
                projection: ContentProjection::Normal,
            },
            Draw::PopScroll,
            Draw::PopScroll,
            Draw::PopGroup,
            Draw::Content {
                node: after,
                index: 0,
                projection: ContentProjection::Normal,
            },
            Draw::PopClip,
        ]),
        Vec::new(),
    )?;
    let properties = renderer_scroll_properties(&commit, 12, 1)?;
    Ok((commit, properties))
}

#[cfg(feature = "renderer-debug")]
pub(crate) fn renderer_scroll_properties(
    commit: &Commit,
    offset_y: i32,
    serial: u64,
) -> Result<Properties, ContractError> {
    let outer = composition::tree::NodeId::renderer_fixture(41);
    let inner = composition::tree::NodeId::renderer_fixture(42);
    Properties::new(
        commit,
        PropertySerial(serial.max(1)),
        vec![
            PropertyValue::ScrollOffset {
                node: outer,
                value: interaction::ScrollOffset::new(4, 0),
            },
            PropertyValue::ScrollOffset {
                node: inner,
                value: interaction::ScrollOffset::new(0, offset_y),
            },
        ],
        vec![
            PropertyRef::new(outer, PropertyKind::ScrollOffset),
            PropertyRef::new(inner, PropertyKind::ScrollOffset),
        ],
    )
}

#[cfg(feature = "renderer-debug")]
pub(crate) fn renderer_scroll_text_runway_pair()
-> Result<((Commit, Properties, Properties), (Commit, Properties)), ContractError> {
    let scroll = composition::tree::NodeId::renderer_fixture(44);
    let expected = composition::tree::NodeId::renderer_fixture(45);
    let size = geometry::Size::new(96, 64);
    let viewport = geometry::Rect::from_size(size);
    let content_bounds = geometry::Rect::new(0, 0, 96, 96);
    let declaration = ScrollDeclaration::new(
        viewport,
        content_bounds,
        content_bounds,
        interaction::ScrollOffset::default(),
        interaction::ScrollOffset::new(0, 32),
    )
    .expect("renderer text-runway fixture has complete resident coverage");
    let runway_rect = geometry::Rect::new(8, 68, 80, 18);
    let visible_rect = geometry::Rect::new(8, 44, 80, 18);
    let background = Color::rgba(24, 72, 128, 255);
    let foreground = Color::rgba(248, 250, 255, 255);
    let runway = Node::new(
        scroll,
        None,
        content_bounds,
        vec![
            Content::Quad(Quad::new(runway_rect, background)),
            Content::Text(Text::new(runway_rect, "RUNWAY", foreground, TextWrap::None)),
        ],
    )
    .with_properties([PropertyKind::ScrollOffset])
    .with_scroll(declaration);
    let actual = Commit::from_parts(
        Revision::renderer_fixture(44),
        size,
        Color::rgba(0, 0, 0, 255),
        vec![Arc::new(runway)],
        Some(vec![
            Draw::PushScroll { node: scroll },
            Draw::Content {
                node: scroll,
                index: 0,
                projection: ContentProjection::Normal,
            },
            Draw::Content {
                node: scroll,
                index: 1,
                projection: ContentProjection::Normal,
            },
            Draw::PopScroll,
        ]),
        Vec::new(),
    )?;
    let initial = Properties::new(
        &actual,
        PropertySerial::INITIAL,
        vec![PropertyValue::ScrollOffset {
            node: scroll,
            value: interaction::ScrollOffset::default(),
        }],
        Vec::new(),
    )?;
    let tick = Properties::new(
        &actual,
        PropertySerial::INITIAL.next(),
        vec![PropertyValue::ScrollOffset {
            node: scroll,
            value: interaction::ScrollOffset::new(0, 24),
        }],
        vec![PropertyRef::new(scroll, PropertyKind::ScrollOffset)],
    )?;

    let expected = Commit::new(
        Revision::renderer_fixture(45),
        size,
        Color::rgba(0, 0, 0, 255),
        vec![Node::new(
            expected,
            None,
            viewport,
            vec![
                Content::Quad(Quad::new(visible_rect, background)),
                Content::Text(Text::new(
                    visible_rect,
                    "RUNWAY",
                    foreground,
                    TextWrap::None,
                )),
            ],
        )],
    )?;
    let expected_properties = Properties::empty(&expected)?;

    Ok(((actual, initial, tick), (expected, expected_properties)))
}

#[cfg(feature = "renderer-debug")]
pub(crate) fn renderer_scroll_semantic_pair()
-> Result<((Commit, Properties), (Commit, Properties)), ContractError> {
    let (initial, initial_properties) = renderer_scroll_fixture()?;
    let mut changed_nodes = initial.nodes.clone();
    changed_nodes[0] = Arc::new(changed_nodes[0].as_ref().clone().with_content_revision(2));
    let changed = Commit::from_parts(
        Revision::renderer_fixture(2),
        initial.size,
        initial.clear,
        changed_nodes,
        initial.order.clone(),
        initial.material_regions.clone(),
    )?;
    let changed_properties = renderer_scroll_properties(&changed, 12, 2)?;
    Ok(((initial, initial_properties), (changed, changed_properties)))
}

#[cfg(feature = "renderer-debug")]
pub(crate) fn renderer_scroll_layer_semantic_pair()
-> Result<((Commit, Properties), (Commit, Properties)), ContractError> {
    let (initial, initial_properties) = renderer_scroll_fixture()?;
    let mut changed_nodes = initial.nodes.clone();
    changed_nodes[2] = Arc::new(changed_nodes[2].as_ref().clone().with_content_revision(2));
    let changed = Commit::from_parts(
        Revision::renderer_fixture(2),
        initial.size,
        initial.clear,
        changed_nodes,
        initial.order.clone(),
        initial.material_regions.clone(),
    )?;
    let changed_properties = renderer_scroll_properties(&changed, 12, 2)?;
    Ok(((initial, initial_properties), (changed, changed_properties)))
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

#[cfg(feature = "renderer-debug")]
pub(crate) fn renderer_text_atlas_pressure_pair()
-> Result<((Commit, Properties), (Commit, Properties)), ContractError> {
    const WIDTH: i32 = 1024;
    const HEIGHT: i32 = 768;
    let size = geometry::Size::new(WIDTH, HEIGHT);
    let bounds = geometry::Rect::new(0, 0, WIDTH, HEIGHT);
    let active = Commit::new(
        Revision::renderer_fixture(301),
        size,
        Color::rgba(12, 16, 24, 255),
        vec![Node::new(
            composition::tree::NodeId::renderer_fixture(301),
            None,
            bounds,
            vec![Content::Text(
                Text::new(
                    geometry::Rect::new(48, 48, 928, 96),
                    "ACTIVE RETAINED GLYPHS 0123456789",
                    Color::rgba(238, 242, 250, 255),
                    TextWrap::None,
                )
                .with_style(TextStyle::new(40.0, crate::text::document::Weight::Normal)),
            )],
        )],
    )?;
    let active_properties = Properties::empty(&active)?;

    let printable = (33_u8..=126).map(char::from).collect::<String>();
    let chunks = printable.as_bytes().chunks(24).collect::<Vec<_>>();
    let mut nodes = Vec::new();
    let mut fixture_id = 400_u64;
    for (row, size_px) in (17_u32..=95).step_by(2).enumerate() {
        for (column, chunk) in chunks.iter().enumerate() {
            fixture_id += 1;
            let value = std::str::from_utf8(chunk)
                .expect("the renderer atlas fixture contains only printable ASCII");
            nodes.push(Node::new(
                composition::tree::NodeId::renderer_fixture(fixture_id),
                None,
                bounds,
                vec![Content::Text(
                    Text::new(
                        geometry::Rect::new(
                            8 + column as i32 * 254,
                            8 + (row % 8) as i32 * 94,
                            246,
                            90,
                        ),
                        value,
                        Color::rgba(220, 226, 238, 255),
                        TextWrap::None,
                    )
                    .with_style(TextStyle::new(
                        size_px as f32,
                        crate::text::document::Weight::Normal,
                    )),
                )],
            ));
        }
    }
    let pressure = Commit::new(
        Revision::renderer_fixture(302),
        size,
        Color::rgba(20, 12, 18, 255),
        nodes,
    )?;
    let pressure_properties = Properties::empty(&pressure)?;

    Ok(((active, active_properties), (pressure, pressure_properties)))
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

    #[test]
    fn candidate_property_values_project_onto_the_active_commit_clock() {
        let node = empty_node(1, None).with_properties([PropertyKind::Opacity]);
        let node_id = node.id();
        let active = Commit::new(
            Revision::INITIAL,
            geometry::Size::new(20, 10),
            Color::rgba(0, 0, 0, 0),
            vec![node.clone()],
        )
        .expect("active commit should be valid");
        let candidate = Commit::new(
            Revision::INITIAL.next(),
            geometry::Size::new(20, 10),
            Color::rgba(0, 0, 0, 0),
            vec![node],
        )
        .expect("candidate commit should be valid");
        let current = Properties::new(
            &active,
            PropertySerial::INITIAL,
            vec![PropertyValue::Opacity {
                node: node_id,
                value: 0.5,
            }],
            Vec::new(),
        )
        .expect("active properties should be complete");
        let candidate_properties = Properties::new(
            &candidate,
            PropertySerial::INITIAL.next(),
            vec![PropertyValue::Opacity {
                node: node_id,
                value: 0.75,
            }],
            vec![PropertyRef::new(node_id, PropertyKind::Opacity)],
        )
        .expect("candidate properties should be complete");

        let (projected, changed) = candidate_properties
            .project_onto(&active, &current)
            .expect("shared property values should project onto the active topology");

        assert!(changed);
        assert!(projected.require_compatible(&active).is_ok());
        assert_eq!(projected.serial(), candidate_properties.serial());
        assert_eq!(projected.changed().len(), 1);
        assert_eq!(
            projected.value(PropertyRef::new(node_id, PropertyKind::Opacity)),
            Some(PropertyValue::Opacity {
                node: node_id,
                value: 0.75,
            })
        );
    }

    #[test]
    fn candidate_scroll_projects_only_while_the_resident_window_covers_its_viewport() {
        let viewport = geometry::Rect::new(0, 0, 100, 100);
        let declaration = ScrollDeclaration::new(
            viewport,
            geometry::Rect::new(viewport.x(), viewport.y(), 100, 200),
            geometry::Rect::new(0, 0, 100, 150),
            interaction::ScrollOffset::new(0, 0),
            interaction::ScrollOffset::new(0, 100),
        )
        .expect("scroll declaration should be valid");
        let node = empty_node(1, None)
            .with_properties([PropertyKind::ScrollOffset])
            .with_scroll(declaration);
        let node_id = node.id();
        let active = Commit::new(
            Revision::INITIAL,
            geometry::Size::new(100, 100),
            Color::rgba(0, 0, 0, 0),
            vec![node.clone()],
        )
        .expect("active scroll commit should be valid");
        let candidate_declaration = ScrollDeclaration::new(
            viewport,
            geometry::Rect::new(viewport.x(), viewport.y(), 100, 200),
            geometry::Rect::new(0, -40, 100, 200),
            interaction::ScrollOffset::new(0, 60),
            interaction::ScrollOffset::new(0, 100),
        )
        .expect("candidate scroll declaration should cover its rebased viewport");
        let candidate_node = node.clone().with_scroll(candidate_declaration);
        let candidate = Commit::new(
            Revision::INITIAL.next(),
            geometry::Size::new(100, 100),
            Color::rgba(0, 0, 0, 0),
            vec![candidate_node],
        )
        .expect("candidate scroll commit should be valid");
        let current = Properties::new(
            &active,
            PropertySerial::INITIAL,
            vec![PropertyValue::ScrollOffset {
                node: node_id,
                value: interaction::ScrollOffset::new(0, 0),
            }],
            Vec::new(),
        )
        .expect("active scroll properties should be complete");
        let in_window = Properties::new(
            &candidate,
            PropertySerial::INITIAL.next(),
            vec![PropertyValue::ScrollOffset {
                node: node_id,
                value: interaction::ScrollOffset::new(0, 24),
            }],
            vec![PropertyRef::new(node_id, PropertyKind::ScrollOffset)],
        )
        .expect("in-window candidate scroll should be valid");
        let (projected, changed) = in_window
            .project_onto(&active, &current)
            .expect("in-window scroll should project onto active structure");
        assert!(changed);
        assert_eq!(
            projected.scroll_offset(node_id),
            Some(interaction::ScrollOffset::new(0, 24))
        );

        let beyond_guard = Properties::new(
            &candidate,
            PropertySerial::INITIAL.next(),
            vec![PropertyValue::ScrollOffset {
                node: node_id,
                value: interaction::ScrollOffset::new(0, 60),
            }],
            vec![PropertyRef::new(node_id, PropertyKind::ScrollOffset)],
        )
        .expect("candidate scroll must remain inside its own rebased guard");
        let (projected, changed) = beyond_guard
            .project_onto(&active, &current)
            .expect("out-of-window candidate should preserve complete active properties");
        assert!(!changed);
        assert_eq!(projected, current);
        assert!(
            Properties::new(
                &active,
                PropertySerial::INITIAL.next(),
                vec![PropertyValue::ScrollOffset {
                    node: node_id,
                    value: interaction::ScrollOffset::new(0, 60),
                }],
                vec![PropertyRef::new(node_id, PropertyKind::ScrollOffset)],
            )
            .is_err(),
            "an active commit must reject pixels outside its retained scroll envelope"
        );
    }

    #[test]
    fn short_scroll_content_requires_only_its_visible_content_pixels() {
        let viewport = geometry::Rect::new(0, 0, 100, 100);
        let content = geometry::Rect::new(0, 0, 100, 60);
        let resident = geometry::Rect::new(0, 0, 100, 60);

        let declaration = ScrollDeclaration::new(
            viewport,
            content,
            resident,
            interaction::ScrollOffset::default(),
            interaction::ScrollOffset::default(),
        )
        .expect("a short content extent must not manufacture residency for its blank tail");
        assert!(declaration.accepts(interaction::ScrollOffset::default()));
        assert!(
            !declaration.accepts(interaction::ScrollOffset::new(0, 1)),
            "blank coverage cannot legitimize an offset beyond the integral maximum"
        );
    }

    #[test]
    fn scroll_property_preserves_absolute_offsets_beyond_float_integer_precision() {
        let viewport = geometry::Rect::new(0, 0, 100, 100);

        for y in [
            16_777_215, 16_777_216, 16_777_217, 23_999_897, 23_999_898, 23_999_899,
        ] {
            let offset = interaction::ScrollOffset::new(0, y);
            let declaration = ScrollDeclaration::new(
                viewport,
                geometry::Rect::new(viewport.x(), viewport.y(), 100, y.saturating_add(100)),
                viewport,
                offset,
                interaction::ScrollOffset::new(0, y),
            )
            .expect("resident pixels should cover their exact large-offset baseline");
            let node = empty_node(1, None)
                .with_properties([PropertyKind::ScrollOffset])
                .with_scroll(declaration);
            let node_id = node.id();
            let commit = Commit::new(
                Revision::INITIAL,
                geometry::Size::new(100, 100),
                Color::rgba(0, 0, 0, 0),
                vec![node],
            )
            .expect("large-offset scroll commit should be valid");
            let properties = Properties::new(
                &commit,
                PropertySerial::INITIAL,
                vec![PropertyValue::ScrollOffset {
                    node: node_id,
                    value: offset,
                }],
                Vec::new(),
            )
            .expect("scene properties must not round-trip absolute scroll through f32");

            assert_eq!(properties.scroll_offset(node_id), Some(offset));
        }
    }

    #[test]
    fn scrollbar_projection_uses_one_axis_generic_integral_ratio() {
        let vertical = ContentProjection::ScrollbarThumb {
            axis: interaction::ScrollbarAxis::Vertical,
            edge: 100,
            base_thickness: 4,
            maximum_thickness: 8,
            baseline_start: 0,
            baseline_extent: 20,
            baseline_position: 0,
            travel: 401,
            maximum_offset: 23_999_900,
        };
        let horizontal = ContentProjection::ScrollbarThumb {
            axis: interaction::ScrollbarAxis::Horizontal,
            edge: 100,
            base_thickness: 4,
            maximum_thickness: 8,
            baseline_start: 0,
            baseline_extent: 20,
            baseline_position: 0,
            travel: 401,
            maximum_offset: 23_999_900,
        };

        assert_eq!(
            vertical.scrollbar_position(interaction::ScrollOffset::new(0, 12_000_000)),
            Some(201)
        );
        assert_eq!(
            horizontal.scrollbar_position(interaction::ScrollOffset::new(12_000_000, 0)),
            Some(201)
        );
        assert_eq!(
            vertical.scrollbar_position(interaction::ScrollOffset::new(12_000_000, 0)),
            Some(0),
            "vertical projection must read only vertical truth"
        );
        assert_eq!(
            horizontal.scrollbar_position(interaction::ScrollOffset::new(0, 12_000_000)),
            Some(0),
            "horizontal projection must read only horizontal truth"
        );
        assert_eq!(
            vertical.scrollbar_position(interaction::ScrollOffset::new(0, 23_999_899)),
            Some(401)
        );
        assert_eq!(
            vertical.scrollbar_position(interaction::ScrollOffset::new(0, 23_999_900)),
            Some(401)
        );
    }

    #[test]
    fn pending_activation_requires_latest_scroll_to_fit_prepared_residency() {
        let viewport = geometry::Rect::new(0, 0, 100, 100);
        let prepared_declaration = ScrollDeclaration::new(
            viewport,
            geometry::Rect::new(viewport.x(), viewport.y(), 100, 260),
            geometry::Rect::new(0, 0, 100, 200),
            interaction::ScrollOffset::new(0, 0),
            interaction::ScrollOffset::new(0, 160),
        )
        .expect("prepared structure should admit a bounded forward rebase");
        let prepared_node = empty_node(1, None)
            .with_properties([PropertyKind::ScrollOffset])
            .with_scroll(prepared_declaration);
        let node_id = prepared_node.id();
        let prepared = Commit::new(
            Revision::INITIAL,
            geometry::Size::new(100, 100),
            Color::rgba(0, 0, 0, 0),
            vec![prepared_node],
        )
        .expect("prepared commit should be valid");
        let prepared_properties = Properties::new(
            &prepared,
            PropertySerial::INITIAL,
            vec![PropertyValue::ScrollOffset {
                node: node_id,
                value: interaction::ScrollOffset::new(0, 0),
            }],
            Vec::new(),
        )
        .expect("prepared properties should be valid");

        let latest_offset = interaction::ScrollOffset::new(0, 60);
        let latest_node = empty_node(1, None)
            .with_properties([PropertyKind::ScrollOffset])
            .with_scroll(
                ScrollDeclaration::new(
                    viewport,
                    geometry::Rect::new(viewport.x(), viewport.y(), 100, 160),
                    viewport,
                    latest_offset,
                    latest_offset,
                )
                .expect("latest commit should cover its baseline"),
            );
        let latest = Commit::new(
            Revision::INITIAL.next(),
            geometry::Size::new(100, 100),
            Color::rgba(0, 0, 0, 0),
            vec![latest_node],
        )
        .expect("latest commit should be valid");
        let latest_properties = Properties::new(
            &latest,
            PropertySerial::INITIAL.next(),
            vec![PropertyValue::ScrollOffset {
                node: node_id,
                value: latest_offset,
            }],
            Vec::new(),
        )
        .expect("latest properties should be valid");

        let rebased = latest_properties
            .rebase_onto_for_activation(&prepared, &prepared_properties)
            .expect("latest scroll inside prepared residency should rebase exactly");
        assert_eq!(rebased.scroll_offset(node_id), Some(latest_offset));
        assert_eq!(rebased.serial(), latest_properties.serial());

        let beyond_prepared = interaction::ScrollOffset::new(0, 160);
        let beyond_node = empty_node(1, None)
            .with_properties([PropertyKind::ScrollOffset])
            .with_scroll(
                ScrollDeclaration::new(
                    viewport,
                    geometry::Rect::new(viewport.x(), viewport.y(), 100, 260),
                    viewport,
                    beyond_prepared,
                    beyond_prepared,
                )
                .expect("successor should cover its own baseline"),
            );
        let beyond = Commit::new(
            Revision::INITIAL.next().next(),
            geometry::Size::new(100, 100),
            Color::rgba(0, 0, 0, 0),
            vec![beyond_node],
        )
        .expect("successor commit should be valid");
        let beyond_properties = Properties::new(
            &beyond,
            latest_properties.serial().next(),
            vec![PropertyValue::ScrollOffset {
                node: node_id,
                value: beyond_prepared,
            }],
            Vec::new(),
        )
        .expect("successor properties should fit their own commit");
        assert!(matches!(
            beyond_properties.rebase_onto_for_activation(&prepared, &prepared_properties),
            Err(ContractError::InvalidValue(_))
        ));

        let without_scroll = Commit::new(
            Revision::INITIAL.next().next().next(),
            geometry::Size::new(100, 100),
            Color::rgba(0, 0, 0, 0),
            vec![empty_node(1, None)],
        )
        .expect("successor without scroll should be valid");
        let without_scroll_properties = Properties::empty(&without_scroll)
            .expect("successor without scroll should need no values");
        assert!(matches!(
            without_scroll_properties.rebase_onto_for_activation(&prepared, &prepared_properties),
            Err(ContractError::MissingValue(_))
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
        let public_scene = commit
            .compatibility_scene(&properties)
            .expect("material commit should project its public scene snapshot");

        assert_eq!(public_scene.material_regions().len(), 1);
        assert_eq!(public_scene.material_regions()[0].id(), owner);
        assert_eq!(public_scene.panes()[0].region_id(), Some(owner));
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
