use crate::{geometry, scene, state::State, window as app_window};

use super::{Shell, work::WindowChanges};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Window {
    facts: app_window::Facts,
}

impl Window {
    pub(super) fn new(facts: app_window::Facts) -> Self {
        Self { facts }
    }

    pub fn id(&self) -> app_window::Id {
        self.facts.id()
    }

    pub fn title(&self) -> &str {
        self.facts.title()
    }

    pub fn size(&self) -> geometry::Size {
        self.facts.inner_size()
    }

    pub fn canvas_color(&self) -> scene::Color {
        self.facts.canvas_color()
    }

    pub fn kind(&self) -> app_window::Kind {
        self.facts.kind()
    }

    pub(crate) fn facts(&self) -> &app_window::Facts {
        &self.facts
    }

    pub(super) fn set_size(&mut self, size: geometry::Size) {
        self.facts.set_inner_size(size);
    }

    pub(super) fn update(&mut self, facts: &app_window::Facts) {
        self.facts.replace_preserving_inner_size(facts);
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
            .map(|window| window.facts().clone())
            .collect::<Vec<_>>();

        let mut changes = WindowChanges::default();
        self.windows.retain(|entry| {
            let retained = windows.iter().any(|facts| facts.id() == entry.id());
            if !retained {
                changes.closed.push(entry.id());
            }
            retained
        });

        for facts in windows {
            if let Some(entry) = self
                .windows
                .iter_mut()
                .find(|entry| entry.id() == facts.id())
            {
                entry.update(&facts);
                continue;
            }

            let entry = Window::new(facts);
            changes.opened.push(entry.clone());
            self.windows.push(entry);
        }

        changes
    }
}

pub(in crate::shell) fn default_size() -> geometry::Size {
    app_window::Options::default_inner_size()
}
