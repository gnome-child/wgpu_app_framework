#[cfg(test)]
use super::super::composition;
use super::super::pointer;
use super::super::{
    clipboard::Clipboard,
    diagnostics::Diagnostics,
    interaction, session,
    state::{self, Store},
    timeline::Timeline,
    window,
};
use super::Runtime;

#[derive(Clone, Copy, PartialEq, Eq)]
enum FinishKind {
    Candidate { property_only: bool },
    ActiveRefresh,
}

impl FinishKind {
    fn property_only(self) -> bool {
        match self {
            Self::Candidate { property_only } => property_only,
            Self::ActiveRefresh => true,
        }
    }

    fn refreshes_active(self) -> bool {
        self == Self::ActiveRefresh
    }
}

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

    pub(crate) fn take_cursor_updates(&mut self) -> Vec<pointer::Update> {
        self.session.take_cursor_updates()
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
            .map(session::Window::requested_presentation_epoch)
        else {
            return;
        };
        let diagnostics = self.diagnostics.get_mut(window);
        diagnostics.render.record_input(epoch, started_at);
        diagnostics.scroll.record_input(epoch, started_at);
    }

    pub(crate) fn requested_presentation_epoch(
        &self,
        window: window::Id,
    ) -> Option<window::PresentationEpoch> {
        self.session
            .window(window)
            .map(session::Window::requested_presentation_epoch)
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
        invalidation: super::super::response::effect::Invalidation,
        layout: &super::super::layout::Layout,
        stack: &std::sync::Arc<super::super::scene::Stack>,
        property_only: bool,
        report: super::super::diagnostics::RenderReport,
    ) -> bool {
        self.finish_render_report_with_kind(
            window,
            epoch,
            invalidation,
            layout,
            stack,
            FinishKind::Candidate { property_only },
            report,
        )
    }

    pub(crate) fn finish_active_refresh(
        &mut self,
        window: window::Id,
        epoch: window::PresentationEpoch,
        invalidation: super::super::response::effect::Invalidation,
        layout: &super::super::layout::Layout,
        stack: &std::sync::Arc<super::super::scene::Stack>,
        report: super::super::diagnostics::RenderReport,
    ) -> bool {
        self.finish_render_report_with_kind(
            window,
            epoch,
            invalidation,
            layout,
            stack,
            FinishKind::ActiveRefresh,
            report,
        )
    }

    fn finish_render_report_with_kind(
        &mut self,
        window: window::Id,
        epoch: window::PresentationEpoch,
        invalidation: super::super::response::effect::Invalidation,
        layout: &super::super::layout::Layout,
        stack: &std::sync::Arc<super::super::scene::Stack>,
        kind: FinishKind,
        report: super::super::diagnostics::RenderReport,
    ) -> bool {
        if !self.session.contains(window) {
            return false;
        }

        let property_only = kind.property_only();
        let refreshes_active = kind.refreshes_active();
        let present_submitted = report.present_submitted();
        let present_submitted_at = report.present_submitted_at();
        let properties = stack.base().properties();
        let property_serial = properties.serial().value();
        let diagnostics = self.diagnostics.get_mut(window);
        diagnostics.scroll.record_candidate(epoch, property_serial);
        if present_submitted {
            diagnostics.scroll.record_present_submitted(
                epoch,
                property_serial,
                present_submitted_at,
            );
        }
        diagnostics
            .render
            .record_property_attempt(properties, property_only, present_submitted);
        diagnostics
            .render
            .record_present(epoch, property_only, report);
        if diagnostics
            .render
            .frames_present_submitted
            .is_multiple_of(10)
        {
            log::debug!(
                target: "wgpu_l3::presentation_clock",
                "events={} prepared={} attempted={} present_submitted={} view_rebuilds={} layout_recomposes={} layout_reuses={} routing_layouts={} event_p95_us={} native_p95_us={} rebuild_p95_us={} reconcile_p95_us={} routing_layout_p95_us={} presentation_layout_p95_us={} scene_p95_us={} batch_p95_us={} acquire_p95_us={} encode_present_p95_us={} draw_p95_us={} interval_p95_us={} scene_items={} batches={} glyph_batches={} geometry_vertices={} geometry_upload_bytes={} geometry_buffer_creations={} draw_passes={} text_hits={} text_misses={} shape_calls={} text_hits_total={} text_misses_total={} shape_calls_total={}",
                diagnostics.pipeline.events_received,
                diagnostics.pipeline.frames_prepared,
                diagnostics.render.frames_attempted,
                diagnostics.render.frames_present_submitted,
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
                diagnostics.render.quad_vertices,
                diagnostics.render.geometry_upload_bytes,
                diagnostics.render.geometry_buffer_creations,
                diagnostics.render.draw_passes,
                diagnostics.render.inline_text_cache_hits,
                diagnostics.render.inline_text_cache_misses,
                diagnostics.render.inline_text_shape_calls,
                diagnostics.render.inline_text_cache_hits_total,
                diagnostics.render.inline_text_cache_misses_total,
                diagnostics.render.inline_text_shape_calls_total,
            );
        }

        if present_submitted {
            let activated =
                !refreshes_active && self.session.record_present_submitted(window, epoch);
            let refreshes_visible = refreshes_active
                && self
                    .session
                    .window(window)
                    .and_then(session::Window::present_submitted_epoch)
                    == Some(epoch)
                && self
                    .presented_geometry
                    .get(&window)
                    .is_some_and(|visible| visible.stack.same_structure(stack));
            let semantic_changed = self.presented_geometry.get(&window).is_none_or(|previous| {
                !std::sync::Arc::ptr_eq(previous.stack.base().commit(), stack.base().commit())
            });
            if activated && !property_only && semantic_changed {
                self.diagnostics
                    .get_mut(window)
                    .render
                    .record_semantic_activation();
            }
            if activated || refreshes_visible {
                let mut present_submitted_offsets = std::collections::HashMap::new();
                for projection in layout.scroll_projections() {
                    if let Some(offset) = stack.scroll_offset(projection.node()) {
                        present_submitted_offsets
                            .entry(projection.target().clone())
                            .and_modify(|current: &mut interaction::ScrollOffset| {
                                *current = interaction::ScrollOffset::new(
                                    current.x().max(offset.x()),
                                    current.y().max(offset.y()),
                                );
                            })
                            .or_insert(offset);
                    }
                }
                for (target, offset) in present_submitted_offsets {
                    self.session.accept_resident_scroll(window, target, offset);
                }
                self.presented_geometry.insert(
                    window,
                    super::PresentedGeometry {
                        layout: std::sync::Arc::new(layout.clone()),
                        stack: std::sync::Arc::clone(stack),
                    },
                );
                let location = self
                    .session
                    .interaction(window)
                    .and_then(|interaction| interaction.pointer().location());
                let geometry = self.presented_geometry.get(&window).cloned();
                let hit = location.and_then(|location| {
                    geometry.as_ref().and_then(|geometry| {
                        geometry.hit_test_on_surface(location.point(), location.surface())
                    })
                });
                let hovered = hit.as_ref().and_then(|hit| hit.target().cloned());
                let hover_tip_eligible = hovered.as_ref().is_some_and(|target| {
                    self.composition.get(window).is_some_and(|composition| {
                        composition.view().hover_tip_eligible(
                            composition.tree(),
                            target,
                            layout.overflow_tip_for_target(target).is_some(),
                        )
                    })
                });
                self.session
                    .project_pointer_hover(window, hovered, hover_tip_eligible);
                if let Some(deadline) = self.session.hover_tip_deadline(
                    window,
                    std::time::Duration::from_millis(
                        self.active_theme().auxiliary_panel().hover_delay_ms,
                    ),
                ) {
                    let schedules = self.animation_schedules.entry(window).or_default();
                    schedules.paint = schedules
                        .paint
                        .merge(crate::animation::Schedule::At(deadline));
                }
                let modifiers = self
                    .session
                    .interaction(window)
                    .map(|interaction| interaction.pointer().modifiers())
                    .unwrap_or_default();
                let resolved = self.resolve_press(
                    window,
                    location
                        .map(|location| location.point())
                        .unwrap_or_else(|| super::super::geometry::Point::new(0, 0)),
                    modifiers,
                    hit,
                );
                self.session.set_cursor(window, resolved.cursor());
            }
        } else if !refreshes_active {
            if property_only {
                self.session.retry_property_tick(window);
            } else {
                self.session.retry_invalidation(window, invalidation);
            }
        }
        !present_submitted
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
    pub(crate) fn presented_properties(
        &self,
        window: window::Id,
    ) -> Option<&super::super::scene::Properties> {
        self.presented_geometry
            .get(&window)
            .map(|geometry| geometry.stack.base().properties())
    }

    #[cfg(test)]
    pub(crate) fn present_submitted_epoch(
        &self,
        window: window::Id,
    ) -> Option<window::PresentationEpoch> {
        self.session
            .window(window)
            .and_then(session::Window::present_submitted_epoch)
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
