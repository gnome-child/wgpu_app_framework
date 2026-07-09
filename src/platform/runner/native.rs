use std::time::Instant;

use winit::{
    event::WindowEvent as WinitWindowEvent,
    event_loop::{ActiveEventLoop, EventLoop},
};

use super::super::{Error, Native, NativeError, Platform, RunError};
use super::Runner;
use crate::animation;
use crate::{host, shell, state::State};

impl<M: State, E: Send + 'static> Runner<M, E, Native> {
    pub fn new(shell: shell::Shell<M, E>) -> Self {
        Self::with_platform(Platform::new(shell, Native::new()))
    }

    pub fn run(mut self) -> Result<(), RunError<NativeError>> {
        let event_loop = EventLoop::<E>::with_user_event().build()?;

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
        if let Some(window) = self.platform.backend().window_for_raw(raw_window) {
            return self.events.window_event(window, event);
        }

        let popup = self.platform.backend().popup_for_raw(raw_window)?;
        log::trace!(
            target: "wgpu_l3::native_popup",
            "routing popup event {:?} for parent {:?}",
            popup.id(),
            popup.parent()
        );
        self.events
            .popup_window_event(popup.parent(), popup.bounds(), popup.scale_factor(), event)
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

        if !self.exit_if_finished(event_loop) {
            self.sync_control_flow(event_loop);
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

        let schedule = if self.platform.backend().poll_requested() {
            animation::Schedule::NextFrame
        } else {
            self.platform.animation_schedule()
        };
        let control_flow = schedule.control_flow(Instant::now());
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

pub fn run<M: State, E: Send + 'static>(
    shell: shell::Shell<M, E>,
) -> Result<(), RunError<NativeError>> {
    Runner::new(shell).run()
}
