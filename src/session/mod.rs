mod focus;
mod interaction;
mod request;
mod selection;
mod service;
mod snapshot;
mod table;
mod window;

pub use focus::{Focus, Reason, Visibility};
pub use request::{FileDialog, Request, RequestKind};
pub use service::{CloseWindow, OpenCommandPalette};
pub use snapshot::Snapshot;
pub use window::{Window, WindowSnapshot};

pub(crate) use service::{Service, register};

use super::{draft, window as app_window};

#[derive(Debug)]
pub struct Session {
    pub(in crate::session) windows: Vec<Window>,
    pub(in crate::session) next_window_id: u64,
    pub(in crate::session) draft_limit: usize,
    pub(in crate::session) departed: Vec<app_window::Id>,
}

impl Default for Session {
    fn default() -> Self {
        Self {
            windows: Vec::new(),
            next_window_id: 0,
            draft_limit: draft::DEFAULT_DRAFT_LIMIT,
            departed: Vec::new(),
        }
    }
}
