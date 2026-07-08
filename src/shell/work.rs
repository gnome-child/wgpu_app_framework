use crate::animation;
use crate::{pointer, runtime, session, window};

use super::{Presentation, Window};

pub struct Work {
    opened_windows: Vec<Window>,
    closed_windows: Vec<window::Id>,
    presentations: Vec<Presentation>,
    requests: Vec<session::Request>,
    cursor_updates: Vec<pointer::Update>,
    pending_tasks: usize,
    task_completions: usize,
    animation_schedule: animation::Schedule,
}

#[derive(Default)]
pub(super) struct WindowChanges {
    pub(super) opened: Vec<Window>,
    pub(super) closed: Vec<window::Id>,
}

impl Work {
    pub(super) fn from_render_work(
        work: runtime::work::RenderWork,
        changes: WindowChanges,
    ) -> Self {
        Self {
            opened_windows: changes.opened,
            closed_windows: changes.closed,
            presentations: work
                .presentations()
                .iter()
                .cloned()
                .map(Presentation::from_scene_presentation)
                .collect(),
            requests: work.requests().to_vec(),
            cursor_updates: work.cursor_updates().to_vec(),
            pending_tasks: work.pending_tasks(),
            task_completions: work.task_completions(),
            animation_schedule: work.animation_schedule(),
        }
    }

    pub fn opened_windows(&self) -> &[Window] {
        &self.opened_windows
    }

    pub fn closed_windows(&self) -> &[window::Id] {
        &self.closed_windows
    }

    pub fn presentations(&self) -> &[Presentation] {
        &self.presentations
    }

    pub fn requests(&self) -> &[session::Request] {
        &self.requests
    }

    pub fn cursor_updates(&self) -> &[pointer::Update] {
        &self.cursor_updates
    }

    pub fn pending_tasks(&self) -> usize {
        self.pending_tasks
    }

    pub fn task_completions(&self) -> usize {
        self.task_completions
    }

    pub fn needs_poll(&self) -> bool {
        self.pending_tasks > 0 || self.task_completions > 0
    }

    pub(crate) fn animation_schedule(&self) -> animation::Schedule {
        self.animation_schedule
    }

    pub fn is_empty(&self) -> bool {
        self.opened_windows.is_empty()
            && self.closed_windows.is_empty()
            && self.presentations.is_empty()
            && self.requests.is_empty()
            && self.cursor_updates.is_empty()
            && self.pending_tasks == 0
            && self.task_completions == 0
            && self.animation_schedule == animation::Schedule::Idle
    }
}
