mod access;
mod dialog;
mod handler;
mod native;

use super::super::state::State;
use super::{Backend, Error, Events, Native, Platform};

#[cfg(test)]
pub(crate) use dialog::file_dialog_selected;
pub use native::run;

pub struct Runner<M: State, E: Send + 'static = (), B: Backend = Native> {
    platform: Platform<M, E, B>,
    events: Events,
    started: bool,
    error: Option<Error<B::Error>>,
}
