use crate::text;

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
    pub(crate) fn add(&mut self, diagnostics: Self) {
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

    pub(crate) fn add_text_layout(&mut self, diagnostics: text::layout::Diagnostics) {
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
