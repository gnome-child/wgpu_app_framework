use std::hint::black_box;
use std::time::{Duration, Instant};

use crate::geometry::area;
use crate::text::{
    Buffer,
    document::Style,
    layout::{Diagnostics as TextDiagnostics, Engine},
    surface::Area,
    view::ViewState,
};

pub const SCROLL_BENCH_VERSION: u32 = 2;
pub const OFFICIAL_PROPERTY_WARMUP: usize = 64;
pub const OFFICIAL_PROPERTY_SAMPLES: usize = 1_024;

const LONG_LINE_BYTES: usize = 1_048_576;
const VIEWPORT_WIDTH: f32 = 920.0;
const VIEWPORT_HEIGHT: f32 = 640.0;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ScrollBenchWorkload {
    TextHorizontalLongLine,
}

impl ScrollBenchWorkload {
    pub const ALL: [Self; 1] = [Self::TextHorizontalLongLine];

    pub fn name(self) -> &'static str {
        match self {
            Self::TextHorizontalLongLine => "text-horizontal-1m",
        }
    }

    pub fn transition_class(self) -> &'static str {
        match self {
            Self::TextHorizontalLongLine => "ResidencyCrossing",
        }
    }

    pub fn from_name(name: &str) -> Option<Self> {
        Self::ALL
            .into_iter()
            .find(|workload| workload.name() == name)
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
    offset_x: usize,
    near_window_width: usize,
    far_window_width: usize,
}

impl ScrollBenchReceipt {
    pub fn receipt_text(&self, commit: &str) -> String {
        format!(
            concat!(
                "scroll-bench-version={} workload={} transition_class={} commit={} ",
                "profile={} timer=std-instant os={} architecture={} ",
                "viewport={}x{} document_bytes={} document_lines={} offset_x={} logical_width={} ",
                "warmup={} samples={} official_matrix={} cold_us={} p50_us={} p95_us={} p99_us={} max_us={} ",
                "near_window_width={} far_window_width={} render_window_width_max={} render_window_height_max={} render_window_area_max={} bounded_window={} ",
                "paint_layout_calls={} visible_lines={} shaped_lines={} line_shape_calls={} render_surface_calls={} render_cache_hits={} render_cache_misses={} render_line_reuses={} ",
                "render_source_lines={} render_source_bytes={} render_total_us={} render_shape_us={} ",
                "width_cache_hits={} width_cache_misses={} width_observed_updates={} width_source_lines={} width_source_bytes={} width_measure_us={} caret_run_scans={} caret_glyph_scans={} highlight_run_scans={} ",
                "cold_render_line_reuses={} cold_render_source_bytes={} cold_render_shape_us={} cold_width_observed_updates={} cold_width_source_bytes={} cold_width_measure_us={}"
            ),
            SCROLL_BENCH_VERSION,
            self.workload.name(),
            self.workload.transition_class(),
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
            self.logical_width,
            self.warmup,
            self.samples.len(),
            self.warmup >= OFFICIAL_PROPERTY_WARMUP
                && self.samples.len() >= OFFICIAL_PROPERTY_SAMPLES,
            self.cold.as_micros(),
            percentile(&self.samples, 50).as_micros(),
            percentile(&self.samples, 95).as_micros(),
            percentile(&self.samples, 99).as_micros(),
            self.samples.last().copied().unwrap_or_default().as_micros(),
            self.near_window_width,
            self.far_window_width,
            self.diagnostics.text_area_render_window_width_max,
            self.diagnostics.text_area_render_window_height_max,
            self.diagnostics.text_area_render_window_area_max,
            self.far_window_width <= bounded_window_limit(),
            self.diagnostics.text_area_paint_layout_calls,
            self.diagnostics.text_area_visible_logical_lines,
            self.diagnostics.text_area_shaped_logical_lines,
            self.diagnostics.text_area_line_shape_calls,
            self.diagnostics.text_area_render_surface_calls,
            self.diagnostics.text_area_render_surface_cache_hits,
            self.diagnostics.text_area_render_surface_cache_misses,
            self.diagnostics.text_area_render_surface_line_reuses,
            self.diagnostics.text_area_render_surface_source_lines,
            self.diagnostics.text_area_render_surface_source_bytes,
            self.diagnostics.text_area_render_surface_total_us,
            self.diagnostics.text_area_render_surface_shape_us,
            self.diagnostics.text_area_width_cache_hits,
            self.diagnostics.text_area_width_cache_misses,
            self.diagnostics.text_area_width_observed_updates,
            self.diagnostics.text_area_width_source_lines,
            self.diagnostics.text_area_width_source_bytes,
            self.diagnostics.text_area_width_measure_us,
            self.diagnostics.text_area_caret_run_scans,
            self.diagnostics.text_area_caret_glyph_scans,
            self.diagnostics.highlight_run_scans,
            self.cold_diagnostics.text_area_render_surface_line_reuses,
            self.cold_diagnostics.text_area_render_surface_source_bytes,
            self.cold_diagnostics.text_area_render_surface_shape_us,
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
        ScrollBenchWorkload::TextHorizontalLongLine => {
            run_text_horizontal_long_line(warmup, samples)
        }
    }
}

fn run_text_horizontal_long_line(
    warmup: usize,
    samples: usize,
) -> Result<ScrollBenchReceipt, String> {
    let source = long_line_source();
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
    let logical_width = ceil_to_usize(cold_layout.layout().content_area().width());
    let near_window_width = cold_layout
        .render_surfaces()
        .first()
        .map(|surface| ceil_to_usize(surface.width()))
        .ok_or_else(|| "text-horizontal-1m produced no cold render surface".to_owned())?;
    let maximum_offset = logical_width.saturating_sub(VIEWPORT_WIDTH as usize);
    let offset_x = maximum_offset.saturating_mul(3) / 4;

    for index in 0..warmup {
        let offset = streamed_offset(offset_x, index, maximum_offset);
        black_box(engine.text_area_paint_layout_for_area_at(
            &area_model,
            style,
            viewport,
            ViewState::default().with_scroll_x(offset as f32),
            now,
        ));
    }

    engine.reset_diagnostics();
    let mut durations = Vec::with_capacity(samples);
    let mut far_window_width = 0_usize;
    for index in 0..samples {
        let offset = streamed_offset(offset_x, warmup.saturating_add(index), maximum_offset);
        let started = Instant::now();
        let layout = engine.text_area_paint_layout_for_area_at(
            &area_model,
            style,
            viewport,
            ViewState::default().with_scroll_x(offset as f32),
            now,
        );
        durations.push(started.elapsed());
        far_window_width = far_window_width.max(
            layout
                .render_surfaces()
                .first()
                .map(|surface| ceil_to_usize(surface.width()))
                .unwrap_or_default(),
        );
        black_box(layout);
    }
    durations.sort_unstable();
    let diagnostics = engine.diagnostics();

    Ok(ScrollBenchReceipt {
        workload: ScrollBenchWorkload::TextHorizontalLongLine,
        warmup,
        samples: durations,
        cold,
        cold_diagnostics,
        diagnostics,
        document_bytes,
        document_lines,
        logical_width,
        offset_x,
        near_window_width,
        far_window_width,
    })
}

fn long_line_source() -> String {
    const PATTERN: &str = "W0123456789 abcdefghijklmnopqrstuvwxyz ";
    let mut source = String::with_capacity(LONG_LINE_BYTES);
    while source.len() + PATTERN.len() <= LONG_LINE_BYTES {
        source.push_str(PATTERN);
    }
    source.push_str(&PATTERN[..LONG_LINE_BYTES - source.len()]);
    source
}

fn streamed_offset(base: usize, index: usize, maximum: usize) -> usize {
    let phase = index % 257;
    let displacement = if phase <= 128 { phase } else { 256 - phase };
    base.saturating_add(displacement.saturating_mul(4))
        .min(maximum)
}

fn bounded_window_limit() -> usize {
    (VIEWPORT_WIDTH + 2.0 * 256.0).ceil() as usize
}

fn ceil_to_usize(value: f32) -> usize {
    value.ceil().clamp(0.0, usize::MAX as f32) as usize
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
}
