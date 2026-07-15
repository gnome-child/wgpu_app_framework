use crate::geometry::area;
use std::collections::{HashMap, hash_map::Entry};
use std::sync::Arc;
use std::time::Instant;

use crate::diagnostics;
use crate::render::{self, Canvas, Context, Renderer, canvas, context, surface};

use super::super::{NativeError, Window};
use super::window::{InitialSize, Options, Window as NativeWindow};
use super::{Native, NativeContext, PopupPrewarmState};
use crate::{shell, window as app_window};

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
            .is_some_and(|active| Arc::ptr_eq(active.commit(), presentation.commit()));
        let mut refreshes_active = false;
        let mut activation_from_pending = false;
        let actual = if active.is_none() || active_matches {
            refreshes_active = active_matches && self.pending_presentations.contains_key(&window);
            presentation.clone()
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
                    .expect("pending presentation was just admitted");
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
                let readiness = renderer.synchronize_commit(
                    context,
                    canvas.canvas(),
                    preparing.commit(),
                    preparing.properties(),
                    remaining,
                    preparation.deadline,
                )?;
                match readiness {
                    render::CommitReadiness::Ready => {
                        let pending = self
                            .pending_presentations
                            .remove(&window)
                            .expect("ready preparation must still be pending");
                        match pending.complete() {
                            super::PendingCompletion::Activate(prepared) => {
                                activation_from_pending = true;
                                break prepared;
                            }
                            super::PendingCompletion::ActivateAndContinue { prepared, latest } => {
                                self.pending_presentations
                                    .insert(window, super::PendingPresentation::new(latest));
                                activation_from_pending = true;
                                break prepared;
                            }
                        }
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
        let report = renderer.draw_commit(
            context,
            native_window.canvas_mut(),
            actual.commit(),
            actual.properties(),
            actual.overlays(),
        );
        let report = match report {
            Ok(report) => report,
            Err(error) => {
                if activation_from_pending {
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

        let presented = report.present_timing.is_some();
        if presented
            && context.windows_popup_composition_supported()
            && !self.popup_prewarm.contains_key(&window)
        {
            self.popup_prewarm.insert(window, PopupPrewarmState::Armed);
        }

        if activation_from_pending {
            if !presented {
                self.requeue_failed_activation(window, actual.clone());
            }
        }

        if presented {
            self.active_presentations.insert(window, actual.clone());
        }

        let report = diagnostics::RenderReport::new(acquire_wait, draw, Instant::now())
            .with_presented(presented)
            .with_acquire_outcome(report.acquire_outcome)
            .with_pipeline_timings(report.batch_prepare, encode_submit_present)
            .with_draw_stats(report.stats)
            .with_environment(environment)
            .with_group_composites(group_composites)
            .with_filter_pool_entries(filter_layer_pool_entries, filter_scratch_pool_entries);
        let presented = super::super::Presented::new(actual, report);
        Ok(if refreshes_active {
            super::super::PresentResult::ActiveRefreshedAndDeferred(presented)
        } else if self.pending_presentations.contains_key(&window) {
            super::super::PresentResult::PresentedAndDeferred(presented)
        } else {
            presented.into()
        })
    }

    pub(in crate::platform::native) fn resume_native_presentations(
        &mut self,
    ) -> Result<Vec<super::super::PresentResult>, NativeError> {
        let pending = self
            .pending_presentations
            .values()
            .map(|pending| pending.preparing.clone())
            .collect::<Vec<_>>();
        let mut results = Vec::with_capacity(pending.len());

        for presentation in pending {
            let window = presentation.window();
            let is_current = self
                .pending_presentations
                .get(&window)
                .is_some_and(|current| {
                    Arc::ptr_eq(current.preparing.commit(), presentation.commit())
                });
            if is_current {
                results.push(self.present_native(&presentation)?);
            }
        }

        Ok(results)
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
    match newest
        .properties()
        .project_onto(active.commit(), active.properties())
    {
        Ok((properties, true)) => active.with_active_properties(properties),
        Ok((_, false)) | Err(_) => active.clone(),
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
