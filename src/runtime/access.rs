use super::super::{
    clipboard::Clipboard,
    composition,
    diagnostics::Diagnostics,
    session,
    state::{self, Store},
    timeline::Timeline,
    window,
};
use super::Runtime;

impl<M: state::State, E: Send + 'static, V> Runtime<M, E, V> {
    pub fn state(&self) -> &M {
        self.store.model()
    }

    pub fn store(&self) -> &Store<M> {
        &self.store
    }

    pub fn timeline(&self) -> &Timeline<M> {
        &self.timeline
    }

    pub fn session(&self) -> &session::Session {
        &self.session
    }

    pub fn composition(&self, window: window::Id) -> Option<&composition::Composition> {
        self.composition.get(window)
    }

    pub fn requests(&self) -> Vec<session::Request> {
        self.session.requests()
    }

    pub fn request_redraw(&mut self, window: window::Id) -> bool {
        self.session.request_redraw(window)
    }

    pub fn clear_redraw_request(&mut self, window: window::Id) -> bool {
        self.session.clear_redraw_request(window)
    }

    pub fn clipboard(&self) -> &Clipboard {
        &self.clipboard
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

    pub fn diagnostics(&self, window: window::Id) -> Option<&Diagnostics> {
        if !self.session.contains(window) {
            return None;
        }

        self.diagnostics.get(window)
    }

    pub fn diagnostics_mut(&mut self, window: window::Id) -> Option<&mut Diagnostics> {
        if !self.session.contains(window) {
            return None;
        }

        Some(self.diagnostics.get_mut(window))
    }

    pub(in crate::runtime) fn request_all_redraws(&mut self) {
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
