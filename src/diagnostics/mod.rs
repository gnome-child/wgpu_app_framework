mod frame;
mod pipeline;
mod render;
mod samples;
mod scroll;
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
};
pub use scroll::Scroll;

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
        let _ = writeln!(
            receipt,
            "text_area_render_surface_calls={}",
            self.text.text_area_render_surface_calls
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
            "frame_scroll_commits=0",
            "text_area_paint_layout_calls=0",
            "text_area_render_surface_calls=0",
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
