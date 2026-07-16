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
    self as render, content,
    renderer::{PlanStep, PreparedClip, PreparedGroup, PreparedPane, PreparedScroll},
};

const RETAINED_QUAD_WGSL: &str = include_str!("retained_quad.wgsl");
const INITIAL_INSTANCE_CAPACITY: usize = 256;
const INITIAL_PROPERTY_CAPACITY: usize = 256;
const RETAINED_PROPERTY_SLOT_RESERVE: usize = 2;
const INITIAL_SCROLL_PROPERTY_CAPACITY: usize = 16;
const RETAINED_SCROLL_PROPERTY_SLOT_RESERVE: usize = 2;

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
    scale: [f32; 2],
    grid: [f32; 2],
    scene_origin: [f32; 2],
    target_size: [f32; 2],
    opacity: f32,
    padding: [f32; 3],
}

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
struct ScrollProperty {
    translation: [f32; 2],
    padding: [f32; 2],
}

impl ScrollProperty {
    const IDENTITY: Self = Self {
        translation: [0.0; 2],
        padding: [0.0; 2],
    };
}

impl NodeProperty {
    const IDENTITY: Self = Self {
        origin: [0.0, 0.0],
        translate: [0.0, 0.0],
        scale: [1.0, 1.0],
        grid: [1.0, 0.0],
        scene_origin: [0.0, 0.0],
        target_size: [1.0, 1.0],
        opacity: 1.0,
        padding: [0.0; 3],
    };
}

fn project_scrollbar_property(
    property: &mut NodeProperty,
    projection: scene::ContentProjection,
    node: composition::tree::NodeId,
    properties: &scene::Properties,
    scale_factor: f32,
) {
    let (axis, edge, base_thickness, maximum_thickness, thumb) = match projection {
        scene::ContentProjection::Normal | scene::ContentProjection::Caret => return,
        scene::ContentProjection::ScrollbarTrack {
            axis,
            edge,
            base_thickness,
            maximum_thickness,
        } => (axis, edge, base_thickness, maximum_thickness, None),
        scene::ContentProjection::ScrollbarThumb {
            axis,
            edge,
            base_thickness,
            maximum_thickness,
            baseline_start,
            baseline_extent,
            baseline_position,
            ..
        } => (
            axis,
            edge,
            base_thickness,
            maximum_thickness,
            Some((baseline_start, baseline_extent, baseline_position)),
        ),
    };
    let (opacity, thickness) = properties
        .scrollbar(node, axis)
        .unwrap_or((0.0, base_thickness as f32));
    property.opacity = opacity.clamp(0.0, 1.0);

    let base_thickness = base_thickness.max(1);
    let maximum_thickness = maximum_thickness.max(base_thickness);
    let thickness = (thickness.round() as i32).clamp(base_thickness, maximum_thickness);
    let grid = crate::paint::Grid::new(scale_factor);
    let cross_axis = match axis {
        crate::interaction::ScrollbarAxis::Vertical => 0,
        crate::interaction::ScrollbarAxis::Horizontal => 1,
    };
    map_property_span(
        property,
        cross_axis,
        grid,
        edge.saturating_sub(base_thickness),
        base_thickness,
        edge.saturating_sub(thickness),
        thickness,
    );

    if let Some((baseline_start, extent, baseline_position)) = thumb
        && let Some(offset) = properties.scroll_offset(node)
    {
        let position = projection
            .scrollbar_position(offset)
            .expect("scrollbar thumb projection must have one integral position");
        let target_start = baseline_start
            .saturating_sub(baseline_position)
            .saturating_add(position);
        let main_axis = 1_usize.saturating_sub(cross_axis);
        map_property_span(
            property,
            main_axis,
            grid,
            baseline_start,
            extent,
            target_start,
            extent,
        );
    }
}

fn map_property_span(
    property: &mut NodeProperty,
    axis: usize,
    grid: crate::paint::Grid,
    source_start: i32,
    source_extent: i32,
    target_start: i32,
    target_extent: i32,
) {
    let source = snapped_span(grid, source_start, source_extent);
    let target = snapped_span(grid, target_start, target_extent);
    property.origin[axis] = source.0;
    property.translate[axis] = target.0 - source.0;
    property.scale[axis] = (target.1 - target.0) / (source.1 - source.0);
}

fn snapped_span(grid: crate::paint::Grid, start: i32, extent: i32) -> (f32, f32) {
    let logical_start = start as f32;
    let start = grid.snap_position(logical_start);
    let mut end = grid.snap_position(logical_start + extent.max(1) as f32);
    if end <= start {
        end = start + grid.logical_pixel();
    }
    (start, end)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub(in crate::render) struct ResourceKey {
    node: composition::tree::NodeId,
    content_revision: composition::tree::ContentRevision,
    geometry_revision: scene::GeometryRevision,
    topology_revision: scene::TopologyRevision,
    content_index: usize,
    part: u16,
    realization: u8,
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
            realization: 0,
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

    fn with_realization(mut self, realization: u8) -> Self {
        self.realization = realization;
        self
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
    batches: Vec<PlanStep>,
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
    pub(in crate::render) fn batches(&self) -> &[PlanStep] {
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
    viewport: render::Viewport,
    projection: Projection,
    plan: Arc<Plan>,
}

#[derive(Debug, Clone, PartialEq)]
struct Projection {
    origin: [f32; 2],
    material: scene::MaterialProjection,
}

impl Projection {
    #[cfg(feature = "renderer-debug")]
    fn source() -> Self {
        Self {
            origin: [0.0, 0.0],
            material: scene::MaterialProjection::Source,
        }
    }

    fn from_layer(layer: &scene::Layer) -> Self {
        Self {
            origin: [layer.origin().x() as f32, layer.origin().y() as f32],
            material: layer.material().clone(),
        }
    }
}

#[derive(Debug, Clone, Copy)]
struct TargetSpace {
    origin: [f32; 2],
    size: [f32; 2],
    text_origin: [f32; 2],
    text_size: [f32; 2],
    target: Option<composition::tree::NodeId>,
    scroll_root: Option<composition::tree::NodeId>,
    current_scroll: Option<composition::tree::NodeId>,
}

impl PartialEq for TargetSpace {
    fn eq(&self, other: &Self) -> bool {
        self.origin.map(f32::to_bits) == other.origin.map(f32::to_bits)
            && self.size.map(f32::to_bits) == other.size.map(f32::to_bits)
            && self.text_origin.map(f32::to_bits) == other.text_origin.map(f32::to_bits)
            && self.text_size.map(f32::to_bits) == other.text_size.map(f32::to_bits)
            && self.target == other.target
            && self.scroll_root == other.scroll_root
            && self.current_scroll == other.current_scroll
    }
}

impl Eq for TargetSpace {}

impl std::hash::Hash for TargetSpace {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.origin.map(f32::to_bits).hash(state);
        self.size.map(f32::to_bits).hash(state);
        self.text_origin.map(f32::to_bits).hash(state);
        self.text_size.map(f32::to_bits).hash(state);
        self.target.hash(state);
        self.scroll_root.hash(state);
        self.current_scroll.hash(state);
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
struct ScrollBinding {
    node: Option<composition::tree::NodeId>,
    root: Option<composition::tree::NodeId>,
}

impl ScrollBinding {
    const IDENTITY: Self = Self {
        node: None,
        root: None,
    };
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
struct PropertyBinding {
    node: composition::tree::NodeId,
    space: TargetSpace,
    projection: scene::ContentProjection,
}

impl PropertyBinding {
    fn scroll(self) -> ScrollBinding {
        ScrollBinding {
            node: self.space.current_scroll,
            root: self.space.scroll_root,
        }
    }
}

pub(in crate::render) struct Prepared {
    pub(in crate::render) plan: Arc<Plan>,
    pub(in crate::render) properties: PropertyBindings,
    pub(in crate::render) stats: render::DrawStats,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(in crate::render) enum Synchronize {
    Pending,
    Prepared,
    Ready,
}

pub(in crate::render) struct Realizer {
    shapes: Shapes,
    plans: Vec<PlanEntry>,
    pending: Vec<PendingPlan>,
    prepared_stats: Vec<PreparedStats>,
}

struct PreparedStats {
    commit: Weak<scene::Commit>,
    viewport: render::Viewport,
    projection: Projection,
    stats: render::DrawStats,
}

struct PendingPlan {
    commit: Arc<scene::Commit>,
    viewport: render::Viewport,
    projection: Projection,
    nodes: HashMap<composition::tree::NodeId, Arc<scene::Node>>,
    order: Option<Arc<[scene::Draw]>>,
    order_index: usize,
    frames: Vec<PendingFrame>,
    rebuilt_nodes: HashSet<composition::tree::NodeId>,
    property_bindings: Vec<PropertyBinding>,
    stats: render::DrawStats,
}

struct PendingFrame {
    kind: PendingFrameKind,
    space: TargetSpace,
    batches: Vec<PlanStep>,
}

enum PendingFrameKind {
    Root,
    Group {
        node: composition::tree::NodeId,
        bounds: crate::paint::Rect,
        opacity: f32,
        parent_origin: [f32; 2],
    },
    Scroll {
        node: composition::tree::NodeId,
        viewport: crate::paint::Rect,
        baseline: crate::interaction::ScrollOffset,
        parent_origin: [f32; 2],
    },
}

impl Realizer {
    pub(in crate::render) fn new(
        render_context: &render::Context,
        format: wgpu::TextureFormat,
    ) -> Self {
        Self {
            shapes: Shapes::new(render_context, format),
            plans: Vec::new(),
            pending: Vec::new(),
            prepared_stats: Vec::new(),
        }
    }

    #[cfg(feature = "renderer-debug")]
    pub(in crate::render) fn synchronize_bounded(
        &mut self,
        render_context: &render::Context,
        viewport: render::Viewport,
        commit: &Arc<scene::Commit>,
        text_renderer: &mut render::text_renderer::TextRenderer,
        budget: std::time::Duration,
        deadline: std::time::Duration,
    ) -> render::Result<Synchronize> {
        self.synchronize_projected(
            render_context,
            viewport,
            commit,
            Projection::source(),
            text_renderer,
            budget,
            deadline,
        )
    }

    pub(in crate::render) fn synchronize_layer(
        &mut self,
        render_context: &render::Context,
        viewport: render::Viewport,
        layer: &scene::Layer,
        text_renderer: &mut render::text_renderer::TextRenderer,
        budget: std::time::Duration,
        deadline: std::time::Duration,
    ) -> render::Result<Synchronize> {
        validate_residencies(layer)?;
        self.synchronize_projected(
            render_context,
            viewport,
            layer.drawable_commit(),
            Projection::from_layer(layer),
            text_renderer,
            budget,
            deadline,
        )
    }

    fn synchronize_projected(
        &mut self,
        render_context: &render::Context,
        viewport: render::Viewport,
        commit: &Arc<scene::Commit>,
        projection: Projection,
        text_renderer: &mut render::text_renderer::TextRenderer,
        budget: std::time::Duration,
        deadline: std::time::Duration,
    ) -> render::Result<Synchronize> {
        self.collect_pending();
        self.retain_commit_projection(commit, viewport, &projection);
        if self.find_plan(commit, viewport, &projection).is_some() {
            return Ok(Synchronize::Ready);
        }

        let started_at = std::time::Instant::now();
        let pending_index = self.pending.iter().position(|pending| {
            pending.viewport == viewport
                && pending.projection == projection
                && Arc::ptr_eq(&pending.commit, commit)
        });
        let mut pending = pending_index.map_or_else(
            || PendingPlan::new(Arc::clone(commit), viewport, projection.clone()),
            |index| self.pending.swap_remove(index),
        );
        let mut builder = PlanBuilder {
            render_context,
            viewport,
            shapes: &mut self.shapes,
            text_renderer,
            projection: &projection,
            rebuilt_nodes: std::mem::take(&mut pending.rebuilt_nodes),
            property_bindings: std::mem::take(&mut pending.property_bindings),
            stats: std::mem::take(&mut pending.stats),
        };
        let ready = pending.advance(&mut builder, budget)?;
        pending.rebuilt_nodes = builder.rebuilt_nodes;
        pending.property_bindings = builder.property_bindings;
        pending.stats = builder.stats;
        let elapsed = started_at.elapsed();
        let elapsed_nanos = elapsed.as_nanos().min(u128::from(u64::MAX)) as u64;
        pending.stats.commit_preparation_slices =
            pending.stats.commit_preparation_slices.saturating_add(1);
        pending.stats.commit_preparation_total_nanos = pending
            .stats
            .commit_preparation_total_nanos
            .saturating_add(elapsed_nanos);
        pending.stats.commit_preparation_max_nanos = pending
            .stats
            .commit_preparation_max_nanos
            .max(elapsed_nanos);
        pending.stats.commit_preparation_deadline_misses = pending
            .stats
            .commit_preparation_deadline_misses
            .saturating_add(usize::from(elapsed >= deadline));

        if !ready {
            self.pending.push(pending);
            return Ok(Synchronize::Pending);
        }

        pending.stats.scene_node_realization_rebuilds = pending.rebuilt_nodes.len();
        pending.stats.render_plan_rebuilds = 1;
        let mut batches = pending
            .frames
            .pop()
            .expect("completed retained preparation must keep its root frame")
            .batches;
        coalesce_shape_batches(&mut batches);
        let requires_surface_sampling = render::renderer::requires_surface_sampling(&batches);
        let plan = Arc::new(Plan {
            batches,
            property_bindings: pending.property_bindings,
            requires_surface_sampling,
            facts: PlanFacts::from_stats(&pending.stats),
        });
        self.plans.push(PlanEntry {
            commit: Arc::downgrade(commit),
            viewport,
            projection: projection.clone(),
            plan,
        });
        self.prepared_stats.push(PreparedStats {
            commit: Arc::downgrade(commit),
            viewport,
            projection,
            stats: pending.stats,
        });
        Ok(Synchronize::Prepared)
    }

    #[cfg(feature = "renderer-debug")]
    pub(in crate::render) fn cancel_synchronization(&mut self, commit: &Arc<scene::Commit>) {
        self.pending
            .retain(|pending| !Arc::ptr_eq(&pending.commit, commit));
        self.shapes.cancel_property_state(commit);
    }

    pub(in crate::render) fn cancel_layer_synchronization(&mut self, layer: &scene::Layer) {
        let projection = Projection::from_layer(layer);
        self.pending.retain(|pending| {
            !Arc::ptr_eq(&pending.commit, layer.drawable_commit())
                || pending.projection != projection
        });
    }

    pub(in crate::render) fn cancel_property_state(&mut self, commit: &Arc<scene::Commit>) {
        self.shapes.cancel_property_state(commit);
    }

    #[cfg(feature = "renderer-debug")]
    pub(in crate::render) fn prepare(
        &mut self,
        render_context: &render::Context,
        viewport: render::Viewport,
        commit: &Arc<scene::Commit>,
        properties: &scene::Properties,
        text_renderer: &mut render::text_renderer::TextRenderer,
    ) -> render::Result<Prepared> {
        self.prepare_projected(
            render_context,
            viewport,
            commit,
            properties,
            Projection::source(),
            text_renderer,
        )
    }

    pub(in crate::render) fn prepare_layer(
        &mut self,
        render_context: &render::Context,
        viewport: render::Viewport,
        layer: &scene::Layer,
        text_renderer: &mut render::text_renderer::TextRenderer,
    ) -> render::Result<Prepared> {
        validate_residencies(layer)?;
        self.prepare_projected(
            render_context,
            viewport,
            layer.drawable_commit(),
            layer.properties(),
            Projection::from_layer(layer),
            text_renderer,
        )
    }

    fn prepare_projected(
        &mut self,
        render_context: &render::Context,
        viewport: render::Viewport,
        commit: &Arc<scene::Commit>,
        properties: &scene::Properties,
        projection: Projection,
        text_renderer: &mut render::text_renderer::TextRenderer,
    ) -> render::Result<Prepared> {
        properties
            .require_compatible(commit)
            .map_err(|_| render::Error::RetainedSceneContract)?;

        let mut stats = render::DrawStats::default();
        let _expired_plans = self.collect_plans();
        self.retain_commit_projection(commit, viewport, &projection);
        stats.retained_gpu_resource_removals += self.shapes.collect().resource_removals;
        stats.retained_gpu_resource_removals += text_renderer.collect_retained();

        let plan = self.find_plan(commit, viewport, &projection);

        let plan = if let Some(plan) = plan {
            if let Some(prepared) = self.take_prepared_stats(commit, viewport, &projection) {
                stats.add(prepared);
            } else {
                stats.render_plan_reuses = 1;
            }
            plan
        } else {
            let mut builder = PlanBuilder {
                render_context,
                viewport,
                shapes: &mut self.shapes,
                text_renderer,
                projection: &projection,
                rebuilt_nodes: HashSet::new(),
                property_bindings: Vec::new(),
                stats: render::DrawStats::default(),
            };
            let built = builder.build(commit)?;
            stats.add(builder.stats);
            stats.scene_node_realization_rebuilds = builder.rebuilt_nodes.len();
            stats.render_plan_rebuilds = 1;
            let plan = Arc::new(built);
            self.plans.push(PlanEntry {
                commit: Arc::downgrade(commit),
                viewport,
                projection,
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
        apply_sync_stats(
            &mut stats,
            prepare_text_transforms(
                render_context,
                viewport,
                commit,
                properties,
                plan.batches(),
                text_renderer,
            ),
        );
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

    #[cfg(feature = "renderer-debug")]
    pub(in crate::render) fn prepare_candidate(
        &mut self,
        render_context: &render::Context,
        viewport: render::Viewport,
        commit: &Arc<scene::Commit>,
        properties: &scene::Properties,
        text_renderer: &mut render::text_renderer::TextRenderer,
    ) -> render::Result<Option<Prepared>> {
        self.prepare_candidate_projected(
            render_context,
            viewport,
            commit,
            properties,
            &Projection::source(),
            text_renderer,
        )
    }

    pub(in crate::render) fn prepare_candidate_layer(
        &mut self,
        render_context: &render::Context,
        viewport: render::Viewport,
        layer: &scene::Layer,
        text_renderer: &mut render::text_renderer::TextRenderer,
    ) -> render::Result<Option<Prepared>> {
        validate_residencies(layer)?;
        self.prepare_candidate_projected(
            render_context,
            viewport,
            layer.drawable_commit(),
            layer.properties(),
            &Projection::from_layer(layer),
            text_renderer,
        )
    }

    fn prepare_candidate_projected(
        &mut self,
        render_context: &render::Context,
        viewport: render::Viewport,
        commit: &Arc<scene::Commit>,
        properties: &scene::Properties,
        projection: &Projection,
        text_renderer: &mut render::text_renderer::TextRenderer,
    ) -> render::Result<Option<Prepared>> {
        properties
            .require_compatible(commit)
            .map_err(|_| render::Error::RetainedSceneContract)?;
        let Some(plan) = self.find_plan(commit, viewport, projection) else {
            return Ok(None);
        };
        let (property_bindings, property_stats) = self.shapes.prepare_properties(
            render_context,
            viewport,
            commit,
            properties,
            &plan.property_bindings,
        );
        let mut stats = render::DrawStats::default();
        apply_sync_stats(&mut stats, property_stats);
        apply_sync_stats(
            &mut stats,
            prepare_text_transforms(
                render_context,
                viewport,
                commit,
                properties,
                plan.batches(),
                text_renderer,
            ),
        );
        if let Some(prepared) = self.prepared_stats.iter_mut().find(|entry| {
            entry.viewport == viewport
                && entry.projection == *projection
                && entry
                    .commit
                    .upgrade()
                    .is_some_and(|candidate| Arc::ptr_eq(&candidate, commit))
        }) {
            prepared.stats.add(stats);
        }

        Ok(Some(Prepared {
            plan,
            properties: property_bindings,
            stats: render::DrawStats::default(),
        }))
    }

    #[cfg(feature = "renderer-debug")]
    pub(in crate::render) fn record_candidate_slice(
        &mut self,
        viewport: render::Viewport,
        commit: &Arc<scene::Commit>,
        elapsed: std::time::Duration,
        deadline: std::time::Duration,
    ) {
        let projection = Projection::source();
        let Some(prepared) = self.prepared_stats.iter_mut().find(|entry| {
            entry.viewport == viewport
                && entry.projection == projection
                && entry
                    .commit
                    .upgrade()
                    .is_some_and(|candidate| Arc::ptr_eq(&candidate, commit))
        }) else {
            return;
        };
        record_candidate_timing(&mut prepared.stats, elapsed, deadline);
    }

    pub(in crate::render) fn record_candidate_layer_slice(
        &mut self,
        viewport: render::Viewport,
        layer: &scene::Layer,
        elapsed: std::time::Duration,
        deadline: std::time::Duration,
    ) {
        let projection = Projection::from_layer(layer);
        let Some(prepared) = self.prepared_stats.iter_mut().find(|entry| {
            entry.viewport == viewport
                && entry.projection == projection
                && entry
                    .commit
                    .upgrade()
                    .is_some_and(|candidate| Arc::ptr_eq(&candidate, layer.drawable_commit()))
        }) else {
            return;
        };
        record_candidate_timing(&mut prepared.stats, elapsed, deadline);
    }

    pub(in crate::render) fn shapes(&self) -> &Shapes {
        &self.shapes
    }

    #[cfg(feature = "renderer-debug")]
    pub(in crate::render) fn debug_state_counts(&self) -> (usize, usize) {
        (self.plans.len(), self.pending.len())
    }

    #[cfg(feature = "renderer-debug")]
    pub(in crate::render) fn debug_resource_state(&self) -> (usize, usize) {
        (self.shapes.resource_count(), self.shapes.resource_bytes())
    }

    fn collect_plans(&mut self) -> usize {
        let before = self.plans.len();
        self.plans.retain(|entry| entry.commit.strong_count() > 0);
        self.prepared_stats
            .retain(|entry| entry.commit.strong_count() > 0);
        before.saturating_sub(self.plans.len())
    }

    fn collect_pending(&mut self) {
        self.pending
            .retain(|pending| Arc::strong_count(&pending.commit) > 1);
    }

    fn retain_commit_projection(
        &mut self,
        commit: &Arc<scene::Commit>,
        viewport: render::Viewport,
        projection: &Projection,
    ) {
        self.shapes.retain_property_viewport(commit, viewport);
        self.pending.retain(|pending| {
            !Arc::ptr_eq(&pending.commit, commit)
                || (pending.viewport == viewport && pending.projection == *projection)
        });
        self.plans.retain(|entry| {
            entry.commit.upgrade().is_none_or(|candidate| {
                !Arc::ptr_eq(&candidate, commit)
                    || (entry.viewport == viewport && entry.projection == *projection)
            })
        });
        self.prepared_stats.retain(|entry| {
            entry.commit.upgrade().is_none_or(|candidate| {
                !Arc::ptr_eq(&candidate, commit)
                    || (entry.viewport == viewport && entry.projection == *projection)
            })
        });
    }

    fn find_plan(
        &self,
        commit: &Arc<scene::Commit>,
        viewport: render::Viewport,
        projection: &Projection,
    ) -> Option<Arc<Plan>> {
        self.plans.iter().find_map(|entry| {
            (entry.viewport == viewport && entry.projection == *projection)
                .then(|| entry.commit.upgrade())
                .flatten()
                .filter(|candidate| Arc::ptr_eq(candidate, commit))
                .map(|_| Arc::clone(&entry.plan))
        })
    }

    fn take_prepared_stats(
        &mut self,
        commit: &Arc<scene::Commit>,
        viewport: render::Viewport,
        projection: &Projection,
    ) -> Option<render::DrawStats> {
        let index = self.prepared_stats.iter().position(|entry| {
            entry.viewport == viewport
                && entry.projection == *projection
                && entry
                    .commit
                    .upgrade()
                    .is_some_and(|candidate| Arc::ptr_eq(&candidate, commit))
        })?;
        Some(self.prepared_stats.swap_remove(index).stats)
    }
}

fn validate_residencies(layer: &scene::Layer) -> render::Result<()> {
    for residency in layer.residencies() {
        residency
            .require_compatible(layer.commit())
            .map_err(|_| render::Error::RetainedSceneContract)?;
        let nodes = residency.node_ids().collect::<HashSet<_>>();
        if !nodes.contains(&residency.scroll())
            || residency
                .draw_order()
                .iter()
                .any(|node| !nodes.contains(node))
            || nodes.iter().any(|node| {
                !layer
                    .drawable_commit()
                    .nodes()
                    .iter()
                    .any(|candidate| candidate.id() == *node)
            })
            || layer
                .properties()
                .scroll_offset(residency.scroll())
                .is_some_and(|offset| !residency.accepts(offset))
        {
            return Err(render::Error::RetainedSceneContract);
        }
    }
    Ok(())
}

fn record_candidate_timing(
    stats: &mut render::DrawStats,
    elapsed: std::time::Duration,
    deadline: std::time::Duration,
) {
    let elapsed_nanos = elapsed.as_nanos().min(u128::from(u64::MAX)) as u64;
    stats.commit_preparation_slices = stats.commit_preparation_slices.saturating_add(1);
    stats.commit_preparation_total_nanos = stats
        .commit_preparation_total_nanos
        .saturating_add(elapsed_nanos);
    stats.commit_preparation_max_nanos = stats.commit_preparation_max_nanos.max(elapsed_nanos);
    stats.commit_preparation_deadline_misses = stats
        .commit_preparation_deadline_misses
        .saturating_add(usize::from(elapsed >= deadline));
}

fn prepare_text_transforms(
    render_context: &render::Context,
    viewport: render::Viewport,
    commit: &Arc<scene::Commit>,
    properties: &scene::Properties,
    batches: &[PlanStep],
    text_renderer: &mut render::text_renderer::TextRenderer,
) -> SyncStats {
    let mut transforms = Vec::new();
    collect_text_transforms(batches, properties, [0.0, 0.0], &mut transforms);
    let report =
        text_renderer.prepare_retained_transforms(render_context, viewport, commit, &transforms);
    SyncStats {
        property_upload_bytes: report.property_upload_bytes,
        resource_creations: report.resource_creations,
        resource_removals: report.resource_removals,
        ..SyncStats::default()
    }
}

fn collect_text_transforms(
    batches: &[PlanStep],
    properties: &scene::Properties,
    translation: [f32; 2],
    transforms: &mut Vec<(render::text_renderer::RetainedBatch, [f32; 2])>,
) {
    for batch in batches {
        match batch {
            PlanStep::Layer(layer) => {
                collect_text_transforms(&layer.render_batches, properties, translation, transforms);
            }
            PlanStep::Text(batch) => {
                transforms.push((*batch, batch.translation(translation)));
            }
            PlanStep::Group(group) => {
                collect_text_transforms(&group.render_batches, properties, [0.0, 0.0], transforms);
            }
            PlanStep::Scroll(scroll) => {
                let current = properties
                    .scroll_offset(scroll.node)
                    .unwrap_or(scroll.baseline);
                let own = [
                    scroll.baseline.x().saturating_sub(current.x()) as f32,
                    scroll.baseline.y().saturating_sub(current.y()) as f32,
                ];
                collect_text_transforms(
                    &scroll.render_batches,
                    properties,
                    [translation[0] + own[0], translation[1] + own[1]],
                    transforms,
                );
            }
            PlanStep::Shapes(_) | PlanStep::Pane(_) | PlanStep::PushClip(_) | PlanStep::PopClip => {
            }
        }
    }
}

impl PendingPlan {
    fn new(commit: Arc<scene::Commit>, viewport: render::Viewport, projection: Projection) -> Self {
        let nodes = commit
            .nodes()
            .iter()
            .map(|node| (node.id(), Arc::clone(node)))
            .collect::<HashMap<_, _>>();
        let mut stats = render::DrawStats::default();
        for node in commit.nodes() {
            match node.opacity() {
                scene::OpacityDeclaration::Opaque => stats.opaque_nodes += 1,
                scene::OpacityDeclaration::Blended | scene::OpacityDeclaration::Variable => {
                    stats.blended_nodes += 1;
                }
            }
        }
        let main = TargetSpace {
            origin: projection.origin,
            size: [
                viewport.logical_area().width().max(1.0),
                viewport.logical_area().height().max(1.0),
            ],
            text_origin: projection.origin,
            text_size: [
                viewport.logical_area().width().max(1.0),
                viewport.logical_area().height().max(1.0),
            ],
            scroll_root: None,
            target: None,
            current_scroll: None,
        };
        let order = commit.order().map(|order| Arc::from(order.to_vec()));
        Self {
            commit,
            viewport,
            projection,
            nodes,
            order,
            order_index: 0,
            frames: vec![PendingFrame {
                kind: PendingFrameKind::Root,
                space: main,
                batches: Vec::new(),
            }],
            rebuilt_nodes: HashSet::new(),
            property_bindings: Vec::new(),
            stats,
        }
    }

    fn advance(
        &mut self,
        builder: &mut PlanBuilder<'_>,
        budget: std::time::Duration,
    ) -> render::Result<bool> {
        let order = self.order.clone();
        let Some(order) = order.as_deref() else {
            let main = self.frames[0].space;
            for node in self
                .commit
                .nodes()
                .iter()
                .filter(|node| node.parent().is_none())
            {
                builder.build_node(&self.commit, node, main, &mut self.frames[0].batches)?;
            }
            return Ok(true);
        };
        let started_at = std::time::Instant::now();
        let mut prepared_content = false;
        while let Some(draw) = order.get(self.order_index) {
            self.order_index = self.order_index.saturating_add(1);
            let space = self
                .frames
                .last()
                .expect("retained preparation must keep a root frame")
                .space;
            match draw {
                scene::Draw::Content {
                    node,
                    index,
                    projection,
                } => {
                    if let Some(owner) = self.nodes.get(node)
                        && let Some(content) = owner.content().get(*index)
                    {
                        let target = &mut self
                            .frames
                            .last_mut()
                            .expect("retained preparation must keep a target frame")
                            .batches;
                        builder.build_content(
                            owner,
                            *index,
                            content,
                            *projection,
                            space,
                            target,
                        )?;
                    }
                    prepared_content = true;
                }
                scene::Draw::PushClip { node, clip } => {
                    builder.stats.scene_items += 1;
                    builder.stats.clip_batches += 1;
                    self.frames
                        .last_mut()
                        .expect("retained preparation must keep a target frame")
                        .batches
                        .push(PlanStep::PushClip(PreparedClip {
                            node: *node,
                            fallback: render::scene::to_paint_clip_value_at_scale(
                                *clip,
                                builder.viewport.scale_factor(),
                            ),
                            scene_origin: space.origin,
                        }));
                }
                scene::Draw::PopClip => {
                    builder.stats.scene_items += 1;
                    builder.stats.clip_batches += 1;
                    self.frames
                        .last_mut()
                        .expect("retained preparation must keep a target frame")
                        .batches
                        .push(PlanStep::PopClip);
                }
                scene::Draw::PushGroup {
                    node,
                    bounds,
                    opacity,
                } => {
                    let mut projected_index = self.order_index;
                    let bounds = builder
                        .project_order_group_bounds(order, &mut projected_index, &self.nodes)
                        .unwrap_or_else(|| {
                            render::scene::to_paint_rect_value_at_scale(
                                *bounds,
                                builder.viewport.scale_factor(),
                            )
                        });
                    self.frames.push(PendingFrame {
                        kind: PendingFrameKind::Group {
                            node: *node,
                            bounds,
                            opacity: *opacity,
                            parent_origin: space.origin,
                        },
                        space: TargetSpace {
                            origin: [bounds.origin.x(), bounds.origin.y()],
                            size: [bounds.area.width().max(1.0), bounds.area.height().max(1.0)],
                            text_origin: [bounds.origin.x(), bounds.origin.y()],
                            text_size: [
                                bounds.area.width().max(1.0),
                                bounds.area.height().max(1.0),
                            ],
                            target: Some(*node),
                            scroll_root: space.current_scroll.or(space.scroll_root),
                            current_scroll: None,
                        },
                        batches: Vec::new(),
                    });
                }
                scene::Draw::PopGroup => self.finish_group(builder),
                scene::Draw::PushScroll { node } => {
                    let Some(declaration) = self.nodes.get(node).and_then(|node| node.scroll())
                    else {
                        continue;
                    };
                    let viewport = render::scene::to_paint_rect_value_at_scale(
                        declaration.viewport(),
                        builder.viewport.scale_factor(),
                    );
                    let resident = render::scene::to_paint_rect_value_at_scale(
                        declaration.resident_bounds(),
                        builder.viewport.scale_factor(),
                    );
                    self.frames.push(PendingFrame {
                        kind: PendingFrameKind::Scroll {
                            node: *node,
                            viewport,
                            baseline: declaration.baseline(),
                            parent_origin: space.origin,
                        },
                        space: TargetSpace {
                            current_scroll: Some(*node),
                            text_origin: [resident.origin.x(), resident.origin.y()],
                            text_size: [
                                resident.area.width().max(1.0),
                                resident.area.height().max(1.0),
                            ],
                            ..space
                        },
                        batches: Vec::new(),
                    });
                }
                scene::Draw::PopScroll => self.finish_scroll(builder),
            }
            if prepared_content && started_at.elapsed() >= budget {
                return Ok(false);
            }
        }
        Ok(self.frames.len() == 1)
    }

    fn finish_group(&mut self, builder: &mut PlanBuilder<'_>) {
        let Some(frame) = (self.frames.len() > 1).then(|| self.frames.pop()).flatten() else {
            return;
        };
        let PendingFrameKind::Group {
            node,
            bounds,
            opacity,
            parent_origin,
        } = frame.kind
        else {
            self.frames.push(frame);
            return;
        };
        builder.stats.scene_items += 1;
        builder.stats.group_composites += 1;
        builder.stats.effect_island_nodes += 1;
        self.frames
            .last_mut()
            .expect("retained group must return to its parent frame")
            .batches
            .push(PlanStep::Group(PreparedGroup {
                node: Some(node),
                bounds: local_group_bounds(bounds, parent_origin),
                opacity,
                render_batches: frame.batches,
            }));
    }

    fn finish_scroll(&mut self, builder: &mut PlanBuilder<'_>) {
        let Some(frame) = (self.frames.len() > 1).then(|| self.frames.pop()).flatten() else {
            return;
        };
        let PendingFrameKind::Scroll {
            node,
            viewport,
            baseline,
            parent_origin,
        } = frame.kind
        else {
            self.frames.push(frame);
            return;
        };
        builder.stats.scene_items += 1;
        self.frames
            .last_mut()
            .expect("retained scroll must return to its parent frame")
            .batches
            .push(PlanStep::Scroll(PreparedScroll {
                node,
                viewport: local_rect(viewport, parent_origin),
                baseline,
                render_batches: frame.batches,
            }));
    }
}

struct PlanBuilder<'a> {
    render_context: &'a render::Context,
    viewport: render::Viewport,
    shapes: &'a mut Shapes,
    text_renderer: &'a mut render::text_renderer::TextRenderer,
    projection: &'a Projection,
    rebuilt_nodes: HashSet<composition::tree::NodeId>,
    property_bindings: Vec<PropertyBinding>,
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
        projection: scene::ContentProjection,
    ) -> PropertyBinding {
        let binding = PropertyBinding {
            node,
            space,
            projection,
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
            origin: self.projection.origin,
            size: [
                self.viewport.logical_area().width().max(1.0),
                self.viewport.logical_area().height().max(1.0),
            ],
            text_origin: self.projection.origin,
            text_size: [
                self.viewport.logical_area().width().max(1.0),
                self.viewport.logical_area().height().max(1.0),
            ],
            scroll_root: None,
            target: None,
            current_scroll: None,
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
        target: &mut Vec<PlanStep>,
    ) -> render::Result<()> {
        while let Some(draw) = order.get(*index) {
            *index = index.saturating_add(1);
            match draw {
                scene::Draw::Content {
                    node,
                    index,
                    projection,
                } => {
                    if let Some(owner) = nodes.get(node)
                        && let Some(content) = owner.content().get(*index)
                    {
                        self.build_content(owner, *index, content, *projection, space, target)?;
                    }
                }
                scene::Draw::PushClip { node, clip } => {
                    self.stats.scene_items += 1;
                    self.stats.clip_batches += 1;
                    target.push(PlanStep::PushClip(PreparedClip {
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
                    target.push(PlanStep::PopClip);
                }
                scene::Draw::PushGroup {
                    node,
                    bounds,
                    opacity,
                } => {
                    let mut paint_index = *index;
                    let bounds = self
                        .project_order_group_bounds(order, &mut paint_index, nodes)
                        .unwrap_or_else(|| {
                            render::scene::to_paint_rect_value_at_scale(
                                *bounds,
                                self.viewport.scale_factor(),
                            )
                        });
                    let group_space = TargetSpace {
                        origin: [bounds.origin.x(), bounds.origin.y()],
                        size: [bounds.area.width().max(1.0), bounds.area.height().max(1.0)],
                        text_origin: [bounds.origin.x(), bounds.origin.y()],
                        text_size: [bounds.area.width().max(1.0), bounds.area.height().max(1.0)],
                        target: Some(*node),
                        scroll_root: space.current_scroll.or(space.scroll_root),
                        current_scroll: None,
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
                    target.push(PlanStep::Group(PreparedGroup {
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
                    let resident = render::scene::to_paint_rect_value_at_scale(
                        declaration.resident_bounds(),
                        self.viewport.scale_factor(),
                    );
                    let mut members = Vec::new();
                    let scroll_space = TargetSpace {
                        current_scroll: Some(*node),
                        text_origin: [resident.origin.x(), resident.origin.y()],
                        text_size: [
                            resident.area.width().max(1.0),
                            resident.area.height().max(1.0),
                        ],
                        ..space
                    };
                    self.build_order(
                        order,
                        index,
                        Some(PlanStop::Scroll),
                        nodes,
                        scroll_space,
                        &mut members,
                    )?;
                    self.stats.scene_items += 1;
                    target.push(PlanStep::Scroll(PreparedScroll {
                        node: *node,
                        viewport: local_rect(viewport, space.origin),
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

    fn project_order_group_bounds(
        &self,
        order: &[scene::Draw],
        index: &mut usize,
        nodes: &HashMap<composition::tree::NodeId, Arc<scene::Node>>,
    ) -> Option<crate::paint::Rect> {
        let mut bounds = None;
        while let Some(draw) = order.get(*index) {
            *index = index.saturating_add(1);
            match draw {
                scene::Draw::Content { node, index, .. } => {
                    let content = nodes.get(node)?.content().get(*index)?;
                    let content =
                        render::scene::prepare_content(content, self.viewport.scale_factor());
                    bounds = Some(bounds.map_or_else(
                        || content.bounds(self.viewport.scale_factor()),
                        |bounds| {
                            crate::paint::union_visual_bounds(
                                bounds,
                                content.bounds(self.viewport.scale_factor()),
                            )
                        },
                    ));
                }
                scene::Draw::PushGroup { .. } => {
                    if let Some(group) = self.project_order_group_bounds(order, index, nodes) {
                        bounds = Some(bounds.map_or(group, |bounds| {
                            crate::paint::union_visual_bounds(bounds, group)
                        }));
                    }
                }
                scene::Draw::PopGroup => break,
                scene::Draw::PushClip { .. }
                | scene::Draw::PopClip
                | scene::Draw::PushScroll { .. }
                | scene::Draw::PopScroll => {}
            }
        }
        bounds.map(|bounds| crate::paint::Grid::new(self.viewport.scale_factor()).snap_rect(bounds))
    }

    fn build_node(
        &mut self,
        commit: &scene::Commit,
        node: &Arc<scene::Node>,
        parent_space: TargetSpace,
        target: &mut Vec<PlanStep>,
    ) -> render::Result<()> {
        if let Some(clip) = node.clip() {
            self.stats.scene_items += 1;
            self.stats.clip_batches += 1;
            target.push(PlanStep::PushClip(PreparedClip {
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
            text_origin: [bounds.origin.x(), bounds.origin.y()],
            text_size: [bounds.area.width().max(1.0), bounds.area.height().max(1.0)],
            target: Some(node.id()),
            scroll_root: parent_space.current_scroll.or(parent_space.scroll_root),
            current_scroll: None,
        });
        let mut body = Vec::new();
        for (index, content) in node.content().iter().enumerate() {
            self.build_content(
                node,
                index,
                content,
                scene::ContentProjection::Normal,
                body_space,
                &mut body,
            )?;
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
                target.push(PlanStep::Group(PreparedGroup {
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
            target.push(PlanStep::PopClip);
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
        self.group_content_bounds(commit, node)
            .map(|bounds| crate::paint::Grid::new(self.viewport.scale_factor()).snap_rect(bounds))
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

    fn group_content_bounds(
        &self,
        commit: &scene::Commit,
        node: &scene::Node,
    ) -> Option<crate::paint::Rect> {
        let mut bounds = node
            .content()
            .iter()
            .map(|content| {
                render::scene::prepare_content(content, self.viewport.scale_factor())
                    .bounds(self.viewport.scale_factor())
            })
            .reduce(crate::paint::union_visual_bounds);
        for child in commit
            .nodes()
            .iter()
            .filter(|child| child.parent() == Some(node.id()))
        {
            if let Some(child) = self.group_content_bounds(commit, child) {
                bounds = Some(bounds.map_or(child, |bounds| {
                    crate::paint::union_visual_bounds(bounds, child)
                }));
            }
        }
        bounds
    }

    fn build_content(
        &mut self,
        node: &Arc<scene::Node>,
        content_index: usize,
        content: &scene::Content,
        projection: scene::ContentProjection,
        space: TargetSpace,
        target: &mut Vec<PlanStep>,
    ) -> render::Result<()> {
        self.stats.scene_items += 1;
        let Some((content, realization)) = self.projection.material.projected_content(content)
        else {
            return Ok(());
        };
        let item = render::scene::prepare_content(&content, self.viewport.scale_factor());
        match &item {
            render::scene::PreparedContent::Quad(value) => {
                let source_rect = match &content {
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
                    realization,
                    content::Shape::Quad(value),
                    source_rect,
                    projection,
                    space,
                    target,
                )
            }
            render::scene::PreparedContent::Rule(value) => self.push_shape(
                node,
                content_index,
                0,
                realization,
                content::Shape::Rule(value),
                None,
                scene::ContentProjection::Normal,
                space,
                target,
            ),
            render::scene::PreparedContent::Shadow(value) => self.push_shape(
                node,
                content_index,
                0,
                realization,
                content::Shape::Shadow(value),
                None,
                scene::ContentProjection::Normal,
                space,
                target,
            ),
            render::scene::PreparedContent::Outline(value) => self.push_shape(
                node,
                content_index,
                0,
                realization,
                content::Shape::Outline(value),
                None,
                scene::ContentProjection::Normal,
                space,
                target,
            ),
            render::scene::PreparedContent::Text(value) => self.push_glyph(
                node,
                content_index,
                content::Glyph::Text(value),
                space,
                target,
            )?,
            render::scene::PreparedContent::TextViewport(value) => {
                self.stats.text_surfaces += value.surfaces.len();
                self.push_glyph(
                    node,
                    content_index,
                    content::Glyph::TextViewport(value),
                    space,
                    target,
                )?;
            }
            render::scene::PreparedContent::Icon(value) => self.push_glyph(
                node,
                content_index,
                content::Glyph::Icon(value),
                space,
                target,
            )?,
            render::scene::PreparedContent::Pane(value) => {
                self.push_pane(node, content_index, realization, value, space, target);
            }
        }
        Ok(())
    }

    fn push_shape(
        &mut self,
        node: &Arc<scene::Node>,
        content_index: usize,
        part: u16,
        realization: u8,
        shape: content::Shape<'_>,
        source_rect: Option<[f32; 4]>,
        projection: scene::ContentProjection,
        space: TargetSpace,
        target: &mut Vec<PlanStep>,
    ) {
        let binding = self.property_binding(node.id(), space, projection);
        let (prepared, sync) = self.shapes.prepare(
            self.render_context,
            self.viewport,
            node,
            content_index,
            part,
            realization,
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
            target.push(PlanStep::Shapes(prepared));
        }
    }

    fn push_glyph(
        &mut self,
        node: &Arc<scene::Node>,
        content_index: usize,
        glyph: content::Glyph<'_>,
        space: TargetSpace,
        target: &mut Vec<PlanStep>,
    ) -> render::Result<()> {
        self.stats.glyph_batches += 1;
        let report = self.text_renderer.prepare_retained(
            self.render_context,
            self.viewport,
            node,
            content_index,
            &[glyph],
            space.text_origin,
            space.text_size,
            space.size,
            [
                space.text_origin[0] - space.origin[0],
                space.text_origin[1] - space.origin[1],
            ],
            space.current_scroll,
            space.target,
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
            target.push(PlanStep::Text(batch));
        }
        Ok(())
    }

    fn push_pane(
        &mut self,
        node: &Arc<scene::Node>,
        content_index: usize,
        realization: u8,
        pane: &crate::paint::Pane,
        space: TargetSpace,
        target: &mut Vec<PlanStep>,
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
        let base = base_brush.and_then(|brush| {
            self.prepare_brush(node, content_index, 0, realization, pane.rect, brush, space)
        });
        let surface_layers = surface_brushes
            .into_iter()
            .enumerate()
            .map(|(index, brush)| {
                brush.and_then(|brush| {
                    self.prepare_brush(
                        node,
                        content_index,
                        u16::try_from(index.saturating_add(1)).unwrap_or(u16::MAX),
                        realization,
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
                target.push(PlanStep::Shapes(base));
            }
        } else {
            self.stats.effect_island_nodes += 1;
            target.push(PlanStep::Pane(prepared));
        }
    }

    fn prepare_brush(
        &mut self,
        node: &Arc<scene::Node>,
        content_index: usize,
        part: u16,
        realization: u8,
        rect: crate::paint::Rect,
        brush: crate::paint::Brush,
        space: TargetSpace,
    ) -> Option<ShapeBatch> {
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
        let binding = self.property_binding(node.id(), space, scene::ContentProjection::Normal);
        let (prepared, sync) = self.shapes.prepare(
            self.render_context,
            self.viewport,
            node,
            content_index,
            part,
            realization,
            &[content::Shape::Quad(&quad)],
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
        prepared
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

fn count_batches(batches: &[PlanStep]) -> usize {
    batches
        .iter()
        .map(|batch| match batch {
            PlanStep::Group(group) => 1 + count_batches(&group.render_batches),
            PlanStep::Scroll(scroll) => 1 + count_batches(&scroll.render_batches),
            _ => 1,
        })
        .sum()
}

fn coalesce_shape_batches(batches: &mut Vec<PlanStep>) {
    let mut coalesced = Vec::with_capacity(batches.len());
    for mut batch in batches.drain(..) {
        if let PlanStep::Group(group) = &mut batch {
            coalesce_shape_batches(&mut group.render_batches);
        } else if let PlanStep::Scroll(scroll) = &mut batch {
            coalesce_shape_batches(&mut scroll.render_batches);
        }
        let merged = match (coalesced.last_mut(), &batch) {
            (Some(PlanStep::Shapes(previous)), PlanStep::Shapes(next)) => {
                previous.merge_adjacent(next)
            }
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

pub(in crate::render) struct PropertyBindings {
    offsets: HashMap<PropertyBinding, u32>,
    slot: usize,
    scroll_offsets: HashMap<ScrollBinding, u32>,
    scroll_slot: usize,
}

impl PropertyBindings {
    fn offset(&self, binding: PropertyBinding) -> u32 {
        self.offsets.get(&binding).copied().unwrap_or_default()
    }

    fn scroll_offset(&self, binding: PropertyBinding) -> u32 {
        self.scroll_offsets
            .get(&binding.scroll())
            .copied()
            .unwrap_or_default()
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
    property_stride: usize,
    bind_group_layout: wgpu::BindGroupLayout,
    property_slots: Vec<PropertySlot>,
    scroll_property_stride: usize,
    scroll_bind_group_layout: wgpu::BindGroupLayout,
    scroll_property_slots: Vec<ScrollPropertySlot>,
}

struct PropertySlot {
    owners: Vec<Weak<scene::Commit>>,
    viewport_key: [u32; 3],
    viewport_buffer: wgpu::Buffer,
    property_buffer: wgpu::Buffer,
    property_capacity: usize,
    bind_group: wgpu::BindGroup,
    bindings: Vec<PropertyBinding>,
    bytes: Vec<u8>,
}

struct ScrollPropertySlot {
    owners: Vec<Weak<scene::Commit>>,
    buffer: wgpu::Buffer,
    capacity: usize,
    bind_group: wgpu::BindGroup,
    bindings: Vec<ScrollBinding>,
    bytes: Vec<u8>,
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
        let scroll_property_stride = align_up(
            std::mem::size_of::<ScrollProperty>(),
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
                    visibility: wgpu::ShaderStages::VERTEX | wgpu::ShaderStages::FRAGMENT,
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
        let scroll_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("Retained Scroll Property Bind Group Layout"),
                entries: &[wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::VERTEX,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: true,
                        min_binding_size: NonZeroU64::new(
                            std::mem::size_of::<ScrollProperty>() as u64
                        ),
                    },
                    count: None,
                }],
            });
        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Retained Shape Pipeline Layout"),
            bind_group_layouts: &[Some(&bind_group_layout), Some(&scroll_bind_group_layout)],
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
        let property_slots = (0..RETAINED_PROPERTY_SLOT_RESERVE)
            .map(|_| {
                create_property_slot(
                    device,
                    &bind_group_layout,
                    property_stride,
                    INITIAL_PROPERTY_CAPACITY,
                )
            })
            .collect();
        let scroll_property_slots = (0..RETAINED_SCROLL_PROPERTY_SLOT_RESERVE)
            .map(|_| {
                create_scroll_property_slot(
                    device,
                    &scroll_bind_group_layout,
                    scroll_property_stride,
                    INITIAL_SCROLL_PROPERTY_CAPACITY,
                )
            })
            .collect();

        Self {
            pipeline,
            unit_buffer,
            instance_buffer,
            instance_capacity: INITIAL_INSTANCE_CAPACITY,
            instances: Vec::new(),
            free: Vec::new(),
            entries: HashMap::new(),
            property_stride,
            bind_group_layout,
            property_slots,
            scroll_property_stride,
            scroll_bind_group_layout,
            scroll_property_slots,
        }
    }

    fn prepare(
        &mut self,
        render_context: &render::Context,
        viewport: render::Viewport,
        node: &Arc<scene::Node>,
        content_index: usize,
        part: u16,
        realization: u8,
        shapes: &[content::Shape<'_>],
        source_rect: Option<[f32; 4]>,
        binding: PropertyBinding,
    ) -> (Option<ShapeBatch>, SyncStats) {
        let mut stats = self.prune();
        let key = ResourceKey::new(node, content_index, part, viewport.scale_factor())
            .with_realization(realization);
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
                    slot: 0,
                    scroll_offsets: HashMap::new(),
                    scroll_slot: 0,
                },
                stats,
            );
        }

        let (offsets, slot) = self.prepare_node_properties(
            render_context,
            viewport,
            commit,
            properties,
            bindings,
            &mut stats,
        );
        let (scroll_offsets, scroll_slot) = self.prepare_scroll_properties(
            render_context,
            commit,
            properties,
            bindings,
            &mut stats,
        );

        (
            PropertyBindings {
                offsets,
                slot,
                scroll_offsets,
                scroll_slot,
            },
            stats,
        )
    }

    fn prepare_node_properties(
        &mut self,
        render_context: &render::Context,
        viewport: render::Viewport,
        commit: &Arc<scene::Commit>,
        properties: &scene::Properties,
        bindings: &[PropertyBinding],
        stats: &mut SyncStats,
    ) -> (HashMap<PropertyBinding, u32>, usize) {
        let required = bindings.len().max(1);
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
        let mut bytes = vec![0_u8; required * self.property_stride];

        for (index, binding) in bindings.iter().copied().enumerate() {
            let Some(node) = commit.nodes().iter().find(|node| node.id() == binding.node) else {
                continue;
            };

            let mut property = NodeProperty::IDENTITY;
            property.grid[0] = viewport.scale_factor();
            property.scene_origin = binding.space.origin;
            property.target_size = binding.space.size;
            match binding.projection {
                scene::ContentProjection::Normal => {
                    if let Some(scene::PropertyValue::Transform { value, .. }) = properties.value(
                        scene::PropertyRef::new(node.id(), scene::PropertyKind::Transform),
                    ) {
                        property.origin = [value.origin_x(), value.origin_y()];
                        property.translate = [value.translate_x(), value.translate_y()];
                        property.scale = [value.scale_x(), value.scale_y()];
                        property.grid[1] = 1.0;
                    }
                }
                scene::ContentProjection::Caret => {
                    if let Some(scene::PropertyValue::Caret { visible, .. }) = properties.value(
                        scene::PropertyRef::new(node.id(), scene::PropertyKind::Caret),
                    ) {
                        property.opacity = f32::from(visible);
                    }
                }
                projection => project_scrollbar_property(
                    &mut property,
                    projection,
                    node.id(),
                    properties,
                    viewport.scale_factor(),
                ),
            }
            let offset = index * self.property_stride;
            bytes[offset..offset + std::mem::size_of::<NodeProperty>()]
                .copy_from_slice(bytemuck::bytes_of(&property));
        }

        for slot in &mut self.property_slots {
            slot.owners.retain(|owner| owner.strong_count() > 0);
        }
        if let Some(slot) = self.property_slots.iter_mut().position(|slot| {
            slot.viewport_key == viewport_key && slot.bindings == bindings && slot.bytes == bytes
        }) {
            add_property_owner(&mut self.property_slots[slot].owners, commit);
            return (offsets, slot);
        }

        let slot = self
            .property_slots
            .iter()
            .position(|slot| {
                let mut owns_commit = false;
                let mut owns_other = false;
                for owner in slot.owners.iter().filter_map(Weak::upgrade) {
                    if Arc::ptr_eq(&owner, commit) {
                        owns_commit = true;
                    } else {
                        owns_other = true;
                    }
                }
                owns_commit && !owns_other
            })
            .or_else(|| {
                self.property_slots
                    .iter()
                    .position(|slot| slot.owners.is_empty())
            })
            .unwrap_or_else(|| {
                self.property_slots.push(create_property_slot(
                    render_context.device(),
                    &self.bind_group_layout,
                    self.property_stride,
                    INITIAL_PROPERTY_CAPACITY,
                ));
                stats.resource_creations += 2;
                self.property_slots.len() - 1
            });

        let property_slot = &mut self.property_slots[slot];
        let viewport_changed = property_slot.viewport_key != viewport_key;
        let mut buffer_recreated = false;
        if required > property_slot.property_capacity {
            property_slot.property_capacity = required.next_power_of_two();
            property_slot.property_buffer = create_property_buffer(
                render_context.device(),
                self.property_stride,
                property_slot.property_capacity,
            );
            property_slot.bind_group = create_bind_group(
                render_context.device(),
                &self.bind_group_layout,
                &property_slot.viewport_buffer,
                &property_slot.property_buffer,
            );
            stats.resource_creations += 1;
            buffer_recreated = true;
        }

        if viewport_changed {
            let viewport_uniform = ViewportUniform {
                size: [
                    viewport.logical_area().width().max(1.0),
                    viewport.logical_area().height().max(1.0),
                ],
                padding: [0.0, 0.0],
            };
            render_context.queue().write_buffer(
                &property_slot.viewport_buffer,
                0,
                bytemuck::bytes_of(&viewport_uniform),
            );
            stats.property_upload_bytes += std::mem::size_of::<ViewportUniform>();
        }

        if buffer_recreated || property_slot.bytes != bytes {
            render_context
                .queue()
                .write_buffer(&property_slot.property_buffer, 0, &bytes);
            stats.property_upload_bytes += bytes.len();
        }
        property_slot.owners.clear();
        property_slot.owners.push(Arc::downgrade(commit));
        property_slot.viewport_key = viewport_key;
        property_slot.bindings = bindings.to_vec();
        property_slot.bytes = bytes;

        (offsets, slot)
    }

    fn prepare_scroll_properties(
        &mut self,
        render_context: &render::Context,
        commit: &Arc<scene::Commit>,
        properties: &scene::Properties,
        bindings: &[PropertyBinding],
        stats: &mut SyncStats,
    ) -> (HashMap<ScrollBinding, u32>, usize) {
        let mut scroll_bindings = Vec::new();
        for binding in bindings.iter().map(|binding| binding.scroll()) {
            if !scroll_bindings.contains(&binding) {
                scroll_bindings.push(binding);
            }
        }
        if scroll_bindings.is_empty() {
            scroll_bindings.push(ScrollBinding::IDENTITY);
        }

        let offsets = scroll_bindings
            .iter()
            .copied()
            .enumerate()
            .map(|(index, binding)| (binding, (index * self.scroll_property_stride) as u32))
            .collect::<HashMap<_, _>>();
        let required = scroll_bindings.len().max(1);
        let mut bytes = vec![0_u8; required * self.scroll_property_stride];
        let mut inherited_scroll = HashMap::with_capacity(commit.nodes().len());
        for node in commit.nodes() {
            let parent_scroll = node
                .parent()
                .and_then(|parent| inherited_scroll.get(&parent).copied())
                .unwrap_or([0.0_f32; 2]);
            let own_scroll = node.scroll().map_or([0.0, 0.0], |declaration| {
                let current = properties
                    .scroll_offset(node.id())
                    .unwrap_or(declaration.baseline());
                [
                    declaration.baseline().x().saturating_sub(current.x()) as f32,
                    declaration.baseline().y().saturating_sub(current.y()) as f32,
                ]
            });
            inherited_scroll.insert(
                node.id(),
                [
                    parent_scroll[0] + own_scroll[0],
                    parent_scroll[1] + own_scroll[1],
                ],
            );
        }

        for (index, binding) in scroll_bindings.iter().copied().enumerate() {
            let inherited = binding
                .node
                .and_then(|node| inherited_scroll.get(&node).copied())
                .unwrap_or_default();
            let root = binding
                .root
                .and_then(|node| inherited_scroll.get(&node).copied())
                .unwrap_or_default();
            let property = ScrollProperty {
                translation: [inherited[0] - root[0], inherited[1] - root[1]],
                ..ScrollProperty::IDENTITY
            };
            let offset = index * self.scroll_property_stride;
            bytes[offset..offset + std::mem::size_of::<ScrollProperty>()]
                .copy_from_slice(bytemuck::bytes_of(&property));
        }

        for slot in &mut self.scroll_property_slots {
            slot.owners.retain(|owner| owner.strong_count() > 0);
        }
        if let Some(slot) = self
            .scroll_property_slots
            .iter_mut()
            .position(|slot| slot.bindings == scroll_bindings && slot.bytes == bytes)
        {
            add_property_owner(&mut self.scroll_property_slots[slot].owners, commit);
            return (offsets, slot);
        }

        let slot = self
            .scroll_property_slots
            .iter()
            .position(|slot| {
                let mut owns_commit = false;
                let mut owns_other = false;
                for owner in slot.owners.iter().filter_map(Weak::upgrade) {
                    if Arc::ptr_eq(&owner, commit) {
                        owns_commit = true;
                    } else {
                        owns_other = true;
                    }
                }
                owns_commit && !owns_other
            })
            .or_else(|| {
                self.scroll_property_slots
                    .iter()
                    .position(|slot| slot.owners.is_empty())
            })
            .unwrap_or_else(|| {
                self.scroll_property_slots.push(create_scroll_property_slot(
                    render_context.device(),
                    &self.scroll_bind_group_layout,
                    self.scroll_property_stride,
                    INITIAL_SCROLL_PROPERTY_CAPACITY,
                ));
                stats.resource_creations += 1;
                self.scroll_property_slots.len() - 1
            });

        let scroll_slot = &mut self.scroll_property_slots[slot];
        let mut buffer_recreated = false;
        if required > scroll_slot.capacity {
            scroll_slot.capacity = required.next_power_of_two();
            scroll_slot.buffer = create_scroll_property_buffer(
                render_context.device(),
                self.scroll_property_stride,
                scroll_slot.capacity,
            );
            scroll_slot.bind_group = create_scroll_bind_group(
                render_context.device(),
                &self.scroll_bind_group_layout,
                &scroll_slot.buffer,
            );
            stats.resource_creations += 1;
            buffer_recreated = true;
        }
        if buffer_recreated || scroll_slot.bytes.len() != bytes.len() {
            render_context
                .queue()
                .write_buffer(&scroll_slot.buffer, 0, &bytes);
            stats.property_upload_bytes += bytes.len();
        } else {
            let property_size = std::mem::size_of::<ScrollProperty>();
            for index in 0..required {
                let offset = index * self.scroll_property_stride;
                let range = offset..offset + property_size;
                if scroll_slot.bytes[range.clone()] != bytes[range.clone()] {
                    render_context.queue().write_buffer(
                        &scroll_slot.buffer,
                        offset as u64,
                        &bytes[range],
                    );
                    stats.property_upload_bytes += property_size;
                }
            }
        }
        scroll_slot.owners.clear();
        scroll_slot.owners.push(Arc::downgrade(commit));
        scroll_slot.bindings = scroll_bindings;
        scroll_slot.bytes = bytes;

        (offsets, slot)
    }

    pub(in crate::render) fn draw<'a>(
        &'a self,
        pass: &mut wgpu::RenderPass<'a>,
        batch: &ShapeBatch,
        properties: &PropertyBindings,
    ) {
        pass.set_pipeline(&self.pipeline);
        pass.set_bind_group(
            0,
            &self.property_slots[properties.slot].bind_group,
            &[properties.offset(batch.binding())],
        );
        pass.set_bind_group(
            1,
            &self.scroll_property_slots[properties.scroll_slot].bind_group,
            &[properties.scroll_offset(batch.binding())],
        );
        pass.set_vertex_buffer(0, self.unit_buffer.slice(..));
        pass.set_vertex_buffer(1, self.instance_buffer.slice(..));
        pass.draw(0..UNIT_QUAD.len() as u32, batch.range());
    }

    pub(in crate::render) fn resource_count(&self) -> usize {
        self.entries
            .len()
            .saturating_add(2)
            .saturating_add(self.property_slots.len().saturating_mul(2))
            .saturating_add(self.scroll_property_slots.len())
    }

    pub(in crate::render) fn collect(&mut self) -> SyncStats {
        self.prune()
    }

    fn cancel_property_state(&mut self, commit: &Arc<scene::Commit>) {
        for slot in &mut self.property_slots {
            remove_property_owner(&mut slot.owners, commit);
        }
        for slot in &mut self.scroll_property_slots {
            remove_property_owner(&mut slot.owners, commit);
        }
    }

    fn retain_property_viewport(
        &mut self,
        commit: &Arc<scene::Commit>,
        viewport: render::Viewport,
    ) {
        let viewport_key = [
            viewport.logical_area().width().to_bits(),
            viewport.logical_area().height().to_bits(),
            viewport.scale_factor().to_bits(),
        ];
        for slot in &mut self.property_slots {
            if slot.viewport_key != viewport_key {
                remove_property_owner(&mut slot.owners, commit);
            }
        }
    }

    pub(in crate::render) fn resource_bytes(&self) -> usize {
        UNIT_QUAD
            .len()
            .saturating_mul(std::mem::size_of::<UnitVertex>())
            .saturating_add(
                self.instance_capacity
                    .saturating_mul(std::mem::size_of::<render::quad::Instance>()),
            )
            .saturating_add(self.property_slots.iter().fold(0_usize, |bytes, slot| {
                bytes
                    .saturating_add(std::mem::size_of::<ViewportUniform>())
                    .saturating_add(slot.property_capacity.saturating_mul(self.property_stride))
            }))
            .saturating_add(
                self.scroll_property_slots
                    .iter()
                    .fold(0_usize, |bytes, slot| {
                        bytes.saturating_add(
                            slot.capacity.saturating_mul(self.scroll_property_stride),
                        )
                    }),
            )
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
        let property_slots_before = self.property_slots.len();
        let mut kept_recycle = 0;
        self.property_slots.retain_mut(|slot| {
            slot.owners.retain(|owner| owner.strong_count() > 0);
            if !slot.owners.is_empty() {
                true
            } else if kept_recycle < RETAINED_PROPERTY_SLOT_RESERVE {
                kept_recycle += 1;
                true
            } else {
                false
            }
        });
        let scroll_property_slots_before = self.scroll_property_slots.len();
        let mut kept_scroll_recycle = 0;
        self.scroll_property_slots.retain_mut(|slot| {
            slot.owners.retain(|owner| owner.strong_count() > 0);
            if !slot.owners.is_empty() {
                true
            } else if kept_scroll_recycle < RETAINED_SCROLL_PROPERTY_SLOT_RESERVE {
                kept_scroll_recycle += 1;
                true
            } else {
                false
            }
        });
        SyncStats {
            resource_removals: expired
                .len()
                .saturating_add(
                    property_slots_before
                        .saturating_sub(self.property_slots.len())
                        .saturating_mul(2),
                )
                .saturating_add(
                    scroll_property_slots_before.saturating_sub(self.scroll_property_slots.len()),
                ),
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

fn create_scroll_property_buffer(
    device: &wgpu::Device,
    stride: usize,
    capacity: usize,
) -> wgpu::Buffer {
    device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("Retained Scroll Properties"),
        size: stride.saturating_mul(capacity.max(1)) as u64,
        usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        mapped_at_creation: false,
    })
}

fn create_property_slot(
    device: &wgpu::Device,
    layout: &wgpu::BindGroupLayout,
    stride: usize,
    capacity: usize,
) -> PropertySlot {
    let viewport_buffer = device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("Retained Viewport Uniform"),
        size: std::mem::size_of::<ViewportUniform>() as u64,
        usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        mapped_at_creation: false,
    });
    let property_buffer = create_property_buffer(device, stride, capacity);
    let bind_group = create_bind_group(device, layout, &viewport_buffer, &property_buffer);
    PropertySlot {
        owners: Vec::new(),
        viewport_key: [0; 3],
        viewport_buffer,
        property_buffer,
        property_capacity: capacity,
        bind_group,
        bindings: Vec::new(),
        bytes: Vec::new(),
    }
}

fn create_scroll_property_slot(
    device: &wgpu::Device,
    layout: &wgpu::BindGroupLayout,
    stride: usize,
    capacity: usize,
) -> ScrollPropertySlot {
    let buffer = create_scroll_property_buffer(device, stride, capacity);
    let bind_group = create_scroll_bind_group(device, layout, &buffer);
    ScrollPropertySlot {
        owners: Vec::new(),
        buffer,
        capacity,
        bind_group,
        bindings: Vec::new(),
        bytes: Vec::new(),
    }
}

fn add_property_owner(owners: &mut Vec<Weak<scene::Commit>>, commit: &Arc<scene::Commit>) {
    owners.retain(|owner| owner.strong_count() > 0);
    if !owners
        .iter()
        .filter_map(Weak::upgrade)
        .any(|owner| Arc::ptr_eq(&owner, commit))
    {
        owners.push(Arc::downgrade(commit));
    }
}

fn remove_property_owner(owners: &mut Vec<Weak<scene::Commit>>, commit: &Arc<scene::Commit>) {
    owners.retain(|owner| {
        owner
            .upgrade()
            .is_some_and(|owner| !Arc::ptr_eq(&owner, commit))
    });
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

fn create_scroll_bind_group(
    device: &wgpu::Device,
    layout: &wgpu::BindGroupLayout,
    properties: &wgpu::Buffer,
) -> wgpu::BindGroup {
    device.create_bind_group(&wgpu::BindGroupDescriptor {
        label: Some("Retained Scroll Property Bind Group"),
        layout,
        entries: &[wgpu::BindGroupEntry {
            binding: 0,
            resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding {
                buffer: properties,
                offset: 0,
                size: NonZeroU64::new(std::mem::size_of::<ScrollProperty>() as u64),
            }),
        }],
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
                text_origin: [0.0, 0.0],
                text_size: [100.0, 80.0],
                target: None,
                scroll_root: None,
                current_scroll: None,
            },
            projection: scene::ContentProjection::Normal,
        }
    }

    fn shapes(range: Range<u32>, binding: PropertyBinding) -> PlanStep {
        PlanStep::Shapes(ShapeBatch { range, binding })
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
        let PlanStep::Shapes(merged) = &batches[0] else {
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
            PlanStep::PopClip,
            shapes(2..4, binding),
        ];

        coalesce_shape_batches(&mut batches);

        assert_eq!(batches.len(), 3);
    }
}
