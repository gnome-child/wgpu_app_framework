use crate::{
    diagnostics, diagnostics::Diagnostics, feedback, session, state, state::Store, task,
    timeline::Timeline, window,
};

pub struct Context<'a, M: state::State> {
    store: &'a mut Store<M>,
    timeline: &'a mut Timeline<M>,
    session: &'a mut session::Session,
    diagnostics: &'a mut diagnostics::Store,
    tasks: task::Sink,
}

impl<'a, M: state::State> Context<'a, M> {
    pub(super) fn new(
        store: &'a mut Store<M>,
        timeline: &'a mut Timeline<M>,
        session: &'a mut session::Session,
        diagnostics: &'a mut diagnostics::Store,
        tasks: task::Sink,
    ) -> Self {
        Self {
            store,
            timeline,
            session,
            diagnostics,
            tasks,
        }
    }

    pub fn state(&self) -> &M {
        self.store.model()
    }

    pub fn revision(&self) -> state::Revision {
        self.store.revision()
    }

    pub fn is_dirty(&self) -> bool {
        self.store.is_dirty()
    }

    pub fn mark_saved(&mut self) {
        self.store.mark_saved();
        self.request_all_redraws();
    }

    pub fn change(&mut self, reason: state::Reason, mutate: impl FnOnce(&mut M)) -> state::Change {
        let before = self.store.prepare_snapshot();
        mutate(self.store.model_mut());
        self.timeline.record(before.into_model());
        let change = self.store.commit_retaining_current(reason);
        self.request_all_redraws();
        change
    }

    pub fn open_window(&mut self, options: window::Options) -> window::Id {
        let window = self.session.open_window(options);
        self.diagnostics.insert_window(window);
        window
    }

    pub fn close_window(&mut self, id: window::Id) -> bool {
        self.session.close_window(id)
    }

    pub fn diagnostics(&self, id: window::Id) -> Option<&Diagnostics> {
        self.session
            .contains(id)
            .then(|| self.diagnostics.get(id))
            .flatten()
    }

    pub fn diagnostics_mut(&mut self, id: window::Id) -> Option<&mut Diagnostics> {
        if !self.session.contains(id) {
            return None;
        }

        Some(self.diagnostics.get_mut(id))
    }

    pub fn request_redraw(&mut self, id: window::Id) -> bool {
        self.session.request_redraw(id)
    }

    pub fn report_feedback(
        &mut self,
        window: window::Id,
        severity: feedback::Severity,
        message: impl std::fmt::Display,
    ) -> bool {
        self.session.report_feedback(window, severity, message)
    }

    pub fn clear_feedback(&mut self, window: window::Id, severity: feedback::Severity) -> bool {
        self.session.clear_feedback(window, severity)
    }

    pub fn clear_all_feedback(&mut self, window: window::Id) -> bool {
        self.session.clear_all_feedback(window)
    }

    pub fn spawn<E: Send + 'static>(&mut self, task: task::Task<E>) -> Option<task::Id> {
        self.tasks.spawn(task.into_any())
    }

    pub fn windows(&self) -> &[session::Window] {
        self.session.windows()
    }

    fn request_all_redraws(&mut self) {
        let windows = self
            .session
            .windows()
            .iter()
            .map(session::Window::id)
            .collect::<Vec<_>>();

        for window in windows {
            self.session.request_redraw(window);
        }
    }
}
