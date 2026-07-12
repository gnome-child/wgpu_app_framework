#[cfg(test)]
use super::super::composition;
#[cfg(test)]
use super::super::pointer;
use super::super::{
    clipboard::Clipboard,
    diagnostics::Diagnostics,
    session,
    state::{self, Store},
    timeline::Timeline,
    window,
};
use super::Runtime;

impl<M: state::State, E: Send + 'static, V> Runtime<M, E, V> {
    pub fn state(&self) -> &M {
        self.store.model()
    }

    pub fn store(&self) -> &Store<M> {
        &self.store
    }

    pub fn timeline(&self) -> &Timeline<M> {
        &self.timeline
    }

    pub fn session(&self) -> &session::Session {
        &self.session
    }

    #[cfg(test)]
    pub(crate) fn composition(&self, window: window::Id) -> Option<&composition::Composition> {
        self.composition.get(window)
    }

    pub fn requests(&self) -> Vec<session::Request> {
        self.session.requests()
    }

    pub fn request_redraw(&mut self, window: window::Id) -> bool {
        self.session.request_redraw(window)
    }

    pub fn clear_redraw_request(&mut self, window: window::Id) -> bool {
        self.session.clear_redraw_request(window)
    }

    #[cfg(test)]
    pub(crate) fn set_pointer_cursor_for_test(
        &mut self,
        window: window::Id,
        cursor: pointer::Cursor,
    ) -> bool {
        self.session.set_cursor(window, cursor)
    }

    pub fn clipboard(&self) -> &Clipboard {
        &self.clipboard
    }

    pub fn revision(&self) -> state::Revision {
        self.store.revision()
    }

    pub fn is_dirty(&self) -> bool {
        self.store.is_dirty()
    }

    pub fn mark_saved(&mut self) {
        self.store.mark_saved();
        self.request_all_redraws();
    }

    pub fn diagnostics(&self, window: window::Id) -> Option<&Diagnostics> {
        if !self.session.contains(window) {
            return None;
        }

        self.diagnostics.get(window)
    }

    pub fn diagnostics_mut(&mut self, window: window::Id) -> Option<&mut Diagnostics> {
        if !self.session.contains(window) {
            return None;
        }

        Some(self.diagnostics.get_mut(window))
    }

    pub(crate) fn record_input_latency_sample(
        &mut self,
        window: window::Id,
        started_at: std::time::Instant,
    ) {
        if !self.session.contains(window) {
            return;
        }

        let Some(epoch) = self
            .session
            .window(window)
            .map(session::Window::desired_presentation_epoch)
        else {
            return;
        };
        self.diagnostics
            .get_mut(window)
            .render
            .record_input(epoch, started_at);
    }

    pub(crate) fn record_native_translation(
        &mut self,
        window: window::Id,
        duration: std::time::Duration,
    ) {
        if !self.session.contains(window) {
            return;
        }

        self.diagnostics
            .get_mut(window)
            .pipeline
            .record_native_translation(duration);
    }

    pub(crate) fn record_event_handling(
        &mut self,
        window: window::Id,
        duration: std::time::Duration,
    ) {
        if !self.session.contains(window) {
            return;
        }

        self.diagnostics
            .get_mut(window)
            .pipeline
            .record_event_handling(duration);
    }

    pub(crate) fn record_native_event_pass(
        &mut self,
        window: window::Id,
        duration: std::time::Duration,
    ) {
        if !self.session.contains(window) {
            return;
        }

        self.diagnostics
            .get_mut(window)
            .pipeline
            .record_native_event_pass(duration);
    }

    pub(crate) fn finish_render_report(
        &mut self,
        window: window::Id,
        epoch: window::PresentationEpoch,
        invalidation: super::super::response::Invalidation,
        layout: &super::super::layout::Layout,
        report: super::super::diagnostics::RenderReport,
    ) -> bool {
        if !self.session.contains(window) {
            return false;
        }

        let diagnostics = self.diagnostics.get_mut(window);
        diagnostics.render.record_present(epoch, report);
        if diagnostics.render.frames_presented.is_multiple_of(10) {
            log::debug!(
                target: "wgpu_l3::presentation_clock",
                "events={} prepared={} attempted={} presented={} view_rebuilds={} layout_recomposes={} layout_reuses={} routing_layouts={} event_p95_us={} native_p95_us={} rebuild_p95_us={} reconcile_p95_us={} routing_layout_p95_us={} presentation_layout_p95_us={} scene_p95_us={} batch_p95_us={} acquire_p95_us={} encode_present_p95_us={} draw_p95_us={} interval_p95_us={} scene_items={} batches={} glyph_batches={} text_hits={} text_misses={} shape_calls={} text_hits_total={} text_misses_total={} shape_calls_total={}",
                diagnostics.pipeline.events_received,
                diagnostics.pipeline.frames_prepared,
                diagnostics.render.frames_attempted,
                diagnostics.render.frames_presented,
                diagnostics.frame.view_rebuilds,
                diagnostics.frame.layout_recomposes,
                diagnostics.frame.layout_reuses,
                diagnostics.pipeline.routing_layouts,
                diagnostics.pipeline.event_handling_p95_us(),
                diagnostics.pipeline.native_event_pass_p95_us(),
                diagnostics.pipeline.view_rebuild_p95_us(),
                diagnostics.pipeline.composition_reconciliation_p95_us(),
                diagnostics.pipeline.routing_layout_p95_us(),
                diagnostics.pipeline.presentation_layout_p95_us(),
                diagnostics.pipeline.scene_assembly_p95_us(),
                diagnostics.render.batch_prepare_p95_us(),
                diagnostics.render.acquire_wait_p95_us(),
                diagnostics.render.encode_submit_present_p95_us(),
                diagnostics.render.draw_p95_us(),
                diagnostics.render.interval_p95_us(),
                diagnostics.render.scene_items,
                diagnostics.render.render_batches,
                diagnostics.render.glyph_batches,
                diagnostics.render.inline_text_cache_hits,
                diagnostics.render.inline_text_cache_misses,
                diagnostics.render.inline_text_shape_calls,
                diagnostics.render.inline_text_cache_hits_total,
                diagnostics.render.inline_text_cache_misses_total,
                diagnostics.render.inline_text_shape_calls_total,
            );
        }

        if report.presented() {
            if self.session.acknowledge_presentation(window, epoch) {
                self.presented_geometry.insert(
                    window,
                    super::PresentedGeometry {
                        layout: std::sync::Arc::new(layout.clone()),
                    },
                );
            }
        } else {
            self.session.retry_invalidation(window, invalidation);
        }
        !report.presented()
    }

    pub(crate) fn presented_layout(
        &self,
        window: window::Id,
    ) -> Option<std::sync::Arc<super::super::layout::Layout>> {
        self.presented_geometry
            .get(&window)
            .map(|geometry| std::sync::Arc::clone(&geometry.layout))
    }

    #[cfg(test)]
    pub(crate) fn acknowledged_presentation_epoch(
        &self,
        window: window::Id,
    ) -> Option<window::PresentationEpoch> {
        self.session
            .window(window)
            .and_then(session::Window::acknowledged_presentation_epoch)
    }

    pub(in crate::runtime) fn request_all_redraws(&mut self) {
        let windows = self
            .session
            .windows()
            .iter()
            .map(session::Window::id)
            .collect::<Vec<_>>();

        for window in windows {
            self.session.request_redraw(window);
        }
    }
}
