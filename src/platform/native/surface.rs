use crate::geometry::area;
use std::collections::{HashMap, hash_map::Entry};
use std::sync::Arc;
use std::time::Instant;

use crate::diagnostics;
use crate::render::{self, Canvas, Context, Renderer, canvas, context, surface};

use super::super::{NativeError, Window};
use super::window::{InitialSize, Options, Window as NativeWindow};
use super::{Native, NativeContext, PopupPrewarmState};
use crate::{interaction, scene, shell, window as app_window};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct Attempts {
    first: context::Backends,
    fallback: Option<context::Backends>,
}

impl Attempts {
    fn initialize(self) -> render::Result<Context> {
        match initialize_context(self.first, 0) {
            Ok(context) => Ok(context),
            Err(first_error) => {
                let Some(fallback) = self.fallback else {
                    return Err(first_error);
                };
                let context = initialize_context(fallback, 1)?;
                log::warn!(
                    target: "wgpu_l3::native_popup",
                    "DX12-first context initialization failed; continuing with backend set {fallback:?} and non-tenancy material realization"
                );
                Ok(context)
            }
        }
    }
}

pub(super) fn renderer_for_format<'a>(
    render_context: &render::Context,
    renderers: &'a mut HashMap<surface::Format, Renderer>,
    format: surface::Format,
) -> (&'a mut Renderer, bool) {
    match renderers.entry(format) {
        Entry::Occupied(entry) => (entry.into_mut(), true),
        Entry::Vacant(entry) => {
            log::debug!("initializing native renderer for surface format {format:?}");
            (entry.insert(Renderer::new(render_context, format)), false)
        }
    }
}

impl Native {
    pub(in crate::platform::native) fn ensure_context(&mut self) -> Result<(), NativeError> {
        if self.context.is_none() {
            log::debug!("initializing native render context");
            let explicit = context::Backends::from_env();
            self.context = Some(native_backend_attempts(explicit).initialize()?);
        }

        Ok(())
    }

    pub(in crate::platform::native) fn create_native_window(
        &mut self,
        context: &NativeContext<'_>,
        window: &Window,
    ) -> Result<NativeWindow, NativeError> {
        self.ensure_context()?;
        log::debug!(
            "opening native window {:?}: title={:?}, size={:?}",
            window.id(),
            window.title(),
            window.size()
        );

        let native_options = Options {
            title: window.title().to_owned(),
            inner_size: InitialSize::Logical(area::Logical::from_size(window.size())),
            kind: window.kind(),
            owner: None,
            popup_presentation_mode: None,
        };
        let handle = NativeWindow::open(native_options, context.event_loop())?;
        let render_context = self
            .context
            .as_ref()
            .expect("render context should exist before creating window canvas");
        let inner_size = handle.inner_size();
        let canvas = Canvas::new(
            canvas::Options {
                area: area::physical(inner_size.width, inner_size.height).clamp_min(1),
                scale_factor: handle.scale_factor() as f32,
                color: window.canvas_color(),
                composite_alpha: surface::CompositeAlphaPreference::Default,
            },
            render_context,
            handle.clone(),
        )?;
        log::debug!(
            "created native window {:?}: raw={:?}, inner={}x{}, scale={}",
            window.id(),
            handle.id(),
            inner_size.width,
            inner_size.height,
            handle.scale_factor()
        );

        Ok(NativeWindow::new(handle, canvas))
    }

    pub(in crate::platform::native) fn clear_window(
        &mut self,
        native_window: &mut NativeWindow,
    ) -> Result<(), NativeError> {
        let render_format = native_window.canvas().surface().render_format();
        let context = self
            .context
            .as_ref()
            .expect("render context should exist before clearing");
        let (renderer, _) = renderer_for_format(context, &mut self.renderers, render_format);
        renderer.clear(context, native_window.canvas_mut())?;

        Ok(())
    }

    fn requeue_failed_activation(&mut self, window: app_window::Id, prepared: shell::Presentation) {
        let successor = self
            .pending_presentations
            .remove(&window)
            .map(super::PendingPresentation::into_newest);
        let mut pending = super::PendingPresentation::new(prepared);
        if let Some(successor) = successor {
            pending.enqueue(successor);
        }
        self.pending_presentations.insert(window, pending);
    }

    pub(in crate::platform::native) fn present_native(
        &mut self,
        presentation: &shell::Presentation,
    ) -> Result<super::super::PresentResult, NativeError> {
        let window = presentation.window();
        let (_, surface_resized) = self.sync_window_surface(window)?;
        let render_format = {
            let native_window = self.windows.get(&window).ok_or_else(|| {
                log::error!("cannot present missing native window: {window:?}");
                NativeError::MissingWindow { window }
            })?;
            native_window.canvas().surface().render_format()
        };
        let context = self
            .context
            .as_ref()
            .expect("render context should exist before presenting");
        let (renderer, _) = renderer_for_format(context, &mut self.renderers, render_format);
        let active = self.active_presentations.get(&window).cloned();
        let active_matches = active
            .as_ref()
            .is_some_and(|active| active.stack().same_structure(presentation.stack()));
        let mut refreshes_active = false;
        let mut activation_from_pending = false;
        let mut residency_retirement = None;
        let mut cancelled_pending = None;
        let residency_preemption = active.as_ref().and_then(|_| {
            self.pending_presentations.get(&window).and_then(|pending| {
                if required_candidate_preempts_proactive(pending, presentation) {
                    Some(
                        super::super::ResidencyCandidateRetirement::PreemptProactive(
                            pending.preparing.epoch(),
                        ),
                    )
                } else {
                    None
                }
            })
        });
        if let Some(retirement) = residency_preemption {
            let pending = self
                .pending_presentations
                .remove(&window)
                .expect("preempted residency was just observed");
            renderer.cancel_stack_synchronization(
                pending.preparing.stack(),
                active.as_ref().map(shell::Presentation::stack),
            );
            residency_retirement = Some(retirement);
            cancelled_pending = Some(pending);
        }
        if active_matches
            && let Some(active) = active.as_ref()
            && self
                .pending_presentations
                .get(&window)
                .is_some_and(|pending| {
                    pending_scroll_intent_is_obsolete(active, pending, presentation)
                })
        {
            let pending = self
                .pending_presentations
                .remove(&window)
                .expect("obsolete pending residency was just observed");
            let selected_epoch = pending.preparing.epoch();
            renderer.cancel_stack_synchronization(pending.preparing.stack(), Some(active.stack()));
            residency_retirement = Some(
                super::super::ResidencyCandidateRetirement::CancelPipeline(selected_epoch),
            );
            cancelled_pending = Some(pending);
        }
        let actual = if active.is_none() || active_matches {
            refreshes_active = incoming_refreshes_active(
                active_matches,
                self.pending_presentations.contains_key(&window),
                active.as_ref().is_some_and(|active| {
                    std::sync::Arc::ptr_eq(active.commit(), presentation.commit())
                }),
            );
            if refreshes_active {
                project_onto_active(
                    active
                        .as_ref()
                        .expect("an active-compatible refresh requires an active presentation"),
                    presentation,
                )
            } else {
                presentation.clone()
            }
        } else {
            match self.pending_presentations.entry(window) {
                Entry::Occupied(mut entry) => entry.get_mut().enqueue(presentation.clone()),
                Entry::Vacant(entry) => {
                    entry.insert(super::PendingPresentation::new(presentation.clone()));
                }
            }
            let canvas = self.windows.get(&window).ok_or_else(|| {
                log::error!("cannot prepare missing native window: {window:?}");
                NativeError::MissingWindow { window }
            })?;
            let preparation = if surface_resized {
                PreparationWindow {
                    budget: std::time::Duration::MAX,
                    deadline: std::time::Duration::MAX,
                }
            } else {
                preparation_window(canvas.display_refresh_millihertz())
            };
            let preparation_started = Instant::now();
            loop {
                let pending = self
                    .pending_presentations
                    .get(&window)
                    .expect("pending presentation was just enqueued");
                let preparing = pending.preparing.clone();
                let newest = pending.newest().clone();
                let remaining = preparation
                    .budget
                    .saturating_sub(preparation_started.elapsed());
                if remaining.is_zero() {
                    refreshes_active = true;
                    break project_onto_active(
                        active
                            .as_ref()
                            .expect("pending realization requires a complete active presentation"),
                        &newest,
                    );
                }
                let readiness = renderer.synchronize_stack(
                    context,
                    canvas.canvas(),
                    preparing.stack(),
                    remaining,
                    preparation.deadline,
                )?;
                match readiness {
                    render::CommitReadiness::Ready => {
                        let pending = self
                            .pending_presentations
                            .remove(&window)
                            .expect("ready preparation must still be pending");
                        let resolution = resolve_completed_presentation(
                            active.as_ref().expect(
                                "pending realization requires a complete active presentation",
                            ),
                            pending.complete(),
                        );
                        if let Some(successor) = resolution.successor {
                            self.pending_presentations
                                .insert(window, super::PendingPresentation::new(successor));
                        }
                        match resolution.outcome {
                            PendingCompletionOutcome::ActivatePrepared => {
                                activation_from_pending = true;
                            }
                            PendingCompletionOutcome::Superseded(error) => {
                                log::debug!(
                                    "skipping obsolete prepared residency while the newest scroll state continues preparation: {error}"
                                );
                                residency_retirement = resolution.superseded_candidate_epoch.map(
                                    super::super::ResidencyCandidateRetirement::SupersedeFront,
                                );
                                refreshes_active = true;
                            }
                        }
                        break resolution.actual;
                    }
                    _ => {
                        refreshes_active = true;
                        break project_onto_active(
                            active.as_ref().expect(
                                "pending realization requires a complete active presentation",
                            ),
                            &newest,
                        );
                    }
                }
            }
        };
        if refreshes_active
            && residency_retirement.is_none()
            && active
                .as_ref()
                .is_some_and(|active| active.stack().same_presented_property_state(actual.stack()))
        {
            let refresh_millihertz = self
                .windows
                .get(&window)
                .and_then(|window| window.display_refresh_millihertz());
            let now = Instant::now();
            let retry_at = now
                .checked_add(preparation_window(refresh_millihertz).deadline)
                .unwrap_or(now);
            return Ok(super::super::PresentResult::Deferred { window, retry_at });
        }
        let candidate_after_present = self
            .pending_presentations
            .get(&window)
            .map(|pending| pending.preparing.clone());
        let native_window = self.windows.get_mut(&window).ok_or_else(|| {
            log::error!("cannot present missing native window: {window:?}");
            NativeError::MissingWindow { window }
        })?;
        let environment = render::RendererEnvironment::new(
            context,
            native_window.canvas(),
            native_window.display_name(),
            native_window.display_refresh_millihertz(),
        );
        let draw_started = Instant::now();
        let report = renderer.draw_stack(context, native_window.canvas_mut(), actual.stack());
        let report = match report {
            Ok(report) => report,
            Err(error) => {
                if let Some(pending) = cancelled_pending.take() {
                    let fallback = activation_from_pending.then(|| actual.clone());
                    restore_retired_pending(
                        &mut self.pending_presentations,
                        window,
                        pending,
                        fallback,
                    );
                } else if activation_from_pending {
                    self.requeue_failed_activation(window, actual);
                }
                return Err(error.into());
            }
        };
        let draw = draw_started.elapsed();
        let acquire_wait = report
            .present_timing
            .map(surface::PresentTiming::acquire_wait)
            .unwrap_or_default();
        let encode_submit_present = report
            .present_timing
            .map(surface::PresentTiming::encode_submit_present)
            .unwrap_or_default();
        let present_submitted_at = report
            .present_timing
            .map(surface::PresentTiming::surface_present_called_at)
            .unwrap_or_else(Instant::now);
        let frame_timeline = render::FrameTimeline::new(
            report.acquire_started_at,
            report.acquire_finished_at,
            report
                .present_timing
                .map(surface::PresentTiming::queue_submitted_at),
            report
                .present_timing
                .map(surface::PresentTiming::surface_present_called_at),
        );
        let group_composites = report.stats.group_composites;
        let filter_layer_pool_entries = report.stats.filter_layer_pool_entries;
        let filter_scratch_pool_entries = report.stats.filter_scratch_pool_entries;
        if acquire_wait.as_millis() >= 8 {
            log::debug!(
                "surface acquire wait for window {:?}: {}us",
                window,
                acquire_wait.as_micros()
            );
        }

        let present_submitted = report.present_timing.is_some();
        if !present_submitted && let Some(pending) = cancelled_pending.take() {
            let fallback = activation_from_pending.then(|| actual.clone());
            restore_retired_pending(&mut self.pending_presentations, window, pending, fallback);
            activation_from_pending = false;
            residency_retirement = None;
        }
        if present_submitted && let Some(candidate) = candidate_after_present {
            let preparation = preparation_window(native_window.display_refresh_millihertz());
            let remaining = preparation
                .deadline
                .saturating_sub(draw_started.elapsed())
                .min(preparation.budget);
            if let Err(error) = renderer.advance_stack_after_present(
                context,
                native_window.canvas(),
                candidate.stack(),
                remaining,
                preparation.deadline,
            ) {
                log::warn!(
                    "candidate realization failed after a successful active present; retaining active state: {error}"
                );
                renderer.cancel_stack_synchronization(
                    candidate.stack(),
                    active.as_ref().map(shell::Presentation::stack),
                );
            }
        }
        if present_submitted
            && context.windows_popup_composition_supported()
            && !self.popup_prewarm.contains_key(&window)
        {
            self.popup_prewarm.insert(window, PopupPrewarmState::Armed);
        }

        if activation_from_pending {
            if !present_submitted {
                self.requeue_failed_activation(window, actual.clone());
            }
        }

        if present_submitted {
            self.active_presentations.insert(window, actual.clone());
        }

        let report = diagnostics::RenderReport::new(acquire_wait, draw, present_submitted_at)
            .with_present_submitted(present_submitted)
            .with_frame_timeline(frame_timeline)
            .with_acquire_outcome(report.acquire_outcome)
            .with_pipeline_timings(report.batch_prepare, encode_submit_present)
            .with_draw_stats(report.stats)
            .with_environment(environment)
            .with_group_composites(group_composites)
            .with_filter_pool_entries(filter_layer_pool_entries, filter_scratch_pool_entries);
        let presented = super::super::Presented::new(actual, report);
        let presented = match residency_retirement {
            Some(retirement) => presented.with_residency_retirement(retirement),
            None => presented,
        };
        Ok(if refreshes_active {
            super::super::PresentResult::ActiveRefreshedAndDeferred(presented)
        } else if self.pending_presentations.contains_key(&window) {
            super::super::PresentResult::PresentedAndDeferred(presented)
        } else {
            presented.into()
        })
    }

    pub(in crate::platform::native) fn resume_native_presentation(
        &mut self,
        window: app_window::Id,
    ) -> Result<Option<super::super::PresentResult>, NativeError> {
        let presentation = self
            .pending_presentations
            .get(&window)
            .map(|pending| pending.preparing.clone());
        let Some(presentation) = presentation else {
            return Ok(None);
        };
        let is_current = self
            .pending_presentations
            .get(&window)
            .is_some_and(|current| Arc::ptr_eq(current.preparing.stack(), presentation.stack()));

        is_current
            .then(|| self.present_native(&presentation))
            .transpose()
    }

    pub(in crate::platform::native) fn sync_window_surface(
        &mut self,
        window: app_window::Id,
    ) -> Result<(surface::Format, bool), NativeError> {
        self.ensure_context()?;
        let native_window = self.windows.get_mut(&window).ok_or_else(|| {
            log::error!("cannot sync surface for missing native window: {window:?}");
            NativeError::MissingWindow { window }
        })?;
        let area = native_window.inner_area().clamp_min(1);
        let scale_factor = native_window.scale_factor() as f32;
        let needs_resize = native_window.canvas().physical_area() != area
            || (native_window.canvas().scale_factor() - scale_factor).abs() > f32::EPSILON;

        if needs_resize {
            log::debug!(
                "syncing native surface {:?}: area={}x{}, scale={}",
                window,
                area.width(),
                area.height(),
                scale_factor
            );
            let context = self
                .context
                .as_ref()
                .expect("render context should exist before resizing");
            native_window.resize(context, area, scale_factor);
        }

        Ok((native_window.canvas().surface().format(), needs_resize))
    }
}

fn incoming_refreshes_active(
    active_matches: bool,
    has_pending_residency: bool,
    same_semantic_commit: bool,
) -> bool {
    active_matches && has_pending_residency && same_semantic_commit
}

fn restore_retired_pending(
    pending_presentations: &mut HashMap<
        app_window::Id,
        super::PendingPresentation<shell::Presentation>,
    >,
    window: app_window::Id,
    mut retired: super::PendingPresentation<shell::Presentation>,
    fallback_successor: Option<shell::Presentation>,
) {
    let successor = pending_presentations
        .remove(&window)
        .map(super::PendingPresentation::into_newest)
        .or(fallback_successor);
    if let Some(successor) = successor {
        retired.enqueue(successor);
    }
    pending_presentations.insert(window, retired);
}

fn required_candidate_preempts_proactive(
    pending: &super::PendingPresentation<shell::Presentation>,
    incoming: &shell::Presentation,
) -> bool {
    pending.preparing.residency_urgency() == Some(scene::ResidencyUrgency::Proactive)
        && incoming.residency_urgency() == Some(scene::ResidencyUrgency::Required)
        && incoming.epoch() > pending.preparing.epoch()
}

fn pending_scroll_intent_is_obsolete(
    active: &shell::Presentation,
    pending: &super::PendingPresentation<shell::Presentation>,
    incoming: &shell::Presentation,
) -> bool {
    let newest = pending.newest();
    if incoming.epoch() <= newest.epoch()
        || !active.stack().same_structure(incoming.stack())
        || active.stack().same_structure(pending.preparing.stack())
        || !std::sync::Arc::ptr_eq(active.commit(), pending.preparing.commit())
        || !std::sync::Arc::ptr_eq(active.commit(), newest.commit())
    {
        return false;
    }

    active.stack().base().residencies().iter().any(|residency| {
        let node = residency.scroll();
        let Some(active_offset) = active.properties().scroll_offset(node) else {
            return false;
        };
        let Some(pending_offset) = newest.properties().scroll_offset(node) else {
            return false;
        };
        let Some(incoming_offset) = incoming.properties().scroll_offset(node) else {
            return false;
        };
        scroll_axis_reversed(
            active_offset,
            pending_offset,
            incoming_offset,
            interaction::ScrollbarAxis::Horizontal,
        ) || scroll_axis_reversed(
            active_offset,
            pending_offset,
            incoming_offset,
            interaction::ScrollbarAxis::Vertical,
        )
    })
}

fn scroll_axis_reversed(
    active: interaction::ScrollOffset,
    pending: interaction::ScrollOffset,
    incoming: interaction::ScrollOffset,
    axis: interaction::ScrollbarAxis,
) -> bool {
    let pending_direction = pending.axis_cmp(active, axis);
    if pending_direction.is_eq() {
        return false;
    }
    incoming.same_axis(active, axis) || pending_direction != incoming.axis_cmp(active, axis)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct PreparationWindow {
    budget: std::time::Duration,
    deadline: std::time::Duration,
}

fn preparation_window(refresh_millihertz: Option<u32>) -> PreparationWindow {
    const MINIMUM: std::time::Duration = std::time::Duration::from_micros(750);
    const DEFAULT: std::time::Duration = std::time::Duration::from_millis(4);
    const DEFAULT_DEADLINE: std::time::Duration = std::time::Duration::from_nanos(16_666_667);
    const MAXIMUM: std::time::Duration = std::time::Duration::from_millis(8);
    const DRAW_RESERVE: std::time::Duration = std::time::Duration::from_millis(2);

    let Some(refresh_millihertz) = refresh_millihertz.filter(|refresh| *refresh > 0) else {
        return PreparationWindow {
            budget: DEFAULT,
            deadline: DEFAULT_DEADLINE,
        };
    };
    let period_nanos = 1_000_000_000_000_u128 / u128::from(refresh_millihertz);
    let period = std::time::Duration::from_nanos(period_nanos.min(u128::from(u64::MAX)) as u64);
    PreparationWindow {
        budget: period.saturating_sub(DRAW_RESERVE).clamp(MINIMUM, MAXIMUM),
        deadline: period,
    }
}

fn project_onto_active(
    active: &shell::Presentation,
    newest: &shell::Presentation,
) -> shell::Presentation {
    match active.stack().project_base_properties(newest.properties()) {
        Some((properties, true)) => active.with_active_properties(properties, newest),
        Some((_, false)) | None => active.with_spatial_supplements(newest),
    }
}

enum PendingCompletionOutcome {
    ActivatePrepared,
    Superseded(String),
}

struct PendingCompletionResolution {
    actual: shell::Presentation,
    successor: Option<shell::Presentation>,
    outcome: PendingCompletionOutcome,
    superseded_candidate_epoch: Option<app_window::PresentationEpoch>,
}

fn resolve_completed_presentation(
    active: &shell::Presentation,
    completed: super::PendingCompletion<shell::Presentation>,
) -> PendingCompletionResolution {
    let prepared_epoch = completed.prepared.epoch();
    let Some(successor) = completed.successor else {
        return match active.properties().rebase_scroll_onto_for_activation(
            completed.prepared.drawable_commit(),
            completed.prepared.properties(),
        ) {
            Ok(properties) => PendingCompletionResolution {
                actual: completed
                    .prepared
                    .with_activation_properties(properties, active),
                successor: None,
                outcome: PendingCompletionOutcome::ActivatePrepared,
                superseded_candidate_epoch: None,
            },
            Err(_) => PendingCompletionResolution {
                actual: completed.prepared.with_spatial_supplements(active),
                successor: None,
                outcome: PendingCompletionOutcome::ActivatePrepared,
                superseded_candidate_epoch: None,
            },
        };
    };
    match successor.properties().rebase_onto_for_activation(
        completed.prepared.drawable_commit(),
        completed.prepared.properties(),
    ) {
        Ok(properties) => PendingCompletionResolution {
            actual: completed
                .prepared
                .with_activation_properties(properties, &successor),
            successor: Some(successor),
            outcome: PendingCompletionOutcome::ActivatePrepared,
            superseded_candidate_epoch: None,
        },
        Err(error) => {
            if let Some(properties) = completed
                .prepared
                .stack()
                .project_base_properties_toward(active.stack(), successor.properties())
            {
                PendingCompletionResolution {
                    actual: completed
                        .prepared
                        .with_activation_properties(properties, &successor),
                    successor: Some(successor),
                    outcome: PendingCompletionOutcome::ActivatePrepared,
                    superseded_candidate_epoch: None,
                }
            } else {
                PendingCompletionResolution {
                    actual: project_onto_active(active, &successor),
                    successor: Some(successor),
                    outcome: PendingCompletionOutcome::Superseded(error.to_string()),
                    superseded_candidate_epoch: Some(prepared_epoch),
                }
            }
        }
    }
}

fn initialize_context(backends: context::Backends, index: usize) -> render::Result<Context> {
    pollster::block_on(Context::new(context::Options::native(backends))).inspect_err(|error| {
        log::warn!(
            target: "wgpu_l3::native_popup",
            "native graphics attempt {index} with {backends:?} failed: {error}"
        );
    })
}

fn native_backend_attempts(explicit: Option<context::Backends>) -> Attempts {
    if let Some(explicit) = explicit {
        return Attempts {
            first: explicit,
            fallback: None,
        };
    }

    #[cfg(target_os = "windows")]
    {
        Attempts {
            first: context::Backends::dx12(),
            fallback: Some(context::Backends::all()),
        }
    }
    #[cfg(not(target_os = "windows"))]
    {
        Attempts {
            first: context::Backends::all(),
            fallback: None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Clone)]
    struct NativeScrollState;

    impl crate::state::State for NativeScrollState {}

    #[derive(Clone)]
    struct NativeScrollRows;

    impl crate::table::Provider for NativeScrollRows {
        fn len(&self) -> usize {
            1_000_000
        }

        fn key(&self, index: usize) -> crate::virtual_list::Key {
            crate::virtual_list::Key::new(index as u64)
        }

        fn index_of(&self, key: crate::virtual_list::Key) -> Option<usize> {
            let index = key.value() as usize;
            (index < self.len()).then_some(index)
        }

        fn cell(&self, row: usize, cell: crate::table::Cell) -> crate::view::Node {
            crate::view::Node::text_area_state(
                crate::view::TextArea::new(format!("{} {row}", cell.column().as_str()))
                    .with_focus(crate::session::Focus::table_cell(cell))
                    .read_only(),
            )
        }
    }

    fn native_scroll_shell() -> crate::shell::Shell<NativeScrollState> {
        let rows = NativeScrollRows;
        crate::shell::Shell::new(
            crate::runtime::Runtime::new(NativeScrollState)
                .started(|cx| {
                    cx.open_window(crate::window::Options::new("Native scroll policy"));
                })
                .view(move |_, _| {
                    crate::widget::view_node(
                        crate::Table::new(
                            "native.scroll.table",
                            24,
                            [
                                crate::table::Column::new(
                                    "name",
                                    "Name",
                                    crate::view::Dimension::fixed(120),
                                ),
                                crate::table::Column::new(
                                    "detail",
                                    "Detail",
                                    crate::view::Dimension::fixed(180),
                                ),
                                crate::table::Column::new(
                                    "status",
                                    "Status",
                                    crate::view::Dimension::fixed(100),
                                ),
                            ],
                            rows.clone(),
                        )
                        .height(crate::view::Dimension::grow()),
                    )
                }),
        )
    }

    #[test]
    fn active_projection_advances_to_the_active_residency_boundary() {
        let mut shell = native_scroll_shell();
        shell.start();
        let window = shell.runtime().session().windows()[0].id();
        let size = crate::geometry::Size::new(360, 180);
        assert!(shell.set_window_size(window, size));
        let initial = shell.drain();
        let active = initial
            .presentations()
            .last()
            .cloned()
            .expect("control gallery should produce an active presentation");
        shell.runtime_mut().finish_render_report(
            active.window(),
            active.epoch(),
            active.invalidation(),
            active.layout(),
            active.stack(),
            active.property_only(),
            crate::diagnostics::RenderReport::new(
                std::time::Duration::ZERO,
                std::time::Duration::ZERO,
                Instant::now(),
            ),
        );
        let cell = active
            .layout()
            .frames()
            .iter()
            .find(|frame| {
                frame.table_cell().is_some_and(|cell| {
                    cell.row() == crate::virtual_list::Key::new(1)
                        && cell.column() == crate::interaction::Id::new("detail")
                })
            })
            .expect("control gallery should materialize a table cell");
        let point = crate::geometry::Point::new(cell.rect().x() + 1, cell.rect().y() + 1);
        let target = active
            .layout()
            .scroll_target_at(point, crate::interaction::ScrollDelta::vertical(1))
            .expect("table cell should route vertical scrolling");
        let projection = active
            .layout()
            .scroll_projections()
            .iter()
            .find(|projection| {
                projection.target() == &target && projection.viewport().max_scroll().y() > 0
            })
            .expect("table should have an active vertical residency");
        let scroll = projection.node();
        let (_, maximum) = projection
            .accepted_offsets()
            .expect("active table residency should prove a resident interval");
        let active_offset = active
            .properties()
            .scroll_offset(scroll)
            .expect("active table should have a scroll property");
        let delta = maximum.y().saturating_add(1);

        shell
            .runtime_mut()
            .scroll_at(
                window,
                size,
                point,
                crate::interaction::ScrollDelta::vertical(delta),
            )
            .expect("scroll beyond active residency should request replenishment");
        let pending = shell.drain();
        let newest = pending
            .presentations()
            .last()
            .expect("replenishment should produce a pending presentation");
        assert!(
            !active.stack().same_structure(newest.stack()),
            "fixture must require a new drawable residency"
        );
        assert!(
            newest
                .properties()
                .scroll_offset(scroll)
                .is_some_and(|offset| offset.y() > maximum.y()),
            "pending presentation must carry the desired offset beyond active coverage"
        );

        let projected = project_onto_active(&active, newest);

        assert_eq!(
            projected.epoch(),
            active.epoch(),
            "active progress must not consume the epoch owned by pending residency realization"
        );
        assert_eq!(
            projected.properties().scroll_offset(scroll),
            Some(maximum),
            "native active refresh must advance to the furthest complete offset instead of snapping back"
        );
        assert_ne!(
            maximum, active_offset,
            "fixture must prove forward progress beyond the previously active offset"
        );
        assert!(
            !active
                .stack()
                .same_presented_property_state(projected.stack()),
            "the first active projection must submit its real boundary progress"
        );
        let stalled = project_onto_active(&projected, newest);
        assert!(
            projected
                .stack()
                .same_presented_property_state(stalled.stack()),
            "continuation after reaching the active boundary must not submit duplicate active frames"
        );
    }

    #[test]
    fn completed_residency_advances_to_its_boundary_toward_newer_scroll_state() {
        let mut shell = native_scroll_shell();
        shell.start();
        let window = shell.runtime().session().windows()[0].id();
        let size = crate::geometry::Size::new(360, 180);
        assert!(shell.set_window_size(window, size));
        let initial = shell.drain();
        let active = initial
            .presentations()
            .last()
            .cloned()
            .expect("control gallery should produce an active presentation");
        shell.runtime_mut().finish_render_report(
            active.window(),
            active.epoch(),
            active.invalidation(),
            active.layout(),
            active.stack(),
            active.property_only(),
            crate::diagnostics::RenderReport::new(
                std::time::Duration::ZERO,
                std::time::Duration::ZERO,
                Instant::now(),
            ),
        );
        let cell = active
            .layout()
            .frames()
            .iter()
            .find(|frame| {
                frame.table_cell().is_some_and(|cell| {
                    cell.row() == crate::virtual_list::Key::new(1)
                        && cell.column() == crate::interaction::Id::new("detail")
                })
            })
            .expect("control gallery should materialize a table cell");
        let point = crate::geometry::Point::new(cell.rect().x() + 1, cell.rect().y() + 1);
        let target = active
            .layout()
            .scroll_target_at(point, crate::interaction::ScrollDelta::vertical(1))
            .expect("table cell should route vertical scrolling");
        let projection = active
            .layout()
            .scroll_projections()
            .iter()
            .find(|projection| {
                projection.target() == &target && projection.viewport().max_scroll().y() > 0
            })
            .expect("table should have an active vertical residency");
        let scroll = projection.node();
        let (_, active_maximum) = projection
            .accepted_offsets()
            .expect("active table residency should prove a resident interval");
        let first_requested =
            crate::interaction::ScrollOffset::new(0, active_maximum.y().saturating_add(1));

        shell
            .runtime_mut()
            .handle_input(
                window,
                crate::Input::scroll_to(target.clone(), first_requested),
            )
            .expect("first crossing request should be handled");
        let first_work = shell.drain();
        let prepared = first_work
            .presentations()
            .last()
            .cloned()
            .expect("first crossing should produce a prepared residency");
        let prepared_projection = prepared
            .layout()
            .scroll_projections()
            .iter()
            .find(|projection| projection.node() == scroll)
            .expect("prepared residency should retain the table scroll node");
        let (_, prepared_maximum) = prepared_projection
            .accepted_offsets()
            .expect("prepared residency should prove its accepted interval");
        let latest_requested =
            crate::interaction::ScrollOffset::new(0, prepared_maximum.y().saturating_add(1));

        shell
            .runtime_mut()
            .handle_input(
                window,
                crate::Input::scroll_to(target.clone(), latest_requested),
            )
            .expect("newer crossing request should coalesce behind the prepared residency");
        let coalesced_work = shell.drain();
        assert!(
            coalesced_work.presentations().is_empty(),
            "same-urgency intent must coalesce before constructing another required candidate"
        );
        assert!(
            shell.runtime_mut().finish_render_report(
                prepared.window(),
                prepared.epoch(),
                prepared.invalidation(),
                prepared.layout(),
                prepared.stack(),
                prepared.property_only(),
                crate::diagnostics::RenderReport::new(
                    std::time::Duration::ZERO,
                    std::time::Duration::ZERO,
                    Instant::now(),
                ),
            ),
            "retiring the selected front must request one final latest-value follow-up"
        );
        let successor_work = shell.drain();
        let successor =
            successor_work.presentations().last().cloned().expect(
                "front retirement must construct one candidate at the final coalesced intent",
            );
        assert_eq!(
            successor.properties().scroll_offset(scroll),
            Some(latest_requested)
        );
        assert!(
            successor
                .properties()
                .rebase_onto_for_activation(prepared.drawable_commit(), prepared.properties(),)
                .is_err(),
            "fixture must put the latest offset outside the obsolete prepared residency"
        );
        assert_eq!(
            prepared.residency_urgency(),
            Some(scene::ResidencyUrgency::Required)
        );
        assert_eq!(
            successor.residency_urgency(),
            Some(scene::ResidencyUrgency::Required)
        );
        let obsolete_offset = prepared
            .properties()
            .scroll_offset(scroll)
            .expect("prepared residency should carry its selected offset");
        assert!(
            prepared
                .stack()
                .project_base_properties_toward(active.stack(), successor.properties())
                .is_some(),
            "prepared residency should make monotonic progress toward the latest intent"
        );
        let resolution = resolve_completed_presentation(
            &active,
            super::super::PendingCompletion {
                prepared,
                successor: Some(successor),
            },
        );

        assert!(matches!(
            &resolution.outcome,
            PendingCompletionOutcome::ActivatePrepared
        ));
        assert_eq!(
            resolution.superseded_candidate_epoch, None,
            "a prepared residency that makes monotonic progress must not be retired"
        );
        assert_eq!(
            resolution.actual.properties().scroll_offset(scroll),
            Some(prepared_maximum),
            "the completed residency must present its furthest legal offset toward the latest intent"
        );
        assert_ne!(
            resolution.actual.properties().scroll_offset(scroll),
            Some(obsolete_offset),
            "the stale construction-time offset must not become an avoidable presentation step"
        );
        assert_eq!(
            resolution
                .successor
                .as_ref()
                .and_then(|successor| successor.properties().scroll_offset(scroll)),
            Some(latest_requested),
            "only the newest residency may remain pending"
        );
    }

    #[test]
    fn newer_required_offset_inside_selected_residency_coalesces_before_construction() {
        let mut shell = native_scroll_shell();
        shell.start();
        let window = shell.runtime().session().windows()[0].id();
        let size = crate::geometry::Size::new(360, 180);
        assert!(shell.set_window_size(window, size));
        let initial = shell.drain();
        let active = initial
            .presentations()
            .last()
            .cloned()
            .expect("table fixture should produce an active presentation");
        shell.runtime_mut().finish_render_report(
            active.window(),
            active.epoch(),
            active.invalidation(),
            active.layout(),
            active.stack(),
            active.property_only(),
            crate::diagnostics::RenderReport::new(
                std::time::Duration::ZERO,
                std::time::Duration::ZERO,
                Instant::now(),
            ),
        );
        let cell = active
            .layout()
            .frames()
            .iter()
            .find(|frame| {
                frame.table_cell().is_some_and(|cell| {
                    cell.row() == crate::virtual_list::Key::new(1)
                        && cell.column() == crate::interaction::Id::new("detail")
                })
            })
            .expect("table fixture should materialize a detail cell");
        let point = crate::geometry::Point::new(cell.rect().x() + 1, cell.rect().y() + 1);
        let target = active
            .layout()
            .scroll_target_at(point, crate::interaction::ScrollDelta::vertical(1))
            .expect("table cell should route vertical scrolling");
        let projection = active
            .layout()
            .scroll_projections()
            .iter()
            .find(|projection| {
                projection.target() == &target && projection.viewport().max_scroll().y() > 0
            })
            .expect("table should have an active vertical residency");
        let scroll = projection.node();
        let (_, active_maximum) = projection
            .accepted_offsets()
            .expect("active table residency should prove a resident interval");
        let first_requested =
            crate::interaction::ScrollOffset::new(0, active_maximum.y().saturating_add(1));

        shell
            .runtime_mut()
            .handle_input(
                window,
                crate::Input::scroll_to(target.clone(), first_requested),
            )
            .expect("first crossing request should be handled");
        let first_work = shell.drain();
        let prepared = first_work
            .presentations()
            .last()
            .cloned()
            .expect("first crossing should produce a selected residency");
        let prepared_projection = prepared
            .layout()
            .scroll_projections()
            .iter()
            .find(|projection| projection.node() == scroll)
            .expect("selected residency should retain the vertical table scroll node");
        let (_, prepared_maximum) = prepared_projection
            .accepted_offsets()
            .expect("selected residency should prove its accepted interval");
        let accepted_y = first_requested.y().saturating_add(1);
        assert!(
            accepted_y <= prepared_maximum.y(),
            "fixture needs one newer offset inside the selected residency"
        );
        let accepted = crate::interaction::ScrollOffset::new(0, accepted_y);

        shell
            .runtime_mut()
            .handle_input(window, crate::Input::scroll_to(target.clone(), accepted))
            .expect("newer in-residency request should be handled");
        let successor_work = shell.drain();
        let previous = prepared
            .properties()
            .scroll_offset(scroll)
            .expect("selected residency should carry its authored offset");
        assert!(
            prepared
                .layout()
                .scroll_property_acceptance(&target, previous, accepted)
                .is_some(),
            "canonical typed-axis acceptance must accept the newer vertical offset"
        );
        assert!(
            successor_work.presentations().is_empty(),
            "a same-urgency request must remain coalesced while the selected required front is in flight"
        );
    }

    #[test]
    fn completed_residency_preserves_newer_active_scroll_without_a_successor() {
        let mut shell = native_scroll_shell();
        shell.start();
        let window = shell.runtime().session().windows()[0].id();
        let size = crate::geometry::Size::new(360, 180);
        assert!(shell.set_window_size(window, size));
        let initial = shell.drain();
        let active = initial
            .presentations()
            .last()
            .cloned()
            .expect("control gallery should produce an active presentation");
        shell.runtime_mut().finish_render_report(
            active.window(),
            active.epoch(),
            active.invalidation(),
            active.layout(),
            active.stack(),
            active.property_only(),
            crate::diagnostics::RenderReport::new(
                std::time::Duration::ZERO,
                std::time::Duration::ZERO,
                Instant::now(),
            ),
        );
        let cell = active
            .layout()
            .frames()
            .iter()
            .find(|frame| {
                frame.table_cell().is_some_and(|cell| {
                    cell.row() == crate::virtual_list::Key::new(1)
                        && cell.column() == crate::interaction::Id::new("detail")
                })
            })
            .expect("control gallery should materialize a table cell");
        let point = crate::geometry::Point::new(cell.rect().x() + 1, cell.rect().y() + 1);
        let target = active
            .layout()
            .scroll_target_at(point, crate::interaction::ScrollDelta::vertical(1))
            .expect("table cell should route vertical scrolling");
        let projection = active
            .layout()
            .scroll_projections()
            .iter()
            .find(|projection| {
                projection.target() == &target && projection.viewport().max_scroll().y() > 0
            })
            .expect("table should have an active vertical residency");
        let scroll = projection.node();
        let (_, active_maximum) = projection
            .accepted_offsets()
            .expect("active table residency should prove a resident interval");
        let requested =
            crate::interaction::ScrollOffset::new(0, active_maximum.y().saturating_add(1));

        shell
            .runtime_mut()
            .handle_input(window, crate::Input::scroll_to(target, requested))
            .expect("crossing request should be handled");
        let crossing = shell.drain();
        let prepared = crossing
            .presentations()
            .last()
            .cloned()
            .expect("crossing should prepare a new residency");
        let latest_active = project_onto_active(&active, &prepared);
        let latest_offset = latest_active
            .properties()
            .scroll_offset(scroll)
            .expect("active projection should retain the scroll property");
        let prepared_offset = prepared
            .properties()
            .scroll_offset(scroll)
            .expect("prepared residency should retain the scroll property");
        assert_ne!(latest_offset, prepared_offset);
        let rebase = latest_active
            .properties()
            .rebase_scroll_onto_for_activation(prepared.drawable_commit(), prepared.properties());
        assert!(
            rebase.is_ok(),
            "latest active scroll must be legal in the completed residency: {rebase:?}"
        );

        let resolution = resolve_completed_presentation(
            &latest_active,
            super::super::PendingCompletion {
                prepared,
                successor: None,
            },
        );

        assert!(matches!(
            resolution.outcome,
            PendingCompletionOutcome::ActivatePrepared
        ));
        assert_eq!(
            resolution.actual.properties().scroll_offset(scroll),
            Some(latest_offset),
            "finishing preparation must not restore its stale construction-time offset"
        );
    }

    #[test]
    fn completed_large_jump_without_a_successor_activates_the_selected_candidate() {
        let mut shell = native_scroll_shell();
        shell.start();
        let window = shell.runtime().session().windows()[0].id();
        let size = crate::geometry::Size::new(360, 180);
        assert!(shell.set_window_size(window, size));
        let initial = shell.drain();
        let active = initial
            .presentations()
            .last()
            .cloned()
            .expect("table fixture should produce an active presentation");
        shell.runtime_mut().finish_render_report(
            active.window(),
            active.epoch(),
            active.invalidation(),
            active.layout(),
            active.stack(),
            active.property_only(),
            crate::diagnostics::RenderReport::new(
                std::time::Duration::ZERO,
                std::time::Duration::ZERO,
                Instant::now(),
            ),
        );
        let cell = active
            .layout()
            .frames()
            .iter()
            .find(|frame| {
                frame.table_cell().is_some_and(|cell| {
                    cell.row() == crate::virtual_list::Key::new(1)
                        && cell.column() == crate::interaction::Id::new("detail")
                })
            })
            .expect("table fixture should materialize a detail cell");
        let point = crate::geometry::Point::new(cell.rect().x() + 1, cell.rect().y() + 1);
        let target = active
            .layout()
            .scroll_target_at(point, crate::interaction::ScrollDelta::vertical(1))
            .expect("table cell should route vertical scrolling");
        let projection = active
            .layout()
            .scroll_projections()
            .iter()
            .find(|projection| {
                projection.target() == &target && projection.viewport().max_scroll().y() > 0
            })
            .expect("table should have an active vertical residency");
        let scroll = projection.node();
        let (_, active_maximum) = projection
            .accepted_offsets()
            .expect("active table residency should prove a resident interval");
        let requested_y = active_maximum
            .y()
            .saturating_add(10_000)
            .min(projection.viewport().max_scroll().y());
        assert!(
            requested_y > active_maximum.y().saturating_add(1_000),
            "fixture must jump beyond the prepared trailing guard"
        );
        let requested = crate::interaction::ScrollOffset::new(0, requested_y);

        shell
            .runtime_mut()
            .handle_input(window, crate::Input::scroll_to(target.clone(), requested))
            .expect("large crossing request should be handled");
        let crossing = shell.drain();
        let prepared = crossing
            .presentations()
            .last()
            .cloned()
            .expect("large crossing should select one residency candidate");
        let latest_active = project_onto_active(&active, &prepared);
        assert_eq!(
            latest_active.properties().scroll_offset(scroll),
            Some(active_maximum),
            "active projection must stop at the last complete residency boundary"
        );
        assert!(
            latest_active
                .properties()
                .rebase_scroll_onto_for_activation(
                    prepared.drawable_commit(),
                    prepared.properties(),
                )
                .is_err(),
            "the large-jump active boundary must lie outside the completed candidate residency"
        );

        let resolution = resolve_completed_presentation(
            &latest_active,
            super::super::PendingCompletion {
                prepared,
                successor: None,
            },
        );
        assert!(matches!(
            resolution.outcome,
            PendingCompletionOutcome::ActivatePrepared
        ));
        assert_eq!(resolution.superseded_candidate_epoch, None);
        assert_eq!(
            resolution.actual.properties().scroll_offset(scroll),
            Some(requested),
            "a selected large-jump candidate must present its authored target when no newer intent exists"
        );
        assert!(resolution.successor.is_none());
        let diagnostics = shell
            .runtime()
            .diagnostics(window)
            .expect("large-jump diagnostics");
        assert_eq!(diagnostics.scroll.scroll_residency_candidates_superseded, 0);
        assert_eq!(diagnostics.scroll.scroll_residency_follow_ups, 0);
    }

    #[test]
    fn explicit_backend_choice_is_the_only_attempt() {
        assert_eq!(
            native_backend_attempts(Some(context::Backends::vulkan())),
            Attempts {
                first: context::Backends::vulkan(),
                fallback: None,
            }
        );
    }

    #[test]
    fn preparation_budget_tracks_refresh_with_bounded_reserve() {
        assert_eq!(
            preparation_window(None),
            PreparationWindow {
                budget: std::time::Duration::from_millis(4),
                deadline: std::time::Duration::from_nanos(16_666_667),
            }
        );
        assert_eq!(
            preparation_window(Some(60_000)),
            PreparationWindow {
                budget: std::time::Duration::from_millis(8),
                deadline: std::time::Duration::from_nanos(16_666_666),
            }
        );
        assert_eq!(
            preparation_window(Some(240_000)),
            PreparationWindow {
                budget: std::time::Duration::from_micros(2_166)
                    + std::time::Duration::from_nanos(666),
                deadline: std::time::Duration::from_micros(4_166)
                    + std::time::Duration::from_nanos(666),
            }
        );
        assert_eq!(
            preparation_window(Some(1_000_000)),
            PreparationWindow {
                budget: std::time::Duration::from_micros(750),
                deadline: std::time::Duration::from_millis(1),
            }
        );
    }

    #[test]
    fn same_structure_update_cannot_replace_selected_residency_identity() {
        let mut shell = native_scroll_shell();
        shell.start();
        let window = shell.runtime().session().windows()[0].id();
        let size = crate::geometry::Size::new(360, 180);
        assert!(shell.set_window_size(window, size));
        let initial = shell.drain();
        let active = initial
            .presentations()
            .last()
            .cloned()
            .expect("table fixture should produce an active presentation");
        shell.runtime_mut().finish_render_report(
            active.window(),
            active.epoch(),
            active.invalidation(),
            active.layout(),
            active.stack(),
            active.property_only(),
            crate::diagnostics::RenderReport::new(
                std::time::Duration::ZERO,
                std::time::Duration::ZERO,
                Instant::now(),
            ),
        );
        let cell = active
            .layout()
            .frames()
            .iter()
            .find(|frame| frame.table_cell().is_some())
            .expect("table fixture should materialize a cell");
        let point = crate::geometry::Point::new(cell.rect().x() + 1, cell.rect().y() + 1);
        let target = active
            .layout()
            .scroll_target_at(point, crate::interaction::ScrollDelta::vertical(1))
            .expect("table cell should route vertical scrolling");
        let projection = active
            .layout()
            .scroll_projections()
            .iter()
            .find(|projection| {
                projection.target() == &target && projection.viewport().max_scroll().y() > 0
            })
            .expect("table should have active vertical residency");
        let (_, active_maximum) = projection
            .accepted_offsets()
            .expect("active table residency should prove an interval");
        let first = crate::interaction::ScrollOffset::new(0, active_maximum.y() + 1);

        shell
            .runtime_mut()
            .handle_input(window, crate::Input::scroll_to(target.clone(), first))
            .expect("first hard-edge request should be handled");
        let selected_work = shell.drain();
        let selected = selected_work
            .presentations()
            .last()
            .cloned()
            .expect("first hard-edge request should select one residency candidate");
        assert!(!selected.property_only());

        let latest = crate::interaction::ScrollOffset::new(
            0,
            selected
                .layout()
                .scroll_projections()
                .iter()
                .find(|projection| projection.target() == &target)
                .and_then(crate::layout::ScrollProjection::accepted_offsets)
                .map(|(_, maximum)| maximum.y().saturating_add(10_000))
                .unwrap_or(first.y().saturating_add(10_000))
                .min(projection.viewport().max_scroll().y()),
        );
        assert!(latest.y() > first.y());
        shell
            .runtime_mut()
            .handle_input(window, crate::Input::scroll_to(target, latest))
            .expect("newer hard-edge intent should coalesce behind the selected front");
        assert!(
            shell.drain().presentations().is_empty(),
            "same-urgency intent must not construct another residency candidate"
        );

        shell
            .runtime_mut()
            .handle_input(
                window,
                crate::Input::pointer_move(Some(crate::interaction::Target::label(
                    "native.same-structure-overtake",
                    "Same-structure overtake",
                ))),
            )
            .expect("an unrelated paint should author a newer frame");
        let update_work = shell.drain();
        let update = update_work
            .presentations()
            .last()
            .cloned()
            .expect("unrelated paint should produce a newer presentation");
        assert!(update.epoch() > selected.epoch());
        assert!(
            selected.stack().same_structure(update.stack()),
            "fixture must exercise the native same-structure enqueue path"
        );

        let mut pending = super::super::PendingPresentation::new(selected.clone());
        pending.enqueue(update);
        let completed = pending.complete();
        assert_eq!(
            completed.prepared.epoch(),
            selected.epoch(),
            "same-structure updates must not replace the residency identity already selected by the scheduler"
        );
        let resolution = resolve_completed_presentation(&active, completed);
        assert!(matches!(
            resolution.outcome,
            PendingCompletionOutcome::ActivatePrepared
        ));
        assert!(shell.runtime_mut().finish_render_report(
            resolution.actual.window(),
            resolution.actual.epoch(),
            resolution.actual.invalidation(),
            resolution.actual.layout(),
            resolution.actual.stack(),
            resolution.actual.property_only(),
            crate::diagnostics::RenderReport::new(
                std::time::Duration::ZERO,
                std::time::Duration::ZERO,
                Instant::now(),
            ),
        ));
        let follow_up = shell
            .drain()
            .presentations()
            .last()
            .cloned()
            .expect("retiring the preserved front must author one latest-intent follow-up");
        let scroll = projection.node();
        assert_eq!(follow_up.properties().scroll_offset(scroll), Some(latest));
    }

    #[test]
    fn pending_residency_keeps_semantically_unchanged_updates_in_the_active_epoch() {
        assert!(
            incoming_refreshes_active(true, true, true),
            "a semantically unchanged active-compatible update must refresh the active epoch while residency realization owns the next candidate epoch"
        );
        assert!(!incoming_refreshes_active(true, false, true));
        assert!(!incoming_refreshes_active(false, true, true));
        assert!(!incoming_refreshes_active(true, true, false));
    }

    #[test]
    fn required_residency_candidate_preempts_selected_proactive_preparation() {
        let mut shell = native_scroll_shell();
        shell.start();
        let window = shell.runtime().session().windows()[0].id();
        let size = crate::geometry::Size::new(360, 180);
        assert!(shell.set_window_size(window, size));
        let initial = shell.drain();
        let active = initial
            .presentations()
            .last()
            .cloned()
            .expect("table fixture should produce an active presentation");
        shell.runtime_mut().finish_render_report(
            active.window(),
            active.epoch(),
            active.invalidation(),
            active.layout(),
            active.stack(),
            active.property_only(),
            crate::diagnostics::RenderReport::new(
                std::time::Duration::ZERO,
                std::time::Duration::ZERO,
                Instant::now(),
            ),
        );
        let cell = active
            .layout()
            .frames()
            .iter()
            .find(|frame| frame.table_cell().is_some())
            .expect("table fixture should materialize a cell");
        let point = crate::geometry::Point::new(cell.rect().x() + 1, cell.rect().y() + 1);
        let target = active
            .layout()
            .scroll_target_at(point, crate::interaction::ScrollDelta::vertical(1))
            .expect("table cell should route vertical scrolling");
        let projection = active
            .layout()
            .scroll_projections()
            .iter()
            .find(|projection| {
                projection.target() == &target && projection.viewport().max_scroll().y() > 0
            })
            .expect("table should have active vertical residency");
        let (minimum, maximum) = projection
            .accepted_offsets()
            .expect("active table residency should prove an interval");
        let threshold = projection.viewport().rect().height().max(2) / 2;
        let soft = crate::interaction::ScrollOffset::new(
            0,
            maximum
                .y()
                .saturating_sub(threshold)
                .max(minimum.y().saturating_add(1)),
        );

        shell
            .runtime_mut()
            .handle_input(window, crate::Input::scroll_to(target.clone(), soft))
            .expect("soft resident motion should be handled");
        let property_work = shell.drain();
        let property = property_work
            .presentations()
            .last()
            .cloned()
            .expect("soft motion should produce a property frame");
        assert!(property.property_only());
        shell.runtime_mut().finish_render_report(
            property.window(),
            property.epoch(),
            property.invalidation(),
            property.layout(),
            property.stack(),
            property.property_only(),
            crate::diagnostics::RenderReport::new(
                std::time::Duration::ZERO,
                std::time::Duration::ZERO,
                Instant::now(),
            ),
        );
        let proactive_work = shell.drain();
        let proactive = proactive_work
            .presentations()
            .last()
            .cloned()
            .expect("soft motion should select proactive preparation");
        assert_eq!(
            proactive.residency_urgency(),
            Some(scene::ResidencyUrgency::Proactive)
        );

        let required_offset = crate::interaction::ScrollOffset::new(
            0,
            maximum
                .y()
                .saturating_add(10_000)
                .min(projection.viewport().max_scroll().y()),
        );
        shell
            .runtime_mut()
            .handle_input(
                window,
                crate::Input::scroll_to(target.clone(), required_offset),
            )
            .expect("out-of-residency motion should be handled");
        let required_work = shell.drain();
        let required = required_work
            .presentations()
            .last()
            .cloned()
            .expect("required motion should replace speculative queued work");
        assert_eq!(
            required.residency_urgency(),
            Some(scene::ResidencyUrgency::Required)
        );

        let pending = super::super::PendingPresentation::new(proactive.clone());
        assert!(required_candidate_preempts_proactive(&pending, &required));
        let required_pending = super::super::PendingPresentation::new(required);
        assert!(!required_candidate_preempts_proactive(
            &required_pending,
            &proactive
        ));
        assert!(
            !shell
                .runtime_mut()
                .preempt_proactive_residency_candidate(window, proactive.epoch()),
            "required queued work already covers the latest request, so preemption needs no extra follow-up candidate"
        );
        let diagnostics = shell
            .runtime()
            .diagnostics(window)
            .expect("preemption diagnostics should remain available");
        assert_eq!(diagnostics.scroll.scroll_residency_candidates_superseded, 1);
        assert_eq!(diagnostics.scroll.scroll_residency_proactive_preemptions, 1);
    }

    #[test]
    fn active_resident_reversal_retires_obsolete_pending_scroll_intent() {
        let mut shell = native_scroll_shell();
        shell.start();
        let window = shell.runtime().session().windows()[0].id();
        let size = crate::geometry::Size::new(360, 180);
        assert!(shell.set_window_size(window, size));
        let initial = shell.drain();
        let active = initial
            .presentations()
            .last()
            .cloned()
            .expect("table fixture should produce an active presentation");
        shell.runtime_mut().finish_render_report(
            active.window(),
            active.epoch(),
            active.invalidation(),
            active.layout(),
            active.stack(),
            active.property_only(),
            crate::diagnostics::RenderReport::new(
                std::time::Duration::ZERO,
                std::time::Duration::ZERO,
                Instant::now(),
            ),
        );
        let cell = active
            .layout()
            .frames()
            .iter()
            .find(|frame| {
                frame.table_cell().is_some_and(|cell| {
                    cell.row() == crate::virtual_list::Key::new(1)
                        && cell.column() == crate::interaction::Id::new("detail")
                })
            })
            .expect("table fixture should materialize a detail cell");
        let point = crate::geometry::Point::new(cell.rect().x() + 1, cell.rect().y() + 1);
        let target = active
            .layout()
            .scroll_target_at(point, crate::interaction::ScrollDelta::vertical(1))
            .expect("table cell should route vertical scrolling");
        let projection = active
            .layout()
            .scroll_projections()
            .iter()
            .find(|projection| {
                projection.target() == &target && projection.viewport().max_scroll().y() > 0
            })
            .expect("table should have an active vertical residency");
        let scroll = projection.node();
        let (_, active_maximum) = projection
            .accepted_offsets()
            .expect("active table residency should prove a resident interval");
        let forward = crate::interaction::ScrollOffset::new(
            0,
            active_maximum
                .y()
                .saturating_add(10_000)
                .min(projection.viewport().max_scroll().y()),
        );

        shell
            .runtime_mut()
            .handle_input(window, crate::Input::scroll_to(target.clone(), forward))
            .expect("large forward request should be handled");
        let forward_work = shell.drain();
        let prepared = forward_work
            .presentations()
            .last()
            .cloned()
            .expect("large forward request should select a residency candidate");
        let prepared_epoch = prepared.epoch();
        assert!(!active.stack().same_structure(prepared.stack()));

        shell
            .runtime_mut()
            .handle_input(
                window,
                crate::Input::scroll_to(target, crate::interaction::ScrollOffset::default()),
            )
            .expect("reversal into the active residency should be handled");
        let reverse_work = shell.drain();
        let reverse = reverse_work
            .presentations()
            .last()
            .cloned()
            .expect("active-resident reversal should produce one property candidate");
        assert!(active.stack().same_structure(reverse.stack()));
        assert_eq!(
            reverse.properties().scroll_offset(scroll),
            Some(crate::interaction::ScrollOffset::default())
        );

        let pending = super::super::PendingPresentation::new(prepared);
        assert!(
            pending_scroll_intent_is_obsolete(&active, &pending, &reverse),
            "a candidate pointing away from the latest active-resident intent must not activate after the reversal"
        );
        assert!(
            shell
                .runtime_mut()
                .cancel_residency_pipeline(window, prepared_epoch),
            "the selected candidate and its queued proactive successor must retire as one obsolete pipeline"
        );
        assert!(shell.drain().presentations().is_empty());
        assert_eq!(
            shell
                .runtime()
                .diagnostics(window)
                .expect("fixture window should retain diagnostics")
                .scroll
                .scroll_residency_pipelines_cancelled,
            1
        );
    }

    #[test]
    #[cfg(target_os = "windows")]
    fn implicit_windows_policy_earns_tenancy_through_dx12_first() {
        let attempts = native_backend_attempts(None);
        assert_eq!(attempts.first, context::Backends::dx12());
        assert!(
            attempts
                .fallback
                .is_some_and(|fallback| fallback.contains(context::Backends::vulkan()))
        );
    }
}
