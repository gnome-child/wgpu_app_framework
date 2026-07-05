use winit::{
    application::ApplicationHandler, event::WindowEvent as WinitWindowEvent,
    event_loop::ActiveEventLoop,
};

use super::super::{Native, NativeContext};
use super::Runner;
use crate::scratch::state::State;

impl<M: State, E: Send + 'static> ApplicationHandler<E> for Runner<M, E, Native> {
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

    fn user_event(&mut self, event_loop: &ActiveEventLoop, event: E) {
        self.platform
            .host_mut()
            .shell_mut()
            .runtime_mut()
            .emit(event);

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
        if self.platform.backend_mut().take_poll_requested() {
            let mut context = NativeContext::new(event_loop);
            if let Err(error) = self.platform.poll_with(&mut context) {
                self.fail(event_loop, error);
                return;
            }
        }

        self.finish_native_pass(event_loop);
    }
}
