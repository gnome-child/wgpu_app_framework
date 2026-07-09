use std::collections::HashMap;

use crate::{interaction, render};

use super::super::{session, window as app_window};

mod adapter;
mod color;
mod context;
mod error;
mod paint;
mod poll;
mod popup;
mod request;
mod surface;
mod window;

pub use context::NativeContext;
pub use error::NativeError;

pub struct Native {
    context: Option<render::Context>,
    renderer: Option<render::Renderer>,
    windows: HashMap<app_window::Id, window::Window>,
    popups: HashMap<PopupKey, window::Window>,
    raw_windows: HashMap<winit::window::WindowId, app_window::Id>,
    raw_popups: HashMap<winit::window::WindowId, PopupKey>,
    requests: Vec<session::Request>,
    poll_requested: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
struct PopupKey {
    parent: app_window::Id,
    id: interaction::Id,
}

impl PopupKey {
    fn new(parent: app_window::Id, id: interaction::Id) -> Self {
        Self { parent, id }
    }
}

impl Native {
    pub fn new() -> Self {
        Self {
            context: None,
            renderer: None,
            windows: HashMap::new(),
            popups: HashMap::new(),
            raw_windows: HashMap::new(),
            raw_popups: HashMap::new(),
            requests: Vec::new(),
            poll_requested: false,
        }
    }

    pub fn initialize(&mut self) -> Result<(), NativeError> {
        self.ensure_context()
    }

    pub fn ready(&self) -> bool {
        self.context.is_some()
    }
}

impl Default for Native {
    fn default() -> Self {
        Self::new()
    }
}
