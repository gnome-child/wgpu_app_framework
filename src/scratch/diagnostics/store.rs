use std::collections::HashMap;

use super::Diagnostics;
use crate::scratch::{session, window};

#[derive(Default)]
pub(in crate::scratch) struct Store {
    windows: HashMap<window::Id, Diagnostics>,
}

impl Store {
    pub(in crate::scratch) fn insert_window(&mut self, window: window::Id) {
        self.windows.entry(window).or_default();
    }

    pub(in crate::scratch) fn remove_window(&mut self, window: window::Id) {
        self.windows.remove(&window);
    }

    pub(in crate::scratch) fn restore_windows(&mut self, windows: &[session::Window]) {
        self.windows.clear();
        for window in windows {
            self.insert_window(window.id());
        }
    }

    pub(in crate::scratch) fn get(&self, window: window::Id) -> Option<&Diagnostics> {
        self.windows.get(&window)
    }

    pub(in crate::scratch) fn get_mut(&mut self, window: window::Id) -> &mut Diagnostics {
        self.windows.entry(window).or_default()
    }
}
