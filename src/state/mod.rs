mod change;
mod reason;
mod revision;
mod snapshot;
mod store;

pub use change::Change;
pub use reason::Reason;
pub use revision::Revision;
pub use snapshot::Snapshot;
pub use store::Store;

pub(crate) use snapshot::PendingSnapshot;
pub(crate) use store::DEFAULT_CHANGE_LIMIT;

/// Durable application model state. The framework owns it through `Store`.
pub trait State: Clone + 'static {}
