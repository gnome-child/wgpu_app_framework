mod frame;
mod pipeline;
mod render;
#[cfg(feature = "renderer-debug")]
mod residency;
mod samples;
mod scroll;
#[cfg(feature = "renderer-debug")]
mod scroll_bench;
mod store;

pub use crate::layout::Text;
pub use crate::render::RenderReport;
pub use crate::render::RendererEnvironment;
pub use frame::Frame;
pub use pipeline::Pipeline;
pub use render::Render;
#[cfg(feature = "renderer-debug")]
#[doc(hidden)]
pub use render::{
    compare_control_gallery_caret_blink, compare_control_gallery_horizontal_table_scroll,
    compare_control_gallery_incremental_activation,
    compare_control_gallery_pending_property_refresh, compare_control_gallery_pending_transition,
    compare_control_gallery_property_tick, compare_control_gallery_slow_scroll,
    compare_group_under_scroll_first_tick, compare_payload_neutral_scroll_oracles,
    measure_control_gallery_horizontal_table_scroll,
    require_payload_neutral_scroll_negative_controls,
};
#[cfg(feature = "renderer-debug")]
#[doc(hidden)]
pub use residency::{
    ResidencyCrossingReceipt, ResidencyPayload, compare_table_runway_property_text,
    measure_residency_crossing_work,
};
pub(crate) use scroll::CandidateWork;
pub use scroll::Scroll;
#[cfg(feature = "renderer-debug")]
#[doc(hidden)]
pub use scroll_bench::{
    OFFICIAL_PROPERTY_SAMPLES, OFFICIAL_PROPERTY_WARMUP, SCROLL_BENCH_VERSION, ScrollBenchReceipt,
    ScrollBenchWorkload, run_scroll_bench,
};

pub(crate) use store::Store;

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Diagnostics {
    pub text: Text,
    pub scroll: Scroll,
    pub frame: Frame,
    pub pipeline: Pipeline,
    pub render: Render,
}

impl Diagnostics {
    pub fn begin_renderer_measurement(&mut self) {
        let environment = self.render.environment.clone();
        *self = Self::default();
        self.render.environment = environment;
    }

    pub fn renderer_receipt_text(&self, workload: &str) -> String {
        use std::fmt::Write as _;

        let mut receipt = self.render.receipt_text(workload);
        let _ = writeln!(receipt, "events_received={}", self.pipeline.events_received);
        let _ = writeln!(receipt, "frames_prepared={}", self.pipeline.frames_prepared);
        let _ = writeln!(
            receipt,
            "scene_assembly_p95_us={}",
            self.pipeline.scene_assembly_p95_us()
        );
        let _ = writeln!(
            receipt,
            "native_translation_p95_us={}",
            self.pipeline.native_translation_p95_us()
        );
        let _ = writeln!(
            receipt,
            "event_handling_p95_us={}",
            self.pipeline.event_handling_p95_us()
        );
        let _ = writeln!(
            receipt,
            "native_event_pass_p95_us={}",
            self.pipeline.native_event_pass_p95_us()
        );
        let _ = writeln!(
            receipt,
            "view_rebuild_p95_us={}",
            self.pipeline.view_rebuild_p95_us()
        );
        let _ = writeln!(
            receipt,
            "composition_reconciliation_p95_us={}",
            self.pipeline.composition_reconciliation_p95_us()
        );
        let _ = writeln!(
            receipt,
            "presentation_layout_p95_us={}",
            self.pipeline.presentation_layout_p95_us()
        );
        let _ = writeln!(receipt, "full_redraws={}", self.frame.full_redraws);
        let _ = writeln!(receipt, "view_rebuilds={}", self.frame.view_rebuilds);
        let _ = writeln!(
            receipt,
            "layout_recomposes={}",
            self.frame.layout_recomposes
        );
        let _ = writeln!(receipt, "layout_reuses={}", self.frame.layout_reuses);
        let _ = writeln!(receipt, "wheel_events={}", self.scroll.wheel_events);
        let _ = writeln!(
            receipt,
            "scroll_input_events={}",
            self.scroll.scroll_input_events
        );
        let _ = writeln!(
            receipt,
            "scroll_desired_changes={}",
            self.scroll.scroll_desired_changes
        );
        let _ = writeln!(
            receipt,
            "scroll_resident_acceptances={}",
            self.scroll.scroll_resident_acceptances
        );
        let _ = writeln!(receipt, "scroll_unchanged={}", self.scroll.scroll_unchanged);
        let _ = writeln!(
            receipt,
            "scroll_property_ticks={}",
            self.scroll.scroll_property_ticks
        );
        let _ = writeln!(
            receipt,
            "scroll_proactive_replenishments={}",
            self.scroll.scroll_proactive_replenishments
        );
        let _ = writeln!(
            receipt,
            "scroll_residency_candidates_scheduled={}",
            self.scroll.scroll_residency_candidates_scheduled
        );
        let _ = writeln!(
            receipt,
            "scroll_residency_candidates_coalesced={}",
            self.scroll.scroll_residency_candidates_coalesced
        );
        let _ = writeln!(
            receipt,
            "scroll_residency_candidates_selected={}",
            self.scroll.scroll_residency_candidates_selected
        );
        let _ = writeln!(
            receipt,
            "scroll_residency_candidates_superseded={}",
            self.scroll.scroll_residency_candidates_superseded
        );
        let _ = writeln!(
            receipt,
            "scroll_residency_proactive_preemptions={}",
            self.scroll.scroll_residency_proactive_preemptions
        );
        let _ = writeln!(
            receipt,
            "scroll_residency_pipelines_cancelled={}",
            self.scroll.scroll_residency_pipelines_cancelled
        );
        let _ = writeln!(
            receipt,
            "scroll_residency_follow_ups={}",
            self.scroll.scroll_residency_follow_ups
        );
        let _ = writeln!(
            receipt,
            "scroll_needs_residency={}",
            self.scroll.scroll_needs_residency
        );
        let _ = writeln!(
            receipt,
            "scroll_desired_resident_lag_x_max={}",
            self.scroll.desired_resident_lag_x_max
        );
        let _ = writeln!(
            receipt,
            "scroll_desired_resident_lag_y_max={}",
            self.scroll.desired_resident_lag_y_max
        );
        let _ = writeln!(
            receipt,
            "virtual_residency_rejections={}",
            self.scroll.virtual_residency_rejections
        );
        let _ = writeln!(
            receipt,
            "virtual_residency_last_issue={}",
            self.scroll
                .virtual_residency_last_issue
                .as_deref()
                .unwrap_or("none")
        );
        let _ = writeln!(
            receipt,
            "scroll_request_p95_us={}",
            self.scroll.request_p95_us()
        );
        let _ = writeln!(
            receipt,
            "scroll_property_tick_p95_us={}",
            self.scroll.property_tick_p95_us()
        );
        let _ = writeln!(
            receipt,
            "scroll_needs_residency_p95_us={}",
            self.scroll.needs_residency_p95_us()
        );
        receipt.push_str(&self.scroll.trace_receipt_text());
        let _ = writeln!(
            receipt,
            "scroll_offset_changes={}",
            self.scroll.scroll_offset_changes
        );
        let _ = writeln!(
            receipt,
            "scroll_redraw_requests={}",
            self.scroll.scroll_redraw_requests
        );
        let _ = writeln!(
            receipt,
            "frame_scroll_commits={}",
            self.scroll.frame_scroll_commits
        );
        let _ = writeln!(
            receipt,
            "text_area_paint_layout_calls={}",
            self.text.text_area_paint_layout_calls
        );
        let _ = writeln!(
            receipt,
            "text_area_shaped_logical_lines={}",
            self.text.text_area_shaped_logical_lines
        );
        let _ = writeln!(
            receipt,
            "text_area_line_cache_hits={}",
            self.text.text_area_line_cache_hits
        );
        let _ = writeln!(
            receipt,
            "text_area_line_cache_misses={}",
            self.text.text_area_line_cache_misses
        );
        for (name, value) in [
            (
                "text_area_horizontal_index_builds",
                self.text.text_area_horizontal_index_builds,
            ),
            (
                "text_area_horizontal_index_hits",
                self.text.text_area_horizontal_index_hits,
            ),
            (
                "text_area_horizontal_index_misses",
                self.text.text_area_horizontal_index_misses,
            ),
            (
                "text_area_horizontal_index_evictions",
                self.text.text_area_horizontal_index_evictions,
            ),
            (
                "text_area_horizontal_index_incremental_updates",
                self.text.text_area_horizontal_index_incremental_updates,
            ),
            (
                "text_area_horizontal_index_incremental_source_bytes",
                self.text
                    .text_area_horizontal_index_incremental_source_bytes,
            ),
            (
                "text_area_horizontal_index_incremental_glyphs",
                self.text.text_area_horizontal_index_incremental_glyphs,
            ),
            (
                "text_area_horizontal_index_source_bytes",
                self.text.text_area_horizontal_index_source_bytes,
            ),
            (
                "text_area_horizontal_index_glyphs",
                self.text.text_area_horizontal_index_glyphs,
            ),
            (
                "text_area_horizontal_index_checkpoints",
                self.text.text_area_horizontal_index_checkpoints,
            ),
            (
                "text_area_horizontal_exact_band_shapes",
                self.text.text_area_horizontal_exact_band_shapes,
            ),
            (
                "text_area_horizontal_exact_band_source_bytes",
                self.text.text_area_horizontal_exact_band_source_bytes,
            ),
            (
                "text_area_horizontal_index_resident_bytes_max",
                self.text.text_area_horizontal_index_resident_bytes_max,
            ),
            (
                "text_area_horizontal_window_shapes",
                self.text.text_area_horizontal_window_shapes,
            ),
            (
                "text_area_horizontal_window_source_bytes",
                self.text.text_area_horizontal_window_source_bytes,
            ),
            (
                "text_area_horizontal_resident_source_bytes_max",
                self.text.text_area_horizontal_resident_source_bytes_max,
            ),
            (
                "text_area_horizontal_resident_glyphs_max",
                self.text.text_area_horizontal_resident_glyphs_max,
            ),
            (
                "text_area_horizontal_resident_bytes_max",
                self.text.text_area_horizontal_resident_bytes_max,
            ),
            (
                "text_area_line_cache_resident_bytes_max",
                self.text.text_area_line_cache_resident_bytes_max,
            ),
        ] {
            let _ = writeln!(receipt, "{name}={value}");
        }
        let _ = writeln!(
            receipt,
            "text_area_render_surface_calls={}",
            self.text.text_area_render_surface_calls
        );
        let _ = writeln!(
            receipt,
            "text_area_render_surface_cache_hits={}",
            self.text.text_area_render_surface_cache_hits
        );
        let _ = writeln!(
            receipt,
            "text_area_render_surface_cache_misses={}",
            self.text.text_area_render_surface_cache_misses
        );
        let _ = writeln!(
            receipt,
            "text_area_render_surface_line_reuses={}",
            self.text.text_area_render_surface_line_reuses
        );
        let _ = writeln!(
            receipt,
            "text_area_render_surface_source_lines={}",
            self.text.text_area_render_surface_source_lines
        );
        let _ = writeln!(
            receipt,
            "text_area_render_surface_source_bytes={}",
            self.text.text_area_render_surface_source_bytes
        );
        let _ = writeln!(
            receipt,
            "text_area_render_surface_total_us={}",
            self.text.text_area_render_surface_total_us
        );
        let _ = writeln!(
            receipt,
            "text_area_render_surface_shape_us={}",
            self.text.text_area_render_surface_shape_us
        );
        let _ = writeln!(
            receipt,
            "text_area_render_window_origin_x_max={}",
            self.text.text_area_render_window_origin_x_max
        );
        let _ = writeln!(
            receipt,
            "text_area_render_window_origin_y_max={}",
            self.text.text_area_render_window_origin_y_max
        );
        let _ = writeln!(
            receipt,
            "text_area_render_window_width_max={}",
            self.text.text_area_render_window_width_max
        );
        let _ = writeln!(
            receipt,
            "text_area_render_window_height_max={}",
            self.text.text_area_render_window_height_max
        );
        let _ = writeln!(
            receipt,
            "text_area_render_window_area_max={}",
            self.text.text_area_render_window_area_max
        );
        let _ = writeln!(
            receipt,
            "text_area_height_index_hits={}",
            self.text.text_area_height_index_hits
        );
        let _ = writeln!(
            receipt,
            "text_area_height_index_misses={}",
            self.text.text_area_height_index_misses
        );
        let _ = writeln!(
            receipt,
            "text_area_height_index_queries={}",
            self.text.text_area_height_index_queries
        );
        let _ = writeln!(
            receipt,
            "text_area_height_index_updates={}",
            self.text.text_area_height_index_updates
        );
        let _ = writeln!(
            receipt,
            "text_area_height_index_refined_pixels={}",
            self.text.text_area_height_index_refined_pixels
        );
        let _ = writeln!(
            receipt,
            "text_area_anchor_candidates={}",
            self.text.text_area_anchor_candidates
        );
        let _ = writeln!(
            receipt,
            "text_area_anchor_corrections={}",
            self.text.text_area_anchor_corrections
        );
        let _ = writeln!(
            receipt,
            "text_area_anchor_correction_pixels={}",
            self.text.text_area_anchor_correction_pixels
        );
        let _ = writeln!(
            receipt,
            "text_area_anchor_correction_pixels_max={}",
            self.text.text_area_anchor_correction_pixels_max
        );
        let _ = writeln!(
            receipt,
            "text_area_width_cache_hits={}",
            self.text.text_area_width_cache_hits
        );
        let _ = writeln!(
            receipt,
            "text_area_width_cache_misses={}",
            self.text.text_area_width_cache_misses
        );
        let _ = writeln!(
            receipt,
            "text_area_width_observed_updates={}",
            self.text.text_area_width_observed_updates
        );
        let _ = writeln!(
            receipt,
            "text_area_width_incremental_updates={}",
            self.text.text_area_width_incremental_updates
        );
        let _ = writeln!(
            receipt,
            "text_area_width_incremental_source_bytes={}",
            self.text.text_area_width_incremental_source_bytes
        );
        let _ = writeln!(
            receipt,
            "text_area_width_incremental_source_bytes_max={}",
            self.text.text_area_width_incremental_source_bytes_max
        );
        let _ = writeln!(
            receipt,
            "text_area_width_source_lines={}",
            self.text.text_area_width_source_lines
        );
        let _ = writeln!(
            receipt,
            "text_area_width_source_bytes={}",
            self.text.text_area_width_source_bytes
        );
        let _ = writeln!(
            receipt,
            "text_area_width_measure_us={}",
            self.text.text_area_width_measure_us
        );
        let _ = writeln!(
            receipt,
            "text_area_caret_run_scans={}",
            self.text.text_area_caret_run_scans
        );
        let _ = writeln!(
            receipt,
            "text_area_caret_glyph_scans={}",
            self.text.text_area_caret_glyph_scans
        );
        receipt
    }
}

#[cfg(test)]
mod tests {
    use super::Diagnostics;

    #[test]
    fn renderer_receipt_includes_upstream_scene_scroll_and_text_work() {
        let receipt = Diagnostics::default().renderer_receipt_text("receipt-test");

        for field in [
            "scene_assembly_p95_us=0",
            "event_handling_p95_us=0",
            "presentation_layout_p95_us=0",
            "scroll_input_events=0",
            "scroll_property_ticks=0",
            "scroll_proactive_replenishments=0",
            "scroll_residency_candidates_scheduled=0",
            "scroll_residency_candidates_coalesced=0",
            "scroll_residency_candidates_selected=0",
            "scroll_residency_candidates_superseded=0",
            "scroll_residency_proactive_preemptions=0",
            "scroll_residency_pipelines_cancelled=0",
            "scroll_residency_follow_ups=0",
            "scroll_needs_residency=0",
            "scroll_request_p95_us=0",
            "scroll_trace_schema=wgpu_l3.scroll_trace.v3",
            "scroll_trace_count=0",
            "frame_scroll_commits=0",
            "text_area_paint_layout_calls=0",
            "text_area_horizontal_index_builds=0",
            "text_area_horizontal_resident_bytes_max=0",
            "text_area_render_surface_calls=0",
            "text_area_render_surface_line_reuses=0",
            "text_area_render_window_area_max=0",
            "text_area_width_observed_updates=0",
            "text_area_width_source_bytes=0",
        ] {
            assert!(receipt.contains(field), "missing receipt field {field}");
        }
    }

    #[test]
    fn renderer_measurement_reset_clears_upstream_and_render_currencies() {
        let mut diagnostics = Diagnostics::default();
        diagnostics.frame.full_redraws = 3;
        diagnostics.render.frames_attempted = 4;

        diagnostics.begin_renderer_measurement();

        assert_eq!(diagnostics.frame.full_redraws, 0);
        assert_eq!(diagnostics.render.frames_attempted, 0);
    }
}
