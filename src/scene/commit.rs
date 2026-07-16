use std::{
    collections::{HashMap, HashSet},
    sync::Arc,
};

use thiserror::Error;

use super::super::{composition, geometry, interaction};
#[cfg(feature = "renderer-debug")]
use super::{Brush, Glass, Material, Offset, Rasterization, Rounding, Style, TextStyle, TextWrap};
use super::{
    Clip, Color, Icon, Outline, Pane, Primitive, Quad, Rule, Scene, Shadow, Text, TextViewport,
    Transform, region::MaterialRegion,
};

const PROPERTY_BLOCK_LEN: usize = 256;

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
    node_indices: HashMap<composition::tree::NodeId, usize>,
    property_topology: Vec<PropertyRef>,
    property_indices: Arc<HashMap<PropertyRef, PropertyIndex>>,
    spatial_topology: super::spatial::SpatialTopology,
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
    scroll_target: Option<super::spatial::ScrollTarget>,
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
    scroll_target: Option<super::spatial::ScrollTarget>,
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
    Caret,
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
    Caret,
    HorizontalScrollbar,
    VerticalScrollbar,
}

impl PropertyKind {
    pub(crate) const fn scrollbar(axis: interaction::ScrollbarAxis) -> Self {
        match axis {
            interaction::ScrollbarAxis::Horizontal => Self::HorizontalScrollbar,
            interaction::ScrollbarAxis::Vertical => Self::VerticalScrollbar,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub(crate) struct PropertyRef {
    node: composition::tree::NodeId,
    kind: PropertyKind,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub(crate) struct PropertyIndex(u32);

#[derive(Debug, Clone, PartialEq)]
struct PropertyValues {
    blocks: Arc<[Arc<[PropertyValue]>]>,
    len: usize,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub(crate) struct PropertyWork {
    value_visits: usize,
    index_lookups: usize,
}

#[derive(Debug, Clone, PartialEq)]
pub(crate) struct Properties {
    commit: Revision,
    serial: PropertySerial,
    predecessor: Option<PropertySerial>,
    indices: Arc<HashMap<PropertyRef, PropertyIndex>>,
    values: PropertyValues,
    changed: Vec<PropertyIndex>,
    work: PropertyWork,
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
    Caret {
        node: composition::tree::NodeId,
        visible: bool,
    },
    Scrollbar {
        node: composition::tree::NodeId,
        axis: interaction::ScrollbarAxis,
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
    #[error("scene commit exceeds its addressable property count")]
    TooManyProperties,
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
    #[error(transparent)]
    InvalidSpatialTopology(#[from] super::spatial::SpatialError),
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
        let mut node_indices = HashMap::with_capacity(nodes.len());
        let mut property_topology = Vec::new();
        let mut property_indices = HashMap::new();

        for (node_index, node) in nodes.iter().enumerate() {
            if node_indices.insert(node.id, node_index).is_some() {
                return Err(ContractError::DuplicateNode(node.id));
            }
            if let Some(parent) = node.parent
                && !node_indices.contains_key(&parent)
            {
                return Err(ContractError::UnknownParent {
                    node: node.id,
                    parent,
                });
            }
            for kind in &node.properties {
                let property = PropertyRef::new(node.id, *kind);
                let index = PropertyIndex::from_usize(property_topology.len())?;
                if property_indices.insert(property, index).is_some() {
                    return Err(ContractError::DuplicateProperty {
                        node: node.id,
                        kind: *kind,
                    });
                }
                property_topology.push(property);
            }
        }

        let spatial_topology = super::spatial::SpatialTopology::compile(&nodes, order.as_deref())?;

        Ok(Self {
            revision,
            size: size.sanitized(),
            clear,
            nodes,
            node_indices,
            property_topology,
            property_indices: Arc::new(property_indices),
            spatial_topology,
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

    pub(crate) fn node(&self, id: composition::tree::NodeId) -> Option<&Arc<Node>> {
        self.node_indices
            .get(&id)
            .and_then(|index| self.nodes.get(*index))
    }

    pub(crate) fn property_index(&self, property: PropertyRef) -> Option<PropertyIndex> {
        self.property_indices.get(&property).copied()
    }

    pub(crate) fn property_count(&self) -> usize {
        self.property_topology.len()
    }

    #[cfg(any(test, feature = "renderer-debug"))]
    pub(crate) fn property_topology(&self) -> &[PropertyRef] {
        &self.property_topology
    }

    pub(crate) fn order(&self) -> Option<&[Draw]> {
        self.order.as_deref()
    }

    pub(crate) fn spatial_topology(&self) -> &super::spatial::SpatialTopology {
        &self.spatial_topology
    }

    pub(crate) fn semantic_projection(
        candidate: &Arc<Self>,
        previous: Option<&Arc<Self>>,
        resident_nodes: &HashSet<composition::tree::NodeId>,
        resident_scrolls: &HashSet<composition::tree::NodeId>,
    ) -> Result<Arc<Self>, ContractError> {
        let has_transient_projection = candidate.order.as_ref().is_some_and(|order| {
            order.iter().any(|draw| {
                matches!(
                    draw,
                    Draw::Content {
                        projection,
                        ..
                    } if *projection != ContentProjection::Normal
                )
            })
        });
        if resident_nodes.is_empty() && resident_scrolls.is_empty() && !has_transient_projection {
            return Ok(Arc::clone(candidate));
        }
        let mut order = candidate.order.as_ref().map(|order| {
            candidate.spatial_topology.project_semantic_order(
                order,
                resident_nodes,
                resident_scrolls,
            )
        });
        let semantic_content_overrides = order
            .as_mut()
            .map(|order| semanticize_transient_content(candidate, order))
            .unwrap_or_default();
        let retained_content = order.as_ref().map(|order| semantic_content_indices(order));
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
                    retained_content
                        .as_ref()
                        .map(|indices| indices.get(&node.id).map_or(&[][..], Vec::as_slice)),
                    &semantic_content_overrides,
                )
            })
            .collect::<Vec<_>>();
        if let (Some(order), Some(indices)) = (&mut order, &retained_content) {
            remap_semantic_content_indices(order, indices);
        }
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
        Ok(Scene {
            size: self.size,
            clear: self.clear,
            primitives: self
                .spatial_topology
                .compatibility_primitives(self, properties),
            material_regions: self.material_regions.clone(),
        })
    }

    fn same_projection(&self, other: &Self) -> bool {
        self.size == other.size
            && self.clear == other.clear
            && self.nodes == other.nodes
            && self.property_topology == other.property_topology
            && self.spatial_topology == other.spatial_topology
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
}

fn semanticize_transient_content(
    candidate: &Commit,
    order: &mut [Draw],
) -> HashMap<(composition::tree::NodeId, usize), Content> {
    let mut overrides = HashMap::new();
    let nodes = candidate
        .nodes
        .iter()
        .map(|node| (node.id, node))
        .collect::<HashMap<_, _>>();
    for draw in order {
        let Draw::Content {
            node,
            index,
            projection,
        } = draw
        else {
            continue;
        };
        let ContentProjection::ScrollbarThumb {
            axis,
            edge,
            base_thickness,
            maximum_thickness,
            baseline_start,
            baseline_extent,
            baseline_position,
            travel,
            maximum_offset,
        } = *projection
        else {
            continue;
        };
        let Some(Content::Quad(quad)) = nodes.get(node).and_then(|node| node.content.get(*index))
        else {
            continue;
        };
        let track_start = baseline_start.saturating_sub(baseline_position);
        let (dx, dy) = match axis {
            interaction::ScrollbarAxis::Horizontal => (0_i32.saturating_sub(baseline_position), 0),
            interaction::ScrollbarAxis::Vertical => (0, 0_i32.saturating_sub(baseline_position)),
        };
        let rect = quad.rect();
        let rect = geometry::Rect::new(
            rect.x().saturating_add(dx),
            rect.y().saturating_add(dy),
            rect.width(),
            rect.height(),
        );
        overrides.insert((*node, *index), Content::Quad(quad.with_rect(rect)));
        *projection = ContentProjection::ScrollbarThumb {
            axis,
            edge,
            base_thickness,
            maximum_thickness,
            baseline_start: track_start,
            baseline_extent,
            baseline_position: 0,
            travel,
            maximum_offset,
        };
    }
    overrides
}

fn semantic_content_indices(order: &[Draw]) -> HashMap<composition::tree::NodeId, Vec<usize>> {
    let mut indices = HashMap::<_, Vec<_>>::new();
    let mut seen = HashSet::new();
    for draw in order {
        let Draw::Content { node, index, .. } = draw else {
            continue;
        };
        if seen.insert((*node, *index)) {
            indices.entry(*node).or_default().push(*index);
        }
    }
    for node_indices in indices.values_mut() {
        node_indices.sort_unstable();
    }
    indices
}

fn remap_semantic_content_indices(
    order: &mut [Draw],
    retained: &HashMap<composition::tree::NodeId, Vec<usize>>,
) {
    for draw in order {
        let Draw::Content { node, index, .. } = draw else {
            continue;
        };
        *index = retained
            .get(node)
            .and_then(|indices| indices.binary_search(index).ok())
            .expect("semantic order must only name retained content");
    }
}

impl Content {
    pub(crate) fn as_primitive(&self, transform: Option<PropertyValue>) -> Primitive {
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
    pub(crate) fn scrollbar_axis(self) -> Option<interaction::ScrollbarAxis> {
        match self {
            Self::Normal | Self::Caret => None,
            Self::ScrollbarTrack { axis, .. } | Self::ScrollbarThumb { axis, .. } => Some(axis),
        }
    }

    fn maximum_thickness(self) -> Option<i32> {
        match self {
            Self::Normal | Self::Caret => None,
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

    pub(crate) fn project_primitive(
        self,
        primitive: Primitive,
        node: composition::tree::NodeId,
        properties: &Properties,
    ) -> Option<Primitive> {
        let (axis, edge, base_thickness, maximum_thickness) = match self {
            Self::Normal => return Some(primitive),
            Self::Caret => {
                return match properties.value(PropertyRef::new(node, PropertyKind::Caret)) {
                    Some(PropertyValue::Caret { visible: true, .. }) => Some(primitive),
                    Some(PropertyValue::Caret { visible: false, .. }) => None,
                    _ => Some(primitive),
                };
            }
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
        }) = properties.value(PropertyRef::new(node, PropertyKind::scrollbar(axis)))
        else {
            return Some(primitive);
        };
        let Primitive::Quad(quad) = primitive else {
            return Some(primitive);
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
        Some(Primitive::Quad(quad.with_rect(rect).with_opacity(opacity)))
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
            scroll_target: None,
        });
    }

    pub(crate) fn declare_scroll(
        &mut self,
        id: composition::tree::NodeId,
        target: interaction::Target,
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
        node.scroll_target = Some(super::spatial::ScrollTarget::Interaction(target));
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
        let property = match projection {
            ContentProjection::Normal => None,
            ContentProjection::Caret => Some(PropertyKind::Caret),
            projection => projection.scrollbar_axis().map(PropertyKind::scrollbar),
        };
        if let Some(property) = property
            && !self.nodes[node_index].properties.contains(&property)
        {
            self.nodes[node_index].properties.push(property);
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
            scroll_target: None,
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
                && previous.scroll_target == draft.scroll_target
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
            scroll_target: draft.scroll_target,
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

    fn semantic(
        previous: Option<&Arc<Self>>,
        source: &Arc<Self>,
        retained_content: Option<&[usize]>,
        content_overrides: &HashMap<(composition::tree::NodeId, usize), Content>,
    ) -> Arc<Self> {
        let scroll = source.scroll.map(ScrollDeclaration::semantic);
        let content = retained_content.map_or_else(
            || source.content.clone(),
            |indices| {
                indices
                    .iter()
                    .filter_map(|index| {
                        content_overrides
                            .get(&(source.id, *index))
                            .cloned()
                            .or_else(|| source.content.get(*index).cloned())
                    })
                    .collect()
            },
        );
        let content_revision = previous.map_or(source.content_revision, |previous| {
            if previous.content == content {
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
                && previous.scroll_target == source.scroll_target
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
            content,
            properties: source.properties.clone(),
            scroll,
            scroll_target: source.scroll_target.clone(),
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
        self.scroll_target = Some(super::spatial::ScrollTarget::scene_node(self.id));
        self
    }

    #[cfg(any(test, feature = "renderer-debug"))]
    pub(crate) fn with_scroll_target(mut self, target: super::spatial::ScrollTarget) -> Self {
        self.scroll_target = Some(target);
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

    pub(crate) fn scroll_target(&self) -> Option<&super::spatial::ScrollTarget> {
        self.scroll_target.as_ref()
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

impl PropertyIndex {
    fn from_usize(index: usize) -> Result<Self, ContractError> {
        u32::try_from(index)
            .map(Self)
            .map_err(|_| ContractError::TooManyProperties)
    }

    pub(crate) const fn as_usize(self) -> usize {
        self.0 as usize
    }
}

impl PropertyValues {
    fn from_vec(values: Vec<PropertyValue>) -> Self {
        let len = values.len();
        let blocks = values
            .chunks(PROPERTY_BLOCK_LEN)
            .map(|block| Arc::<[PropertyValue]>::from(block.to_vec()))
            .collect::<Vec<_>>()
            .into();
        Self { blocks, len }
    }

    fn get(&self, index: PropertyIndex) -> Option<PropertyValue> {
        let index = index.as_usize();
        (index < self.len)
            .then(|| {
                self.blocks
                    .get(index / PROPERTY_BLOCK_LEN)
                    .and_then(|block| block.get(index % PROPERTY_BLOCK_LEN))
                    .copied()
            })
            .flatten()
    }

    fn iter(&self) -> impl Iterator<Item = PropertyValue> + '_ {
        self.blocks.iter().flat_map(|block| block.iter().copied())
    }

    fn len(&self) -> usize {
        self.len
    }
}

impl PropertyWork {
    pub(crate) const fn value_visits(self) -> usize {
        self.value_visits
    }

    pub(crate) const fn index_lookups(self) -> usize {
        self.index_lookups
    }
}

impl Properties {
    pub(crate) fn new(
        commit: &Commit,
        serial: PropertySerial,
        values: Vec<PropertyValue>,
        changed: Vec<PropertyRef>,
    ) -> Result<Self, ContractError> {
        let value_visits = values.len();
        let mut index_lookups = 0_usize;
        let mut indexed = vec![None; commit.property_count()];
        for value in values {
            let property = value.property_ref();
            index_lookups = index_lookups.saturating_add(1);
            let Some(index) = commit.property_index(property) else {
                return Err(ContractError::UndeclaredValue(property));
            };
            let slot = &mut indexed[index.as_usize()];
            if slot.replace(value).is_some() {
                return Err(ContractError::DuplicateValue(property));
            }
            if !value.is_valid(commit) {
                return Err(ContractError::InvalidValue(property));
            }
        }
        if let Some((index, _)) = indexed
            .iter()
            .enumerate()
            .find(|(_, value)| value.is_none())
        {
            return Err(ContractError::MissingValue(commit.property_topology[index]));
        }

        let mut changed_indices = Vec::with_capacity(changed.len());
        let mut seen_changed = HashSet::new();
        for property in changed {
            index_lookups = index_lookups.saturating_add(1);
            let Some(index) = commit.property_index(property) else {
                return Err(ContractError::UndeclaredChange(property));
            };
            if seen_changed.insert(index) {
                changed_indices.push(index);
            }
        }
        changed_indices.sort_unstable();

        Ok(Self {
            commit: commit.revision,
            serial,
            predecessor: None,
            indices: Arc::clone(&commit.property_indices),
            values: PropertyValues::from_vec(indexed.into_iter().flatten().collect()),
            changed: changed_indices,
            work: PropertyWork {
                value_visits,
                index_lookups,
            },
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
        let mut snapshot = Self::new(commit, serial, values, Vec::new())?;
        let Some(previous) = previous.filter(|value| value.commit == commit.revision) else {
            snapshot.changed = (0..snapshot.values.len())
                .map(|index| PropertyIndex::from_usize(index).expect("validated property index"))
                .collect();
            return Ok((snapshot, true));
        };
        snapshot.predecessor = Some(previous.serial);
        if previous.values == snapshot.values {
            return Ok((previous.clone(), false));
        }
        snapshot.changed = snapshot
            .values
            .iter()
            .enumerate()
            .filter_map(|(index, value)| {
                let index = PropertyIndex::from_usize(index).expect("validated property index");
                (previous.value_at(index) != Some(value)).then_some(index)
            })
            .collect();
        snapshot.work.value_visits = snapshot
            .work
            .value_visits
            .saturating_add(snapshot.values.len());
        Ok((snapshot, true))
    }

    pub(crate) fn apply_updates(
        commit: &Commit,
        previous: &Self,
        serial: PropertySerial,
        updates: Vec<PropertyValue>,
    ) -> Result<(Self, bool), ContractError> {
        previous.require_compatible(commit)?;
        let mut coalesced = HashMap::with_capacity(updates.len());
        let mut work = PropertyWork::default();
        for value in updates {
            work.value_visits = work.value_visits.saturating_add(1);
            work.index_lookups = work.index_lookups.saturating_add(1);
            let property = value.property_ref();
            let Some(index) = commit.property_index(property) else {
                return Err(ContractError::UndeclaredValue(property));
            };
            if !value.is_valid(commit) {
                return Err(ContractError::InvalidValue(property));
            }
            coalesced.insert(index, value);
        }

        let mut changed = coalesced
            .into_iter()
            .filter(|(index, value)| previous.value_at(*index) != Some(*value))
            .collect::<Vec<_>>();
        changed.sort_unstable_by_key(|(index, _)| *index);
        if changed.is_empty() {
            return Ok((previous.clone(), false));
        }

        let mut blocks = previous.values.blocks.as_ref().to_vec();
        let mut cursor = 0;
        while cursor < changed.len() {
            let block_index = changed[cursor].0.as_usize() / PROPERTY_BLOCK_LEN;
            let mut block = blocks[block_index].as_ref().to_vec();
            while cursor < changed.len()
                && changed[cursor].0.as_usize() / PROPERTY_BLOCK_LEN == block_index
            {
                let (index, value) = changed[cursor];
                block[index.as_usize() % PROPERTY_BLOCK_LEN] = value;
                cursor += 1;
            }
            blocks[block_index] = block.into();
        }

        Ok((
            Self {
                commit: commit.revision,
                serial,
                predecessor: Some(previous.serial),
                indices: Arc::clone(&commit.property_indices),
                values: PropertyValues {
                    blocks: blocks.into(),
                    len: previous.values.len(),
                },
                changed: changed.into_iter().map(|(index, _)| index).collect(),
                work,
            },
            true,
        ))
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
        Self::new(commit, self.serial, values, changed).map(|mut properties| {
            properties.predecessor = Some(current.serial);
            (properties, true)
        })
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
        Self::new(commit, self.serial, values, changed).map(|mut properties| {
            properties.predecessor = Some(current.serial);
            properties
        })
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
        self.indices = Arc::clone(&commit.property_indices);
        self
    }

    pub(crate) fn value(&self, property: PropertyRef) -> Option<PropertyValue> {
        self.indices
            .get(&property)
            .and_then(|index| self.values.get(*index))
    }

    pub(crate) fn value_at(&self, index: PropertyIndex) -> Option<PropertyValue> {
        self.values.get(index)
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

    pub(crate) fn scrollbar(
        &self,
        node: composition::tree::NodeId,
        axis: interaction::ScrollbarAxis,
    ) -> Option<(f32, f32)> {
        match self.value(PropertyRef::new(node, PropertyKind::scrollbar(axis))) {
            Some(PropertyValue::Scrollbar {
                opacity, thickness, ..
            }) => Some((opacity, thickness)),
            _ => None,
        }
    }

    pub(crate) fn serial(&self) -> PropertySerial {
        self.serial
    }

    pub(crate) fn predecessor_serial(&self) -> Option<PropertySerial> {
        self.predecessor
    }

    pub(crate) fn changed(&self) -> &[PropertyIndex] {
        &self.changed
    }

    pub(crate) fn changed_values(&self) -> impl Iterator<Item = PropertyValue> + '_ {
        self.changed
            .iter()
            .filter_map(|index| self.value_at(*index))
    }

    pub(crate) fn work(&self) -> PropertyWork {
        self.work
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
            Self::Caret { node, .. } => PropertyRef::new(node, PropertyKind::Caret),
            Self::Scrollbar { node, axis, .. } => {
                PropertyRef::new(node, PropertyKind::scrollbar(axis))
            }
        }
    }

    fn is_valid(self, commit: &Commit) -> bool {
        let Some(node) = commit.node(self.property_ref().node) else {
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
            Self::Caret { .. } => node.declares(PropertyKind::Caret),
            Self::Scrollbar {
                axis,
                opacity,
                thickness,
                ..
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
                            } if *owner == node.id && projection.scrollbar_axis() == Some(axis) => {
                                projection
                                    .maximum_thickness()
                                    .is_some_and(|maximum| thickness <= maximum as f32)
                            }
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
            .node(node)
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

    pub(crate) fn maximum(self) -> interaction::ScrollOffset {
        self.maximum
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
pub(crate) fn renderer_group_under_scroll_pair()
-> Result<((Commit, Properties, Properties), (Commit, Properties)), ContractError> {
    const DELTA: i32 = 20;

    let scroll = composition::tree::NodeId::renderer_fixture(46);
    let group = composition::tree::NodeId::renderer_fixture(47);
    let expected_rule = composition::tree::NodeId::renderer_fixture(48);
    let expected_group = composition::tree::NodeId::renderer_fixture(49);
    let size = geometry::Size::new(112, 64);
    let viewport = geometry::Rect::new(8, 8, 96, 48);
    let content_bounds = geometry::Rect::new(8, 8, 140, 48);
    let declaration = ScrollDeclaration::new(
        viewport,
        content_bounds,
        content_bounds,
        interaction::ScrollOffset::default(),
        interaction::ScrollOffset::new(44, 0),
    )
    .expect("group-under-scroll fixture has complete resident coverage");
    let rule_rect = geometry::Rect::new(92, 12, 4, 40);
    let group_rect = geometry::Rect::new(52, 20, 16, 24);
    let rule_color = Color::rgba(242, 179, 26, 255);
    let group_color = Color::rgba(230, 51, 38, 255);

    let scroll_node = Node::new(
        scroll,
        None,
        content_bounds,
        vec![Content::Rule(Rule::vertical(rule_rect, rule_color, 3))],
    )
    .with_properties([PropertyKind::ScrollOffset])
    .with_scroll(declaration);
    let group_node = Node::new(
        group,
        Some(scroll),
        group_rect,
        vec![Content::Quad(Quad::new(group_rect, group_color))],
    );
    let actual = Commit::from_parts(
        Revision::renderer_fixture(46),
        size,
        Color::rgba(0, 0, 0, 255),
        vec![Arc::new(scroll_node), Arc::new(group_node)],
        Some(vec![
            Draw::PushScroll { node: scroll },
            Draw::Content {
                node: scroll,
                index: 0,
                projection: ContentProjection::Normal,
            },
            Draw::PushGroup {
                node: group,
                bounds: group_rect,
                opacity: 1.0,
            },
            Draw::Content {
                node: group,
                index: 0,
                projection: ContentProjection::Normal,
            },
            Draw::PopGroup,
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
            value: interaction::ScrollOffset::new(DELTA, 0),
        }],
        vec![PropertyRef::new(scroll, PropertyKind::ScrollOffset)],
    )?;

    let translated_rule = geometry::Rect::new(
        rule_rect.x().saturating_sub(DELTA),
        rule_rect.y(),
        rule_rect.width(),
        rule_rect.height(),
    );
    let translated_group = geometry::Rect::new(
        group_rect.x().saturating_sub(DELTA),
        group_rect.y(),
        group_rect.width(),
        group_rect.height(),
    );
    let expected_rule_node = Node::new(
        expected_rule,
        None,
        translated_rule,
        vec![Content::Rule(Rule::vertical(
            translated_rule,
            rule_color,
            3,
        ))],
    );
    let expected_group_node = Node::new(
        expected_group,
        None,
        translated_group,
        vec![Content::Quad(Quad::new(translated_group, group_color))],
    );
    let expected = Commit::from_parts(
        Revision::renderer_fixture(47),
        size,
        Color::rgba(0, 0, 0, 255),
        vec![Arc::new(expected_rule_node), Arc::new(expected_group_node)],
        Some(vec![
            Draw::Content {
                node: expected_rule,
                index: 0,
                projection: ContentProjection::Normal,
            },
            Draw::PushGroup {
                node: expected_group,
                bounds: translated_group,
                opacity: 1.0,
            },
            Draw::Content {
                node: expected_group,
                index: 0,
                projection: ContentProjection::Normal,
            },
            Draw::PopGroup,
        ]),
        Vec::new(),
    )?;
    let expected_properties = Properties::empty(&expected)?;

    Ok(((actual, initial, tick), (expected, expected_properties)))
}

#[cfg(feature = "renderer-debug")]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum ScrollOracleCase {
    F01,
    F02,
    F03,
    F04,
    F05,
    F06,
    F07,
    F08,
}

#[cfg(feature = "renderer-debug")]
impl ScrollOracleCase {
    pub(crate) const ALL: [Self; 8] = [
        Self::F01,
        Self::F02,
        Self::F03,
        Self::F04,
        Self::F05,
        Self::F06,
        Self::F07,
        Self::F08,
    ];

    pub(crate) const fn name(self) -> &'static str {
        match self {
            Self::F01 => "F01",
            Self::F02 => "F02",
            Self::F03 => "F03",
            Self::F04 => "F04",
            Self::F05 => "F05",
            Self::F06 => "F06",
            Self::F07 => "F07",
            Self::F08 => "F08",
        }
    }
}

#[cfg(feature = "renderer-debug")]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct ScrollOracleRegion {
    pub(crate) name: &'static str,
    pub(crate) initial: geometry::Rect,
    pub(crate) translated: geometry::Rect,
}

#[cfg(feature = "renderer-debug")]
pub(crate) struct ScrollOracleFixture {
    pub(crate) actual: Commit,
    pub(crate) initial: Properties,
    pub(crate) tick: Properties,
    pub(crate) expected: Commit,
    pub(crate) expected_properties: Properties,
    pub(crate) moving: Vec<ScrollOracleRegion>,
    pub(crate) fixed: Vec<(&'static str, geometry::Rect)>,
}

#[cfg(feature = "renderer-debug")]
pub(crate) fn renderer_scroll_oracle_fixture(
    case: ScrollOracleCase,
) -> Result<ScrollOracleFixture, ContractError> {
    match case {
        ScrollOracleCase::F01 => renderer_scroll_oracle_f01(),
        ScrollOracleCase::F02 => renderer_scroll_oracle_f02(),
        ScrollOracleCase::F03 => {
            let ((actual, initial, tick), (expected, expected_properties)) =
                renderer_group_under_scroll_pair()?;
            Ok(ScrollOracleFixture {
                actual,
                initial,
                tick,
                expected,
                expected_properties,
                moving: vec![
                    ScrollOracleRegion {
                        name: "ungrouped-rule",
                        initial: geometry::Rect::new(92, 12, 4, 40),
                        translated: geometry::Rect::new(72, 12, 4, 40),
                    },
                    ScrollOracleRegion {
                        name: "grouped-quad",
                        initial: geometry::Rect::new(52, 20, 16, 24),
                        translated: geometry::Rect::new(32, 20, 16, 24),
                    },
                ],
                fixed: Vec::new(),
            })
        }
        ScrollOracleCase::F04 => renderer_scroll_oracle_f04(),
        ScrollOracleCase::F05 => renderer_scroll_oracle_f05(),
        ScrollOracleCase::F06 => renderer_scroll_oracle_f06(),
        ScrollOracleCase::F07 => renderer_scroll_oracle_f07(),
        ScrollOracleCase::F08 => renderer_scroll_oracle_f08(),
    }
}

#[cfg(feature = "renderer-debug")]
fn renderer_oracle_scroll_declaration(
    viewport: geometry::Rect,
    content_bounds: geometry::Rect,
    maximum: interaction::ScrollOffset,
) -> ScrollDeclaration {
    ScrollDeclaration::new(
        viewport,
        content_bounds,
        content_bounds,
        interaction::ScrollOffset::default(),
        maximum,
    )
    .expect("renderer scroll oracle has complete resident coverage")
}

#[cfg(feature = "renderer-debug")]
fn renderer_oracle_properties(
    commit: &Commit,
    serial: PropertySerial,
    offsets: &[(composition::tree::NodeId, interaction::ScrollOffset)],
    changed: &[composition::tree::NodeId],
) -> Result<Properties, ContractError> {
    Properties::new(
        commit,
        serial,
        offsets
            .iter()
            .map(|(node, value)| PropertyValue::ScrollOffset {
                node: *node,
                value: *value,
            })
            .collect(),
        changed
            .iter()
            .map(|node| PropertyRef::new(*node, PropertyKind::ScrollOffset))
            .collect(),
    )
}

#[cfg(feature = "renderer-debug")]
fn renderer_scroll_oracle_f01() -> Result<ScrollOracleFixture, ContractError> {
    let scroll = composition::tree::NodeId::renderer_fixture(501);
    let expected_node = composition::tree::NodeId::renderer_fixture(502);
    let size = geometry::Size::new(112, 64);
    let viewport = geometry::Rect::new(8, 8, 96, 48);
    let bounds = geometry::Rect::new(8, 8, 140, 48);
    let quad = geometry::Rect::new(52, 20, 16, 24);
    let rule = geometry::Rect::new(92, 12, 4, 40);
    let actual_node = Node::new(
        scroll,
        None,
        bounds,
        vec![
            Content::Quad(Quad::new(quad, Color::rgba(38, 179, 230, 255))),
            Content::Rule(Rule::vertical(rule, Color::rgba(242, 179, 26, 255), 3)),
        ],
    )
    .with_properties([PropertyKind::ScrollOffset])
    .with_scroll(renderer_oracle_scroll_declaration(
        viewport,
        bounds,
        interaction::ScrollOffset::new(44, 0),
    ));
    let actual = Commit::from_parts(
        Revision::renderer_fixture(501),
        size,
        Color::rgba(0, 0, 0, 255),
        vec![Arc::new(actual_node)],
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
    let initial = renderer_oracle_properties(
        &actual,
        PropertySerial::INITIAL,
        &[(scroll, interaction::ScrollOffset::default())],
        &[],
    )?;
    let tick = renderer_oracle_properties(
        &actual,
        PropertySerial::INITIAL.next(),
        &[(scroll, interaction::ScrollOffset::new(20, 0))],
        &[scroll],
    )?;
    let translated_quad = geometry::Rect::new(32, 20, 16, 24);
    let translated_rule = geometry::Rect::new(72, 12, 4, 40);
    let expected_node = Node::new(
        expected_node,
        None,
        bounds,
        vec![
            Content::Quad(Quad::new(translated_quad, Color::rgba(38, 179, 230, 255))),
            Content::Rule(Rule::vertical(
                translated_rule,
                Color::rgba(242, 179, 26, 255),
                3,
            )),
        ],
    );
    let expected = Commit::new(
        Revision::renderer_fixture(502),
        size,
        Color::rgba(0, 0, 0, 255),
        vec![expected_node],
    )?;
    let expected_properties = Properties::empty(&expected)?;
    Ok(ScrollOracleFixture {
        actual,
        initial,
        tick,
        expected,
        expected_properties,
        moving: vec![
            ScrollOracleRegion {
                name: "opaque-quad",
                initial: quad,
                translated: translated_quad,
            },
            ScrollOracleRegion {
                name: "independent-rule",
                initial: rule,
                translated: translated_rule,
            },
        ],
        fixed: Vec::new(),
    })
}

#[cfg(feature = "renderer-debug")]
fn renderer_scroll_oracle_f02() -> Result<ScrollOracleFixture, ContractError> {
    let scroll = composition::tree::NodeId::renderer_fixture(511);
    let expected_node = composition::tree::NodeId::renderer_fixture(512);
    let size = geometry::Size::new(112, 96);
    let viewport = geometry::Rect::new(8, 8, 96, 80);
    let bounds = geometry::Rect::new(8, 8, 96, 120);
    let text = geometry::Rect::new(12, 58, 40, 20);
    let quad = geometry::Rect::new(68, 60, 20, 16);
    let actual_node = Node::new(
        scroll,
        None,
        bounds,
        vec![
            Content::Text(Text::new(
                text,
                "Move",
                Color::rgba(230, 230, 235, 255),
                TextWrap::None,
            )),
            Content::Quad(Quad::new(quad, Color::rgba(38, 191, 89, 255))),
        ],
    )
    .with_properties([PropertyKind::ScrollOffset])
    .with_scroll(renderer_oracle_scroll_declaration(
        viewport,
        bounds,
        interaction::ScrollOffset::new(0, 40),
    ));
    let actual = Commit::from_parts(
        Revision::renderer_fixture(511),
        size,
        Color::rgba(0, 0, 0, 255),
        vec![Arc::new(actual_node)],
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
    let initial = renderer_oracle_properties(
        &actual,
        PropertySerial::INITIAL,
        &[(scroll, interaction::ScrollOffset::default())],
        &[],
    )?;
    let tick = renderer_oracle_properties(
        &actual,
        PropertySerial::INITIAL.next(),
        &[(scroll, interaction::ScrollOffset::new(0, 20))],
        &[scroll],
    )?;
    let translated_text = geometry::Rect::new(12, 38, 40, 20);
    let translated_quad = geometry::Rect::new(68, 40, 20, 16);
    let expected_node = Node::new(
        expected_node,
        None,
        bounds,
        vec![
            Content::Text(Text::new(
                translated_text,
                "Move",
                Color::rgba(230, 230, 235, 255),
                TextWrap::None,
            )),
            Content::Quad(Quad::new(translated_quad, Color::rgba(38, 191, 89, 255))),
        ],
    );
    let expected = Commit::new(
        Revision::renderer_fixture(512),
        size,
        Color::rgba(0, 0, 0, 255),
        vec![expected_node],
    )?;
    let expected_properties = Properties::empty(&expected)?;
    Ok(ScrollOracleFixture {
        actual,
        initial,
        tick,
        expected,
        expected_properties,
        moving: vec![
            ScrollOracleRegion {
                name: "text",
                initial: text,
                translated: translated_text,
            },
            ScrollOracleRegion {
                name: "opaque-quad",
                initial: quad,
                translated: translated_quad,
            },
        ],
        fixed: Vec::new(),
    })
}

#[cfg(feature = "renderer-debug")]
fn renderer_scroll_oracle_f04() -> Result<ScrollOracleFixture, ContractError> {
    let group = composition::tree::NodeId::renderer_fixture(521);
    let scroll = composition::tree::NodeId::renderer_fixture(522);
    let expected_group = composition::tree::NodeId::renderer_fixture(523);
    let expected_content = composition::tree::NodeId::renderer_fixture(524);
    let size = geometry::Size::new(112, 72);
    let group_bounds = geometry::Rect::new(8, 8, 96, 56);
    let content_bounds = geometry::Rect::new(8, 8, 140, 56);
    let text = geometry::Rect::new(52, 18, 36, 20);
    let quad = geometry::Rect::new(88, 40, 16, 16);
    let group_node = Node::new(group, None, group_bounds, Vec::new());
    let scroll_node = Node::new(
        scroll,
        Some(group),
        content_bounds,
        vec![
            Content::Text(Text::new(
                text,
                "Fx",
                Color::rgba(230, 230, 235, 255),
                TextWrap::None,
            )),
            Content::Quad(Quad::new(quad, Color::rgba(217, 64, 230, 255))),
        ],
    )
    .with_properties([PropertyKind::ScrollOffset])
    .with_scroll(renderer_oracle_scroll_declaration(
        group_bounds,
        content_bounds,
        interaction::ScrollOffset::new(44, 0),
    ));
    let actual = Commit::from_parts(
        Revision::renderer_fixture(521),
        size,
        Color::rgba(0, 0, 0, 255),
        vec![Arc::new(group_node), Arc::new(scroll_node)],
        Some(vec![
            Draw::PushGroup {
                node: group,
                bounds: group_bounds,
                opacity: 1.0,
            },
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
            Draw::PopGroup,
        ]),
        Vec::new(),
    )?;
    let initial = renderer_oracle_properties(
        &actual,
        PropertySerial::INITIAL,
        &[(scroll, interaction::ScrollOffset::default())],
        &[],
    )?;
    let tick = renderer_oracle_properties(
        &actual,
        PropertySerial::INITIAL.next(),
        &[(scroll, interaction::ScrollOffset::new(20, 0))],
        &[scroll],
    )?;
    let translated_text = geometry::Rect::new(32, 18, 36, 20);
    let translated_quad = geometry::Rect::new(68, 40, 16, 16);
    let expected_group_node = Node::new(expected_group, None, group_bounds, Vec::new());
    let expected_content_node = Node::new(
        expected_content,
        Some(expected_group),
        content_bounds,
        vec![
            Content::Text(Text::new(
                translated_text,
                "Fx",
                Color::rgba(230, 230, 235, 255),
                TextWrap::None,
            )),
            Content::Quad(Quad::new(translated_quad, Color::rgba(217, 64, 230, 255))),
        ],
    );
    let expected = Commit::from_parts(
        Revision::renderer_fixture(522),
        size,
        Color::rgba(0, 0, 0, 255),
        vec![
            Arc::new(expected_group_node),
            Arc::new(expected_content_node),
        ],
        Some(vec![
            Draw::PushGroup {
                node: expected_group,
                bounds: group_bounds,
                opacity: 1.0,
            },
            Draw::Content {
                node: expected_content,
                index: 0,
                projection: ContentProjection::Normal,
            },
            Draw::Content {
                node: expected_content,
                index: 1,
                projection: ContentProjection::Normal,
            },
            Draw::PopGroup,
        ]),
        Vec::new(),
    )?;
    let expected_properties = Properties::empty(&expected)?;
    Ok(ScrollOracleFixture {
        actual,
        initial,
        tick,
        expected,
        expected_properties,
        moving: vec![
            ScrollOracleRegion {
                name: "grouped-text-below-scroll",
                initial: text,
                translated: translated_text,
            },
            ScrollOracleRegion {
                name: "grouped-quad-below-scroll",
                initial: quad,
                translated: translated_quad,
            },
        ],
        fixed: Vec::new(),
    })
}

#[cfg(feature = "renderer-debug")]
fn renderer_scroll_oracle_f05() -> Result<ScrollOracleFixture, ContractError> {
    let chrome = composition::tree::NodeId::renderer_fixture(531);
    let scroll = composition::tree::NodeId::renderer_fixture(532);
    let expected_chrome = composition::tree::NodeId::renderer_fixture(533);
    let expected_content = composition::tree::NodeId::renderer_fixture(534);
    let size = geometry::Size::new(112, 72);
    let outer_clip = Clip::new(geometry::Rect::new(8, 8, 96, 56));
    let inner_clip = Clip::new(geometry::Rect::new(48, 12, 48, 48));
    let translated_inner_clip = Clip::new(geometry::Rect::new(28, 12, 48, 48));
    let content_bounds = geometry::Rect::new(8, 8, 140, 56);
    let chrome_rect = geometry::Rect::new(8, 8, 4, 56);
    let content = geometry::Rect::new(44, 18, 56, 32);
    let translated_content = geometry::Rect::new(24, 18, 56, 32);
    let chrome_node = Node::new(
        chrome,
        None,
        chrome_rect,
        vec![Content::Quad(Quad::new(
            chrome_rect,
            Color::rgba(242, 179, 26, 255),
        ))],
    );
    let scroll_node = Node::new(
        scroll,
        None,
        content_bounds,
        vec![Content::Quad(Quad::new(
            content,
            Color::rgba(38, 140, 217, 255),
        ))],
    )
    .with_properties([PropertyKind::ScrollOffset])
    .with_scroll(renderer_oracle_scroll_declaration(
        geometry::Rect::new(8, 8, 96, 56),
        content_bounds,
        interaction::ScrollOffset::new(44, 0),
    ));
    let actual = Commit::from_parts(
        Revision::renderer_fixture(531),
        size,
        Color::rgba(0, 0, 0, 255),
        vec![Arc::new(chrome_node), Arc::new(scroll_node)],
        Some(vec![
            Draw::PushClip {
                node: None,
                clip: outer_clip,
            },
            Draw::Content {
                node: chrome,
                index: 0,
                projection: ContentProjection::Normal,
            },
            Draw::PushScroll { node: scroll },
            Draw::PushClip {
                node: None,
                clip: inner_clip,
            },
            Draw::Content {
                node: scroll,
                index: 0,
                projection: ContentProjection::Normal,
            },
            Draw::PopClip,
            Draw::PopScroll,
            Draw::PopClip,
        ]),
        Vec::new(),
    )?;
    let initial = renderer_oracle_properties(
        &actual,
        PropertySerial::INITIAL,
        &[(scroll, interaction::ScrollOffset::default())],
        &[],
    )?;
    let tick = renderer_oracle_properties(
        &actual,
        PropertySerial::INITIAL.next(),
        &[(scroll, interaction::ScrollOffset::new(20, 0))],
        &[scroll],
    )?;
    let expected_chrome_node = Node::new(
        expected_chrome,
        None,
        chrome_rect,
        vec![Content::Quad(Quad::new(
            chrome_rect,
            Color::rgba(242, 179, 26, 255),
        ))],
    );
    let expected_content_node = Node::new(
        expected_content,
        None,
        content_bounds,
        vec![Content::Quad(Quad::new(
            translated_content,
            Color::rgba(38, 140, 217, 255),
        ))],
    );
    let expected = Commit::from_parts(
        Revision::renderer_fixture(532),
        size,
        Color::rgba(0, 0, 0, 255),
        vec![
            Arc::new(expected_chrome_node),
            Arc::new(expected_content_node),
        ],
        Some(vec![
            Draw::PushClip {
                node: None,
                clip: outer_clip,
            },
            Draw::Content {
                node: expected_chrome,
                index: 0,
                projection: ContentProjection::Normal,
            },
            Draw::PushClip {
                node: None,
                clip: translated_inner_clip,
            },
            Draw::Content {
                node: expected_content,
                index: 0,
                projection: ContentProjection::Normal,
            },
            Draw::PopClip,
            Draw::PopClip,
        ]),
        Vec::new(),
    )?;
    let expected_properties = Properties::empty(&expected)?;
    Ok(ScrollOracleFixture {
        actual,
        initial,
        tick,
        expected,
        expected_properties,
        moving: vec![ScrollOracleRegion {
            name: "inner-clipped-content",
            initial: geometry::Rect::new(84, 22, 12, 20),
            translated: geometry::Rect::new(32, 22, 12, 20),
        }],
        fixed: vec![("fixed-outer-chrome", chrome_rect)],
    })
}

#[cfg(feature = "renderer-debug")]
fn renderer_scroll_oracle_f06() -> Result<ScrollOracleFixture, ContractError> {
    let outer = composition::tree::NodeId::renderer_fixture(541);
    let group = composition::tree::NodeId::renderer_fixture(542);
    let inner = composition::tree::NodeId::renderer_fixture(543);
    let rows = composition::tree::NodeId::renderer_fixture(544);
    let expected_group = composition::tree::NodeId::renderer_fixture(545);
    let expected_rows = composition::tree::NodeId::renderer_fixture(546);
    let size = geometry::Size::new(112, 96);
    let viewport = geometry::Rect::new(8, 8, 96, 80);
    let outer_bounds = geometry::Rect::new(8, 8, 96, 120);
    let group_bounds = geometry::Rect::new(8, 8, 96, 80);
    let inner_bounds = geometry::Rect::new(8, 8, 96, 120);
    let text = geometry::Rect::new(16, 58, 36, 20);
    let row = geometry::Rect::new(60, 60, 32, 16);
    let outer_node = Node::new(outer, None, outer_bounds, Vec::new())
        .with_properties([PropertyKind::ScrollOffset])
        .with_scroll(renderer_oracle_scroll_declaration(
            viewport,
            outer_bounds,
            interaction::ScrollOffset::new(0, 40),
        ));
    let group_node = Node::new(group, Some(outer), group_bounds, Vec::new());
    let inner_node = Node::new(inner, Some(group), inner_bounds, Vec::new())
        .with_properties([PropertyKind::ScrollOffset])
        .with_scroll(renderer_oracle_scroll_declaration(
            viewport,
            inner_bounds,
            interaction::ScrollOffset::new(0, 40),
        ));
    let rows_node = Node::new(
        rows,
        Some(inner),
        inner_bounds,
        vec![
            Content::Text(Text::new(
                text,
                "Row",
                Color::rgba(230, 230, 235, 255),
                TextWrap::None,
            )),
            Content::Quad(Quad::new(row, Color::rgba(230, 51, 38, 255))),
        ],
    );
    let actual = Commit::from_parts(
        Revision::renderer_fixture(541),
        size,
        Color::rgba(0, 0, 0, 255),
        vec![
            Arc::new(outer_node),
            Arc::new(group_node),
            Arc::new(inner_node),
            Arc::new(rows_node),
        ],
        Some(vec![
            Draw::PushScroll { node: outer },
            Draw::PushGroup {
                node: group,
                bounds: group_bounds,
                opacity: 1.0,
            },
            Draw::PushScroll { node: inner },
            Draw::Content {
                node: rows,
                index: 0,
                projection: ContentProjection::Normal,
            },
            Draw::Content {
                node: rows,
                index: 1,
                projection: ContentProjection::Normal,
            },
            Draw::PopScroll,
            Draw::PopGroup,
            Draw::PopScroll,
        ]),
        Vec::new(),
    )?;
    let initial = renderer_oracle_properties(
        &actual,
        PropertySerial::INITIAL,
        &[
            (outer, interaction::ScrollOffset::default()),
            (inner, interaction::ScrollOffset::default()),
        ],
        &[],
    )?;
    let tick = renderer_oracle_properties(
        &actual,
        PropertySerial::INITIAL.next(),
        &[
            (outer, interaction::ScrollOffset::default()),
            (inner, interaction::ScrollOffset::new(0, 20)),
        ],
        &[inner],
    )?;
    let translated_text = geometry::Rect::new(16, 38, 36, 20);
    let translated_row = geometry::Rect::new(60, 40, 32, 16);
    let expected_group_node = Node::new(expected_group, None, group_bounds, Vec::new());
    let expected_rows_node = Node::new(
        expected_rows,
        Some(expected_group),
        inner_bounds,
        vec![
            Content::Text(Text::new(
                translated_text,
                "Row",
                Color::rgba(230, 230, 235, 255),
                TextWrap::None,
            )),
            Content::Quad(Quad::new(translated_row, Color::rgba(230, 51, 38, 255))),
        ],
    );
    let expected = Commit::from_parts(
        Revision::renderer_fixture(542),
        size,
        Color::rgba(0, 0, 0, 255),
        vec![Arc::new(expected_group_node), Arc::new(expected_rows_node)],
        Some(vec![
            Draw::PushGroup {
                node: expected_group,
                bounds: group_bounds,
                opacity: 1.0,
            },
            Draw::Content {
                node: expected_rows,
                index: 0,
                projection: ContentProjection::Normal,
            },
            Draw::Content {
                node: expected_rows,
                index: 1,
                projection: ContentProjection::Normal,
            },
            Draw::PopGroup,
        ]),
        Vec::new(),
    )?;
    let expected_properties = Properties::empty(&expected)?;
    Ok(ScrollOracleFixture {
        actual,
        initial,
        tick,
        expected,
        expected_properties,
        moving: vec![
            ScrollOracleRegion {
                name: "nested-group-text",
                initial: text,
                translated: translated_text,
            },
            ScrollOracleRegion {
                name: "nested-virtual-row",
                initial: row,
                translated: translated_row,
            },
        ],
        fixed: Vec::new(),
    })
}

#[cfg(feature = "renderer-debug")]
fn renderer_scroll_oracle_f07() -> Result<ScrollOracleFixture, ContractError> {
    let scroll = composition::tree::NodeId::renderer_fixture(551);
    let expected_node = composition::tree::NodeId::renderer_fixture(552);
    let size = geometry::Size::new(112, 88);
    let viewport = geometry::Rect::new(8, 8, 96, 72);
    let bounds = geometry::Rect::new(8, 8, 148, 72);
    let fill = geometry::Rect::new(52, 16, 32, 48);
    let rule = geometry::Rect::new(88, 12, 4, 56);
    let text = geometry::Rect::new(100, 28, 36, 20);
    let actual_node = Node::new(
        scroll,
        None,
        bounds,
        vec![
            Content::Quad(Quad::new(fill, Color::rgba(26, 77, 153, 255))),
            Content::Rule(Rule::vertical(rule, Color::rgba(242, 179, 26, 255), 3)),
            Content::Text(Text::new(
                text,
                "Cell",
                Color::rgba(230, 230, 235, 255),
                TextWrap::None,
            )),
        ],
    )
    .with_properties([PropertyKind::ScrollOffset])
    .with_scroll(renderer_oracle_scroll_declaration(
        viewport,
        bounds,
        interaction::ScrollOffset::new(52, 0),
    ));
    let actual = Commit::from_parts(
        Revision::renderer_fixture(551),
        size,
        Color::rgba(0, 0, 0, 255),
        vec![Arc::new(actual_node)],
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
            Draw::Content {
                node: scroll,
                index: 2,
                projection: ContentProjection::Normal,
            },
            Draw::PopScroll,
        ]),
        Vec::new(),
    )?;
    let initial = renderer_oracle_properties(
        &actual,
        PropertySerial::INITIAL,
        &[(scroll, interaction::ScrollOffset::default())],
        &[],
    )?;
    let tick = renderer_oracle_properties(
        &actual,
        PropertySerial::INITIAL.next(),
        &[(scroll, interaction::ScrollOffset::new(20, 0))],
        &[scroll],
    )?;
    let translated_fill = geometry::Rect::new(32, 16, 32, 48);
    let translated_rule = geometry::Rect::new(68, 12, 4, 56);
    let translated_text = geometry::Rect::new(80, 28, 36, 20);
    let expected_node = Node::new(
        expected_node,
        None,
        bounds,
        vec![
            Content::Quad(Quad::new(translated_fill, Color::rgba(26, 77, 153, 255))),
            Content::Rule(Rule::vertical(
                translated_rule,
                Color::rgba(242, 179, 26, 255),
                3,
            )),
            Content::Text(Text::new(
                translated_text,
                "Cell",
                Color::rgba(230, 230, 235, 255),
                TextWrap::None,
            )),
        ],
    );
    let expected_node_id = expected_node.id();
    let expected = Commit::from_parts(
        Revision::renderer_fixture(552),
        size,
        Color::rgba(0, 0, 0, 255),
        vec![Arc::new(expected_node)],
        Some(vec![
            Draw::PushClip {
                node: None,
                clip: Clip::new(viewport),
            },
            Draw::Content {
                node: expected_node_id,
                index: 0,
                projection: ContentProjection::Normal,
            },
            Draw::Content {
                node: expected_node_id,
                index: 1,
                projection: ContentProjection::Normal,
            },
            Draw::Content {
                node: expected_node_id,
                index: 2,
                projection: ContentProjection::Normal,
            },
            Draw::PopClip,
        ]),
        Vec::new(),
    )?;
    let expected_properties = Properties::empty(&expected)?;
    Ok(ScrollOracleFixture {
        actual,
        initial,
        tick,
        expected,
        expected_properties,
        moving: vec![
            ScrollOracleRegion {
                name: "table-fill",
                initial: fill,
                translated: translated_fill,
            },
            ScrollOracleRegion {
                name: "table-rule",
                initial: rule,
                translated: translated_rule,
            },
            ScrollOracleRegion {
                name: "table-text",
                initial: text,
                translated: translated_text,
            },
        ],
        fixed: Vec::new(),
    })
}

#[cfg(feature = "renderer-debug")]
fn renderer_scroll_oracle_f08() -> Result<ScrollOracleFixture, ContractError> {
    let horizontal = composition::tree::NodeId::renderer_fixture(561);
    let vertical = composition::tree::NodeId::renderer_fixture(562);
    let body = composition::tree::NodeId::renderer_fixture(563);
    let expected_header = composition::tree::NodeId::renderer_fixture(564);
    let expected_body = composition::tree::NodeId::renderer_fixture(565);
    let size = geometry::Size::new(112, 96);
    let viewport = geometry::Rect::new(8, 8, 96, 80);
    let horizontal_bounds = geometry::Rect::new(8, 8, 148, 80);
    let vertical_bounds = geometry::Rect::new(8, 8, 148, 120);
    let header = geometry::Rect::new(88, 12, 32, 16);
    let cell = geometry::Rect::new(64, 60, 32, 20);
    let rule = geometry::Rect::new(100, 48, 4, 40);
    let text = geometry::Rect::new(84, 62, 20, 18);
    let horizontal_node = Node::new(
        horizontal,
        None,
        horizontal_bounds,
        vec![Content::Quad(Quad::new(
            header,
            Color::rgba(38, 140, 217, 255),
        ))],
    )
    .with_properties([PropertyKind::ScrollOffset])
    .with_scroll(renderer_oracle_scroll_declaration(
        viewport,
        horizontal_bounds,
        interaction::ScrollOffset::new(52, 0),
    ));
    let vertical_node = Node::new(vertical, Some(horizontal), vertical_bounds, Vec::new())
        .with_properties([PropertyKind::ScrollOffset])
        .with_scroll(renderer_oracle_scroll_declaration(
            viewport,
            vertical_bounds,
            interaction::ScrollOffset::new(0, 40),
        ))
        .with_scroll_target(super::spatial::ScrollTarget::scene_node(horizontal));
    let body_node = Node::new(
        body,
        Some(vertical),
        vertical_bounds,
        vec![
            Content::Quad(Quad::new(cell, Color::rgba(26, 77, 153, 255))),
            Content::Rule(Rule::vertical(rule, Color::rgba(242, 179, 26, 255), 3)),
            Content::Text(Text::new(
                text,
                "XY",
                Color::rgba(230, 230, 235, 255),
                TextWrap::None,
            )),
        ],
    );
    let actual = Commit::from_parts(
        Revision::renderer_fixture(561),
        size,
        Color::rgba(0, 0, 0, 255),
        vec![
            Arc::new(horizontal_node),
            Arc::new(vertical_node),
            Arc::new(body_node),
        ],
        Some(vec![
            Draw::PushScroll { node: horizontal },
            Draw::Content {
                node: horizontal,
                index: 0,
                projection: ContentProjection::Normal,
            },
            Draw::PushScroll { node: vertical },
            Draw::Content {
                node: body,
                index: 0,
                projection: ContentProjection::Normal,
            },
            Draw::Content {
                node: body,
                index: 1,
                projection: ContentProjection::Normal,
            },
            Draw::Content {
                node: body,
                index: 2,
                projection: ContentProjection::Normal,
            },
            Draw::PopScroll,
            Draw::PopScroll,
        ]),
        Vec::new(),
    )?;
    let initial = renderer_oracle_properties(
        &actual,
        PropertySerial::INITIAL,
        &[
            (horizontal, interaction::ScrollOffset::default()),
            (vertical, interaction::ScrollOffset::default()),
        ],
        &[],
    )?;
    let tick = renderer_oracle_properties(
        &actual,
        PropertySerial::INITIAL.next(),
        &[
            (horizontal, interaction::ScrollOffset::new(20, 0)),
            (vertical, interaction::ScrollOffset::new(0, 20)),
        ],
        &[horizontal, vertical],
    )?;
    let translated_header = geometry::Rect::new(68, 12, 32, 16);
    let translated_cell = geometry::Rect::new(44, 40, 32, 20);
    let translated_rule = geometry::Rect::new(80, 28, 4, 40);
    let translated_text = geometry::Rect::new(64, 42, 20, 18);
    let expected_header_node = Node::new(
        expected_header,
        None,
        horizontal_bounds,
        vec![Content::Quad(Quad::new(
            translated_header,
            Color::rgba(38, 140, 217, 255),
        ))],
    );
    let expected_body_node = Node::new(
        expected_body,
        None,
        vertical_bounds,
        vec![
            Content::Quad(Quad::new(translated_cell, Color::rgba(26, 77, 153, 255))),
            Content::Rule(Rule::vertical(
                translated_rule,
                Color::rgba(242, 179, 26, 255),
                3,
            )),
            Content::Text(Text::new(
                translated_text,
                "XY",
                Color::rgba(230, 230, 235, 255),
                TextWrap::None,
            )),
        ],
    );
    let expected_header_id = expected_header_node.id();
    let expected_body_id = expected_body_node.id();
    let expected = Commit::from_parts(
        Revision::renderer_fixture(562),
        size,
        Color::rgba(0, 0, 0, 255),
        vec![Arc::new(expected_header_node), Arc::new(expected_body_node)],
        Some(vec![
            Draw::PushClip {
                node: None,
                clip: Clip::new(viewport),
            },
            Draw::Content {
                node: expected_header_id,
                index: 0,
                projection: ContentProjection::Normal,
            },
            Draw::PushClip {
                node: None,
                clip: Clip::new(geometry::Rect::new(-12, 8, 96, 80)),
            },
            Draw::Content {
                node: expected_body_id,
                index: 0,
                projection: ContentProjection::Normal,
            },
            Draw::Content {
                node: expected_body_id,
                index: 1,
                projection: ContentProjection::Normal,
            },
            Draw::Content {
                node: expected_body_id,
                index: 2,
                projection: ContentProjection::Normal,
            },
            Draw::PopClip,
            Draw::PopClip,
        ]),
        Vec::new(),
    )?;
    let expected_properties = Properties::empty(&expected)?;
    Ok(ScrollOracleFixture {
        actual,
        initial,
        tick,
        expected,
        expected_properties,
        moving: vec![
            ScrollOracleRegion {
                name: "split-axis-header",
                initial: header,
                translated: translated_header,
            },
            ScrollOracleRegion {
                name: "diagonal-cell",
                initial: cell,
                translated: translated_cell,
            },
            ScrollOracleRegion {
                name: "diagonal-rule",
                initial: rule,
                translated: translated_rule,
            },
            ScrollOracleRegion {
                name: "diagonal-text",
                initial: text,
                translated: translated_text,
            },
        ],
        fixed: Vec::new(),
    })
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

#[cfg(feature = "renderer-debug")]
pub(crate) fn renderer_property_economics_fixture(
    property_count: usize,
    dirty_count: usize,
) -> Result<(Commit, Properties, Properties), ContractError> {
    renderer_property_economics_fixture_at(property_count, dirty_count, 20_000)
}

#[cfg(feature = "renderer-debug")]
pub(crate) fn renderer_property_economics_fixture_at(
    property_count: usize,
    dirty_count: usize,
    fixture_base: u64,
) -> Result<(Commit, Properties, Properties), ContractError> {
    let property_count = property_count.max(1);
    let dirty_count = dirty_count.min(property_count);
    let bounds = geometry::Rect::new(0, 0, 64, 64);
    let mut nodes = Vec::with_capacity(property_count);
    let mut initial_values = Vec::with_capacity(property_count);
    let mut updates = Vec::with_capacity(dirty_count);

    for index in 0..property_count {
        let node =
            composition::tree::NodeId::renderer_fixture(fixture_base.saturating_add(index as u64));
        let rect = geometry::Rect::new((index % 64) as i32, ((index / 64) % 64) as i32, 1, 1);
        nodes.push(
            Node::new(
                node,
                None,
                bounds,
                vec![Content::Quad(Quad::new(
                    rect,
                    Color::rgba(117, 191, 242, 255),
                ))],
            )
            .with_properties([PropertyKind::Transform]),
        );
        initial_values.push(PropertyValue::Transform {
            node,
            value: Transform::identity(),
        });
        if index < dirty_count {
            updates.push(PropertyValue::Transform {
                node,
                value: Transform::translate(1.0, 0.0),
            });
        }
    }

    let commit = Commit::new(
        Revision::renderer_fixture(fixture_base),
        geometry::Size::new(64, 64),
        Color::rgba(0, 0, 0, 255),
        nodes,
    )?;
    let initial = Properties::new(&commit, PropertySerial::INITIAL, initial_values, Vec::new())?;
    let (tick, advanced) =
        Properties::apply_updates(&commit, &initial, PropertySerial::INITIAL.next(), updates)?;
    debug_assert_eq!(advanced, dirty_count > 0);
    Ok((commit, initial, tick))
}

#[cfg(test)]
mod tests {
    use super::super::{Glass, Material};
    use super::*;

    #[cfg(feature = "renderer-debug")]
    #[test]
    fn tier_a_compatibility_output_is_generated_from_the_same_spatial_topology() {
        for case in ScrollOracleCase::ALL {
            let fixture = renderer_scroll_oracle_fixture(case).expect("Tier A fixture");
            let actual = fixture
                .actual
                .compatibility_scene(&fixture.tick)
                .expect("actual compatibility scene");
            let expected = fixture
                .expected
                .compatibility_scene(&fixture.expected_properties)
                .expect("expected compatibility scene");

            let actual_geometry = compatibility_payload_geometry(actual.primitives());
            let expected_geometry = compatibility_payload_geometry(expected.primitives());
            let mut unmatched = actual_geometry.clone();
            for expected_payload in &expected_geometry {
                let position = unmatched
                    .iter()
                    .position(|actual_payload| actual_payload == expected_payload)
                    .unwrap_or_else(|| {
                        panic!(
                            "{} compatibility projection omitted {expected_payload:?}; actual payload geometry: {actual_geometry:?}",
                            case.name()
                        )
                    });
                unmatched.remove(position);
            }
            assert!(
                unmatched.is_empty(),
                "{} compatibility projection added payload geometry: {unmatched:?}",
                case.name()
            );
            for region in &fixture.moving {
                assert!(
                    actual_geometry
                        .iter()
                        .any(|(_, rect)| compatibility_rect_contains(*rect, region.translated)),
                    "{} compatibility projection did not move {} to {:?}; actual payload geometry: {actual_geometry:?}",
                    case.name(),
                    region.name,
                    region.translated
                );
            }
        }
    }

    #[cfg(feature = "renderer-debug")]
    fn compatibility_payload_geometry(
        primitives: &[Primitive],
    ) -> Vec<(&'static str, geometry::Rect)> {
        let mut geometry = Vec::new();
        for primitive in primitives {
            match primitive {
                Primitive::Quad(quad) => geometry.push(("quad", quad.rect())),
                Primitive::Rule(rule) => geometry.push(("rule", rule.rect())),
                Primitive::Text(text) => geometry.push(("text", text.rect())),
                Primitive::TextViewport(viewport) => {
                    geometry.push(("text-viewport", viewport.rect()));
                }
                Primitive::Icon(icon) => geometry.push(("icon", icon.rect())),
                Primitive::Shadow(shadow) => geometry.push(("shadow", shadow.rect())),
                Primitive::Pane(pane) => geometry.push(("pane", pane.rect())),
                Primitive::Outline(outline) => geometry.push(("outline", outline.rect())),
                Primitive::Group(group) => {
                    geometry.extend(compatibility_payload_geometry(group.primitives()));
                }
                Primitive::Clip(_) | Primitive::PopClip => {}
            }
        }
        geometry
    }

    #[cfg(feature = "renderer-debug")]
    fn compatibility_rect_contains(outer: geometry::Rect, inner: geometry::Rect) -> bool {
        outer.x() <= inner.x()
            && outer.y() <= inner.y()
            && outer.right() >= inner.right()
            && outer.bottom() >= inner.bottom()
    }

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
    fn semantic_projection_excludes_bounded_and_caret_content_and_normalizes_scrollbars() {
        let owner = id(1);
        let size = geometry::Size::new(100, 80);
        let viewport = geometry::Rect::from_size(size);
        let content_bounds = geometry::Rect::new(0, 0, 1_000, 80);
        let resident_bounds = geometry::Rect::new(0, 0, 320, 80);
        let make_commit = |revision, baseline, body_color, runway_x| {
            let declaration = ScrollDeclaration::new(
                viewport,
                content_bounds,
                resident_bounds,
                interaction::ScrollOffset::new(baseline, 0),
                interaction::ScrollOffset::new(900, 0),
            )
            .expect("fixture residency covers its baseline");
            let node = Node::new(
                owner,
                None,
                content_bounds,
                vec![
                    Content::Quad(Quad::new(geometry::Rect::new(0, 0, 100, 80), body_color)),
                    Content::Rule(Rule::horizontal(
                        geometry::Rect::new(runway_x, 20, 160, 1),
                        Color::rgba(230, 230, 230, 255),
                        1,
                    )),
                    Content::Rule(Rule::vertical(
                        geometry::Rect::new(runway_x + 12, 20, 2, 18),
                        Color::rgba(255, 255, 255, 255),
                        2,
                    )),
                    Content::Quad(Quad::new(
                        geometry::Rect::new(baseline, 76, 24, 4),
                        Color::rgba(160, 160, 160, 255),
                    )),
                ],
            )
            .with_properties([
                PropertyKind::ScrollOffset,
                PropertyKind::Caret,
                PropertyKind::HorizontalScrollbar,
            ])
            .with_scroll(declaration);
            Arc::new(
                Commit::from_parts(
                    revision,
                    size,
                    Color::rgba(0, 0, 0, 0),
                    vec![Arc::new(node)],
                    Some(vec![
                        Draw::Content {
                            node: owner,
                            index: 0,
                            projection: ContentProjection::Normal,
                        },
                        Draw::PushScroll { node: owner },
                        Draw::Content {
                            node: owner,
                            index: 1,
                            projection: ContentProjection::Normal,
                        },
                        Draw::Content {
                            node: owner,
                            index: 2,
                            projection: ContentProjection::Caret,
                        },
                        Draw::PopScroll,
                        Draw::Content {
                            node: owner,
                            index: 3,
                            projection: ContentProjection::ScrollbarThumb {
                                axis: interaction::ScrollbarAxis::Horizontal,
                                edge: 80,
                                base_thickness: 4,
                                maximum_thickness: 8,
                                baseline_start: baseline,
                                baseline_extent: 24,
                                baseline_position: baseline,
                                travel: 76,
                                maximum_offset: 900,
                            },
                        },
                    ]),
                    Vec::new(),
                )
                .expect("fixture commit satisfies the scene contract"),
            )
        };

        let first_drawable = make_commit(Revision::INITIAL, 0, Color::rgba(20, 30, 40, 255), 0);
        let resident_scrolls = HashSet::from([owner]);
        let first =
            Commit::semantic_projection(&first_drawable, None, &HashSet::new(), &resident_scrolls)
                .expect("first semantic projection");
        let second_drawable = make_commit(
            Revision::INITIAL.next(),
            128,
            Color::rgba(20, 30, 40, 255),
            128,
        );
        let second = Commit::semantic_projection(
            &second_drawable,
            Some(&first),
            &HashSet::new(),
            &resident_scrolls,
        )
        .expect("replenished semantic projection");

        assert!(
            Arc::ptr_eq(&first, &second),
            "{}; left={:?}; right={:?}",
            first.projection_difference(&second),
            first.nodes()[0],
            second.nodes()[0]
        );
        assert_eq!(first.nodes()[0].content().len(), 2);
        assert!(first.nodes()[0].declares(PropertyKind::ScrollOffset));
        assert!(first.nodes()[0].declares(PropertyKind::Caret));
        assert!(first.nodes()[0].declares(PropertyKind::HorizontalScrollbar));

        let changed_drawable = make_commit(
            Revision::INITIAL.next().next(),
            128,
            Color::rgba(90, 30, 40, 255),
            128,
        );
        let changed = Commit::semantic_projection(
            &changed_drawable,
            Some(&second),
            &HashSet::new(),
            &resident_scrolls,
        )
        .expect("semantic body change projection");
        assert!(!Arc::ptr_eq(&second, &changed));
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
    fn indexed_property_updates_share_untouched_blocks_and_coalesce_one_dirty_slot() {
        let mut nodes = Vec::new();
        let mut values = Vec::new();
        for value in 1_u64..=300 {
            let node = empty_node(value, None).with_properties([PropertyKind::Opacity]);
            values.push(PropertyValue::Opacity {
                node: node.id(),
                value: 1.0,
            });
            nodes.push(node);
        }
        values.reverse();
        let commit = Commit::new(
            Revision::INITIAL,
            geometry::Size::new(20, 10),
            Color::rgba(0, 0, 0, 0),
            nodes,
        )
        .expect("indexed property commit");
        let initial = Properties::new(&commit, PropertySerial::INITIAL, values, Vec::new())
            .expect("canonical indexed values");
        let property = PropertyRef::new(id(300), PropertyKind::Opacity);
        let (updated, advanced) = Properties::apply_updates(
            &commit,
            &initial,
            PropertySerial::INITIAL.next(),
            vec![
                PropertyValue::Opacity {
                    node: id(300),
                    value: 0.25,
                },
                PropertyValue::Opacity {
                    node: id(300),
                    value: 0.5,
                },
                PropertyValue::Opacity {
                    node: id(300),
                    value: 0.75,
                },
            ],
        )
        .expect("coalesced indexed update");

        assert!(advanced);
        assert_eq!(updated.changed().len(), 1);
        assert_eq!(updated.work().value_visits(), 3);
        assert_eq!(updated.work().index_lookups(), 3);
        assert_eq!(
            updated.value(property),
            Some(PropertyValue::Opacity {
                node: id(300),
                value: 0.75,
            })
        );
        assert!(Arc::ptr_eq(
            &initial.values.blocks[0],
            &updated.values.blocks[0]
        ));
        assert!(!Arc::ptr_eq(
            initial.values.blocks.last().expect("initial tail block"),
            updated.values.blocks.last().expect("updated tail block")
        ));
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
