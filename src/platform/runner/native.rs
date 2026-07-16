use std::time::Instant;

use winit::{
    event::{ElementState, WindowEvent as WinitWindowEvent},
    event_loop::{ActiveEventLoop, ControlFlow, EventLoop},
};

use super::super::{Error, Native, NativeError, Platform, RunError};
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

        if !self.exit_if_finished(event_loop) {
            self.sync_control_flow(event_loop);
        }
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
        let control_flow = control_flow(schedule, now);
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
    use crate::{platform::RedrawRequests, window};

    #[derive(Clone, Copy)]
    enum PacingTrace {
        Steady,
        Burst,
        Delayed,
    }

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

    fn legacy_completion_anchor_is_not_due(
        presented: Instant,
        demand: Instant,
        refresh_hz: u32,
    ) -> bool {
        demand
            >= presented
                .checked_add(Duration::from_secs_f64(1.0 / f64::from(refresh_hz)))
                .expect("test refresh deadline")
    }

    #[test]
    fn legacy_completion_anchor_delays_demand_after_present_submission() {
        let presented = Instant::now();
        let demand = presented + Duration::from_millis(1);

        assert!(!legacy_completion_anchor_is_not_due(presented, demand, 60));
    }

    fn next_refresh_after(origin: Instant, demand: Instant, interval: Duration) -> Instant {
        let mut refresh = origin + interval;
        while refresh < demand {
            refresh += interval;
        }
        refresh
    }

    fn require_pacing_trace(refresh_hz: u32, trace: PacingTrace) {
        let interval = Duration::from_secs_f64(1.0 / f64::from(refresh_hz));
        let origin = Instant::now();
        let window = window::Id::new(u64::from(refresh_hz));
        let mut requests = RedrawRequests::default();

        match trace {
            PacingTrace::Steady => {
                let mut presented = origin;
                for _ in 0..3 {
                    let demand = presented + Duration::from_millis(1);
                    assert!(requests.begin(window));
                    assert!(
                        !requests.begin(window),
                        "duplicate steady redraw escaped dedupe"
                    );
                    let delivered = next_refresh_after(origin, demand, interval);
                    assert!(delivered.saturating_duration_since(demand) <= interval);
                    requests.delivered(window);
                    presented = delivered;
                }
            }
            PacingTrace::Burst => {
                let demands = [
                    origin + Duration::from_millis(1),
                    origin + Duration::from_millis(2),
                    origin + Duration::from_millis(3),
                ];
                assert!(requests.begin(window));
                assert!(!requests.begin(window));
                assert!(!requests.begin(window));
                let delivered = next_refresh_after(origin, demands[0], interval);
                assert!(delivered.saturating_duration_since(demands[0]) <= interval);
                requests.delivered(window);
                assert!(
                    requests.begin(window),
                    "post-delivery demand must issue once"
                );
            }
            PacingTrace::Delayed => {
                let demand = origin + Duration::from_millis(1);
                assert!(requests.begin(window));
                let nominal = next_refresh_after(origin, demand, interval);
                let delivered = nominal + interval + interval;
                assert!(
                    !requests.begin(window),
                    "delayed delivery must remain coalesced"
                );
                requests.delivered(window);
                let next_demand = delivered + Duration::from_millis(1);
                assert!(requests.begin(window));
                assert!(next_demand < delivered + interval);
            }
        }
    }

    #[test]
    fn pacing_case_steady_60hz() {
        require_pacing_trace(60, PacingTrace::Steady);
    }

    #[test]
    fn pacing_case_burst_60hz() {
        require_pacing_trace(60, PacingTrace::Burst);
    }

    #[test]
    fn pacing_case_delayed_60hz() {
        require_pacing_trace(60, PacingTrace::Delayed);
    }

    #[test]
    fn pacing_case_steady_90hz() {
        require_pacing_trace(90, PacingTrace::Steady);
    }

    #[test]
    fn pacing_case_burst_90hz() {
        require_pacing_trace(90, PacingTrace::Burst);
    }

    #[test]
    fn pacing_case_delayed_90hz() {
        require_pacing_trace(90, PacingTrace::Delayed);
    }

    #[test]
    fn pacing_case_steady_120hz() {
        require_pacing_trace(120, PacingTrace::Steady);
    }

    #[test]
    fn pacing_case_burst_120hz() {
        require_pacing_trace(120, PacingTrace::Burst);
    }

    #[test]
    fn pacing_case_delayed_120hz() {
        require_pacing_trace(120, PacingTrace::Delayed);
    }

    #[test]
    fn pacing_case_steady_144hz() {
        require_pacing_trace(144, PacingTrace::Steady);
    }

    #[test]
    fn pacing_case_burst_144hz() {
        require_pacing_trace(144, PacingTrace::Burst);
    }

    #[test]
    fn pacing_case_delayed_144hz() {
        require_pacing_trace(144, PacingTrace::Delayed);
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
