#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub(crate) struct HighlightStats {
    pub run_scans: usize,
    pub highlight_calls: usize,
    pub spans: usize,
    pub skips: usize,
    pub cache_hits: usize,
    pub cache_misses: usize,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct Diagnostics {
    pub text_area_metrics_layout_calls: usize,
    pub text_area_paint_layout_calls: usize,
    pub text_area_render_surface_calls: usize,
    pub text_area_render_surface_cache_hits: usize,
    pub text_area_render_surface_cache_misses: usize,
    pub text_area_render_surface_line_reuses: usize,
    pub text_area_render_surface_source_lines: usize,
    pub text_area_render_surface_source_bytes: usize,
    pub text_area_render_surface_anchor_us: u128,
    pub text_area_render_surface_text_us: u128,
    pub text_area_render_surface_buffer_us: u128,
    pub text_area_render_surface_attrs_us: u128,
    pub text_area_render_surface_size_us: u128,
    pub text_area_render_surface_shape_us: u128,
    pub text_area_render_surface_metadata_us: u128,
    pub text_area_render_surface_total_us: u128,
    pub text_area_render_window_origin_x_max: usize,
    pub text_area_render_window_origin_y_max: usize,
    pub text_area_render_window_width_max: usize,
    pub text_area_render_window_height_max: usize,
    pub text_area_render_window_area_max: usize,
    pub text_area_line_cache_hits: usize,
    pub text_area_line_cache_misses: usize,
    pub text_area_line_shape_calls: usize,
    pub text_area_horizontal_index_builds: usize,
    pub text_area_horizontal_index_hits: usize,
    pub text_area_horizontal_index_misses: usize,
    pub text_area_horizontal_index_evictions: usize,
    pub text_area_horizontal_index_incremental_updates: usize,
    pub text_area_horizontal_index_incremental_source_bytes: usize,
    pub text_area_horizontal_index_incremental_source_bytes_max: usize,
    pub text_area_horizontal_index_incremental_glyphs: usize,
    pub text_area_horizontal_index_incremental_glyphs_max: usize,
    pub text_area_horizontal_index_source_bytes: usize,
    pub text_area_horizontal_index_glyphs: usize,
    pub text_area_horizontal_index_checkpoints: usize,
    pub text_area_horizontal_exact_band_shapes: usize,
    pub text_area_horizontal_exact_band_source_bytes: usize,
    pub text_area_horizontal_index_resident_bytes_max: usize,
    pub text_area_horizontal_window_shapes: usize,
    pub text_area_horizontal_window_source_bytes: usize,
    pub text_area_horizontal_resident_source_bytes_max: usize,
    pub text_area_horizontal_resident_glyphs_max: usize,
    pub text_area_horizontal_resident_bytes_max: usize,
    pub text_area_line_cache_resident_bytes_max: usize,
    pub text_area_shaped_logical_lines: usize,
    pub text_area_shaped_visual_lines: usize,
    pub text_area_visible_logical_lines: usize,
    pub text_area_layout_segments: usize,
    pub text_area_overscan_segments: usize,
    pub text_area_interaction_surfaces: usize,
    pub text_area_hit_run_scans: usize,
    pub text_area_height_index_hits: usize,
    pub text_area_height_index_misses: usize,
    pub text_area_height_index_queries: usize,
    pub text_area_height_index_updates: usize,
    pub text_area_height_index_refined_pixels: usize,
    pub text_area_anchor_candidates: usize,
    pub text_area_anchor_corrections: usize,
    pub text_area_anchor_correction_pixels: usize,
    pub text_area_anchor_correction_pixels_max: usize,
    pub text_area_width_cache_hits: usize,
    pub text_area_width_cache_misses: usize,
    pub text_area_width_observed_updates: usize,
    pub text_area_width_incremental_updates: usize,
    pub text_area_width_incremental_source_bytes: usize,
    pub text_area_width_incremental_source_bytes_max: usize,
    pub text_area_width_source_lines: usize,
    pub text_area_width_source_bytes: usize,
    pub text_area_width_measure_us: u128,
    pub text_area_caret_run_scans: usize,
    pub text_area_caret_glyph_scans: usize,
    pub highlight_run_scans: usize,
    pub highlight_spans: usize,
    pub highlight_skips: usize,
}

impl Diagnostics {
    pub(super) fn add_highlight_stats(&mut self, stats: HighlightStats) {
        self.highlight_run_scans += stats.run_scans;
        self.highlight_spans += stats.spans;
        self.highlight_skips += stats.skips;
    }
}

impl HighlightStats {
    pub(super) fn record_run_scan(&mut self) {
        self.run_scans += 1;
    }

    pub(super) fn record_span(&mut self) {
        self.spans += 1;
    }

    pub(super) fn record_skip(&mut self) {
        self.skips += 1;
    }

    #[cfg(test)]
    pub(super) fn add(&mut self, other: Self) {
        self.run_scans += other.run_scans;
        self.highlight_calls += other.highlight_calls;
        self.spans += other.spans;
        self.skips += other.skips;
        self.cache_hits += other.cache_hits;
        self.cache_misses += other.cache_misses;
    }
}

#[cfg(test)]
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub(in crate::text) struct TextInteractionStats {
    pub(in crate::text) text_area_shape_until_scroll_calls: usize,
    pub(in crate::text) text_area_frame_cache_hits: usize,
    pub(in crate::text) text_area_frame_cache_misses: usize,
    pub(in crate::text) text_area_frame_shape_calls: usize,
    pub(in crate::text) text_area_frame_shaped_logical_lines: usize,
    pub(in crate::text) text_area_frame_shaped_visual_lines: usize,
    pub(in crate::text) hit_run_scans: usize,
    pub(in crate::text) aggregate_buffer_fallbacks: usize,
}
