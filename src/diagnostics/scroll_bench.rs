use std::hint::black_box;
use std::time::{Duration, Instant};

use crate::geometry::area;
use crate::text::{
    Buffer, Edit,
    buffer::Position,
    document::Style,
    edit::Editor,
    layout::{Diagnostics as TextDiagnostics, Engine},
    selection,
    surface::Area,
    view::ViewState,
};

pub const SCROLL_BENCH_VERSION: u32 = 10;
pub const OFFICIAL_PROPERTY_WARMUP: usize = 64;
pub const OFFICIAL_PROPERTY_SAMPLES: usize = 1_024;

const LONG_LINE_BYTES: usize = 1_048_576;
const EXACT_LONG_LINE_BYTES: usize = 4 * 1_048_576;
const ASCII_64_MIB_LONG_LINE_BYTES: usize = 64 * 1_048_576;
const VERTICAL_DOCUMENT_BYTES: usize = 8 * 1024 * 1024;
const VIEWPORT_WIDTH: f32 = 920.0;
const VIEWPORT_HEIGHT: f32 = 640.0;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ScrollBenchWorkload {
    TextHorizontalLongLine,
    TextHorizontalExactLongLine,
    TextHorizontalAscii64MiB,
    TextHorizontalEdit4MiB,
    TextHorizontalEdit64MiB,
    TextHorizontalTypingScroll4MiB,
    TextHorizontalCaretReveal4MiB,
    TextVerticalVariableHeight,
}

impl ScrollBenchWorkload {
    pub const ALL: [Self; 8] = [
        Self::TextHorizontalLongLine,
        Self::TextHorizontalExactLongLine,
        Self::TextHorizontalAscii64MiB,
        Self::TextHorizontalEdit4MiB,
        Self::TextHorizontalEdit64MiB,
        Self::TextHorizontalTypingScroll4MiB,
        Self::TextHorizontalCaretReveal4MiB,
        Self::TextVerticalVariableHeight,
    ];

    pub fn name(self) -> &'static str {
        match self {
            Self::TextHorizontalLongLine => "text-horizontal-1m",
            Self::TextHorizontalExactLongLine => "text-horizontal-4m-exact",
            Self::TextHorizontalAscii64MiB => "text-horizontal-64m-ascii",
            Self::TextHorizontalEdit4MiB => "text-horizontal-edit-4m",
            Self::TextHorizontalEdit64MiB => "text-horizontal-edit-64m",
            Self::TextHorizontalTypingScroll4MiB => "text-horizontal-typing-scroll-4m",
            Self::TextHorizontalCaretReveal4MiB => "text-horizontal-caret-reveal-4m",
            Self::TextVerticalVariableHeight => "text-vertical-8m",
        }
    }

    pub fn transition_class(self) -> &'static str {
        match self {
            Self::TextHorizontalLongLine => "ResidencyCrossing",
            Self::TextHorizontalExactLongLine => "ResidencyCrossing",
            Self::TextHorizontalAscii64MiB => "ResidencyCrossing",
            Self::TextHorizontalEdit4MiB
            | Self::TextHorizontalEdit64MiB
            | Self::TextHorizontalTypingScroll4MiB
            | Self::TextHorizontalCaretReveal4MiB => "SemanticChange",
            Self::TextVerticalVariableHeight => "ResidencyCrossing",
        }
    }

    fn sample_work_class(self) -> &'static str {
        match self {
            Self::TextHorizontalEdit4MiB | Self::TextHorizontalEdit64MiB => "EditLocality",
            Self::TextHorizontalTypingScroll4MiB => "EditLocality+ResidentHorizontalProjection",
            Self::TextHorizontalCaretReveal4MiB => {
                "EditLocality+CaretReveal+ResidentHorizontalProjection"
            }
            Self::TextHorizontalLongLine
            | Self::TextHorizontalExactLongLine
            | Self::TextHorizontalAscii64MiB
            | Self::TextVerticalVariableHeight => "ResidentWindowProjection",
        }
    }

    fn cold_work_class(self) -> &'static str {
        match self {
            Self::TextVerticalVariableHeight => "ColdLayoutStart",
            _ => "GlobalExtentColdStart",
        }
    }

    pub fn from_name(name: &str) -> Option<Self> {
        Self::ALL
            .into_iter()
            .find(|workload| workload.name() == name)
    }

    fn requires_precision_offsets(self) -> bool {
        matches!(
            self,
            Self::TextHorizontalExactLongLine
                | Self::TextHorizontalAscii64MiB
                | Self::TextHorizontalEdit4MiB
                | Self::TextHorizontalEdit64MiB
                | Self::TextHorizontalTypingScroll4MiB
                | Self::TextHorizontalCaretReveal4MiB
        )
    }
}

#[derive(Debug, Clone, Default)]
struct ScrollBenchPhases {
    edits: Vec<Duration>,
    projections: Vec<Duration>,
    reveals: Vec<Duration>,
    horizontal_projection_changes: usize,
    reveal_scroll_changes: usize,
    reveal_scroll_distance_max: usize,
}

impl ScrollBenchPhases {
    fn sort(&mut self) {
        self.edits.sort_unstable();
        self.projections.sort_unstable();
        self.reveals.sort_unstable();
    }
}

#[derive(Debug, Clone)]
pub struct ScrollBenchReceipt {
    workload: ScrollBenchWorkload,
    warmup: usize,
    samples: Vec<Duration>,
    cold: Duration,
    cold_diagnostics: TextDiagnostics,
    diagnostics: TextDiagnostics,
    document_bytes: usize,
    document_lines: usize,
    logical_width: usize,
    logical_height: usize,
    offset_x: usize,
    offset_y: usize,
    near_window_width: usize,
    near_window_height: usize,
    far_window_width: usize,
    far_window_height: usize,
    precision_offsets_required: bool,
    precision_offsets_checked: bool,
    phases: ScrollBenchPhases,
}

impl ScrollBenchReceipt {
    pub fn receipt_text(&self, commit: &str) -> String {
        format!(
            concat!(
                "scroll-bench-version={} workload={} transition_class={} cold_transition_class=ColdStart sample_work_class={} cold_work_class={} property_scroll_measured=false commit={} ",
                "profile={} timer=std-instant os={} architecture={} ",
                "viewport={}x{} document_bytes={} document_lines={} offset_x={} offset_y={} logical_width={} logical_height={} precision_offsets_required={} precision_offsets_checked={} ",
                "warmup={} samples={} official_matrix={} cold_us={} p50_us={} p95_us={} p99_us={} max_us={} edit_samples={} edit_p50_us={} edit_p95_us={} edit_p99_us={} edit_max_us={} projection_samples={} projection_p50_us={} projection_p95_us={} projection_p99_us={} projection_max_us={} reveal_samples={} reveal_p50_us={} reveal_p95_us={} reveal_p99_us={} reveal_max_us={} horizontal_projection_changes={} reveal_scroll_changes={} reveal_scroll_distance_max={} ",
                "near_window_width={} near_window_height={} far_window_width={} far_window_height={} render_window_width_max={} render_window_height_max={} render_window_area_max={} bounded_window={} ",
                "paint_layout_calls={} visible_lines={} shaped_lines={} line_shape_calls={} render_surface_calls={} render_cache_hits={} render_cache_misses={} render_line_reuses={} ",
                "horizontal_index_builds={} horizontal_index_hits={} horizontal_index_misses={} horizontal_index_evictions={} horizontal_index_incremental_updates={} horizontal_index_incremental_source_bytes={} horizontal_index_incremental_source_bytes_max={} horizontal_index_incremental_glyphs={} horizontal_index_incremental_glyphs_max={} horizontal_index_source_bytes={} horizontal_index_glyphs={} horizontal_index_checkpoints={} horizontal_exact_band_shapes={} horizontal_exact_band_source_bytes={} horizontal_index_resident_bytes_max={} horizontal_window_shapes={} horizontal_window_source_bytes={} horizontal_resident_source_bytes_max={} horizontal_resident_glyphs_max={} horizontal_resident_bytes_max={} line_cache_resident_bytes_max={} ",
                "render_source_lines={} render_source_bytes={} render_total_us={} render_shape_us={} ",
                "height_index_hits={} height_index_misses={} height_index_queries={} height_index_updates={} height_index_refined_pixels={} anchor_candidates={} anchor_corrections={} anchor_correction_pixels={} anchor_correction_pixels_max={} ",
                "width_cache_hits={} width_cache_misses={} width_observed_updates={} width_source_lines={} width_source_bytes={} width_measure_us={} caret_run_scans={} caret_glyph_scans={} highlight_run_scans={} ",
                "cold_horizontal_index_builds={} cold_horizontal_index_hits={} cold_horizontal_index_misses={} cold_horizontal_index_evictions={} cold_horizontal_index_incremental_updates={} cold_horizontal_index_incremental_source_bytes={} cold_horizontal_index_incremental_glyphs={} cold_horizontal_index_source_bytes={} cold_horizontal_index_glyphs={} cold_horizontal_index_checkpoints={} cold_horizontal_exact_band_shapes={} cold_horizontal_exact_band_source_bytes={} cold_horizontal_index_resident_bytes_max={} cold_horizontal_window_shapes={} cold_horizontal_window_source_bytes={} cold_horizontal_resident_source_bytes_max={} cold_horizontal_resident_glyphs_max={} cold_horizontal_resident_bytes_max={} cold_line_cache_resident_bytes_max={} ",
                "cold_render_line_reuses={} cold_render_source_bytes={} cold_render_shape_us={} cold_height_index_queries={} cold_height_index_updates={} cold_height_index_refined_pixels={} cold_anchor_candidates={} cold_anchor_corrections={} cold_anchor_correction_pixels={} cold_width_observed_updates={} cold_width_source_bytes={} cold_width_measure_us={}"
            ),
            SCROLL_BENCH_VERSION,
            self.workload.name(),
            self.workload.transition_class(),
            self.workload.sample_work_class(),
            self.workload.cold_work_class(),
            commit,
            if cfg!(debug_assertions) {
                "debug"
            } else {
                "release"
            },
            std::env::consts::OS,
            std::env::consts::ARCH,
            VIEWPORT_WIDTH as usize,
            VIEWPORT_HEIGHT as usize,
            self.document_bytes,
            self.document_lines,
            self.offset_x,
            self.offset_y,
            self.logical_width,
            self.logical_height,
            self.precision_offsets_required,
            self.precision_offsets_checked,
            self.warmup,
            self.samples.len(),
            self.warmup >= OFFICIAL_PROPERTY_WARMUP
                && self.samples.len() >= OFFICIAL_PROPERTY_SAMPLES,
            self.cold.as_micros(),
            percentile(&self.samples, 50).as_micros(),
            percentile(&self.samples, 95).as_micros(),
            percentile(&self.samples, 99).as_micros(),
            self.samples.last().copied().unwrap_or_default().as_micros(),
            self.phases.edits.len(),
            percentile(&self.phases.edits, 50).as_micros(),
            percentile(&self.phases.edits, 95).as_micros(),
            percentile(&self.phases.edits, 99).as_micros(),
            self.phases
                .edits
                .last()
                .copied()
                .unwrap_or_default()
                .as_micros(),
            self.phases.projections.len(),
            percentile(&self.phases.projections, 50).as_micros(),
            percentile(&self.phases.projections, 95).as_micros(),
            percentile(&self.phases.projections, 99).as_micros(),
            self.phases
                .projections
                .last()
                .copied()
                .unwrap_or_default()
                .as_micros(),
            self.phases.reveals.len(),
            percentile(&self.phases.reveals, 50).as_micros(),
            percentile(&self.phases.reveals, 95).as_micros(),
            percentile(&self.phases.reveals, 99).as_micros(),
            self.phases
                .reveals
                .last()
                .copied()
                .unwrap_or_default()
                .as_micros(),
            self.phases.horizontal_projection_changes,
            self.phases.reveal_scroll_changes,
            self.phases.reveal_scroll_distance_max,
            self.near_window_width,
            self.near_window_height,
            self.far_window_width,
            self.far_window_height,
            self.diagnostics.text_area_render_window_width_max,
            self.diagnostics.text_area_render_window_height_max,
            self.diagnostics.text_area_render_window_area_max,
            self.far_window_width <= horizontal_window_limit()
                && self.far_window_height <= vertical_window_limit(),
            self.diagnostics.text_area_paint_layout_calls,
            self.diagnostics.text_area_visible_logical_lines,
            self.diagnostics.text_area_shaped_logical_lines,
            self.diagnostics.text_area_line_shape_calls,
            self.diagnostics.text_area_render_surface_calls,
            self.diagnostics.text_area_render_surface_cache_hits,
            self.diagnostics.text_area_render_surface_cache_misses,
            self.diagnostics.text_area_render_surface_line_reuses,
            self.diagnostics.text_area_horizontal_index_builds,
            self.diagnostics.text_area_horizontal_index_hits,
            self.diagnostics.text_area_horizontal_index_misses,
            self.diagnostics.text_area_horizontal_index_evictions,
            self.diagnostics
                .text_area_horizontal_index_incremental_updates,
            self.diagnostics
                .text_area_horizontal_index_incremental_source_bytes,
            self.diagnostics
                .text_area_horizontal_index_incremental_source_bytes_max,
            self.diagnostics
                .text_area_horizontal_index_incremental_glyphs,
            self.diagnostics
                .text_area_horizontal_index_incremental_glyphs_max,
            self.diagnostics.text_area_horizontal_index_source_bytes,
            self.diagnostics.text_area_horizontal_index_glyphs,
            self.diagnostics.text_area_horizontal_index_checkpoints,
            self.diagnostics.text_area_horizontal_exact_band_shapes,
            self.diagnostics
                .text_area_horizontal_exact_band_source_bytes,
            self.diagnostics
                .text_area_horizontal_index_resident_bytes_max,
            self.diagnostics.text_area_horizontal_window_shapes,
            self.diagnostics.text_area_horizontal_window_source_bytes,
            self.diagnostics
                .text_area_horizontal_resident_source_bytes_max,
            self.diagnostics.text_area_horizontal_resident_glyphs_max,
            self.diagnostics.text_area_horizontal_resident_bytes_max,
            self.diagnostics.text_area_line_cache_resident_bytes_max,
            self.diagnostics.text_area_render_surface_source_lines,
            self.diagnostics.text_area_render_surface_source_bytes,
            self.diagnostics.text_area_render_surface_total_us,
            self.diagnostics.text_area_render_surface_shape_us,
            self.diagnostics.text_area_height_index_hits,
            self.diagnostics.text_area_height_index_misses,
            self.diagnostics.text_area_height_index_queries,
            self.diagnostics.text_area_height_index_updates,
            self.diagnostics.text_area_height_index_refined_pixels,
            self.diagnostics.text_area_anchor_candidates,
            self.diagnostics.text_area_anchor_corrections,
            self.diagnostics.text_area_anchor_correction_pixels,
            self.diagnostics.text_area_anchor_correction_pixels_max,
            self.diagnostics.text_area_width_cache_hits,
            self.diagnostics.text_area_width_cache_misses,
            self.diagnostics.text_area_width_observed_updates,
            self.diagnostics.text_area_width_source_lines,
            self.diagnostics.text_area_width_source_bytes,
            self.diagnostics.text_area_width_measure_us,
            self.diagnostics.text_area_caret_run_scans,
            self.diagnostics.text_area_caret_glyph_scans,
            self.diagnostics.highlight_run_scans,
            self.cold_diagnostics.text_area_horizontal_index_builds,
            self.cold_diagnostics.text_area_horizontal_index_hits,
            self.cold_diagnostics.text_area_horizontal_index_misses,
            self.cold_diagnostics.text_area_horizontal_index_evictions,
            self.cold_diagnostics
                .text_area_horizontal_index_incremental_updates,
            self.cold_diagnostics
                .text_area_horizontal_index_incremental_source_bytes,
            self.cold_diagnostics
                .text_area_horizontal_index_incremental_glyphs,
            self.cold_diagnostics
                .text_area_horizontal_index_source_bytes,
            self.cold_diagnostics.text_area_horizontal_index_glyphs,
            self.cold_diagnostics.text_area_horizontal_index_checkpoints,
            self.cold_diagnostics.text_area_horizontal_exact_band_shapes,
            self.cold_diagnostics
                .text_area_horizontal_exact_band_source_bytes,
            self.cold_diagnostics
                .text_area_horizontal_index_resident_bytes_max,
            self.cold_diagnostics.text_area_horizontal_window_shapes,
            self.cold_diagnostics
                .text_area_horizontal_window_source_bytes,
            self.cold_diagnostics
                .text_area_horizontal_resident_source_bytes_max,
            self.cold_diagnostics
                .text_area_horizontal_resident_glyphs_max,
            self.cold_diagnostics
                .text_area_horizontal_resident_bytes_max,
            self.cold_diagnostics
                .text_area_line_cache_resident_bytes_max,
            self.cold_diagnostics.text_area_render_surface_line_reuses,
            self.cold_diagnostics.text_area_render_surface_source_bytes,
            self.cold_diagnostics.text_area_render_surface_shape_us,
            self.cold_diagnostics.text_area_height_index_queries,
            self.cold_diagnostics.text_area_height_index_updates,
            self.cold_diagnostics.text_area_height_index_refined_pixels,
            self.cold_diagnostics.text_area_anchor_candidates,
            self.cold_diagnostics.text_area_anchor_corrections,
            self.cold_diagnostics.text_area_anchor_correction_pixels,
            self.cold_diagnostics.text_area_width_observed_updates,
            self.cold_diagnostics.text_area_width_source_bytes,
            self.cold_diagnostics.text_area_width_measure_us,
        )
    }

    pub fn diagnostics(&self) -> TextDiagnostics {
        self.diagnostics
    }

    pub fn far_window_width(&self) -> usize {
        self.far_window_width
    }
}

pub fn run_scroll_bench(
    workload: ScrollBenchWorkload,
    warmup: usize,
    samples: usize,
) -> Result<ScrollBenchReceipt, String> {
    if samples == 0 {
        return Err("scroll-bench samples must be positive".to_owned());
    }
    match workload {
        ScrollBenchWorkload::TextHorizontalLongLine => run_text_horizontal_long_line(
            ScrollBenchWorkload::TextHorizontalLongLine,
            LONG_LINE_BYTES,
            warmup,
            samples,
        ),
        ScrollBenchWorkload::TextHorizontalExactLongLine => run_text_horizontal_long_line(
            ScrollBenchWorkload::TextHorizontalExactLongLine,
            EXACT_LONG_LINE_BYTES,
            warmup,
            samples,
        ),
        ScrollBenchWorkload::TextHorizontalAscii64MiB => run_text_horizontal_long_line(
            ScrollBenchWorkload::TextHorizontalAscii64MiB,
            ASCII_64_MIB_LONG_LINE_BYTES,
            warmup,
            samples,
        ),
        ScrollBenchWorkload::TextHorizontalEdit4MiB => run_text_horizontal_edit(
            ScrollBenchWorkload::TextHorizontalEdit4MiB,
            EXACT_LONG_LINE_BYTES,
            warmup,
            samples,
            HorizontalEditTrace::FixedFar,
        ),
        ScrollBenchWorkload::TextHorizontalEdit64MiB => run_text_horizontal_edit(
            ScrollBenchWorkload::TextHorizontalEdit64MiB,
            ASCII_64_MIB_LONG_LINE_BYTES,
            warmup,
            samples,
            HorizontalEditTrace::FixedFar,
        ),
        ScrollBenchWorkload::TextHorizontalTypingScroll4MiB => run_text_horizontal_edit(
            ScrollBenchWorkload::TextHorizontalTypingScroll4MiB,
            EXACT_LONG_LINE_BYTES,
            warmup,
            samples,
            HorizontalEditTrace::TypingScroll,
        ),
        ScrollBenchWorkload::TextHorizontalCaretReveal4MiB => run_text_horizontal_edit(
            ScrollBenchWorkload::TextHorizontalCaretReveal4MiB,
            EXACT_LONG_LINE_BYTES,
            warmup,
            samples,
            HorizontalEditTrace::CaretReveal,
        ),
        ScrollBenchWorkload::TextVerticalVariableHeight => {
            run_text_vertical_variable_height(warmup, samples)
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum HorizontalEditTrace {
    FixedFar,
    TypingScroll,
    CaretReveal,
}

fn run_text_horizontal_long_line(
    workload: ScrollBenchWorkload,
    source_bytes: usize,
    warmup: usize,
    samples: usize,
) -> Result<ScrollBenchReceipt, String> {
    let source = long_line_source(source_bytes);
    let document_bytes = source.len();
    let area_model = Area::new(Buffer::from_multiline_text(source)).no_wrap();
    let document_lines = area_model.buffer().logical_line_count();
    let style = Style::default().with_size(13.0);
    let viewport = area::logical(VIEWPORT_WIDTH, VIEWPORT_HEIGHT);
    let mut engine = Engine::new();
    let now = Instant::now();

    engine.reset_diagnostics();
    let cold_started = Instant::now();
    let cold_layout = engine.text_area_paint_layout_for_area_at(
        &area_model,
        style,
        viewport,
        ViewState::default(),
        now,
    );
    let cold = cold_started.elapsed();
    let cold_diagnostics = engine.diagnostics();
    let logical_width = ceil_f64_to_usize(cold_layout.layout().content_width_exact());
    let logical_height = ceil_f64_to_usize(cold_layout.layout().content_height_exact());
    let (near_window_width, near_window_height) = surface_window(cold_layout.render_surfaces())
        .ok_or_else(|| format!("{} produced no cold render surface", workload.name()))?;
    let maximum_offset = logical_width.saturating_sub(VIEWPORT_WIDTH as usize);
    let offset_x = maximum_offset.saturating_mul(3) / 4;
    let precision_offsets_required = workload.requires_precision_offsets();
    let precision_offsets_checked = if precision_offsets_required {
        verify_exact_horizontal_offsets(
            &mut engine,
            &area_model,
            style,
            viewport,
            now,
            maximum_offset,
        )?
    } else {
        false
    };

    for index in 0..warmup {
        let offset = horizontal_streamed_offset(workload, offset_x, index, maximum_offset);
        black_box(engine.text_area_paint_layout_for_area_at(
            &area_model,
            style,
            viewport,
            ViewState::default().with_exact_scroll(offset as f64, 0.0),
            now,
        ));
    }

    engine.reset_diagnostics();
    let mut durations = Vec::with_capacity(samples);
    let mut far_window_width = 0_usize;
    let mut far_window_height = 0_usize;
    let mut previous_offset = None;
    let mut horizontal_projection_changes = 0_usize;
    for index in 0..samples {
        let offset = horizontal_streamed_offset(
            workload,
            offset_x,
            warmup.saturating_add(index),
            maximum_offset,
        );
        if previous_offset.is_some_and(|previous| previous != offset) {
            horizontal_projection_changes += 1;
        }
        previous_offset = Some(offset);
        let started = Instant::now();
        let layout = engine.text_area_paint_layout_for_area_at(
            &area_model,
            style,
            viewport,
            ViewState::default().with_exact_scroll(offset as f64, 0.0),
            now,
        );
        durations.push(started.elapsed());
        if let Some((width, height)) = surface_window(layout.render_surfaces()) {
            far_window_width = far_window_width.max(width);
            far_window_height = far_window_height.max(height);
        }
        black_box(layout);
    }
    durations.sort_unstable();
    let diagnostics = engine.diagnostics();
    let phases = ScrollBenchPhases {
        projections: durations.clone(),
        horizontal_projection_changes,
        ..ScrollBenchPhases::default()
    };

    Ok(ScrollBenchReceipt {
        workload,
        warmup,
        samples: durations,
        cold,
        cold_diagnostics,
        diagnostics,
        document_bytes,
        document_lines,
        logical_width,
        logical_height,
        offset_x,
        offset_y: 0,
        near_window_width,
        near_window_height,
        far_window_width,
        far_window_height,
        precision_offsets_required,
        precision_offsets_checked,
        phases,
    })
}

fn run_text_horizontal_edit(
    workload: ScrollBenchWorkload,
    source_bytes: usize,
    warmup: usize,
    samples: usize,
    trace: HorizontalEditTrace,
) -> Result<ScrollBenchReceipt, String> {
    let source = long_line_source(source_bytes);
    let document_bytes = source.len();
    let edit_index = source[..source.len() / 2]
        .rfind("abcdefgh")
        .map(|index| index + 4)
        .ok_or_else(|| format!("{} has no interior edit point", workload.name()))?;
    let mut buffer = Buffer::from_multiline_text(source);
    let mut state = buffer.initial_state();
    selection::apply(
        &buffer,
        &mut state,
        selection::Operation::set_position(Position::new(edit_index)),
    );
    let style = Style::default().with_size(13.0);
    let viewport = area::logical(VIEWPORT_WIDTH, VIEWPORT_HEIGHT);
    let now = Instant::now();
    let mut engine = Engine::new();
    let mut editor = Editor::new();
    let cold_area = Area::new(buffer.clone()).with_state(state).no_wrap();

    engine.reset_diagnostics();
    let cold_started = Instant::now();
    let cold_layout = engine.text_area_paint_layout_for_area_at(
        &cold_area,
        style,
        viewport,
        ViewState::default(),
        now,
    );
    let cold = cold_started.elapsed();
    let cold_diagnostics = engine.diagnostics();
    let logical_width = ceil_f64_to_usize(cold_layout.layout().content_width_exact());
    let logical_height = ceil_f64_to_usize(cold_layout.layout().content_height_exact());
    let (near_window_width, near_window_height) = surface_window(cold_layout.render_surfaces())
        .ok_or_else(|| "text-horizontal-edit-4m produced no cold render surface".to_owned())?;
    let maximum_offset = logical_width.saturating_sub(VIEWPORT_WIDTH as usize);
    let fixed_far_offset = maximum_offset.saturating_mul(3) / 4;
    let precision_offsets_checked = verify_exact_horizontal_offsets(
        &mut engine,
        &cold_area,
        style,
        viewport,
        now,
        maximum_offset,
    )?;
    let caret_base_offset = if trace == HorizontalEditTrace::FixedFar {
        0.0
    } else {
        engine
            .ensure_caret_visible_for_area(
                &cold_area,
                style,
                viewport,
                ViewState::default().ensure_caret_visible(now),
                None,
            )
            .exact_scroll_x()
            .clamp(0.0, maximum_offset as f64)
    };
    let offset_x = match trace {
        HorizontalEditTrace::FixedFar => fixed_far_offset,
        HorizontalEditTrace::TypingScroll | HorizontalEditTrace::CaretReveal => {
            ceil_f64_to_usize(caret_base_offset)
        }
    };

    for step in 0..warmup {
        let edit = if step % 2 == 0 {
            Edit::insert("Z")
        } else {
            Edit::backspace()
        };
        if !editor
            .apply_edit(&mut buffer, &mut state, edit)
            .text_changed()
        {
            return Err(format!(
                "{} warmup edit {step} did not change text",
                workload.name()
            ));
        }
        let area_model = Area::new(buffer.clone()).with_state(state).no_wrap();
        let projection_offset = horizontal_edit_projection_offset(
            trace,
            fixed_far_offset,
            caret_base_offset,
            step,
            maximum_offset,
        );
        let mut view_state = ViewState::default().with_exact_scroll(projection_offset, 0.0);
        if trace == HorizontalEditTrace::CaretReveal {
            view_state = engine.ensure_caret_visible_for_area(
                &area_model,
                style,
                viewport,
                view_state.ensure_caret_visible(now),
                None,
            );
        }
        black_box(engine.text_area_paint_layout_for_area_at(
            &area_model,
            style,
            viewport,
            view_state,
            now,
        ));
    }

    engine.reset_diagnostics();
    let mut durations = Vec::with_capacity(samples);
    let mut phases = ScrollBenchPhases {
        edits: Vec::with_capacity(samples),
        projections: Vec::with_capacity(samples),
        reveals: if trace == HorizontalEditTrace::CaretReveal {
            Vec::with_capacity(samples)
        } else {
            Vec::new()
        },
        ..ScrollBenchPhases::default()
    };
    let mut far_window_width = 0_usize;
    let mut far_window_height = 0_usize;
    let mut previous_projection_offset = None;
    for index in 0..samples {
        let step = warmup.saturating_add(index);
        let started = Instant::now();
        let edit = if step % 2 == 0 {
            Edit::insert("Z")
        } else {
            Edit::backspace()
        };
        let edit_started = Instant::now();
        if !editor
            .apply_edit(&mut buffer, &mut state, edit)
            .text_changed()
        {
            return Err(format!(
                "{} measured edit {index} did not change text",
                workload.name()
            ));
        }
        phases.edits.push(edit_started.elapsed());
        let area_model = Area::new(buffer.clone()).with_state(state).no_wrap();
        let projection_offset = horizontal_edit_projection_offset(
            trace,
            fixed_far_offset,
            caret_base_offset,
            step,
            maximum_offset,
        );
        if previous_projection_offset.is_some_and(|previous| previous != projection_offset) {
            phases.horizontal_projection_changes += 1;
        }
        previous_projection_offset = Some(projection_offset);
        let mut view_state = ViewState::default().with_exact_scroll(projection_offset, 0.0);
        if trace == HorizontalEditTrace::CaretReveal {
            let reveal_started = Instant::now();
            view_state = engine.ensure_caret_visible_for_area(
                &area_model,
                style,
                viewport,
                view_state.ensure_caret_visible(now),
                None,
            );
            phases.reveals.push(reveal_started.elapsed());
            let reveal_distance = (view_state.exact_scroll_x() - projection_offset).abs();
            if reveal_distance <= f64::EPSILON {
                return Err(format!(
                    "{} measured reveal {index} did not move hidden caret",
                    workload.name()
                ));
            }
            phases.reveal_scroll_changes += 1;
            phases.reveal_scroll_distance_max = phases
                .reveal_scroll_distance_max
                .max(ceil_f64_to_usize(reveal_distance));
        }
        let projection_started = Instant::now();
        let layout = engine.text_area_paint_layout_for_area_at(
            &area_model,
            style,
            viewport,
            view_state,
            now,
        );
        phases.projections.push(projection_started.elapsed());
        durations.push(started.elapsed());
        if let Some((width, height)) = surface_window(layout.render_surfaces()) {
            far_window_width = far_window_width.max(width);
            far_window_height = far_window_height.max(height);
        }
        black_box(layout);
    }
    durations.sort_unstable();
    phases.sort();
    let diagnostics = engine.diagnostics();
    validate_horizontal_edit_trace(
        workload,
        trace,
        samples,
        &phases,
        diagnostics,
        far_window_width,
        far_window_height,
    )?;

    Ok(ScrollBenchReceipt {
        workload,
        warmup,
        samples: durations,
        cold,
        cold_diagnostics,
        diagnostics,
        document_bytes,
        document_lines: buffer.logical_line_count(),
        logical_width,
        logical_height,
        offset_x,
        offset_y: 0,
        near_window_width,
        near_window_height,
        far_window_width,
        far_window_height,
        precision_offsets_required: true,
        precision_offsets_checked,
        phases,
    })
}

fn validate_horizontal_edit_trace(
    workload: ScrollBenchWorkload,
    trace: HorizontalEditTrace,
    samples: usize,
    phases: &ScrollBenchPhases,
    diagnostics: TextDiagnostics,
    far_window_width: usize,
    far_window_height: usize,
) -> Result<(), String> {
    const MAX_GUARDED_EDIT_SOURCE_BYTES: usize = 4_096;
    if phases.edits.len() != samples || phases.projections.len() != samples {
        return Err(format!(
            "{} did not account for every measured edit/projection phase",
            workload.name()
        ));
    }
    if diagnostics.text_area_horizontal_index_incremental_updates != samples
        || diagnostics.text_area_horizontal_index_builds != 0
        || diagnostics.text_area_horizontal_index_source_bytes != 0
        || diagnostics.text_area_width_source_bytes != 0
        || diagnostics.text_area_horizontal_index_incremental_source_bytes_max
            > MAX_GUARDED_EDIT_SOURCE_BYTES
        || diagnostics.text_area_horizontal_index_incremental_source_bytes
            > samples.saturating_mul(MAX_GUARDED_EDIT_SOURCE_BYTES)
    {
        return Err(format!(
            "{} escaped guarded edit locality: updates={} builds={} full_source_bytes={} width_source_bytes={} edit_source_bytes={} edit_source_bytes_max={}",
            workload.name(),
            diagnostics.text_area_horizontal_index_incremental_updates,
            diagnostics.text_area_horizontal_index_builds,
            diagnostics.text_area_horizontal_index_source_bytes,
            diagnostics.text_area_width_source_bytes,
            diagnostics.text_area_horizontal_index_incremental_source_bytes,
            diagnostics.text_area_horizontal_index_incremental_source_bytes_max,
        ));
    }
    if far_window_width > horizontal_window_limit() || far_window_height > vertical_window_limit() {
        return Err(format!(
            "{} escaped its resident projection window: {}x{}",
            workload.name(),
            far_window_width,
            far_window_height
        ));
    }
    match trace {
        HorizontalEditTrace::FixedFar => {
            if !phases.reveals.is_empty() || phases.reveal_scroll_changes != 0 {
                return Err(format!(
                    "{} unexpectedly mixed caret reveal into its edit-only currency",
                    workload.name()
                ));
            }
        }
        HorizontalEditTrace::TypingScroll => {
            if samples > 1 && phases.horizontal_projection_changes < samples / 2 {
                return Err(format!(
                    "{} changed horizontal projection only {} times across {samples} samples",
                    workload.name(),
                    phases.horizontal_projection_changes
                ));
            }
        }
        HorizontalEditTrace::CaretReveal => {
            if phases.reveals.len() != samples || phases.reveal_scroll_changes != samples {
                return Err(format!(
                    "{} revealed {}/{} hidden carets across {samples} samples",
                    workload.name(),
                    phases.reveals.len(),
                    phases.reveal_scroll_changes
                ));
            }
        }
    }
    Ok(())
}

fn horizontal_edit_projection_offset(
    trace: HorizontalEditTrace,
    fixed_far_offset: usize,
    caret_base_offset: f64,
    index: usize,
    maximum_offset: usize,
) -> f64 {
    const HIDDEN_CARET_GAP: f64 = 256.0;
    match trace {
        HorizontalEditTrace::FixedFar => fixed_far_offset as f64,
        HorizontalEditTrace::TypingScroll => {
            let phase = index % 257;
            let displacement = if phase <= 128 { phase } else { 256 - phase };
            (caret_base_offset + displacement as f64 * 4.0).min(maximum_offset as f64)
        }
        HorizontalEditTrace::CaretReveal => (caret_base_offset - HIDDEN_CARET_GAP).max(0.0),
    }
}

fn run_text_vertical_variable_height(
    warmup: usize,
    samples: usize,
) -> Result<ScrollBenchReceipt, String> {
    let source = vertical_document_source();
    let document_bytes = source.len();
    let area_model = Area::new(Buffer::from_multiline_text(source)).read_only();
    let document_lines = area_model.buffer().logical_line_count();
    let style = Style::default().with_size(13.0);
    let viewport = area::logical(VIEWPORT_WIDTH, VIEWPORT_HEIGHT);
    let mut engine = Engine::new();
    let now = Instant::now();

    engine.reset_diagnostics();
    let cold_started = Instant::now();
    let cold_layout = engine.text_area_paint_layout_for_area_at(
        &area_model,
        style,
        viewport,
        ViewState::default(),
        now,
    );
    let cold = cold_started.elapsed();
    let cold_diagnostics = engine.diagnostics();
    let logical_width = ceil_to_usize(cold_layout.layout().content_area().width());
    let logical_height = ceil_to_usize(cold_layout.layout().content_area().height());
    let (near_window_width, near_window_height) = surface_window(cold_layout.render_surfaces())
        .ok_or_else(|| "text-vertical-8m produced no cold render surface".to_owned())?;
    let maximum_offset = logical_height.saturating_sub(VIEWPORT_HEIGHT as usize);
    let offset_y = maximum_offset.saturating_mul(3) / 4;

    for index in 0..warmup {
        let offset = streamed_offset(offset_y, index, maximum_offset);
        black_box(engine.text_area_paint_layout_for_area_at(
            &area_model,
            style,
            viewport,
            ViewState::default().with_scroll_y(offset as f32),
            now,
        ));
    }

    engine.reset_diagnostics();
    let mut durations = Vec::with_capacity(samples);
    let mut far_window_width = 0_usize;
    let mut far_window_height = 0_usize;
    for index in 0..samples {
        let offset = streamed_offset(offset_y, warmup.saturating_add(index), maximum_offset);
        let started = Instant::now();
        let layout = engine.text_area_paint_layout_for_area_at(
            &area_model,
            style,
            viewport,
            ViewState::default().with_scroll_y(offset as f32),
            now,
        );
        durations.push(started.elapsed());
        if let Some((width, height)) = surface_window(layout.render_surfaces()) {
            far_window_width = far_window_width.max(width);
            far_window_height = far_window_height.max(height);
        }
        black_box(layout);
    }
    durations.sort_unstable();
    let diagnostics = engine.diagnostics();
    let phases = ScrollBenchPhases {
        projections: durations.clone(),
        ..ScrollBenchPhases::default()
    };

    Ok(ScrollBenchReceipt {
        workload: ScrollBenchWorkload::TextVerticalVariableHeight,
        warmup,
        samples: durations,
        cold,
        cold_diagnostics,
        diagnostics,
        document_bytes,
        document_lines,
        logical_width,
        logical_height,
        offset_x: 0,
        offset_y,
        near_window_width,
        near_window_height,
        far_window_width,
        far_window_height,
        precision_offsets_required: false,
        precision_offsets_checked: false,
        phases,
    })
}

fn long_line_source(bytes: usize) -> String {
    const PATTERN: &str = "W0123456789 abcdefghijklmnopqrstuvwxyz ";
    let mut source = String::with_capacity(bytes);
    while source.len() + PATTERN.len() <= bytes {
        source.push_str(PATTERN);
    }
    source.push_str(&PATTERN[..bytes - source.len()]);
    source
}

fn vertical_document_source() -> String {
    const SHORT: &str = "short variable-height line";
    const MEDIUM: &str = "medium wrapped line alpha beta gamma delta epsilon zeta eta theta iota kappa lambda mu nu xi omicron pi rho sigma tau";
    const LONG: &str = "long wrapped line alpha beta gamma delta epsilon zeta eta theta iota kappa lambda mu nu xi omicron pi rho sigma tau upsilon phi chi psi omega alpha beta gamma delta epsilon zeta eta theta iota kappa lambda mu nu xi omicron pi rho sigma tau upsilon phi chi psi omega";
    let patterns = [SHORT, MEDIUM, LONG, MEDIUM];
    let mut source = String::with_capacity(VERTICAL_DOCUMENT_BYTES);
    let mut line = 0_usize;
    while source.len() < VERTICAL_DOCUMENT_BYTES {
        let pattern = patterns[line % patterns.len()];
        let separator = usize::from(!source.is_empty());
        let remaining = VERTICAL_DOCUMENT_BYTES.saturating_sub(source.len());
        if remaining <= separator {
            source.push_str(&"x".repeat(remaining));
            break;
        }
        if separator == 1 {
            source.push('\n');
        }
        let available = VERTICAL_DOCUMENT_BYTES.saturating_sub(source.len());
        source.push_str(&pattern[..pattern.len().min(available)]);
        line += 1;
    }
    source
}

fn streamed_offset(base: usize, index: usize, maximum: usize) -> usize {
    let phase = index % 257;
    let displacement = if phase <= 128 { phase } else { 256 - phase };
    base.saturating_add(displacement.saturating_mul(4))
        .min(maximum)
}

fn horizontal_streamed_offset(
    workload: ScrollBenchWorkload,
    base: usize,
    index: usize,
    maximum: usize,
) -> usize {
    if workload.requires_precision_offsets() {
        match index % 257 {
            0 => return 16_777_215.min(maximum),
            1 => return 16_777_216.min(maximum),
            2 => return 16_777_217.min(maximum),
            3 => return 24_000_001.min(maximum),
            _ => {}
        }
    }
    streamed_offset(base, index, maximum)
}

fn verify_exact_horizontal_offsets(
    engine: &mut Engine,
    area_model: &Area,
    style: Style,
    viewport: crate::geometry::area::Logical,
    now: Instant,
    maximum_offset: usize,
) -> Result<bool, String> {
    const VALUES: [usize; 4] = [16_777_215, 16_777_216, 16_777_217, 24_000_001];
    if maximum_offset < *VALUES.last().unwrap_or(&0) {
        return Err(format!(
            "exact horizontal workload maximum {maximum_offset} does not cover 24,000,001"
        ));
    }

    let mut adjacent = Vec::with_capacity(3);
    for value in VALUES {
        let layout = engine.text_area_paint_layout_for_area_at(
            area_model,
            style,
            viewport,
            ViewState::default().with_exact_scroll(value as f64, 0.0),
            now,
        );
        let surface = layout
            .render_surfaces()
            .first()
            .ok_or_else(|| format!("exact horizontal offset {value} produced no render surface"))?;
        if value <= 16_777_217 {
            adjacent.push((surface.source_start(), surface.x()));
        }
    }

    let boundary_pair_is_distinct = adjacent[0] != adjacent[1];
    let stable_pair_moves_one_pixel = adjacent[1].0 == adjacent[2].0
        && (adjacent[1].1 - adjacent[2].1 - 1.0).abs() <= f32::EPSILON;
    if !boundary_pair_is_distinct || !stable_pair_moves_one_pixel {
        return Err(format!(
            "exact horizontal offsets collapsed or changed residency: {adjacent:?}"
        ));
    }
    Ok(true)
}

fn horizontal_window_limit() -> usize {
    (VIEWPORT_WIDTH + 2.0 * 256.0).ceil() as usize
}

fn vertical_window_limit() -> usize {
    16_384
}

fn surface_window(surfaces: &[crate::text::layout::TextAreaSurface]) -> Option<(usize, usize)> {
    let first = surfaces.first()?;
    let mut left = first.x();
    let mut top = first.y();
    let mut right = first.x() + first.width();
    let mut bottom = first.y() + first.height();
    for surface in &surfaces[1..] {
        left = left.min(surface.x());
        top = top.min(surface.y());
        right = right.max(surface.x() + surface.width());
        bottom = bottom.max(surface.y() + surface.height());
    }
    Some((
        ceil_to_usize((right - left).max(0.0)),
        ceil_to_usize((bottom - top).max(0.0)),
    ))
}

fn ceil_to_usize(value: f32) -> usize {
    value.ceil().clamp(0.0, usize::MAX as f32) as usize
}

fn ceil_f64_to_usize(value: f64) -> usize {
    value.ceil().clamp(0.0, usize::MAX as f64) as usize
}

fn percentile(samples: &[Duration], percentile: usize) -> Duration {
    if samples.is_empty() {
        return Duration::ZERO;
    }
    let index = (samples.len().saturating_sub(1) * percentile.min(100)) / 100;
    samples[index]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn scroll_bench_workload_names_are_stable_and_round_trip() {
        for workload in ScrollBenchWorkload::ALL {
            assert_eq!(
                ScrollBenchWorkload::from_name(workload.name()),
                Some(workload)
            );
        }
        assert_eq!(ScrollBenchWorkload::from_name("unknown"), None);
    }

    #[test]
    fn scroll_bench_percentiles_use_sorted_nearest_rank_samples() {
        let samples = [
            Duration::from_micros(1),
            Duration::from_micros(2),
            Duration::from_micros(3),
            Duration::from_micros(4),
            Duration::from_micros(5),
        ];
        assert_eq!(percentile(&samples, 50), Duration::from_micros(3));
        assert_eq!(percentile(&samples, 95), Duration::from_micros(4));
        assert_eq!(percentile(&samples, 99), Duration::from_micros(4));
    }

    #[test]
    fn horizontal_sources_and_precision_trace_are_exact() {
        assert_eq!(long_line_source(LONG_LINE_BYTES).len(), LONG_LINE_BYTES);
        assert_eq!(
            long_line_source(EXACT_LONG_LINE_BYTES).len(),
            EXACT_LONG_LINE_BYTES
        );
        let maximum = 30_000_000;
        let offsets = (0..4)
            .map(|index| {
                horizontal_streamed_offset(
                    ScrollBenchWorkload::TextHorizontalExactLongLine,
                    20_000_000,
                    index,
                    maximum,
                )
            })
            .collect::<Vec<_>>();
        assert_eq!(offsets, [16_777_215, 16_777_216, 16_777_217, 24_000_001]);
    }

    #[test]
    fn vertical_scroll_source_is_exact_and_variable_height() {
        let source = vertical_document_source();
        assert_eq!(source.len(), VERTICAL_DOCUMENT_BYTES);
        assert!(source.lines().count() > 10_000);
        let mut lengths = source.lines().take(8).map(str::len).collect::<Vec<_>>();
        lengths.sort_unstable();
        lengths.dedup();
        assert!(lengths.len() >= 3);
    }

    #[test]
    fn horizontal_edit_trace_rejects_source_work_outside_the_guard() {
        let phases = ScrollBenchPhases {
            edits: vec![Duration::from_micros(1)],
            projections: vec![Duration::from_micros(1)],
            ..ScrollBenchPhases::default()
        };
        let diagnostics = TextDiagnostics {
            text_area_horizontal_index_incremental_updates: 1,
            text_area_horizontal_index_incremental_source_bytes: 4_097,
            text_area_horizontal_index_incremental_source_bytes_max: 4_097,
            ..TextDiagnostics::default()
        };

        let error = validate_horizontal_edit_trace(
            ScrollBenchWorkload::TextHorizontalEdit4MiB,
            HorizontalEditTrace::FixedFar,
            1,
            &phases,
            diagnostics,
            horizontal_window_limit(),
            VIEWPORT_HEIGHT as usize,
        )
        .expect_err("source-wide edit work must fail the executable benchmark contract");
        assert!(error.contains("escaped guarded edit locality"));
    }
}
