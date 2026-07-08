#[cfg(test)]
use super::super::view;
use super::super::{scene, session};
use crate::animation;
use crate::pointer;

#[cfg(test)]
pub(crate) struct Work {
    presentations: Vec<view::Presentation>,
    requests: Vec<session::Request>,
    cursor_updates: Vec<pointer::Update>,
    pending_tasks: usize,
    task_completions: usize,
    animation_schedule: animation::Schedule,
}

pub(crate) struct RenderWork {
    presentations: Vec<scene::Presentation>,
    requests: Vec<session::Request>,
    cursor_updates: Vec<pointer::Update>,
    pending_tasks: usize,
    task_completions: usize,
    animation_schedule: animation::Schedule,
}

#[cfg(test)]
impl Work {
    pub(super) fn new(
        presentations: Vec<view::Presentation>,
        requests: Vec<session::Request>,
        cursor_updates: Vec<pointer::Update>,
        pending_tasks: usize,
        task_completions: usize,
        animation_schedule: animation::Schedule,
    ) -> Self {
        Self {
            presentations,
            requests,
            cursor_updates,
            pending_tasks,
            task_completions,
            animation_schedule,
        }
    }

    pub(crate) fn presentations(&self) -> &[view::Presentation] {
        &self.presentations
    }

    pub(crate) fn requests(&self) -> &[session::Request] {
        &self.requests
    }

    pub(crate) fn pending_tasks(&self) -> usize {
        self.pending_tasks
    }

    pub(crate) fn task_completions(&self) -> usize {
        self.task_completions
    }

    pub(crate) fn is_empty(&self) -> bool {
        self.presentations.is_empty()
            && self.requests.is_empty()
            && self.cursor_updates.is_empty()
            && self.pending_tasks == 0
            && self.task_completions == 0
            && self.animation_schedule == animation::Schedule::Idle
    }
}

impl RenderWork {
    pub(super) fn new(
        presentations: Vec<scene::Presentation>,
        requests: Vec<session::Request>,
        cursor_updates: Vec<pointer::Update>,
        pending_tasks: usize,
        task_completions: usize,
        animation_schedule: animation::Schedule,
    ) -> Self {
        Self {
            presentations,
            requests,
            cursor_updates,
            pending_tasks,
            task_completions,
            animation_schedule,
        }
    }

    pub(crate) fn presentations(&self) -> &[scene::Presentation] {
        &self.presentations
    }

    pub(crate) fn requests(&self) -> &[session::Request] {
        &self.requests
    }

    pub(crate) fn cursor_updates(&self) -> &[pointer::Update] {
        &self.cursor_updates
    }

    pub(crate) fn pending_tasks(&self) -> usize {
        self.pending_tasks
    }

    pub(crate) fn task_completions(&self) -> usize {
        self.task_completions
    }

    pub(crate) fn animation_schedule(&self) -> animation::Schedule {
        self.animation_schedule
    }

    #[cfg(test)]
    pub(crate) fn is_empty(&self) -> bool {
        self.presentations.is_empty()
            && self.requests.is_empty()
            && self.cursor_updates.is_empty()
            && self.pending_tasks == 0
            && self.task_completions == 0
            && self.animation_schedule == animation::Schedule::Idle
    }
}
