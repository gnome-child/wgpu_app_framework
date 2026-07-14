use super::super::{
    feedback, geometry, interaction, pointer, response, scene, state, window as app_window,
};
use super::{FileDialog, Focus, Session, Snapshot};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Window {
    pub(super) facts: app_window::Facts,
    pub(super) invalidation: Option<response::effect::Invalidation>,
    pub(super) projected_revision: Option<state::Revision>,
    pub(super) desired_presentation_epoch: app_window::PresentationEpoch,
    pub(super) acknowledged_presentation_epoch: Option<app_window::PresentationEpoch>,
    cursor: Cursor,
    pub(super) focus: Option<Focus>,
    pub(super) menu_restore_focus: Option<Focus>,
    pub(super) file_dialog: Option<FileDialog>,
    pub(super) feedback: feedback::Stack,
    pub(super) interaction: interaction::Interaction,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Cursor {
    Synced(pointer::Cursor),
    Pending(pointer::Cursor),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WindowSnapshot {
    pub(super) facts: app_window::Facts,
    pub(super) focus: Option<Focus>,
    pub(super) selections: Vec<(interaction::Id, crate::selection::Selection)>,
    pub(super) tables: interaction::table::Snapshot,
}

impl Window {
    pub(super) fn new(facts: app_window::Facts, draft_limit: usize) -> Self {
        Self {
            facts,
            invalidation: Some(response::effect::Invalidation::Rebuild),
            projected_revision: None,
            desired_presentation_epoch: app_window::PresentationEpoch::initial(),
            acknowledged_presentation_epoch: None,
            cursor: Cursor::default(),
            focus: None,
            menu_restore_focus: None,
            file_dialog: None,
            feedback: feedback::Stack::default(),
            interaction: interaction::Interaction::new(draft_limit),
        }
    }

    pub(super) fn restore(snapshot: WindowSnapshot, draft_limit: usize) -> Self {
        let mut interaction = interaction::Interaction::new(draft_limit);
        interaction.selections_mut().restore(snapshot.selections);
        interaction.tables_mut().restore(snapshot.tables);
        Self {
            facts: snapshot.facts,
            invalidation: Some(response::effect::Invalidation::Rebuild),
            projected_revision: None,
            desired_presentation_epoch: app_window::PresentationEpoch::initial(),
            acknowledged_presentation_epoch: None,
            cursor: Cursor::default(),
            focus: snapshot.focus,
            menu_restore_focus: None,
            file_dialog: None,
            feedback: feedback::Stack::default(),
            interaction,
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

    pub(crate) fn invalidation(&self) -> Option<response::effect::Invalidation> {
        self.invalidation
    }

    pub(crate) fn projected_revision(&self) -> Option<state::Revision> {
        self.projected_revision
    }

    pub(crate) fn desired_presentation_epoch(&self) -> app_window::PresentationEpoch {
        self.desired_presentation_epoch
    }

    #[cfg(test)]
    pub(crate) fn acknowledged_presentation_epoch(&self) -> Option<app_window::PresentationEpoch> {
        self.acknowledged_presentation_epoch
    }

    pub fn cursor(&self) -> pointer::Cursor {
        self.cursor.value()
    }

    pub fn focus(&self) -> Option<Focus> {
        self.focus
    }

    pub(super) fn command_focus(&self) -> Option<Focus> {
        if let Some(palette) = self.interaction.command_palette() {
            return palette.captured_focus();
        }

        self.menu_restore_focus.or(self.focus).or_else(|| {
            self.interaction
                .text_input()
                .target()
                .and_then(Focus::from_text_target)
        })
    }

    pub(crate) fn interaction(&self) -> &interaction::Interaction {
        &self.interaction
    }

    pub fn feedback(&self) -> Option<(feedback::Severity, &str)> {
        self.feedback.winner()
    }
}

impl Default for Cursor {
    fn default() -> Self {
        Self::Synced(pointer::Cursor::Default)
    }
}

impl Cursor {
    fn value(self) -> pointer::Cursor {
        match self {
            Self::Synced(cursor) | Self::Pending(cursor) => cursor,
        }
    }

    fn set(&mut self, next: pointer::Cursor) -> bool {
        if self.value() == next {
            return false;
        }
        *self = Self::Pending(next);
        true
    }

    fn take_pending(&mut self) -> Option<pointer::Cursor> {
        let Self::Pending(cursor) = *self else {
            return None;
        };
        *self = Self::Synced(cursor);
        Some(cursor)
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
            selections: Vec::new(),
            tables: interaction::table::Snapshot::default(),
        }
    }

    pub(super) fn from_window(window: &Window) -> Self {
        Self {
            facts: window.facts.clone(),
            focus: window.focus,
            selections: window.interaction.selections().snapshot(),
            tables: window.interaction.tables().snapshot(),
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
        self.request_invalidation(id, response::effect::Invalidation::Rebuild)
    }

    pub fn report_feedback(
        &mut self,
        id: app_window::Id,
        severity: feedback::Severity,
        message: impl std::fmt::Display,
    ) -> bool {
        let Some(window) = self.window_mut(id) else {
            return false;
        };
        if !window.feedback.report(severity, message) {
            return false;
        }
        self.request_redraw(id);
        true
    }

    pub fn clear_feedback(&mut self, id: app_window::Id, severity: feedback::Severity) -> bool {
        let Some(window) = self.window_mut(id) else {
            return false;
        };
        if !window.feedback.clear(severity) {
            return false;
        }
        self.request_redraw(id);
        true
    }

    pub fn clear_all_feedback(&mut self, id: app_window::Id) -> bool {
        let Some(window) = self.window_mut(id) else {
            return false;
        };
        if !window.feedback.clear_all() {
            return false;
        }
        self.request_redraw(id);
        true
    }

    pub(crate) fn request_invalidation(
        &mut self,
        id: app_window::Id,
        invalidation: response::effect::Invalidation,
    ) -> bool {
        let Some(window) = self.window_mut(id) else {
            return false;
        };
        let previous = window.invalidation;
        window.desired_presentation_epoch = window.desired_presentation_epoch.next();
        window.invalidation =
            Some(previous.map_or(invalidation, |previous| previous.max(invalidation)));
        previous != window.invalidation
    }

    pub(crate) fn retry_invalidation(
        &mut self,
        id: app_window::Id,
        invalidation: response::effect::Invalidation,
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

    pub(crate) fn mark_projected(&mut self, id: app_window::Id, revision: state::Revision) -> bool {
        let Some(window) = self.window_mut(id) else {
            return false;
        };
        let changed = window.projected_revision != Some(revision);
        window.projected_revision = Some(revision);
        changed
    }

    pub(crate) fn acknowledge_presentation(
        &mut self,
        id: app_window::Id,
        epoch: app_window::PresentationEpoch,
    ) -> bool {
        let Some(window) = self.window_mut(id) else {
            return false;
        };
        if window
            .acknowledged_presentation_epoch
            .is_some_and(|acknowledged| acknowledged >= epoch)
        {
            return false;
        }
        window.acknowledged_presentation_epoch = Some(epoch);
        true
    }

    pub(crate) fn set_cursor(&mut self, id: app_window::Id, cursor: pointer::Cursor) -> bool {
        let Some(window) = self.window_mut(id) else {
            return false;
        };
        window.cursor.set(cursor)
    }

    pub(crate) fn take_cursor_updates(&mut self) -> Vec<pointer::Update> {
        self.windows
            .iter_mut()
            .filter_map(|window| {
                let id = window.id();
                window
                    .cursor
                    .take_pending()
                    .map(|cursor| pointer::Update::new(id, cursor))
            })
            .collect()
    }

    pub(crate) fn prune_removed_interaction(
        &mut self,
        id: app_window::Id,
        removed_nodes: &[super::super::composition::tree::NodeId],
        removed_elements: &[interaction::Id],
        removed_table_cells: &[crate::table::Cell],
    ) -> interaction::Pruned {
        let Some(window) = self.window_mut(id) else {
            return interaction::Pruned::default();
        };

        let pruned =
            window
                .interaction
                .prune_removed(removed_nodes, removed_elements, removed_table_cells);
        if pruned.menu_removed() {
            window.menu_restore_focus = None;
        }
        pruned
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cursor_publication_advances_from_pending_to_synced() {
        let mut session = Session::default();
        let window = session.open_window(app_window::Options::new("Cursor"));

        assert_eq!(
            session.window(window).map(Window::cursor),
            Some(pointer::Cursor::Default)
        );
        assert!(session.take_cursor_updates().is_empty());
        assert!(session.set_cursor(window, pointer::Cursor::Text));
        assert_eq!(
            session.window(window).map(Window::cursor),
            Some(pointer::Cursor::Text)
        );
        assert_eq!(
            session.take_cursor_updates(),
            vec![pointer::Update::new(window, pointer::Cursor::Text)]
        );
        assert!(session.take_cursor_updates().is_empty());
        assert!(!session.set_cursor(window, pointer::Cursor::Text));
    }

    #[test]
    fn window_feedback_is_ephemeral_ranked_session_truth() {
        let mut session = Session::default();
        let window = session.open_window(app_window::Options::new("Feedback"));
        session.clear_redraw_request(window);

        assert!(session.report_feedback(window, feedback::Severity::Info, "saved"));
        assert!(session.report_feedback(window, feedback::Severity::Warning, "offline"));
        assert!(session.report_feedback(window, feedback::Severity::Error, "save failed"));
        assert_eq!(
            session.window(window).and_then(Window::feedback),
            Some((feedback::Severity::Error, "save failed"))
        );
        assert!(session.clear_feedback(window, feedback::Severity::Error));
        assert_eq!(
            session.window(window).and_then(Window::feedback),
            Some((feedback::Severity::Warning, "offline"))
        );
        assert!(session.clear_all_feedback(window));
        assert_eq!(session.window(window).and_then(Window::feedback), None);
    }

    #[test]
    fn closing_window_destroys_its_feedback() {
        let mut session = Session::default();
        let window = session.open_window(app_window::Options::new("Feedback"));
        session.report_feedback(window, feedback::Severity::Info, "temporary");

        assert!(session.close_window(window));
        assert_eq!(session.window(window), None);
    }

    #[test]
    fn closing_window_destroys_its_text_draft_feedback() {
        let mut session = Session::default();
        let window = session.open_window(app_window::Options::new("Draft feedback"));
        let focus = Focus::text("temporary-draft");
        assert!(session.focus(window, focus));
        assert!(session.activate_text_draft(window, focus, "draft"));
        assert!(session.reject_text_input(window, focus, "invalid".to_owned()));
        assert_eq!(
            session.text_input_feedback(window, focus),
            Some((feedback::Severity::Error, "invalid"))
        );

        assert!(session.close_window(window));
        assert_eq!(session.window(window), None);
        assert_eq!(session.text_input_feedback(window, focus), None);
    }

    #[test]
    fn command_focus_has_one_surface_and_live_focus_precedence_ladder() {
        let mut session = Session::default();
        let window = session.open_window(app_window::Options::new("Command focus"));
        let draft_focus = Focus::text("draft-focus");
        assert!(session.focus(window, draft_focus));
        assert!(session.activate_text_draft(window, draft_focus, "draft"));
        assert!(session.clear_focus(window));
        assert_eq!(session.command_focus(window), Some(draft_focus));

        assert!(session.open_menu(window, interaction::Menu::new("file", "File")));
        let live_focus = Focus::text("live-focus");
        assert!(session.focus(window, live_focus));
        assert_eq!(session.command_focus(window), Some(draft_focus));
        assert!(session.close_menu(window));
        assert_eq!(session.focused(window), Some(draft_focus));

        assert!(session.open_command_palette(window));
        assert_eq!(session.command_focus(window), Some(draft_focus));
        assert!(session.focus(window, live_focus));
        assert_eq!(session.command_focus(window), Some(draft_focus));
        assert!(session.close_command_palette(window));
        assert_eq!(session.focused(window), Some(draft_focus));
    }
}
