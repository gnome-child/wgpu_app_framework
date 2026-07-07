mod focus;
mod interaction;
mod request;
mod service;
mod snapshot;
mod window;

pub use focus::{Focus, Reason, Visibility};
pub use request::{FileDialog, Request, RequestKind};
pub use service::{CloseWindow, OpenCommandPalette};
pub use snapshot::Snapshot;
pub use window::{Window, WindowSnapshot};

pub(crate) use service::{Service, register};

use super::draft;

#[derive(Debug)]
pub struct Session {
    pub(in crate::session) windows: Vec<Window>,
    pub(in crate::session) next_window_id: u64,
    pub(in crate::session) draft_limit: usize,
}

impl Default for Session {
    fn default() -> Self {
        Self {
            windows: Vec::new(),
            next_window_id: 0,
            draft_limit: draft::input::DEFAULT_DRAFT_LIMIT,
        }
    }
}
