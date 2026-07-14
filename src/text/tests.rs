use crate::geometry::{area, point};
use std::cell::RefCell;
use std::ops::{Deref, DerefMut};
use std::rc::Rc;
use std::time::{Duration, Instant};

use super::buffer::{
    Affinity, Cursor, CursorSelection, Mark, Position, Range, TEXT_DOCUMENT_TARGET_LEAF_BYTES,
};
use super::document::{Align, Block, ResolvedTextDirection, Run, Style, Weight};
use super::edit::Editor;
use super::layout::{
    Engine, HighlightStats, Measure, TEXT_AREA_FRAME_MAX_LOGICAL_LINES,
    TEXT_AREA_FRAME_MIN_OVERSCAN_LINES, TEXT_AREA_LINE_DISPLAY_CACHE_CAPACITY,
    TEXT_AREA_RENDER_GUARD_LINES, TEXT_FIELD_CARET_MARGIN, TEXT_LAYOUT_VISUAL_LINE_EPSILON,
    TextAreaSurface, TextLayoutMap, VisualLineGroup, clamp_cursor_in_buffer,
    text_area_estimated_line_height,
};
use super::selection::{self, Motion, PointerKind, State};
use super::surface::{Area, Field};
use super::view::{Preedit, ViewState, Viewport, Visibility};
use super::{Buffer, Color, Document, Edit, edit, layout};

thread_local! {
    static TEST_ENGINE: RefCell<Option<Engine>> = const { RefCell::new(None) };
}

struct TestEngine {
    inner: Option<Engine>,
}

fn engine() -> TestEngine {
    let mut engine = TEST_ENGINE
        .with(|slot| slot.borrow_mut().take())
        .unwrap_or_default();
    engine.reset_for_test();
    TestEngine {
        inner: Some(engine),
    }
}

impl Deref for TestEngine {
    type Target = Engine;

    fn deref(&self) -> &Self::Target {
        self.inner
            .as_ref()
            .expect("test engine should be present while borrowed")
    }
}

impl DerefMut for TestEngine {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.inner
            .as_mut()
            .expect("test engine should be present while mutably borrowed")
    }
}

impl Drop for TestEngine {
    fn drop(&mut self) {
        let Some(engine) = self.inner.take() else {
            return;
        };
        TEST_ENGINE.with(|slot| *slot.borrow_mut() = Some(engine));
    }
}

#[test]
fn text_sources_do_not_import_framework_or_renderer_modules() {
    let text_dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("src")
        .join("text");
    let modules = [
        "app", "ui", "widget", "command", "window", "render", "native", "scratch",
    ];

    assert_text_sources_do_not_import_modules(&text_dir, &modules);
}

fn assert_text_sources_do_not_import_modules(path: &std::path::Path, modules: &[&str]) {
    for entry in std::fs::read_dir(path).expect("text source directory should be readable") {
        let path = entry.expect("text source entry should be readable").path();
        if path.is_dir() {
            assert_text_sources_do_not_import_modules(&path, modules);
            continue;
        }

        if path.extension().and_then(|extension| extension.to_str()) != Some("rs") {
            continue;
        }

        let source = std::fs::read_to_string(&path).expect("text source file should read");
        for module in modules {
            assert!(
                !source_imports_crate_module(&source, module),
                "{} must not import or reference framework/render module {}",
                path.display(),
                module
            );
        }
    }
}

fn source_imports_crate_module(source: &str, module: &str) -> bool {
    if source.contains(&format!("crate::{module}::")) {
        return true;
    }

    source.lines().any(|line| {
        let line = line.trim();
        if line == format!("use crate::{module};")
            || line.starts_with(&format!("use crate::{module}::"))
        {
            return true;
        }

        let Some(grouped) = line
            .strip_prefix("use crate::{")
            .and_then(|line| line.strip_suffix(';'))
        else {
            return false;
        };

        grouped.split(',').any(|segment| {
            let segment = segment.trim();
            let root = segment
                .split_once("::")
                .map_or(segment, |(root, _)| root.trim())
                .split_once(" as ")
                .map_or_else(|| segment, |(root, _)| root.trim());
            root == module
        })
    })
}

fn surface_line_text(surfaces: &[TextAreaSurface], line: usize) -> String {
    surfaces
        .get(line)
        .and_then(|surface| {
            let buffer = surface.buffer.borrow();
            buffer.lines.first().map(|line| line.text().to_owned())
        })
        .unwrap_or_default()
}

fn surface_visual_runs(surfaces: &[TextAreaSurface]) -> usize {
    surfaces
        .iter()
        .map(|surface| surface.buffer.borrow().layout_runs().count())
        .sum()
}

fn visual_group_source_range(
    runs: &[glyphon::cosmic_text::LayoutRun<'_>],
    group: VisualLineGroup,
    source_start: usize,
) -> Option<std::ops::Range<usize>> {
    let mut start = usize::MAX;
    let mut end = 0usize;
    for run in &runs[group.start..group.end] {
        for glyph in run.glyphs {
            start = start.min(source_start + glyph.start.min(glyph.end));
            end = end.max(source_start + glyph.start.max(glyph.end));
        }
    }
    (start < end).then_some(start..end)
}
fn apply_edit(editor: &mut Editor, buffer: &mut Buffer, state: &mut State, edit: Edit) -> bool {
    editor.apply_edit(buffer, state, edit).buffer_changed()
}

fn apply_selection(buffer: &Buffer, state: &mut State, operation: selection::Operation) -> bool {
    selection::apply(buffer, state, operation)
}

fn apply_edit_with_result(
    editor: &mut Editor,
    buffer: &mut Buffer,
    state: &mut State,
    edit: Edit,
) -> edit::Outcome {
    editor.apply_edit(buffer, state, edit)
}

fn apply_selection_with_caret_map(
    buffer: &Buffer,
    state: &mut State,
    operation: selection::Operation,
    caret_map: &mut dyn super::selection::CaretMap,
) -> bool {
    selection::apply_with_caret_map(buffer, state, operation, caret_map)
}

fn position(buffer: &Buffer, state: State) -> Position {
    buffer.position_for_state(state)
}

fn mark(state: State) -> Mark {
    state.cursor()
}

fn cursor(buffer: &Buffer, state: State) -> Cursor {
    buffer.cursor_for_state(state)
}

fn selected_range(buffer: &Buffer, state: State) -> Option<Range> {
    buffer.selected_range_for_state(state)
}

fn selected_text(buffer: &Buffer, state: State) -> Option<String> {
    buffer.selected_text_for_state(state)
}

fn has_selection(buffer: &Buffer, state: State) -> bool {
    buffer.has_non_empty_selection_for_state(state)
}

struct StubCaretMap {
    calls: usize,
    position: Position,
}

impl super::selection::CaretMap for StubCaretMap {
    fn position_for_motion(
        &mut self,
        _buffer: &Buffer,
        _state: super::selection::State,
        motion: Motion,
    ) -> Option<Position> {
        self.calls += 1;
        (motion == Motion::VisualDown).then_some(self.position)
    }
}

#[test]
fn document_stores_block_run_and_style_data() {
    let style = Style::default()
        .with_size(18.0)
        .with_color(Color::RED)
        .with_weight(Weight::Bold);
    let mut block = Block::new(Align::Center);
    block.push_run(Run::new("Label", style));
    let document = Document::from_block(block);

    assert_eq!(document.blocks().len(), 1);
    assert_eq!(document.blocks()[0].align(), Align::Center);
    assert_eq!(document.blocks()[0].runs()[0].text(), "Label");
    assert_eq!(document.blocks()[0].runs()[0].style(), style);
}

#[test]
fn empty_document_is_empty() {
    assert!(Document::new().is_empty());
    assert!(Document::plain("").is_empty());
    assert!(!Document::plain("x").is_empty());
}

#[test]
fn document_color_can_be_overridden() {
    let document = Document::plain("Label").with_color(Color::BLACK);

    assert_eq!(document.blocks()[0].runs()[0].style().color(), Color::BLACK);
}

#[test]
fn document_size_can_be_overridden() {
    let document = Document::plain("Label").with_size(12.5);

    assert_eq!(document.blocks()[0].runs()[0].style().size(), 12.5);
}

#[test]
fn plain_document_keeps_raw_default_style() {
    let document = Document::plain("Label");

    assert_eq!(document.blocks()[0].runs()[0].style(), Style::default());
}

#[test]
fn document_first_style_preserves_empty_style_carrier() {
    let empty_style = Style::default().with_size(13.0);
    let text_style = Style::default().with_size(18.0);
    let mut empty_block = Block::new(Align::Start);
    empty_block.push_run(Run::new("", empty_style));
    let empty_document = Document::from_block(empty_block);

    assert_eq!(empty_document.first_style(), Some(empty_style));

    let mut mixed_block = Block::new(Align::Start);
    mixed_block.push_run(Run::new("", empty_style));
    mixed_block.push_run(Run::new("Label", text_style));
    let mixed_document = Document::from_block(mixed_block);

    assert_eq!(mixed_document.first_style(), Some(text_style));
}

#[test]
fn engine_returns_non_zero_metrics_for_non_empty_text() {
    let mut engine = engine();
    let metrics = engine.measure(&Document::plain("Label"), Measure::unbounded());

    assert!(metrics.width() > 0.0);
    assert!(metrics.height() > 0.0);
    assert_eq!(metrics.line_count(), 1);
}

#[test]
fn longer_text_measures_wider_than_shorter_text() {
    let mut engine = engine();
    let short = engine.measure(&Document::plain("Run"), Measure::unbounded());
    let long = engine.measure(&Document::plain("Run workspace task"), Measure::unbounded());

    assert!(long.width() > short.width());
    assert!(long.height() >= short.height());
}

#[test]
fn cloning_buffer_creates_independent_value_snapshot() {
    let mut editor = Editor::new();
    let buffer = Buffer::from_multiline_text("one\ntwo\nthree");
    let buffer_state = buffer.initial_state();
    let mut clone = buffer.clone();
    let mut clone_state = buffer_state;

    assert_ne!(buffer.id(), clone.id());
    assert_eq!(buffer.revision(), clone.revision());
    assert_eq!(buffer.text(), clone.text());
    assert_eq!(
        buffer.position_for_state(buffer_state),
        clone.position_for_state(clone_state)
    );
    assert!(buffer.shares_text_root_with(&clone));
    assert!(buffer.shares_line_index_root_with(&clone));

    editor.apply_edit(&mut clone, &mut clone_state, Edit::insert("!"));

    assert_eq!(buffer.text(), "one\ntwo\nthree");
    assert_eq!(clone.text(), "one\ntwo\nthree!");
    assert!(!buffer.shares_text_root_with(&clone));
    assert!(!buffer.shares_line_index_root_with(&clone));
}

#[test]
fn buffer_state_can_be_copied_separately_from_buffer_content() {
    let buffer = Buffer::from_multiline_text("alpha beta gamma");
    let mut state = buffer.initial_state();
    apply_selection(
        &buffer,
        &mut state,
        selection::Operation::set_position(Position::new(0)),
    );
    apply_selection(
        &buffer,
        &mut state,
        selection::Operation::pointer(PointerKind::Drag, Position::new("alpha beta".len())),
    );
    let initial_state = state;
    let clone = buffer.clone();
    let mut clone_state = initial_state;

    assert_eq!(
        clone.selected_text_for_state(clone_state).as_deref(),
        Some("alpha beta")
    );

    apply_selection(
        &clone,
        &mut clone_state,
        selection::Operation::set_position(Position::new(0)),
    );

    assert_eq!(state, initial_state);
    assert_ne!(clone_state, initial_state);
    assert_eq!(
        buffer.selected_text_for_state(state).as_deref(),
        Some("alpha beta")
    );
    assert_eq!(clone.selected_text_for_state(clone_state), None);
    assert_eq!(buffer.text(), clone.text());
    assert!(buffer.shares_text_root_with(&clone));
    assert!(buffer.shares_line_index_root_with(&clone));
}

#[test]
fn area_model_owns_buffer_state_separately_from_buffer() {
    let buffer = Buffer::from_multiline_text("alpha beta");
    let mut start_state = buffer.initial_state();
    let end_state = buffer.initial_state();

    apply_selection(
        &buffer,
        &mut start_state,
        selection::Operation::set_position(Position::new(0)),
    );

    let start_area = Area::new(buffer.clone()).with_state(start_state);
    let end_area = Area::new(buffer.clone()).with_state(end_state);
    let mut engine = engine();
    let viewport = area::logical(240.0, 80.0);
    let style = Style::default();
    let now = Instant::now();
    let state = ViewState::new_at(0.0, now);
    let start_caret = engine
        .text_area_paint_layout_for_area_at(&start_area, style, viewport, state.clone(), now)
        .layout()
        .caret()
        .expect("start-state area should paint a caret");
    let end_caret = engine
        .text_area_paint_layout_for_area_at(&end_area, style, viewport, state, now)
        .layout()
        .caret()
        .expect("end-state area should paint a caret");

    assert_eq!(buffer.initial_state(), end_state);
    assert!(start_caret.x < end_caret.x);
}

#[test]
fn cloned_buffer_edit_shares_untouched_span_and_line_index_leaves() {
    let mut editor = Editor::new();
    let text = (0..300)
        .map(|index| format!("line {index} {}", "x".repeat(64)))
        .collect::<Vec<_>>()
        .join("\n");
    let buffer = Buffer::from_multiline_text(text);
    let mut clone = buffer.clone();
    let mut clone_state = clone.initial_state();

    assert!(buffer.shares_text_root_with(&clone));
    assert!(buffer.shares_line_index_root_with(&clone));

    apply_selection(
        &clone,
        &mut clone_state,
        selection::Operation::set_position(0),
    );
    apply_edit(&mut editor, &mut clone, &mut clone_state, Edit::insert("!"));

    assert!(buffer.text().starts_with("line 0"));
    assert!(clone.text().starts_with("!line 0"));
    assert!(!buffer.shares_text_root_with(&clone));
    assert!(!buffer.shares_line_index_root_with(&clone));
    assert!(buffer.shared_text_leaf_count(&clone) >= 1);
    assert!(buffer.shared_line_index_leaf_count(&clone) >= 1);
}

#[test]
fn buffer_clone_preserves_line_layout_identity() {
    let mut editor = Editor::new();
    let text = (0..300)
        .map(|index| format!("line {index}"))
        .collect::<Vec<_>>()
        .join("\n");
    let buffer = Buffer::from_multiline_text(text);
    let mut clone = buffer.clone();
    let mut clone_state = clone.initial_state();

    assert_ne!(buffer.id(), clone.id());
    for line in 0..buffer.logical_line_count() {
        assert_eq!(
            buffer.line_layout_identity(line),
            clone.line_layout_identity(line),
            "line {line} identity should survive value clone"
        );
    }

    apply_selection(
        &clone,
        &mut clone_state,
        selection::Operation::set_position(0),
    );
    apply_edit(&mut editor, &mut clone, &mut clone_state, Edit::insert("!"));

    assert_ne!(
        buffer.line_layout_identity(0),
        clone.line_layout_identity(0)
    );
    assert_eq!(
        buffer.line_layout_identity(250),
        clone.line_layout_identity(250)
    );
}

#[test]
fn typing_edit_records_transaction_delta() {
    let mut editor = Editor::new();
    let mut buffer = Buffer::from_multiline_text("one\ntwo\nthree");
    let mut edit_state = buffer.initial_state();
    let before_revision = buffer.revision();
    let result =
        apply_edit_with_result(&mut editor, &mut buffer, &mut edit_state, Edit::insert("!"));
    let change = result.change.expect("typing should produce an undo delta");

    assert!(result.text_changed);
    assert!(buffer.revision() > before_revision);
    assert_eq!(change.transaction.deltas.len(), 1);
    assert_eq!(
        change.transaction.deltas[0].kind,
        edit::TransactionKind::Insert
    );
    assert_eq!(change.transaction.deltas[0].inserted, "!");
}

#[test]
fn text_area_frame_cache_reuses_unchanged_frame_and_rebuilds_after_typing() {
    let mut engine = engine();
    let mut editor = Editor::new();
    let mut buffer = Buffer::from_multiline_text("one\ntwo\nthree");
    let mut edit_state = buffer.initial_state();
    let style = Style::default().with_size(13.0);
    let viewport = area::logical(240.0, 120.0);
    let state = ViewState::default();
    let now = Instant::now();

    let first = engine
        .text_area_paint_layout_for_area_at(
            &Area::new(buffer.clone()),
            style,
            viewport,
            state.clone(),
            now,
        )
        .into_interaction_parts()
        .1;
    let second = engine
        .text_area_paint_layout_for_area_at(
            &Area::new(buffer.clone()),
            style,
            viewport,
            state.clone(),
            now,
        )
        .into_interaction_parts()
        .1;
    assert_eq!(surface_line_text(&first, 2), surface_line_text(&second, 2));
    assert!(!engine.text_area_line_displays.is_empty());

    apply_edit(&mut editor, &mut buffer, &mut edit_state, Edit::insert("!"));
    let third = engine
        .text_area_paint_layout_for_area_at(
            &Area::new(buffer).with_state(edit_state),
            style,
            viewport,
            state,
            now,
        )
        .into_interaction_parts()
        .1;
    assert_eq!(surface_line_text(&third, 2), "three!");
}

#[test]
fn undo_restored_clone_reuses_line_keyed_text_area_caches() {
    let mut engine = engine();
    let mut editor = Editor::new();
    let mut buffer = Buffer::from_multiline_text(
        "one two three four five\nsix seven eight nine ten\neleven twelve thirteen",
    );
    let mut edit_state = buffer.initial_state();
    let undo_snapshot = buffer.clone();
    let style = Style::default().with_size(13.0);
    let viewport = area::logical(240.0, 120.0);
    let content_area = area::logical(240.0, 360.0);
    let state = ViewState::default();
    let now = Instant::now();

    assert_ne!(buffer.id(), undo_snapshot.id());

    engine.text_area_paint_layout_for_area_at(
        &Area::new(buffer.clone()),
        style,
        viewport,
        state.clone(),
        now,
    );
    engine.text_area_render_layout_for_area_at(
        &Area::new(buffer.clone()),
        style,
        viewport,
        state.clone(),
        now,
        content_area,
    );

    apply_selection(
        &buffer,
        &mut edit_state,
        selection::Operation::set_position(0),
    );
    let edit = apply_edit_with_result(
        &mut editor,
        &mut buffer,
        &mut edit_state,
        Edit::insert("edited "),
    );
    assert!(edit.text_changed);
    engine.text_area_paint_layout_for_area_at(
        &Area::new(buffer.clone()).with_state(edit_state),
        style,
        viewport,
        state.clone(),
        now,
    );
    engine.text_area_render_layout_for_area_at(
        &Area::new(buffer).with_state(edit_state),
        style,
        viewport,
        state.clone(),
        now,
        content_area,
    );

    engine.reset_diagnostics();
    engine.text_area_paint_layout_for_area_at(
        &Area::new(undo_snapshot.clone()),
        style,
        viewport,
        state.clone(),
        now,
    );
    let paint = engine.diagnostics();
    assert!(
        paint.text_area_line_cache_hits > 0,
        "undo snapshot with a fresh buffer id should reuse line display caches: {paint:?}"
    );
    assert_eq!(
        paint.text_area_line_shape_calls, 0,
        "line-keyed cache hits should avoid reshaping unchanged undo-restored lines"
    );
    assert_eq!(paint.text_area_shaped_logical_lines, 0);

    engine.reset_diagnostics();
    engine.text_area_render_layout_for_area_at(
        &Area::new(undo_snapshot),
        style,
        viewport,
        state,
        now,
        content_area,
    );
    let render = engine.diagnostics();
    assert_eq!(
        render.text_area_render_surface_cache_hits, 1,
        "undo snapshot with a fresh buffer id should reuse render buffers: {render:?}"
    );
    assert_eq!(render.text_area_render_surface_cache_misses, 0);
    assert_eq!(
        render.text_area_render_surface_shape_us, 0,
        "render cache hit should reuse the shaped surface"
    );
}

#[test]
fn text_area_interaction_surfaces_keep_bounded_observation_coverage() {
    let text = (0..200)
        .map(|line| format!("line {line:03}"))
        .collect::<Vec<_>>()
        .join("\n");
    let buffer = Buffer::from_multiline_text(text);
    let area_model = Area::new(buffer).read_only();
    let style = Style::default();
    let viewport = area::logical(360.0, 72.0);
    let state = ViewState::new_at(0.0, Instant::now()).with_scroll_y(900.0);
    let mut engine = engine();

    let layout = engine.text_area_paint_layout_for_area_at(
        &area_model,
        style,
        viewport,
        state,
        Instant::now(),
    );
    let diagnostics = engine.diagnostics();

    assert!(diagnostics.text_area_visible_logical_lines > 0);
    assert_eq!(
        diagnostics.text_area_layout_segments, diagnostics.text_area_interaction_surfaces,
        "observed line coverage is interaction layout; render coverage is a separate window"
    );
    assert_eq!(
        diagnostics.text_area_interaction_surfaces,
        layout.interaction_surfaces().len()
    );
    assert!(
        diagnostics.text_area_interaction_surfaces > diagnostics.text_area_visible_logical_lines,
        "interaction surfaces should retain an overscan band for smooth overlay and hit-test reuse"
    );
    assert!(
        diagnostics.text_area_overscan_segments > 0,
        "overscan should still be retained for cache/layout"
    );
}

#[test]
fn text_diagnostics_record_visible_text_area_cache_work() {
    let mut engine = engine();
    let buffer = Buffer::from_multiline_text("one\ntwo\nthree");
    let area_model = Area::new(buffer);
    let style = Style::default().with_size(13.0);
    let viewport = area::logical(240.0, 120.0);
    let state = ViewState::default();
    let now = Instant::now();

    engine.reset_diagnostics();
    engine.text_area_paint_layout_for_area_at(&area_model, style, viewport, state.clone(), now);
    let first = engine.diagnostics();
    assert_eq!(first.text_area_paint_layout_calls, 1);
    assert!(first.text_area_line_cache_misses > 0);
    assert!(first.text_area_line_shape_calls > 0);
    assert!(first.text_area_visible_logical_lines > 0);

    engine.reset_diagnostics();
    engine.text_area_paint_layout_for_area_at(&area_model, style, viewport, state, now);
    let cached = engine.diagnostics();
    assert_eq!(cached.text_area_paint_layout_calls, 1);
    assert!(cached.text_area_line_cache_hits > 0);
    assert_eq!(cached.text_area_line_shape_calls, 0);
}

#[test]
fn text_area_render_buffer_is_shaped_once_and_reused_without_resize() {
    let mut engine = engine();
    let buffer = Buffer::from_multiline_text(
        "one two three four five\nsix seven eight nine ten\neleven twelve thirteen",
    );
    let area_model = Area::new(buffer).read_only();
    let style = Style::default().with_size(13.0);
    let viewport = area::logical(240.0, 120.0);
    let state = ViewState::default();
    let now = Instant::now();
    let content_area = area::logical(240.0, 360.0);

    engine.reset_diagnostics();
    let first = engine.text_area_render_layout_for_area_at(
        &area_model,
        style,
        viewport,
        state.clone(),
        now,
        content_area,
    );
    let first_diagnostics = engine.diagnostics();
    assert_eq!(first.render_surfaces().len(), 1);
    assert_eq!(first.interaction_surfaces().len(), 0);
    assert_eq!(first_diagnostics.text_area_render_surface_cache_misses, 1);
    assert_eq!(first_diagnostics.text_area_render_surface_cache_hits, 0);
    assert!(
        first_diagnostics.text_area_render_surface_shape_us > 0,
        "a cold render surface should perform the one required text layout pass"
    );

    engine.reset_diagnostics();
    let cached = engine.text_area_render_layout_for_area_at(
        &area_model,
        style,
        viewport,
        state,
        now,
        content_area,
    );
    let cached_diagnostics = engine.diagnostics();
    assert_eq!(cached.render_surfaces().len(), 1);
    assert_eq!(cached.interaction_surfaces().len(), 0);
    assert_eq!(cached_diagnostics.text_area_render_surface_cache_hits, 1);
    assert_eq!(cached_diagnostics.text_area_render_surface_cache_misses, 0);
    assert_eq!(
        cached_diagnostics.text_area_render_surface_size_us, 0,
        "cache hits must not resize text layout; viewport height is render clipping"
    );
    assert_eq!(
        cached_diagnostics.text_area_render_surface_shape_us, 0,
        "cache hits must reuse the existing shaped text surface"
    );
}

#[test]
fn text_area_render_layout_never_builds_interaction_surfaces() {
    let mut engine = engine();
    let buffer = Buffer::from_multiline_text(
        "alpha
beta
gamma
delta",
    );
    let mut edit_state = buffer.initial_state();
    apply_selection(&buffer, &mut edit_state, selection::Operation::SelectAll);
    let area_model = Area::new(buffer).with_state(edit_state);
    let style = Style::default().with_size(13.0);
    let viewport = area::logical(240.0, 120.0);
    let state = ViewState::default().with_preedit(Some(Preedit::new("x", None)));
    let content_area = area::logical(240.0, 360.0);

    let layout = engine.text_area_render_layout_for_area_at(
        &area_model,
        style,
        viewport,
        state,
        Instant::now(),
        content_area,
    );

    assert_eq!(layout.render_surfaces().len(), 1);
    assert_eq!(
        layout.interaction_surfaces().len(),
        0,
        "render-only layout must not promote itself into observed hit-test/editing layout"
    );
}

#[test]
fn text_area_render_buffer_reuses_chunk_after_small_scroll() {
    let mut engine = engine();
    let text = (0..200)
        .map(|line| format!("line {line:03}"))
        .collect::<Vec<_>>()
        .join("\n");
    let area_model = Area::new(Buffer::from_multiline_text(text)).read_only();
    let style = Style::default().with_size(13.0);
    let line_height = text_area_estimated_line_height(style);
    let viewport = area::logical(240.0, line_height * 8.0);
    let now = Instant::now();
    let content_area = area::logical(240.0, line_height * 200.0);

    engine.reset_diagnostics();
    let first = engine.text_area_render_layout_for_area_at(
        &area_model,
        style,
        viewport,
        ViewState::default(),
        now,
        content_area,
    );
    let first_surface = first
        .render_surfaces()
        .first()
        .expect("initial render layout should prepare one text surface");

    engine.reset_diagnostics();
    let scrolled = engine.text_area_render_layout_for_area_at(
        &area_model,
        style,
        viewport,
        ViewState::default().with_scroll_y(line_height * 4.0),
        now,
        content_area,
    );
    let diagnostics = engine.diagnostics();
    let scrolled_surface = scrolled
        .render_surfaces()
        .first()
        .expect("scrolled render layout should prepare one text surface");

    assert_eq!(
        scrolled_surface.source_line(),
        first_surface.source_line(),
        "source windows should be chunked, not rebuilt for every visible line"
    );
    assert_eq!(diagnostics.text_area_render_surface_cache_hits, 1);
    assert_eq!(diagnostics.text_area_render_surface_cache_misses, 0);
    assert!(
        diagnostics.text_area_render_surface_source_lines
            >= diagnostics.text_area_visible_logical_lines + TEXT_AREA_RENDER_GUARD_LINES,
        "render surface should own a reusable guard band"
    );
    assert!(
        diagnostics.text_area_render_surface_source_lines
            <= diagnostics.text_area_visible_logical_lines + TEXT_AREA_RENDER_GUARD_LINES * 2,
        "render surface should not shape an excessive scroll window"
    );
    assert_eq!(
        diagnostics.text_area_render_surface_shape_us, 0,
        "scrolling inside a render chunk must reuse the shaped glyphon buffer"
    );
}

#[test]
fn text_area_frame_cache_is_bounded() {
    let mut engine = engine();
    let style = Style::default().with_size(13.0);
    let viewport = area::logical(240.0, 80.0);
    let state = ViewState::default();
    let now = Instant::now();

    for index in 0..(TEXT_AREA_LINE_DISPLAY_CACHE_CAPACITY + 16) {
        let buffer = Buffer::from_multiline_text(format!("line {index}\nnext"));
        engine.text_area_paint_layout_for_area_at(
            &Area::new(buffer),
            style,
            viewport,
            state.clone(),
            now,
        );
    }

    assert_eq!(
        engine.text_area_line_displays.len(),
        TEXT_AREA_LINE_DISPLAY_CACHE_CAPACITY
    );
}

#[test]
fn text_area_preedit_projection_is_not_cached() {
    let mut engine = engine();
    let buffer = Buffer::from_multiline_text("hello");
    let area_model = Area::new(buffer.clone());
    let style = Style::default().with_size(13.0);
    let viewport = area::logical(240.0, 80.0);
    let state = ViewState::default();
    let now = Instant::now();
    let committed = engine
        .text_area_paint_layout_for_area_at(&area_model, style, viewport, state.clone(), now)
        .into_interaction_parts()
        .1;
    let preedit_state = state.with_preedit(Some(Preedit::new("x", None)));
    let preedit = engine
        .text_area_paint_layout_for_area_at(&area_model, style, viewport, preedit_state, now)
        .into_interaction_parts()
        .1;
    let after = engine
        .text_area_paint_layout_for_area_at(
            &Area::new(buffer),
            style,
            viewport,
            ViewState::default(),
            now,
        )
        .into_interaction_parts()
        .1;

    assert_eq!(surface_line_text(&preedit, 0), "hellox");
    assert_eq!(
        surface_line_text(&committed, 0),
        surface_line_text(&after, 0)
    );
    assert_eq!(surface_line_text(&after, 0), "hello");
    assert!(!engine.text_area_line_displays.is_empty());
}

#[test]
fn text_area_prepared_frame_is_bounded_to_viewport_window() {
    let mut engine = engine();
    let text = (0..1_000)
        .map(|index| format!("line {index}"))
        .collect::<Vec<_>>()
        .join("\n");
    let buffer = Buffer::from_multiline_text(text);
    let style = Style::default().with_size(13.0);
    let viewport = area::logical(240.0, 52.0);
    let state = ViewState::default();
    let now = Instant::now();
    let (layout, surfaces) = engine
        .text_area_paint_layout_for_area_at(&Area::new(buffer), style, viewport, state, now)
        .into_interaction_parts();

    assert!(surfaces.len() <= TEXT_AREA_FRAME_MAX_LOGICAL_LINES);
    assert!(surfaces.len() < 1_000);
    assert!(layout.content_area().height() > viewport.height());
}
#[test]
fn large_text_area_scroll_and_highlight_work_are_viewport_bounded() {
    let mut engine = engine();
    let text = (0..100_000)
        .map(|index| format!("line {index}"))
        .collect::<Vec<_>>()
        .join("\n");
    let buffer = Buffer::from_multiline_text(text);
    let mut edit_state = buffer.initial_state();
    apply_selection(&buffer, &mut edit_state, selection::Operation::SelectAll);
    let area_model = Area::new(buffer).with_state(edit_state);
    let style = Style::default().with_size(13.0);
    let viewport = area::logical(240.0, 52.0);
    let state = ViewState::default().with_scroll_y(13.0 * 1.25 * 50_000.0);

    engine.reset_interaction_stats();
    engine.reset_highlight_stats();
    let (layout, surfaces) = engine
        .text_area_paint_layout_for_area_at(&area_model, style, viewport, state, Instant::now())
        .into_interaction_parts();
    let interaction_stats = engine.interaction_stats();
    let highlight_stats = engine.highlight_stats();
    let diagnostics = engine.diagnostics();
    let visible_runs = surface_visual_runs(&surfaces);

    assert!(!layout.selection_spans().is_empty());
    assert!(surfaces.len() <= TEXT_AREA_FRAME_MAX_LOGICAL_LINES);
    assert_eq!(diagnostics.text_area_interaction_surfaces, surfaces.len());
    assert_eq!(
        diagnostics.text_area_layout_segments,
        diagnostics.text_area_interaction_surfaces
    );
    assert!(interaction_stats.text_area_frame_shape_calls <= TEXT_AREA_FRAME_MAX_LOGICAL_LINES);
    assert!(
        interaction_stats.text_area_frame_shaped_logical_lines <= TEXT_AREA_FRAME_MAX_LOGICAL_LINES
    );
    assert_eq!(interaction_stats.text_area_shape_until_scroll_calls, 0);
    assert!(highlight_stats.run_scans >= visible_runs);
    assert!(highlight_stats.run_scans <= TEXT_AREA_FRAME_MAX_LOGICAL_LINES);
    assert_eq!(highlight_stats.highlight_calls, 0);
}
#[test]
fn piece_tree_insert_updates_touched_storage_without_full_materialization() {
    let mut editor = Editor::new();
    let text = (0..100_000)
        .map(|index| format!("line {index}"))
        .collect::<Vec<_>>()
        .join("\n");
    let mut buffer = Buffer::from_multiline_text(text);
    let mut edit_state = buffer.initial_state();
    let paste = (0..512)
        .map(|index| format!("paste {index}"))
        .collect::<Vec<_>>()
        .join("\n");

    buffer.reset_document_stats();
    apply_edit(
        &mut editor,
        &mut buffer,
        &mut edit_state,
        Edit::insert(paste.clone()),
    );
    let stats = buffer.document_stats();
    let (_owned, _mapped, add) = buffer.document_piece_source_lengths();

    assert_eq!(stats.full_materializations, 0);
    assert_eq!(stats.total_document_scans, 0);
    assert_eq!(stats.piece_tree_updates, 1);
    assert!(add >= paste.lines().map(str::len).sum::<usize>());
}

#[test]
fn source_span_stream_writes_without_full_document_materialization() {
    let mut editor = Editor::new();
    let mut buffer = Buffer::from_multiline_text("alpha\nbeta\ngamma");
    let mut state = buffer.initial_state();
    apply_edit(&mut editor, &mut buffer, &mut state, Edit::insert("!"));
    buffer.reset_document_stats();
    let mut written = Vec::new();

    buffer
        .write_to(&mut written)
        .expect("source spans should stream to a writer");

    assert_eq!(String::from_utf8(written).unwrap(), "alpha\nbeta\ngamma!");
    assert_eq!(buffer.document_stats().full_materializations, 0);
}

#[test]
fn marks_round_trip_through_line_identity() {
    let mut editor = Editor::new();
    let mut buffer = Buffer::from_multiline_text("alpha\nbeta\ngamma");
    let mut edit_state = buffer.initial_state();
    let beta = "alpha\nbe".len();

    apply_selection(
        &buffer,
        &mut edit_state,
        selection::Operation::set_position(Position::new(beta)),
    );
    let anchor = mark(edit_state);
    apply_edit(&mut editor, &mut buffer, &mut edit_state, Edit::insert("X"));
    let after = mark(edit_state);

    assert_eq!(anchor.line_id, after.line_id);
    assert!(after.byte_offset > anchor.byte_offset);
}

#[test]
fn file_buffer_owns_original_source_without_mapping() {
    let path = std::env::temp_dir().join(format!(
        "wgpu_l3_text_mapped_{}_{}.txt",
        std::process::id(),
        Instant::now().elapsed().as_nanos()
    ));
    std::fs::write(&path, "one\ntwo\nthree").expect("temp mapped text should be writable");

    let buffer = Buffer::from_file(&path).expect("owned text buffer should open");
    let (owned, mapped, add) = buffer.document_piece_source_lengths();

    assert_eq!(buffer.to_plain_text(), "one\ntwo\nthree");
    assert_eq!(buffer.original_len(), "one\ntwo\nthree".len());
    assert_eq!(owned, "one\ntwo\nthree".len());
    assert_eq!(mapped, 0);
    assert_eq!(add, 0);

    let _ = std::fs::remove_file(path);
}

#[test]
fn file_buffer_preserves_crlf_and_uses_dominant_ending_for_inserted_breaks() {
    let path = std::env::temp_dir().join(format!(
        "wgpu_l3_text_crlf_{}_{}.txt",
        std::process::id(),
        Instant::now().elapsed().as_nanos()
    ));
    std::fs::write(&path, "one\r\ntwo\r\nthree\n").expect("temporary CRLF text should be writable");
    let mut buffer = Buffer::from_file(&path).expect("owned file buffer should open");
    let mut state = buffer.initial_state();
    let mut editor = Editor::new();

    assert_eq!(buffer.text(), "one\r\ntwo\r\nthree\n");
    assert_eq!(buffer.logical_line_count(), 4);
    assert_eq!(buffer.text_for_line_range(0, 1), "one");
    apply_edit(
        &mut editor,
        &mut buffer,
        &mut state,
        Edit::insert_line_break(),
    );
    assert_eq!(buffer.text(), "one\r\ntwo\r\nthree\n\r\n");

    let _ = std::fs::remove_file(path);
}
#[test]
fn source_span_seek_handles_leaf_boundaries() {
    let first = "x".repeat(TEXT_DOCUMENT_TARGET_LEAF_BYTES - 1);
    let text = format!("{first}\nsecond");
    let buffer = Buffer::from_multiline_text(text);
    let boundary_line = 1;
    let boundary_index = TEXT_DOCUMENT_TARGET_LEAF_BYTES;

    let cursor = buffer.cursor_for_text_index(boundary_index);
    let position = buffer.position_for_text_index(boundary_index);

    assert_eq!(cursor.line, boundary_line);
    assert_eq!(cursor.index, 0);
    assert_eq!(position.index, boundary_index);
}
#[test]
fn larger_font_measures_taller_than_smaller_font() {
    let mut engine = engine();
    let small = Document::from_block({
        let mut block = Block::new(Align::Start);
        block.push_run(Run::new("Label", Style::default().with_size(10.0)));
        block
    });
    let large = Document::from_block({
        let mut block = Block::new(Align::Start);
        block.push_run(Run::new("Label", Style::default().with_size(24.0)));
        block
    });

    let small = engine.measure(&small, Measure::unbounded());
    let large = engine.measure(&large, Measure::unbounded());

    assert!(large.height() > small.height());
}

#[test]
fn repeated_measurement_reuses_cached_metrics() {
    let mut engine = engine();
    let document = Document::plain("Cached Label");

    let first = engine.measure(&document, Measure::unbounded());
    let second = engine.measure(&document, Measure::unbounded());

    assert_eq!(first, second);
    assert_eq!(engine.uncached_measure_count(), 1);
    assert_eq!(engine.cache_len(), 1);
}

#[test]
fn color_only_changes_reuse_cached_metrics() {
    let mut engine = engine();
    let red = Document::plain("Cached Label").with_color(Color::RED);
    let black = Document::plain("Cached Label").with_color(Color::BLACK);

    let red = engine.measure(&red, Measure::unbounded());
    let black = engine.measure(&black, Measure::unbounded());

    assert_eq!(red, black);
    assert_eq!(engine.uncached_measure_count(), 1);
}

#[test]
fn shaping_relevant_document_and_bounds_changes_use_distinct_cache_keys() {
    let mut engine = engine();
    let base = styled_document("Cached Label", Align::Start, 16.0, Weight::Normal);
    let text = styled_document("Different Label", Align::Start, 16.0, Weight::Normal);
    let size = styled_document("Cached Label", Align::Start, 20.0, Weight::Normal);
    let weight = styled_document("Cached Label", Align::Start, 16.0, Weight::Bold);
    let align = styled_document("Cached Label", Align::End, 16.0, Weight::Normal);

    engine.measure(&base, Measure::unbounded());
    engine.measure(&text, Measure::unbounded());
    engine.measure(&size, Measure::unbounded());
    engine.measure(&weight, Measure::unbounded());
    engine.measure(&align, Measure::unbounded());
    engine.measure(&base, Measure::bounded(area::logical(40.0, 100.0)));

    assert_eq!(engine.uncached_measure_count(), 6);
    assert_eq!(engine.cache_len(), 6);
}

#[test]
fn bounded_fifo_cache_evicts_oldest_entries() {
    let mut engine = Engine::with_cache_capacity(2);
    let first = Document::plain("First");
    let second = Document::plain("Second");
    let third = Document::plain("Third");

    engine.measure(&first, Measure::unbounded());
    engine.measure(&second, Measure::unbounded());
    engine.measure(&third, Measure::unbounded());
    engine.measure(&first, Measure::unbounded());

    assert_eq!(engine.cache_len(), 2);
    assert_eq!(engine.uncached_measure_count(), 4);
}

#[test]
fn buffer_inserts_and_deletes_text() {
    let mut editor = Editor::new();
    let mut buffer = Buffer::from_text("ab");
    let mut state = buffer.initial_state();

    apply_edit(&mut editor, &mut buffer, &mut state, Edit::insert("c"));
    apply_selection(
        &buffer,
        &mut state,
        selection::Operation::move_position(Motion::VisualLeft),
    );
    apply_edit(&mut editor, &mut buffer, &mut state, Edit::backspace());

    assert_eq!(buffer.text(), "ac");
    assert_eq!(cursor(&buffer, state).index, 1);

    apply_edit(&mut editor, &mut buffer, &mut state, Edit::delete());

    assert_eq!(buffer.text(), "a");
    assert_eq!(cursor(&buffer, state).index, 1);
}

#[test]
fn buffer_select_all_replaces_selection() {
    let mut editor = Editor::new();
    let mut buffer = Buffer::from_text("hello");
    let mut state = buffer.initial_state();

    apply_selection(&buffer, &mut state, selection::Operation::SelectAll);
    assert_eq!(selected_range(&buffer, state), Some(Range::new(0, 5)));

    apply_edit(&mut editor, &mut buffer, &mut state, Edit::insert("hi"));

    assert_eq!(buffer.text(), "hi");
    assert_eq!(cursor(&buffer, state).index, 2);
    assert_eq!(selected_range(&buffer, state), None);
}

#[test]
fn replace_range_normalizes_inserted_text_and_restores_caret() {
    let mut editor = Editor::new();
    let mut buffer = Buffer::from_text("hello world");
    let mut state = buffer.initial_state();

    assert!(apply_edit(
        &mut editor,
        &mut buffer,
        &mut state,
        Edit::replace_range(6..11, "there\nfriend")
    ));

    assert_eq!(buffer.text(), "hello there friend");
    assert_eq!(
        cursor(&buffer, state),
        Cursor::new(0, "hello there friend".len())
    );
    assert_eq!(selected_range(&buffer, state), None);
}

#[test]
fn move_range_adjusts_forward_drop_position() {
    let mut editor = Editor::new();
    let mut buffer = Buffer::from_text("abcdef");
    let mut state = buffer.initial_state();

    assert!(apply_edit(
        &mut editor,
        &mut buffer,
        &mut state,
        Edit::move_range(1..3, 5)
    ));

    assert_eq!(buffer.text(), "adebcf");
    assert_eq!(cursor(&buffer, state), Cursor::new(0, 5));
    assert_eq!(selected_range(&buffer, state), None);
}

#[test]
fn repeated_large_paste_updates_line_index_without_full_rebuild() {
    let mut editor = Editor::new();
    let text = (0..100_000)
        .map(|line| format!("line {line}"))
        .collect::<Vec<_>>()
        .join("\n");
    let mut buffer = Buffer::from_multiline_text(text);
    let mut state = buffer.initial_state();
    let block = (0..64)
        .map(|line| format!("paste {line}"))
        .collect::<Vec<_>>()
        .join("\n");
    let end = Position::new(buffer.len());

    apply_selection(&buffer, &mut state, selection::Operation::set_position(end));
    buffer.reset_line_index_stats();

    assert!(apply_edit(
        &mut editor,
        &mut buffer,
        &mut state,
        Edit::insert(format!("\n{block}"))
    ));
    assert!(apply_edit(
        &mut editor,
        &mut buffer,
        &mut state,
        Edit::insert(format!("\n{block}"))
    ));

    let (full_rebuilds, splice_updates) = buffer.line_index_stats();
    assert_eq!(full_rebuilds, 0);
    assert_eq!(splice_updates, 2);
    assert_eq!(buffer.logical_line_count(), 100_000 + 128);
    assert!(buffer.text().ends_with("paste 63"));
}
#[test]
fn buffer_shift_motion_extends_selection() {
    let buffer = Buffer::from_text("hello");
    let mut state = buffer.initial_state();

    apply_selection(
        &buffer,
        &mut state,
        selection::Operation::extend_position(Motion::VisualLeft),
    );

    assert_eq!(cursor(&buffer, state).index, 4);
    assert_eq!(selected_range(&buffer, state), Some(Range::new(4, 5)));

    apply_selection(
        &buffer,
        &mut state,
        selection::Operation::extend_position(Motion::LineStart),
    );

    assert_eq!(cursor(&buffer, state).index, 0);
    assert_eq!(selected_range(&buffer, state), Some(Range::new(0, 5)));
}

#[test]
fn buffer_plain_motion_collapses_selection() {
    let buffer = Buffer::from_text("hello");
    let mut state = buffer.initial_state();

    apply_selection(&buffer, &mut state, selection::Operation::SelectAll);
    apply_selection(
        &buffer,
        &mut state,
        selection::Operation::move_position(Motion::VisualLeft),
    );

    assert_eq!(cursor(&buffer, state).index, 0);
    assert_eq!(selected_range(&buffer, state), None);

    apply_selection(&buffer, &mut state, selection::Operation::SelectAll);
    apply_selection(
        &buffer,
        &mut state,
        selection::Operation::move_position(Motion::VisualRight),
    );

    assert_eq!(cursor(&buffer, state).index, 5);
    assert_eq!(selected_range(&buffer, state), None);
}

#[test]
fn buffer_word_delete_uses_cosmic_word_motion() {
    let mut editor = Editor::new();
    let mut buffer = Buffer::from_text("hello world again");
    let mut state = buffer.initial_state();

    apply_edit(
        &mut editor,
        &mut buffer,
        &mut state,
        Edit::delete_word_backward(),
    );

    assert_eq!(buffer.text(), "hello world ");
    assert_eq!(cursor(&buffer, state).index, "hello world ".len());

    apply_selection(
        &buffer,
        &mut state,
        selection::Operation::set_cursor(Cursor::new(0, 0)),
    );
    apply_edit(
        &mut editor,
        &mut buffer,
        &mut state,
        Edit::delete_word_forward(),
    );

    assert_eq!(buffer.text(), " world ");
    assert_eq!(cursor(&buffer, state).index, 0);
}

#[test]
fn buffer_pointer_double_click_selects_word_and_triple_click_selects_all() {
    let buffer = Buffer::from_text("hello world");
    let mut state = buffer.initial_state();

    apply_selection(
        &buffer,
        &mut state,
        selection::Operation::pointer(PointerKind::DoubleClick, Cursor::new(0, 1)),
    );

    assert_eq!(selected_range(&buffer, state), Some(Range::new(0, 5)));

    apply_selection(
        &buffer,
        &mut state,
        selection::Operation::pointer(PointerKind::TripleClick, Cursor::new(0, 7)),
    );

    assert_eq!(
        selected_range(&buffer, state),
        Some(Range::new(0, "hello world".len()))
    );
}

#[test]
fn buffer_pointer_drag_extends_from_click_anchor() {
    let buffer = Buffer::from_text("hello world");
    let mut state = buffer.initial_state();

    apply_selection(
        &buffer,
        &mut state,
        selection::Operation::pointer(PointerKind::Click, Cursor::new(0, 0)),
    );
    apply_selection(
        &buffer,
        &mut state,
        selection::Operation::pointer(PointerKind::Drag, Cursor::new(0, 5)),
    );

    assert_eq!(selected_range(&buffer, state), Some(Range::new(0, 5)));
}

#[test]
fn buffer_edits_preserve_unicode_boundaries() {
    let mut editor = Editor::new();
    let mut buffer = Buffer::from_text("aé🙂");
    let mut state = buffer.initial_state();

    apply_selection(
        &buffer,
        &mut state,
        selection::Operation::set_cursor(Cursor::new(0, 3)),
    );
    assert_eq!(cursor(&buffer, state).index, "aé".len());

    apply_edit(&mut editor, &mut buffer, &mut state, Edit::backspace());
    assert_eq!(buffer.text(), "a🙂");

    apply_selection(
        &buffer,
        &mut state,
        selection::Operation::move_position(Motion::LineEnd),
    );
    apply_edit(&mut editor, &mut buffer, &mut state, Edit::backspace());
    assert_eq!(buffer.text(), "a");
    assert!(buffer.text().is_char_boundary(cursor(&buffer, state).index));
}
#[test]
fn byte_index_edits_snap_to_grapheme_boundaries() {
    let mut editor = Editor::new();
    let combining = "e\u{301}";
    let family = "👨‍👩‍👧‍👦";
    let flag = "🇺🇸";

    let mut replace = Buffer::from_text(format!("a{combining}b"));
    let mut replace_state = replace.initial_state();
    assert!(apply_edit(
        &mut editor,
        &mut replace,
        &mut replace_state,
        Edit::replace_range(2..3, "X")
    ));
    assert_eq!(replace.text(), "aXb");

    let mut insert = Buffer::from_text(format!("a{family}b"));
    let mut insert_state = insert.initial_state();
    let inside_family = 1 + "👨".len();
    assert!(apply_edit(
        &mut editor,
        &mut insert,
        &mut insert_state,
        Edit::insert_at(inside_family, "X")
    ));
    assert_eq!(insert.text(), format!("aX{family}b"));

    let flag_source = format!("a{flag}bc");
    let mut moved = Buffer::from_text(flag_source.clone());
    let mut moved_state = moved.initial_state();
    assert!(apply_edit(
        &mut editor,
        &mut moved,
        &mut moved_state,
        Edit::move_range(3..6, flag_source.len())
    ));
    assert_eq!(moved.text(), format!("abc{flag}"));

    let cursor_buffer = Buffer::from_text(format!("a{family}b"));
    let cursor = cursor_buffer.cursor_for_text_index(inside_family);
    assert_eq!(cursor_buffer.text_index_for_cursor(cursor), 1);
    assert_eq!(
        Field::new(format!("{combining}{family}{flag}"))
            .obscured_dot()
            .presentation_text(),
        "•••"
    );
}

#[test]
fn logical_motion_respects_grapheme_boundaries() {
    let family = "👨‍👩‍👧‍👦";
    let buffer = Buffer::from_text(format!("a{family}b"));
    let mut state = buffer.initial_state();

    let end = buffer.text().len();
    apply_selection(&buffer, &mut state, selection::Operation::set_position(end));
    apply_selection(
        &buffer,
        &mut state,
        selection::Operation::move_position(Motion::LogicalPrevious),
    );
    assert_eq!(position(&buffer, state).index, 1 + family.len());

    apply_selection(
        &buffer,
        &mut state,
        selection::Operation::move_position(Motion::LogicalPrevious),
    );
    assert_eq!(position(&buffer, state).index, 1);
}

#[test]
fn unresolved_visual_motion_uses_caret_map() {
    let buffer = Buffer::from_text("one\ntwo\nthree");
    let mut state = buffer.initial_state();
    let target = Position::new("one\n".len());
    let mut caret_map = StubCaretMap {
        calls: 0,
        position: target,
    };

    let changed = apply_selection_with_caret_map(
        &buffer,
        &mut state,
        selection::Operation::move_position(Motion::VisualDown),
        &mut caret_map,
    );

    assert_eq!(caret_map.calls, 1);
    assert!(changed);
    assert_eq!(position(&buffer, state), target);
}

#[test]
fn unicode_word_boundaries_drive_selection_and_delete() {
    let mut engine = engine();
    let mut editor = Editor::new();
    let mut buffer = Buffer::from_text("hello שלום again");
    let mut state = buffer.initial_state();
    let hebrew_start = "hello ".len();

    apply_selection(
        &buffer,
        &mut state,
        selection::Operation::pointer(PointerKind::DoubleClick, Position::new(hebrew_start + 2)),
    );
    assert_eq!(selected_text(&buffer, state).as_deref(), Some("שלום"));
    assert_eq!(
        selected_range(&buffer, state),
        Some(Range::new(hebrew_start, hebrew_start + "שלום".len()))
    );

    let end = buffer.text().len();
    apply_selection(&buffer, &mut state, selection::Operation::set_position(end));
    engine.reset_interaction_stats();
    apply_edit(
        &mut editor,
        &mut buffer,
        &mut state,
        Edit::delete_word_backward(),
    );
    assert_eq!(buffer.text(), "hello שלום ");
}

#[test]
fn bidi_hit_testing_preserves_visual_affinity() {
    let mut engine = engine();
    let buffer = Buffer::from_text("abc אבג");
    let prepared = engine.prepare_text_field_buffer(
        &buffer,
        Style::default().with_size(18.0),
        area::logical(400.0, 32.0),
    );
    let prepared = prepared.0.borrow();
    let map = TextLayoutMap::new(&prepared);
    let rtl_glyph = prepared
        .layout_runs()
        .flat_map(|run| {
            let line_start = map.line_starts.get(run.line_i).copied().unwrap_or(0);
            run.glyphs
                .iter()
                .map(move |glyph| (run.line_top, run.line_height, line_start, glyph))
        })
        .find(|(_, _, _, glyph)| glyph.level.is_rtl())
        .expect("mixed Hebrew text should produce an RTL glyph");
    let (line_top, line_height, line_start, glyph) = rtl_glyph;
    let y = line_top + line_height * 0.5;

    let left = map
        .hit(&prepared, glyph.x + glyph.w * 0.25, y)
        .expect("left half should hit the RTL glyph");
    let right = map
        .hit(&prepared, glyph.x + glyph.w * 0.75, y)
        .expect("right half should hit the RTL glyph");

    assert_eq!(
        left,
        Position::with_affinity(line_start + glyph.end, Affinity::Upstream)
    );
    assert_eq!(
        right,
        Position::with_affinity(line_start + glyph.start, Affinity::Downstream)
    );
}

#[test]
fn rtl_paragraph_embedded_ltr_glyph_owns_its_hit_direction() {
    let mut engine = engine();
    let text = "אבג abc דהו";
    let buffer = Buffer::from_text(text);
    let prepared = engine.prepare_text_field_buffer(
        &buffer,
        Style::default().with_size(18.0),
        area::logical(400.0, 32.0),
    );
    let prepared = prepared.0.borrow();
    let map = TextLayoutMap::new(&prepared);
    let embedded_ltr = prepared
        .layout_runs()
        .flat_map(|run| {
            let line_start = map.line_starts.get(run.line_i).copied().unwrap_or(0);
            run.glyphs.iter().filter_map(move |glyph| {
                let start = glyph.start.min(glyph.end);
                let end = glyph.start.max(glyph.end);
                let source = text.get(line_start + start..line_start + end)?;
                (run.rtl
                    && !glyph.level.is_rtl()
                    && source
                        .chars()
                        .any(|character| character.is_ascii_alphabetic()))
                .then_some((run.line_top, run.line_height, line_start, glyph))
            })
        })
        .next()
        .expect("RTL paragraph should contain an embedded LTR glyph");
    let (line_top, line_height, line_start, glyph) = embedded_ltr;
    let y = line_top + line_height * 0.5;

    let left = map
        .hit(&prepared, glyph.x + glyph.w * 0.25, y)
        .expect("left half should hit the embedded LTR glyph");
    let right = map
        .hit(&prepared, glyph.x + glyph.w * 0.75, y)
        .expect("right half should hit the embedded LTR glyph");

    assert_eq!(
        left,
        Position::with_affinity(line_start + glyph.start, Affinity::Downstream)
    );
    assert_eq!(
        right,
        Position::with_affinity(line_start + glyph.end, Affinity::Upstream)
    );
}

#[test]
fn text_field_surface_cache_reuses_shape_but_projects_current_color() {
    let mut engine = engine();
    let field = Field::new("cached field");
    let area = area::logical(240.0, 32.0);
    let state = ViewState::default();
    let red = Style::default().with_color(Color::RED);
    let black = Style::default().with_color(Color::BLACK);

    let first = engine.text_field_paint_layout_for_field(&field, red, area, state.clone());
    let second = engine.text_field_paint_layout_for_field(&field, black, area, state);
    let first = first.surface().expect("field should produce a surface");
    let second = second.surface().expect("field should produce a surface");

    assert!(std::rc::Rc::ptr_eq(&first.buffer(), &second.buffer()));
    assert_eq!(first.default_color(), Color::RED);
    assert_eq!(second.default_color(), Color::BLACK);
}

#[test]
fn start_end_alignment_resolves_against_base_direction() {
    assert_eq!(
        layout::glyphon_align(Align::Start, ResolvedTextDirection::Ltr),
        glyphon::cosmic_text::Align::Left
    );
    assert_eq!(
        layout::glyphon_align(Align::Start, ResolvedTextDirection::Rtl),
        glyphon::cosmic_text::Align::Right
    );
    assert_eq!(
        layout::glyphon_align(Align::End, ResolvedTextDirection::Rtl),
        glyphon::cosmic_text::Align::Left
    );
}

#[test]
fn mixed_direction_preedit_spans_are_projected_inline() {
    let mut engine = engine();
    let buffer = Buffer::from_text("abc אבג");
    let field = Field::new(buffer);
    let state =
        ViewState::default().with_preedit(Some(Preedit::new("שלום", Some((0, "של".len())))));
    let layout = engine.text_field_layout_for_field_at(
        &field,
        Style::default().with_size(18.0),
        area::logical(400.0, 32.0),
        state,
        Instant::now(),
    );

    assert!(layout.caret().is_some());
    assert!(!layout.preedit_underline_spans().is_empty());
    assert!(!layout.preedit_selection_spans().is_empty());
}
#[test]
fn buffer_normalizes_inserted_line_endings_to_spaces() {
    let mut editor = Editor::new();
    let mut buffer = Buffer::from_text("a\nb");
    let mut edit_state = buffer.initial_state();

    assert_eq!(buffer.text(), "a b");

    apply_edit(
        &mut editor,
        &mut buffer,
        &mut edit_state,
        Edit::insert("\nc\r"),
    );

    assert_eq!(buffer.text(), "a b c ");
}

#[test]
fn text_field_selection_layout_uses_shaped_text_span() {
    let mut engine = engine();
    let buffer = Buffer::from_text("hello");
    let mut edit_state = buffer.initial_state();

    apply_selection(&buffer, &mut edit_state, selection::Operation::SelectAll);
    let field = Field::new(buffer.clone()).with_state(edit_state);

    let layout = engine.text_field_layout_for_field(
        &field,
        Style::default().with_size(16.0),
        area::logical(240.0, 32.0),
        ViewState::default(),
    );
    let span = layout
        .selection_spans()
        .first()
        .expect("select all should create a highlight span");

    assert!(span.width() > 0.0);
    assert!(span.width() < 240.0);
    assert!(span.x() >= 0.0);
}
#[test]
fn text_field_preedit_renders_inline_text_spans_and_commit_clears_projection() {
    let mut engine = engine();
    let mut editor = Editor::new();
    let mut buffer = Buffer::from_text("hello");
    let mut edit_state = buffer.initial_state();
    apply_selection(&buffer, &mut edit_state, selection::Operation::SelectAll);
    let state = ViewState::default().with_preedit(Some(Preedit::new("xy", Some((0, 1)))));
    let field = Field::new(buffer.clone()).with_state(edit_state);

    assert_eq!(field.presentation_text_for_state(&state), "xy");

    let layout = engine.text_field_layout_for_field_at(
        &field,
        Style::default().with_size(16.0),
        area::logical(240.0, 32.0),
        state,
        Instant::now(),
    );

    assert!(layout.caret().is_some());
    assert!(!layout.preedit_underline_spans().is_empty());
    assert!(!layout.preedit_selection_spans().is_empty());

    editor.apply_edit(&mut buffer, &mut edit_state, Edit::ime_commit("xy"));
    let committed_field = Field::new(buffer.clone()).with_state(edit_state);
    let committed = engine.text_field_layout_for_field(
        &committed_field,
        Style::default().with_size(16.0),
        area::logical(240.0, 32.0),
        ViewState::default(),
    );

    assert_eq!(buffer.text(), "xy");
    assert!(committed.preedit_underline_spans().is_empty());
    assert!(committed.preedit_selection_spans().is_empty());
}

#[test]
fn text_field_preedit_caret_uses_composed_projection() {
    let mut engine = engine();
    let buffer = Buffer::from_text("hello");
    let field = Field::new(buffer);
    let style = Style::default().with_size(16.0);
    let viewport = area::logical(240.0, 32.0);
    let now = Instant::now();
    let committed = engine
        .text_field_layout_for_field_at(&field, style, viewport, ViewState::default(), now)
        .caret()
        .expect("committed caret should be visible");
    let composed = engine
        .text_field_layout_for_field_at(
            &field,
            style,
            viewport,
            ViewState::default().with_preedit(Some(Preedit::new(" world", None))),
            now,
        )
        .caret()
        .expect("preedit caret should be visible");

    assert!(composed.x() > committed.x());
}

#[test]
fn text_area_metrics_layout_skips_highlight_overlay_work() {
    let mut engine = engine();
    let text = (0..1_000)
        .map(|index| format!("line {index}"))
        .collect::<Vec<_>>()
        .join("\n");
    let buffer = Buffer::from_multiline_text(text);
    let mut edit_state = buffer.initial_state();
    apply_selection(&buffer, &mut edit_state, selection::Operation::SelectAll);
    let area_model = Area::new(buffer).with_state(edit_state);
    let style = Style::default().with_size(13.0);
    let viewport = area::logical(240.0, 52.0);

    engine.reset_highlight_stats();
    engine.reset_interaction_stats();
    let layout = engine.text_area_metrics_layout_for_area_at(
        &area_model,
        style,
        viewport,
        ViewState::default(),
        Instant::now(),
    );

    assert_eq!(engine.highlight_stats(), HighlightStats::default());
    let interaction_stats = engine.interaction_stats();
    assert_eq!(interaction_stats.text_area_frame_shape_calls, 0);
    assert_eq!(interaction_stats.text_area_shape_until_scroll_calls, 0);
    assert!(layout.selection_spans().is_empty());
    assert!(layout.preedit_underline_spans().is_empty());
    assert!(layout.preedit_selection_spans().is_empty());
}

#[test]
fn text_area_paint_layout_computes_highlight_overlays_from_interaction_surfaces() {
    let mut engine = engine();
    let text = (0..1_000)
        .map(|index| format!("line {index}"))
        .collect::<Vec<_>>()
        .join("\n");
    let buffer = Buffer::from_multiline_text(text);
    let mut edit_state = buffer.initial_state();
    apply_selection(&buffer, &mut edit_state, selection::Operation::SelectAll);
    let area_model = Area::new(buffer).with_state(edit_state);
    let style = Style::default().with_size(13.0);
    let viewport = area::logical(240.0, 52.0);
    let state = ViewState::default();
    let now = Instant::now();

    engine.reset_highlight_stats();
    let (layout, surfaces) = engine
        .text_area_paint_layout_for_area_at(&area_model, style, viewport, state.clone(), now)
        .into_interaction_parts();
    let stats = engine.highlight_stats();
    let diagnostics = engine.diagnostics();
    let visible_runs = surface_visual_runs(&surfaces);

    assert!(!layout.selection_spans().is_empty());
    assert_eq!(diagnostics.text_area_interaction_surfaces, surfaces.len());
    assert_eq!(
        diagnostics.text_area_layout_segments,
        diagnostics.text_area_interaction_surfaces
    );
    assert!(visible_runs <= TEXT_AREA_FRAME_MAX_LOGICAL_LINES);
    assert!(visible_runs < 1_000);
    assert!(stats.run_scans >= visible_runs);
    assert!(stats.run_scans <= TEXT_AREA_FRAME_MAX_LOGICAL_LINES);
    assert_eq!(stats.highlight_calls, 0);
    assert_eq!(stats.spans, layout.selection_spans().len());

    engine.reset_highlight_stats();
    let cached =
        engine.text_area_paint_layout_for_area_at(&area_model, style, viewport, state, now);
    let cached_stats = engine.highlight_stats();

    assert!(!cached.layout().selection_spans().is_empty());
    assert!(cached_stats.run_scans >= visible_runs);
    assert!(cached_stats.run_scans <= TEXT_AREA_FRAME_MAX_LOGICAL_LINES);
    assert_eq!(cached_stats.highlight_calls, 0);
    assert_eq!(cached_stats.spans, cached.layout().selection_spans().len());
}

#[test]
fn wrapped_text_area_line_displays_do_not_overlap() {
    let mut engine = engine();
    let long = "wrap ".repeat(40);
    let area_model = Area::new(Buffer::from_multiline_text(format!("{long}\nnext")));
    let style = Style::default().with_size(16.0);
    let viewport = area::logical(72.0, 220.0);

    let paint_layout = engine.text_area_paint_layout_for_area_at(
        &area_model,
        style,
        viewport,
        ViewState::default(),
        Instant::now(),
    );
    let surfaces = paint_layout.interaction_surfaces();

    assert!(surfaces.len() >= 2);
    assert_eq!(surface_line_text(surfaces, 1), "next");
    let first_bottom = surfaces[0].y() + surfaces[0].height();
    assert!(
        surfaces[1].y() >= first_bottom - 0.5,
        "next line started at {}, before wrapped first line bottom {}",
        surfaces[1].y(),
        first_bottom
    );
}

#[test]
fn wrapped_text_area_hit_testing_uses_clicked_visual_row() {
    let mut engine = engine();
    let text = "alpha beta gamma delta epsilon zeta eta theta iota kappa lambda mu";
    let buffer = Buffer::from_multiline_text(text);
    let area_model = Area::new(buffer);
    let style = Style::default().with_size(16.0);
    let viewport = area::logical(86.0, 180.0);

    let (x, y, first_row_end, second_row_start, second_row_end) = {
        let display = engine.text_area_line_display(
            &area_model,
            area_model.buffer(),
            true,
            style,
            viewport,
            0,
        );
        let prepared = display.buffer.borrow();
        let runs = prepared.layout_runs().collect::<Vec<_>>();
        let groups = TextLayoutMap::visual_line_groups(&runs);
        assert!(
            groups.len() >= 2,
            "test text should wrap into at least two visual rows"
        );
        let first_range = visual_group_source_range(&runs, groups[0], display.source_start)
            .expect("first visual row should have glyphs");
        let second_range = visual_group_source_range(&runs, groups[1], display.source_start)
            .expect("second visual row should have glyphs");
        let first_run = &runs[groups[1].start];
        let first_glyph = first_run
            .glyphs
            .first()
            .expect("second visual row should have a first glyph");
        (
            first_glyph.x + first_glyph.w * 0.25,
            (groups[1].top + groups[1].bottom) * 0.5,
            first_range.end,
            second_range.start,
            second_range.end,
        )
    };

    assert!(second_row_start >= first_row_end);
    let hit = engine
        .text_area_position_at_for_area(
            &area_model,
            style,
            viewport,
            point::logical(x, y),
            ViewState::default(),
        )
        .expect("wrapped visual row hit should resolve to a caret");

    assert!(
        hit.index >= second_row_start && hit.index <= second_row_end,
        "hit index {} should be inside second visual row range {}..{} instead of first row ending at {}",
        hit.index,
        second_row_start,
        second_row_end,
        first_row_end
    );
}

#[test]
fn wrapped_line_boundary_pointer_carets_preserve_visual_affinity() {
    let mut engine = engine();
    let text = "abcdefghijklmnopqrstuvwxyz";
    let source = Buffer::from_multiline_text(text);
    let area_model = Area::new(source.clone());
    let style = Style::default().with_size(18.0);
    let viewport = area::logical(54.0, 180.0);

    let (previous_row_end, next_row_start) = {
        let display = engine.text_area_line_display(
            &area_model,
            area_model.buffer(),
            true,
            style,
            viewport,
            0,
        );
        let prepared = display.buffer.borrow();
        let runs = prepared.layout_runs().collect::<Vec<_>>();
        let groups = TextLayoutMap::visual_line_groups(&runs);
        assert!(
            groups.len() >= 2,
            "test text should wrap into at least two visual rows"
        );
        let map = TextLayoutMap::from_line_starts(Rc::new(vec![display.source_start]));
        let first_right = runs[groups[0].start..groups[0].end]
            .iter()
            .flat_map(|run| run.glyphs.iter())
            .map(|glyph| glyph.x + glyph.w)
            .fold(f32::NEG_INFINITY, f32::max);
        let second_left = runs[groups[1].start..groups[1].end]
            .iter()
            .flat_map(|run| run.glyphs.iter())
            .map(|glyph| glyph.x)
            .fold(f32::INFINITY, f32::min);
        let previous_row_end = map
            .hit(
                &prepared,
                first_right + 1.0,
                (groups[0].top + groups[0].bottom) * 0.5,
            )
            .expect("first-row end should resolve to a caret");
        let next_row_start = map
            .hit(
                &prepared,
                second_left - 1.0,
                (groups[1].top + groups[1].bottom) * 0.5,
            )
            .expect("second-row start should resolve to a caret");

        for position in [previous_row_end, next_row_start] {
            let local = Cursor::new_with_affinity(
                0,
                position.index - display.source_start,
                position.affinity,
            );
            assert_eq!(
                clamp_cursor_in_buffer(&prepared, local).affinity,
                position.affinity
            );
        }
        (previous_row_end, next_row_start)
    };

    assert_eq!(previous_row_end.index, next_row_start.index);
    assert_eq!(previous_row_end.affinity, Affinity::Upstream);
    assert_eq!(next_row_start.affinity, Affinity::Downstream);

    let buffer = source;
    let mut state = buffer.initial_state();
    for position in [previous_row_end, next_row_start] {
        apply_selection(
            &buffer,
            &mut state,
            selection::Operation::pointer(PointerKind::Click, position),
        );
        assert_eq!(buffer.position_for_state(state), position);
    }
}

#[test]
fn wrapped_text_area_drag_selection_extends_into_lower_visual_row() {
    let mut engine = engine();
    let text = "alpha beta gamma delta epsilon zeta eta theta iota kappa lambda mu";
    let buffer = Buffer::from_multiline_text(text);
    let mut edit_state = buffer.initial_state();
    let area_model = Area::new(buffer.clone());
    let style = Style::default().with_size(16.0);
    let viewport = area::logical(86.0, 180.0);
    let (start_point, end_point, first_row_end, second_row_top) = {
        let display = engine.text_area_line_display(
            &area_model,
            area_model.buffer(),
            true,
            style,
            viewport,
            0,
        );
        let prepared = display.buffer.borrow();
        let runs = prepared.layout_runs().collect::<Vec<_>>();
        let groups = TextLayoutMap::visual_line_groups(&runs);
        assert!(
            groups.len() >= 2,
            "test text should wrap into at least two visual rows"
        );
        let first_range = visual_group_source_range(&runs, groups[0], display.source_start)
            .expect("first visual row should have glyphs");
        let first_run = &runs[groups[0].start];
        let second_run = &runs[groups[1].start];
        let start_glyph = first_run
            .glyphs
            .first()
            .expect("first visual row should have a first glyph");
        let end_glyph = second_run
            .glyphs
            .last()
            .expect("second visual row should have a last glyph");
        (
            point::logical(
                start_glyph.x + start_glyph.w * 0.25,
                (groups[0].top + groups[0].bottom) * 0.5,
            ),
            point::logical(
                end_glyph.x + end_glyph.w * 0.75,
                (groups[1].top + groups[1].bottom) * 0.5,
            ),
            first_range.end,
            groups[1].top,
        )
    };

    let start = engine
        .text_area_position_at_for_area(
            &area_model,
            style,
            viewport,
            start_point,
            ViewState::default(),
        )
        .expect("drag start should resolve to a caret");
    let end = engine
        .text_area_position_at_for_area(
            &area_model,
            style,
            viewport,
            end_point,
            ViewState::default(),
        )
        .expect("drag end should resolve to a caret");

    apply_selection(
        &buffer,
        &mut edit_state,
        selection::Operation::pointer(PointerKind::Click, start),
    );
    apply_selection(
        &buffer,
        &mut edit_state,
        selection::Operation::pointer(PointerKind::Drag, end),
    );

    let selected = buffer
        .selected_range_for_state(edit_state)
        .expect("drag across wrapped rows should create a selection");
    assert!(
        selected.end > first_row_end,
        "selection {:?} should extend beyond first visual row ending at {}",
        selected,
        first_row_end
    );

    let selected_area = Area::new(buffer.clone()).with_wrap(area_model.wrap());
    let selected_area = selected_area.with_state(edit_state);
    let layout = engine.text_area_paint_layout_for_area_at(
        &selected_area,
        style,
        viewport,
        ViewState::default(),
        Instant::now(),
    );
    assert!(
        layout
            .layout()
            .selection_spans()
            .iter()
            .any(|span| (span.y() - second_row_top).abs() <= TEXT_LAYOUT_VISUAL_LINE_EPSILON),
        "selection highlight should include the lower wrapped visual row"
    );
}
#[test]
fn text_area_metrics_reuse_measured_wrapped_heights_after_paint() {
    let mut engine = engine();
    let long = "wrap ".repeat(40);
    let area_model = Area::new(Buffer::from_multiline_text(format!("{long}\nnext")));
    let style = Style::default().with_size(16.0);
    let viewport = area::logical(72.0, 24.0);
    let state = ViewState::default();

    let cold = engine.text_area_metrics_layout_for_area_at(
        &area_model,
        style,
        viewport,
        state.clone(),
        Instant::now(),
    );
    let _paint = engine.text_area_paint_layout_for_area_at(
        &area_model,
        style,
        viewport,
        state.clone(),
        Instant::now(),
    );
    let warm = engine.text_area_metrics_layout_for_area_at(
        &area_model,
        style,
        viewport,
        state,
        Instant::now(),
    );

    assert!(
        warm.content_area().height() > cold.content_area().height(),
        "painted wrapped line measurements should refine content height from {} to more than it, got {}",
        cold.content_area().height(),
        warm.content_area().height()
    );
}
#[test]
fn text_area_overlay_cache_key_tracks_scroll_window() {
    let mut engine = engine();
    let text = (0..1_000)
        .map(|index| format!("line {index}"))
        .collect::<Vec<_>>()
        .join("\n");
    let buffer = Buffer::from_multiline_text(text);
    let mut edit_state = buffer.initial_state();
    apply_selection(&buffer, &mut edit_state, selection::Operation::SelectAll);
    let area_model = Area::new(buffer).with_state(edit_state);
    let style = Style::default().with_size(13.0);
    let viewport = area::logical(240.0, 52.0);
    let line_height = 13.0 * 1.25;
    let now = Instant::now();

    engine.reset_highlight_stats();
    engine.text_area_paint_layout_for_area_at(
        &area_model,
        style,
        viewport,
        ViewState::default(),
        now,
    );
    let first = engine.highlight_stats();
    assert!(first.run_scans > 0);
    assert_eq!(first.highlight_calls, 0);

    engine.reset_highlight_stats();
    let scrolled_state = ViewState::default().with_scroll_y(line_height * 100.0);
    engine.text_area_paint_layout_for_area_at(
        &area_model,
        style,
        viewport,
        scrolled_state.clone(),
        now,
    );
    let scrolled = engine.highlight_stats();
    assert!(scrolled.run_scans > 0);
    assert_eq!(scrolled.highlight_calls, 0);

    engine.reset_highlight_stats();
    engine.text_area_paint_layout_for_area_at(&area_model, style, viewport, scrolled_state, now);
    let cached = engine.highlight_stats();
    assert!(cached.run_scans > 0);
    assert_eq!(cached.highlight_calls, 0);
}
#[test]
fn offscreen_text_area_selection_skips_run_highlight_calls() {
    let mut engine = engine();
    let text = (0..1_000)
        .map(|index| format!("line {index}"))
        .collect::<Vec<_>>()
        .join("\n");
    let buffer = Buffer::from_multiline_text(text);
    let mut edit_state = buffer.initial_state();
    apply_selection(
        &buffer,
        &mut edit_state,
        selection::Operation::set_position(Position::new(0)),
    );
    apply_selection(
        &buffer,
        &mut edit_state,
        selection::Operation::extend_position(Motion::WordNext),
    );
    assert!(has_selection(&buffer, edit_state));
    let area_model = Area::new(buffer).with_state(edit_state);
    let style = Style::default().with_size(13.0);
    let viewport = area::logical(240.0, 52.0);
    let state = ViewState::default().with_scroll_y(13.0 * 1.25 * 500.0);

    engine.reset_highlight_stats();
    let layout = engine
        .text_area_paint_layout_for_area_at(&area_model, style, viewport, state, Instant::now())
        .into_interaction_parts()
        .0;
    let stats = engine.highlight_stats();

    assert!(layout.selection_spans().is_empty());
    assert!(stats.run_scans <= TEXT_AREA_FRAME_MAX_LOGICAL_LINES);
    assert_eq!(stats.highlight_calls, 0);
    assert_eq!(stats.spans, 0);
}

#[test]
fn fast_selection_check_matches_canonical_selected_range() {
    fn assert_matches(buffer: &Buffer, state: State) {
        assert_eq!(
            buffer.has_non_empty_selection_for_state(state),
            buffer.selected_range_for_state(state).is_some()
        );
    }

    let mut editor = Editor::new();

    let collapsed = Buffer::from_text("abc");
    let mut collapsed_state = collapsed.initial_state();
    let cursor = collapsed.cursor_for_text_index(1);
    collapsed.set_cursor_and_selection_for_state(
        &mut collapsed_state,
        cursor,
        CursorSelection::Normal(cursor),
    );
    assert_matches(&collapsed, collapsed_state);

    let mut single = Buffer::from_text("hello world");
    let mut single_state = single.initial_state();
    assert_matches(&single, single_state);
    apply_selection(&single, &mut single_state, selection::Operation::SelectAll);
    assert_matches(&single, single_state);
    apply_edit(
        &mut editor,
        &mut single,
        &mut single_state,
        Edit::insert("x"),
    );
    assert_matches(&single, single_state);

    let multiline = Buffer::from_multiline_text("one\ntwo\nthree");
    let mut multiline_state = multiline.initial_state();
    apply_selection(
        &multiline,
        &mut multiline_state,
        selection::Operation::set_position(Position::new(0)),
    );
    apply_selection(
        &multiline,
        &mut multiline_state,
        selection::Operation::extend_position(Motion::DocumentEnd),
    );
    assert_matches(&multiline, multiline_state);

    let word = Buffer::from_text("hello world");
    let mut word_state = word.initial_state();
    apply_selection(
        &word,
        &mut word_state,
        selection::Operation::pointer(PointerKind::DoubleClick, Position::new(1)),
    );
    assert_matches(&word, word_state);
}
#[test]
fn selection_operations_do_not_bump_revision_or_invalidate_surfaces() {
    let mut engine = engine();
    let text = (0..200)
        .map(|index| format!("line {index}"))
        .collect::<Vec<_>>()
        .join("\n");
    let buffer = Buffer::from_multiline_text(text);
    let mut edit_state = buffer.initial_state();
    let area_model = Area::new(buffer.clone());
    let style = Style::default().with_size(13.0);
    let viewport = area::logical(240.0, 52.0);
    engine.text_area_paint_layout_for_area_at(
        &area_model,
        style,
        viewport,
        ViewState::default(),
        Instant::now(),
    );
    let revision = buffer.revision();
    let cached_frames = engine.text_area_line_displays.len();

    let set = apply_selection(
        &buffer,
        &mut edit_state,
        selection::Operation::set_position(0),
    );
    assert!(set);
    assert_eq!(buffer.revision(), revision);
    assert_eq!(engine.text_area_line_displays.len(), cached_frames);

    let drag = apply_selection(
        &buffer,
        &mut edit_state,
        selection::Operation::pointer(PointerKind::Drag, Position::new(20)),
    );
    assert!(drag);
    assert!(selected_range(&buffer, edit_state).is_some());
    assert_eq!(buffer.revision(), revision);
    assert_eq!(engine.text_area_line_displays.len(), cached_frames);
}

#[test]
fn text_area_hit_testing_refreshes_cached_line_offsets_after_edit_above() {
    let mut engine = engine();
    let mut editor = Editor::new();
    let mut buffer = Buffer::from_multiline_text("abcdefghij\nclick target\nlast line");
    let mut edit_state = buffer.initial_state();
    let style = Style::default().with_size(16.0);
    let viewport = area::logical(320.0, 120.0);

    engine.text_area_paint_layout_for_area_at(
        &Area::new(buffer.clone()),
        style,
        viewport,
        ViewState::default(),
        Instant::now(),
    );
    assert!(
        engine.text_area_line_displays.len() >= 2,
        "warm paint should cache multiple line displays"
    );

    let result = apply_edit_with_result(
        &mut editor,
        &mut buffer,
        &mut edit_state,
        Edit::replace_range(0..4, ""),
    );
    assert!(result.text_changed);
    let area_model = Area::new(buffer.clone());
    let paint_layout = engine.text_area_paint_layout_for_area_at(
        &area_model,
        style,
        viewport,
        ViewState::default(),
        Instant::now(),
    );
    let target_y = paint_layout
        .interaction_surfaces()
        .iter()
        .find(|surface| {
            let buffer = surface.buffer();
            let buffer = buffer.borrow();
            buffer
                .lines
                .first()
                .is_some_and(|line| line.text() == "click target")
        })
        .map(|surface| surface.y() + surface.height() * 0.5)
        .expect("target line should be visible after edit");

    let expected_current_start = "efghij\n".len();
    let stale_start_before_delete = "abcdefghij\n".len();
    assert_ne!(expected_current_start, stale_start_before_delete);

    engine.reset_interaction_stats();
    let hit = engine
        .text_area_position_at_for_area(
            &area_model,
            style,
            viewport,
            point::logical(1.0, target_y),
            ViewState::default(),
        )
        .expect("clicking visible lower line should resolve a caret");
    let stats = engine.interaction_stats();
    assert!(
        stats.text_area_frame_cache_hits > 0,
        "hit testing should reuse warmed line displays: {stats:?}"
    );
    assert_eq!(hit.index, expected_current_start);
    assert_ne!(hit.index, stale_start_before_delete);
}

#[test]
fn text_area_hit_testing_uses_current_line_order_after_line_delete_above() {
    let mut engine = engine();
    let mut editor = Editor::new();
    let lines = (0..80)
        .map(|line| {
            if line == 30 {
                "click target".to_owned()
            } else {
                format!("line {line:02}")
            }
        })
        .collect::<Vec<_>>();
    let text = lines.join("\n");
    let mut buffer = Buffer::from_multiline_text(text);
    let mut edit_state = buffer.initial_state();
    let style = Style::default().with_size(16.0);
    let viewport = area::logical(320.0, 120.0);
    let state = ViewState::default().with_scroll(0.0, 500.0);
    let now = Instant::now();

    engine.text_area_paint_layout_for_area_at(
        &Area::new(buffer.clone()),
        style,
        viewport,
        state.clone(),
        now,
    );

    let delete_len = lines[0].len() + 1 + lines[1].len() + 1 + lines[2].len() + 1;
    let result = apply_edit_with_result(
        &mut editor,
        &mut buffer,
        &mut edit_state,
        Edit::replace_range(0..delete_len, ""),
    );
    assert!(result.text_changed);
    let expected_current_start = buffer
        .text()
        .find("click target")
        .expect("target should remain after deleting lines above it");
    let stale_start_before_delete = expected_current_start + delete_len;
    assert_ne!(expected_current_start, stale_start_before_delete);

    let area_model = Area::new(buffer.clone());
    engine.reset_interaction_stats();
    let paint_layout =
        engine.text_area_paint_layout_for_area_at(&area_model, style, viewport, state.clone(), now);
    let paint_stats = engine.interaction_stats();
    assert!(
        paint_stats.text_area_frame_cache_hits > 0,
        "line delete should preserve lower-line cache hits: {paint_stats:?}"
    );
    let target_y = paint_layout
        .interaction_surfaces()
        .iter()
        .find(|surface| {
            let buffer = surface.buffer();
            let buffer = buffer.borrow();
            buffer
                .lines
                .first()
                .is_some_and(|line| line.text() == "click target")
        })
        .map(|surface| surface.y() + surface.height() * 0.5)
        .expect("target line should be visible after deleting lines above it");

    let observed_hit = engine
        .text_area_position_at_for_paint_layout(
            &area_model,
            point::logical(1.0, target_y),
            state.clone(),
            &paint_layout,
        )
        .expect("observed painted layout should resolve the target line");
    assert_eq!(observed_hit.index, expected_current_start);
    assert_ne!(observed_hit.index, stale_start_before_delete);

    engine.reset_interaction_stats();
    let fallback_hit = engine
        .text_area_position_at_for_area(
            &area_model,
            style,
            viewport,
            point::logical(1.0, target_y),
            state,
        )
        .expect("fallback hit testing should resolve the target line");
    let fallback_stats = engine.interaction_stats();
    assert!(
        fallback_stats.text_area_frame_cache_hits > 0,
        "fallback hit testing should reuse warmed lower-line displays: {fallback_stats:?}"
    );
    assert_eq!(fallback_hit.index, expected_current_start);
    assert_ne!(fallback_hit.index, stale_start_before_delete);
}

#[test]
fn text_area_hit_testing_uses_current_line_order_after_line_insert_above() {
    let mut engine = engine();
    let mut editor = Editor::new();
    let lines = (0..80)
        .map(|line| {
            if line == 24 {
                "click target".to_owned()
            } else {
                format!("line {line:02}")
            }
        })
        .collect::<Vec<_>>();
    let text = lines.join("\n");
    let mut buffer = Buffer::from_multiline_text(text);
    let mut edit_state = buffer.initial_state();
    let style = Style::default().with_size(16.0);
    let viewport = area::logical(320.0, 120.0);
    let state = ViewState::default().with_scroll(0.0, 480.0);
    let now = Instant::now();

    engine.text_area_paint_layout_for_area_at(
        &Area::new(buffer.clone()),
        style,
        viewport,
        state.clone(),
        now,
    );

    let inserted = "inserted a\ninserted b\n";
    let result = apply_edit_with_result(
        &mut editor,
        &mut buffer,
        &mut edit_state,
        Edit::replace_range(0..0, inserted),
    );
    assert!(result.text_changed);
    let expected_current_start = buffer
        .text()
        .find("click target")
        .expect("target should remain after inserting lines above it");
    let stale_start_before_insert = expected_current_start - inserted.len();
    assert_ne!(expected_current_start, stale_start_before_insert);

    let area_model = Area::new(buffer.clone());
    engine.reset_interaction_stats();
    let paint_layout =
        engine.text_area_paint_layout_for_area_at(&area_model, style, viewport, state.clone(), now);
    let paint_stats = engine.interaction_stats();
    assert!(
        paint_stats.text_area_frame_cache_hits > 0,
        "line insert should preserve lower-line cache hits: {paint_stats:?}"
    );
    let target_y = paint_layout
        .interaction_surfaces()
        .iter()
        .find(|surface| {
            let buffer = surface.buffer();
            let buffer = buffer.borrow();
            buffer
                .lines
                .first()
                .is_some_and(|line| line.text() == "click target")
        })
        .map(|surface| surface.y() + surface.height() * 0.5)
        .expect("target line should be visible after inserting lines above it");

    let observed_hit = engine
        .text_area_position_at_for_paint_layout(
            &area_model,
            point::logical(1.0, target_y),
            state,
            &paint_layout,
        )
        .expect("observed painted layout should resolve the target line");
    assert_eq!(observed_hit.index, expected_current_start);
    assert_ne!(observed_hit.index, stale_start_before_insert);
}

#[test]
fn text_area_observed_hit_testing_uses_observed_horizontal_scroll() {
    let mut engine = engine();
    let buffer = Buffer::from_multiline_text("abcdefghijklmnopqrstuvwxyz");
    let area_model = Area::new(buffer).no_wrap();
    let style = Style::default().with_size(16.0);
    let viewport = area::logical(120.0, 48.0);
    let observed_state = ViewState::default().with_scroll(80.0, 0.0);
    let stale_state = ViewState::default();
    let paint_layout = engine.text_area_paint_layout_for_area_at(
        &area_model,
        style,
        viewport,
        observed_state,
        Instant::now(),
    );

    let observed_hit = engine
        .text_area_position_at_for_paint_layout(
            &area_model,
            point::logical(0.0, 8.0),
            stale_state.clone(),
            &paint_layout,
        )
        .expect("observed paint layout should hit with observed scroll");
    let stale_hit = engine
        .text_area_position_at_for_area(
            &area_model,
            style,
            viewport,
            point::logical(0.0, 8.0),
            stale_state,
        )
        .expect("fallback hit should use state scroll");

    assert!(
        observed_hit.index > stale_hit.index,
        "observed hit should use paint-layout scroll, not stale state scroll: observed={observed_hit:?} stale={stale_hit:?}"
    );
}

#[test]
fn text_area_hit_testing_uses_nearest_caret_in_empty_space() {
    let mut engine = engine();
    let buffer = Buffer::from_multiline_text("one\ntwo");
    let area_model = Area::new(buffer.clone());
    let style = Style::default().with_size(16.0);
    let viewport = area::logical(240.0, 120.0);

    let below_near_start = engine
        .text_area_position_at_for_area(
            &area_model,
            style,
            viewport,
            point::logical(-4.0, 100.0),
            ViewState::default(),
        )
        .expect("click below short text should resolve on the nearest line");
    assert_eq!(below_near_start.index, "one\n".len());

    let below_far_right = engine
        .text_area_position_at_for_area(
            &area_model,
            style,
            viewport,
            point::logical(220.0, 100.0),
            ViewState::default(),
        )
        .expect("click below short text should still honor x on the nearest line");
    assert_eq!(below_far_right.index, buffer.text().len());

    let right_of_first_line = engine
        .text_area_position_at_for_area(
            &area_model,
            style,
            viewport,
            point::logical(220.0, 8.0),
            ViewState::default(),
        )
        .expect("click to the right of a line should resolve to a caret");
    assert_eq!(right_of_first_line.index, "one".len());

    let above_near_start = engine
        .text_area_position_at_for_area(
            &area_model,
            style,
            viewport,
            point::logical(-4.0, -8.0),
            ViewState::default(),
        )
        .expect("click above text should resolve on the nearest line");
    assert_eq!(above_near_start.index, 0);

    let above_far_right = engine
        .text_area_position_at_for_area(
            &area_model,
            style,
            viewport,
            point::logical(220.0, -8.0),
            ViewState::default(),
        )
        .expect("click above text should still honor x on the nearest line");
    assert_eq!(above_far_right.index, "one".len());

    let empty = Area::new(Buffer::from_multiline_text(""));
    let empty_hit = engine
        .text_area_position_at_for_area(
            &empty,
            style,
            viewport,
            point::logical(12.0, 80.0),
            ViewState::default(),
        )
        .expect("empty text area should still resolve to a caret");
    assert_eq!(empty_hit.index, 0);
}

#[test]
fn mixed_direction_line_edges_preserve_affinity_for_nearest_line_hits() {
    let mut engine = engine();
    let buffer = Buffer::from_multiline_text("abc אבג\nxyz");
    let area_model = Area::new(buffer);
    let style = Style::default().with_size(18.0);
    let viewport = area::logical(280.0, 120.0);
    let display =
        engine.text_area_line_display(&area_model, area_model.buffer(), true, style, viewport, 0);
    let prepared = display.buffer.borrow();
    let map = TextLayoutMap::from_line_starts(Rc::new(vec![display.source_start]));
    let runs = prepared.layout_runs().collect::<Vec<_>>();
    assert!(
        runs.iter()
            .any(|run| run.glyphs.iter().any(|glyph| glyph.level.is_rtl()))
    );

    let mut left_edge = None::<(f32, &glyphon::cosmic_text::LayoutRun<'_>)>;
    let mut right_edge = None::<(f32, &glyphon::cosmic_text::LayoutRun<'_>)>;
    for run in &runs {
        let Some((left, right)) = TextLayoutMap::run_visual_bounds(run) else {
            continue;
        };
        if left_edge.is_none_or(|(best, _)| left < best) {
            left_edge = Some((left, run));
        }
        if right_edge.is_none_or(|(best, _)| right > best) {
            right_edge = Some((right, run));
        }
    }
    let (left, left_run) = left_edge.expect("mixed line should have a left visual edge");
    let (right, right_run) = right_edge.expect("mixed line should have a right visual edge");
    let y = runs[0].line_top - runs[0].line_height;

    let left_hit = map
        .hit(&prepared, left - 8.0, y)
        .expect("above-line left edge should resolve to a caret");
    let right_hit = map
        .hit(&prepared, right + 8.0, y)
        .expect("above-line right edge should resolve to a caret");

    assert_eq!(left_hit, map.run_edge_position(left_run, true).unwrap());
    assert_eq!(right_hit, map.run_edge_position(right_run, false).unwrap());
}

#[test]
fn repeated_large_text_area_hit_tests_reuse_cached_frame() {
    let mut engine = engine();
    let text = (0..5_000)
        .map(|index| format!("line {index}"))
        .collect::<Vec<_>>()
        .join("\n");
    let area_model = Area::new(Buffer::from_multiline_text(text));
    let style = Style::default().with_size(13.0);
    let viewport = area::logical(240.0, 52.0);
    let state = ViewState::default();

    engine.reset_interaction_stats();
    let first = engine.text_area_position_at_for_area(
        &area_model,
        style,
        viewport,
        point::logical(16.0, 18.0),
        state.clone(),
    );
    let second = engine.text_area_position_at_for_area(
        &area_model,
        style,
        viewport,
        point::logical(18.0, 18.0),
        state.clone(),
    );
    let stats = engine.interaction_stats();

    assert!(first.is_some());
    assert!(second.is_some());
    assert!(stats.text_area_frame_cache_misses <= TEXT_AREA_FRAME_MAX_LOGICAL_LINES);
    assert!(stats.text_area_frame_cache_hits > 0);
    assert!(stats.text_area_frame_shape_calls <= TEXT_AREA_FRAME_MAX_LOGICAL_LINES);
    assert!(stats.text_area_frame_shaped_logical_lines <= TEXT_AREA_FRAME_MAX_LOGICAL_LINES);
    assert_eq!(stats.text_area_shape_until_scroll_calls, 0);
    assert!(stats.hit_run_scans <= stats.text_area_frame_shaped_visual_lines * 2);
}

#[test]
fn warmed_large_text_area_hit_test_does_not_reshape_visible_window() {
    let mut engine = engine();
    let text = (0..5_000)
        .map(|index| format!("line {index}"))
        .collect::<Vec<_>>()
        .join("\n");
    let area_model = Area::new(Buffer::from_multiline_text(text));
    let style = Style::default().with_size(13.0);
    let viewport = area::logical(240.0, 52.0);
    let state = ViewState::default();

    let _ = engine.text_area_position_at_for_area(
        &area_model,
        style,
        viewport,
        point::logical(16.0, 18.0),
        state.clone(),
    );
    engine.reset_interaction_stats();
    let hit = engine.text_area_position_at_for_area(
        &area_model,
        style,
        viewport,
        point::logical(20.0, 18.0),
        state,
    );
    let stats = engine.interaction_stats();

    assert!(hit.is_some());
    assert_eq!(stats.text_area_shape_until_scroll_calls, 0);
    assert!(stats.text_area_frame_cache_hits > 0);
    assert_eq!(stats.text_area_frame_cache_misses, 0);
    assert_eq!(stats.text_area_frame_shape_calls, 0);
    assert!(stats.hit_run_scans <= TEXT_AREA_FRAME_MAX_LOGICAL_LINES);
}

#[test]
fn text_area_preedit_reveal_scroll_uses_composed_projection() {
    let mut engine = engine();
    let buffer = Buffer::from_multiline_text("one\ntwo");
    let mut edit_state = buffer.initial_state();
    let end = buffer.position_for_text_index(buffer.text().len());
    apply_selection(
        &buffer,
        &mut edit_state,
        selection::Operation::set_position(end),
    );
    let area_model = Area::new(buffer).with_state(edit_state);
    let style = Style::default().with_size(16.0);
    let viewport = area::logical(120.0, 36.0);
    let state =
        ViewState::default().with_preedit(Some(Preedit::new("\nthree\nfour\nfive\nsix", None)));

    let revealed =
        engine.ensure_caret_visible_for_area(&area_model, style, viewport, state.clone(), None);
    let layout = engine
        .text_area_paint_layout_for_area_at(
            &area_model,
            style,
            viewport,
            revealed.clone(),
            Instant::now(),
        )
        .into_interaction_parts()
        .0;

    assert!(revealed.scroll_y() > 0.0);
    assert!(!layout.preedit_underline_spans().is_empty());
}

#[test]
fn obscured_text_field_hit_testing_maps_display_cursor_to_source_cursor() {
    let mut engine = engine();
    let field = Field::new("åb").obscured_dot();
    let position = engine
        .text_field_position_at_for_field(
            &field,
            Style::default().with_size(16.0),
            area::logical(200.0, 24.0),
            point::logical(200.0, 8.0),
            ViewState::default(),
        )
        .expect("hit testing should return a position");

    assert_eq!(field.presentation_text(), "••");
    assert_eq!(field.buffer().text(), "åb");
    assert_eq!(position.index, field.buffer().text().len());
}

#[test]
fn empty_obscured_text_field_has_no_phantom_dot() {
    let field = Field::new("").obscured_dot();

    assert!(field.buffer().is_empty());
    assert_eq!(field.presentation_text(), "");
    assert_eq!(super::unicode::source_grapheme_boundaries(""), vec![0]);
}

#[test]
fn ensure_caret_visible_keeps_caret_inside_content_rect() {
    let mut engine = engine();
    let buffer = Buffer::from_text("hello world this is a long single-line field");
    let field = Field::new(buffer);
    let area = area::logical(80.0, 32.0);
    let state = engine.ensure_caret_visible_for_field(
        &field,
        Style::default().with_size(16.0),
        area,
        ViewState::default(),
    );

    assert!(state.scroll_x() > 0.0);

    let layout =
        engine.text_field_layout_for_field(&field, Style::default().with_size(16.0), area, state);
    let caret = layout.caret().expect("focused long text should have caret");

    assert!(caret.x() >= 0.0);
    assert!(caret.x() <= area.width());
}

#[test]
fn text_field_caret_visibility_follows_blink_phase() {
    let mut engine = engine();
    let buffer = Buffer::from_text("hello");
    let field = Field::new(buffer);
    let area = area::logical(100.0, 24.0);
    let epoch = Instant::now();
    let state = ViewState::new_at(0.0, epoch);

    let visible = engine.text_field_layout_for_field_at(
        &field,
        Style::default().with_size(16.0),
        area,
        state.clone(),
        epoch,
    );
    let hidden = engine.text_field_layout_for_field_at(
        &field,
        Style::default().with_size(16.0),
        area,
        state.clone(),
        epoch + Duration::from_millis(500),
    );
    let visible_again = engine.text_field_layout_for_field_at(
        &field,
        Style::default().with_size(16.0),
        area,
        state,
        epoch + Duration::from_millis(1000),
    );

    assert!(visible.caret().is_some());
    assert_eq!(hidden.caret(), None);
    assert!(visible_again.caret().is_some());
}

#[test]
fn text_field_selection_suppresses_caret_layout() {
    let mut engine = engine();
    let buffer = Buffer::from_text("hello");
    let mut edit_state = buffer.initial_state();
    let area = area::logical(100.0, 24.0);
    let epoch = Instant::now();

    apply_selection(&buffer, &mut edit_state, selection::Operation::SelectAll);
    let field = Field::new(buffer.clone()).with_state(edit_state);

    let layout = engine.text_field_layout_for_field_at(
        &field,
        Style::default().with_size(16.0),
        area,
        ViewState::new_at(0.0, epoch),
        epoch,
    );

    assert_eq!(layout.caret(), None);
    assert!(!layout.selection_spans().is_empty());
}

#[test]
fn multiline_buffer_preserves_line_breaks_and_enter_inserts_newline() {
    let mut editor = Editor::new();
    let mut buffer = Buffer::from_multiline_text("one\r\ntwo\rthree");
    let mut edit_state = buffer.initial_state();

    assert_eq!(buffer.text(), "one\ntwo\nthree");

    let end = buffer.position_for_text_index(buffer.text().len());
    apply_selection(
        &buffer,
        &mut edit_state,
        selection::Operation::set_position(end),
    );
    apply_edit(
        &mut editor,
        &mut buffer,
        &mut edit_state,
        Edit::insert_line_break(),
    );
    apply_edit(
        &mut editor,
        &mut buffer,
        &mut edit_state,
        Edit::insert("four\nfive"),
    );

    assert_eq!(buffer.text(), "one\ntwo\nthree\nfour\nfive");
}

#[test]
fn empty_multiline_buffer_accepts_line_breaks() {
    let mut editor = Editor::new();
    let mut buffer = Buffer::new_multiline();
    let mut edit_state = buffer.initial_state();

    assert_eq!(buffer.text(), "");
    assert!(buffer.is_multiline());

    apply_edit(
        &mut editor,
        &mut buffer,
        &mut edit_state,
        Edit::insert("one"),
    );
    apply_edit(
        &mut editor,
        &mut buffer,
        &mut edit_state,
        Edit::insert_line_break(),
    );
    apply_edit(
        &mut editor,
        &mut buffer,
        &mut edit_state,
        Edit::insert("two"),
    );

    assert_eq!(buffer.text(), "one\ntwo");
}

#[test]
fn text_area_promotes_its_owned_buffer_value_to_multiline() {
    let mut editor = Editor::new();
    let mut buffer = Buffer::from_text("hello");
    let mut edit_state = buffer.initial_state();
    let original_id = buffer.id();

    apply_selection(
        &buffer,
        &mut edit_state,
        selection::Operation::set_position(Position::new(0)),
    );
    apply_selection(
        &buffer,
        &mut edit_state,
        selection::Operation::extend_position(Motion::WordNext),
    );
    let before_position = buffer.position_for_state(edit_state);
    let before_selection = buffer.selected_range_for_state(edit_state);

    let area = Area::new(buffer.clone()).with_state(edit_state);

    assert_ne!(area.buffer().id(), original_id);
    assert_eq!(buffer.id(), original_id);
    assert!(area.buffer().is_multiline());
    assert!(!buffer.is_multiline());
    assert_eq!(buffer.text(), "hello");
    assert_eq!(area.buffer().text(), "hello");
    assert_eq!(buffer.position_for_state(edit_state), before_position);
    assert_eq!(
        buffer.selected_range_for_state(edit_state),
        before_selection
    );
    assert_eq!(
        area.buffer().position_for_state(area.state()),
        before_position
    );
    assert_eq!(
        area.buffer().selected_range_for_state(area.state()),
        before_selection
    );

    let end = buffer.len();
    apply_selection(
        &buffer,
        &mut edit_state,
        selection::Operation::set_position(Position::new(end)),
    );
    editor.apply_edit(&mut buffer, &mut edit_state, Edit::insert("!"));

    assert_eq!(buffer.text(), "hello!");
    assert_eq!(area.buffer().text(), "hello");
    assert_eq!(Area::new(buffer.clone()).buffer().text(), "hello!");
}

#[test]
fn multiline_select_all_selects_the_entire_document() {
    let buffer = Buffer::from_multiline_text("alpha\nbeta\ngamma");
    let mut edit_state = buffer.initial_state();

    apply_selection(&buffer, &mut edit_state, selection::Operation::SelectAll);

    assert_eq!(
        selected_text(&buffer, edit_state),
        Some("alpha\nbeta\ngamma".to_owned())
    );
    assert_eq!(
        selected_range(&buffer, edit_state),
        Some(Range::new(0, "alpha\nbeta\ngamma".len()))
    );
}

#[test]
fn text_area_reveal_scroll_uses_wrapped_visual_caret_row() {
    let mut engine = engine();
    let text = "alpha beta gamma delta epsilon zeta eta theta iota kappa lambda mu";
    let buffer = Buffer::from_multiline_text(text);
    let area_model = Area::new(buffer.clone());
    let style = Style::default().with_size(16.0);
    let viewport = area::logical(86.0, 24.0);
    let (cursor_index, second_row_top, row_height) = {
        let display = engine.text_area_line_display(
            &area_model,
            area_model.buffer(),
            true,
            style,
            viewport,
            0,
        );
        let prepared = display.buffer.borrow();
        let runs = prepared.layout_runs().collect::<Vec<_>>();
        let groups = TextLayoutMap::visual_line_groups(&runs);
        assert!(
            groups.len() >= 2,
            "test text should wrap into at least two visual rows"
        );
        let second_run = &runs[groups[1].start];
        let first_glyph = second_run
            .glyphs
            .first()
            .expect("second visual row should have a first glyph");
        (
            display.source_start + first_glyph.start,
            groups[1].top,
            second_run.line_height,
        )
    };

    let mut edit_state = buffer.initial_state();
    apply_selection(
        &buffer,
        &mut edit_state,
        selection::Operation::set_position(Position::new(cursor_index)),
    );
    let area_model = Area::new(buffer).with_state(edit_state);
    let scroll_y = (second_row_top - 2.0).max(0.0);
    let state = ViewState::default().with_scroll_y(scroll_y);
    let revealed = engine.ensure_caret_visible_for_area(&area_model, style, viewport, state, None);

    assert!(
        (revealed.scroll_y() - scroll_y).abs() <= TEXT_LAYOUT_VISUAL_LINE_EPSILON,
        "visible wrapped-row caret should not reveal to hard-line top: before {scroll_y}, after {}",
        revealed.scroll_y()
    );

    let layout = engine
        .text_area_paint_layout_for_area_at(
            &area_model,
            style,
            area::logical(viewport.width(), row_height + 4.0),
            revealed,
            Instant::now(),
        )
        .into_interaction_parts()
        .0;
    let caret = layout.caret().expect("wrapped row caret should be visible");
    assert!(caret.y() >= 0.0);
}
#[test]
fn text_area_reveal_scroll_keeps_caret_inside_vertical_viewport() {
    let mut engine = engine();
    let buffer = Buffer::from_multiline_text("one\ntwo\nthree\nfour\nfive\nsix");
    let mut edit_state = buffer.initial_state();
    let end = buffer.position_for_text_index(buffer.text().len());
    apply_selection(
        &buffer,
        &mut edit_state,
        selection::Operation::set_position(end),
    );

    let area_model = Area::new(buffer).with_state(edit_state);
    let viewport = area::logical(120.0, 36.0);
    let state = engine.ensure_caret_visible_for_area(
        &area_model,
        Style::default().with_size(16.0),
        viewport,
        ViewState::default(),
        None,
    );

    assert!(state.scroll_y() > 0.0);

    let paint_layout = engine.text_area_paint_layout_for_area_at(
        &area_model,
        Style::default().with_size(16.0),
        viewport,
        state,
        Instant::now(),
    );
    let caret = paint_layout
        .layout()
        .caret()
        .expect("area caret should be visible");

    assert!(caret.y() >= 0.0);
    assert!(caret.y() + caret.height() <= viewport.height() + TEXT_FIELD_CARET_MARGIN);
}

#[test]
fn text_area_ensure_caret_visible_preserves_visible_caret_scroll_after_backspace() {
    let mut engine = engine();
    let mut editor = Editor::new();
    let text = (0..40)
        .map(|line| format!("line {line:02} abc"))
        .collect::<Vec<_>>()
        .join("\n");
    let mut buffer = Buffer::from_multiline_text(text.clone());
    let cursor_index = text.find("line 20 abc").unwrap() + "line 20 abc".len();
    let mut edit_state = buffer.initial_state();
    apply_selection(
        &buffer,
        &mut edit_state,
        selection::Operation::set_position(Position::new(cursor_index)),
    );
    editor.apply_edit(&mut buffer, &mut edit_state, Edit::backspace());

    let area_model = Area::new(buffer).with_state(edit_state);
    let style = Style::default().with_size(16.0);
    let viewport = area::logical(200.0, 64.0);
    let scroll_y = text_area_estimated_line_height(style) * 18.0;
    let state = ViewState::default()
        .with_scroll_y(scroll_y)
        .ensure_caret_visible(Instant::now());

    let revealed = engine.ensure_caret_visible_for_area(&area_model, style, viewport, state, None);

    assert!(
        (revealed.scroll_y() - scroll_y).abs() <= TEXT_LAYOUT_VISUAL_LINE_EPSILON,
        "visible caret should preserve scroll after backspace: before {scroll_y}, after {}",
        revealed.scroll_y()
    );

    let layout = engine.text_area_paint_layout_for_area_at(
        &area_model,
        style,
        viewport,
        revealed,
        Instant::now(),
    );
    let caret = layout
        .layout()
        .caret()
        .expect("caret should remain visible after preserving scroll");
    assert!(caret.y() >= -TEXT_FIELD_CARET_MARGIN);
    assert!(caret.y() + caret.height() <= viewport.height() + TEXT_FIELD_CARET_MARGIN);
}

#[test]
fn large_wrapped_text_area_ensure_caret_visible_uses_observed_visible_caret_after_backspace() {
    let mut engine = engine();
    let mut editor = Editor::new();
    let wrapped = "alpha beta gamma delta epsilon zeta eta theta iota kappa lambda mu nu xi omicron pi rho sigma tau";
    let text = (0..2_000)
        .map(|line| format!("line {line:04} {wrapped}"))
        .collect::<Vec<_>>()
        .join("\n");
    let target_line = 900;
    let needle = format!("line {target_line:04} ");
    let cursor_index = text.find(&needle).unwrap() + needle.len();
    let mut buffer = Buffer::from_multiline_text(text);
    let mut edit_state = buffer.initial_state();
    apply_selection(
        &buffer,
        &mut edit_state,
        selection::Operation::set_position(Position::new(cursor_index)),
    );

    let style = Style::default().with_size(16.0);
    let viewport = area::logical(122.0, 72.0);
    let initial_scroll_y = text_area_estimated_line_height(style)
        * (target_line + TEXT_AREA_FRAME_MIN_OVERSCAN_LINES) as f32;
    let now = Instant::now();
    let mut state = ViewState::default()
        .with_scroll_y(initial_scroll_y)
        .ensure_caret_visible(now);

    let area_model = Area::new(buffer.clone()).with_state(edit_state);
    let mut warm_layout = None;
    for _ in 0..8 {
        let layout = engine.text_area_paint_layout_for_area_at(
            &area_model,
            style,
            viewport,
            state.clone(),
            now,
        );
        let caret = layout
            .layout()
            .caret()
            .expect("target caret should be in the warmed viewport");
        let visibility =
            Viewport::new(viewport, point::logical(state.scroll_x(), state.scroll_y()))
                .visibility_of_local_caret(caret, TEXT_FIELD_CARET_MARGIN);
        if visibility.is_visible() {
            warm_layout = Some(layout);
            break;
        }

        let mut next_scroll_y = state.scroll_y();
        match visibility {
            Visibility::Above => {
                next_scroll_y = next_scroll_y + caret.y() - TEXT_FIELD_CARET_MARGIN;
            }
            Visibility::Below => {
                next_scroll_y =
                    next_scroll_y + caret.y() + caret.height() + TEXT_FIELD_CARET_MARGIN
                        - viewport.height();
            }
            Visibility::Visible | Visibility::Before | Visibility::After | Visibility::Unknown => {}
        }
        state = state.with_scroll_y(next_scroll_y.max(0.0));
    }

    let warm_layout = warm_layout.expect("target caret should settle into view");
    let warm_caret = warm_layout
        .layout()
        .caret()
        .expect("target caret should be in the warmed viewport");
    let warm_visibility =
        Viewport::new(viewport, point::logical(state.scroll_x(), state.scroll_y()))
            .visibility_of_local_caret(warm_caret, TEXT_FIELD_CARET_MARGIN);
    assert!(
        warm_visibility.is_visible(),
        "warm caret {warm_caret:?} should be visible, got {warm_visibility:?}"
    );
    let scroll_y = state.scroll_y();

    let expected_misses_per_backspace = 2;
    let mut observed = None;
    for step in 0..3 {
        let result = editor.apply_edit(&mut buffer, &mut edit_state, Edit::backspace());
        assert_eq!(
            result.impacts.len(),
            1,
            "backspace {step} should report one edit impact"
        );
        assert_eq!(
            result.impacts[0].affected_line_count(),
            1,
            "backspace {step} should dirty only the edited logical line"
        );
        assert!(result.impacts[0].affected_start_line_id.is_some());

        let area_model = Area::new(buffer.clone()).with_state(edit_state);
        engine.reset_interaction_stats();
        let next = engine.text_area_paint_layout_for_area_at(
            &area_model,
            style,
            viewport,
            state.clone(),
            now,
        );
        let paint_stats = engine.interaction_stats();
        assert!(
            paint_stats.text_area_frame_cache_hits > 0,
            "backspace {step} should reuse unaffected visible line displays"
        );
        assert!(
            paint_stats.text_area_frame_cache_misses <= expected_misses_per_backspace,
            "backspace {step} should not cold-cache the whole viewport: {paint_stats:?}"
        );
        assert!(
            paint_stats.text_area_frame_shape_calls <= expected_misses_per_backspace,
            "backspace {step} should shape only touched lines and edge fill: {paint_stats:?}"
        );
        observed = Some(next);
    }

    let area_model = Area::new(buffer).with_state(edit_state);
    let observed = observed.expect("backspace loop should produce an observed layout");
    let observed_caret = observed
        .layout()
        .caret()
        .expect("caret should remain visible after backspace");
    assert!(
        Viewport::new(viewport, point::logical(state.scroll_x(), state.scroll_y()))
            .visibility_of_local_caret(observed_caret, TEXT_FIELD_CARET_MARGIN)
            .is_visible()
    );

    engine.reset_interaction_stats();
    let revealed = engine.ensure_caret_visible_for_area(
        &area_model,
        style,
        viewport,
        state.clone(),
        Some(observed.layout()),
    );
    let stats = engine.interaction_stats();

    assert!(
        (revealed.scroll_y() - scroll_y).abs() <= TEXT_LAYOUT_VISUAL_LINE_EPSILON,
        "observed visible caret should preserve scroll after large wrapped backspace: before {scroll_y}, after {}",
        revealed.scroll_y()
    );
    assert_eq!(stats.text_area_frame_shape_calls, 0);
    assert_eq!(stats.text_area_frame_cache_misses, 0);
}

#[test]
fn text_area_ensure_caret_visible_uses_observed_hidden_caret_minimally() {
    let mut engine = engine();
    let text = (0..40)
        .map(|line| format!("line {line:02}"))
        .collect::<Vec<_>>()
        .join("\n");
    let buffer = Buffer::from_multiline_text(text.clone());
    let cursor_index = text.find("line 08").unwrap() + "line 08".len();
    let mut edit_state = buffer.initial_state();
    apply_selection(
        &buffer,
        &mut edit_state,
        selection::Operation::set_position(Position::new(cursor_index)),
    );

    let area_model = Area::new(buffer).with_state(edit_state);
    let style = Style::default().with_size(16.0);
    let viewport = area::logical(200.0, 36.0);
    let now = Instant::now();
    let state = ViewState::default().ensure_caret_visible(now);
    let observed =
        engine.text_area_paint_layout_for_area_at(&area_model, style, viewport, state.clone(), now);
    let caret = observed
        .layout()
        .caret()
        .expect("overscan should include the below-viewport caret");
    assert!(matches!(
        Viewport::new(viewport, point::logical(state.scroll_x(), state.scroll_y()))
            .visibility_of_local_caret(caret, TEXT_FIELD_CARET_MARGIN),
        Visibility::Below
    ));

    let revealed = engine.ensure_caret_visible_for_area(
        &area_model,
        style,
        viewport,
        state,
        Some(observed.layout()),
    );
    let expected = caret.y() + caret.height() + TEXT_FIELD_CARET_MARGIN - viewport.height();

    assert!(
        (revealed.scroll_y() - expected).abs() <= TEXT_LAYOUT_VISUAL_LINE_EPSILON,
        "observed hidden caret should reveal by the minimal local delta: expected {expected}, got {}",
        revealed.scroll_y()
    );
    let layout =
        engine.text_area_paint_layout_for_area_at(&area_model, style, viewport, revealed, now);
    let caret = layout
        .layout()
        .caret()
        .expect("observed hidden caret should be revealed into the painted viewport");
    assert!(caret.y() >= -TEXT_FIELD_CARET_MARGIN);
    assert!(caret.y() + caret.height() <= viewport.height() + TEXT_FIELD_CARET_MARGIN);
}
#[test]
fn text_area_ensure_caret_visible_scrolls_hidden_caret_into_view() {
    let mut engine = engine();
    let text = (0..40)
        .map(|line| format!("line {line:02}"))
        .collect::<Vec<_>>()
        .join("\n");
    let buffer = Buffer::from_multiline_text(text.clone());
    let cursor_index = text.find("line 30").unwrap() + "line 30".len();
    let mut edit_state = buffer.initial_state();
    apply_selection(
        &buffer,
        &mut edit_state,
        selection::Operation::set_position(Position::new(cursor_index)),
    );

    let area_model = Area::new(buffer).with_state(edit_state);
    let style = Style::default().with_size(16.0);
    let viewport = area::logical(200.0, 64.0);
    let state = ViewState::default().ensure_caret_visible(Instant::now());

    let revealed = engine.ensure_caret_visible_for_area(&area_model, style, viewport, state, None);

    assert!(revealed.scroll_y() > 0.0);
    let layout = engine.text_area_paint_layout_for_area_at(
        &area_model,
        style,
        viewport,
        revealed,
        Instant::now(),
    );
    let caret = layout
        .layout()
        .caret()
        .expect("hidden caret should be revealed into the painted viewport");
    assert!(caret.y() >= -TEXT_FIELD_CARET_MARGIN);
    assert!(caret.y() + caret.height() <= viewport.height() + TEXT_FIELD_CARET_MARGIN);
}
#[test]
fn large_wrapped_text_area_ensure_caret_visible_preserves_scroll_after_selection_delete() {
    let mut engine = engine();
    let mut editor = Editor::new();
    let wrapped = "alpha beta gamma delta epsilon zeta eta theta iota kappa lambda mu nu xi omicron pi rho sigma tau";
    let text = (0..1_200)
        .map(|line| format!("line {line:04} {wrapped}"))
        .collect::<Vec<_>>()
        .join("\n");
    let target_line = 560;
    let selection_start = text.find(&format!("line {target_line:04}")).unwrap();
    let selection_end =
        text.find(&format!("line {:04}", target_line + 3)).unwrap() + "line 0000 alpha beta".len();
    let mut buffer = Buffer::from_multiline_text(text);
    let mut edit_state = buffer.initial_state();
    apply_selection(
        &buffer,
        &mut edit_state,
        selection::Operation::set_position(Position::new(selection_start)),
    );

    let style = Style::default().with_size(16.0);
    let viewport = area::logical(126.0, 80.0);
    let now = Instant::now();
    let mut state = ViewState::default()
        .with_scroll_y(text_area_estimated_line_height(style) * target_line as f32)
        .ensure_caret_visible(now);
    let area_model = Area::new(buffer.clone()).with_state(edit_state);
    for _ in 0..8 {
        let observed = engine.text_area_paint_layout_for_area_at(
            &area_model,
            style,
            viewport,
            state.clone(),
            now,
        );
        let next = engine.ensure_caret_visible_for_area(
            &area_model,
            style,
            viewport,
            state.clone(),
            Some(observed.layout()),
        );
        let layout = engine.text_area_paint_layout_for_area_at(
            &area_model,
            style,
            viewport,
            next.clone(),
            now,
        );
        if let Some(caret) = layout.layout().caret()
            && Viewport::new(viewport, point::logical(next.scroll_x(), next.scroll_y()))
                .visibility_of_local_caret(caret, TEXT_FIELD_CARET_MARGIN)
                .is_visible()
        {
            state = next.ensure_caret_visible(now);
            break;
        }
        state = next.ensure_caret_visible(now);
    }
    let scroll_y = state.scroll_y();

    apply_selection(
        &buffer,
        &mut edit_state,
        selection::Operation::pointer(PointerKind::Drag, Position::new(selection_end)),
    );
    assert!(buffer.has_selection_for_state(edit_state));
    let result = editor.apply_edit(&mut buffer, &mut edit_state, Edit::backspace());
    assert!(result.text_changed);
    engine.invalidate_text_area_surfaces_for(&buffer);
    assert!(!buffer.has_selection_for_state(edit_state));

    let area_model = Area::new(buffer).with_state(edit_state);
    let state = state.ensure_caret_visible(now);
    let observed =
        engine.text_area_paint_layout_for_area_at(&area_model, style, viewport, state.clone(), now);
    let ensured = engine.ensure_caret_visible_for_area(
        &area_model,
        style,
        viewport,
        state,
        Some(observed.layout()),
    );

    assert!(
        (ensured.scroll_y() - scroll_y).abs() <= TEXT_LAYOUT_VISUAL_LINE_EPSILON,
        "visible collapsed caret should preserve scroll after block delete: before {scroll_y}, after {}",
        ensured.scroll_y()
    );
    let layout =
        engine.text_area_paint_layout_for_area_at(&area_model, style, viewport, ensured, now);
    let caret = layout
        .layout()
        .caret()
        .expect("collapsed caret should remain visible after deleting selection");
    assert!(
        Viewport::new(viewport, point::logical(0.0, scroll_y))
            .visibility_of_local_caret(caret, TEXT_FIELD_CARET_MARGIN)
            .is_visible(),
        "caret should remain visible after preserving scroll: {caret:?}"
    );
}

#[test]
fn text_area_edit_commands_preserve_scroll_when_resulting_caret_is_visible() {
    fn assert_command_preserves_scroll(
        engine: &mut Engine,
        area_model: &Area,
        style: Style,
        viewport: area::Logical,
        state: ViewState,
        scroll_y: f32,
        now: Instant,
    ) -> ViewState {
        let state = state.ensure_caret_visible(now);
        let observed = engine.text_area_paint_layout_for_area_at(
            area_model,
            style,
            viewport,
            state.clone(),
            now,
        );
        let ensured = engine.ensure_caret_visible_for_area(
            area_model,
            style,
            viewport,
            state,
            Some(observed.layout()),
        );
        assert!(
            (ensured.scroll_y() - scroll_y).abs() <= TEXT_LAYOUT_VISUAL_LINE_EPSILON,
            "command ensure should preserve visible caret scroll: before {scroll_y}, after {}",
            ensured.scroll_y()
        );
        ensured
    }

    let style = Style::default().with_size(16.0);
    let viewport = area::logical(240.0, 72.0);
    let now = Instant::now();
    let line_height = text_area_estimated_line_height(style);
    let scroll_y = line_height * 28.0;
    let text = (0..80)
        .map(|line| format!("line {line:02} command target"))
        .collect::<Vec<_>>()
        .join("\n");
    let target_start = text.find("line 30").unwrap();
    let target_end = target_start + "line 30".len();

    let mut engine = engine();
    let mut editor = Editor::new();
    let mut buffer = Buffer::from_multiline_text(text.clone());
    let mut edit_state = buffer.initial_state();
    let state = ViewState::default().with_scroll_y(scroll_y);
    apply_selection(
        &buffer,
        &mut edit_state,
        selection::Operation::set_position(Position::new(target_start)),
    );
    apply_selection(
        &buffer,
        &mut edit_state,
        selection::Operation::pointer(PointerKind::Drag, Position::new(target_end)),
    );
    let cut = editor.apply_edit(&mut buffer, &mut edit_state, Edit::insert(""));
    assert!(cut.text_changed);
    let _ = assert_command_preserves_scroll(
        &mut engine,
        &Area::new(buffer.clone()).with_state(edit_state),
        style,
        viewport,
        state,
        scroll_y,
        now,
    );

    let mut buffer = Buffer::from_multiline_text(text.clone());
    let mut edit_state = buffer.initial_state();
    let state = ViewState::default().with_scroll_y(scroll_y);
    apply_selection(
        &buffer,
        &mut edit_state,
        selection::Operation::set_position(Position::new(target_start)),
    );
    apply_selection(
        &buffer,
        &mut edit_state,
        selection::Operation::pointer(PointerKind::Drag, Position::new(target_end)),
    );
    let paste = editor.apply_edit(&mut buffer, &mut edit_state, Edit::insert("XX"));
    assert!(paste.text_changed);
    let _ = assert_command_preserves_scroll(
        &mut engine,
        &Area::new(buffer.clone()).with_state(edit_state),
        style,
        viewport,
        state,
        scroll_y,
        now,
    );

    let mut buffer = Buffer::from_multiline_text(text);
    let mut edit_state = buffer.initial_state();
    let mut state = ViewState::default().with_scroll_y(scroll_y);
    apply_selection(
        &buffer,
        &mut edit_state,
        selection::Operation::set_position(Position::new(target_end)),
    );
    let edit = editor.apply_edit(&mut buffer, &mut edit_state, Edit::insert("!"));
    assert!(edit.text_changed);
    let change = edit.change.expect("insert should produce transaction");
    assert!(buffer.apply_transaction_for_state(&mut edit_state, &change.transaction.inverse()));
    buffer.restore_marker_for_state(&mut edit_state, change.before.clone());
    state = assert_command_preserves_scroll(
        &mut engine,
        &Area::new(buffer.clone()).with_state(edit_state),
        style,
        viewport,
        state,
        scroll_y,
        now,
    );
    assert!(buffer.apply_transaction_for_state(&mut edit_state, &change.transaction));
    buffer.restore_marker_for_state(&mut edit_state, change.after);
    let _ = assert_command_preserves_scroll(
        &mut engine,
        &Area::new(buffer).with_state(edit_state),
        style,
        viewport,
        state,
        scroll_y,
        now,
    );
}
fn styled_document(text: &str, align: Align, size: f32, weight: Weight) -> Document {
    let mut block = Block::new(align);
    block.push_run(Run::new(
        text,
        Style::default().with_size(size).with_weight(weight),
    ));

    Document::from_block(block)
}
