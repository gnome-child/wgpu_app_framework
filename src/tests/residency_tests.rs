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

    fn cell(&self, row: usize, cell: crate::table::Cell) -> view::Node {
        self.calls.set(self.calls.get().saturating_add(1));
        view::Node::world_text(
            format!("{} {row}", cell.column().as_str()),
            text::Overflow::EllipsisEnd,
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
