use super::super::{geometry, interaction, response, scene, state, window as app_window};
use super::{FileDialog, Focus, Session, Snapshot};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Window {
    pub(super) id: app_window::Id,
    pub(super) title: String,
    pub(super) inner_size: geometry::Size,
    pub(super) canvas_color: scene::Color,
    pub(super) invalidation: Option<response::Invalidation>,
    pub(super) presented_revision: Option<state::Revision>,
    pub(super) focus: Option<Focus>,
    pub(super) menu_restore_focus: Option<Focus>,
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
        draft_limit: usize,
    ) -> Self {
        Self {
            id,
            title,
            inner_size,
            canvas_color,
            invalidation: Some(response::Invalidation::Rebuild),
            presented_revision: None,
            focus: None,
            menu_restore_focus: None,
            file_dialog: None,
            interaction: interaction::Interaction::new(draft_limit),
        }
    }

    pub(super) fn restore(snapshot: WindowSnapshot, draft_limit: usize) -> Self {
        Self {
            id: snapshot.id,
            title: snapshot.title,
            inner_size: snapshot.inner_size,
            canvas_color: snapshot.canvas_color,
            invalidation: Some(response::Invalidation::Rebuild),
            presented_revision: None,
            focus: snapshot.focus,
            menu_restore_focus: None,
            file_dialog: None,
            interaction: interaction::Interaction::new(draft_limit),
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
        self.invalidation.is_some()
    }

    pub(in crate::scratch) fn invalidation(&self) -> Option<response::Invalidation> {
        self.invalidation
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
        self.windows.push(Window::new(
            id,
            title,
            inner_size,
            canvas_color,
            self.draft_limit,
        ));

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
        self.request_invalidation(id, response::Invalidation::Rebuild)
    }

    pub(in crate::scratch) fn request_invalidation(
        &mut self,
        id: app_window::Id,
        invalidation: response::Invalidation,
    ) -> bool {
        let Some(window) = self.window_mut(id) else {
            return false;
        };
        let previous = window.invalidation;
        window.invalidation =
            Some(previous.map_or(invalidation, |previous| previous.max(invalidation)));
        let changed = previous != window.invalidation;
        changed
    }

    pub fn clear_redraw_request(&mut self, id: app_window::Id) -> bool {
        let Some(window) = self.window_mut(id) else {
            return false;
        };
        let changed = window.invalidation.is_some();
        window.invalidation = None;
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

    pub(in crate::scratch) fn prune_removed_interaction(
        &mut self,
        id: app_window::Id,
        removed_nodes: &[super::super::composition::NodeId],
        removed_elements: &[interaction::Id],
    ) -> bool {
        let Some(window) = self.window_mut(id) else {
            return false;
        };

        window
            .interaction
            .prune_removed(removed_nodes, removed_elements)
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
            .map(|window| Window::restore(window, self.draft_limit))
            .collect();
        self.next_window_id = self
            .windows
            .iter()
            .map(|window| window.id.get() + 1)
            .max()
            .unwrap_or_default();
    }

    pub(in crate::scratch) fn set_draft_limit(&mut self, limit: usize) {
        self.draft_limit = limit;
        for window in &mut self.windows {
            window.interaction.set_text_draft_limit(limit);
        }
    }

    pub(in crate::scratch::session) fn window_mut(
        &mut self,
        id: app_window::Id,
    ) -> Option<&mut Window> {
        self.windows.iter_mut().find(|window| window.id == id)
    }
}
