use crate::{state::State, window as app_window};
use std::time::Instant;

use super::{Shell, Work, window};

impl<M: State, E: Send + 'static> Shell<M, E> {
    pub fn start(&mut self) {
        self.runtime.start();
    }

    pub fn drain(&mut self) -> Work {
        let changes = self.sync_windows();
        let windows = self.windows.clone();
        let work = self.runtime.drain_scenes(|id| {
            windows
                .iter()
                .find(|entry| entry.id() == id)
                .map(super::Window::size)
                .unwrap_or_else(window::default_size)
        });

        Work::from_render_work(work, changes)
    }

    pub fn step(&mut self) -> Work {
        self.runtime.invalidate_due_animation_frames(Instant::now());

        if self.runtime.pending_task_completions() > 0 {
            self.runtime.dispatch_next_task_completion();
        } else if self.runtime.pending_tasks() > 0 {
            self.runtime.run_next_task();
        }

        self.drain()
    }

    pub(in crate::shell) fn request_close_window(&mut self, window: app_window::Id) {
        let trigger = self
            .runtime
            .trigger::<super::super::session::CloseWindow>(());
        self.runtime.invoke_focused(window, trigger);
    }
}
