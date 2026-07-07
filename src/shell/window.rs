use crate::{geometry, scene, state::State, window as app_window};

use super::{Shell, work::WindowChanges};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Window {
    id: app_window::Id,
    title: String,
    size: geometry::Size,
    canvas_color: scene::Color,
}

impl Window {
    pub(super) fn new(
        id: app_window::Id,
        title: impl Into<String>,
        size: geometry::Size,
        canvas_color: scene::Color,
    ) -> Self {
        Self {
            id,
            title: title.into(),
            size,
            canvas_color,
        }
    }

    pub fn id(&self) -> app_window::Id {
        self.id
    }

    pub fn title(&self) -> &str {
        &self.title
    }

    pub fn size(&self) -> geometry::Size {
        self.size
    }

    pub fn canvas_color(&self) -> scene::Color {
        self.canvas_color
    }

    pub(super) fn set_size(&mut self, size: geometry::Size) {
        self.size = size;
    }

    pub(super) fn update(&mut self, title: String, canvas_color: scene::Color) {
        self.title = title;
        self.canvas_color = canvas_color;
    }
}

impl<M: State, E: Send + 'static> Shell<M, E> {
    pub fn set_window_size(&mut self, window: app_window::Id, size: geometry::Size) -> bool {
        self.sync_windows();
        let Some(window) = self.windows.iter_mut().find(|entry| entry.id() == window) else {
            return false;
        };

        let changed = window.size() != size;
        window.set_size(size);
        if changed {
            self.runtime.request_redraw(window.id());
        }
        true
    }

    pub fn window_size(&self, window: app_window::Id) -> Option<geometry::Size> {
        self.windows
            .iter()
            .find(|entry| entry.id() == window)
            .map(Window::size)
    }

    pub(in crate::shell) fn sync_windows(&mut self) -> WindowChanges {
        let windows = self
            .runtime
            .session()
            .windows()
            .iter()
            .map(|window| {
                (
                    window.id(),
                    window.title().to_owned(),
                    window.inner_size(),
                    window.canvas_color(),
                )
            })
            .collect::<Vec<_>>();

        let mut changes = WindowChanges::default();
        self.windows.retain(|entry| {
            let retained = windows
                .iter()
                .any(|(window, _, _, _)| *window == entry.id());
            if !retained {
                changes.closed.push(entry.id());
            }
            retained
        });

        for (window, title, inner_size, canvas_color) in windows {
            if let Some(entry) = self.windows.iter_mut().find(|entry| entry.id() == window) {
                entry.update(title, canvas_color);
                continue;
            }

            let entry = Window::new(window, title, inner_size, canvas_color);
            changes.opened.push(entry.clone());
            self.windows.push(entry);
        }

        changes
    }
}

pub(in crate::shell) fn default_size() -> geometry::Size {
    app_window::Options::default_inner_size()
}
