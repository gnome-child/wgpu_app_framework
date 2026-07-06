pub mod focus;
mod interaction;
mod request;
mod service;
mod snapshot;
mod window;

pub use focus::Focus;
pub use request::{FileDialog, Request, RequestKind};
pub use service::CloseWindow;
pub use snapshot::Snapshot;
pub use window::{Window, WindowSnapshot};

pub(in crate::scratch) use service::{Service, register};

use super::draft;

#[derive(Debug)]
pub struct Session {
    pub(in crate::scratch::session) windows: Vec<Window>,
    pub(in crate::scratch::session) next_window_id: u64,
    pub(in crate::scratch::session) draft_limit: usize,
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
