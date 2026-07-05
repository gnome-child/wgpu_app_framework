use super::super::{geometry, interaction, scene, state, window as app_window};
use super::{FileDialog, Focus, Session, Snapshot};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Window {
    pub(super) id: app_window::Id,
    pub(super) title: String,
    pub(super) inner_size: geometry::Size,
    pub(super) canvas_color: scene::Color,
    pub(super) redraw_requested: bool,
    pub(super) presented_revision: Option<state::Revision>,
    pub(super) focus: Option<Focus>,
    pub(super) file_dialog: Option<FileDialog>,
    pub(super) interaction: interaction::Interaction,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WindowSnapshot {
    pub(super) id: app_window::Id,
    pub(super) title: String,
    pub(super) inner_size: geometry::Size,
    pub(super) canvas_color: scene::Color,
    pub(super) focus: Option<Focus>,
}

impl Window {
    pub(super) fn new(
        id: app_window::Id,
        title: String,
        inner_size: geometry::Size,
        canvas_color: scene::Color,
    ) -> Self {
        Self {
            id,
            title,
            inner_size,
            canvas_color,
            redraw_requested: true,
            presented_revision: None,
            focus: None,
            file_dialog: None,
            interaction: interaction::Interaction::default(),
        }
    }

    pub(super) fn restore(snapshot: WindowSnapshot) -> Self {
        Self {
            id: snapshot.id,
            title: snapshot.title,
            inner_size: snapshot.inner_size,
            canvas_color: snapshot.canvas_color,
            redraw_requested: true,
            presented_revision: None,
            focus: snapshot.focus,
            file_dialog: None,
            interaction: interaction::Interaction::default(),
        }
    }

    pub fn id(&self) -> app_window::Id {
        self.id
    }

    pub fn title(&self) -> &str {
        &self.title
    }

    pub fn inner_size(&self) -> geometry::Size {
        self.inner_size
    }

    pub fn canvas_color(&self) -> scene::Color {
        self.canvas_color
    }

    pub fn redraw_requested(&self) -> bool {
        self.redraw_requested
    }

    pub fn presented_revision(&self) -> Option<state::Revision> {
        self.presented_revision
    }

    pub fn focus(&self) -> Option<Focus> {
        self.focus.clone()
    }

    pub fn interaction(&self) -> &interaction::Interaction {
        &self.interaction
    }
}

impl WindowSnapshot {
    pub fn new(id: app_window::Id, title: impl Into<String>, focus: Option<Focus>) -> Self {
        Self {
            id,
            title: title.into(),
            inner_size: app_window::Options::default_inner_size(),
            canvas_color: app_window::Options::default_canvas_color(),
            focus,
        }
    }

    pub(super) fn from_window(window: &Window) -> Self {
        Self {
            id: window.id,
            title: window.title.clone(),
            inner_size: window.inner_size,
            canvas_color: window.canvas_color,
            focus: window.focus.clone(),
        }
    }

    pub fn id(&self) -> app_window::Id {
        self.id
    }

    pub fn title(&self) -> &str {
        &self.title
    }

    pub fn inner_size(&self) -> geometry::Size {
        self.inner_size
    }

    pub fn canvas_color(&self) -> scene::Color {
        self.canvas_color
    }

    pub fn focus(&self) -> Option<Focus> {
        self.focus.clone()
    }
}

impl Session {
    pub fn open_window(&mut self, options: app_window::Options) -> app_window::Id {
        let (title, inner_size, canvas_color) = options.into_parts();
        let id = app_window::Id::new(self.next_window_id);
        self.next_window_id += 1;
        self.windows
            .push(Window::new(id, title, inner_size, canvas_color));

        id
    }

    pub fn close_window(&mut self, id: app_window::Id) -> bool {
        let Some(index) = self.windows.iter().position(|window| window.id == id) else {
            return false;
        };

        self.windows.remove(index);
        true
    }

    pub fn request_redraw(&mut self, id: app_window::Id) -> bool {
        let Some(window) = self.window_mut(id) else {
            return false;
        };
        let changed = !window.redraw_requested;
        window.redraw_requested = true;
        changed
    }

    pub fn clear_redraw_request(&mut self, id: app_window::Id) -> bool {
        let Some(window) = self.window_mut(id) else {
            return false;
        };
        let changed = window.redraw_requested;
        window.redraw_requested = false;
        changed
    }

    pub(in crate::scratch) fn mark_presented(
        &mut self,
        id: app_window::Id,
        revision: state::Revision,
    ) -> bool {
        let Some(window) = self.window_mut(id) else {
            return false;
        };
        let changed = window.presented_revision != Some(revision);
        window.presented_revision = Some(revision);
        changed
    }

    pub fn windows(&self) -> &[Window] {
        &self.windows
    }

    pub fn window(&self, id: app_window::Id) -> Option<&Window> {
        self.windows.iter().find(|window| window.id == id)
    }

    pub fn contains(&self, id: app_window::Id) -> bool {
        self.window(id).is_some()
    }

    pub(in crate::scratch) fn snapshot(&self) -> Snapshot {
        Snapshot::new(
            self.windows
                .iter()
                .map(WindowSnapshot::from_window)
                .collect(),
        )
    }

    pub(in crate::scratch) fn restore(&mut self, snapshot: Snapshot) {
        self.windows = snapshot
            .into_windows()
            .into_iter()
            .map(Window::restore)
            .collect();
        self.next_window_id = self
            .windows
            .iter()
            .map(|window| window.id.get() + 1)
            .max()
            .unwrap_or_default();
    }

    pub(in crate::scratch::session) fn window_mut(
        &mut self,
        id: app_window::Id,
    ) -> Option<&mut Window> {
        self.windows.iter_mut().find(|window| window.id == id)
    }
}
