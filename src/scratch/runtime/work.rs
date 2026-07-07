use super::super::{scene, session, view};
use crate::animation;

pub struct Work {
    presentations: Vec<view::Presentation>,
    requests: Vec<session::Request>,
    pending_tasks: usize,
    task_completions: usize,
    animation_schedule: animation::Schedule,
}

pub struct RenderWork {
    presentations: Vec<scene::Presentation>,
    requests: Vec<session::Request>,
    pending_tasks: usize,
    task_completions: usize,
    animation_schedule: animation::Schedule,
}

impl Work {
    pub(super) fn new(
        presentations: Vec<view::Presentation>,
        requests: Vec<session::Request>,
        pending_tasks: usize,
        task_completions: usize,
        animation_schedule: animation::Schedule,
    ) -> Self {
        Self {
            presentations,
            requests,
            pending_tasks,
            task_completions,
            animation_schedule,
        }
    }

    pub fn presentations(&self) -> &[view::Presentation] {
        &self.presentations
    }

    pub fn requests(&self) -> &[session::Request] {
        &self.requests
    }

    pub fn pending_tasks(&self) -> usize {
        self.pending_tasks
    }

    pub fn task_completions(&self) -> usize {
        self.task_completions
    }

    pub fn is_empty(&self) -> bool {
        self.presentations.is_empty()
            && self.requests.is_empty()
            && self.pending_tasks == 0
            && self.task_completions == 0
            && self.animation_schedule == animation::Schedule::Idle
    }
}

impl RenderWork {
    pub(super) fn new(
        presentations: Vec<scene::Presentation>,
        requests: Vec<session::Request>,
        pending_tasks: usize,
        task_completions: usize,
        animation_schedule: animation::Schedule,
    ) -> Self {
        Self {
            presentations,
            requests,
            pending_tasks,
            task_completions,
            animation_schedule,
        }
    }

    pub fn presentations(&self) -> &[scene::Presentation] {
        &self.presentations
    }

    pub fn requests(&self) -> &[session::Request] {
        &self.requests
    }

    pub fn pending_tasks(&self) -> usize {
        self.pending_tasks
    }

    pub fn task_completions(&self) -> usize {
        self.task_completions
    }

    pub(in crate::scratch) fn animation_schedule(&self) -> animation::Schedule {
        self.animation_schedule
    }

    pub fn is_empty(&self) -> bool {
        self.presentations.is_empty()
            && self.requests.is_empty()
            && self.pending_tasks == 0
            && self.task_completions == 0
            && self.animation_schedule == animation::Schedule::Idle
    }
}
