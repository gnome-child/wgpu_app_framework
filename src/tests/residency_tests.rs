use super::*;

use std::{cell::Cell, rc::Rc, sync::Arc};

const TEXT_TARGET: &str = "residency.text";
const TABLE_TARGET: &str = "residency.table";
const VIRTUAL_TARGET: &str = "residency.virtual";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Payload {
    Text,
    Table,
    VirtualList,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Transition {
    ResidentInterior,
    GuardEdge,
    ForwardCrossing,
    ReverseCrossing,
    LargeJump,
}

#[derive(Clone)]
struct ResidencyState {
    document: TextDocument,
}

impl State for ResidencyState {}

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

    fn item_revision(&self, _index: usize) -> Option<u64> {
        Some(0)
    }

    fn factory_revision(&self) -> Option<u64> {
        Some(0)
    }

    fn row(&self, index: usize) -> view::Node {
        self.calls.set(self.calls.get().saturating_add(1));
        view::Node::world_text(
            format!("Residency row {index}"),
            text::Overflow::EllipsisEnd,
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

    fn item_revision(&self, _index: usize) -> Option<u64> {
        Some(0)
    }

    fn cell(&self, row: usize, cell: crate::table::Cell) -> view::Node {
        self.calls.set(self.calls.get().saturating_add(1));
        view::Node::text_area_state(
            view::TextArea::new(format!("{} {row}", cell.column().as_str()))
                .with_focus(session::Focus::table_cell(cell))
                .read_only(),
        )
    }
}

struct Fixture {
    app: Runtime<ResidencyState, (), View>,
    payload_calls: Rc<Cell<usize>>,
}

#[derive(Clone)]
struct Snapshot {
    commit: Arc<scene::Commit>,
    drawable: Arc<scene::Commit>,
    properties: scene::Properties,
    residency: scene::Residency,
    node: composition::tree::NodeId,
    target: interaction::Target,
    accepted: (interaction::ScrollOffset, interaction::ScrollOffset),
    maximum: interaction::ScrollOffset,
    viewport_height: i32,
}

fn fixture(payload: Payload) -> Fixture {
    let payload_calls = Rc::new(Cell::new(0));
    let rows = Rows {
        calls: Rc::clone(&payload_calls),
    };
    let table_rows = TableRows {
        calls: Rc::clone(&payload_calls),
    };
    let text = (0..8_192)
        .map(|line| {
            format!("resident text line {line:05} alpha beta gamma delta epsilon zeta eta theta")
        })
        .collect::<Vec<_>>()
        .join("\n");
    let state = ResidencyState {
        document: TextDocument::from_multiline_text(text),
    };
    let mut app = Runtime::new(state)
        .started(|cx| {
            cx.open_window(window::Options::new("Residency contract"));
        })
        .view(move |state, _| match payload {
            Payload::Text => widget::view_node(
                widget::TextArea::from_document(&state.document)
                    .id(TEXT_TARGET)
                    .wrap(view::Wrap::None),
            ),
            Payload::Table => widget::view_node(
                crate::Table::new(
                    TABLE_TARGET,
                    24,
                    [
                        crate::table::Column::new("name", "Name", view::Dimension::fixed(120)),
                        crate::table::Column::new("detail", "Detail", view::Dimension::fixed(180)),
                        crate::table::Column::new("status", "Status", view::Dimension::fixed(100)),
                    ],
                    table_rows.clone(),
                )
                .height(view::Dimension::grow()),
            ),
            Payload::VirtualList => widget::view_node(
                crate::VirtualList::new(VIRTUAL_TARGET, 24, rows.clone())
                    .width(view::Dimension::grow())
                    .height(view::Dimension::grow()),
            ),
        });
    app.start();
    Fixture { app, payload_calls }
}

fn logical_size(scale_milli: u32) -> geometry::Size {
    let scale = scale_milli.max(1) as i64;
    geometry::Size::new(
        i32::try_from(360_000_i64 / scale).unwrap_or(360),
        i32::try_from(180_000_i64 / scale).unwrap_or(180),
    )
}

fn snapshot(presentation: &scene::Presentation) -> Snapshot {
    let projection = presentation
        .layout()
        .scroll_projections()
        .iter()
        .filter(|projection| projection.viewport().max_scroll().y() > 0)
        .max_by_key(|projection| projection.viewport().max_scroll().y())
        .expect("fixture must expose one vertically scrollable payload viewport");
    let node = projection.node();
    let residency = presentation
        .stack()
        .base()
        .residencies()
        .iter()
        .find(|residency| residency.scroll() == node)
        .cloned()
        .expect("drawable payload viewport must carry scene residency");
    Snapshot {
        commit: Arc::clone(presentation.commit()),
        drawable: Arc::clone(presentation.stack().base().drawable_commit()),
        properties: presentation.properties().clone(),
        residency,
        node,
        target: projection.target().clone(),
        accepted: projection
            .accepted_offsets()
            .expect("drawable payload viewport must expose an accepted interval"),
        maximum: projection.viewport().max_scroll(),
        viewport_height: projection.viewport().rect().height(),
    }
}

fn prepare_reverse_baseline(
    app: &mut Runtime<ResidencyState, (), View>,
    window: window::Id,
    size: geometry::Size,
    initial: &Snapshot,
) -> Snapshot {
    let middle = interaction::ScrollOffset::new(0, initial.maximum.y() / 2);
    app.handle_input(window, Input::scroll_to(initial.target.clone(), middle))
        .expect("reverse fixture jump should be handled");
    let presentation = app
        .show_scene(window, size)
        .expect("reverse fixture jump should prepare complete residency");
    let prepared = snapshot(&presentation);
    assert!(
        Arc::ptr_eq(&initial.commit, &prepared.commit),
        "reverse baseline created semantic work: {}",
        initial.commit.projection_difference(&prepared.commit)
    );
    assert!(prepared.residency.accepts(middle));
    assert!(prepared.accepted.0.y() > 0);
    prepared
}

fn requested_offset(snapshot: &Snapshot, transition: Transition) -> interaction::ScrollOffset {
    let (minimum, maximum) = snapshot.accepted;
    match transition {
        Transition::ResidentInterior => {
            assert!(maximum.y() >= minimum.y().saturating_add(2));
            interaction::ScrollOffset::new(0, minimum.y().saturating_add(1))
        }
        Transition::GuardEdge => maximum,
        Transition::ForwardCrossing => {
            assert!(maximum.y() < snapshot.maximum.y());
            interaction::ScrollOffset::new(0, maximum.y().saturating_add(1))
        }
        Transition::ReverseCrossing => {
            assert!(minimum.y() > 0);
            interaction::ScrollOffset::new(0, minimum.y().saturating_sub(1))
        }
        Transition::LargeJump => {
            let middle = snapshot.maximum.y() / 2;
            let y = if (minimum.y()..=maximum.y()).contains(&middle) {
                snapshot.maximum.y()
            } else {
                middle
            };
            assert!(!(minimum.y()..=maximum.y()).contains(&y));
            interaction::ScrollOffset::new(0, y)
        }
    }
}

fn semantic_difference(left: &scene::Commit, right: &scene::Commit) -> String {
    let changed = left
        .nodes()
        .iter()
        .filter_map(|node| {
            right
                .nodes()
                .iter()
                .find(|candidate| candidate.id() == node.id())
                .filter(|candidate| candidate.as_ref() != node.as_ref())
                .map(|candidate| format!("left={node:?} right={candidate:?}"))
        })
        .collect::<Vec<_>>();
    format!(
        "{} left_order={:?} right_order={:?} changed={changed:?}",
        left.projection_difference(right),
        left.order(),
        right.order(),
    )
}

fn candidate_trace_line(
    app: &Runtime<ResidencyState, (), View>,
    window: window::Id,
    serial: scene::PropertySerial,
) -> String {
    let needle = format!("candidate_property_serial={}", serial.value());
    app.diagnostics(window)
        .expect("residency trace diagnostics")
        .scroll
        .trace_receipt_text()
        .lines()
        .find(|line| line.starts_with("scroll_trace_") && line.contains(&needle))
        .unwrap_or_else(|| panic!("missing trace for {needle}"))
        .to_owned()
}

fn trace_field<'a>(trace: &'a str, field: &str) -> &'a str {
    trace
        .split(',')
        .find_map(|entry| entry.strip_prefix(&format!("{field}=")))
        .unwrap_or_else(|| panic!("missing {field} in {trace}"))
}

fn run_case(payload: Payload, transition: Transition, scale_milli: u32) {
    let mut fixture = fixture(payload);
    let window = fixture.app.session().windows()[0].id();
    let size = logical_size(scale_milli);
    let initial_presentation = fixture
        .app
        .show_scene(window, size)
        .expect("initial payload residency should prepare");
    let initial = snapshot(&initial_presentation);
    drop(initial_presentation);

    let baseline = if transition == Transition::ReverseCrossing {
        prepare_reverse_baseline(&mut fixture.app, window, size, &initial)
    } else {
        initial
    };
    let requested = requested_offset(&baseline, transition);
    let resident_case = matches!(
        transition,
        Transition::ResidentInterior | Transition::GuardEdge
    );
    let calls_before = fixture.payload_calls.get();
    let diagnostics_before = fixture
        .app
        .diagnostics(window)
        .expect("fixture diagnostics before transition")
        .clone();

    fixture
        .app
        .handle_input(window, Input::scroll_to(baseline.target.clone(), requested))
        .expect("residency request should be handled");
    let interaction_before_candidate = fixture
        .app
        .session()
        .interaction(window)
        .expect("residency fixture interaction")
        .scroll();
    assert_eq!(
        interaction_before_candidate.desired_offset(&baseline.target),
        requested
    );
    if resident_case {
        assert_eq!(
            interaction_before_candidate.resident_offset(&baseline.target),
            requested
        );
    } else {
        assert_eq!(
            interaction_before_candidate.resident_offset(&baseline.target),
            baseline
                .properties
                .scroll_offset(baseline.node)
                .expect("baseline scroll property")
        );
    }

    let candidate = fixture
        .app
        .show_scene(window, size)
        .expect("residency transition should produce a complete candidate");
    let candidate_epoch = candidate.epoch();
    let candidate_serial = candidate.properties().serial();
    let next = snapshot(&candidate);

    assert!(
        Arc::ptr_eq(&baseline.commit, &next.commit),
        "residency-only motion must not create a semantic commit: {}",
        semantic_difference(&baseline.commit, &next.commit)
    );
    assert_eq!(next.properties.scroll_offset(next.node), Some(requested));
    assert!(next.residency.accepts(requested));
    assert!(candidate.layout().scene_residency_is_complete());
    if resident_case {
        assert!(candidate.property_only());
        assert!(Arc::ptr_eq(&baseline.drawable, &next.drawable));
        assert_eq!(baseline.residency.revision(), next.residency.revision());
    } else {
        assert!(!candidate.property_only());
        assert!(next.residency.revision() > baseline.residency.revision());
    }

    assert_eq!(
        fixture
            .app
            .session()
            .interaction(window)
            .expect("interaction after residency receipt")
            .scroll()
            .resident_offset(&next.target),
        requested
    );
    assert_eq!(
        fixture.app.present_submitted_epoch(window),
        Some(candidate_epoch)
    );
    assert_eq!(
        fixture
            .app
            .presented_properties(window)
            .map(scene::Properties::serial),
        Some(candidate_serial)
    );
    assert_eq!(
        fixture
            .app
            .presented_properties(window)
            .and_then(|properties| properties.scroll_offset(next.node)),
        Some(requested)
    );

    let diagnostics_after = fixture
        .app
        .diagnostics(window)
        .expect("fixture diagnostics after transition");
    let trace = candidate_trace_line(&fixture.app, window, candidate_serial);
    assert_eq!(
        trace_field(&trace, "present_submitted_property_serial"),
        candidate_serial.value().to_string()
    );
    assert_eq!(
        diagnostics_after.render.semantic_commits_created,
        diagnostics_before.render.semantic_commits_created
    );
    if resident_case {
        assert_eq!(
            diagnostics_after.scroll.scroll_needs_residency,
            diagnostics_before.scroll.scroll_needs_residency
        );
        assert_eq!(
            diagnostics_after.text.text_area_line_shape_calls,
            diagnostics_before.text.text_area_line_shape_calls
        );
        assert_eq!(fixture.payload_calls.get(), calls_before);
        assert_eq!(trace_field(&trace, "outcome"), "property-tick");
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
            assert_eq!(
                trace_field(&trace, field),
                "none",
                "resident property tick reported cold work in {field}"
            );
        }
    } else {
        assert_eq!(
            diagnostics_after.scroll.scroll_needs_residency,
            diagnostics_before.scroll.scroll_needs_residency + 1
        );
        let payload_calls = fixture.payload_calls.get().saturating_sub(calls_before);
        assert!(
            payload_calls <= 256,
            "crossing materialization must remain viewport bounded, got {payload_calls} provider calls"
        );
        if transition == Transition::ForwardCrossing {
            let entering_budget = match payload {
                Payload::Text => 0,
                Payload::Table => 9,
                Payload::VirtualList => 3,
            };
            assert!(
                payload_calls <= entering_budget,
                "one residency crossing may realize only entering rows: payload={payload:?} budget={entering_budget} calls={payload_calls}"
            );
        }
        let shaped = diagnostics_after
            .text
            .text_area_line_shape_calls
            .saturating_sub(diagnostics_before.text.text_area_line_shape_calls);
        assert!(
            shaped <= 128,
            "crossing text shaping must remain guarded, got {shaped} line shapes"
        );
        let non_root_frames = candidate
            .layout()
            .frames()
            .iter()
            .filter(|frame| frame.role() != view::Role::Root)
            .count();
        assert!(
            non_root_frames <= 512,
            "crossing layout must remain payload bounded, got {non_root_frames} frames"
        );
        assert_eq!(trace_field(&trace, "outcome"), "needs-residency");
        for field in [
            "residency_layout_recomposes",
            "residency_semantic_commits",
            "residency_scene_node_paints",
            "residency_text_line_shape_calls",
            "residency_text_horizontal_index_source_bytes",
            "residency_text_horizontal_window_source_bytes",
            "residency_text_render_source_bytes",
        ] {
            assert_ne!(
                trace_field(&trace, field),
                "none",
                "crossing candidate did not attribute {field}"
            );
        }
        assert_eq!(trace_field(&trace, "residency_semantic_commits"), "0");

        let (minimum, maximum) = next.accepted;
        let follow_y = if requested.y() < maximum.y() {
            requested.y().saturating_add(1)
        } else {
            assert!(requested.y() > minimum.y());
            requested.y().saturating_sub(1)
        };
        let follow = interaction::ScrollOffset::new(0, follow_y);
        drop(candidate);
        fixture
            .app
            .handle_input(window, Input::scroll_to(next.target.clone(), follow))
            .expect("post-crossing property request should be handled");
        let property_tick = fixture
            .app
            .show_scene(window, size)
            .expect("post-crossing property tick should present");
        let property_snapshot = snapshot(&property_tick);
        assert!(property_tick.property_only());
        assert!(Arc::ptr_eq(&next.drawable, &property_snapshot.drawable));
        assert_eq!(
            property_snapshot
                .properties
                .scroll_offset(property_snapshot.node),
            Some(follow),
            "the first post-crossing tick must not snap geometry to another offset"
        );
    }
}

#[test]
fn resident_motion_starts_bounded_replenishment_before_the_hard_edge() {
    for payload in [Payload::Table, Payload::VirtualList] {
        let mut fixture = fixture(payload);
        let window = fixture.app.session().windows()[0].id();
        let size = logical_size(1_000);
        let initial_presentation = fixture
            .app
            .show_scene(window, size)
            .expect("initial payload residency should prepare");
        let initial = snapshot(&initial_presentation);
        let (minimum, maximum) = initial.accepted;
        assert!(maximum.y() < initial.maximum.y());
        let threshold = initial.viewport_height.max(2).saturating_add(1) / 2;
        let soft_y = maximum
            .y()
            .saturating_sub(threshold)
            .max(minimum.y().saturating_add(1));
        let soft = interaction::ScrollOffset::new(0, soft_y);
        assert!(initial.residency.accepts(soft));

        fixture
            .app
            .handle_input(window, Input::scroll_to(initial.target.clone(), soft))
            .expect("soft-threshold property request should be handled");
        let property = fixture
            .app
            .show_scene(window, size)
            .expect("soft-threshold motion must present as a property tick first");
        assert!(property.property_only());
        assert_eq!(
            property.properties().scroll_offset(initial.node),
            Some(soft)
        );

        let latest = interaction::ScrollOffset::new(0, soft.y().saturating_add(1).min(maximum.y()));
        assert!(latest.y() > soft.y());
        fixture
            .app
            .handle_input(window, Input::scroll_to(initial.target.clone(), latest))
            .expect("a newer resident tick should supersede replenishment scheduling");
        let (latest_ticks, _, _) = fixture.app.render_pending(|_| size);
        let latest_tick = latest_ticks
            .last()
            .expect("newer resident motion must retain property-frame priority");
        assert!(latest_tick.property_only());
        assert_eq!(
            latest_tick.properties().scroll_offset(initial.node),
            Some(latest)
        );
        fixture.app.finish_render_report(
            latest_tick.window(),
            latest_tick.epoch(),
            latest_tick.invalidation(),
            latest_tick.layout(),
            latest_tick.stack(),
            latest_tick.property_only(),
            crate::diagnostics::RenderReport::new(
                std::time::Duration::ZERO,
                std::time::Duration::ZERO,
                std::time::Instant::now(),
            ),
        );

        let (presentations, _, _) = fixture.app.render_pending(|_| size);
        let replenishment = presentations
            .last()
            .expect("soft-threshold motion must schedule one proactive residency candidate");
        assert!(!replenishment.property_only());
        let replenished = snapshot(replenishment);
        let provided_rows = replenishment
            .layout()
            .frames()
            .iter()
            .filter_map(|frame| frame.provided_row().map(|row| row.index()))
            .collect::<Vec<_>>();
        let projection_receipt = replenishment
            .layout()
            .scroll_projections()
            .iter()
            .filter(|projection| projection.target() == &replenished.target)
            .map(|projection| {
                (
                    projection.node(),
                    projection.accepted_offsets(),
                    projection.resident_bounds(),
                    projection.viewport().resolved_scroll(),
                )
            })
            .collect::<Vec<_>>();
        assert_eq!(
            replenished.properties.scroll_offset(replenished.node),
            Some(latest)
        );
        assert!(
            replenished.accepted.1.y() > maximum.y(),
            "proactive candidate must move the hard edge ahead of active motion for {payload:?}: old={:?} new={:?} soft={soft:?} rows={:?} projections={:?}",
            initial.accepted,
            replenished.accepted,
            provided_rows,
            projection_receipt,
        );
        assert!(
            replenished.accepted.1.y().saturating_sub(latest.y()) >= initial.viewport_height,
            "proactive candidate must retain at least one forward viewport after the latest property position for {payload:?}"
        );
        assert!(
            provided_rows.len() <= 80,
            "proactive materialization must remain within the declared row cap"
        );
        fixture.app.finish_render_report(
            replenishment.window(),
            replenishment.epoch(),
            replenishment.invalidation(),
            replenishment.layout(),
            replenishment.stack(),
            replenishment.property_only(),
            crate::diagnostics::RenderReport::new(
                std::time::Duration::ZERO,
                std::time::Duration::ZERO,
                std::time::Instant::now(),
            ),
        );

        let beyond_old = interaction::ScrollOffset::new(
            0,
            maximum
                .y()
                .saturating_add(1)
                .min(replenished.accepted.1.y()),
        );
        assert!(beyond_old.y() > maximum.y());
        fixture
            .app
            .handle_input(
                window,
                Input::scroll_to(replenished.target.clone(), beyond_old),
            )
            .expect("motion beyond the old hard edge should be handled");
        let continued = fixture
            .app
            .show_scene(window, size)
            .expect("motion beyond the old edge should remain property-only");
        assert!(continued.property_only());
        assert_eq!(
            continued.properties().scroll_offset(replenished.node),
            Some(beyond_old)
        );
    }
}

#[test]
fn required_large_jump_materializes_the_critical_viewport_before_predictive_runway() {
    for payload in [Payload::Table, Payload::VirtualList] {
        let mut fixture = fixture(payload);
        let window = fixture.app.session().windows()[0].id();
        let size = logical_size(1_000);
        let initial_presentation = fixture
            .app
            .show_scene(window, size)
            .expect("initial payload residency should prepare");
        let initial = snapshot(&initial_presentation);
        let jump = interaction::ScrollOffset::new(0, initial.maximum.y() / 2);
        assert!(!initial.residency.accepts(jump));

        fixture
            .app
            .handle_input(window, Input::scroll_to(initial.target.clone(), jump))
            .expect("required large jump should be handled");
        let candidate = fixture
            .app
            .show_scene(window, size)
            .expect("required large jump should produce one residency candidate");
        let candidate_snapshot = snapshot(&candidate);
        let provided_rows = candidate
            .layout()
            .frames()
            .iter()
            .filter_map(|frame| frame.provided_row())
            .count();
        let visible_rows = (initial.viewport_height.max(1) as usize).div_ceil(24);
        // Two rows of overscan on each side plus one partially intersected row
        // are the complete critical request; directional runway is separate.
        let critical_row_limit = visible_rows.saturating_add(5);

        assert!(!candidate.property_only());
        assert_eq!(
            candidate_snapshot
                .properties
                .scroll_offset(candidate_snapshot.node),
            Some(jump)
        );
        assert!(
            provided_rows <= critical_row_limit,
            "required {payload:?} jump materialized {provided_rows} rows before first presentation; critical viewport limit is {critical_row_limit}"
        );
    }
}

#[test]
fn consecutive_required_crossings_do_not_accumulate_drawable_table_rows() {
    let mut fixture = fixture(Payload::Table);
    let window = fixture.app.session().windows()[0].id();
    let size = logical_size(1_000);
    let initial_presentation = fixture
        .app
        .show_scene(window, size)
        .expect("initial table residency should prepare");
    let initial = snapshot(&initial_presentation);
    drop(initial_presentation);

    let first_offset = interaction::ScrollOffset::new(0, initial.accepted.1.y().saturating_add(1));
    fixture
        .app
        .handle_input(
            window,
            Input::scroll_to(initial.target.clone(), first_offset),
        )
        .expect("first hard-edge crossing should be handled");
    let first = fixture
        .app
        .show_scene(window, size)
        .expect("first required table residency should prepare");
    let first_snapshot = snapshot(&first);
    let first_rows = first
        .layout()
        .frames()
        .iter()
        .filter_map(|frame| frame.provided_row().map(|row| row.index()))
        .collect::<std::collections::BTreeSet<_>>();
    assert!(first_snapshot.residency.accepts(first_offset));
    drop(first);

    let second_offset =
        interaction::ScrollOffset::new(0, first_snapshot.accepted.1.y().saturating_add(1));
    fixture
        .app
        .handle_input(
            window,
            Input::scroll_to(initial.target.clone(), second_offset),
        )
        .expect("second hard-edge crossing should be handled");
    let second = fixture
        .app
        .show_scene(window, size)
        .expect("second required table residency should prepare");
    let second_snapshot = snapshot(&second);
    let second_rows = second
        .layout()
        .frames()
        .iter()
        .filter_map(|frame| frame.provided_row().map(|row| row.index()))
        .collect::<std::collections::BTreeSet<_>>();

    assert!(second_snapshot.residency.accepts(second_offset));
    assert!(
        second_rows.len() <= first_rows.len().saturating_add(2),
        "nearby required crossings must not accumulate old drawable rows: first={first_rows:?} second={second_rows:?}"
    );
}

#[test]
fn control_gallery_required_crossing_does_not_draw_the_retention_cache() {
    let mut app = control_gallery::app(control_gallery::State::default());
    app.start();
    let window = app.session().windows()[0].id();
    let size = geometry::Size::new(1_240, 760);
    let initial_presentation = app
        .show_scene(window, size)
        .expect("control gallery should prepare its initial 500px table viewport");
    let initial = snapshot(&initial_presentation);
    assert_eq!(
        initial.target.element_id(),
        Some(interaction::Id::new("control_gallery.records")),
        "the production-scale residency witness must target the control-gallery table"
    );
    let mut previous_rows = initial_presentation
        .layout()
        .frames()
        .iter()
        .filter_map(|frame| frame.provided_row().map(|row| row.key()))
        .collect::<std::collections::BTreeSet<_>>();
    drop(initial_presentation);

    let mut current = initial;
    let mut final_observation = None;
    for crossing in 0..16 {
        let requested = interaction::ScrollOffset::new(0, current.accepted.1.y().saturating_add(1));
        let before = app
            .diagnostics(window)
            .expect("control-gallery diagnostics before a required crossing")
            .clone();
        app.handle_input(window, Input::scroll_to(current.target.clone(), requested))
            .expect("production-scale hard-edge crossing should be handled");
        let candidate = app
            .show_scene(window, size)
            .expect("production-scale required residency should prepare");
        let next = snapshot(&candidate);
        let after = app
            .diagnostics(window)
            .expect("control-gallery diagnostics after a required crossing");
        assert!(next.residency.accepts(requested));
        let candidate_rows = candidate
            .layout()
            .frames()
            .iter()
            .filter_map(|frame| frame.provided_row().map(|row| row.key()))
            .collect::<std::collections::BTreeSet<_>>();
        if crossing == 15 {
            let entering_rows = candidate_rows.difference(&previous_rows).count();
            final_observation = Some((
                next.clone(),
                candidate_rows.len(),
                entering_rows,
                after
                    .text
                    .text_area_line_shape_calls
                    .saturating_sub(before.text.text_area_line_shape_calls),
                after
                    .render
                    .scene_painted_nodes
                    .saturating_sub(before.render.scene_painted_nodes),
            ));
        }
        previous_rows = candidate_rows;
        current = next;
    }

    let (final_snapshot, provided_rows, entering_rows, line_shapes, scene_paints) =
        final_observation.expect("the final required crossing must be observed");
    let visible_rows = (final_snapshot.viewport_height.max(1) as usize).div_ceil(24);
    // The base request includes two overscan rows on both sides and can retain
    // a small number of pinned/partially intersected rows. This budget is
    // deliberately well below the independent 80-row reuse-cache cap.
    let critical_row_limit = visible_rows.saturating_add(8);

    assert!(
        provided_rows <= critical_row_limit,
        "required table residency must draw only its exact critical rows; retained rows belong to a separate reuse cache: provided={provided_rows} critical_limit={critical_row_limit}"
    );
    assert!(
        entering_rows > 0 && entering_rows < provided_rows,
        "the final control-gallery crossing must contain both reused and newly entering rows: entering={entering_rows} provided={provided_rows}"
    );
    assert!(
        line_shapes <= entering_rows.saturating_mul(4),
        "the four text columns must shape only newly entering row keys: shapes={line_shapes} entering_rows={entering_rows} limit={}",
        entering_rows.saturating_mul(4)
    );
    assert!(
        scene_paints <= critical_row_limit.saturating_mul(8).saturating_add(64),
        "candidate scene work must stay proportional to the critical control-gallery rows: paints={scene_paints} critical_rows={critical_row_limit}"
    );
}

#[test]
fn table_resident_runway_prepares_nested_text_before_parent_property_scroll() {
    let mut fixture = fixture(Payload::Table);
    let window = fixture.app.session().windows()[0].id();
    let size = logical_size(1_000);
    let presentation = fixture
        .app
        .show_scene(window, size)
        .expect("initial typed-table residency should prepare");
    let layout = presentation.layout();
    let projection = layout
        .scroll_projections()
        .iter()
        .filter(|projection| projection.viewport().max_scroll().y() > 0)
        .max_by_key(|projection| projection.viewport().max_scroll().y())
        .expect("typed table must expose a vertical parent viewport");
    let (minimum, maximum) = projection
        .accepted_offsets()
        .expect("typed table parent must expose a resident runway");
    let baseline = projection.viewport().resolved_scroll();
    assert_eq!(minimum.y(), 0);
    assert!(maximum.y() > baseline.y());
    let delta_y = maximum.y().saturating_sub(baseline.y());
    let visible = projection.viewport().visible_content();
    let mut entering_rows = 0_usize;
    let mut entering_text = 0_usize;

    for row in layout
        .frames()
        .iter()
        .filter(|frame| frame.provided_row().is_some())
    {
        let rect = row.rect();
        let translated = geometry::Rect::new(
            rect.x(),
            rect.y().saturating_sub(delta_y),
            rect.width(),
            rect.height(),
        );
        let initially_visible = rect.y() < visible.bottom() && rect.bottom() > visible.y();
        let enters = translated.y() < visible.bottom() && translated.bottom() > visible.y();
        if initially_visible || !enters {
            continue;
        }
        entering_rows += 1;
        for frame in layout
            .frames()
            .iter()
            .filter(|frame| frame.role() == view::Role::TextArea && frame.is_descendant_of(row))
        {
            entering_text += 1;
            assert!(
                frame
                    .text_area_layout()
                    .is_some_and(|text| !text.render_surfaces().is_empty()),
                "runway row {} must prepare its nested text payload before parent motion",
                row.provided_row().expect("filtered row").index(),
            );
            assert!(
                layout.scene_scroll_node_is_drawable(frame.node_id()),
                "runway row {} omitted nested text node {:?} from the drawable commit: projection={:?} frame={:?} clip={:?}",
                row.provided_row().expect("filtered row").index(),
                frame.node_id(),
                layout
                    .scroll_projections()
                    .iter()
                    .find(|projection| projection.node() == frame.node_id()),
                frame.rect(),
                frame.clip(),
            );
        }
    }

    assert!(
        entering_rows > 0,
        "fixture must contain rows that enter on the accepted property runway"
    );
    assert!(
        entering_text >= entering_rows.saturating_mul(3),
        "every entering table row must carry all three nested text cells"
    );
}

fn run_table_absolute_burst(forward: bool) {
    let mut fixture = fixture(Payload::Table);
    let window = fixture.app.session().windows()[0].id();
    let size = logical_size(1_000);
    let initial_presentation = fixture
        .app
        .show_scene(window, size)
        .expect("initial typed-table residency should prepare");
    let initial = snapshot(&initial_presentation);
    drop(initial_presentation);
    let baseline = if forward {
        initial
    } else {
        prepare_reverse_baseline(&mut fixture.app, window, size, &initial)
    };
    let baseline_offset = baseline
        .properties
        .scroll_offset(baseline.node)
        .expect("baseline must carry the table scroll offset");
    let offsets = if forward {
        let first = baseline.accepted.1.y().saturating_add(1);
        [first, first.saturating_add(24), first.saturating_add(96)]
    } else {
        let first = baseline.accepted.0.y().saturating_sub(1);
        [first, first.saturating_sub(24), first.saturating_sub(96)]
    };
    let latest = interaction::ScrollOffset::new(0, offsets[2]);
    assert!(
        !(baseline.accepted.0.y()..=baseline.accepted.1.y()).contains(&latest.y()),
        "burst fixture must cross the active accepted interval"
    );

    for y in offsets {
        fixture
            .app
            .handle_input(
                window,
                Input::scroll_to(
                    baseline.target.clone(),
                    interaction::ScrollOffset::new(0, y),
                ),
            )
            .expect("every absolute burst request should be handled");
    }
    let before_candidate = fixture
        .app
        .session()
        .interaction(window)
        .expect("typed-table burst interaction")
        .scroll();
    assert_eq!(
        before_candidate.desired_offset(&baseline.target),
        latest,
        "the latest burst intent must be the sole desired offset"
    );
    assert_eq!(
        before_candidate.resident_offset(&baseline.target),
        baseline_offset,
        "candidate construction must not leak an unprepared offset"
    );

    let candidate = fixture
        .app
        .show_scene(window, size)
        .expect("one candidate should coalesce the complete absolute burst");
    let candidate_snapshot = snapshot(&candidate);
    let candidate_serial = candidate.properties().serial();
    let mut submitted_serial = candidate_serial;
    let candidate_invalidation = candidate.invalidation();
    assert_eq!(
        candidate_snapshot
            .properties
            .scroll_offset(candidate_snapshot.node),
        Some(latest),
        "the selected residency must carry the final burst offset exactly"
    );
    assert!(candidate_snapshot.residency.accepts(latest));
    assert!(!candidate.property_only());
    drop(candidate);

    if let Some(follow_up) = fixture.app.show_scene(window, size) {
        let follow_up_snapshot = snapshot(&follow_up);
        submitted_serial = follow_up.properties().serial();
        assert!(
            submitted_serial >= candidate_serial,
            "an input-free settle submission must not regress the selected property generation"
        );
        assert_eq!(
            follow_up_snapshot
                .properties
                .scroll_offset(follow_up_snapshot.node),
            Some(latest),
            "an input-free follow-up candidate must not advance through queued burst offsets"
        );
        assert!(
            Arc::ptr_eq(&candidate_snapshot.drawable, &follow_up_snapshot.drawable),
            "an input-free follow-up must reuse the final burst residency: property_only={} semantic_diff={} candidate_residency={:?} follow_up_residency={:?} candidate_accepted={:?} follow_up_accepted={:?} candidate_invalidation={:?} follow_up_invalidation={:?} semantic_ptr={} drawable_nodes={}/{}",
            follow_up.property_only(),
            candidate_snapshot
                .commit
                .projection_difference(&follow_up_snapshot.commit),
            candidate_snapshot.residency.revision(),
            follow_up_snapshot.residency.revision(),
            candidate_snapshot.accepted,
            follow_up_snapshot.accepted,
            candidate_invalidation,
            follow_up.invalidation(),
            Arc::ptr_eq(&candidate_snapshot.commit, &follow_up_snapshot.commit),
            candidate_snapshot.drawable.nodes().len(),
            follow_up_snapshot.drawable.nodes().len(),
        );
    }
    assert_eq!(
        fixture
            .app
            .presented_properties(window)
            .map(scene::Properties::serial),
        Some(submitted_serial),
        "presented geometry must name the last frame actually submitted, including a same-epoch settle frame"
    );
    assert_eq!(
        fixture
            .app
            .presented_properties(window)
            .and_then(|properties| properties.scroll_offset(candidate_snapshot.node)),
        Some(latest),
        "present-submitted scroll state must remain at the final burst offset"
    );
}

#[test]
fn table_forward_absolute_burst_selects_only_the_latest_residency() {
    run_table_absolute_burst(true);
}

#[test]
fn table_reverse_absolute_burst_selects_only_the_latest_residency() {
    run_table_absolute_burst(false);
}

fn run_fast_residency_burst_with_pre_realization_coalescing(payload: Payload) {
    let mut fixture = fixture(payload);
    let window = fixture.app.session().windows()[0].id();
    let size = logical_size(1_000);
    let initial_presentation = fixture
        .app
        .show_scene(window, size)
        .expect("initial payload residency should prepare");
    let initial = snapshot(&initial_presentation);
    drop(initial_presentation);

    let first = interaction::ScrollOffset::new(0, initial.accepted.1.y().saturating_add(1));
    fixture
        .app
        .handle_input(window, Input::scroll_to(initial.target.clone(), first))
        .expect("the first hard-edge request should be handled");
    let in_flight = fixture
        .app
        .render_scene(window, size)
        .expect("the first hard-edge request must select one residency candidate");
    assert!(!in_flight.property_only());
    let in_flight_snapshot = snapshot(&in_flight);
    let prepared_maximum = in_flight_snapshot.accepted.1.y();
    let document_maximum = initial.maximum.y();
    assert!(
        document_maximum > prepared_maximum.saturating_add(12),
        "fixture must have enough unloaded rows for a fast-scroll burst"
    );

    let span = document_maximum.saturating_sub(prepared_maximum);
    let mut immediate_successors = 0_usize;
    let mut latest = first;
    for step in 1_i32..=12 {
        let y =
            prepared_maximum.saturating_add(span.saturating_mul(step).saturating_div(13).max(1));
        latest = interaction::ScrollOffset::new(0, y);
        fixture
            .app
            .handle_input(window, Input::scroll_to(initial.target.clone(), latest))
            .expect("every fast-scroll request should be handled");
        let (presentations, _, _) = fixture.app.render_pending(|_| size);
        immediate_successors += presentations.len();
    }
    assert_eq!(
        immediate_successors, 0,
        "same-urgency cold offsets must coalesce before candidate construction while the selected front is in flight"
    );

    let follow_up_requested = fixture.app.finish_render_report(
        in_flight.window(),
        in_flight.epoch(),
        in_flight.invalidation(),
        in_flight.layout(),
        in_flight.stack(),
        in_flight.property_only(),
        crate::diagnostics::RenderReport::new(
            std::time::Duration::ZERO,
            std::time::Duration::ZERO,
            std::time::Instant::now(),
        ),
    );
    assert!(
        follow_up_requested,
        "requests newer than the selected front must expose one final latest-value redraw when that front retires"
    );

    let follow_up = fixture
        .app
        .render_scene(window, size)
        .expect("completion of the front candidate must schedule one final latest-value follow-up");
    let follow_up_snapshot = snapshot(&follow_up);
    assert!(!follow_up.property_only());
    assert_eq!(
        follow_up_snapshot
            .properties
            .scroll_offset(follow_up_snapshot.node),
        Some(latest),
        "the final follow-up candidate must jump directly to the final coalesced intent"
    );
    assert!(follow_up_snapshot.residency.accepts(latest));
    let scroll = &fixture
        .app
        .diagnostics(window)
        .expect("fast-scroll diagnostics")
        .scroll;
    assert_eq!(scroll.scroll_residency_candidates_scheduled, 1);
    assert_eq!(scroll.scroll_residency_candidates_coalesced, 12);
    assert_eq!(scroll.scroll_residency_candidates_selected, 2);
    assert_eq!(scroll.scroll_residency_follow_ups, 1);
}

#[test]
fn fast_residency_burst_coalesces_before_candidate_construction() {
    for payload in [Payload::Text, Payload::Table, Payload::VirtualList] {
        run_fast_residency_burst_with_pre_realization_coalescing(payload);
    }
}

#[test]
fn late_selected_residency_completion_schedules_the_latest_follow_up() {
    let mut fixture = fixture(Payload::Table);
    let window = fixture.app.session().windows()[0].id();
    let size = logical_size(1_000);
    let initial_presentation = fixture
        .app
        .show_scene(window, size)
        .expect("initial table residency should prepare");
    let initial = snapshot(&initial_presentation);
    drop(initial_presentation);

    let first = interaction::ScrollOffset::new(0, initial.accepted.1.y().saturating_add(1));
    fixture
        .app
        .handle_input(window, Input::scroll_to(initial.target.clone(), first))
        .expect("the first hard-edge request should be handled");
    let selected = fixture
        .app
        .render_scene(window, size)
        .expect("the first hard-edge request must select one residency candidate");
    assert!(!selected.property_only());
    let selected_snapshot = snapshot(&selected);
    let span = initial
        .maximum
        .y()
        .saturating_sub(selected_snapshot.accepted.1.y());
    let latest = interaction::ScrollOffset::new(
        0,
        selected_snapshot
            .accepted
            .1
            .y()
            .saturating_add(span.saturating_div(2).max(1)),
    );
    assert!(latest.y() > selected_snapshot.accepted.1.y());

    fixture
        .app
        .handle_input(window, Input::scroll_to(initial.target.clone(), latest))
        .expect("the latest absolute request should coalesce behind the selected front");
    let (coalesced_presentations, _, _) = fixture.app.render_pending(|_| size);
    assert!(
        coalesced_presentations.is_empty(),
        "coalesced intent must not construct an immediate required successor"
    );

    fixture
        .app
        .handle_input(
            window,
            Input::pointer_move(Some(interaction::Target::label(
                "residency.overtaking-frame",
                "Overtaking frame",
            ))),
        )
        .expect("an unrelated paint request should be handled");
    let (mut overtaking_presentations, _, _) = fixture.app.render_pending(|_| size);
    let overtaking = overtaking_presentations
        .pop()
        .expect("the unrelated request should author a newer presentation epoch");
    assert!(overtaking.epoch() > selected.epoch());
    fixture.app.finish_render_report(
        overtaking.window(),
        overtaking.epoch(),
        overtaking.invalidation(),
        overtaking.layout(),
        overtaking.stack(),
        overtaking.property_only(),
        crate::diagnostics::RenderReport::new(
            std::time::Duration::ZERO,
            std::time::Duration::ZERO,
            std::time::Instant::now(),
        ),
    );
    assert_eq!(
        fixture.app.present_submitted_epoch(window),
        Some(overtaking.epoch())
    );

    assert!(
        fixture.app.finish_render_report(
            selected.window(),
            selected.epoch(),
            selected.invalidation(),
            selected.layout(),
            selected.stack(),
            selected.property_only(),
            crate::diagnostics::RenderReport::new(
                std::time::Duration::ZERO,
                std::time::Duration::ZERO,
                std::time::Instant::now(),
            ),
        ),
        "a successfully submitted selected residency must retire by its own identity even when a newer active-compatible frame already advanced present-submitted state"
    );
    assert_eq!(
        fixture.app.present_submitted_epoch(window),
        Some(overtaking.epoch()),
        "scheduler retirement must not let the older selected epoch replace newer presented interaction state"
    );
    let (mut follow_up_presentations, _, _) = fixture.app.render_pending(|_| size);
    let follow_up = follow_up_presentations
        .pop()
        .expect("selected-front retirement must author one final latest-intent candidate");
    let follow_up_snapshot = snapshot(&follow_up);
    assert_eq!(
        follow_up_snapshot
            .properties
            .scroll_offset(follow_up_snapshot.node),
        Some(latest)
    );
    assert!(follow_up_snapshot.residency.accepts(latest));
}

#[test]
fn virtual_materialization_request_cannot_false_converge_a_stale_composition() {
    let mut fixture = fixture(Payload::Table);
    let window = fixture.app.session().windows()[0].id();
    let size = geometry::Size::new(811, 1_075);
    let initial_presentation = fixture
        .app
        .show_scene(window, size)
        .expect("initial table residency should prepare");
    let initial = snapshot(&initial_presentation);
    drop(initial_presentation);

    let jump = interaction::ScrollOffset::new(0, initial.maximum.y() / 3);
    fixture
        .app
        .handle_input(window, Input::scroll_to(initial.target, jump))
        .expect("large table jump should install its requested materialization");

    let layout = fixture
        .app
        .compose_layout_without_view_rebuild_for_test(window, size)
        .expect(
            "layout refinement must detect that requested materialization has not entered the composition and rebuild it",
        );
    assert!(layout.scene_residency_is_complete());
}

#[test]
fn stale_presented_table_click_during_large_absolute_jump_converges() {
    let mut fixture = fixture(Payload::Table);
    let window = fixture.app.session().windows()[0].id();
    let size = logical_size(1_000);
    let initial_presentation = fixture
        .app
        .show_scene(window, size)
        .expect("initial table residency should prepare");
    let initial = snapshot(&initial_presentation);
    let cell = initial_presentation
        .layout()
        .frames()
        .iter()
        .find(|frame| frame.table_cell().is_some() && frame.rect().width() > 8)
        .expect("the presented table must expose a clickable cell")
        .rect();
    let presented_point = geometry::Point::new(
        cell.x().saturating_add(4),
        cell.y().saturating_add(cell.height().saturating_div(2)),
    );
    drop(initial_presentation);

    let edge = interaction::ScrollOffset::new(0, initial.accepted.1.y().saturating_add(1));
    fixture
        .app
        .handle_input(window, Input::scroll_to(initial.target.clone(), edge))
        .expect("first hard-edge table request should be handled");
    let in_flight = fixture
        .app
        .render_scene(window, size)
        .expect("the first hard-edge request should select a residency candidate");
    assert!(!in_flight.property_only());

    let jump = interaction::ScrollOffset::new(0, initial.maximum.y() / 2);
    assert!(!((initial.accepted.0.y())..=initial.accepted.1.y()).contains(&jump.y()));
    fixture
        .app
        .handle_input(window, Input::scroll_to(initial.target.clone(), jump))
        .expect("large absolute table jump should coalesce behind the in-flight candidate");
    fixture
        .app
        .pointer_down_at(window, size, presented_point)
        .expect(
            "the still-presented table cell should remain interactive while the jump is pending",
        );

    fixture.app.finish_render_report(
        in_flight.window(),
        in_flight.epoch(),
        in_flight.invalidation(),
        in_flight.layout(),
        in_flight.stack(),
        in_flight.property_only(),
        crate::diagnostics::RenderReport::new(
            std::time::Duration::ZERO,
            std::time::Duration::ZERO,
            std::time::Instant::now(),
        ),
    );

    let candidate = fixture
        .app
        .show_scene(window, size)
        .expect("stale presented interaction must not trap virtual residency refinement");
    assert!(candidate.layout().scene_residency_is_complete());
}

#[test]
fn stale_presented_table_click_after_reversed_large_jump_converges() {
    let mut fixture = fixture(Payload::Table);
    let window = fixture.app.session().windows()[0].id();
    let size = geometry::Size::new(811, 1_075);
    let initial_presentation = fixture
        .app
        .show_scene(window, size)
        .expect("initial table residency should prepare");
    let initial = snapshot(&initial_presentation);
    drop(initial_presentation);

    let baseline = interaction::ScrollOffset::new(0, 6_395);
    fixture
        .app
        .handle_input(window, Input::scroll_to(initial.target.clone(), baseline))
        .expect("resident baseline jump should be handled");
    let baseline_presentation = fixture
        .app
        .show_scene(window, size)
        .expect("resident baseline should prepare");
    let baseline_snapshot = snapshot(&baseline_presentation);
    assert!(baseline_snapshot.residency.accepts(baseline));
    let cell = baseline_presentation
        .layout()
        .frames()
        .iter()
        .find(|frame| frame.table_cell().is_some() && frame.rect().width() > 8)
        .expect("the presented table must expose a clickable cell")
        .rect();
    let presented_point = geometry::Point::new(
        cell.x().saturating_add(4),
        cell.y().saturating_add(cell.height().saturating_div(2)),
    );
    drop(baseline_presentation);

    let earlier_forward = interaction::ScrollOffset::new(0, 13_333_086);
    fixture
        .app
        .handle_input(
            window,
            Input::scroll_to(initial.target.clone(), earlier_forward),
        )
        .expect("first large forward table jump should be handled");
    let forward = interaction::ScrollOffset::new(0, 18_893_267);
    fixture
        .app
        .handle_input(window, Input::scroll_to(initial.target.clone(), forward))
        .expect("large forward table jump should be handled");
    let in_flight = fixture
        .app
        .render_scene(window, size)
        .expect("large forward jump should select a residency candidate");
    assert!(!in_flight.property_only());

    let reversed = interaction::ScrollOffset::new(0, 8_283_534);
    fixture
        .app
        .handle_input(window, Input::scroll_to(initial.target.clone(), reversed))
        .expect("large reverse table jump should coalesce behind the in-flight candidate");
    assert_ne!(forward, reversed);
    assert!(fixture.app.finish_render_report(
        in_flight.window(),
        in_flight.epoch(),
        in_flight.invalidation(),
        in_flight.layout(),
        in_flight.stack(),
        in_flight.property_only(),
        crate::diagnostics::RenderReport::new(
            std::time::Duration::ZERO,
            std::time::Duration::ZERO,
            std::time::Instant::now(),
        ),
    ));

    fixture
        .app
        .pointer_down_at(window, size, presented_point)
        .expect("the stale presented row should remain interactive after direction reversal");
    let candidate = fixture
        .app
        .show_scene(window, size)
        .expect("reversed coalesced intent and stale-row selection must converge");
    assert!(
        candidate.layout().scene_residency_is_complete(),
        "reversed coalesced intent must materialize the exact requested rows"
    );
}

macro_rules! residency_cases {
    (
        $payload:ident,
        $resident_interior:ident,
        $guard_edge:ident,
        $forward_crossing:ident,
        $reverse_crossing:ident,
        $large_jump:ident,
        $fractional_crossing:ident
    ) => {
        #[test]
        fn $resident_interior() {
            run_case(Payload::$payload, Transition::ResidentInterior, 1_000);
        }

        #[test]
        fn $guard_edge() {
            run_case(Payload::$payload, Transition::GuardEdge, 1_000);
        }

        #[test]
        fn $forward_crossing() {
            run_case(Payload::$payload, Transition::ForwardCrossing, 1_000);
        }

        #[test]
        fn $reverse_crossing() {
            run_case(Payload::$payload, Transition::ReverseCrossing, 1_000);
        }

        #[test]
        fn $large_jump() {
            run_case(Payload::$payload, Transition::LargeJump, 1_000);
        }

        #[test]
        fn $fractional_crossing() {
            run_case(Payload::$payload, Transition::ForwardCrossing, 1_250);
        }
    };
}

// The names are deliberately source-counted by the architecture gate. Keep the
// suite at exactly 18 cases: six transitions for each payload species.
residency_cases!(
    Text,
    residency_case_text_resident_interior_scale_100,
    residency_case_text_guard_edge_scale_100,
    residency_case_text_forward_crossing_scale_100,
    residency_case_text_reverse_crossing_scale_100,
    residency_case_text_large_jump_scale_100,
    residency_case_text_fractional_scale_crossing_125
);
residency_cases!(
    Table,
    residency_case_table_resident_interior_scale_100,
    residency_case_table_guard_edge_scale_100,
    residency_case_table_forward_crossing_scale_100,
    residency_case_table_reverse_crossing_scale_100,
    residency_case_table_large_jump_scale_100,
    residency_case_table_fractional_scale_crossing_125
);
residency_cases!(
    VirtualList,
    residency_case_virtual_list_resident_interior_scale_100,
    residency_case_virtual_list_guard_edge_scale_100,
    residency_case_virtual_list_forward_crossing_scale_100,
    residency_case_virtual_list_reverse_crossing_scale_100,
    residency_case_virtual_list_large_jump_scale_100,
    residency_case_virtual_list_fractional_scale_crossing_125
);
