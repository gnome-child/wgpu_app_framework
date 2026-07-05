mod focus;
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

#[derive(Debug, Default)]
pub struct Session {
    pub(in crate::scratch::session) windows: Vec<Window>,
    pub(in crate::scratch::session) next_window_id: u64,
}
