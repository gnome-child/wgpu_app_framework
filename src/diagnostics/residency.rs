use std::{cell::Cell, collections::HashMap, rc::Rc, sync::Arc, time::Duration, time::Instant};

use crate::{interaction, scene};

const TEXT_TARGET: &str = "diagnostics.residency.text";
const TABLE_TARGET: &str = "diagnostics.residency.table";
const VIRTUAL_TARGET: &str = "diagnostics.residency.virtual";
const MAX_CANDIDATE_CPU_US_RELEASE: u128 = 100_000;
// A hard crossing must rebuild the exhausted directional runway, not merely
// the first visible rows. The fixture admits at most one bounded runway step;
// provider work must still be proportional to rows newly entering that step.
const MAX_RUNWAY_ENTERING_ROWS_PER_CROSSING: usize = 16;
const TABLE_CELLS_PER_ROW: usize = 3;
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
const MAX_JUMP_GPU_RESOURCE_CHURN: usize = 32;
const MAX_JUMP_ROWS: usize = 32;
const MAX_TABLE_GLYPH_BATCHES: usize = 4;
const MAX_TABLE_TEXT_PREPARES: usize = 4;
const MAX_TABLE_DRAW_CALLS: usize = 56;
const MAX_TABLE_DRAW_PASSES: usize = 10;
const MAX_TABLE_CLIP_BATCHES: usize = 16;
const MAX_TABLE_GPU_RESOURCES: usize = 112;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ResidencyMovement {
    Crossing,
    Jump,
}

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
    integrated_receipt: String,
    environment: crate::render::debug::Environment,
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

    pub fn integrated_receipt(&self) -> &str {
        &self.integrated_receipt
    }

    pub fn environment(&self) -> &crate::render::debug::Environment {
        &self.environment
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

impl crate::list::Model for Rows {
    fn len(&self) -> usize {
        1_000_000
    }

    fn key(&self, index: usize) -> crate::list::Key {
        crate::list::Key::new(index as u64)
    }

    fn index_of(&self, key: crate::list::Key) -> Option<usize> {
        let index = key.value() as usize;
        (index < self.len()).then_some(index)
    }

    fn membership_revision(&self) -> u64 {
        0
    }

    fn changes_since(&self, _revision: u64) -> Vec<crate::list::Change> {
        Vec::new()
    }

    fn item_revision(&self, _index: usize) -> u64 {
        0
    }
}

impl crate::list::Factory for Rows {
    fn revision(&self) -> u64 {
        0
    }

    fn bind(&self, _slot: crate::list::Slot, index: usize) -> crate::view::Node {
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

    fn key(&self, index: usize) -> crate::list::Key {
        crate::list::Key::new(index as u64)
    }

    fn index_of(&self, key: crate::list::Key) -> Option<usize> {
        let index = key.value() as usize;
        (index < self.len()).then_some(index)
    }

    fn item_revision(&self, _index: usize) -> u64 {
        0
    }

    fn residency_revision(&self) -> Option<u64> {
        Some(0)
    }

    fn cell(&self, row: usize, cell: crate::table::Cell) -> crate::view::Node {
        self.calls.set(self.calls.get().saturating_add(1));
        crate::view::Node::text_area_state(
            crate::view::TextArea::new(format!("{} {row}", cell.column().as_str()))
                .with_focus(crate::session::Focus::table_cell(cell))
                .read_only(),
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
    accepted: (interaction::Offset, interaction::Offset),
    maximum: interaction::Offset,
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
                crate::list::List::new(VIRTUAL_TARGET, 24, rows.clone(), rows.clone())
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

pub async fn compare_table_runway_property_text(
    scale_factor: f32,
) -> Result<crate::renderer_debug::Work, String> {
    if !scale_factor.is_finite() || scale_factor <= 0.0 {
        return Err("table runway scale must be finite and positive".to_owned());
    }
    let size = crate::geometry::Size::new(360, 180);
    let (mut app, _) = fixture(ResidencyPayload::Table);
    let window = app.session().windows()[0].id();
    let initial = app
        .render_scene(window, size)
        .ok_or_else(|| "typed table runway fixture produced no initial candidate".to_owned())?;
    let baseline = snapshot(&initial)?;
    let projection = initial
        .layout()
        .scroll_projections()
        .iter()
        .find(|projection| projection.node() == baseline.node)
        .ok_or_else(|| "typed table runway fixture lost its parent projection".to_owned())?;
    let requested = baseline.accepted.1;
    if requested.y()
        <= baseline
            .properties
            .scroll_offset(baseline.node)
            .unwrap_or_default()
            .y()
    {
        return Err(format!(
            "typed table fixture has no resident property runway: accepted={:?}",
            baseline.accepted
        ));
    }
    let baseline_offset = projection.viewport().resolved_scroll();
    let delta_y = requested.y().saturating_sub(baseline_offset.y());
    let visible = projection.viewport().visible_content();
    let mut regions = Vec::new();
    for row in initial.layout().frames().iter().filter(|frame| {
        frame.provided_row().is_some()
            && initial
                .layout()
                .scene_scroll_path(frame.node_id())
                .contains(&baseline.node)
    }) {
        let translated_row = translate_y(row.rect(), delta_y);
        if rects_overlap(row.rect(), visible) || !rects_overlap(translated_row, visible) {
            continue;
        }
        for text in initial.layout().frames().iter().filter(|frame| {
            frame.role() == crate::view::Role::TextArea && frame.is_descendant_of(row)
        }) {
            let translated = translate_y(text.text_area_text_rect(), delta_y);
            if let Some(clipped) = intersect_rect(translated, visible)
                && clipped.width() >= 8
                && clipped.height() >= 6
            {
                regions.push((
                    row.provided_row().expect("filtered table row").index(),
                    clipped,
                ));
            }
        }
    }
    if regions.len() < 3 {
        return Err(format!(
            "typed table fixture exposed only {} entering text regions at {requested:?}",
            regions.len()
        ));
    }

    let mut harness = crate::render::debug::Harness::new(scale_factor).await?;
    let initial_stats =
        harness.draw_retained_candidate_exact(&baseline.drawable, &baseline.properties)?;
    finish(&mut app, window, &initial, initial_stats);
    drop(initial);

    app.handle_input(
        window,
        crate::Input::scroll_to(baseline.target.clone(), requested),
    )
    .map_err(|error| error.to_string())?;
    let property = app
        .render_scene(window, size)
        .ok_or_else(|| "typed table runway produced no property candidate".to_owned())?;
    let next = snapshot(&property)?;
    if !property.property_only()
        || !Arc::ptr_eq(&baseline.commit, &next.commit)
        || !Arc::ptr_eq(&baseline.drawable, &next.drawable)
        || baseline.residency.revision() != next.residency.revision()
        || next.properties.scroll_offset(next.node) != Some(requested)
    {
        return Err(format!(
            "typed table runway did not remain one resident property transition: property_only={} semantic_same={} drawable_same={} residency={:?}->{:?} expected={requested:?} actual={:?}",
            property.property_only(),
            Arc::ptr_eq(&baseline.commit, &next.commit),
            Arc::ptr_eq(&baseline.drawable, &next.drawable),
            baseline.residency.revision(),
            next.residency.revision(),
            next.properties.scroll_offset(next.node),
        ));
    }
    let (image, work) = harness.draw_retained_candidate_image(&next.drawable, &next.properties)?;
    let mut proven = 0_usize;
    for (row, region) in regions {
        let (ink, samples) = region_ink_pixels(&image, region, scale_factor);
        if ink < 2 {
            return Err(format!(
                "typed table row {row} entered on the first property tick without text ink: region={region:?} samples={samples} non_dominant_pixels={ink}"
            ));
        }
        proven = proven.saturating_add(1);
    }
    if proven < 3 {
        return Err("typed table runway did not prove all three text columns".to_owned());
    }
    if work.scene_node_realization_rebuilds() != 0
        || work.primitive_prepare_calls() != 0
        || work.text_prepare_calls() != 0
        || work.text_shape_calls() != 0
        || work.content_upload_bytes() != 0
        || work.gpu_resource_creations() != 0
        || work.gpu_resource_replacements() != 0
        || work.gpu_resource_removals() != 0
        || work.render_plan_rebuilds() != 0
    {
        return Err(format!(
            "typed table runway property tick performed payload or topology work: {work:?}"
        ));
    }
    Ok(work)
}

fn translate_y(rect: crate::geometry::Rect, delta_y: i32) -> crate::geometry::Rect {
    crate::geometry::Rect::new(
        rect.x(),
        rect.y().saturating_sub(delta_y),
        rect.width(),
        rect.height(),
    )
}

fn rects_overlap(left: crate::geometry::Rect, right: crate::geometry::Rect) -> bool {
    left.x() < right.right()
        && left.right() > right.x()
        && left.y() < right.bottom()
        && left.bottom() > right.y()
}

fn intersect_rect(
    left: crate::geometry::Rect,
    right: crate::geometry::Rect,
) -> Option<crate::geometry::Rect> {
    let x = left.x().max(right.x());
    let y = left.y().max(right.y());
    let right_edge = left.right().min(right.right());
    let bottom = left.bottom().min(right.bottom());
    (right_edge > x && bottom > y).then(|| {
        crate::geometry::Rect::new(x, y, right_edge.saturating_sub(x), bottom.saturating_sub(y))
    })
}

fn region_ink_pixels(
    image: &crate::renderer_debug::Image,
    region: crate::geometry::Rect,
    scale_factor: f32,
) -> (usize, usize) {
    let x0 = ((region.x().saturating_add(1) as f32) * scale_factor)
        .ceil()
        .clamp(0.0, image.width() as f32) as u32;
    let y0 = ((region.y().saturating_add(1) as f32) * scale_factor)
        .ceil()
        .clamp(0.0, image.height() as f32) as u32;
    let x1 = ((region.right().saturating_sub(1) as f32) * scale_factor)
        .floor()
        .clamp(0.0, image.width() as f32) as u32;
    let y1 = ((region.bottom().saturating_sub(1) as f32) * scale_factor)
        .floor()
        .clamp(0.0, image.height() as f32) as u32;
    let mut colors = HashMap::<[u8; 4], usize>::new();
    for y in y0..y1 {
        for x in x0..x1 {
            let pixel = image.pixels()[(y as usize)
                .saturating_mul(image.width() as usize)
                .saturating_add(x as usize)];
            let color = pixel.map(|channel| (channel * 255.0).round().clamp(0.0, 255.0) as u8);
            *colors.entry(color).or_default() += 1;
        }
    }
    let samples = colors.values().copied().sum::<usize>();
    let dominant = colors.values().copied().max().unwrap_or_default();
    (samples.saturating_sub(dominant), samples)
}

pub async fn measure_residency_crossing_work(
    payload: ResidencyPayload,
    scale_factor: f32,
) -> Result<ResidencyCrossingReceipt, String> {
    measure_residency_work(
        payload,
        scale_factor,
        180,
        false,
        ResidencyMovement::Crossing,
    )
    .await
}

pub async fn measure_residency_crossing_work_at_height(
    payload: ResidencyPayload,
    scale_factor: f32,
    physical_height: i32,
) -> Result<ResidencyCrossingReceipt, String> {
    measure_residency_work(
        payload,
        scale_factor,
        physical_height,
        true,
        ResidencyMovement::Crossing,
    )
    .await
}

pub async fn measure_residency_jump_work(
    payload: ResidencyPayload,
    scale_factor: f32,
) -> Result<ResidencyCrossingReceipt, String> {
    measure_residency_work(payload, scale_factor, 180, false, ResidencyMovement::Jump).await
}

async fn measure_residency_work(
    payload: ResidencyPayload,
    scale_factor: f32,
    physical_height: i32,
    warm_crossing: bool,
    movement: ResidencyMovement,
) -> Result<ResidencyCrossingReceipt, String> {
    if !scale_factor.is_finite() || scale_factor <= 0.0 {
        return Err("residency scale must be finite and positive".to_owned());
    }
    if physical_height <= 0 {
        return Err("residency physical height must be positive".to_owned());
    }
    let size = crate::geometry::Size::new(
        (360.0_f32 / scale_factor).round().max(1.0) as i32,
        (physical_height as f32 / scale_factor).round().max(1.0) as i32,
    );
    let (mut app, calls) = fixture(payload);
    let window = app.session().windows()[0].id();
    let mut harness = crate::render::debug::Harness::new(scale_factor).await?;
    let environment = harness.environment();

    let initial = app
        .render_scene(window, size)
        .ok_or_else(|| format!("{} fixture produced no initial candidate", payload.name()))?;
    let mut baseline = snapshot(&initial)?;
    let initial_stats =
        harness.draw_retained_candidate_exact(&baseline.drawable, &baseline.properties)?;
    finish(&mut app, window, &initial, initial_stats);
    drop(initial);

    if warm_crossing || movement == ResidencyMovement::Jump {
        let warm_y =
            if warm_crossing {
                baseline.accepted.1.y().saturating_add(1)
            } else {
                baseline.accepted.1.y().saturating_add(
                    baseline.maximum.y().saturating_sub(baseline.accepted.1.y()) / 3,
                )
            };
        let warm = interaction::Offset::new(0, warm_y);
        app.handle_input(
            window,
            crate::Input::scroll_to(baseline.target.clone(), warm),
        )
        .map_err(|error| error.to_string())?;
        let warm_candidate = app
            .render_scene(window, size)
            .ok_or_else(|| format!("{} jump produced no warm candidate", payload.name()))?;
        let warm_snapshot = snapshot(&warm_candidate)?;
        if warm_candidate.property_only()
            || !Arc::ptr_eq(&baseline.commit, &warm_snapshot.commit)
            || warm_snapshot.residency.revision() <= baseline.residency.revision()
            || warm_snapshot.properties.scroll_offset(warm_snapshot.node) != Some(warm)
            || !warm_snapshot.residency.accepts(warm)
        {
            return Err(format!(
                "{} jump warmup violated the residency-only handoff: property_only={} semantic_same={} baseline_revision={:?} next_revision={:?} requested={warm:?} actual={:?}",
                payload.name(),
                warm_candidate.property_only(),
                Arc::ptr_eq(&baseline.commit, &warm_snapshot.commit),
                baseline.residency.revision(),
                warm_snapshot.residency.revision(),
                warm_snapshot.properties.scroll_offset(warm_snapshot.node),
            ));
        }
        let warm_stats = harness
            .draw_retained_candidate_exact(&warm_snapshot.drawable, &warm_snapshot.properties)?;
        finish(&mut app, window, &warm_candidate, warm_stats);
        drop(warm_candidate);
        baseline = warm_snapshot;
    }

    app.diagnostics_mut(window)
        .ok_or_else(|| "residency fixture lost diagnostics before measurement".to_owned())?
        .begin_renderer_measurement();

    let requested_y = match movement {
        ResidencyMovement::Crossing => baseline.accepted.1.y().saturating_add(1),
        ResidencyMovement::Jump => baseline
            .accepted
            .1
            .y()
            .saturating_add(baseline.maximum.y().saturating_sub(baseline.accepted.1.y()) / 2),
    };
    if requested_y > baseline.maximum.y() {
        return Err(format!(
            "{} fixture has no forward residency {movement:?}: accepted={:?}, maximum={:?}",
            payload.name(),
            baseline.accepted,
            baseline.maximum
        ));
    }
    let requested = interaction::Offset::new(0, requested_y);
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
            "{} {movement:?} candidate exceeded the {MAX_CANDIDATE_CPU_US_RELEASE}us release budget: {candidate_cpu_us}us",
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

    let gpu_resource_churn = match (movement, payload) {
        (ResidencyMovement::Jump, ResidencyPayload::Table | ResidencyPayload::VirtualList) => {
            MAX_JUMP_GPU_RESOURCE_CHURN
        }
        _ => MAX_GPU_RESOURCE_CHURN,
    };
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
            gpu_resource_churn,
        ),
        (
            "GPU resource replacements",
            crossing_stats.retained_gpu_resource_replacements,
            gpu_resource_churn,
        ),
        (
            "GPU resource removals",
            crossing_stats.retained_gpu_resource_removals,
            gpu_resource_churn,
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
    let provider_row_budget = match movement {
        ResidencyMovement::Crossing => MAX_RUNWAY_ENTERING_ROWS_PER_CROSSING,
        ResidencyMovement::Jump => MAX_JUMP_ROWS,
    };
    let provider_call_budget = match payload {
        ResidencyPayload::Text => 0,
        ResidencyPayload::Table => provider_row_budget.saturating_mul(TABLE_CELLS_PER_ROW),
        ResidencyPayload::VirtualList => provider_row_budget,
    };
    if physical_height == 180 && provider_calls > provider_call_budget {
        return Err(format!(
            "{} crossing rebuilt more than its entering rows: budget={provider_call_budget} calls={provider_calls}",
            payload.name(),
        ));
    }
    let crossing = crate::renderer_debug::Work::from(crossing_stats);
    if payload == ResidencyPayload::Table && physical_height == 180 {
        for (field, observed, maximum) in [
            (
                "glyph batches",
                crossing.glyph_batches(),
                MAX_TABLE_GLYPH_BATCHES,
            ),
            (
                "text prepares",
                crossing.text_prepare_calls(),
                MAX_TABLE_TEXT_PREPARES,
            ),
            ("draw calls", crossing.draw_calls(), MAX_TABLE_DRAW_CALLS),
            ("draw passes", crossing.draw_passes(), MAX_TABLE_DRAW_PASSES),
            (
                "clip batches",
                crossing.clip_batches(),
                MAX_TABLE_CLIP_BATCHES,
            ),
            (
                "retained GPU resources",
                crossing.gpu_resource_count(),
                MAX_TABLE_GPU_RESOURCES,
            ),
        ] {
            if observed > maximum {
                return Err(format!(
                    "table renderer batching exceeded the {field} budget {maximum}: {observed}"
                ));
            }
        }
    }

    let follow_y = if requested.y() < next.accepted.1.y() {
        requested.y().saturating_add(1)
    } else {
        requested.y().saturating_sub(1)
    };
    let follow = interaction::Offset::new(0, follow_y);
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

    let integrated_receipt = app
        .diagnostics(window)
        .ok_or_else(|| "residency fixture lost integrated diagnostics".to_owned())?
        .renderer_receipt_text(match movement {
            ResidencyMovement::Crossing => "residency-crossing",
            ResidencyMovement::Jump => "residency-jump",
        });
    if !integrated_receipt.contains("presentation_receipt_complete=true") {
        return Err(format!(
            "{} residency fixture produced an incomplete integrated receipt:\n{integrated_receipt}",
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
        integrated_receipt,
        environment,
    })
}
