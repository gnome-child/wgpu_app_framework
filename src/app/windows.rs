use std::collections::HashMap;

use winit::event_loop::ActiveEventLoop;

use crate::app::rendering;
use crate::{native, window};

use super::Result;

pub struct Windows {
    windows: HashMap<window::Id, native::Window>,
    raw_windows: HashMap<winit::window::WindowId, window::Id>,
    next_window_id: u64,
}

impl Windows {
    pub fn new() -> Self {
        Self {
            windows: HashMap::new(),
            raw_windows: HashMap::new(),
            next_window_id: 1,
        }
    }

    pub fn open(
        &mut self,
        options: window::Options,
        rendering: &mut rendering::Driver,
        event_loop: &ActiveEventLoop,
    ) -> Result<window::Id> {
        let id = self.allocate_id();
        let native_options = native::window::Options {
            title: options.title,
            inner_area: options.inner_area,
        };
        let handle = native::Window::open(native_options, event_loop)?;
        let mut native_window = rendering.create_window(handle, options.canvas_color)?;

        use crate::render::frame::Status::*;
        match rendering.clear_window(&mut native_window)? {
            Presented => {}
            Skipped(reason) => {
                log::warn!("initial frame was skipped: {:#?}", reason);
            }
        }
        native_window.set_visibility(true);
        native_window.request_redraw();

        self.raw_windows.insert(native_window.raw_id(), id);
        self.windows.insert(id, native_window);

        Ok(id)
    }

    pub fn get(&self, id: window::Id) -> Option<&native::Window> {
        self.windows.get(&id)
    }

    pub fn get_mut(&mut self, id: window::Id) -> Option<&mut native::Window> {
        self.windows.get_mut(&id)
    }

    pub fn contains(&self, id: window::Id) -> bool {
        self.windows.contains_key(&id)
    }

    pub fn raw_id(&self, raw: winit::window::WindowId) -> Option<window::Id> {
        self.raw_windows.get(&raw).copied()
    }

    pub fn remove(&mut self, id: window::Id) -> Option<native::Window> {
        let native_window = self.windows.remove(&id)?;
        self.raw_windows.remove(&native_window.raw_id());
        Some(native_window)
    }

    pub fn request_redraw(&self, id: window::Id) {
        if let Some(window) = self.windows.get(&id) {
            window.request_redraw();
        }
    }

    pub fn is_empty(&self) -> bool {
        self.windows.is_empty()
    }

    fn allocate_id(&mut self) -> window::Id {
        if self.next_window_id == 0 {
            self.next_window_id = 1;
        }

        let id = window::Id::new(self.next_window_id);
        self.next_window_id += 1;
        id
    }
}

impl Default for Windows {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn allocates_stable_window_ids() {
        let mut windows = Windows::new();

        assert_eq!(windows.allocate_id(), window::Id::new(1));
        assert_eq!(windows.allocate_id(), window::Id::new(2));
    }

    #[test]
    fn raw_window_lookup_uses_registry_map() {
        let mut windows = Windows::new();
        let raw = winit::window::WindowId::dummy();
        let id = window::Id::new(7);

        windows.raw_windows.insert(raw, id);

        assert_eq!(windows.raw_id(raw), Some(id));
        assert_eq!(windows.raw_id(winit::window::WindowId::dummy()), Some(id));
    }
}
