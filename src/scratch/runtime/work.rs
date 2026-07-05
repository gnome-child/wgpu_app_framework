use super::super::{scene, session, view};

pub struct Work {
    presentations: Vec<view::Presentation>,
    requests: Vec<session::Request>,
    pending_tasks: usize,
    task_completions: usize,
}

pub struct RenderWork {
    presentations: Vec<scene::Presentation>,
    requests: Vec<session::Request>,
    pending_tasks: usize,
    task_completions: usize,
}

impl Work {
    pub(super) fn new(
        presentations: Vec<view::Presentation>,
        requests: Vec<session::Request>,
        pending_tasks: usize,
        task_completions: usize,
    ) -> Self {
        Self {
            presentations,
            requests,
            pending_tasks,
            task_completions,
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
    }
}

impl RenderWork {
    pub(super) fn new(
        presentations: Vec<scene::Presentation>,
        requests: Vec<session::Request>,
        pending_tasks: usize,
        task_completions: usize,
    ) -> Self {
        Self {
            presentations,
            requests,
            pending_tasks,
            task_completions,
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

    pub fn is_empty(&self) -> bool {
        self.presentations.is_empty()
            && self.requests.is_empty()
            && self.pending_tasks == 0
            && self.task_completions == 0
    }
}
