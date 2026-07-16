mod access;
mod dialog;
mod handler;
mod native;

use super::super::state::State;
use super::{Backend, Error, Events, Native, Platform};
use crate::task;
use crate::window;
use std::collections::{HashMap, HashSet};
use std::time::{Duration, Instant};
use winit::event_loop::EventLoopProxy;

#[cfg(test)]
pub(crate) use dialog::file_dialog_selected;
pub use native::run;

pub(crate) enum RunnerEvent<E: Send + 'static> {
    TaskCompleted { id: task::Id, event: E },
}

pub struct Runner<M: State, E: Send + 'static = (), B: Backend = Native> {
    platform: Platform<M, E, B>,
    events: Events,
    started: bool,
    error: Option<Error<B::Error>>,
    executor: Option<task::Executor>,
    task_proxy: Option<EventLoopProxy<RunnerEvent<E>>>,
    presentation_pulses: HashMap<window::Id, PresentationPulse>,
    frame_demands: HashSet<window::Id>,
    issued_frame_redraws: HashSet<window::Id>,
}

#[derive(Default)]
struct PresentationPulse {
    last_present_submitted_at: Option<Instant>,
}

impl PresentationPulse {
    const FALLBACK_REFRESH_MILLIHERTZ: u32 = 60_000;

    fn is_due(&self, now: Instant, refresh_millihertz: Option<u32>) -> bool {
        self.deadline(refresh_millihertz)
            .is_none_or(|deadline| deadline <= now)
    }

    fn deadline(&self, refresh_millihertz: Option<u32>) -> Option<Instant> {
        let refresh_millihertz = refresh_millihertz
            .filter(|refresh| *refresh > 0)
            .unwrap_or(Self::FALLBACK_REFRESH_MILLIHERTZ);
        let interval = Duration::from_secs_f64(1_000.0 / f64::from(refresh_millihertz));
        self.last_present_submitted_at
            .and_then(|last| last.checked_add(interval))
    }

    fn mark_present_submitted(&mut self, now: Instant) {
        self.last_present_submitted_at = Some(now);
    }
}
