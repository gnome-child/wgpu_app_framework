use std::{cell::Cell, rc::Rc, sync::Arc, time::Duration, time::Instant};

use crate::{interaction, scene};

const TEXT_TARGET: &str = "diagnostics.residency.text";
const TABLE_TARGET: &str = "diagnostics.residency.table";
const VIRTUAL_TARGET: &str = "diagnostics.residency.virtual";
const MAX_CANDIDATE_CPU_US_RELEASE: u128 = 100_000;
const MAX_PROVIDER_CALLS: usize = 256;
const MAX_SCENE_NODE_PAINTS: usize = 512;
const MAX_LINE_SHAPE_CALLS: usize = 128;
const MAX_HORIZONTAL_INDEX_SOURCE_BYTES: usize = 262_144;
const MAX_RESIDENT_SOURCE_BYTES: usize = 65_536;
const MAX_PRIMITIVE_PREPARES: usize = 256;
const MAX_TEXT_PREPARES: usize = 256;
const MAX_RENDERER_SHAPES: usize = 128;
const MAX_CONTENT_UPLOAD_BYTES: usize = 65_536;
const MAX_PROPERTY_UPLOAD_BYTES: usize = 65_536;
const MAX_POST_CROSSING_PROPERTY_UPLOAD_BYTES: usize = 4_096;
const MAX_GPU_RESOURCE_CHURN: usize = 256;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ResidencyPayload {
    Text,
    Table,
    VirtualList,
}

impl ResidencyPayload {
    pub const ALL: [Self; 3] = [Self::Text, Self::Table, Self::VirtualList];

    pub const fn name(self) -> &'static str {
        match self {
            Self::Text => "text",
            Self::Table => "table",
            Self::VirtualList => "virtual-list",
        }
    }

    pub fn from_name(name: &str) -> Option<Self> {
        Self::ALL.into_iter().find(|payload| payload.name() == name)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ResidencyCrossingReceipt {
    payload: ResidencyPayload,
    scale_factor: f32,
    candidate_property_serial: u64,
    candidate_cpu_us: u128,
    provider_calls: usize,
    crossing: crate::renderer_debug::Work,
    post_crossing_property: crate::renderer_debug::Work,
    trace: String,
}

impl ResidencyCrossingReceipt {
    pub fn payload(&self) -> ResidencyPayload {
        self.payload
    }

    pub fn scale_factor(&self) -> f32 {
        self.scale_factor
    }

    pub fn candidate_property_serial(&self) -> u64 {
        self.candidate_property_serial
    }

    pub fn provider_calls(&self) -> usize {
        self.provider_calls
    }

    pub fn candidate_cpu_us(&self) -> u128 {
        self.candidate_cpu_us
    }

    pub fn crossing_work(&self) -> crate::renderer_debug::Work {
        self.crossing
    }

    pub fn post_crossing_property_work(&self) -> crate::renderer_debug::Work {
        self.post_crossing_property
    }

    pub fn trace(&self) -> &str {
        &self.trace
    }
}

#[derive(Clone)]
struct FixtureState {
    document: crate::Document,
}

impl crate::State for FixtureState {}

#[derive(Clone)]
struct Rows {
    calls: Rc<Cell<usize>>,
}

impl crate::virtual_list::Provider for Rows {
    fn len(&self) -> usize {
        1_000_000
    }

    fn key(&self, index: usize) -> crate::virtual_list::Key {
        crate::virtual_list::Key::new(index as u64)
    }

    fn index_of(&self, key: crate::virtual_list::Key) -> Option<usize> {
        let index = key.value() as usize;
        (index < self.len()).then_some(index)
    }

    fn row(&self, index: usize) -> crate::view::Node {
        self.calls.set(self.calls.get().saturating_add(1));
        crate::view::Node::world_text(
            format!("Residency row {index}"),
            crate::text::Overflow::EllipsisEnd,
        )
    }
}

#[derive(Clone)]
struct TableRows {
    calls: Rc<Cell<usize>>,
}

impl crate::table::Provider for TableRows {
    fn len(&self) -> usize {
        1_000_000
    }

    fn key(&self, index: usize) -> crate::virtual_list::Key {
        crate::virtual_list::Key::new(index as u64)
    }

    fn index_of(&self, key: crate::virtual_list::Key) -> Option<usize> {
        let index = key.value() as usize;
        (index < self.len()).then_some(index)
    }

    fn cell(&self, row: usize, cell: crate::table::Cell) -> crate::view::Node {
        self.calls.set(self.calls.get().saturating_add(1));
        crate::view::Node::world_text(
            format!("{} {row}", cell.column().as_str()),
            crate::text::Overflow::EllipsisEnd,
        )
    }
}

struct Snapshot {
    commit: Arc<scene::Commit>,
    drawable: Arc<scene::Commit>,
    properties: scene::Properties,
    residency: scene::Residency,
    node: crate::composition::tree::NodeId,
    target: interaction::Target,
    accepted: (interaction::ScrollOffset, interaction::ScrollOffset),
    maximum: interaction::ScrollOffset,
}

fn fixture(
    payload: ResidencyPayload,
) -> (
    crate::Runtime<FixtureState, (), crate::View>,
    Rc<Cell<usize>>,
) {
    let calls = Rc::new(Cell::new(0));
    let rows = Rows {
        calls: Rc::clone(&calls),
    };
    let table_rows = TableRows {
        calls: Rc::clone(&calls),
    };
    let text = (0..8_192)
        .map(|line| {
            format!("resident text line {line:05} alpha beta gamma delta epsilon zeta eta theta")
        })
        .collect::<Vec<_>>()
        .join("\n");
    let state = FixtureState {
        document: crate::Document::from_multiline_text(text),
    };
    let mut app = crate::Runtime::new(state)
        .started(|cx| {
            cx.open_window(crate::window::Options::new("Residency GPU witness"));
        })
        .view(move |state, _| match payload {
            ResidencyPayload::Text => crate::widget::view_node(
                crate::widget::TextArea::from_document(&state.document)
                    .id(TEXT_TARGET)
                    .wrap(crate::view::Wrap::None),
            ),
            ResidencyPayload::Table => crate::widget::view_node(
                crate::Table::new(
                    TABLE_TARGET,
                    24,
                    [
                        crate::table::Column::new(
                            "name",
                            "Name",
                            crate::view::Dimension::fixed(120),
                        ),
                        crate::table::Column::new(
                            "detail",
                            "Detail",
                            crate::view::Dimension::fixed(180),
                        ),
                        crate::table::Column::new(
                            "status",
                            "Status",
                            crate::view::Dimension::fixed(100),
                        ),
                    ],
                    table_rows.clone(),
                )
                .height(crate::view::Dimension::grow()),
            ),
            ResidencyPayload::VirtualList => crate::widget::view_node(
                crate::virtual_list::VirtualList::new(VIRTUAL_TARGET, 24, rows.clone())
                    .width(crate::view::Dimension::grow())
                    .height(crate::view::Dimension::grow()),
            ),
        });
    app.start();
    (app, calls)
}

fn snapshot(presentation: &scene::Presentation) -> Result<Snapshot, String> {
    let projection = presentation
        .layout()
        .scroll_projections()
        .iter()
        .filter(|projection| projection.viewport().max_scroll().y() > 0)
        .max_by_key(|projection| projection.viewport().max_scroll().y())
        .ok_or_else(|| "residency fixture has no vertically scrollable viewport".to_owned())?;
    let node = projection.node();
    let residency = presentation
        .stack()
        .base()
        .residencies()
        .iter()
        .find(|residency| residency.scroll() == node)
        .cloned()
        .ok_or_else(|| "residency fixture has no drawable residency".to_owned())?;
    Ok(Snapshot {
        commit: Arc::clone(presentation.commit()),
        drawable: Arc::clone(presentation.stack().base().drawable_commit()),
        properties: presentation.properties().clone(),
        residency,
        node,
        target: projection.target().clone(),
        accepted: projection
            .accepted_offsets()
            .ok_or_else(|| "residency fixture has no accepted interval".to_owned())?,
        maximum: projection.viewport().max_scroll(),
    })
}

fn report(stats: crate::render::DrawStats) -> crate::render::RenderReport {
    crate::render::RenderReport::new(Duration::ZERO, Duration::ZERO, Instant::now())
        .with_draw_stats(stats)
}

fn finish(
    app: &mut crate::Runtime<FixtureState, (), crate::View>,
    window: crate::window::Id,
    presentation: &scene::Presentation,
    stats: crate::render::DrawStats,
) {
    let _retry_requested = app.finish_render_report(
        window,
        presentation.epoch(),
        presentation.invalidation(),
        presentation.layout(),
        presentation.stack(),
        presentation.property_only(),
        report(stats),
    );
}

fn trace_field<'a>(trace: &'a str, field: &str) -> Result<&'a str, String> {
    trace
        .split(',')
        .find_map(|entry| entry.strip_prefix(&format!("{field}=")))
        .ok_or_else(|| format!("residency trace omitted {field}: {trace}"))
}

fn require_numeric_trace_field(trace: &str, field: &str) -> Result<(), String> {
    let value = trace_field(trace, field)?;
    if value == "none" || value.parse::<usize>().is_err() {
        return Err(format!(
            "residency trace field {field} is not generation-attributed: {value}"
        ));
    }
    Ok(())
}

fn trace_count(trace: &str, field: &str) -> Result<usize, String> {
    trace_field(trace, field)?
        .parse::<usize>()
        .map_err(|_| format!("residency trace field {field} is not numeric: {trace}"))
}

pub async fn measure_residency_crossing_work(
    payload: ResidencyPayload,
    scale_factor: f32,
) -> Result<ResidencyCrossingReceipt, String> {
    if !scale_factor.is_finite() || scale_factor <= 0.0 {
        return Err("residency scale must be finite and positive".to_owned());
    }
    let size = crate::geometry::Size::new(
        (360.0_f32 / scale_factor).round().max(1.0) as i32,
        (180.0_f32 / scale_factor).round().max(1.0) as i32,
    );
    let (mut app, calls) = fixture(payload);
    let window = app.session().windows()[0].id();
    let mut harness = crate::render::debug::Harness::new(scale_factor).await?;

    let initial = app
        .render_scene(window, size)
        .ok_or_else(|| format!("{} fixture produced no initial candidate", payload.name()))?;
    let baseline = snapshot(&initial)?;
    let initial_stats =
        harness.draw_retained_candidate_exact(&baseline.drawable, &baseline.properties)?;
    finish(&mut app, window, &initial, initial_stats);
    drop(initial);

    let requested_y = baseline.accepted.1.y().saturating_add(1);
    if requested_y > baseline.maximum.y() {
        return Err(format!(
            "{} fixture has no forward residency crossing: accepted={:?}, maximum={:?}",
            payload.name(),
            baseline.accepted,
            baseline.maximum
        ));
    }
    let requested = interaction::ScrollOffset::new(0, requested_y);
    let calls_before = calls.get();
    let candidate_started = Instant::now();
    app.handle_input(
        window,
        crate::Input::scroll_to(baseline.target.clone(), requested),
    )
    .map_err(|error| error.to_string())?;

    let candidate = app
        .render_scene(window, size)
        .ok_or_else(|| format!("{} crossing produced no candidate", payload.name()))?;
    let candidate_cpu_us = candidate_started.elapsed().as_micros();
    if !cfg!(debug_assertions) && candidate_cpu_us > MAX_CANDIDATE_CPU_US_RELEASE {
        return Err(format!(
            "{} crossing candidate exceeded the {MAX_CANDIDATE_CPU_US_RELEASE}us release budget: {candidate_cpu_us}us",
            payload.name()
        ));
    }
    let next = snapshot(&candidate)?;
    if candidate.property_only()
        || !Arc::ptr_eq(&baseline.commit, &next.commit)
        || next.residency.revision() <= baseline.residency.revision()
        || next.properties.scroll_offset(next.node) != Some(requested)
        || !next.residency.accepts(requested)
    {
        return Err(format!(
            "{} crossing violated the residency-only handoff: property_only={} semantic_same={} baseline_revision={:?} next_revision={:?} requested={requested:?} actual={:?}",
            payload.name(),
            candidate.property_only(),
            Arc::ptr_eq(&baseline.commit, &next.commit),
            baseline.residency.revision(),
            next.residency.revision(),
            next.properties.scroll_offset(next.node),
        ));
    }
    let candidate_epoch = candidate.epoch();
    let candidate_serial = candidate.properties().serial().value();
    let crossing_stats = harness.draw_retained_candidate_exact(&next.drawable, &next.properties)?;
    finish(&mut app, window, &candidate, crossing_stats);
    drop(candidate);

    let trace_needle = format!("candidate_property_serial={candidate_serial}");
    let trace = app
        .diagnostics(window)
        .ok_or_else(|| "residency fixture lost diagnostics".to_owned())?
        .scroll
        .trace_receipt_text()
        .lines()
        .find(|line| line.starts_with("scroll_trace_") && line.contains(&trace_needle))
        .ok_or_else(|| format!("missing residency trace for {trace_needle}"))?
        .to_owned();
    if trace_field(&trace, "outcome")? != "needs-residency"
        || trace_field(&trace, "candidate_epoch")? != candidate_epoch.value().to_string()
        || trace_field(&trace, "present_submitted_property_serial")? != candidate_serial.to_string()
    {
        return Err(format!(
            "{} crossing trace does not describe its selected submitted generation: {trace}",
            payload.name()
        ));
    }
    for field in [
        "residency_layout_recomposes",
        "residency_semantic_commits",
        "residency_scene_node_paints",
        "residency_text_line_shape_calls",
        "residency_text_horizontal_index_source_bytes",
        "residency_text_horizontal_window_source_bytes",
        "residency_text_render_source_bytes",
        "residency_primitive_prepare_calls",
        "residency_text_prepare_calls",
        "residency_text_shape_calls",
        "residency_content_upload_bytes",
        "residency_property_upload_bytes",
        "residency_gpu_resource_creations",
        "residency_gpu_resource_replacements",
        "residency_gpu_resource_removals",
    ] {
        require_numeric_trace_field(&trace, field)?;
    }
    if trace_field(&trace, "residency_semantic_commits")? != "0" {
        return Err(format!(
            "{} residency crossing created a semantic commit: {trace}",
            payload.name()
        ));
    }
    for (field, maximum) in [
        ("residency_layout_recomposes", 1),
        ("residency_scene_node_paints", MAX_SCENE_NODE_PAINTS),
        ("residency_text_line_shape_calls", MAX_LINE_SHAPE_CALLS),
        (
            "residency_text_horizontal_index_source_bytes",
            MAX_HORIZONTAL_INDEX_SOURCE_BYTES,
        ),
        (
            "residency_text_horizontal_window_source_bytes",
            MAX_RESIDENT_SOURCE_BYTES,
        ),
        (
            "residency_text_render_source_bytes",
            MAX_RESIDENT_SOURCE_BYTES,
        ),
    ] {
        let observed = trace_count(&trace, field)?;
        if observed > maximum {
            return Err(format!(
                "{} crossing exceeded the {field} viewport/guard budget {maximum}: {observed}",
                payload.name()
            ));
        }
    }
    let render_expectations = [
        (
            "residency_primitive_prepare_calls",
            crossing_stats.quad_prepare_calls,
        ),
        (
            "residency_text_prepare_calls",
            crossing_stats.text_prepare_calls,
        ),
        (
            "residency_text_shape_calls",
            crossing_stats.inline_text_shape_calls,
        ),
        (
            "residency_content_upload_bytes",
            crossing_stats.content_upload_bytes,
        ),
        (
            "residency_property_upload_bytes",
            crossing_stats.property_upload_bytes,
        ),
        (
            "residency_gpu_resource_creations",
            crossing_stats.retained_gpu_resource_creations,
        ),
        (
            "residency_gpu_resource_replacements",
            crossing_stats.retained_gpu_resource_replacements,
        ),
        (
            "residency_gpu_resource_removals",
            crossing_stats.retained_gpu_resource_removals,
        ),
    ];
    for (field, expected) in render_expectations {
        if trace_field(&trace, field)? != expected.to_string() {
            return Err(format!(
                "{} trace field {field} does not match renderer work {expected}: {trace}",
                payload.name()
            ));
        }
    }

    for (field, observed, maximum) in [
        (
            "primitive prepares",
            crossing_stats.quad_prepare_calls,
            MAX_PRIMITIVE_PREPARES,
        ),
        (
            "text prepares",
            crossing_stats.text_prepare_calls,
            MAX_TEXT_PREPARES,
        ),
        (
            "renderer shapes",
            crossing_stats.inline_text_shape_calls,
            MAX_RENDERER_SHAPES,
        ),
        (
            "content upload bytes",
            crossing_stats.content_upload_bytes,
            MAX_CONTENT_UPLOAD_BYTES,
        ),
        (
            "property upload bytes",
            crossing_stats.property_upload_bytes,
            MAX_PROPERTY_UPLOAD_BYTES,
        ),
        (
            "GPU resource creations",
            crossing_stats.retained_gpu_resource_creations,
            MAX_GPU_RESOURCE_CHURN,
        ),
        (
            "GPU resource replacements",
            crossing_stats.retained_gpu_resource_replacements,
            MAX_GPU_RESOURCE_CHURN,
        ),
        (
            "GPU resource removals",
            crossing_stats.retained_gpu_resource_removals,
            MAX_GPU_RESOURCE_CHURN,
        ),
    ] {
        if observed > maximum {
            return Err(format!(
                "{} crossing exceeded the {field} viewport/guard budget {maximum}: {observed}",
                payload.name()
            ));
        }
    }

    let provider_calls = calls.get().saturating_sub(calls_before);
    if provider_calls > MAX_PROVIDER_CALLS {
        return Err(format!(
            "{} crossing exceeded the {MAX_PROVIDER_CALLS}-call payload guard: {provider_calls}",
            payload.name()
        ));
    }
    let crossing = crate::renderer_debug::Work::from(crossing_stats);

    let follow_y = if requested.y() < next.accepted.1.y() {
        requested.y().saturating_add(1)
    } else {
        requested.y().saturating_sub(1)
    };
    let follow = interaction::ScrollOffset::new(0, follow_y);
    app.handle_input(window, crate::Input::scroll_to(next.target.clone(), follow))
        .map_err(|error| error.to_string())?;
    let property = app
        .render_scene(window, size)
        .ok_or_else(|| format!("{} produced no post-crossing property tick", payload.name()))?;
    let property_snapshot = snapshot(&property)?;
    if !property.property_only()
        || !Arc::ptr_eq(&next.drawable, &property_snapshot.drawable)
        || property_snapshot
            .properties
            .scroll_offset(property_snapshot.node)
            != Some(follow)
    {
        return Err(format!(
            "{} post-crossing tick snapped or rebuilt geometry: property_only={} drawable_same={} expected={follow:?} actual={:?}",
            payload.name(),
            property.property_only(),
            Arc::ptr_eq(&next.drawable, &property_snapshot.drawable),
            property_snapshot
                .properties
                .scroll_offset(property_snapshot.node),
        ));
    }
    let property_stats = harness.draw_retained_candidate_exact(
        &property_snapshot.drawable,
        &property_snapshot.properties,
    )?;
    finish(&mut app, window, &property, property_stats);
    let post_crossing_property = crate::renderer_debug::Work::from(property_stats);
    if post_crossing_property.scene_node_realization_rebuilds() != 0
        || post_crossing_property.primitive_prepare_calls() != 0
        || post_crossing_property.text_prepare_calls() != 0
        || post_crossing_property.text_shape_calls() != 0
        || post_crossing_property.content_upload_bytes() != 0
        || post_crossing_property.gpu_resource_creations() != 0
        || post_crossing_property.gpu_resource_replacements() != 0
        || post_crossing_property.gpu_resource_removals() != 0
        || post_crossing_property.render_plan_rebuilds() != 0
        || post_crossing_property.property_upload_bytes() > MAX_POST_CROSSING_PROPERTY_UPLOAD_BYTES
    {
        return Err(format!(
            "{} post-crossing resident tick performed cold work: {post_crossing_property:?}",
            payload.name()
        ));
    }

    Ok(ResidencyCrossingReceipt {
        payload,
        scale_factor,
        candidate_property_serial: candidate_serial,
        candidate_cpu_us,
        provider_calls,
        crossing,
        post_crossing_property,
        trace,
    })
}
