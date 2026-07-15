use std::{
    collections::{HashMap, HashSet},
    hash::Hash,
    num::{NonZeroU64, NonZeroUsize},
    ops::Range,
    sync::{Arc, Weak},
};

use bytemuck::{Pod, Zeroable};
use wgpu::util::DeviceExt;

use crate::{composition, scene};

use super::{
    self as render, batch,
    renderer::{
        PreparedClip, PreparedGroup, PreparedPane, PreparedScroll, RenderBatch,
        ShapeBatch as EncodedShapeBatch, TextBatch,
    },
};

const RETAINED_QUAD_WGSL: &str = include_str!("retained_quad.wgsl");
const INITIAL_INSTANCE_CAPACITY: usize = 256;
const INITIAL_PROPERTY_CAPACITY: usize = 32;

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
struct UnitVertex {
    corner: [f32; 2],
}

impl UnitVertex {
    fn layout() -> wgpu::VertexBufferLayout<'static> {
        const ATTRIBUTES: [wgpu::VertexAttribute; 1] = wgpu::vertex_attr_array![0 => Float32x2];
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<Self>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &ATTRIBUTES,
        }
    }
}

const UNIT_QUAD: [UnitVertex; 6] = [
    UnitVertex { corner: [0.0, 0.0] },
    UnitVertex { corner: [0.0, 1.0] },
    UnitVertex { corner: [1.0, 1.0] },
    UnitVertex { corner: [0.0, 0.0] },
    UnitVertex { corner: [1.0, 1.0] },
    UnitVertex { corner: [1.0, 0.0] },
];

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
struct ViewportUniform {
    size: [f32; 2],
    padding: [f32; 2],
}

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
struct NodeProperty {
    origin: [f32; 2],
    translate: [f32; 2],
    scroll: [f32; 2],
    scale: [f32; 2],
    grid: [f32; 2],
    scene_origin: [f32; 2],
    target_size: [f32; 2],
    padding: [f32; 2],
}

impl NodeProperty {
    const IDENTITY: Self = Self {
        origin: [0.0, 0.0],
        translate: [0.0, 0.0],
        scroll: [0.0, 0.0],
        scale: [1.0, 1.0],
        grid: [1.0, 0.0],
        scene_origin: [0.0, 0.0],
        target_size: [1.0, 1.0],
        padding: [0.0, 0.0],
    };
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub(in crate::render) struct ResourceKey {
    node: composition::tree::NodeId,
    content_revision: composition::tree::ContentRevision,
    geometry_revision: scene::GeometryRevision,
    topology_revision: scene::TopologyRevision,
    content_index: usize,
    part: u16,
    scale_bits: u32,
    target_bits: [u32; 4],
}

impl ResourceKey {
    pub(in crate::render) fn new(
        node: &scene::Node,
        content_index: usize,
        part: u16,
        scale_factor: f32,
    ) -> Self {
        Self {
            node: node.id(),
            content_revision: node.content_revision(),
            geometry_revision: node.geometry_revision(),
            topology_revision: node.topology_revision(),
            content_index,
            part,
            scale_bits: scale_factor.to_bits(),
            target_bits: [0; 4],
        }
    }

    pub(in crate::render) fn for_target(
        node: &scene::Node,
        content_index: usize,
        part: u16,
        scale_factor: f32,
        target_origin: [f32; 2],
        target_size: [f32; 2],
    ) -> Self {
        let mut key = Self::new(node, content_index, part, scale_factor);
        key.target_bits = [
            target_origin[0].to_bits(),
            target_origin[1].to_bits(),
            target_size[0].to_bits(),
            target_size[1].to_bits(),
        ];
        key
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(in crate::render) struct ShapeBatch {
    range: Range<u32>,
    binding: PropertyBinding,
}

impl ShapeBatch {
    fn binding(&self) -> PropertyBinding {
        self.binding
    }

    fn range(&self) -> Range<u32> {
        self.range.clone()
    }

    pub(in crate::render) fn instance_count(&self) -> usize {
        (self.range.end - self.range.start) as usize
    }

    fn merge_adjacent(&mut self, next: &Self) -> bool {
        if self.binding != next.binding || self.range.end != next.range.start {
            return false;
        }
        self.range.end = next.range.end;
        true
    }
}

struct Entry {
    owners: Vec<Weak<scene::Node>>,
    range: Option<Range<u32>>,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub(in crate::render) struct SyncStats {
    pub(in crate::render) realization_rebuilds: usize,
    pub(in crate::render) primitive_prepare_calls: usize,
    pub(in crate::render) content_upload_bytes: usize,
    pub(in crate::render) property_upload_bytes: usize,
    pub(in crate::render) resource_creations: usize,
    pub(in crate::render) resource_replacements: usize,
    pub(in crate::render) resource_removals: usize,
}

pub(in crate::render) struct Plan {
    batches: Vec<RenderBatch>,
    property_bindings: Vec<PropertyBinding>,
    requires_surface_sampling: bool,
    facts: PlanFacts,
}

#[derive(Clone, Copy, Default)]
struct PlanFacts {
    scene_items: usize,
    glyph_batches: usize,
    text_surfaces: usize,
    quad_instances: usize,
    clip_batches: usize,
    opaque_nodes: usize,
    blended_nodes: usize,
    effect_island_nodes: usize,
    group_composites: usize,
}

impl Plan {
    pub(in crate::render) fn batches(&self) -> &[RenderBatch] {
        &self.batches
    }

    pub(in crate::render) fn requires_surface_sampling(&self) -> bool {
        self.requires_surface_sampling
    }
}

impl PlanFacts {
    fn from_stats(stats: &render::DrawStats) -> Self {
        Self {
            scene_items: stats.scene_items,
            glyph_batches: stats.glyph_batches,
            text_surfaces: stats.text_surfaces,
            quad_instances: stats.quad_instances,
            clip_batches: stats.clip_batches,
            opaque_nodes: stats.opaque_nodes,
            blended_nodes: stats.blended_nodes,
            effect_island_nodes: stats.effect_island_nodes,
            group_composites: stats.group_composites,
        }
    }

    fn apply(self, stats: &mut render::DrawStats) {
        stats.scene_items = self.scene_items;
        stats.glyph_batches = self.glyph_batches;
        stats.text_surfaces = self.text_surfaces;
        stats.quad_instances = self.quad_instances;
        stats.clip_batches = self.clip_batches;
        stats.opaque_nodes = self.opaque_nodes;
        stats.blended_nodes = self.blended_nodes;
        stats.effect_island_nodes = self.effect_island_nodes;
        stats.group_composites = self.group_composites;
    }
}

struct PlanEntry {
    commit: Weak<scene::Commit>,
    scale_bits: u32,
    plan: Arc<Plan>,
}

#[derive(Debug, Clone, Copy)]
struct TargetSpace {
    origin: [f32; 2],
    size: [f32; 2],
}

impl PartialEq for TargetSpace {
    fn eq(&self, other: &Self) -> bool {
        self.origin.map(f32::to_bits) == other.origin.map(f32::to_bits)
            && self.size.map(f32::to_bits) == other.size.map(f32::to_bits)
    }
}

impl Eq for TargetSpace {}

impl std::hash::Hash for TargetSpace {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.origin.map(f32::to_bits).hash(state);
        self.size.map(f32::to_bits).hash(state);
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
struct PropertyBinding {
    node: composition::tree::NodeId,
    space: TargetSpace,
    composited_scroll: bool,
}

pub(in crate::render) struct Prepared {
    pub(in crate::render) plan: Arc<Plan>,
    pub(in crate::render) properties: PropertyBindings,
    pub(in crate::render) stats: render::DrawStats,
}

pub(in crate::render) struct Realizer {
    shapes: Shapes,
    plans: Vec<PlanEntry>,
}

impl Realizer {
    pub(in crate::render) fn new(
        render_context: &render::Context,
        format: wgpu::TextureFormat,
    ) -> Self {
        Self {
            shapes: Shapes::new(render_context, format),
            plans: Vec::new(),
        }
    }

    pub(in crate::render) fn prepare(
        &mut self,
        render_context: &render::Context,
        viewport: render::Viewport,
        commit: &Arc<scene::Commit>,
        properties: &scene::Properties,
        text_renderer: &mut render::text_renderer::TextRenderer,
    ) -> render::Result<Prepared> {
        properties
            .require_compatible(commit)
            .map_err(|_| render::Error::RetainedSceneContract)?;

        let mut stats = render::DrawStats::default();
        let _expired_plans = self.collect_plans();
        stats.retained_gpu_resource_removals += self.shapes.collect().resource_removals;
        stats.retained_gpu_resource_removals += text_renderer.collect_retained();

        let scale_bits = viewport.scale_factor().to_bits();
        let plan = self.plans.iter().find_map(|entry| {
            (entry.scale_bits == scale_bits)
                .then(|| entry.commit.upgrade())
                .flatten()
                .filter(|candidate| Arc::ptr_eq(candidate, commit))
                .map(|_| Arc::clone(&entry.plan))
        });

        let plan = if let Some(plan) = plan {
            stats.render_plan_reuses = 1;
            plan
        } else {
            let mut builder = PlanBuilder {
                render_context,
                viewport,
                shapes: &mut self.shapes,
                text_renderer,
                commit: Arc::downgrade(commit),
                rebuilt_nodes: HashSet::new(),
                property_bindings: Vec::new(),
                composited_scroll_depth: 0,
                stats: render::DrawStats::default(),
            };
            let built = builder.build(commit)?;
            stats.add(builder.stats);
            stats.scene_node_realization_rebuilds = builder.rebuilt_nodes.len();
            stats.render_plan_rebuilds = 1;
            let plan = Arc::new(built);
            self.plans.push(PlanEntry {
                commit: Arc::downgrade(commit),
                scale_bits,
                plan: Arc::clone(&plan),
            });
            plan
        };

        let (property_bindings, property_stats) = self.shapes.prepare_properties(
            render_context,
            viewport,
            commit,
            properties,
            &plan.property_bindings,
        );
        apply_sync_stats(&mut stats, property_stats);
        plan.facts.apply(&mut stats);
        stats.render_batches = count_batches(plan.batches());
        stats.direct_surface_plans = usize::from(!plan.requires_surface_sampling());
        stats.surface_sampling_plans = usize::from(plan.requires_surface_sampling());
        stats.retained_gpu_resource_count = self
            .shapes
            .resource_count()
            .saturating_add(text_renderer.retained_resource_count());
        stats.retained_gpu_resource_bytes = self
            .shapes
            .resource_bytes()
            .saturating_add(text_renderer.retained_resource_bytes());

        Ok(Prepared {
            plan,
            properties: property_bindings,
            stats,
        })
    }

    pub(in crate::render) fn shapes(&self) -> &Shapes {
        &self.shapes
    }

    fn collect_plans(&mut self) -> usize {
        let before = self.plans.len();
        self.plans.retain(|entry| entry.commit.strong_count() > 0);
        before.saturating_sub(self.plans.len())
    }
}

struct PlanBuilder<'a> {
    render_context: &'a render::Context,
    viewport: render::Viewport,
    shapes: &'a mut Shapes,
    text_renderer: &'a mut render::text_renderer::TextRenderer,
    commit: Weak<scene::Commit>,
    rebuilt_nodes: HashSet<composition::tree::NodeId>,
    property_bindings: Vec<PropertyBinding>,
    composited_scroll_depth: usize,
    stats: render::DrawStats,
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum PlanStop {
    Group,
    Scroll,
}

impl PlanBuilder<'_> {
    fn property_binding(
        &mut self,
        node: composition::tree::NodeId,
        space: TargetSpace,
    ) -> PropertyBinding {
        let binding = PropertyBinding {
            node,
            space,
            composited_scroll: self.composited_scroll_depth > 0,
        };
        if !self.property_bindings.contains(&binding) {
            self.property_bindings.push(binding);
        }
        binding
    }

    fn build(&mut self, commit: &Arc<scene::Commit>) -> render::Result<Plan> {
        let nodes = commit
            .nodes()
            .iter()
            .map(|node| (node.id(), Arc::clone(node)))
            .collect::<HashMap<_, _>>();
        for node in commit.nodes() {
            match node.opacity() {
                scene::OpacityDeclaration::Opaque => self.stats.opaque_nodes += 1,
                scene::OpacityDeclaration::Blended | scene::OpacityDeclaration::Variable => {
                    self.stats.blended_nodes += 1;
                }
            }
        }
        let mut batches = Vec::new();
        let main = TargetSpace {
            origin: [0.0, 0.0],
            size: [
                self.viewport.logical_area().width().max(1.0),
                self.viewport.logical_area().height().max(1.0),
            ],
        };
        if let Some(order) = commit.order() {
            let mut index = 0;
            self.build_order(order, &mut index, None, &nodes, main, &mut batches)?;
        } else {
            for node in commit.nodes().iter().filter(|node| node.parent().is_none()) {
                self.build_node(commit, node, main, &mut batches)?;
            }
        }
        coalesce_shape_batches(&mut batches);
        let requires_surface_sampling = render::renderer::requires_surface_sampling(&batches);
        Ok(Plan {
            batches,
            property_bindings: std::mem::take(&mut self.property_bindings),
            requires_surface_sampling,
            facts: PlanFacts::from_stats(&self.stats),
        })
    }

    fn build_order(
        &mut self,
        order: &[scene::Draw],
        index: &mut usize,
        stop: Option<PlanStop>,
        nodes: &HashMap<composition::tree::NodeId, Arc<scene::Node>>,
        space: TargetSpace,
        target: &mut Vec<RenderBatch>,
    ) -> render::Result<()> {
        while let Some(draw) = order.get(*index) {
            let order_index = *index;
            *index = index.saturating_add(1);
            match draw {
                scene::Draw::Content { node, index } => {
                    if let Some(owner) = nodes.get(node)
                        && let Some(content) = owner.content().get(*index)
                    {
                        self.build_content(owner, *index, content, space, target)?;
                    }
                }
                scene::Draw::PushClip { node, clip } => {
                    self.stats.scene_items += 1;
                    self.stats.clip_batches += 1;
                    target.push(RenderBatch::PushClip(PreparedClip {
                        node: *node,
                        fallback: render::scene::to_paint_clip_value_at_scale(
                            *clip,
                            self.viewport.scale_factor(),
                        ),
                        scene_origin: space.origin,
                    }));
                }
                scene::Draw::PopClip => {
                    self.stats.scene_items += 1;
                    self.stats.clip_batches += 1;
                    target.push(RenderBatch::PopClip);
                }
                scene::Draw::PushGroup {
                    node,
                    bounds,
                    opacity,
                } => {
                    let mut paint_index = *index;
                    let bounds = self
                        .project_order_group(order, &mut paint_index, nodes, *opacity)
                        .map(|group| group.bounds)
                        .unwrap_or_else(|| {
                            render::scene::to_paint_rect_value_at_scale(
                                *bounds,
                                self.viewport.scale_factor(),
                            )
                        });
                    let group_space = TargetSpace {
                        origin: [bounds.origin.x(), bounds.origin.y()],
                        size: [bounds.area.width().max(1.0), bounds.area.height().max(1.0)],
                    };
                    let mut members = Vec::new();
                    self.build_order(
                        order,
                        index,
                        Some(PlanStop::Group),
                        nodes,
                        group_space,
                        &mut members,
                    )?;
                    self.stats.scene_items += 1;
                    self.stats.group_composites += 1;
                    self.stats.effect_island_nodes += 1;
                    target.push(RenderBatch::Group(PreparedGroup {
                        node: Some(*node),
                        bounds: local_group_bounds(bounds, space.origin),
                        opacity: *opacity,
                        render_batches: members,
                    }));
                }
                scene::Draw::PushScroll { node } => {
                    let Some(declaration) = nodes.get(node).and_then(|node| node.scroll()) else {
                        continue;
                    };
                    let viewport = render::scene::to_paint_rect_value_at_scale(
                        declaration.viewport(),
                        self.viewport.scale_factor(),
                    );
                    let layer_bounds = render::scene::to_paint_rect_value_at_scale(
                        declaration.layer_bounds(),
                        self.viewport.scale_factor(),
                    );
                    let scroll_space = TargetSpace {
                        origin: [layer_bounds.origin.x(), layer_bounds.origin.y()],
                        size: [
                            layer_bounds.area.width().max(1.0),
                            layer_bounds.area.height().max(1.0),
                        ],
                    };
                    let mut members = Vec::new();
                    self.composited_scroll_depth = self.composited_scroll_depth.saturating_add(1);
                    self.build_order(
                        order,
                        index,
                        Some(PlanStop::Scroll),
                        nodes,
                        scroll_space,
                        &mut members,
                    )?;
                    self.composited_scroll_depth = self.composited_scroll_depth.saturating_sub(1);
                    self.stats.scene_items += 1;
                    target.push(RenderBatch::Scroll(PreparedScroll {
                        commit: self.commit.clone(),
                        node: *node,
                        scope: order_index,
                        viewport: local_rect(viewport, space.origin),
                        layer_bounds: local_rect(layer_bounds, space.origin),
                        baseline: declaration.baseline(),
                        render_batches: members,
                    }));
                }
                scene::Draw::PopGroup if stop == Some(PlanStop::Group) => return Ok(()),
                scene::Draw::PopGroup => {}
                scene::Draw::PopScroll if stop == Some(PlanStop::Scroll) => return Ok(()),
                scene::Draw::PopScroll => {}
            }
        }
        Ok(())
    }

    fn project_order_group(
        &self,
        order: &[scene::Draw],
        index: &mut usize,
        nodes: &HashMap<composition::tree::NodeId, Arc<scene::Node>>,
        opacity: f32,
    ) -> Option<crate::paint::Group> {
        let mut items = Vec::new();
        while let Some(draw) = order.get(*index) {
            *index = index.saturating_add(1);
            match draw {
                scene::Draw::Content { node, index } => {
                    let content = nodes.get(node)?.content().get(*index)?;
                    items.push(render::scene::to_paint_content_at_scale(
                        content,
                        self.viewport.scale_factor(),
                    ));
                }
                scene::Draw::PushClip { clip, .. } => {
                    items.push(crate::paint::Item::Clip(
                        render::scene::to_paint_clip_value_at_scale(
                            *clip,
                            self.viewport.scale_factor(),
                        ),
                    ));
                }
                scene::Draw::PopClip => items.push(crate::paint::Item::PopClip),
                scene::Draw::PushGroup { opacity, .. } => {
                    if let Some(group) = self.project_order_group(order, index, nodes, *opacity) {
                        items.push(crate::paint::Item::Group(group));
                    }
                }
                scene::Draw::PopGroup => break,
                scene::Draw::PushScroll { .. } | scene::Draw::PopScroll => {}
            }
        }
        crate::paint::group_from_items(
            &items,
            opacity,
            crate::paint::Grid::new(self.viewport.scale_factor()),
        )
    }

    fn build_node(
        &mut self,
        commit: &scene::Commit,
        node: &Arc<scene::Node>,
        parent_space: TargetSpace,
        target: &mut Vec<RenderBatch>,
    ) -> render::Result<()> {
        if let Some(clip) = node.clip() {
            self.stats.scene_items += 1;
            self.stats.clip_batches += 1;
            target.push(RenderBatch::PushClip(PreparedClip {
                node: Some(node.id()),
                fallback: render::scene::to_paint_clip_value_at_scale(
                    clip,
                    self.viewport.scale_factor(),
                ),
                scene_origin: parent_space.origin,
            }));
        }

        let own_group_bounds = match node.effect() {
            scene::EffectDeclaration::GroupOpacity(envelope) => {
                Some(self.group_bounds(commit, node, envelope.bounds()))
            }
            _ => None,
        };
        let body_space = own_group_bounds.map_or(parent_space, |bounds| TargetSpace {
            origin: [bounds.origin.x(), bounds.origin.y()],
            size: [bounds.area.width().max(1.0), bounds.area.height().max(1.0)],
        });
        let mut body = Vec::new();
        for (index, content) in node.content().iter().enumerate() {
            self.build_content(node, index, content, body_space, &mut body)?;
        }
        for child in commit
            .nodes()
            .iter()
            .filter(|child| child.parent() == Some(node.id()))
        {
            self.build_node(commit, child, body_space, &mut body)?;
        }

        match node.effect() {
            scene::EffectDeclaration::GroupOpacity(envelope) => {
                self.stats.scene_items += 1;
                self.stats.group_composites += 1;
                self.stats.effect_island_nodes += 1;
                target.push(RenderBatch::Group(PreparedGroup {
                    node: Some(node.id()),
                    bounds: local_group_bounds(
                        own_group_bounds.unwrap_or_else(|| {
                            render::scene::to_paint_rect_value_at_scale(
                                envelope.bounds(),
                                self.viewport.scale_factor(),
                            )
                        }),
                        parent_space.origin,
                    ),
                    opacity: 1.0,
                    render_batches: body,
                }));
            }
            scene::EffectDeclaration::Blur { envelope, .. }
            | scene::EffectDeclaration::Backdrop(envelope) => {
                let _ = envelope.maximum_sampling_reach();
                self.stats.effect_island_nodes += 1;
                target.extend(body);
            }
            scene::EffectDeclaration::None => target.extend(body),
        }

        if node.clip().is_some() {
            self.stats.scene_items += 1;
            self.stats.clip_batches += 1;
            target.push(RenderBatch::PopClip);
        }
        Ok(())
    }

    fn group_bounds(
        &self,
        commit: &scene::Commit,
        node: &scene::Node,
        envelope: crate::geometry::Rect,
    ) -> crate::paint::Rect {
        let fallback =
            render::scene::to_paint_rect_value_at_scale(envelope, self.viewport.scale_factor());
        if self.subtree_has_dynamic_geometry(commit, node) {
            return fallback;
        }
        let mut items = Vec::new();
        self.collect_group_items(commit, node, &mut items);
        crate::paint::group_from_items(
            &items,
            1.0,
            crate::paint::Grid::new(self.viewport.scale_factor()),
        )
        .map(|group| group.bounds)
        .unwrap_or(fallback)
    }

    fn subtree_has_dynamic_geometry(&self, commit: &scene::Commit, node: &scene::Node) -> bool {
        node.declares(scene::PropertyKind::Transform)
            || node.declares(scene::PropertyKind::ScrollOffset)
            || commit
                .nodes()
                .iter()
                .filter(|child| child.parent() == Some(node.id()))
                .any(|child| self.subtree_has_dynamic_geometry(commit, child))
    }

    fn collect_group_items(
        &self,
        commit: &scene::Commit,
        node: &scene::Node,
        target: &mut Vec<crate::paint::Item>,
    ) {
        target.extend(node.content().iter().map(|content| {
            render::scene::to_paint_content_at_scale(content, self.viewport.scale_factor())
        }));
        for child in commit
            .nodes()
            .iter()
            .filter(|child| child.parent() == Some(node.id()))
        {
            self.collect_group_items(commit, child, target);
        }
    }

    fn build_content(
        &mut self,
        node: &Arc<scene::Node>,
        content_index: usize,
        content: &scene::Content,
        space: TargetSpace,
        target: &mut Vec<RenderBatch>,
    ) -> render::Result<()> {
        self.stats.scene_items += 1;
        let item = render::scene::to_paint_content_at_scale(content, self.viewport.scale_factor());
        match &item {
            crate::paint::Item::Quad(value) => {
                let source_rect = match content {
                    scene::Content::Quad(quad) => Some([
                        quad.rect().x() as f32,
                        quad.rect().y() as f32,
                        quad.rect().width() as f32,
                        quad.rect().height() as f32,
                    ]),
                    _ => None,
                };
                self.push_shape(
                    node,
                    content_index,
                    0,
                    batch::Shape::Quad(value),
                    source_rect,
                    space,
                    target,
                )
            }
            crate::paint::Item::Rule(value) => self.push_shape(
                node,
                content_index,
                0,
                batch::Shape::Rule(value),
                None,
                space,
                target,
            ),
            crate::paint::Item::Shadow(value) => self.push_shape(
                node,
                content_index,
                0,
                batch::Shape::Shadow(value),
                None,
                space,
                target,
            ),
            crate::paint::Item::Outline(value) => self.push_shape(
                node,
                content_index,
                0,
                batch::Shape::Outline(value),
                None,
                space,
                target,
            ),
            crate::paint::Item::Text(_) => {
                let local = local_item_for_space(&item, space, self.viewport.scale_factor());
                let crate::paint::Item::Text(value) = &local else {
                    unreachable!("localized text must remain text")
                };
                self.push_glyph(
                    node,
                    content_index,
                    batch::Glyph::Text(value),
                    space,
                    target,
                )?
            }
            crate::paint::Item::TextViewport(_) => {
                let local = local_item_for_space(&item, space, self.viewport.scale_factor());
                let crate::paint::Item::TextViewport(value) = &local else {
                    unreachable!("localized text viewport must remain a text viewport")
                };
                self.stats.text_surfaces += value.surfaces.len();
                self.push_glyph(
                    node,
                    content_index,
                    batch::Glyph::TextViewport(value),
                    space,
                    target,
                )?;
            }
            crate::paint::Item::Icon(_) => {
                let local = local_item_for_space(&item, space, self.viewport.scale_factor());
                let crate::paint::Item::Icon(value) = &local else {
                    unreachable!("localized icon must remain an icon")
                };
                self.push_glyph(
                    node,
                    content_index,
                    batch::Glyph::Icon(value),
                    space,
                    target,
                )?
            }
            crate::paint::Item::Pane(value) => {
                self.push_pane(node, content_index, value, space, target);
            }
            crate::paint::Item::Clip(_)
            | crate::paint::Item::PopClip
            | crate::paint::Item::Group(_) => unreachable!("scene content is never structural"),
        }
        Ok(())
    }

    fn push_shape(
        &mut self,
        node: &Arc<scene::Node>,
        content_index: usize,
        part: u16,
        shape: batch::Shape<'_>,
        source_rect: Option<[f32; 4]>,
        space: TargetSpace,
        target: &mut Vec<RenderBatch>,
    ) {
        let binding = self.property_binding(node.id(), space);
        let (prepared, sync) = self.shapes.prepare(
            self.render_context,
            self.viewport,
            node,
            content_index,
            part,
            &[shape],
            source_rect,
            binding,
        );
        if sync.realization_rebuilds > 0 {
            self.rebuilt_nodes.insert(node.id());
        }
        apply_sync_stats(&mut self.stats, sync);
        if let Some(prepared) = prepared {
            self.stats.quad_instances += prepared.instance_count();
            target.push(RenderBatch::Shapes(EncodedShapeBatch::Retained(prepared)));
        }
    }

    fn push_glyph(
        &mut self,
        node: &Arc<scene::Node>,
        content_index: usize,
        glyph: batch::Glyph<'_>,
        space: TargetSpace,
        target: &mut Vec<RenderBatch>,
    ) -> render::Result<()> {
        self.stats.glyph_batches += 1;
        let report = self.text_renderer.prepare_retained(
            self.render_context,
            self.viewport,
            node,
            content_index,
            &[glyph],
            space.origin,
            space.size,
        )?;
        if report.prepare_calls > 0 {
            self.rebuilt_nodes.insert(node.id());
        }
        self.stats.text_prepare_calls += report.prepare_calls;
        self.stats.inline_text_cache_hits += report.stats.text_cache_hits;
        self.stats.inline_text_cache_misses += report.stats.text_cache_misses;
        self.stats.inline_text_shape_calls += report.stats.text_shape_calls;
        self.stats.inline_icon_cache_hits += report.stats.icon_cache_hits;
        self.stats.inline_icon_cache_misses += report.stats.icon_cache_misses;
        self.stats.inline_icon_shape_calls += report.stats.icon_shape_calls;
        self.stats.retained_gpu_resource_creations += report.resource_creations;
        self.stats.retained_gpu_resource_removals += report.resource_removals;
        if let Some(batch) = report.batch {
            target.push(RenderBatch::Text(TextBatch::Retained(batch)));
        }
        Ok(())
    }

    fn push_pane(
        &mut self,
        node: &Arc<scene::Node>,
        content_index: usize,
        pane: &crate::paint::Pane,
        space: TargetSpace,
        target: &mut Vec<RenderBatch>,
    ) {
        let base_brush = match &pane.material {
            crate::paint::Material::Solid(brush) => Some(*brush),
            crate::paint::Material::Glass(glass)
                if glass.base == crate::paint::GlassBase::Fallback =>
            {
                Some(glass.fallback)
            }
            crate::paint::Material::Glass(_) => None,
        };
        let surface_brushes = match &pane.material {
            crate::paint::Material::Solid(_) => Vec::new(),
            crate::paint::Material::Glass(glass) => glass
                .surface_layers
                .iter()
                .map(|layer| match *layer {
                    crate::paint::SurfaceLayer::Tint { brush, opacity } => {
                        Some(render::material::brush_with_opacity(brush, opacity))
                    }
                    crate::paint::SurfaceLayer::Noise(_) => None,
                })
                .collect::<Vec<_>>(),
        };
        let base = base_brush
            .and_then(|brush| self.prepare_brush(node, content_index, 0, pane.rect, brush, space));
        let surface_layers = surface_brushes
            .into_iter()
            .enumerate()
            .map(|(index, brush)| {
                brush.and_then(|brush| {
                    self.prepare_brush(
                        node,
                        content_index,
                        u16::try_from(index.saturating_add(1)).unwrap_or(u16::MAX),
                        pane.rect,
                        brush,
                        space,
                    )
                })
            })
            .collect();
        let prepared = PreparedPane {
            pane: pane
                .clone()
                .translated_for_group(crate::geometry::point::logical(
                    space.origin[0],
                    space.origin[1],
                )),
            base,
            surface_layers,
        };
        if matches!(pane.material, crate::paint::Material::Solid(_)) {
            if let Some(base) = prepared.base {
                target.push(RenderBatch::Shapes(base));
            }
        } else {
            self.stats.effect_island_nodes += 1;
            target.push(RenderBatch::Pane(prepared));
        }
    }

    fn prepare_brush(
        &mut self,
        node: &Arc<scene::Node>,
        content_index: usize,
        part: u16,
        rect: crate::paint::Rect,
        brush: crate::paint::Brush,
        space: TargetSpace,
    ) -> Option<EncodedShapeBatch> {
        if !brush.is_visible() {
            return None;
        }
        let quad = crate::paint::Quad::resolved_for_grid(
            rect,
            crate::paint::Style {
                fill: Some(crate::paint::Fill::Brush(brush)),
                stroke: None,
                tint: None,
            },
            crate::paint::Rasterization::default(),
            crate::paint::Transform::identity(),
            crate::paint::Grid::new(self.viewport.scale_factor()),
        );
        let binding = self.property_binding(node.id(), space);
        let (prepared, sync) = self.shapes.prepare(
            self.render_context,
            self.viewport,
            node,
            content_index,
            part,
            &[batch::Shape::Quad(&quad)],
            None,
            binding,
        );
        if sync.realization_rebuilds > 0 {
            self.rebuilt_nodes.insert(node.id());
        }
        apply_sync_stats(&mut self.stats, sync);
        if let Some(prepared) = &prepared {
            self.stats.quad_instances += prepared.instance_count();
        }
        prepared.map(EncodedShapeBatch::Retained)
    }
}

fn apply_sync_stats(stats: &mut render::DrawStats, sync: SyncStats) {
    stats.quad_prepare_calls += sync.primitive_prepare_calls;
    stats.content_upload_bytes += sync.content_upload_bytes;
    stats.property_upload_bytes += sync.property_upload_bytes;
    stats.retained_gpu_resource_creations += sync.resource_creations;
    stats.retained_gpu_resource_replacements += sync.resource_replacements;
    stats.retained_gpu_resource_removals += sync.resource_removals;
}

fn count_batches(batches: &[RenderBatch]) -> usize {
    batches
        .iter()
        .map(|batch| match batch {
            RenderBatch::Group(group) => 1 + count_batches(&group.render_batches),
            RenderBatch::Scroll(scroll) => 1 + count_batches(&scroll.render_batches),
            _ => 1,
        })
        .sum()
}

fn coalesce_shape_batches(batches: &mut Vec<RenderBatch>) {
    let mut coalesced = Vec::with_capacity(batches.len());
    for mut batch in batches.drain(..) {
        if let RenderBatch::Group(group) = &mut batch {
            coalesce_shape_batches(&mut group.render_batches);
        } else if let RenderBatch::Scroll(scroll) = &mut batch {
            coalesce_shape_batches(&mut scroll.render_batches);
        }
        let merged = match (coalesced.last_mut(), &batch) {
            (
                Some(RenderBatch::Shapes(EncodedShapeBatch::Retained(previous))),
                RenderBatch::Shapes(EncodedShapeBatch::Retained(next)),
            ) => previous.merge_adjacent(next),
            _ => false,
        };
        if !merged {
            coalesced.push(batch);
        }
    }
    *batches = coalesced;
}

fn local_group_bounds(
    mut bounds: crate::paint::Rect,
    parent_origin: [f32; 2],
) -> crate::paint::Rect {
    bounds.origin = crate::geometry::point::logical(
        bounds.origin.x() - parent_origin[0],
        bounds.origin.y() - parent_origin[1],
    );
    bounds
}

fn local_rect(mut bounds: crate::paint::Rect, parent_origin: [f32; 2]) -> crate::paint::Rect {
    bounds.origin = crate::geometry::point::logical(
        bounds.origin.x() - parent_origin[0],
        bounds.origin.y() - parent_origin[1],
    );
    bounds
}

fn local_item_for_space(
    item: &crate::paint::Item,
    space: TargetSpace,
    scale_factor: f32,
) -> crate::paint::Item {
    crate::paint::translate_item_for_group(
        item,
        crate::geometry::point::logical(space.origin[0], space.origin[1]),
        crate::paint::Grid::new(scale_factor),
    )
}

pub(in crate::render) struct PropertyBindings {
    offsets: HashMap<PropertyBinding, u32>,
}

impl PropertyBindings {
    fn offset(&self, binding: PropertyBinding) -> u32 {
        self.offsets.get(&binding).copied().unwrap_or_default()
    }
}

pub(in crate::render) struct Shapes {
    pipeline: wgpu::RenderPipeline,
    unit_buffer: wgpu::Buffer,
    instance_buffer: wgpu::Buffer,
    instance_capacity: usize,
    instances: Vec<render::quad::Instance>,
    free: Vec<Range<u32>>,
    entries: HashMap<ResourceKey, Entry>,
    viewport_buffer: wgpu::Buffer,
    property_buffer: wgpu::Buffer,
    property_capacity: usize,
    property_stride: usize,
    bind_group_layout: wgpu::BindGroupLayout,
    bind_group: wgpu::BindGroup,
    last_property_commit: Weak<scene::Commit>,
    last_property_serial: Option<scene::PropertySerial>,
    last_property_bindings: Vec<PropertyBinding>,
    last_property_viewport: [u32; 3],
}

impl Shapes {
    pub(in crate::render) fn new(
        render_context: &render::Context,
        format: wgpu::TextureFormat,
    ) -> Self {
        let device = render_context.device();
        let property_stride = align_up(
            std::mem::size_of::<NodeProperty>(),
            device.limits().min_uniform_buffer_offset_alignment as usize,
        );
        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("Retained Shape Bind Group Layout"),
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::VERTEX,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: NonZeroU64::new(
                            std::mem::size_of::<ViewportUniform>() as u64
                        ),
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::VERTEX,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: true,
                        min_binding_size: NonZeroU64::new(
                            std::mem::size_of::<NodeProperty>() as u64
                        ),
                    },
                    count: None,
                },
            ],
        });
        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Retained Shape Pipeline Layout"),
            bind_group_layouts: &[Some(&bind_group_layout)],
            immediate_size: 0,
        });
        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Retained Shape Shader"),
            source: wgpu::ShaderSource::Wgsl(
                render::silhouette::wgsl_module_source(RETAINED_QUAD_WGSL).into(),
            ),
        });
        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Retained Shape Pipeline"),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: Some("vs_main"),
                buffers: &[UnitVertex::layout(), render::quad::Instance::layout()],
                compilation_options: Default::default(),
            },
            primitive: wgpu::PrimitiveState::default(),
            depth_stencil: None,
            multisample: wgpu::MultisampleState::default(),
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: Some("fs_main"),
                targets: &[Some(render::alpha::color_target(
                    format,
                    render::alpha::FragmentOutput::Straight,
                ))],
                compilation_options: Default::default(),
            }),
            multiview_mask: None,
            cache: None,
        });
        let unit_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Retained Unit Quad"),
            contents: bytemuck::cast_slice(&UNIT_QUAD),
            usage: wgpu::BufferUsages::VERTEX,
        });
        let instance_buffer = create_instance_buffer(device, INITIAL_INSTANCE_CAPACITY);
        let viewport_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Retained Viewport Uniform"),
            size: std::mem::size_of::<ViewportUniform>() as u64,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        let property_buffer =
            create_property_buffer(device, property_stride, INITIAL_PROPERTY_CAPACITY);
        let bind_group = create_bind_group(
            device,
            &bind_group_layout,
            &viewport_buffer,
            &property_buffer,
        );

        Self {
            pipeline,
            unit_buffer,
            instance_buffer,
            instance_capacity: INITIAL_INSTANCE_CAPACITY,
            instances: Vec::new(),
            free: Vec::new(),
            entries: HashMap::new(),
            viewport_buffer,
            property_buffer,
            property_capacity: INITIAL_PROPERTY_CAPACITY,
            property_stride,
            bind_group_layout,
            bind_group,
            last_property_commit: Weak::new(),
            last_property_serial: None,
            last_property_bindings: Vec::new(),
            last_property_viewport: [0; 3],
        }
    }

    fn prepare(
        &mut self,
        render_context: &render::Context,
        viewport: render::Viewport,
        node: &Arc<scene::Node>,
        content_index: usize,
        part: u16,
        shapes: &[batch::Shape<'_>],
        source_rect: Option<[f32; 4]>,
        binding: PropertyBinding,
    ) -> (Option<ShapeBatch>, SyncStats) {
        let mut stats = self.prune();
        let key = ResourceKey::new(node, content_index, part, viewport.scale_factor());
        if let Some(entry) = self.entries.get_mut(&key) {
            entry.owners.retain(|owner| owner.strong_count() > 0);
            if !entry
                .owners
                .iter()
                .filter_map(Weak::upgrade)
                .any(|owner| Arc::ptr_eq(&owner, node))
            {
                entry.owners.push(Arc::downgrade(node));
            }
            return (
                entry
                    .range
                    .clone()
                    .map(|range| ShapeBatch { range, binding }),
                stats,
            );
        }

        stats.realization_rebuilds = 1;
        stats.primitive_prepare_calls = 1;
        let mut instances = render::quad::prepare_instances(viewport, shapes);
        if let Some(source_rect) = source_rect {
            for instance in &mut instances {
                instance.set_source_rect(source_rect);
            }
        }
        let range = NonZeroUsize::new(instances.len()).map(|count| self.allocate(count.get()));
        if let Some(range) = &range {
            self.instances[range.start as usize..range.end as usize].copy_from_slice(&instances);
            let required = self.instances.len();
            if required > self.instance_capacity {
                self.instance_capacity = required.next_power_of_two();
                self.instance_buffer =
                    create_instance_buffer(render_context.device(), self.instance_capacity);
                let bytes = bytemuck::cast_slice(&self.instances);
                render_context
                    .queue()
                    .write_buffer(&self.instance_buffer, 0, bytes);
                stats.content_upload_bytes += bytes.len();
                stats.resource_creations += 1;
            } else {
                let bytes = bytemuck::cast_slice(&instances);
                render_context.queue().write_buffer(
                    &self.instance_buffer,
                    range.start as u64 * std::mem::size_of::<render::quad::Instance>() as u64,
                    bytes,
                );
                stats.content_upload_bytes += bytes.len();
            }
        }
        stats.resource_creations += 1;
        self.entries.insert(
            key,
            Entry {
                owners: vec![Arc::downgrade(node)],
                range: range.clone(),
            },
        );

        (range.map(|range| ShapeBatch { range, binding }), stats)
    }

    fn prepare_properties(
        &mut self,
        render_context: &render::Context,
        viewport: render::Viewport,
        commit: &Arc<scene::Commit>,
        properties: &scene::Properties,
        bindings: &[PropertyBinding],
    ) -> (PropertyBindings, SyncStats) {
        let mut stats = SyncStats::default();
        if bindings.is_empty() {
            return (
                PropertyBindings {
                    offsets: HashMap::new(),
                },
                stats,
            );
        }
        let required = bindings.len().max(1);
        let mut buffer_recreated = false;
        if required > self.property_capacity {
            self.property_capacity = required.next_power_of_two();
            self.property_buffer = create_property_buffer(
                render_context.device(),
                self.property_stride,
                self.property_capacity,
            );
            self.bind_group = create_bind_group(
                render_context.device(),
                &self.bind_group_layout,
                &self.viewport_buffer,
                &self.property_buffer,
            );
            stats.resource_creations += 1;
            buffer_recreated = true;
        }

        let viewport_key = [
            viewport.logical_area().width().to_bits(),
            viewport.logical_area().height().to_bits(),
            viewport.scale_factor().to_bits(),
        ];
        let offsets = bindings
            .iter()
            .copied()
            .enumerate()
            .map(|(index, binding)| (binding, (index * self.property_stride) as u32))
            .collect::<HashMap<_, _>>();
        let unchanged = !buffer_recreated
            && self
                .last_property_commit
                .upgrade()
                .is_some_and(|previous| Arc::ptr_eq(&previous, commit))
            && self.last_property_serial == Some(properties.serial())
            && self.last_property_bindings == bindings
            && self.last_property_viewport == viewport_key;
        if unchanged {
            return (PropertyBindings { offsets }, stats);
        }

        let viewport_uniform = ViewportUniform {
            size: [
                viewport.logical_area().width().max(1.0),
                viewport.logical_area().height().max(1.0),
            ],
            padding: [0.0, 0.0],
        };
        render_context.queue().write_buffer(
            &self.viewport_buffer,
            0,
            bytemuck::bytes_of(&viewport_uniform),
        );
        stats.property_upload_bytes += std::mem::size_of::<ViewportUniform>();

        let mut bytes = vec![0_u8; required * self.property_stride];
        let mut inherited_scroll = HashMap::with_capacity(commit.nodes().len());
        for node in commit.nodes() {
            let parent_scroll = node
                .parent()
                .and_then(|parent| inherited_scroll.get(&parent).copied())
                .unwrap_or([0.0_f32; 2]);
            let own_scroll = match properties.value(scene::PropertyRef::new(
                node.id(),
                scene::PropertyKind::ScrollOffset,
            )) {
                Some(scene::PropertyValue::ScrollOffset { x, y, .. }) => [-x.round(), -y.round()],
                _ => [0.0, 0.0],
            };
            let scroll = [
                parent_scroll[0] + own_scroll[0],
                parent_scroll[1] + own_scroll[1],
            ];
            inherited_scroll.insert(node.id(), scroll);
        }

        for (index, binding) in bindings.iter().copied().enumerate() {
            let Some(node) = commit.nodes().iter().find(|node| node.id() == binding.node) else {
                continue;
            };

            let mut property = NodeProperty::IDENTITY;
            if !binding.composited_scroll {
                property.scroll = inherited_scroll
                    .get(&node.id())
                    .copied()
                    .unwrap_or_default();
            }
            property.grid[0] = viewport.scale_factor();
            property.scene_origin = binding.space.origin;
            property.target_size = binding.space.size;
            if let Some(scene::PropertyValue::Transform { value, .. }) = properties.value(
                scene::PropertyRef::new(node.id(), scene::PropertyKind::Transform),
            ) {
                property.origin = [value.origin_x(), value.origin_y()];
                property.translate = [value.translate_x(), value.translate_y()];
                property.scale = [value.scale_x(), value.scale_y()];
                property.grid[1] = 1.0;
            }
            let offset = index * self.property_stride;
            bytes[offset..offset + std::mem::size_of::<NodeProperty>()]
                .copy_from_slice(bytemuck::bytes_of(&property));
        }
        render_context
            .queue()
            .write_buffer(&self.property_buffer, 0, &bytes);
        stats.property_upload_bytes += bytes.len();

        self.last_property_commit = Arc::downgrade(commit);
        self.last_property_serial = Some(properties.serial());
        self.last_property_bindings = bindings.to_vec();
        self.last_property_viewport = viewport_key;

        (PropertyBindings { offsets }, stats)
    }

    pub(in crate::render) fn draw<'a>(
        &'a self,
        pass: &mut wgpu::RenderPass<'a>,
        batch: &ShapeBatch,
        properties: &PropertyBindings,
    ) {
        pass.set_pipeline(&self.pipeline);
        pass.set_bind_group(0, &self.bind_group, &[properties.offset(batch.binding())]);
        pass.set_vertex_buffer(0, self.unit_buffer.slice(..));
        pass.set_vertex_buffer(1, self.instance_buffer.slice(..));
        pass.draw(0..UNIT_QUAD.len() as u32, batch.range());
    }

    pub(in crate::render) fn resource_count(&self) -> usize {
        self.entries.len().saturating_add(4)
    }

    pub(in crate::render) fn collect(&mut self) -> SyncStats {
        self.prune()
    }

    pub(in crate::render) fn resource_bytes(&self) -> usize {
        UNIT_QUAD
            .len()
            .saturating_mul(std::mem::size_of::<UnitVertex>())
            .saturating_add(
                self.instance_capacity
                    .saturating_mul(std::mem::size_of::<render::quad::Instance>()),
            )
            .saturating_add(std::mem::size_of::<ViewportUniform>())
            .saturating_add(self.property_capacity.saturating_mul(self.property_stride))
    }

    fn prune(&mut self) -> SyncStats {
        let expired = self
            .entries
            .iter()
            .filter_map(|(key, entry)| {
                entry
                    .owners
                    .iter()
                    .all(|owner| owner.strong_count() == 0)
                    .then_some(*key)
            })
            .collect::<Vec<_>>();
        for key in &expired {
            if let Some(entry) = self.entries.remove(key)
                && let Some(range) = entry.range
            {
                self.release(range);
            }
        }
        SyncStats {
            resource_removals: expired.len(),
            ..SyncStats::default()
        }
    }

    fn allocate(&mut self, count: usize) -> Range<u32> {
        if let Some(index) = self
            .free
            .iter()
            .position(|range| (range.end - range.start) as usize >= count)
        {
            let range = self.free[index].clone();
            let allocated = range.start..range.start + count as u32;
            if allocated.end == range.end {
                self.free.swap_remove(index);
            } else {
                self.free[index].start = allocated.end;
            }
            return allocated;
        }

        let start = self.instances.len();
        self.instances.resize(
            start.saturating_add(count),
            render::quad::Instance::zeroed(),
        );
        start as u32..(start + count) as u32
    }

    fn release(&mut self, range: Range<u32>) {
        self.free.push(range);
        self.free.sort_by_key(|range| range.start);
        let mut merged: Vec<Range<u32>> = Vec::with_capacity(self.free.len());
        for range in self.free.drain(..) {
            if let Some(previous) = merged.last_mut()
                && previous.end == range.start
            {
                previous.end = range.end;
            } else {
                merged.push(range);
            }
        }
        self.free = merged;
    }
}

fn create_instance_buffer(device: &wgpu::Device, capacity: usize) -> wgpu::Buffer {
    device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("Retained Shape Instances"),
        size: capacity.max(1) as u64 * std::mem::size_of::<render::quad::Instance>() as u64,
        usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
        mapped_at_creation: false,
    })
}

fn create_property_buffer(device: &wgpu::Device, stride: usize, capacity: usize) -> wgpu::Buffer {
    device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("Retained Node Properties"),
        size: stride.saturating_mul(capacity.max(1)) as u64,
        usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        mapped_at_creation: false,
    })
}

fn create_bind_group(
    device: &wgpu::Device,
    layout: &wgpu::BindGroupLayout,
    viewport: &wgpu::Buffer,
    properties: &wgpu::Buffer,
) -> wgpu::BindGroup {
    device.create_bind_group(&wgpu::BindGroupDescriptor {
        label: Some("Retained Shape Bind Group"),
        layout,
        entries: &[
            wgpu::BindGroupEntry {
                binding: 0,
                resource: viewport.as_entire_binding(),
            },
            wgpu::BindGroupEntry {
                binding: 1,
                resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding {
                    buffer: properties,
                    offset: 0,
                    size: NonZeroU64::new(std::mem::size_of::<NodeProperty>() as u64),
                }),
            },
        ],
    })
}

fn align_up(value: usize, alignment: usize) -> usize {
    let alignment = alignment.max(1);
    value.div_ceil(alignment).saturating_mul(alignment)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn binding(value: u64) -> PropertyBinding {
        let mut value = value;
        PropertyBinding {
            node: composition::tree::NodeId::layout(&mut value),
            space: TargetSpace {
                origin: [0.0, 0.0],
                size: [100.0, 80.0],
            },
            composited_scroll: false,
        }
    }

    fn shapes(range: Range<u32>, binding: PropertyBinding) -> RenderBatch {
        RenderBatch::Shapes(EncodedShapeBatch::Retained(ShapeBatch { range, binding }))
    }

    #[test]
    fn coalescing_merges_only_adjacent_instances_with_the_same_property_binding() {
        let first = binding(1);
        let second = binding(2);
        let mut batches = vec![
            shapes(0..2, first),
            shapes(2..5, first),
            shapes(5..6, second),
            shapes(8..9, second),
        ];

        coalesce_shape_batches(&mut batches);

        assert_eq!(batches.len(), 3);
        let RenderBatch::Shapes(EncodedShapeBatch::Retained(merged)) = &batches[0] else {
            panic!("first batch should remain retained shapes");
        };
        assert_eq!(merged.range(), 0..5);
        assert_eq!(merged.instance_count(), 5);
    }

    #[test]
    fn semantic_boundaries_prevent_shape_coalescing() {
        let binding = binding(1);
        let mut batches = vec![
            shapes(0..2, binding),
            RenderBatch::Text(TextBatch::Immediate { renderer_index: 0 }),
            shapes(2..4, binding),
        ];

        coalesce_shape_batches(&mut batches);

        assert_eq!(batches.len(), 3);
    }
}
