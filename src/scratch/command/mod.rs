mod observer;
mod registry;
mod spec;
mod state;
mod trigger;

pub use observer::Observation;
pub(in crate::scratch) use observer::Observers;
pub use registry::Registry;
pub use spec::{KeyChord, Spec};
pub(in crate::scratch) use state::Availability;
pub use state::State;
pub use trigger::Trigger;
pub(in crate::scratch) use trigger::{AnyTrigger, AnyValueTrigger};

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
}

impl HistoryGroup {
    pub fn new(key: &'static str) -> Self {
        Self { key }
    }
}
