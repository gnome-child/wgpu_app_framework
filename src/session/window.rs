use super::super::{geometry, interaction, pointer, response, scene, state, window as app_window};
use super::{FileDialog, Focus, Session, Snapshot};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Window {
    pub(super) facts: app_window::Facts,
    pub(super) invalidation: Option<response::Invalidation>,
    pub(super) presented_revision: Option<state::Revision>,
    pub(super) cursor: pointer::Cursor,
    pub(super) cursor_changed: bool,
    pub(super) focus: Option<Focus>,
    pub(super) menu_restore_focus: Option<Focus>,
    pub(super) file_dialog: Option<FileDialog>,
    pub(super) interaction: interaction::Interaction,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WindowSnapshot {
    pub(super) facts: app_window::Facts,
    pub(super) focus: Option<Focus>,
}

impl Window {
    pub(super) fn new(facts: app_window::Facts, draft_limit: usize) -> Self {
        Self {
            facts,
            invalidation: Some(response::Invalidation::Rebuild),
            presented_revision: None,
            cursor: pointer::Cursor::Default,
            cursor_changed: false,
            focus: None,
            menu_restore_focus: None,
            file_dialog: None,
            interaction: interaction::Interaction::new(draft_limit),
        }
    }

    pub(super) fn restore(snapshot: WindowSnapshot, draft_limit: usize) -> Self {
        Self {
            facts: snapshot.facts,
            invalidation: Some(response::Invalidation::Rebuild),
            presented_revision: None,
            cursor: pointer::Cursor::Default,
            cursor_changed: false,
            focus: snapshot.focus,
            menu_restore_focus: None,
            file_dialog: None,
            interaction: interaction::Interaction::new(draft_limit),
        }
    }

    pub fn id(&self) -> app_window::Id {
        self.facts.id()
    }

    pub fn title(&self) -> &str {
        self.facts.title()
    }

    pub fn inner_size(&self) -> geometry::Size {
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

    pub fn redraw_requested(&self) -> bool {
        self.invalidation.is_some()
    }

    pub(crate) fn invalidation(&self) -> Option<response::Invalidation> {
        self.invalidation
    }

    pub fn presented_revision(&self) -> Option<state::Revision> {
        self.presented_revision
    }

    pub fn cursor(&self) -> pointer::Cursor {
        self.cursor
    }

    pub fn focus(&self) -> Option<Focus> {
        self.focus
    }

    pub(crate) fn interaction(&self) -> &interaction::Interaction {
        &self.interaction
    }
}

impl WindowSnapshot {
    pub fn new(id: app_window::Id, title: impl Into<String>, focus: Option<Focus>) -> Self {
        Self {
            facts: app_window::Facts::new(
                id,
                title,
                app_window::Options::default_inner_size(),
                app_window::Options::default_canvas_color(),
                app_window::Kind::Application,
            ),
            focus,
        }
    }

    pub(super) fn from_window(window: &Window) -> Self {
        Self {
            facts: window.facts.clone(),
            focus: window.focus,
        }
    }

    pub fn id(&self) -> app_window::Id {
        self.facts.id()
    }

    pub fn title(&self) -> &str {
        self.facts.title()
    }

    pub fn inner_size(&self) -> geometry::Size {
        self.facts.inner_size()
    }

    pub fn canvas_color(&self) -> scene::Color {
        self.facts.canvas_color()
    }

    pub fn kind(&self) -> app_window::Kind {
        self.facts.kind()
    }

    pub fn focus(&self) -> Option<Focus> {
        self.focus
    }
}

impl Session {
    pub fn open_window(&mut self, options: app_window::Options) -> app_window::Id {
        let (title, inner_size, canvas_color, kind) = options.into_parts();
        let id = app_window::Id::new(self.next_window_id);
        self.next_window_id += 1;
        self.windows.push(Window::new(
            app_window::Facts::new(id, title, inner_size, canvas_color, kind),
            self.draft_limit,
        ));

        id
    }

    pub fn close_window(&mut self, id: app_window::Id) -> bool {
        let Some(index) = self.windows.iter().position(|window| window.id() == id) else {
            return false;
        };

        self.windows.remove(index);
        self.departed.push(id);
        true
    }

    pub(crate) fn take_departed(&mut self) -> Vec<app_window::Id> {
        std::mem::take(&mut self.departed)
    }

    pub fn request_redraw(&mut self, id: app_window::Id) -> bool {
        self.request_invalidation(id, response::Invalidation::Rebuild)
    }

    pub(crate) fn request_invalidation(
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
        previous != window.invalidation
    }

    pub fn clear_redraw_request(&mut self, id: app_window::Id) -> bool {
        let Some(window) = self.window_mut(id) else {
            return false;
        };
        let changed = window.invalidation.is_some();
        window.invalidation = None;
        changed
    }

    pub(crate) fn mark_presented(&mut self, id: app_window::Id, revision: state::Revision) -> bool {
        let Some(window) = self.window_mut(id) else {
            return false;
        };
        let changed = window.presented_revision != Some(revision);
        window.presented_revision = Some(revision);
        changed
    }

    pub(crate) fn set_cursor(&mut self, id: app_window::Id, cursor: pointer::Cursor) -> bool {
        let Some(window) = self.window_mut(id) else {
            return false;
        };
        let changed = window.cursor != cursor;
        if changed {
            window.cursor = cursor;
            window.cursor_changed = true;
        }
        changed
    }

    pub(crate) fn take_cursor_updates(&mut self) -> Vec<pointer::Update> {
        self.windows
            .iter_mut()
            .filter_map(|window| {
                window.cursor_changed.then(|| {
                    window.cursor_changed = false;
                    pointer::Update::new(window.id(), window.cursor)
                })
            })
            .collect()
    }

    pub(crate) fn prune_removed_interaction(
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
        self.windows.iter().find(|window| window.id() == id)
    }

    pub fn contains(&self, id: app_window::Id) -> bool {
        self.window(id).is_some()
    }

    pub(crate) fn snapshot(&self) -> Snapshot {
        Snapshot::new(
            self.windows
                .iter()
                .map(WindowSnapshot::from_window)
                .collect(),
        )
    }

    pub(crate) fn restore(&mut self, snapshot: Snapshot) {
        self.departed.clear();
        self.windows = snapshot
            .into_windows()
            .into_iter()
            .map(|window| Window::restore(window, self.draft_limit))
            .collect();
        self.next_window_id = self
            .windows
            .iter()
            .map(|window| window.id().get() + 1)
            .max()
            .unwrap_or_default();
    }

    pub(crate) fn set_draft_limit(&mut self, limit: usize) {
        self.draft_limit = limit;
        for window in &mut self.windows {
            window.interaction.set_text_draft_limit(limit);
        }
    }

    pub(in crate::session) fn window_mut(&mut self, id: app_window::Id) -> Option<&mut Window> {
        self.windows.iter_mut().find(|window| window.id() == id)
    }
}
