mod event;
mod window;

pub use event::{Event, WindowEvent};
pub use window::Window;

use super::{Error, session, shell, state::State, window as app_window};
use crate::animation;

pub struct Host<M: State, E: Send + 'static = ()> {
    shell: shell::Shell<M, E>,
    windows: Vec<Window>,
    presentations: Vec<shell::Presentation>,
    requests: Vec<session::Request>,
    needs_poll: bool,
    animation_schedule: animation::Schedule,
}

impl<M: State, E: Send + 'static> Host<M, E> {
    pub fn new(shell: shell::Shell<M, E>) -> Self {
        Self {
            shell,
            windows: Vec::new(),
            presentations: Vec::new(),
            requests: Vec::new(),
            needs_poll: false,
            animation_schedule: animation::Schedule::Idle,
        }
    }

    pub fn shell(&self) -> &shell::Shell<M, E> {
        &self.shell
    }

    pub fn shell_mut(&mut self) -> &mut shell::Shell<M, E> {
        &mut self.shell
    }

    pub fn into_shell(self) -> shell::Shell<M, E> {
        self.shell
    }

    pub fn start(&mut self) -> Result<shell::Work, Error> {
        self.handle_event(Event::Started)
    }

    pub fn poll(&mut self) -> Result<shell::Work, Error> {
        self.handle_event(Event::Poll)
    }

    pub fn handle_event(&mut self, event: Event) -> Result<shell::Work, Error> {
        let Some(event) = self.shell_event_for(event) else {
            return Ok(self.drain());
        };

        self.handle_shell_event(event)
    }

    fn handle_shell_event(&mut self, event: shell::Event) -> Result<shell::Work, Error> {
        let work = self.shell.handle_event(event)?;
        self.apply_work(&work);
        Ok(work)
    }

    pub fn drain(&mut self) -> shell::Work {
        let work = self.shell.drain_immediate();
        self.apply_work(&work);
        work
    }

    pub fn windows(&self) -> &[Window] {
        &self.windows
    }

    pub fn window(&self, id: app_window::Id) -> Option<&Window> {
        self.windows.iter().find(|window| window.id() == id)
    }

    pub fn presentations(&self) -> &[shell::Presentation] {
        &self.presentations
    }

    pub fn presentation(&self, window: app_window::Id) -> Option<&shell::Presentation> {
        self.presentations
            .iter()
            .find(|presentation| presentation.window() == window)
    }

    pub fn requests(&self) -> &[session::Request] {
        &self.requests
    }

    pub fn needs_poll(&self) -> bool {
        self.needs_poll
    }

    fn shell_event_for(&self, event: Event) -> Option<shell::Event> {
        match event {
            Event::Started => Some(shell::Event::Started),
            Event::Window { window, event } => {
                if self.window(window).is_none() {
                    log::debug!("dropping event for unknown host window: {window:?}");
                    return None;
                }
                Some(event.into_shell_event(window))
            }
            Event::Popup {
                parent,
                popup,
                event,
            } => {
                if self.window(parent).is_none() {
                    log::debug!("dropping popup event for unknown host window: {parent:?}");
                    return None;
                }
                Some(event.into_popup_shell_event(parent, popup))
            }
            Event::FilePathSelected { window, path } => {
                let Some(window) = self.window(window) else {
                    log::warn!("dropping file dialog result for unknown host window: {window:?}");
                    return None;
                };
                Some(shell::Event::FilePathSelected {
                    window: window.id(),
                    path,
                })
            }
            Event::Poll => Some(shell::Event::Poll),
        }
    }

    fn apply_work(&mut self, work: &shell::Work) {
        for closed in work.closed_windows() {
            log::debug!("host removing closed window: {closed:?}");
            self.windows.retain(|entry| entry.id() != *closed);
            self.presentations
                .retain(|presentation| presentation.window() != *closed);
        }

        for window in work.opened_windows() {
            if let Some(entry) = self
                .windows
                .iter_mut()
                .find(|entry| entry.id() == window.id())
            {
                log::debug!(
                    "host updating window {:?}: title={:?}, size={:?}",
                    window.id(),
                    window.title(),
                    window.size()
                );
                entry.update(window.facts());
                continue;
            }

            log::debug!(
                "host tracking opened window {:?}: title={:?}, size={:?}",
                window.id(),
                window.title(),
                window.size()
            );
            self.windows.push(Window::new(window.facts().clone()));
        }

        for presentation in work.presentations() {
            if let Some(window) = self
                .windows
                .iter_mut()
                .find(|window| window.id() == presentation.window())
            {
                window.set_size(presentation.size());
            }

            self.presentations
                .retain(|entry| entry.window() != presentation.window());
            self.presentations.push(presentation.clone());
        }

        self.requests = work.requests().to_vec();
        self.needs_poll = work.needs_poll();
        self.animation_schedule = work.animation_schedule();
    }
}
