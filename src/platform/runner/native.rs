use std::time::Instant;

use winit::{
    event::{ElementState, WindowEvent as WinitWindowEvent},
    event_loop::{ActiveEventLoop, ControlFlow, EventLoop},
};

use super::super::{Backend, Error, Native, NativeError, Platform, RunError};
use super::{Runner, RunnerEvent};
use crate::animation;
use crate::{host, shell, state::State, task};

impl<M: State, E: Send + 'static> Runner<M, E, Native> {
    pub fn new(shell: shell::Shell<M, E>) -> Self {
        Self::with_platform(Platform::new(shell, Native::new()))
    }

    pub fn run(mut self) -> Result<(), RunError<NativeError>> {
        let event_loop = EventLoop::<RunnerEvent<E>>::with_user_event().build()?;
        self.task_proxy = Some(event_loop.create_proxy());
        self.executor = Some(task::Executor::new());

        event_loop.run_app(&mut self)?;

        if let Some(error) = self.take_error() {
            return Err(error.into());
        }

        Ok(())
    }

    pub fn translate_window_event(
        &mut self,
        raw_window: winit::window::WindowId,
        event: &WinitWindowEvent,
    ) -> Option<host::Event> {
        if matches!(
            event,
            WinitWindowEvent::MouseInput {
                state: ElementState::Pressed,
                ..
            }
        ) {
            self.platform
                .host_mut()
                .shell_mut()
                .runtime_mut()
                .set_multi_click_settings(super::super::event::system_multi_click_settings());
        }
        self.platform
            .backend_mut()
            .route_cursor_host_event(raw_window, event);

        if let Some(window) = self.platform.backend().window_for_raw(raw_window) {
            return self.events.window_event(window, event);
        }

        let popup = self.platform.backend().popup_for_raw(raw_window)?;
        log::trace!(
            target: "wgpu_l3::native_popup",
            "routing popup event {:?} for parent {:?}: kind={} first_present_stage={} elapsed_us={}",
            popup.id(),
            popup.parent(),
            popup_window_event_kind(event),
            popup.first_present_stage(),
            popup.first_present_elapsed_micros()
        );
        self.events
            .popup_window_event(popup.realization(), popup.scale_factor(), event)
    }

    pub(in crate::platform::runner) fn sync_native_event_state(&mut self) {
        let windows = self
            .platform
            .host()
            .windows()
            .iter()
            .map(|window| window.id())
            .collect::<Vec<_>>();

        self.events
            .retain_windows(|window| windows.contains(&window));
        self.presentation_pulses
            .retain(|window, _| windows.contains(window));
        self.frame_demands.retain(|window| windows.contains(window));
        self.issued_frame_redraws
            .retain(|window| windows.contains(window));

        self.frame_demands.extend(
            self.platform
                .host()
                .shell()
                .runtime()
                .session()
                .windows()
                .iter()
                .filter(|window| window.redraw_requested())
                .map(crate::session::Window::id),
        );

        for window in windows {
            if let Some(scale_factor) = self.platform.backend().scale_factor(window) {
                self.events.set_window_scale_factor(window, scale_factor);
            }
        }
    }

    pub(in crate::platform::runner) fn finish_native_pass(&mut self, event_loop: &ActiveEventLoop) {
        self.sync_native_event_state();

        if let Err(error) = self.handle_native_requests(event_loop) {
            self.fail(event_loop, error);
            return;
        }

        self.dispatch_pending_tasks();

        if let Err(error) = self.present_due_interaction_frame(event_loop) {
            self.fail(event_loop, error);
            return;
        }

        if !self.exit_if_finished(event_loop) {
            self.sync_control_flow(event_loop);
        }
    }

    fn present_due_interaction_frame(
        &mut self,
        event_loop: &ActiveEventLoop,
    ) -> Result<(), Error<NativeError>> {
        let now = Instant::now();
        let due = self
            .frame_demands
            .iter()
            .copied()
            .filter(|window| !self.issued_frame_redraws.contains(window))
            .filter(|window| {
                let refresh = self.platform.backend().display_refresh_millihertz(*window);
                self.presentation_pulses
                    .get(window)
                    .is_none_or(|pulse| pulse.is_due(now, refresh))
            })
            .collect::<Vec<_>>();
        if due.is_empty() {
            return Ok(());
        }

        let mut context = super::super::NativeContext::new(event_loop);
        for window in due {
            self.platform
                .backend_mut()
                .request_redraw(&mut context, window)
                .map_err(Error::Backend)?;
            self.issued_frame_redraws.insert(window);
        }
        Ok(())
    }

    fn dispatch_pending_tasks(&mut self) {
        let (Some(executor), Some(proxy)) = (&self.executor, &self.task_proxy) else {
            return;
        };

        while let Some((id, task)) = self
            .platform
            .host_mut()
            .shell_mut()
            .runtime_mut()
            .take_next_task()
        {
            let proxy = proxy.clone();
            let scheduled = executor.spawn(move || {
                let event = task.run();
                let _ = proxy.send_event(RunnerEvent::TaskCompleted { id, event });
            });
            if !scheduled {
                log::error!("worker executor rejected task {id:?}");
                self.platform
                    .host_mut()
                    .shell_mut()
                    .runtime_mut()
                    .cancel_task(id);
            }
        }
    }

    pub(in crate::platform::runner) fn fail(
        &mut self,
        event_loop: &ActiveEventLoop,
        error: Error<NativeError>,
    ) {
        self.error = Some(error);
        event_loop.exit();
    }

    pub(in crate::platform::runner) fn sync_control_flow(&self, event_loop: &ActiveEventLoop) {
        if event_loop.exiting() {
            return;
        }

        let now = Instant::now();
        let schedule = if self.platform.backend().poll_requested() {
            animation::Schedule::NextFrame
        } else {
            self.platform.animation_schedule()
        };
        let pulse_schedule =
            self.frame_demands
                .iter()
                .fold(animation::Schedule::Idle, |schedule, window| {
                    if self.issued_frame_redraws.contains(window) {
                        return schedule;
                    }
                    let refresh = self.platform.backend().display_refresh_millihertz(*window);
                    let due = self
                        .presentation_pulses
                        .get(window)
                        .and_then(|pulse| pulse.deadline(refresh));
                    schedule.merge(
                        due.map(animation::Schedule::At)
                            .unwrap_or(animation::Schedule::NextFrame),
                    )
                });
        let control_flow = control_flow(schedule.merge(pulse_schedule), now);
        event_loop.set_control_flow(control_flow);
    }

    pub(in crate::platform::runner) fn exit_if_finished(
        &self,
        event_loop: &ActiveEventLoop,
    ) -> bool {
        if self.started && self.platform.host().windows().is_empty() {
            event_loop.exit();
            true
        } else {
            false
        }
    }
}

fn control_flow(schedule: animation::Schedule, now: Instant) -> ControlFlow {
    match schedule {
        animation::Schedule::Idle => ControlFlow::Wait,
        animation::Schedule::At(deadline) if deadline > now => ControlFlow::WaitUntil(deadline),
        animation::Schedule::At(_) | animation::Schedule::NextFrame => ControlFlow::Poll,
    }
}

#[cfg(test)]
mod tests {
    use std::time::{Duration, Instant};

    use winit::event_loop::ControlFlow;

    use super::control_flow;
    use crate::animation::Schedule;
    use crate::platform::runner::PresentationPulse;

    #[test]
    fn event_loop_projection_preserves_every_schedule_outcome() {
        let now = Instant::now();
        let deadline = now + Duration::from_millis(10);

        assert_eq!(control_flow(Schedule::Idle, now), ControlFlow::Wait);
        assert_eq!(
            control_flow(Schedule::At(deadline), now),
            ControlFlow::WaitUntil(deadline)
        );
        assert_eq!(control_flow(Schedule::At(now), now), ControlFlow::Poll);
        assert_eq!(control_flow(Schedule::NextFrame, now), ControlFlow::Poll);
    }

    #[test]
    fn presentation_pulses_are_window_local_refresh_clocks() {
        let mut slow = PresentationPulse::default();
        let mut fast = PresentationPulse::default();
        let now = Instant::now();

        assert!(slow.is_due(now, Some(60_000)));
        assert!(fast.is_due(now, Some(144_000)));
        slow.mark_presented(now);
        fast.mark_presented(now);
        assert!(!slow.is_due(now + Duration::from_millis(8), Some(60_000)));
        assert!(fast.is_due(now + Duration::from_millis(8), Some(144_000)));
        assert!(slow.is_due(now + Duration::from_millis(17), Some(60_000)));

        fast.mark_presented(now + Duration::from_millis(8));
        assert_eq!(
            slow.deadline(Some(60_000)),
            now.checked_add(Duration::from_secs_f64(1.0 / 60.0))
        );
    }
}

fn popup_window_event_kind(event: &WinitWindowEvent) -> &'static str {
    match event {
        WinitWindowEvent::RedrawRequested => "redraw-requested",
        WinitWindowEvent::Occluded(true) => "occluded",
        WinitWindowEvent::Occluded(false) => "unoccluded",
        WinitWindowEvent::Resized(_) => "resized",
        WinitWindowEvent::Moved(_) => "moved",
        WinitWindowEvent::ScaleFactorChanged { .. } => "scale-factor-changed",
        WinitWindowEvent::CursorEntered { .. } => "cursor-entered",
        WinitWindowEvent::CursorMoved { .. } => "cursor-moved",
        WinitWindowEvent::CursorLeft { .. } => "cursor-left",
        WinitWindowEvent::MouseInput { .. } => "mouse-input",
        WinitWindowEvent::MouseWheel { .. } => "mouse-wheel",
        WinitWindowEvent::Focused(true) => "focused",
        WinitWindowEvent::Focused(false) => "unfocused",
        WinitWindowEvent::Ime(_) => "ime",
        WinitWindowEvent::Destroyed => "destroyed",
        _ => "other",
    }
}

pub fn run<M: State, E: Send + 'static>(
    shell: shell::Shell<M, E>,
) -> Result<(), RunError<NativeError>> {
    Runner::new(shell).run()
}
