use std::collections::HashMap;

use super::Diagnostics;
use crate::{session, window};

#[derive(Default)]
pub(crate) struct Store {
    windows: HashMap<window::Id, Diagnostics>,
}

impl Store {
    pub(crate) fn insert_window(&mut self, window: window::Id) {
        self.windows.entry(window).or_default();
    }

    pub(crate) fn remove_window(&mut self, window: window::Id) {
        self.windows.remove(&window);
    }

    pub(crate) fn restore_windows(&mut self, windows: &[session::Window]) {
        self.windows.clear();
        for window in windows {
            self.insert_window(window.id());
        }
    }

    pub(crate) fn get(&self, window: window::Id) -> Option<&Diagnostics> {
        self.windows.get(&window)
    }

    pub(crate) fn get_mut(&mut self, window: window::Id) -> &mut Diagnostics {
        self.windows.entry(window).or_default()
    }
}
