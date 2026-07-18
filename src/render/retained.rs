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
const RETAINED_SHAPE_RECYCLE_RESERVE: usize = 128;
const MAX_TEXT_AREAS_PER_PREPARED_BATCH: usize = 128;

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
    value: Option<scene::PropertyValue>,
    scroll_offset: Option<crate::interaction::Offset>,
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
    let (opacity, thickness) = match value {
        Some(scene::PropertyValue::Scrollbar {
            opacity, thickness, ..
        }) => (opacity, thickness),
        _ => (0.0, base_thickness as f32),
    };
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
        && let Some(offset) = scroll_offset
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

fn node_property_for_binding(
    commit: &scene::Commit,
    properties: &scene::Properties,
    binding: PropertyBinding,
    scale_factor: f32,
    stats: &mut SyncStats,
) -> NodeProperty {
    let mut value = |kind| {
        stats.property_index_lookups = stats.property_index_lookups.saturating_add(1);
        let value = commit
            .property_index(scene::PropertyRef::new(binding.node, kind))
            .and_then(|index| properties.value_at(index));
        stats.property_value_visits = stats
            .property_value_visits
            .saturating_add(usize::from(value.is_some()));
        value
    };

    let mut property = NodeProperty::IDENTITY;
    property.grid[0] = scale_factor;
    property.scene_origin = binding.space.origin;
    property.target_size = binding.space.size;
    match binding.projection {
        scene::ContentProjection::Normal => {
            if let Some(scene::PropertyValue::Transform { value, .. }) =
                value(scene::PropertyKind::Transform)
            {
                property.origin = [value.origin_x(), value.origin_y()];
                property.translate = [value.translate_x(), value.translate_y()];
                property.scale = [value.scale_x(), value.scale_y()];
                property.grid[1] = 1.0;
            }
        }
        scene::ContentProjection::Caret => {
            if let Some(scene::PropertyValue::Caret { visible, .. }) =
                value(scene::PropertyKind::Caret)
            {
                property.opacity = f32::from(visible);
            }
        }
        projection => {
            let visual = projection
                .scrollbar_axis()
                .and_then(|axis| value(scene::PropertyKind::scrollbar(axis)));
            let scroll_offset =
                matches!(projection, scene::ContentProjection::ScrollbarThumb { .. })
                    .then(|| value(scene::PropertyKind::Offset))
                    .flatten()
                    .and_then(|value| match value {
                        scene::PropertyValue::Offset { value, .. } => Some(value),
                        _ => None,
                    });
            project_scrollbar_property(
                &mut property,
                projection,
                visual,
                scroll_offset,
                scale_factor,
            );
        }
    }
    property
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
    pub(in crate::render) viewport_property_upload_bytes: usize,
    pub(in crate::render) node_property_upload_bytes: usize,
    pub(in crate::render) scroll_property_upload_bytes: usize,
    pub(in crate::render) text_property_upload_bytes: usize,
    pub(in crate::render) property_value_visits: usize,
    pub(in crate::render) property_index_lookups: usize,
    pub(in crate::render) property_dirty_indices: usize,
    pub(in crate::render) property_write_ranges: usize,
    pub(in crate::render) property_full_initializations: usize,
    pub(in crate::render) property_full_buffer_replacements: usize,
    pub(in crate::render) property_full_topology_replacements: usize,
    pub(in crate::render) property_full_dense_transfers: usize,
    pub(in crate::render) property_full_generation_resyncs: usize,
    pub(in crate::render) resource_creations: usize,
    pub(in crate::render) resource_replacements: usize,
    pub(in crate::render) resource_removals: usize,
}

pub(in crate::render) struct Plan {
    batches: Vec<PlanStep>,
    property_bindings: Vec<PropertyBinding>,
    property_offsets: Arc<HashMap<PropertyBinding, u32>>,
    property_dependents: Arc<HashMap<scene::PropertyIndex, Vec<usize>>>,
    scroll_bindings: Arc<[ScrollBinding]>,
    scroll_offsets: Arc<HashMap<ScrollBinding, u32>>,
    scroll_dependents: Arc<HashMap<scene::PropertyIndex, Vec<usize>>>,
    spatial_bindings: Vec<scene::SpatialBinding>,
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

    #[cfg(feature = "renderer-debug")]
    fn debug_signature(&self) -> String {
        let mut property_dependents = self
            .property_dependents
            .iter()
            .map(|(property, dependents)| (*property, dependents.as_slice()))
            .collect::<Vec<_>>();
        property_dependents.sort_unstable_by_key(|(property, _)| *property);
        let mut scroll_dependents = self
            .scroll_dependents
            .iter()
            .map(|(property, dependents)| (*property, dependents.as_slice()))
            .collect::<Vec<_>>();
        scroll_dependents.sort_unstable_by_key(|(property, _)| *property);
        format!(
            "batches={:?};property_bindings={:?};property_dependents={:?};scroll_bindings={:?};scroll_dependents={:?};spatial_bindings={:?};requires_surface_sampling={}",
            self.batches,
            self.property_bindings,
            property_dependents,
            self.scroll_bindings,
            scroll_dependents,
            self.spatial_bindings,
            self.requires_surface_sampling
        )
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
    spatial: scene::SpatialBinding,
}

impl TargetSpace {
    fn with_spatial(mut self, state: scene::SpatialPropertyState) -> Self {
        self.spatial = state.spatial();
        self
    }
}

impl PartialEq for TargetSpace {
    fn eq(&self, other: &Self) -> bool {
        self.origin.map(f32::to_bits) == other.origin.map(f32::to_bits)
            && self.size.map(f32::to_bits) == other.size.map(f32::to_bits)
            && self.text_origin.map(f32::to_bits) == other.text_origin.map(f32::to_bits)
            && self.text_size.map(f32::to_bits) == other.text_size.map(f32::to_bits)
            && self.spatial == other.spatial
    }
}

impl Eq for TargetSpace {}

impl std::hash::Hash for TargetSpace {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.origin.map(f32::to_bits).hash(state);
        self.size.map(f32::to_bits).hash(state);
        self.text_origin.map(f32::to_bits).hash(state);
        self.text_size.map(f32::to_bits).hash(state);
        self.spatial.hash(state);
    }
}

fn text_batch_spaces_compatible(left: TargetSpace, right: TargetSpace) -> bool {
    left.origin.map(f32::to_bits) == right.origin.map(f32::to_bits)
        && left.size.map(f32::to_bits) == right.size.map(f32::to_bits)
        && left.spatial == right.spatial
}

fn union_text_target(mut left: TargetSpace, right: TargetSpace) -> TargetSpace {
    debug_assert!(text_batch_spaces_compatible(left, right));
    let left_right = left.text_origin[0] + left.text_size[0].max(0.0);
    let left_bottom = left.text_origin[1] + left.text_size[1].max(0.0);
    let right_right = right.text_origin[0] + right.text_size[0].max(0.0);
    let right_bottom = right.text_origin[1] + right.text_size[1].max(0.0);
    let origin = [
        left.text_origin[0].min(right.text_origin[0]),
        left.text_origin[1].min(right.text_origin[1]),
    ];
    left.text_origin = origin;
    left.text_size = [
        (left_right.max(right_right) - origin[0]).max(1.0),
        (left_bottom.max(right_bottom) - origin[1]).max(1.0),
    ];
    left
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
struct ScrollBinding {
    path: scene::ScrollPathId,
}

impl ScrollBinding {
    const IDENTITY: Self = Self {
        path: scene::ScrollPathId::ROOT,
    };
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
struct PropertyBinding {
    node: composition::tree::NodeId,
    space: TargetSpace,
    projection: scene::ContentProjection,
    scroll: ScrollBinding,
}

impl PropertyBinding {
    fn scroll(self) -> ScrollBinding {
        self.scroll
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
    pending_candidates: Vec<PendingCandidate>,
    prepared_stats: Vec<PreparedStats>,
}

struct PreparedStats {
    commit: Weak<scene::Commit>,
    viewport: render::Viewport,
    projection: Projection,
    stats: render::DrawStats,
}

struct PendingCandidate {
    commit: Arc<scene::Commit>,
    viewport: render::Viewport,
    projection: Projection,
    properties: scene::PropertySerial,
    transforms: Vec<(render::text_renderer::RetainedBatch, [f32; 2])>,
    next_transform: usize,
}

struct PendingPlan {
    commit: Arc<scene::Commit>,
    viewport: render::Viewport,
    projection: Projection,
    nodes: HashMap<composition::tree::NodeId, Arc<scene::Node>>,
    order: Option<Arc<[OrderedDraw]>>,
    order_index: usize,
    frames: Vec<PendingFrame>,
    rebuilt_nodes: HashSet<composition::tree::NodeId>,
    property_bindings: Vec<PropertyBinding>,
    stats: render::DrawStats,
}

#[derive(Clone)]
struct OrderedDraw {
    original_index: usize,
    draw: scene::Draw,
}

#[derive(Clone, Copy)]
struct PendingTextBounds {
    bounds: crate::paint::Rect,
    spatial: scene::SpatialBinding,
}

struct PendingGlyph {
    node: Arc<scene::Node>,
    content_index: usize,
    content: scene::Content,
}

struct PreparedGlyph {
    node: Arc<scene::Node>,
    content_index: usize,
    content: render::scene::PreparedContent,
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
        spatial: scene::SpatialBinding,
    },
    Scroll {
        viewport: crate::paint::Rect,
        parent_origin: [f32; 2],
        spatial: scene::SpatialBinding,
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
            pending_candidates: Vec::new(),
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
            spatial_topology: commit.spatial_topology(),
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
        let spatial_bindings = collect_plan_spatial_bindings(&batches);
        let property_bindings = pending.property_bindings;
        let property_offsets = Arc::new(collect_property_offsets(
            &property_bindings,
            self.shapes.property_stride,
        ));
        let property_dependents = Arc::new(collect_property_dependents(commit, &property_bindings));
        let (scroll_bindings, scroll_offsets, scroll_dependents) = collect_scroll_bindings(
            commit,
            &property_bindings,
            self.shapes.scroll_property_stride,
        );
        let plan = Arc::new(Plan {
            batches,
            property_bindings,
            property_offsets,
            property_dependents,
            scroll_bindings,
            scroll_offsets,
            scroll_dependents,
            spatial_bindings,
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
        self.pending_candidates
            .retain(|pending| !Arc::ptr_eq(&pending.commit, commit));
        self.shapes.cancel_property_state(commit);
    }

    pub(in crate::render) fn cancel_layer_synchronization(&mut self, layer: &scene::Layer) {
        let projection = Projection::from_layer(layer);
        self.pending.retain(|pending| {
            !Arc::ptr_eq(&pending.commit, layer.drawable_commit())
                || pending.projection != projection
        });
        self.pending_candidates.retain(|pending| {
            !Arc::ptr_eq(&pending.commit, layer.drawable_commit())
                || pending.projection != projection
        });
    }

    pub(in crate::render) fn cancel_property_state(&mut self, commit: &Arc<scene::Commit>) {
        self.pending_candidates
            .retain(|pending| !Arc::ptr_eq(&pending.commit, commit));
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
                spatial_topology: commit.spatial_topology(),
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
                projection: projection.clone(),
                plan: Arc::clone(&plan),
            });
            plan
        };

        let (mut property_bindings, property_stats) = self.shapes.prepare_properties(
            render_context,
            viewport,
            commit,
            properties,
            &plan.property_bindings,
            Arc::clone(&plan.property_offsets),
            &plan.property_dependents,
            &plan.scroll_bindings,
            Arc::clone(&plan.scroll_offsets),
            &plan.scroll_dependents,
        );
        property_bindings.prepare_spatial_translations(
            commit.spatial_topology(),
            properties,
            &plan.spatial_bindings,
        );
        apply_sync_stats(&mut stats, property_stats);
        apply_sync_stats(
            &mut stats,
            prepare_text_transforms(
                render_context,
                viewport,
                commit,
                plan.batches(),
                &mut property_bindings,
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
        self.pending_candidates.retain(|pending| {
            !Arc::ptr_eq(&pending.commit, commit)
                || pending.viewport != viewport
                || pending.projection != projection
                || pending.properties != properties.serial()
        });

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

    pub(in crate::render) fn synchronize_candidate_layer(
        &mut self,
        render_context: &render::Context,
        viewport: render::Viewport,
        layer: &scene::Layer,
        text_renderer: &mut render::text_renderer::TextRenderer,
        budget: std::time::Duration,
    ) -> render::Result<Synchronize> {
        validate_residencies(layer)?;
        self.synchronize_candidate_projected(
            render_context,
            viewport,
            layer.drawable_commit(),
            layer.properties(),
            Projection::from_layer(layer),
            text_renderer,
            budget,
        )
    }

    fn synchronize_candidate_projected(
        &mut self,
        render_context: &render::Context,
        viewport: render::Viewport,
        commit: &Arc<scene::Commit>,
        properties: &scene::Properties,
        projection: Projection,
        text_renderer: &mut render::text_renderer::TextRenderer,
        budget: std::time::Duration,
    ) -> render::Result<Synchronize> {
        properties
            .require_compatible(commit)
            .map_err(|_| render::Error::RetainedSceneContract)?;
        let Some(plan) = self.find_plan(commit, viewport, &projection) else {
            return Ok(Synchronize::Pending);
        };
        let started = std::time::Instant::now();
        self.pending_candidates.retain(|pending| {
            !Arc::ptr_eq(&pending.commit, commit)
                || pending.viewport != viewport
                || pending.projection != projection
                || pending.properties == properties.serial()
        });
        let pending_index = self.pending_candidates.iter().position(|pending| {
            pending.viewport == viewport
                && pending.projection == projection
                && pending.properties == properties.serial()
                && Arc::ptr_eq(&pending.commit, commit)
        });
        let index = if let Some(index) = pending_index {
            index
        } else {
            let (mut property_bindings, property_stats) = self.shapes.prepare_properties(
                render_context,
                viewport,
                commit,
                properties,
                &plan.property_bindings,
                Arc::clone(&plan.property_offsets),
                &plan.property_dependents,
                &plan.scroll_bindings,
                Arc::clone(&plan.scroll_offsets),
                &plan.scroll_dependents,
            );
            property_bindings.prepare_spatial_translations(
                commit.spatial_topology(),
                properties,
                &plan.spatial_bindings,
            );
            self.add_prepared_sync_stats(commit, viewport, &projection, property_stats);
            let mut transforms = Vec::new();
            collect_text_transforms(plan.batches(), &mut property_bindings, &mut transforms);
            self.pending_candidates.push(PendingCandidate {
                commit: Arc::clone(commit),
                viewport,
                projection: projection.clone(),
                properties: properties.serial(),
                transforms,
                next_transform: 0,
            });
            self.pending_candidates.len().saturating_sub(1)
        };

        let remaining = budget.saturating_sub(started.elapsed());
        let (report, next, complete) = {
            let pending = &mut self.pending_candidates[index];
            let (report, next) = text_renderer.prepare_retained_transforms_bounded(
                render_context,
                viewport,
                commit,
                &pending.transforms,
                pending.next_transform,
                remaining,
            );
            pending.next_transform = next;
            (report, next, next == pending.transforms.len())
        };
        self.add_prepared_sync_stats(
            commit,
            viewport,
            &projection,
            SyncStats {
                property_upload_bytes: report.property_upload_bytes,
                text_property_upload_bytes: report.property_upload_bytes,
                resource_creations: report.resource_creations,
                resource_removals: report.resource_removals,
                ..SyncStats::default()
            },
        );
        if complete {
            Ok(Synchronize::Ready)
        } else {
            debug_assert!(next > 0 || remaining.is_zero());
            Ok(Synchronize::Pending)
        }
    }

    #[cfg(feature = "renderer-debug")]
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
        let (mut property_bindings, property_stats) = self.shapes.prepare_properties(
            render_context,
            viewport,
            commit,
            properties,
            &plan.property_bindings,
            Arc::clone(&plan.property_offsets),
            &plan.property_dependents,
            &plan.scroll_bindings,
            Arc::clone(&plan.scroll_offsets),
            &plan.scroll_dependents,
        );
        property_bindings.prepare_spatial_translations(
            commit.spatial_topology(),
            properties,
            &plan.spatial_bindings,
        );
        let mut stats = render::DrawStats::default();
        apply_sync_stats(&mut stats, property_stats);
        apply_sync_stats(
            &mut stats,
            prepare_text_transforms(
                render_context,
                viewport,
                commit,
                plan.batches(),
                &mut property_bindings,
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
        (
            self.plans.len(),
            self.pending.len().saturating_add(
                self.pending_candidates
                    .iter()
                    .filter(|pending| pending.next_transform < pending.transforms.len())
                    .count(),
            ),
        )
    }

    #[cfg(feature = "renderer-debug")]
    pub(in crate::render) fn debug_plan_signature(
        &self,
        commit: &Arc<scene::Commit>,
        viewport: render::Viewport,
    ) -> Option<String> {
        self.find_plan(commit, viewport, &Projection::source())
            .map(|plan| plan.debug_signature())
    }

    #[cfg(feature = "renderer-debug")]
    pub(in crate::render) fn debug_resource_state(&self) -> (usize, usize) {
        (self.shapes.resource_count(), self.shapes.resource_bytes())
    }

    fn collect_plans(&mut self) -> usize {
        let before = self.plans.len();
        self.plans.retain(|entry| entry.commit.strong_count() > 0);
        self.pending_candidates
            .retain(|pending| Arc::strong_count(&pending.commit) > 1);
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
        self.pending_candidates.retain(|pending| {
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

    fn add_prepared_sync_stats(
        &mut self,
        commit: &Arc<scene::Commit>,
        viewport: render::Viewport,
        projection: &Projection,
        sync: SyncStats,
    ) {
        if let Some(prepared) = self.prepared_stats.iter_mut().find(|entry| {
            entry.viewport == viewport
                && entry.projection == *projection
                && entry
                    .commit
                    .upgrade()
                    .is_some_and(|candidate| Arc::ptr_eq(&candidate, commit))
        }) {
            apply_sync_stats(&mut prepared.stats, sync);
        }
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
    batches: &[PlanStep],
    property_bindings: &mut PropertyBindings,
    text_renderer: &mut render::text_renderer::TextRenderer,
) -> SyncStats {
    let mut transforms = Vec::new();
    collect_text_transforms(batches, property_bindings, &mut transforms);
    let report =
        text_renderer.prepare_retained_transforms(render_context, viewport, commit, &transforms);
    SyncStats {
        property_upload_bytes: report.property_upload_bytes,
        text_property_upload_bytes: report.property_upload_bytes,
        resource_creations: report.resource_creations,
        resource_removals: report.resource_removals,
        ..SyncStats::default()
    }
}

fn collect_text_transforms(
    batches: &[PlanStep],
    property_bindings: &mut PropertyBindings,
    transforms: &mut Vec<(render::text_renderer::RetainedBatch, [f32; 2])>,
) {
    for batch in batches {
        match batch {
            PlanStep::Layer(layer) => {
                collect_text_transforms(&layer.render_batches, property_bindings, transforms);
            }
            PlanStep::Text(batch) => {
                let translation = property_bindings.spatial_translation(batch.spatial());
                transforms.push((batch.clone(), batch.translation(translation)));
            }
            PlanStep::Group(group) => {
                collect_text_transforms(&group.render_batches, property_bindings, transforms);
            }
            PlanStep::Scroll(scroll) => {
                collect_text_transforms(&scroll.render_batches, property_bindings, transforms);
            }
            PlanStep::Shapes(_) | PlanStep::Pane(_) | PlanStep::PushClip(_) | PlanStep::PopClip => {
            }
        }
    }
}

enum TextOrderClass {
    Text {
        bounds: crate::paint::Rect,
        spatial: scene::SpatialBinding,
        areas: usize,
    },
    Reorderable {
        bounds: crate::paint::Rect,
        spatial: scene::SpatialBinding,
    },
    Barrier,
}

fn ordered_draws_for_text_batching(
    commit: &Arc<scene::Commit>,
    viewport: render::Viewport,
    projection: &Projection,
) -> Option<Arc<[OrderedDraw]>> {
    let source = commit.order()?;
    let mut output = Vec::with_capacity(source.len());
    let mut pending_text = Vec::<(OrderedDraw, PendingTextBounds, usize)>::new();
    let mut pending_areas = 0_usize;
    let mut scroll_scopes = Vec::new();

    let flush = |output: &mut Vec<OrderedDraw>,
                 pending: &mut Vec<(OrderedDraw, PendingTextBounds, usize)>,
                 pending_areas: &mut usize| {
        output.extend(pending.drain(..).map(|(draw, _, _)| draw));
        *pending_areas = 0;
    };

    for (original_index, draw) in source.iter().cloned().enumerate() {
        match &draw {
            scene::Draw::PushScroll { node } => {
                let is_noop = commit
                    .node(*node)
                    .and_then(|owner| owner.scroll())
                    .is_some_and(|scroll| scroll.maximum().x() == 0 && scroll.maximum().y() == 0);
                scroll_scopes.push(is_noop);
                if is_noop {
                    continue;
                }
            }
            scene::Draw::PopScroll => {
                let is_noop = scroll_scopes
                    .pop()
                    .expect("ordered scene scroll scopes must be balanced");
                if is_noop {
                    continue;
                }
            }
            scene::Draw::Content { .. }
            | scene::Draw::PushClip { .. }
            | scene::Draw::PopClip
            | scene::Draw::PushGroup { .. }
            | scene::Draw::PopGroup => {}
        }
        let ordered = OrderedDraw {
            original_index,
            draw,
        };
        match text_order_class(commit, viewport, projection, &ordered) {
            TextOrderClass::Text {
                bounds,
                spatial,
                areas,
            } => {
                let incompatible_spatial = pending_text
                    .first()
                    .is_some_and(|(_, pending, _)| pending.spatial != spatial);
                let exceeds_limit = !pending_text.is_empty()
                    && pending_areas.saturating_add(areas) > MAX_TEXT_AREAS_PER_PREPARED_BATCH;
                if incompatible_spatial || exceeds_limit {
                    flush(&mut output, &mut pending_text, &mut pending_areas);
                }
                pending_areas = pending_areas.saturating_add(areas);
                pending_text.push((ordered, PendingTextBounds { bounds, spatial }, areas));
            }
            TextOrderClass::Reorderable { bounds, spatial } => {
                let blocks_pending = pending_text.iter().any(|(_, pending, _)| {
                    pending.spatial != spatial || paint_rects_overlap(pending.bounds, bounds)
                });
                if blocks_pending {
                    flush(&mut output, &mut pending_text, &mut pending_areas);
                }
                output.push(ordered);
            }
            TextOrderClass::Barrier => {
                flush(&mut output, &mut pending_text, &mut pending_areas);
                output.push(ordered);
            }
        }
    }
    debug_assert!(scroll_scopes.is_empty());
    flush(&mut output, &mut pending_text, &mut pending_areas);
    Some(output.into())
}

fn text_order_class(
    commit: &Arc<scene::Commit>,
    viewport: render::Viewport,
    projection: &Projection,
    ordered: &OrderedDraw,
) -> TextOrderClass {
    let scene::Draw::Content {
        node,
        index,
        projection: content_projection,
    } = &ordered.draw
    else {
        return TextOrderClass::Barrier;
    };
    if *content_projection != scene::ContentProjection::Normal {
        return TextOrderClass::Barrier;
    }
    let Some(owner) = commit.node(*node) else {
        return TextOrderClass::Barrier;
    };
    if owner.declares(scene::PropertyKind::Transform) {
        return TextOrderClass::Barrier;
    }
    let Some(content) = owner.content().get(*index) else {
        return TextOrderClass::Barrier;
    };
    let Some((content, _)) = projection.material.projected_content(content) else {
        return TextOrderClass::Barrier;
    };
    let spatial = commit
        .spatial_topology()
        .draw_state(ordered.original_index)
        .map(scene::SpatialPropertyState::spatial)
        .unwrap_or(scene::SpatialBinding::ROOT);
    let prepared = render::scene::prepare_content(&content, viewport.scale_factor());
    let bounds = prepared_content_batch_bounds(&prepared, spatial, viewport.scale_factor());
    match content {
        scene::Content::Text(_) | scene::Content::Icon(_) => TextOrderClass::Text {
            bounds,
            spatial,
            areas: 1,
        },
        scene::Content::TextViewport(ref text) => TextOrderClass::Text {
            bounds,
            spatial,
            areas: text.surfaces().len().max(1),
        },
        scene::Content::Pane(_) => TextOrderClass::Barrier,
        scene::Content::Quad(_)
        | scene::Content::Rule(_)
        | scene::Content::Shadow(_)
        | scene::Content::Outline(_) => TextOrderClass::Reorderable { bounds, spatial },
    }
}

fn prepared_content_batch_bounds(
    content: &render::scene::PreparedContent,
    spatial: scene::SpatialBinding,
    scale_factor: f32,
) -> crate::paint::Rect {
    match content {
        // A resident text viewport clips each surface to its resident surface rect so that
        // scrolling can expose already-shaped glyphs outside the visible viewport. Reordering
        // against only the viewport rect would therefore understate the pixels this draw can
        // cover. Identity-bound viewports retain the ordinary viewport clip.
        render::scene::PreparedContent::TextViewport(viewport) if !spatial.is_identity() => {
            viewport
                .surfaces
                .iter()
                .map(|surface| surface.rect)
                .fold(viewport.rect, crate::paint::union_visual_bounds)
        }
        _ => content.bounds(scale_factor),
    }
}

fn paint_rects_overlap(left: crate::paint::Rect, right: crate::paint::Rect) -> bool {
    let left_right = left.origin.x() + left.area.width().max(0.0);
    let left_bottom = left.origin.y() + left.area.height().max(0.0);
    let right_right = right.origin.x() + right.area.width().max(0.0);
    let right_bottom = right.origin.y() + right.area.height().max(0.0);
    left.origin.x() < right_right
        && right.origin.x() < left_right
        && left.origin.y() < right_bottom
        && right.origin.y() < left_bottom
}

fn pending_glyph(
    nodes: &HashMap<composition::tree::NodeId, Arc<scene::Node>>,
    ordered: &OrderedDraw,
) -> Option<(PendingGlyph, usize)> {
    let scene::Draw::Content {
        node,
        index,
        projection,
    } = &ordered.draw
    else {
        return None;
    };
    if *projection != scene::ContentProjection::Normal {
        return None;
    }
    let owner = nodes.get(node)?;
    if owner.declares(scene::PropertyKind::Transform) {
        return None;
    }
    let content = owner.content().get(*index)?;
    let areas = match content {
        scene::Content::Text(_) | scene::Content::Icon(_) => 1,
        scene::Content::TextViewport(text) => text.surfaces().len().max(1),
        scene::Content::Quad(_)
        | scene::Content::Rule(_)
        | scene::Content::Shadow(_)
        | scene::Content::Pane(_)
        | scene::Content::Outline(_) => return None,
    };
    Some((
        PendingGlyph {
            node: Arc::clone(owner),
            content_index: *index,
            content: content.clone(),
        },
        areas,
    ))
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
            spatial: scene::SpatialBinding::ROOT,
        };
        let order = ordered_draws_for_text_batching(&commit, viewport, &projection);
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
        while let Some(ordered) = order.get(self.order_index) {
            let draw_index = ordered.original_index;
            self.order_index = self.order_index.saturating_add(1);
            let draw = &ordered.draw;
            let space = self
                .frames
                .last()
                .expect("retained preparation must keep a root frame")
                .space;
            if let Some((first, first_areas)) = pending_glyph(&self.nodes, ordered) {
                let glyph_space = self
                    .commit
                    .spatial_topology()
                    .draw_state(draw_index)
                    .map_or(space, |state| space.with_spatial(state));
                let mut glyphs = vec![(first, glyph_space)];
                let mut areas = first_areas;
                while let Some(next) = order.get(self.order_index) {
                    let Some((glyph, glyph_areas)) = pending_glyph(&self.nodes, next) else {
                        break;
                    };
                    let next_space = self
                        .commit
                        .spatial_topology()
                        .draw_state(next.original_index)
                        .map_or(space, |state| space.with_spatial(state));
                    if !text_batch_spaces_compatible(glyph_space, next_space)
                        || areas.saturating_add(glyph_areas) > MAX_TEXT_AREAS_PER_PREPARED_BATCH
                    {
                        break;
                    }
                    self.order_index = self.order_index.saturating_add(1);
                    areas = areas.saturating_add(glyph_areas);
                    glyphs.push((glyph, next_space));
                }
                let target = &mut self
                    .frames
                    .last_mut()
                    .expect("retained preparation must keep a target frame")
                    .batches;
                builder.build_glyph_batch(&self.commit, glyphs, target)?;
                prepared_content = true;
                if started_at.elapsed() >= budget {
                    return Ok(false);
                }
                continue;
            }
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
                            &self.commit,
                            owner,
                            *index,
                            content,
                            *projection,
                            self.commit
                                .spatial_topology()
                                .draw_state(draw_index)
                                .map_or(space, |state| space.with_spatial(state)),
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
                            spatial: builder
                                .spatial_topology
                                .draw_state(draw_index)
                                .map(scene::SpatialPropertyState::spatial)
                                .unwrap_or(space.spatial),
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
                    let bounds = render::scene::to_paint_rect_value_at_scale(
                        builder
                            .spatial_topology
                            .surface_bounds_for_draw(draw_index)
                            .unwrap_or(*bounds),
                        builder.viewport.scale_factor(),
                    );
                    self.frames.push(PendingFrame {
                        kind: PendingFrameKind::Group {
                            node: *node,
                            bounds,
                            opacity: *opacity,
                            parent_origin: space.origin,
                            spatial: builder
                                .spatial_topology
                                .draw_state(draw_index)
                                .map(scene::SpatialPropertyState::spatial)
                                .unwrap_or(space.spatial),
                        },
                        space: TargetSpace {
                            origin: [bounds.origin.x(), bounds.origin.y()],
                            size: [bounds.area.width().max(1.0), bounds.area.height().max(1.0)],
                            text_origin: [bounds.origin.x(), bounds.origin.y()],
                            text_size: [
                                bounds.area.width().max(1.0),
                                bounds.area.height().max(1.0),
                            ],
                            spatial: space.spatial,
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
                            viewport,
                            parent_origin: space.origin,
                            spatial: builder
                                .spatial_topology
                                .draw_state(draw_index)
                                .map(scene::SpatialPropertyState::spatial)
                                .unwrap_or(space.spatial),
                        },
                        space: TargetSpace {
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
            spatial,
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
                spatial,
                render_batches: frame.batches,
            }));
    }

    fn finish_scroll(&mut self, builder: &mut PlanBuilder<'_>) {
        let Some(frame) = (self.frames.len() > 1).then(|| self.frames.pop()).flatten() else {
            return;
        };
        let PendingFrameKind::Scroll {
            viewport,
            parent_origin,
            spatial,
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
                viewport: local_rect(viewport, parent_origin),
                spatial,
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
    spatial_topology: &'a scene::SpatialTopology,
    rebuilt_nodes: HashSet<composition::tree::NodeId>,
    property_bindings: Vec<PropertyBinding>,
    stats: render::DrawStats,
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
            scroll: ScrollBinding {
                path: self
                    .spatial_topology
                    .scroll_path(space.spatial)
                    .unwrap_or(scene::ScrollPathId::ROOT),
            },
        };
        if !self.property_bindings.contains(&binding) {
            self.property_bindings.push(binding);
        }
        binding
    }

    fn build(&mut self, commit: &Arc<scene::Commit>) -> render::Result<Plan> {
        let mut compiler =
            PendingPlan::new(Arc::clone(commit), self.viewport, self.projection.clone());
        self.stats = std::mem::take(&mut compiler.stats);
        let ready = compiler.advance(self, std::time::Duration::MAX)?;
        debug_assert!(ready, "an unbounded plan compile must finish in one slice");
        let mut batches = compiler
            .frames
            .pop()
            .expect("completed plan compilation must retain its root frame")
            .batches;
        coalesce_shape_batches(&mut batches);
        let requires_surface_sampling = render::renderer::requires_surface_sampling(&batches);
        let spatial_bindings = collect_plan_spatial_bindings(&batches);
        let property_bindings = std::mem::take(&mut self.property_bindings);
        let property_offsets = Arc::new(collect_property_offsets(
            &property_bindings,
            self.shapes.property_stride,
        ));
        let property_dependents = Arc::new(collect_property_dependents(commit, &property_bindings));
        let (scroll_bindings, scroll_offsets, scroll_dependents) = collect_scroll_bindings(
            commit,
            &property_bindings,
            self.shapes.scroll_property_stride,
        );
        Ok(Plan {
            batches,
            property_bindings,
            property_offsets,
            property_dependents,
            scroll_bindings,
            scroll_offsets,
            scroll_dependents,
            spatial_bindings,
            requires_surface_sampling,
            facts: PlanFacts::from_stats(&self.stats),
        })
    }

    fn build_node(
        &mut self,
        commit: &Arc<scene::Commit>,
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
                spatial: parent_space.spatial,
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
            spatial: parent_space.spatial,
        });
        let mut body = Vec::new();
        for (index, content) in node.content().iter().enumerate() {
            let content_space = commit
                .spatial_topology()
                .content_state(node.id(), index, scene::ContentProjection::Normal)
                .map_or(body_space, |state| body_space.with_spatial(state));
            self.build_content(
                commit,
                node,
                index,
                content,
                scene::ContentProjection::Normal,
                content_space,
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
                    spatial: parent_space.spatial,
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
            || node.declares(scene::PropertyKind::Offset)
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
        commit: &Arc<scene::Commit>,
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
                projection,
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
                commit,
                node,
                content_index,
                content::Glyph::Text(value),
                space,
                target,
            )?,
            render::scene::PreparedContent::TextViewport(value) => {
                self.stats.text_surfaces += value.surfaces.len();
                self.push_glyph(
                    commit,
                    node,
                    content_index,
                    content::Glyph::TextViewport(value),
                    space,
                    target,
                )?;
            }
            render::scene::PreparedContent::Icon(value) => self.push_glyph(
                commit,
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

    fn build_glyph_batch(
        &mut self,
        commit: &Arc<scene::Commit>,
        glyphs: Vec<(PendingGlyph, TargetSpace)>,
        target: &mut Vec<PlanStep>,
    ) -> render::Result<()> {
        let space = glyphs
            .iter()
            .map(|(_, space)| *space)
            .reduce(union_text_target)
            .expect("a glyph batch must contain at least one text area");
        let mut prepared = Vec::with_capacity(glyphs.len());
        for (glyph, _) in glyphs {
            self.stats.scene_items = self.stats.scene_items.saturating_add(1);
            let Some((content, _)) = self.projection.material.projected_content(&glyph.content)
            else {
                continue;
            };
            let content = render::scene::prepare_content(&content, self.viewport.scale_factor());
            if let render::scene::PreparedContent::TextViewport(viewport) = &content {
                self.stats.text_surfaces = self
                    .stats
                    .text_surfaces
                    .saturating_add(viewport.surfaces.len());
            }
            prepared.push(PreparedGlyph {
                node: glyph.node,
                content_index: glyph.content_index,
                content,
            });
        }
        let retained = prepared
            .iter()
            .filter_map(|glyph| {
                let content = match &glyph.content {
                    render::scene::PreparedContent::Text(text) => content::Glyph::Text(text),
                    render::scene::PreparedContent::TextViewport(text) => {
                        content::Glyph::TextViewport(text)
                    }
                    render::scene::PreparedContent::Icon(icon) => content::Glyph::Icon(icon),
                    render::scene::PreparedContent::Quad(_)
                    | render::scene::PreparedContent::Rule(_)
                    | render::scene::PreparedContent::Shadow(_)
                    | render::scene::PreparedContent::Pane(_)
                    | render::scene::PreparedContent::Outline(_) => return None,
                };
                Some(render::text_renderer::RetainedGlyph {
                    node: &glyph.node,
                    content_index: glyph.content_index,
                    glyph: content,
                })
            })
            .collect::<Vec<_>>();
        if retained.is_empty() {
            return Ok(());
        }

        self.stats.glyph_batches = self.stats.glyph_batches.saturating_add(1);
        let report = self.text_renderer.prepare_retained(
            self.render_context,
            self.viewport,
            commit,
            &retained,
            space.text_origin,
            space.text_size,
            space.size,
            [
                space.text_origin[0] - space.origin[0],
                space.text_origin[1] - space.origin[1],
            ],
            space.spatial,
        )?;
        if report.prepare_calls > 0 {
            self.rebuilt_nodes
                .extend(prepared.iter().map(|glyph| glyph.node.id()));
        }
        self.stats.text_prepare_calls = self
            .stats
            .text_prepare_calls
            .saturating_add(report.prepare_calls);
        self.stats.inline_text_cache_hits = self
            .stats
            .inline_text_cache_hits
            .saturating_add(report.stats.text_cache_hits);
        self.stats.inline_text_cache_misses = self
            .stats
            .inline_text_cache_misses
            .saturating_add(report.stats.text_cache_misses);
        self.stats.inline_text_shape_calls = self
            .stats
            .inline_text_shape_calls
            .saturating_add(report.stats.text_shape_calls);
        self.stats.inline_icon_cache_hits = self
            .stats
            .inline_icon_cache_hits
            .saturating_add(report.stats.icon_cache_hits);
        self.stats.inline_icon_cache_misses = self
            .stats
            .inline_icon_cache_misses
            .saturating_add(report.stats.icon_cache_misses);
        self.stats.inline_icon_shape_calls = self
            .stats
            .inline_icon_shape_calls
            .saturating_add(report.stats.icon_shape_calls);
        self.stats.retained_gpu_resource_creations = self
            .stats
            .retained_gpu_resource_creations
            .saturating_add(report.resource_creations);
        self.stats.retained_gpu_resource_removals = self
            .stats
            .retained_gpu_resource_removals
            .saturating_add(report.resource_removals);
        if let Some(batch) = report.batch {
            target.push(PlanStep::Text(batch));
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
        commit: &Arc<scene::Commit>,
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
            commit,
            &[render::text_renderer::RetainedGlyph {
                node,
                content_index,
                glyph,
            }],
            space.text_origin,
            space.text_size,
            space.size,
            [
                space.text_origin[0] - space.origin[0],
                space.text_origin[1] - space.origin[1],
            ],
            space.spatial,
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
            spatial: space.spatial,
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
    stats.viewport_property_upload_bytes += sync.viewport_property_upload_bytes;
    stats.node_property_upload_bytes += sync.node_property_upload_bytes;
    stats.scroll_property_upload_bytes += sync.scroll_property_upload_bytes;
    stats.text_property_upload_bytes += sync.text_property_upload_bytes;
    stats.property_value_visits += sync.property_value_visits;
    stats.property_index_lookups += sync.property_index_lookups;
    stats.property_dirty_indices += sync.property_dirty_indices;
    stats.property_write_ranges += sync.property_write_ranges;
    stats.property_full_initializations += sync.property_full_initializations;
    stats.property_full_buffer_replacements += sync.property_full_buffer_replacements;
    stats.property_full_topology_replacements += sync.property_full_topology_replacements;
    stats.property_full_dense_transfers += sync.property_full_dense_transfers;
    stats.property_full_generation_resyncs += sync.property_full_generation_resyncs;
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

fn collect_property_offsets(
    bindings: &[PropertyBinding],
    stride: usize,
) -> HashMap<PropertyBinding, u32> {
    bindings
        .iter()
        .copied()
        .enumerate()
        .map(|(index, binding)| (binding, index.saturating_mul(stride) as u32))
        .collect()
}

fn property_write_ranges(
    sorted_binding_indices: &[usize],
    stride: usize,
    property_size: usize,
) -> Vec<Range<usize>> {
    let Some(first) = sorted_binding_indices.first().copied() else {
        return Vec::new();
    };
    let mut ranges = Vec::new();
    let mut start = first;
    let mut previous = first;
    for index in sorted_binding_indices.iter().copied().skip(1) {
        if index == previous.saturating_add(1) {
            previous = index;
            continue;
        }
        ranges.push(start.saturating_mul(stride)..previous.saturating_mul(stride) + property_size);
        start = index;
        previous = index;
    }
    ranges.push(start.saturating_mul(stride)..previous.saturating_mul(stride) + property_size);
    ranges
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum PropertyTransfer {
    Unchanged,
    Sparse {
        ranges: Vec<Range<usize>>,
        bytes: usize,
    },
    Dense,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum PropertyFullReason {
    Initialization,
    BufferReplacement,
    TopologyReplacement,
    Dense,
    GenerationResync,
}

fn record_property_full_transfer(stats: &mut SyncStats, reason: PropertyFullReason) {
    let counter = match reason {
        PropertyFullReason::Initialization => &mut stats.property_full_initializations,
        PropertyFullReason::BufferReplacement => &mut stats.property_full_buffer_replacements,
        PropertyFullReason::TopologyReplacement => &mut stats.property_full_topology_replacements,
        PropertyFullReason::Dense => &mut stats.property_full_dense_transfers,
        PropertyFullReason::GenerationResync => &mut stats.property_full_generation_resyncs,
    };
    *counter = counter.saturating_add(1);
}

fn plan_property_transfer(
    sorted_binding_indices: &[usize],
    stride: usize,
    property_size: usize,
    full_bytes: usize,
) -> PropertyTransfer {
    let ranges = property_write_ranges(sorted_binding_indices, stride, property_size);
    if ranges.is_empty() {
        return PropertyTransfer::Unchanged;
    }
    let bytes = ranges.iter().map(|range| range.len()).sum::<usize>();
    // Model each queue write as one aligned property slot in addition to its payload. This keeps
    // the policy deterministic across node and scroll buffers while accounting for command count.
    let sparse_cost = bytes.saturating_add(ranges.len().saturating_mul(stride));
    if sparse_cost < full_bytes {
        PropertyTransfer::Sparse { ranges, bytes }
    } else {
        PropertyTransfer::Dense
    }
}

fn collect_property_dependents(
    commit: &scene::Commit,
    bindings: &[PropertyBinding],
) -> HashMap<scene::PropertyIndex, Vec<usize>> {
    let mut dependents = HashMap::new();
    for (binding_index, binding) in bindings.iter().copied().enumerate() {
        let mut kinds = Vec::with_capacity(2);
        match binding.projection {
            scene::ContentProjection::Normal => kinds.push(scene::PropertyKind::Transform),
            scene::ContentProjection::Caret => kinds.push(scene::PropertyKind::Caret),
            scene::ContentProjection::ScrollbarTrack { axis, .. } => {
                kinds.push(scene::PropertyKind::scrollbar(axis));
            }
            scene::ContentProjection::ScrollbarThumb { axis, .. } => {
                kinds.push(scene::PropertyKind::scrollbar(axis));
                kinds.push(scene::PropertyKind::Offset);
            }
        };
        for kind in kinds {
            let Some(property) = commit.property_index(scene::PropertyRef::new(binding.node, kind))
            else {
                continue;
            };
            dependents
                .entry(property)
                .or_insert_with(Vec::new)
                .push(binding_index);
        }
    }
    dependents
}

fn collect_scroll_bindings(
    commit: &scene::Commit,
    bindings: &[PropertyBinding],
    stride: usize,
) -> (
    Arc<[ScrollBinding]>,
    Arc<HashMap<ScrollBinding, u32>>,
    Arc<HashMap<scene::PropertyIndex, Vec<usize>>>,
) {
    let mut scroll_bindings = Vec::new();
    for binding in bindings.iter().map(|binding| binding.scroll()) {
        if !scroll_bindings.contains(&binding) {
            scroll_bindings.push(binding);
        }
    }
    if scroll_bindings.is_empty() && !bindings.is_empty() {
        scroll_bindings.push(ScrollBinding::IDENTITY);
    }
    let offsets = scroll_bindings
        .iter()
        .copied()
        .enumerate()
        .map(|(index, binding)| (binding, index.saturating_mul(stride) as u32))
        .collect::<HashMap<_, _>>();
    let mut dependents = HashMap::new();
    for (binding_index, binding) in scroll_bindings.iter().copied().enumerate() {
        let owners = commit
            .spatial_topology()
            .scroll_path_owners(binding.path)
            .unwrap_or_default();
        for owner in owners {
            let Some(property) =
                commit.property_index(scene::PropertyRef::new(owner, scene::PropertyKind::Offset))
            else {
                continue;
            };
            dependents
                .entry(property)
                .or_insert_with(Vec::new)
                .push(binding_index);
        }
    }
    (
        scroll_bindings.into(),
        Arc::new(offsets),
        Arc::new(dependents),
    )
}

fn collect_plan_spatial_bindings(batches: &[PlanStep]) -> Vec<scene::SpatialBinding> {
    fn push_unique(target: &mut Vec<scene::SpatialBinding>, binding: scene::SpatialBinding) {
        if !target.contains(&binding) {
            target.push(binding);
        }
    }

    fn collect(batches: &[PlanStep], target: &mut Vec<scene::SpatialBinding>) {
        for batch in batches {
            match batch {
                PlanStep::Layer(layer) => collect(&layer.render_batches, target),
                PlanStep::Shapes(batch) => push_unique(target, batch.binding().space.spatial),
                PlanStep::Pane(pane) => push_unique(target, pane.spatial),
                PlanStep::Text(batch) => push_unique(target, batch.spatial()),
                PlanStep::PushClip(clip) => push_unique(target, clip.spatial),
                PlanStep::PopClip => {}
                PlanStep::Group(group) => {
                    push_unique(target, group.spatial);
                    collect(&group.render_batches, target);
                }
                PlanStep::Scroll(scroll) => {
                    push_unique(target, scroll.spatial);
                    collect(&scroll.render_batches, target);
                }
            }
        }
    }

    let mut bindings = Vec::new();
    collect(batches, &mut bindings);
    bindings
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
    offsets: Arc<HashMap<PropertyBinding, u32>>,
    slot: usize,
    scroll_offsets: Arc<HashMap<ScrollBinding, u32>>,
    scroll_slot: usize,
    spatial_translations: HashMap<scene::SpatialBinding, [f32; 2]>,
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

    pub(in crate::render) fn spatial_translation(
        &self,
        spatial: scene::SpatialBinding,
    ) -> [f32; 2] {
        self.spatial_translations
            .get(&spatial)
            .copied()
            .unwrap_or_default()
    }

    fn prepare_spatial_translations(
        &mut self,
        topology: &scene::SpatialTopology,
        properties: &scene::Properties,
        bindings: &[scene::SpatialBinding],
    ) {
        self.spatial_translations.clear();
        self.spatial_translations
            .extend(bindings.iter().copied().map(|binding| {
                (
                    binding,
                    topology
                        .scroll_translation(binding, properties)
                        .unwrap_or_default(),
                )
            }));
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
    recycled: Vec<Entry>,
    property_stride: usize,
    bind_group_layout: wgpu::BindGroupLayout,
    property_slots: Vec<PropertySlot>,
    scroll_property_stride: usize,
    scroll_bind_group_layout: wgpu::BindGroupLayout,
    scroll_property_slots: Vec<ScrollPropertySlot>,
}

struct PropertySlot {
    owners: Vec<Weak<scene::Commit>>,
    property_serial: Option<scene::PropertySerial>,
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
    property_serial: Option<scene::PropertySerial>,
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
            recycled: Vec::new(),
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
        let mut entry = self.recycled.pop().unwrap_or_else(|| {
            stats.resource_creations += 1;
            Entry {
                owners: Vec::new(),
                range: None,
            }
        });
        let old_range = entry.range.take();
        let range = match (NonZeroUsize::new(instances.len()), old_range) {
            (Some(count), Some(old_range))
                if (old_range.end - old_range.start) as usize == count.get() =>
            {
                Some(old_range)
            }
            (Some(count), Some(old_range)) => {
                self.release(old_range);
                Some(self.allocate(count.get()))
            }
            (Some(count), None) => Some(self.allocate(count.get())),
            (None, Some(old_range)) => {
                self.release(old_range);
                None
            }
            (None, None) => None,
        };
        self.write_instances(render_context, &instances, range.as_ref(), &mut stats);
        entry.owners.clear();
        entry.owners.push(Arc::downgrade(node));
        entry.range = range.clone();
        self.entries.insert(key, entry);

        (range.map(|range| ShapeBatch { range, binding }), stats)
    }

    fn write_instances(
        &mut self,
        render_context: &render::Context,
        instances: &[render::quad::Instance],
        range: Option<&Range<u32>>,
        stats: &mut SyncStats,
    ) {
        let Some(range) = range else {
            return;
        };
        self.instances[range.start as usize..range.end as usize].copy_from_slice(instances);
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
            let bytes = bytemuck::cast_slice(instances);
            render_context.queue().write_buffer(
                &self.instance_buffer,
                range.start as u64 * std::mem::size_of::<render::quad::Instance>() as u64,
                bytes,
            );
            stats.content_upload_bytes += bytes.len();
        }
    }

    fn prepare_properties(
        &mut self,
        render_context: &render::Context,
        viewport: render::Viewport,
        commit: &Arc<scene::Commit>,
        properties: &scene::Properties,
        bindings: &[PropertyBinding],
        offsets: Arc<HashMap<PropertyBinding, u32>>,
        dependents: &HashMap<scene::PropertyIndex, Vec<usize>>,
        scroll_bindings: &[ScrollBinding],
        scroll_offsets: Arc<HashMap<ScrollBinding, u32>>,
        scroll_dependents: &HashMap<scene::PropertyIndex, Vec<usize>>,
    ) -> (PropertyBindings, SyncStats) {
        let mut stats = SyncStats::default();
        let property_work = properties.work();
        stats.property_value_visits = property_work.value_visits();
        stats.property_index_lookups = property_work.index_lookups();
        stats.property_dirty_indices = properties.changed().len();
        if bindings.is_empty() {
            return (
                PropertyBindings {
                    offsets: Arc::new(HashMap::new()),
                    slot: 0,
                    scroll_offsets: Arc::new(HashMap::new()),
                    scroll_slot: 0,
                    spatial_translations: HashMap::new(),
                },
                stats,
            );
        }

        let slot = self.prepare_node_properties(
            render_context,
            viewport,
            commit,
            properties,
            bindings,
            dependents,
            &mut stats,
        );
        let scroll_slot = self.prepare_scroll_properties(
            render_context,
            commit,
            properties,
            scroll_bindings,
            scroll_dependents,
            &mut stats,
        );

        (
            PropertyBindings {
                offsets,
                slot,
                scroll_offsets,
                scroll_slot,
                spatial_translations: HashMap::new(),
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
        dependents: &HashMap<scene::PropertyIndex, Vec<usize>>,
        stats: &mut SyncStats,
    ) -> usize {
        let required = bindings.len().max(1);
        let viewport_key = [
            viewport.logical_area().width().to_bits(),
            viewport.logical_area().height().to_bits(),
            viewport.scale_factor().to_bits(),
        ];
        for slot in &mut self.property_slots {
            slot.owners.retain(|owner| owner.strong_count() > 0);
        }

        let owned_slot = self.property_slots.iter().position(|slot| {
            slot.owners
                .iter()
                .filter_map(Weak::upgrade)
                .any(|owner| Arc::ptr_eq(&owner, commit))
        });
        if let Some(slot) = owned_slot
            && self.property_slots[slot].viewport_key == viewport_key
            && self.property_slots[slot].bindings == bindings
            && self.property_slots[slot].bytes.len() == required * self.property_stride
            && (self.property_slots[slot].property_serial == Some(properties.serial())
                || (self.property_slots[slot].property_serial == properties.predecessor_serial()
                    && property_slot_exclusively_owned_by(
                        &self.property_slots[slot].owners,
                        commit,
                    )))
        {
            if self.property_slots[slot].property_serial == Some(properties.serial()) {
                return slot;
            }
            let mut dirty_bindings = properties
                .changed()
                .iter()
                .flat_map(|property| dependents.get(property).into_iter().flatten().copied())
                .collect::<Vec<_>>();
            dirty_bindings.sort_unstable();
            dirty_bindings.dedup();

            let property_slot = &mut self.property_slots[slot];
            let mut changed_bindings = Vec::with_capacity(dirty_bindings.len());
            for binding_index in dirty_bindings {
                let property = node_property_for_binding(
                    commit,
                    properties,
                    bindings[binding_index],
                    viewport.scale_factor(),
                    stats,
                );
                let offset = binding_index * self.property_stride;
                let range = offset..offset + std::mem::size_of::<NodeProperty>();
                let bytes = bytemuck::bytes_of(&property);
                if property_slot.bytes[range.clone()] != *bytes {
                    property_slot.bytes[range].copy_from_slice(bytes);
                    changed_bindings.push(binding_index);
                }
            }
            if changed_bindings.is_empty() {
                property_slot.property_serial = Some(properties.serial());
                return slot;
            }

            let transfer = plan_property_transfer(
                &changed_bindings,
                self.property_stride,
                std::mem::size_of::<NodeProperty>(),
                property_slot.bytes.len(),
            );
            if let PropertyTransfer::Sparse {
                ranges,
                bytes: sparse_bytes,
            } = transfer
            {
                for range in &ranges {
                    render_context.queue().write_buffer(
                        &property_slot.property_buffer,
                        range.start as u64,
                        &property_slot.bytes[range.clone()],
                    );
                }
                stats.property_upload_bytes =
                    stats.property_upload_bytes.saturating_add(sparse_bytes);
                stats.node_property_upload_bytes = stats
                    .node_property_upload_bytes
                    .saturating_add(sparse_bytes);
                stats.property_write_ranges =
                    stats.property_write_ranges.saturating_add(ranges.len());
            } else {
                debug_assert_eq!(transfer, PropertyTransfer::Dense);
                render_context.queue().write_buffer(
                    &property_slot.property_buffer,
                    0,
                    &property_slot.bytes,
                );
                stats.property_upload_bytes = stats
                    .property_upload_bytes
                    .saturating_add(property_slot.bytes.len());
                stats.node_property_upload_bytes = stats
                    .node_property_upload_bytes
                    .saturating_add(property_slot.bytes.len());
                stats.property_write_ranges = stats.property_write_ranges.saturating_add(1);
                record_property_full_transfer(stats, PropertyFullReason::Dense);
            }
            property_slot.property_serial = Some(properties.serial());
            return slot;
        }

        let mut bytes = vec![0_u8; required * self.property_stride];
        for (index, binding) in bindings.iter().copied().enumerate() {
            let property = node_property_for_binding(
                commit,
                properties,
                binding,
                viewport.scale_factor(),
                stats,
            );
            let offset = index * self.property_stride;
            bytes[offset..offset + std::mem::size_of::<NodeProperty>()]
                .copy_from_slice(bytemuck::bytes_of(&property));
        }

        if let Some(slot) = self.property_slots.iter_mut().position(|slot| {
            slot.viewport_key == viewport_key && slot.bindings == bindings && slot.bytes == bytes
        }) {
            add_property_owner(&mut self.property_slots[slot].owners, commit);
            self.property_slots[slot].property_serial = Some(properties.serial());
            return slot;
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
        let initialized = property_slot.bytes.is_empty();
        let topology_replaced = !initialized
            && (property_slot.bindings != bindings || property_slot.bytes.len() != bytes.len());
        let generation_resync = !initialized
            && !topology_replaced
            && !viewport_changed
            && property_slot.property_serial != properties.predecessor_serial();
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
            stats.viewport_property_upload_bytes += std::mem::size_of::<ViewportUniform>();
        }

        if buffer_recreated || property_slot.bytes != bytes {
            render_context
                .queue()
                .write_buffer(&property_slot.property_buffer, 0, &bytes);
            stats.property_upload_bytes += bytes.len();
            stats.node_property_upload_bytes += bytes.len();
            stats.property_write_ranges = stats.property_write_ranges.saturating_add(1);
            let reason = if buffer_recreated {
                PropertyFullReason::BufferReplacement
            } else if initialized {
                PropertyFullReason::Initialization
            } else if topology_replaced || viewport_changed {
                PropertyFullReason::TopologyReplacement
            } else if generation_resync {
                PropertyFullReason::GenerationResync
            } else {
                PropertyFullReason::Dense
            };
            record_property_full_transfer(stats, reason);
        }
        property_slot.owners.clear();
        property_slot.owners.push(Arc::downgrade(commit));
        property_slot.property_serial = Some(properties.serial());
        property_slot.viewport_key = viewport_key;
        property_slot.bindings = bindings.to_vec();
        property_slot.bytes = bytes;

        slot
    }

    fn prepare_scroll_properties(
        &mut self,
        render_context: &render::Context,
        commit: &Arc<scene::Commit>,
        properties: &scene::Properties,
        scroll_bindings: &[ScrollBinding],
        dependents: &HashMap<scene::PropertyIndex, Vec<usize>>,
        stats: &mut SyncStats,
    ) -> usize {
        let required = scroll_bindings.len().max(1);
        for slot in &mut self.scroll_property_slots {
            slot.owners.retain(|owner| owner.strong_count() > 0);
        }

        let owned_slot = self.scroll_property_slots.iter().position(|slot| {
            slot.owners
                .iter()
                .filter_map(Weak::upgrade)
                .any(|owner| Arc::ptr_eq(&owner, commit))
        });
        if let Some(slot) = owned_slot
            && self.scroll_property_slots[slot].bindings == scroll_bindings
            && self.scroll_property_slots[slot].bytes.len()
                == required * self.scroll_property_stride
            && (self.scroll_property_slots[slot].property_serial == Some(properties.serial())
                || (self.scroll_property_slots[slot].property_serial
                    == properties.predecessor_serial()
                    && property_slot_exclusively_owned_by(
                        &self.scroll_property_slots[slot].owners,
                        commit,
                    )))
        {
            if self.scroll_property_slots[slot].property_serial == Some(properties.serial()) {
                return slot;
            }
            let mut dirty_bindings = properties
                .changed()
                .iter()
                .flat_map(|property| dependents.get(property).into_iter().flatten().copied())
                .collect::<Vec<_>>();
            dirty_bindings.sort_unstable();
            dirty_bindings.dedup();

            let scroll_slot = &mut self.scroll_property_slots[slot];
            let mut changed_bindings = Vec::with_capacity(dirty_bindings.len());
            for binding_index in dirty_bindings {
                let binding = scroll_bindings[binding_index];
                let dependency_count = commit
                    .spatial_topology()
                    .scroll_path_owners(binding.path)
                    .map_or(0, |owners| owners.len());
                stats.property_index_lookups = stats
                    .property_index_lookups
                    .saturating_add(dependency_count);
                stats.property_value_visits =
                    stats.property_value_visits.saturating_add(dependency_count);
                let translation = commit
                    .spatial_topology()
                    .scroll_path_translation(binding.path, properties)
                    .unwrap_or_default();
                let property = ScrollProperty {
                    translation,
                    ..ScrollProperty::IDENTITY
                };
                let offset = binding_index * self.scroll_property_stride;
                let range = offset..offset + std::mem::size_of::<ScrollProperty>();
                let bytes = bytemuck::bytes_of(&property);
                if scroll_slot.bytes[range.clone()] != bytes[..] {
                    scroll_slot.bytes[range].copy_from_slice(bytes);
                    changed_bindings.push(binding_index);
                }
            }
            if changed_bindings.is_empty() {
                scroll_slot.property_serial = Some(properties.serial());
                return slot;
            }
            let transfer = plan_property_transfer(
                &changed_bindings,
                self.scroll_property_stride,
                std::mem::size_of::<ScrollProperty>(),
                scroll_slot.bytes.len(),
            );
            if let PropertyTransfer::Sparse {
                ranges,
                bytes: sparse_bytes,
            } = transfer
            {
                for range in &ranges {
                    render_context.queue().write_buffer(
                        &scroll_slot.buffer,
                        range.start as u64,
                        &scroll_slot.bytes[range.clone()],
                    );
                }
                stats.property_upload_bytes =
                    stats.property_upload_bytes.saturating_add(sparse_bytes);
                stats.scroll_property_upload_bytes = stats
                    .scroll_property_upload_bytes
                    .saturating_add(sparse_bytes);
                stats.property_write_ranges =
                    stats.property_write_ranges.saturating_add(ranges.len());
            } else {
                debug_assert_eq!(transfer, PropertyTransfer::Dense);
                render_context
                    .queue()
                    .write_buffer(&scroll_slot.buffer, 0, &scroll_slot.bytes);
                stats.property_upload_bytes = stats
                    .property_upload_bytes
                    .saturating_add(scroll_slot.bytes.len());
                stats.scroll_property_upload_bytes = stats
                    .scroll_property_upload_bytes
                    .saturating_add(scroll_slot.bytes.len());
                stats.property_write_ranges = stats.property_write_ranges.saturating_add(1);
                record_property_full_transfer(stats, PropertyFullReason::Dense);
            }
            scroll_slot.property_serial = Some(properties.serial());
            return slot;
        }

        let mut bytes = vec![0_u8; required * self.scroll_property_stride];
        for (index, binding) in scroll_bindings.iter().copied().enumerate() {
            let dependency_count = commit
                .spatial_topology()
                .scroll_path_owners(binding.path)
                .map_or(0, |owners| owners.len());
            stats.property_index_lookups = stats
                .property_index_lookups
                .saturating_add(dependency_count);
            stats.property_value_visits =
                stats.property_value_visits.saturating_add(dependency_count);
            let translation = commit
                .spatial_topology()
                .scroll_path_translation(binding.path, properties)
                .unwrap_or_default();
            let property = ScrollProperty {
                translation,
                ..ScrollProperty::IDENTITY
            };
            let offset = index * self.scroll_property_stride;
            bytes[offset..offset + std::mem::size_of::<ScrollProperty>()]
                .copy_from_slice(bytemuck::bytes_of(&property));
        }

        if let Some(slot) = self
            .scroll_property_slots
            .iter_mut()
            .position(|slot| slot.bindings == scroll_bindings && slot.bytes == bytes)
        {
            add_property_owner(&mut self.scroll_property_slots[slot].owners, commit);
            self.scroll_property_slots[slot].property_serial = Some(properties.serial());
            return slot;
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
        let initialized = scroll_slot.bytes.is_empty();
        let topology_replaced = !initialized
            && (scroll_slot.bindings != scroll_bindings || scroll_slot.bytes.len() != bytes.len());
        let generation_resync = !initialized
            && !topology_replaced
            && scroll_slot.property_serial != properties.predecessor_serial();
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
        if buffer_recreated || scroll_slot.bytes != bytes {
            render_context
                .queue()
                .write_buffer(&scroll_slot.buffer, 0, &bytes);
            stats.property_upload_bytes += bytes.len();
            stats.scroll_property_upload_bytes += bytes.len();
            stats.property_write_ranges = stats.property_write_ranges.saturating_add(1);
            let reason = if buffer_recreated {
                PropertyFullReason::BufferReplacement
            } else if initialized {
                PropertyFullReason::Initialization
            } else if topology_replaced {
                PropertyFullReason::TopologyReplacement
            } else if generation_resync {
                PropertyFullReason::GenerationResync
            } else {
                PropertyFullReason::Dense
            };
            record_property_full_transfer(stats, reason);
        }
        scroll_slot.owners.clear();
        scroll_slot.owners.push(Arc::downgrade(commit));
        scroll_slot.property_serial = Some(properties.serial());
        scroll_slot.bindings = scroll_bindings.to_vec();
        scroll_slot.bytes = bytes;

        slot
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
            .saturating_add(self.recycled.len())
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
        let mut removed = 0_usize;
        for key in expired {
            if let Some(mut entry) = self.entries.remove(&key) {
                entry.owners.clear();
                if self.recycled.len() < RETAINED_SHAPE_RECYCLE_RESERVE {
                    self.recycled.push(entry);
                } else {
                    if let Some(range) = entry.range {
                        self.release(range);
                    }
                    removed = removed.saturating_add(1);
                }
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
            resource_removals: removed
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
        property_serial: None,
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
        property_serial: None,
        buffer,
        capacity,
        bind_group,
        bindings: vec![ScrollBinding::IDENTITY],
        bytes: vec![0; stride],
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

fn property_slot_exclusively_owned_by(
    owners: &[Weak<scene::Commit>],
    commit: &Arc<scene::Commit>,
) -> bool {
    let mut owns_commit = false;
    for owner in owners.iter().filter_map(Weak::upgrade) {
        if Arc::ptr_eq(&owner, commit) {
            owns_commit = true;
        } else {
            return false;
        }
    }
    owns_commit
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

    fn text_batching_commit(second_quad: crate::geometry::Rect) -> Arc<scene::Commit> {
        let size = crate::geometry::Size::new(80, 40);
        let clear = scene::Color::rgba(0, 0, 0, 0);
        let mut next_id = 1_u64;
        let first = composition::tree::NodeId::layout(&mut next_id);
        let second = composition::tree::NodeId::layout(&mut next_id);
        let mut builder = scene::CommitBuilder::new(size, clear);
        builder.register(
            first,
            None,
            composition::tree::ContentRevision::INITIAL,
            crate::geometry::Rect::new(0, 0, 20, 20),
        );
        builder.register(
            second,
            None,
            composition::tree::ContentRevision::INITIAL,
            crate::geometry::Rect::new(20, 0, 20, 20),
        );
        builder.push_projected_content(
            first,
            scene::Content::Quad(scene::Quad::test_new(
                crate::geometry::Rect::new(0, 0, 20, 20),
                scene::Color::rgba(20, 20, 20, 255),
            )),
            scene::ContentProjection::Normal,
        );
        builder.push_projected_content(
            first,
            scene::Content::Text(scene::Text::test_new(
                crate::geometry::Rect::new(2, 2, 16, 16),
                "first",
                scene::Color::rgba(255, 255, 255, 255),
                scene::TextWrap::None,
            )),
            scene::ContentProjection::Normal,
        );
        builder.push_projected_content(
            second,
            scene::Content::Quad(scene::Quad::test_new(
                second_quad,
                scene::Color::rgba(30, 30, 30, 255),
            )),
            scene::ContentProjection::Normal,
        );
        builder.push_projected_content(
            second,
            scene::Content::Text(scene::Text::test_new(
                crate::geometry::Rect::new(22, 2, 16, 16),
                "second",
                scene::Color::rgba(255, 255, 255, 255),
                scene::TextWrap::None,
            )),
            scene::ContentProjection::Normal,
        );
        let mut retained = HashMap::new();
        builder.finish(None, &mut retained).unwrap()
    }

    fn ordered_content_kinds(commit: &Arc<scene::Commit>) -> Vec<&'static str> {
        let viewport =
            render::Viewport::from_logical_area(crate::geometry::area::logical(80.0, 40.0), 1.0);
        let projection = Projection {
            origin: [0.0, 0.0],
            material: scene::MaterialProjection::Source,
        };
        ordered_draws_for_text_batching(commit, viewport, &projection)
            .unwrap()
            .iter()
            .filter_map(|ordered| {
                let scene::Draw::Content { node, index, .. } = ordered.draw else {
                    return None;
                };
                match commit.node(node)?.content().get(index)? {
                    scene::Content::Quad(_) => Some("quad"),
                    scene::Content::Text(_) => Some("text"),
                    _ => Some("other"),
                }
            })
            .collect()
    }

    fn binding(value: u64) -> PropertyBinding {
        let mut value = value;
        PropertyBinding {
            node: composition::tree::NodeId::layout(&mut value),
            space: TargetSpace {
                origin: [0.0, 0.0],
                size: [100.0, 80.0],
                text_origin: [0.0, 0.0],
                text_size: [100.0, 80.0],
                spatial: scene::SpatialBinding::ROOT,
            },
            projection: scene::ContentProjection::Normal,
            scroll: ScrollBinding::IDENTITY,
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

    #[test]
    fn property_binding_uses_the_compiled_spatial_identity() {
        let local = binding(12);

        assert_eq!(local.scroll(), ScrollBinding::IDENTITY);
    }

    #[test]
    fn property_write_ranges_merge_adjacent_bindings_without_spanning_gaps() {
        assert_eq!(
            property_write_ranges(&[1, 2, 3, 7], 256, 64),
            vec![256..832, 1792..1856]
        );
        assert!(property_write_ranges(&[], 256, 64).is_empty());
    }

    #[test]
    fn one_transfer_planner_selects_sparse_and_dense_property_writes() {
        assert_eq!(
            plan_property_transfer(&[], 256, 64, 4_096),
            PropertyTransfer::Unchanged
        );
        assert_eq!(
            plan_property_transfer(&[0], 256, 64, 4_096),
            PropertyTransfer::Sparse {
                ranges: vec![0..64],
                bytes: 64,
            }
        );
        assert_eq!(
            plan_property_transfer(&[1, 2], 256, 64, 4_096),
            PropertyTransfer::Sparse {
                ranges: vec![256..576],
                bytes: 320,
            }
        );
        assert_eq!(
            plan_property_transfer(&[0], 256, 64, 256),
            PropertyTransfer::Dense
        );
    }

    #[test]
    fn text_batch_order_moves_only_non_overlapping_draws() {
        let disjoint = text_batching_commit(crate::geometry::Rect::new(20, 0, 20, 20));
        assert_eq!(
            ordered_content_kinds(&disjoint),
            vec!["quad", "quad", "text", "text"]
        );

        let overlapping = text_batching_commit(crate::geometry::Rect::new(10, 0, 20, 20));
        assert_eq!(
            ordered_content_kinds(&overlapping),
            vec!["quad", "text", "quad", "text"]
        );
    }
}
