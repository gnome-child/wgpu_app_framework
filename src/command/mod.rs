mod observer;
mod registry;
mod set;
mod spec;
mod state;
mod surface;
mod trigger;

use std::time::Duration;

pub use observer::Observation;
pub(crate) use observer::Observers;
pub use registry::Registry;
pub use set::{Member, Set};
pub(crate) use spec::KeyChordKind;
pub use spec::{KeyChord, Listing, Spec, Standard};
pub(crate) use state::Availability;
pub use state::State;
pub(crate) use surface::{Candidates, Global, ResolvedAction, ResolvedActions};
pub use trigger::Trigger;
pub(crate) use trigger::{AnyTrigger, AnyValueTrigger};

/// App code dispatches by this type, not by an action id.
pub trait Command: 'static + Sized {
    type Args: Send + 'static;
    type Output: Send + 'static;

    /// Stable metadata for keymaps, debugging, settings, plugins, etc.
    /// Normal compiled app code should still use the command type.
    const NAME: &'static str;
    const HISTORY: History = History::Automatic;

    fn history_group(_args: &Self::Args) -> Option<HistoryGroup> {
        None
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum History {
    /// Runtime snapshots the model before dispatch and records changed state in undo history.
    Automatic,
    /// Handling target commits through framework services; runtime repairs changed user overrides.
    Committed,
    /// Command is not undoable; changed responses still advance revision but do not snapshot.
    Ignored,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct HistoryGroup {
    key: &'static str,
    coalesce_window: Duration,
}

impl HistoryGroup {
    pub fn new(key: &'static str) -> Self {
        Self {
            key,
            coalesce_window: DEFAULT_HISTORY_GROUP_COALESCE_WINDOW,
        }
    }

    pub fn with_coalesce_window(mut self, coalesce_window: Duration) -> Self {
        self.coalesce_window = coalesce_window;
        self
    }

    pub(crate) fn coalesce_window(&self) -> Duration {
        self.coalesce_window
    }
}

const DEFAULT_HISTORY_GROUP_COALESCE_WINDOW: Duration = Duration::from_millis(1000);
