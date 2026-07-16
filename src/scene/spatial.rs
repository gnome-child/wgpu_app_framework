use std::collections::{HashMap, HashSet};

use thiserror::Error;

use super::{
    Clip, Commit, ContentProjection, Draw, EffectDeclaration, Group, Node, Primitive, Properties,
    PropertyKind, PropertyRef, PropertyValue, ScrollDeclaration,
};
use crate::{composition, interaction};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub(crate) enum ScrollTarget {
    Interaction(interaction::Target),
    SceneNode(composition::tree::NodeId),
}

impl ScrollTarget {
    pub(crate) fn scene_node(node: composition::tree::NodeId) -> Self {
        Self::SceneNode(node)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub(crate) struct AxisOwnership {
    horizontal: bool,
    vertical: bool,
}

impl AxisOwnership {
    fn from_declaration(declaration: ScrollDeclaration) -> Self {
        let maximum = declaration.maximum();
        Self {
            horizontal: maximum.x() > 0,
            vertical: maximum.y() > 0,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub(crate) struct SpatialNodeId(u32);

impl SpatialNodeId {
    const ROOT: Self = Self(0);

    fn index(self) -> usize {
        self.0 as usize
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub(crate) struct ScrollPathId(u32);

impl ScrollPathId {
    pub(crate) const ROOT: Self = Self(0);

    fn index(self) -> usize {
        self.0 as usize
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub(crate) struct ClipNodeId(u32);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub(crate) struct EffectNodeId(u32);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub(crate) struct SpatialBinding {
    spatial: SpatialNodeId,
    surface: SpatialNodeId,
}

impl SpatialBinding {
    pub(crate) const ROOT: Self = Self {
        spatial: SpatialNodeId::ROOT,
        surface: SpatialNodeId::ROOT,
    };

    pub(crate) fn is_identity(self) -> bool {
        self.spatial == self.surface
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub(crate) struct PropertyState {
    spatial: SpatialBinding,
    clip: Option<ClipNodeId>,
    effect: Option<EffectNodeId>,
}

impl PropertyState {
    const ROOT: Self = Self {
        spatial: SpatialBinding::ROOT,
        clip: None,
        effect: None,
    };

    pub(crate) fn spatial(self) -> SpatialBinding {
        self.spatial
    }
}

#[derive(Debug, Clone, PartialEq)]
pub(crate) struct SpatialNode {
    parent: Option<SpatialNodeId>,
    kind: SpatialNodeKind,
}

#[derive(Debug, Clone, PartialEq)]
pub(crate) enum SpatialNodeKind {
    Root,
    Transform {
        owner: composition::tree::NodeId,
    },
    Scroll {
        owner: composition::tree::NodeId,
        target: ScrollTarget,
        axes: AxisOwnership,
        baseline: interaction::ScrollOffset,
    },
    SurfaceRoot {
        owner: composition::tree::NodeId,
        bounds: crate::geometry::Rect,
    },
}

#[derive(Debug, Clone, PartialEq)]
pub(crate) struct ClipNode {
    parent: Option<ClipNodeId>,
    spatial: SpatialBinding,
    owner: Option<composition::tree::NodeId>,
    clip: Clip,
}

#[derive(Debug, Clone, PartialEq)]
pub(crate) struct EffectNode {
    parent: Option<EffectNodeId>,
    spatial: SpatialBinding,
    kind: EffectKind,
}

#[derive(Debug, Clone, PartialEq)]
pub(crate) enum EffectKind {
    Group {
        owner: composition::tree::NodeId,
        bounds: crate::geometry::Rect,
        opacity: f32,
    },
    Blur {
        owner: composition::tree::NodeId,
        bounds: crate::geometry::Rect,
        maximum_sigma: f32,
    },
    Backdrop {
        owner: composition::tree::NodeId,
        bounds: crate::geometry::Rect,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub(crate) struct ContentKey {
    node: composition::tree::NodeId,
    index: usize,
    projection: ContentProjection,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub(crate) struct ContentBinding {
    key: ContentKey,
    state: PropertyState,
    draw_index: Option<usize>,
}

#[derive(Debug, Clone, PartialEq)]
pub(crate) struct SpatialTopology {
    nodes: Vec<SpatialNode>,
    clips: Vec<ClipNode>,
    effects: Vec<EffectNode>,
    draw_states: Vec<PropertyState>,
    draw_surfaces: Vec<Option<SpatialNodeId>>,
    content: Vec<ContentBinding>,
    scroll_paths: Vec<Vec<(composition::tree::NodeId, interaction::ScrollOffset)>>,
    binding_scroll_paths: HashMap<SpatialBinding, ScrollPathId>,
}

#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub(crate) enum SpatialError {
    #[error("spatial topology references unknown scene node {0:?}")]
    UnknownNode(composition::tree::NodeId),
    #[error("scroll scope {0:?} has no scroll declaration")]
    MissingScroll(composition::tree::NodeId),
    #[error("scroll scope {0:?} has no interaction target identity")]
    MissingScrollTarget(composition::tree::NodeId),
    #[error("spatial scope closes {actual} while {expected} is active")]
    ScopeMismatch {
        expected: &'static str,
        actual: &'static str,
    },
    #[error("spatial topology ends with an unclosed {0} scope")]
    UnclosedScope(&'static str),
    #[error("scroll target {target:?} assigns its {axis} axis to both {first:?} and {second:?}")]
    ConflictingAxisOwner {
        target: ScrollTarget,
        axis: &'static str,
        first: composition::tree::NodeId,
        second: composition::tree::NodeId,
    },
    #[error("spatial topology exceeds its addressable node count")]
    TooManyNodes,
    #[error("spatial binding does not reach its declared surface root")]
    InvalidSurfaceRoot,
}

#[derive(Clone, Copy)]
struct CompileState {
    spatial: SpatialNodeId,
    surface: SpatialNodeId,
    clip: Option<ClipNodeId>,
    effect: Option<EffectNodeId>,
}

impl CompileState {
    const ROOT: Self = Self {
        spatial: SpatialNodeId::ROOT,
        surface: SpatialNodeId::ROOT,
        clip: None,
        effect: None,
    };

    fn property(self) -> PropertyState {
        PropertyState {
            spatial: SpatialBinding {
                spatial: self.spatial,
                surface: self.surface,
            },
            clip: self.clip,
            effect: self.effect,
        }
    }
}

enum Scope {
    Clip(CompileState),
    Group(CompileState),
    Scroll(CompileState),
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum CompatibilityStop {
    Group,
    Scroll,
}

impl Scope {
    fn name(&self) -> &'static str {
        match self {
            Self::Clip(_) => "clip",
            Self::Group(_) => "group",
            Self::Scroll(_) => "scroll",
        }
    }

    fn previous(&self) -> CompileState {
        match *self {
            Self::Clip(state) | Self::Group(state) | Self::Scroll(state) => state,
        }
    }
}

impl SpatialTopology {
    pub(crate) fn compile(
        nodes: &[std::sync::Arc<Node>],
        order: Option<&[Draw]>,
    ) -> Result<Self, SpatialError> {
        let mut topology = Self {
            nodes: vec![SpatialNode {
                parent: None,
                kind: SpatialNodeKind::Root,
            }],
            clips: Vec::new(),
            effects: Vec::new(),
            draw_states: vec![PropertyState::ROOT; order.map_or(0, <[Draw]>::len)],
            draw_surfaces: vec![None; order.map_or(0, <[Draw]>::len)],
            content: Vec::new(),
            scroll_paths: vec![Vec::new()],
            binding_scroll_paths: HashMap::new(),
        };
        let scene_nodes = nodes
            .iter()
            .map(|node| (node.id(), node.as_ref()))
            .collect::<HashMap<_, _>>();
        let mut axis_owners = HashMap::new();
        if let Some(order) = order {
            topology.compile_ordered(order, &scene_nodes, &mut axis_owners)?;
        } else {
            topology.compile_node_tree(nodes, &scene_nodes, &mut axis_owners)?;
        }
        topology.compile_scroll_paths()?;
        Ok(topology)
    }

    pub(crate) fn draw_state(&self, index: usize) -> Option<PropertyState> {
        self.draw_states.get(index).copied()
    }

    pub(crate) fn surface_bounds_for_draw(&self, index: usize) -> Option<crate::geometry::Rect> {
        let surface = self.draw_surfaces.get(index).copied().flatten()?;
        match self.nodes.get(surface.index()).map(|node| &node.kind) {
            Some(SpatialNodeKind::SurfaceRoot { bounds, .. }) => Some(*bounds),
            _ => None,
        }
    }

    pub(crate) fn content_state(
        &self,
        node: composition::tree::NodeId,
        index: usize,
        projection: ContentProjection,
    ) -> Option<PropertyState> {
        self.content
            .iter()
            .find(|binding| {
                binding.key
                    == ContentKey {
                        node,
                        index,
                        projection,
                    }
            })
            .map(|binding| binding.state)
    }

    pub(crate) fn project_semantic_order(
        &self,
        order: &[Draw],
        resident_nodes: &HashSet<composition::tree::NodeId>,
        resident_scrolls: &HashSet<composition::tree::NodeId>,
    ) -> Vec<Draw> {
        debug_assert_eq!(order.len(), self.draw_states.len());
        let mut projected = Vec::with_capacity(order.len());
        let mut scopes = Vec::new();
        let mut omitted_scroll_depth = 0_usize;
        for draw in order {
            if omitted_scroll_depth > 0 {
                match draw {
                    Draw::PushScroll { .. } => {
                        omitted_scroll_depth = omitted_scroll_depth.saturating_add(1);
                    }
                    Draw::PopScroll => {
                        omitted_scroll_depth = omitted_scroll_depth.saturating_sub(1);
                    }
                    _ => {}
                }
                continue;
            }
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
                    if resident_scrolls.contains(node) {
                        omitted_scroll_depth = 1;
                        continue;
                    }
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
                Draw::Content {
                    node, projection, ..
                } if !resident_nodes.contains(node) && *projection != ContentProjection::Caret => {
                    projected.push(draw.clone());
                }
                Draw::Content { .. } => {}
            }
        }
        projected
    }

    pub(crate) fn compatibility_primitives(
        &self,
        commit: &Commit,
        properties: &Properties,
    ) -> Vec<Primitive> {
        let Some(order) = commit.order() else {
            let mut primitives = Vec::new();
            for node in commit.nodes().iter().filter(|node| node.parent().is_none()) {
                primitives.extend(self.compatibility_node_tree(commit, node, properties));
            }
            return primitives;
        };
        debug_assert_eq!(order.len(), self.draw_states.len());
        let mut primitives = Vec::new();
        let mut index = 0;
        self.emit_compatibility_until(commit, order, &mut index, None, properties, &mut primitives);
        primitives
    }

    pub(crate) fn scroll_translation(
        &self,
        binding: SpatialBinding,
        properties: &Properties,
    ) -> Result<[f32; 2], SpatialError> {
        self.scroll_path_translation(self.scroll_path(binding)?, properties)
    }

    pub(crate) fn scroll_path(
        &self,
        binding: SpatialBinding,
    ) -> Result<ScrollPathId, SpatialError> {
        self.binding_scroll_paths
            .get(&binding)
            .copied()
            .ok_or(SpatialError::InvalidSurfaceRoot)
    }

    pub(crate) fn scroll_path_translation(
        &self,
        path: ScrollPathId,
        properties: &Properties,
    ) -> Result<[f32; 2], SpatialError> {
        let mut translation = [0.0_f32; 2];
        for (owner, baseline) in self
            .scroll_paths
            .get(path.index())
            .ok_or(SpatialError::InvalidSurfaceRoot)?
        {
            let offset = properties.scroll_offset(*owner).unwrap_or(*baseline);
            translation[0] += baseline.x().saturating_sub(offset.x()) as f32;
            translation[1] += baseline.y().saturating_sub(offset.y()) as f32;
        }
        Ok(translation)
    }

    pub(crate) fn scroll_path_owners(
        &self,
        path: ScrollPathId,
    ) -> Result<Vec<composition::tree::NodeId>, SpatialError> {
        self.scroll_paths
            .get(path.index())
            .map(|path| path.iter().map(|(owner, _)| *owner).collect())
            .ok_or(SpatialError::InvalidSurfaceRoot)
    }

    fn world_scroll_translation(&self, state: PropertyState, properties: &Properties) -> [f32; 2] {
        self.scroll_translation(
            SpatialBinding {
                spatial: state.spatial.spatial,
                surface: SpatialNodeId::ROOT,
            },
            properties,
        )
        .unwrap_or_default()
    }

    fn compile_scroll_paths(&mut self) -> Result<(), SpatialError> {
        let mut bindings = vec![SpatialBinding::ROOT];
        let mut push_binding = |binding: SpatialBinding| {
            if !bindings.contains(&binding) {
                bindings.push(binding);
            }
            let world = SpatialBinding {
                spatial: binding.spatial,
                surface: SpatialNodeId::ROOT,
            };
            if !bindings.contains(&world) {
                bindings.push(world);
            }
        };
        for state in &self.draw_states {
            push_binding(state.spatial);
        }
        for binding in &self.content {
            push_binding(binding.state.spatial);
        }
        for clip in &self.clips {
            push_binding(clip.spatial);
        }
        for effect in &self.effects {
            push_binding(effect.spatial);
        }

        let mut interned = HashMap::from([(Vec::new(), ScrollPathId::ROOT)]);
        for binding in bindings {
            let entries = self.scroll_path_entries(binding)?;
            let key = entries.iter().map(|(owner, _)| *owner).collect::<Vec<_>>();
            let path = if let Some(path) = interned.get(&key).copied() {
                path
            } else {
                let path = ScrollPathId(
                    u32::try_from(self.scroll_paths.len())
                        .map_err(|_| SpatialError::TooManyNodes)?,
                );
                self.scroll_paths.push(entries);
                interned.insert(key, path);
                path
            };
            self.binding_scroll_paths.insert(binding, path);
        }
        Ok(())
    }

    fn scroll_path_entries(
        &self,
        binding: SpatialBinding,
    ) -> Result<Vec<(composition::tree::NodeId, interaction::ScrollOffset)>, SpatialError> {
        let mut path = Vec::new();
        let mut current = binding.spatial;
        while current != binding.surface {
            let node = self
                .nodes
                .get(current.index())
                .ok_or(SpatialError::InvalidSurfaceRoot)?;
            if let SpatialNodeKind::Scroll {
                owner, baseline, ..
            } = node.kind
            {
                path.push((owner, baseline));
            }
            current = node.parent.ok_or(SpatialError::InvalidSurfaceRoot)?;
        }
        Ok(path)
    }

    fn compile_ordered(
        &mut self,
        order: &[Draw],
        scene_nodes: &HashMap<composition::tree::NodeId, &Node>,
        axis_owners: &mut HashMap<(ScrollTarget, &'static str), composition::tree::NodeId>,
    ) -> Result<(), SpatialError> {
        let mut state = CompileState::ROOT;
        let mut scopes = Vec::new();
        let mut transforms = HashMap::new();
        let mut scrolls = HashMap::new();
        let mut surfaces = HashMap::new();
        for (draw_index, draw) in order.iter().enumerate() {
            self.draw_states[draw_index] = state.property();
            match *draw {
                Draw::Content {
                    node,
                    index,
                    projection,
                } => {
                    let owner = scene_nodes
                        .get(&node)
                        .copied()
                        .ok_or(SpatialError::UnknownNode(node))?;
                    let content_state =
                        self.content_state_for_owner(owner, state, &mut transforms)?;
                    let binding = ContentBinding {
                        key: ContentKey {
                            node,
                            index,
                            projection,
                        },
                        state: content_state.property(),
                        draw_index: Some(draw_index),
                    };
                    self.draw_states[draw_index] = binding.state;
                    self.content.push(binding);
                }
                Draw::PushClip { node, clip } => {
                    scopes.push(Scope::Clip(state));
                    let id = self.push_clip(state, node, clip)?;
                    state.clip = Some(id);
                }
                Draw::PopClip => state = close_scope(&mut scopes, "clip")?,
                Draw::PushGroup {
                    node,
                    bounds,
                    opacity,
                } => {
                    if !scene_nodes.contains_key(&node) {
                        return Err(SpatialError::UnknownNode(node));
                    }
                    scopes.push(Scope::Group(state));
                    let effect = self.push_effect(
                        state,
                        EffectKind::Group {
                            owner: node,
                            bounds,
                            opacity,
                        },
                    )?;
                    let surface = self.push_surface(state.spatial, node, bounds, &mut surfaces)?;
                    self.draw_surfaces[draw_index] = Some(surface);
                    state = CompileState {
                        spatial: surface,
                        surface,
                        effect: Some(effect),
                        ..state
                    };
                }
                Draw::PopGroup => state = close_scope(&mut scopes, "group")?,
                Draw::PushScroll { node } => {
                    let owner = scene_nodes
                        .get(&node)
                        .copied()
                        .ok_or(SpatialError::UnknownNode(node))?;
                    scopes.push(Scope::Scroll(state));
                    state.spatial =
                        self.push_scroll(owner, state.spatial, axis_owners, &mut scrolls)?;
                }
                Draw::PopScroll => state = close_scope(&mut scopes, "scroll")?,
            }
        }
        if let Some(scope) = scopes.last() {
            return Err(SpatialError::UnclosedScope(scope.name()));
        }
        Ok(())
    }

    #[allow(clippy::too_many_arguments)]
    fn emit_compatibility_until(
        &self,
        commit: &Commit,
        order: &[Draw],
        index: &mut usize,
        stop: Option<CompatibilityStop>,
        properties: &Properties,
        target: &mut Vec<Primitive>,
    ) {
        while let Some(draw) = order.get(*index) {
            let draw_index = *index;
            *index = index.saturating_add(1);
            match draw {
                Draw::Content {
                    node,
                    index,
                    projection,
                } => {
                    let Some(content) = commit
                        .nodes()
                        .iter()
                        .find(|candidate| candidate.id() == *node)
                        .and_then(|node| node.content().get(*index))
                    else {
                        continue;
                    };
                    let transform =
                        properties.value(PropertyRef::new(*node, PropertyKind::Transform));
                    let Some(primitive) = projection.project_primitive(
                        content.as_primitive(transform),
                        *node,
                        properties,
                    ) else {
                        continue;
                    };
                    let translation = self
                        .draw_states
                        .get(draw_index)
                        .copied()
                        .map(|state| self.world_scroll_translation(state, properties))
                        .unwrap_or_default();
                    target.push(
                        primitive.translated(
                            translation[0].round() as i32,
                            translation[1].round() as i32,
                        ),
                    );
                }
                Draw::PushClip { clip, .. } => {
                    let translation = self
                        .draw_states
                        .get(draw_index)
                        .copied()
                        .map(|state| self.world_scroll_translation(state, properties))
                        .unwrap_or_default();
                    target.push(
                        Primitive::Clip(*clip).translated(
                            translation[0].round() as i32,
                            translation[1].round() as i32,
                        ),
                    );
                }
                Draw::PopClip => target.push(Primitive::PopClip),
                Draw::PushGroup { opacity, .. } => {
                    let mut members = Vec::new();
                    self.emit_compatibility_until(
                        commit,
                        order,
                        index,
                        Some(CompatibilityStop::Group),
                        properties,
                        &mut members,
                    );
                    if let Some(group) = Group::new(members, *opacity) {
                        target.push(Primitive::Group(group));
                    }
                }
                Draw::PushScroll { node } => {
                    let mut members = Vec::new();
                    self.emit_compatibility_until(
                        commit,
                        order,
                        index,
                        Some(CompatibilityStop::Scroll),
                        properties,
                        &mut members,
                    );
                    let declaration = commit
                        .nodes()
                        .iter()
                        .find(|candidate| candidate.id() == *node)
                        .and_then(|node| node.scroll());
                    if let Some(declaration) = declaration {
                        let translation = self
                            .draw_states
                            .get(draw_index)
                            .copied()
                            .map(|state| self.world_scroll_translation(state, properties))
                            .unwrap_or_default();
                        let viewport = Primitive::Clip(Clip::new(declaration.viewport()))
                            .translated(
                                translation[0].round() as i32,
                                translation[1].round() as i32,
                            );
                        target.push(viewport);
                        target.extend(members);
                        target.push(Primitive::PopClip);
                    } else {
                        target.extend(members);
                    }
                }
                Draw::PopGroup if stop == Some(CompatibilityStop::Group) => return,
                Draw::PopGroup => {}
                Draw::PopScroll if stop == Some(CompatibilityStop::Scroll) => return,
                Draw::PopScroll => {}
            }
        }
    }

    fn compatibility_node_tree(
        &self,
        commit: &Commit,
        node: &Node,
        properties: &Properties,
    ) -> Vec<Primitive> {
        let transform = properties.value(PropertyRef::new(node.id(), PropertyKind::Transform));
        let mut primitives = node
            .content()
            .iter()
            .enumerate()
            .map(|(index, content)| {
                let primitive = content.as_primitive(transform);
                let translation = self
                    .content_state(node.id(), index, ContentProjection::Normal)
                    .map(|state| self.world_scroll_translation(state, properties))
                    .unwrap_or_default();
                primitive.translated(translation[0].round() as i32, translation[1].round() as i32)
            })
            .collect::<Vec<_>>();
        for child in commit
            .nodes()
            .iter()
            .filter(|child| child.parent() == Some(node.id()))
        {
            primitives.extend(self.compatibility_node_tree(commit, child, properties));
        }
        if matches!(node.effect(), EffectDeclaration::GroupOpacity(_)) {
            let opacity = match properties.value(PropertyRef::new(node.id(), PropertyKind::Opacity))
            {
                Some(PropertyValue::Opacity { value, .. }) => value,
                _ => 1.0,
            };
            primitives = Group::new(primitives, opacity)
                .map(Primitive::Group)
                .into_iter()
                .collect();
        }
        let clip = match properties.value(PropertyRef::new(node.id(), PropertyKind::Clip)) {
            Some(PropertyValue::Clip { rect, .. }) => Some(
                Clip::new(rect).with_rounding(node.clip().map(Clip::rounding).unwrap_or_default()),
            ),
            _ => node.clip(),
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

    fn compile_node_tree(
        &mut self,
        nodes: &[std::sync::Arc<Node>],
        scene_nodes: &HashMap<composition::tree::NodeId, &Node>,
        axis_owners: &mut HashMap<(ScrollTarget, &'static str), composition::tree::NodeId>,
    ) -> Result<(), SpatialError> {
        let mut node_states = HashMap::new();
        let mut transforms = HashMap::new();
        let mut scrolls = HashMap::new();
        for node in nodes {
            let mut state = match node.parent() {
                Some(parent) => *node_states
                    .get(&parent)
                    .ok_or(SpatialError::UnknownNode(parent))?,
                None => CompileState::ROOT,
            };
            state = self.content_state_for_owner(node, state, &mut transforms)?;
            if node.scroll().is_some() {
                state.spatial = self.push_scroll(node, state.spatial, axis_owners, &mut scrolls)?;
            }
            if let Some(clip) = node.clip() {
                state.clip = Some(self.push_clip(state, Some(node.id()), clip)?);
            }
            state = self.push_declared_effect(node, state)?;
            for index in 0..node.content().len() {
                self.content.push(ContentBinding {
                    key: ContentKey {
                        node: node.id(),
                        index,
                        projection: ContentProjection::Normal,
                    },
                    state: state.property(),
                    draw_index: None,
                });
            }
            node_states.insert(node.id(), state);
        }
        debug_assert_eq!(node_states.len(), scene_nodes.len());
        Ok(())
    }

    fn content_state_for_owner(
        &mut self,
        owner: &Node,
        mut state: CompileState,
        transforms: &mut HashMap<(SpatialNodeId, composition::tree::NodeId), SpatialNodeId>,
    ) -> Result<CompileState, SpatialError> {
        if owner.declares(PropertyKind::Transform) {
            let key = (state.spatial, owner.id());
            state.spatial = if let Some(node) = transforms.get(&key).copied() {
                node
            } else {
                let node = self.push_spatial(
                    state.spatial,
                    SpatialNodeKind::Transform { owner: owner.id() },
                )?;
                transforms.insert(key, node);
                node
            };
        }
        Ok(state)
    }

    fn push_declared_effect(
        &mut self,
        owner: &Node,
        mut state: CompileState,
    ) -> Result<CompileState, SpatialError> {
        let kind = match owner.effect() {
            EffectDeclaration::None => return Ok(state),
            EffectDeclaration::GroupOpacity(envelope) => EffectKind::Group {
                owner: owner.id(),
                bounds: envelope.bounds(),
                opacity: 1.0,
            },
            EffectDeclaration::Blur {
                envelope,
                maximum_sigma,
            } => EffectKind::Blur {
                owner: owner.id(),
                bounds: envelope.bounds(),
                maximum_sigma,
            },
            EffectDeclaration::Backdrop(envelope) => EffectKind::Backdrop {
                owner: owner.id(),
                bounds: envelope.bounds(),
            },
        };
        let bounds = match &kind {
            EffectKind::Group { bounds, .. }
            | EffectKind::Blur { bounds, .. }
            | EffectKind::Backdrop { bounds, .. } => *bounds,
        };
        let effect = self.push_effect(state, kind)?;
        let surface = self.push_spatial(
            state.spatial,
            SpatialNodeKind::SurfaceRoot {
                owner: owner.id(),
                bounds,
            },
        )?;
        state.spatial = surface;
        state.surface = surface;
        state.effect = Some(effect);
        Ok(state)
    }

    fn push_scroll(
        &mut self,
        owner: &Node,
        parent: SpatialNodeId,
        axis_owners: &mut HashMap<(ScrollTarget, &'static str), composition::tree::NodeId>,
        scrolls: &mut HashMap<(SpatialNodeId, composition::tree::NodeId), SpatialNodeId>,
    ) -> Result<SpatialNodeId, SpatialError> {
        if let Some(node) = scrolls.get(&(parent, owner.id())).copied() {
            return Ok(node);
        }
        let declaration = owner
            .scroll()
            .ok_or(SpatialError::MissingScroll(owner.id()))?;
        let target = owner
            .scroll_target()
            .cloned()
            .ok_or(SpatialError::MissingScrollTarget(owner.id()))?;
        let axes = AxisOwnership::from_declaration(declaration);
        for (axis, owned) in [("horizontal", axes.horizontal), ("vertical", axes.vertical)] {
            if !owned {
                continue;
            }
            let key = (target.clone(), axis);
            if let Some(first) = axis_owners.insert(key, owner.id())
                && first != owner.id()
            {
                return Err(SpatialError::ConflictingAxisOwner {
                    target,
                    axis,
                    first,
                    second: owner.id(),
                });
            }
        }
        let node = self.push_spatial(
            parent,
            SpatialNodeKind::Scroll {
                owner: owner.id(),
                target,
                axes,
                baseline: declaration.baseline(),
            },
        )?;
        scrolls.insert((parent, owner.id()), node);
        Ok(node)
    }

    fn push_surface(
        &mut self,
        parent: SpatialNodeId,
        owner: composition::tree::NodeId,
        bounds: crate::geometry::Rect,
        surfaces: &mut HashMap<(SpatialNodeId, composition::tree::NodeId, [i32; 4]), SpatialNodeId>,
    ) -> Result<SpatialNodeId, SpatialError> {
        let key = (
            parent,
            owner,
            [bounds.x(), bounds.y(), bounds.width(), bounds.height()],
        );
        if let Some(surface) = surfaces.get(&key).copied() {
            return Ok(surface);
        }
        let surface = self.push_spatial(parent, SpatialNodeKind::SurfaceRoot { owner, bounds })?;
        surfaces.insert(key, surface);
        Ok(surface)
    }

    fn push_spatial(
        &mut self,
        parent: SpatialNodeId,
        kind: SpatialNodeKind,
    ) -> Result<SpatialNodeId, SpatialError> {
        let id =
            SpatialNodeId(u32::try_from(self.nodes.len()).map_err(|_| SpatialError::TooManyNodes)?);
        self.nodes.push(SpatialNode {
            parent: Some(parent),
            kind,
        });
        Ok(id)
    }

    fn push_clip(
        &mut self,
        state: CompileState,
        owner: Option<composition::tree::NodeId>,
        clip: Clip,
    ) -> Result<ClipNodeId, SpatialError> {
        let id =
            ClipNodeId(u32::try_from(self.clips.len()).map_err(|_| SpatialError::TooManyNodes)?);
        self.clips.push(ClipNode {
            parent: state.clip,
            spatial: state.property().spatial,
            owner,
            clip,
        });
        Ok(id)
    }

    fn push_effect(
        &mut self,
        state: CompileState,
        kind: EffectKind,
    ) -> Result<EffectNodeId, SpatialError> {
        let id = EffectNodeId(
            u32::try_from(self.effects.len()).map_err(|_| SpatialError::TooManyNodes)?,
        );
        self.effects.push(EffectNode {
            parent: state.effect,
            spatial: state.property().spatial,
            kind,
        });
        Ok(id)
    }
}

fn close_scope(
    scopes: &mut Vec<Scope>,
    actual: &'static str,
) -> Result<CompileState, SpatialError> {
    let Some(scope) = scopes.pop() else {
        return Err(SpatialError::ScopeMismatch {
            expected: "root",
            actual,
        });
    };
    if scope.name() != actual {
        return Err(SpatialError::ScopeMismatch {
            expected: scope.name(),
            actual,
        });
    }
    Ok(scope.previous())
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use super::*;
    use crate::geometry;
    use crate::scene::commit::EffectEnvelope;

    fn id(value: u64) -> composition::tree::NodeId {
        let mut value = value;
        composition::tree::NodeId::layout(&mut value)
    }

    fn node(value: u64, parent: Option<composition::tree::NodeId>) -> Node {
        Node::new(
            id(value),
            parent,
            geometry::Rect::new(0, 0, 100, 100),
            Vec::new(),
        )
    }

    fn scroll_node(
        value: u64,
        parent: Option<composition::tree::NodeId>,
        maximum: interaction::ScrollOffset,
        target: ScrollTarget,
    ) -> Node {
        node(value, parent)
            .with_properties([PropertyKind::ScrollOffset])
            .with_scroll(
                ScrollDeclaration::new(
                    geometry::Rect::new(0, 0, 40, 40),
                    geometry::Rect::new(0, 0, 100, 100),
                    geometry::Rect::new(0, 0, 100, 100),
                    interaction::ScrollOffset::default(),
                    maximum,
                )
                .expect("test scroll declaration"),
            )
            .with_scroll_target(target)
    }

    #[test]
    fn surface_roots_exclude_outer_scroll_and_keep_inner_scroll() {
        let outer = id(1);
        let group = id(2);
        let inner = id(3);
        let nodes = vec![
            Arc::new(scroll_node(
                1,
                None,
                interaction::ScrollOffset::new(60, 0),
                ScrollTarget::scene_node(outer),
            )),
            Arc::new(node(2, Some(outer))),
            Arc::new(scroll_node(
                3,
                Some(group),
                interaction::ScrollOffset::new(0, 60),
                ScrollTarget::scene_node(inner),
            )),
        ];
        let bounds = geometry::Rect::new(0, 0, 80, 80);
        let outer_group = SpatialTopology::compile(
            &nodes,
            Some(&[
                Draw::PushScroll { node: outer },
                Draw::PushGroup {
                    node: group,
                    bounds,
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
        )
        .expect("outer-scroll group topology");
        assert!(
            outer_group
                .content_state(group, 0, ContentProjection::Normal)
                .expect("group content state")
                .spatial()
                .is_identity(),
            "direct surface-local content must not inherit the scroll that moves its surface"
        );

        let inner_scroll = SpatialTopology::compile(
            &nodes,
            Some(&[
                Draw::PushGroup {
                    node: group,
                    bounds,
                    opacity: 1.0,
                },
                Draw::PushScroll { node: inner },
                Draw::Content {
                    node: inner,
                    index: 0,
                    projection: ContentProjection::Normal,
                },
                Draw::PopScroll,
                Draw::PopGroup,
            ]),
        )
        .expect("inner-scroll group topology");
        assert!(
            !inner_scroll
                .content_state(inner, 0, ContentProjection::Normal)
                .expect("nested content state")
                .spatial()
                .is_identity(),
            "scrolling below a surface root must remain in the surface-local binding"
        );
    }

    #[test]
    fn shared_target_axes_are_typed_and_conflicting_owners_are_rejected() {
        let horizontal = id(11);
        let vertical = id(12);
        let shared = ScrollTarget::scene_node(id(99));
        let nodes = vec![
            Arc::new(scroll_node(
                11,
                None,
                interaction::ScrollOffset::new(60, 0),
                shared.clone(),
            )),
            Arc::new(scroll_node(
                12,
                None,
                interaction::ScrollOffset::new(0, 60),
                shared.clone(),
            )),
        ];
        let topology = SpatialTopology::compile(
            &nodes,
            Some(&[
                Draw::PushScroll { node: horizontal },
                Draw::PopScroll,
                Draw::PushScroll { node: vertical },
                Draw::PopScroll,
            ]),
        )
        .expect("split-axis target topology");
        let axes = topology
            .nodes
            .iter()
            .filter_map(|node| match &node.kind {
                SpatialNodeKind::Scroll { target, axes, .. } if target == &shared => Some(*axes),
                _ => None,
            })
            .collect::<Vec<_>>();
        assert_eq!(axes.len(), 2);
        assert!(axes.iter().any(|axes| axes.horizontal && !axes.vertical));
        assert!(axes.iter().any(|axes| !axes.horizontal && axes.vertical));

        let conflicting = vec![
            Arc::new(scroll_node(
                21,
                None,
                interaction::ScrollOffset::new(60, 0),
                shared.clone(),
            )),
            Arc::new(scroll_node(
                22,
                None,
                interaction::ScrollOffset::new(60, 0),
                shared,
            )),
        ];
        let error = SpatialTopology::compile(
            &conflicting,
            Some(&[
                Draw::PushScroll { node: id(21) },
                Draw::PopScroll,
                Draw::PushScroll { node: id(22) },
                Draw::PopScroll,
            ]),
        )
        .expect_err("one target axis cannot have two owners");
        assert!(matches!(
            error,
            SpatialError::ConflictingAxisOwner {
                axis: "horizontal",
                ..
            }
        ));
    }

    #[test]
    fn repeated_sibling_scroll_scopes_share_one_compiled_spatial_node() {
        let scroll = id(31);
        let nodes = vec![Arc::new(scroll_node(
            31,
            None,
            interaction::ScrollOffset::new(60, 0),
            ScrollTarget::scene_node(scroll),
        ))];
        let topology = SpatialTopology::compile(
            &nodes,
            Some(&[
                Draw::PushScroll { node: scroll },
                Draw::PopScroll,
                Draw::PushScroll { node: scroll },
                Draw::PopScroll,
            ]),
        )
        .expect("repeated sibling scroll topology");

        assert_eq!(
            topology
                .nodes
                .iter()
                .filter(|node| matches!(node.kind, SpatialNodeKind::Scroll { .. }))
                .count(),
            1,
            "one logical scroll scope must compile to one spatial node"
        );
        assert_eq!(topology.draw_states[1], topology.draw_states[3]);
    }

    #[test]
    fn empty_payload_compiles_to_the_root_topology() {
        let topology = SpatialTopology::compile(&[], Some(&[])).expect("empty topology");

        assert_eq!(
            topology.nodes,
            vec![SpatialNode {
                parent: None,
                kind: SpatialNodeKind::Root,
            }]
        );
        assert!(topology.clips.is_empty());
        assert!(topology.effects.is_empty());
        assert!(topology.draw_states.is_empty());
        assert!(topology.content.is_empty());
    }

    #[test]
    fn declared_filter_surface_keeps_its_stable_coordinate_root_without_payload() {
        let bounds = geometry::Rect::new(12, 14, 70, 52);
        let owner = id(41);
        let nodes = vec![Arc::new(node(41, None).with_effect(
            EffectDeclaration::Backdrop(
                EffectEnvelope::new(bounds, 8.0).expect("test effect envelope"),
            ),
        ))];
        let topology = SpatialTopology::compile(&nodes, None).expect("filter surface topology");

        assert!(matches!(
            topology.nodes.as_slice(),
            [
                SpatialNode {
                    parent: None,
                    kind: SpatialNodeKind::Root,
                },
                SpatialNode {
                    parent: Some(SpatialNodeId::ROOT),
                    kind: SpatialNodeKind::SurfaceRoot {
                        owner: actual_owner,
                        bounds: actual_bounds,
                    },
                }
            ] if *actual_owner == owner && *actual_bounds == bounds
        ));
        assert!(matches!(
            topology.effects.as_slice(),
            [EffectNode {
                parent: None,
                spatial: SpatialBinding::ROOT,
                kind: EffectKind::Backdrop {
                    owner: actual_owner,
                    bounds: actual_bounds,
                },
            }] if *actual_owner == owner && *actual_bounds == bounds
        ));
    }

    #[test]
    fn transform_nodes_share_identity_scroll_path_when_no_scroll_ancestor_exists() {
        let first = id(51);
        let second = id(52);
        let nodes = vec![
            Arc::new(node(51, None).with_properties([PropertyKind::Transform])),
            Arc::new(node(52, None).with_properties([PropertyKind::Transform])),
        ];
        let topology = SpatialTopology::compile(
            &nodes,
            Some(&[
                Draw::Content {
                    node: first,
                    index: 0,
                    projection: ContentProjection::Normal,
                },
                Draw::Content {
                    node: second,
                    index: 0,
                    projection: ContentProjection::Normal,
                },
            ]),
        )
        .expect("transform-only topology");

        let first_path = topology
            .scroll_path(topology.draw_states[0].spatial)
            .expect("first scroll path");
        let second_path = topology
            .scroll_path(topology.draw_states[1].spatial)
            .expect("second scroll path");
        assert_eq!(first_path, ScrollPathId::ROOT);
        assert_eq!(second_path, ScrollPathId::ROOT);
    }
}
