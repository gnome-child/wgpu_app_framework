use std::collections::HashMap;

use crate::text;

use super::{session, window};

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Diagnostics {
    pub text: Text,
    pub scroll: Scroll,
    pub frame: Frame,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Text {
    pub text_area_paint_layout_calls: usize,
    pub text_area_metrics_layout_calls: usize,
    pub text_area_visible_logical_lines: usize,
    pub text_area_shaped_logical_lines: usize,
    pub text_area_layout_segments: usize,
    pub text_area_overscan_segments: usize,
    pub text_area_interaction_surfaces: usize,
    pub highlight_run_scans: usize,
    pub text_area_line_cache_hits: usize,
    pub text_area_line_cache_misses: usize,
    pub text_area_render_surface_calls: usize,
    pub text_area_render_surface_cache_hits: usize,
    pub text_area_render_surface_cache_misses: usize,
    pub text_area_render_surface_source_lines: usize,
    pub text_area_render_surface_source_bytes: usize,
}

impl Text {
    pub(super) fn add(&mut self, diagnostics: Self) {
        self.text_area_paint_layout_calls += diagnostics.text_area_paint_layout_calls;
        self.text_area_metrics_layout_calls += diagnostics.text_area_metrics_layout_calls;
        self.text_area_visible_logical_lines += diagnostics.text_area_visible_logical_lines;
        self.text_area_shaped_logical_lines += diagnostics.text_area_shaped_logical_lines;
        self.text_area_layout_segments += diagnostics.text_area_layout_segments;
        self.text_area_overscan_segments += diagnostics.text_area_overscan_segments;
        self.text_area_interaction_surfaces += diagnostics.text_area_interaction_surfaces;
        self.highlight_run_scans += diagnostics.highlight_run_scans;
        self.text_area_line_cache_hits += diagnostics.text_area_line_cache_hits;
        self.text_area_line_cache_misses += diagnostics.text_area_line_cache_misses;
        self.text_area_render_surface_calls += diagnostics.text_area_render_surface_calls;
        self.text_area_render_surface_cache_hits += diagnostics.text_area_render_surface_cache_hits;
        self.text_area_render_surface_cache_misses +=
            diagnostics.text_area_render_surface_cache_misses;
        self.text_area_render_surface_source_lines +=
            diagnostics.text_area_render_surface_source_lines;
        self.text_area_render_surface_source_bytes +=
            diagnostics.text_area_render_surface_source_bytes;
    }

    pub(super) fn add_text_layout(&mut self, diagnostics: text::layout::Diagnostics) {
        self.text_area_paint_layout_calls += diagnostics.text_area_paint_layout_calls;
        self.text_area_metrics_layout_calls += diagnostics.text_area_metrics_layout_calls;
        self.text_area_visible_logical_lines += diagnostics.text_area_visible_logical_lines;
        self.text_area_shaped_logical_lines += diagnostics.text_area_shaped_logical_lines;
        self.text_area_layout_segments += diagnostics.text_area_layout_segments;
        self.text_area_overscan_segments += diagnostics.text_area_overscan_segments;
        self.text_area_interaction_surfaces += diagnostics.text_area_interaction_surfaces;
        self.highlight_run_scans += diagnostics.highlight_run_scans;
        self.text_area_line_cache_hits += diagnostics.text_area_line_cache_hits;
        self.text_area_line_cache_misses += diagnostics.text_area_line_cache_misses;
        self.text_area_render_surface_calls += diagnostics.text_area_render_surface_calls;
        self.text_area_render_surface_cache_hits += diagnostics.text_area_render_surface_cache_hits;
        self.text_area_render_surface_cache_misses +=
            diagnostics.text_area_render_surface_cache_misses;
        self.text_area_render_surface_source_lines +=
            diagnostics.text_area_render_surface_source_lines;
        self.text_area_render_surface_source_bytes +=
            diagnostics.text_area_render_surface_source_bytes;
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Scroll {
    pub wheel_events: usize,
    pub scroll_offset_changes: usize,
    pub queued_scroll_updates: usize,
    pub scroll_redraw_requests: usize,
    pub frame_scroll_commits: usize,
    pub pending_scroll_applications: usize,
    pub pending_scroll_updates: usize,
    pub projection_count: usize,
    pub text_area_resolves: usize,
    pub text_area_projection_reuses: usize,
    pub text_area_projection_shifts: usize,
    pub text_area_projection_shift_misses: usize,
    pub text_area_projection_cold_jumps: usize,
    pub async_scroll_reconciles: usize,
    pub async_scroll_projection_sync_skips: usize,
    pub retained_scroll_layer_hits: usize,
    pub retained_scroll_layer_text_prepare_skips: usize,
    pub retained_scroll_target_repaint_fallbacks: usize,
    pub retained_scroll_layer_rebuilds: usize,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Frame {
    pub full_redraws: usize,
    pub scroll_only_redraws: usize,
    pub scroll_only_fallbacks_to_full: usize,
    pub render_skips: usize,
    pub paint: Timing,
    pub render: Timing,
    pub render_text_prepare: Timing,
    pub total: Timing,
    pub last_scroll_frame: LastScrollFrame,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Timing {
    pub latest_us: u64,
    pub average_us: u64,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct LastScrollFrame {
    pub render_text_prepare_us: u64,
    pub render_total_us: u64,
    pub total_us: u64,
    pub text_surfaces: usize,
    pub glyph_batches: usize,
}

#[derive(Default)]
pub(super) struct Store {
    windows: HashMap<window::Id, Diagnostics>,
}

impl Store {
    pub(super) fn insert_window(&mut self, window: window::Id) {
        self.windows.entry(window).or_default();
    }

    pub(super) fn remove_window(&mut self, window: window::Id) {
        self.windows.remove(&window);
    }

    pub(super) fn restore_windows(&mut self, windows: &[session::Window]) {
        self.windows.clear();
        for window in windows {
            self.insert_window(window.id());
        }
    }

    pub(super) fn get(&self, window: window::Id) -> Option<&Diagnostics> {
        self.windows.get(&window)
    }

    pub(super) fn get_mut(&mut self, window: window::Id) -> &mut Diagnostics {
        self.windows.entry(window).or_default()
    }
}
