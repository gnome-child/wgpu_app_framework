use std::time::Instant;

use winit::{
    application::ApplicationHandler, event::WindowEvent as WinitWindowEvent,
    event_loop::ActiveEventLoop,
};

use super::super::{Native, NativeContext};
use super::{Runner, RunnerEvent};
use crate::state::State;

impl<M: State, E: Send + 'static> ApplicationHandler<RunnerEvent<E>> for Runner<M, E, Native> {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if self.started {
            self.sync_control_flow(event_loop);
            return;
        }

        let mut context = NativeContext::new(event_loop);
        if let Err(error) = self.platform.start_with(&mut context) {
            self.fail(event_loop, error);
            return;
        }

        self.started = true;
        self.finish_native_pass(event_loop);
    }

    fn user_event(&mut self, event_loop: &ActiveEventLoop, event: RunnerEvent<E>) {
        match event {
            RunnerEvent::TaskCompleted { id, event } => {
                let runtime = self.platform.host_mut().shell_mut().runtime_mut();
                if runtime.accept_task_completion(id, event) {
                    runtime.dispatch_next_task_completion();
                }
            }
        }

        let mut context = NativeContext::new(event_loop);
        if let Err(error) = self.platform.drain_with(&mut context) {
            self.fail(event_loop, error);
            return;
        }

        self.finish_native_pass(event_loop);
    }

    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        raw_window: winit::window::WindowId,
        event: WinitWindowEvent,
    ) {
        let Some(event) = self.translate_window_event(raw_window, &event) else {
            return;
        };

        let mut context = NativeContext::new(event_loop);
        if let Err(error) = self.platform.handle_event_with(&mut context, event) {
            self.fail(event_loop, error);
            return;
        }

        self.finish_native_pass(event_loop);
    }

    fn about_to_wait(&mut self, event_loop: &ActiveEventLoop) {
        let poll_requested = self.platform.backend_mut().take_poll_requested();
        let animation_due = self.platform.animation_schedule().is_due(Instant::now());

        if poll_requested || animation_due {
            let mut context = NativeContext::new(event_loop);
            if let Err(error) = self.platform.poll_with(&mut context) {
                self.fail(event_loop, error);
                return;
            }
        }

        self.finish_native_pass(event_loop);
    }
}
