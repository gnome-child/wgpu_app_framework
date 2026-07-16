#[cfg(test)]
use super::super::view;
use super::super::{scene, session};
use crate::animation;
use crate::ime;
use crate::overlay;
use crate::pointer;
use crate::window;

pub(crate) struct ImmediateWork {
    requests: Vec<session::Request>,
    cursor_updates: Vec<pointer::Update>,
    pending_tasks: usize,
    task_completions: usize,
    animation_schedule: animation::Schedule,
    redraw_windows: Vec<window::Id>,
}

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
    popup_presentations: Option<Vec<overlay::PopupPresentation>>,
    ime_projections: Vec<ime::Projection>,
    requests: Vec<session::Request>,
    cursor_updates: Vec<pointer::Update>,
    pending_tasks: usize,
    task_completions: usize,
    animation_schedule: animation::Schedule,
    redraw_windows: Vec<window::Id>,
}

impl ImmediateWork {
    pub(super) fn new(
        requests: Vec<session::Request>,
        cursor_updates: Vec<pointer::Update>,
        pending_tasks: usize,
        task_completions: usize,
        animation_schedule: animation::Schedule,
        redraw_windows: Vec<window::Id>,
    ) -> Self {
        Self {
            requests,
            cursor_updates,
            pending_tasks,
            task_completions,
            animation_schedule,
            redraw_windows,
        }
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

    pub(crate) fn redraw_windows(&self) -> &[window::Id] {
        &self.redraw_windows
    }
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
        popup_presentations: Option<Vec<overlay::PopupPresentation>>,
        ime_projections: Vec<ime::Projection>,
        requests: Vec<session::Request>,
        cursor_updates: Vec<pointer::Update>,
        pending_tasks: usize,
        task_completions: usize,
        animation_schedule: animation::Schedule,
        redraw_windows: Vec<window::Id>,
    ) -> Self {
        Self {
            presentations,
            popup_presentations,
            ime_projections,
            requests,
            cursor_updates,
            pending_tasks,
            task_completions,
            animation_schedule,
            redraw_windows,
        }
    }

    pub(crate) fn presentations(&self) -> &[scene::Presentation] {
        &self.presentations
    }

    pub(crate) fn popup_presentations(&self) -> Option<&[overlay::PopupPresentation]> {
        self.popup_presentations.as_deref()
    }

    pub(crate) fn ime_projections(&self) -> &[ime::Projection] {
        &self.ime_projections
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

    pub(crate) fn redraw_windows(&self) -> &[window::Id] {
        &self.redraw_windows
    }

    #[cfg(test)]
    pub(crate) fn is_empty(&self) -> bool {
        self.presentations.is_empty()
            && self.popup_presentations.is_none()
            && self.ime_projections.is_empty()
            && self.requests.is_empty()
            && self.cursor_updates.is_empty()
            && self.pending_tasks == 0
            && self.task_completions == 0
            && self.animation_schedule == animation::Schedule::Idle
            && self.redraw_windows.is_empty()
    }
}
