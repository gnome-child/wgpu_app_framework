mod access;
mod dialog;
mod handler;
mod native;

use super::super::state::State;
use super::{Backend, Error, Events, Native, Platform};
use crate::task;
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
}
