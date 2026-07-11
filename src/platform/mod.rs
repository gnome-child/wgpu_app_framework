use super::{host, pointer, runtime, session, shell, state::State, view};
use crate::animation;

mod backend;
mod error;
mod event;
mod native;
mod runner;

pub use backend::{Backend, Window};
pub use error::{Error, RunError};
pub use event::{
    Events, key, key_text, modifiers, point_from_physical, scroll_delta, size_from_physical,
};
pub use native::{Native, NativeContext, NativeError};
#[cfg(test)]
pub(crate) use runner::RunnerEvent;
#[cfg(test)]
pub(super) use runner::file_dialog_selected;
pub use runner::{Runner, run};

pub fn launch<M: State, E: Send + 'static>(
    app: runtime::Runtime<M, E, view::View>,
) -> Result<(), RunError<NativeError>> {
    run(native_shell(app))
}

pub(crate) fn native_shell<M: State, E: Send + 'static>(
    app: runtime::Runtime<M, E, view::View>,
) -> shell::Shell<M, E> {
    shell::Shell::new(app.with_system_clipboard_default())
}

pub struct Platform<M: State, E: Send + 'static = (), B = ()> {
    host: host::Host<M, E>,
    backend: B,
    active_requests: Vec<session::Request>,
    active_cursors: Vec<pointer::Update>,
    poll_scheduled: bool,
    animation_schedule: animation::Schedule,
}

impl<M: State, E: Send + 'static, B: Backend> Platform<M, E, B> {
    pub fn new(shell: shell::Shell<M, E>, backend: B) -> Self {
        Self::with_host(host::Host::new(shell), backend)
    }

    pub fn with_host(host: host::Host<M, E>, backend: B) -> Self {
        Self {
            host,
            backend,
            active_requests: Vec::new(),
            active_cursors: Vec::new(),
            poll_scheduled: false,
            animation_schedule: animation::Schedule::Idle,
        }
    }

    pub fn host(&self) -> &host::Host<M, E> {
        &self.host
    }

    pub fn host_mut(&mut self) -> &mut host::Host<M, E> {
        &mut self.host
    }

    pub fn backend(&self) -> &B {
        &self.backend
    }

    pub fn backend_mut(&mut self) -> &mut B {
        &mut self.backend
    }

    pub fn into_parts(self) -> (host::Host<M, E>, B) {
        (self.host, self.backend)
    }

    pub(crate) fn animation_schedule(&self) -> animation::Schedule {
        self.animation_schedule
    }

    pub fn start(&mut self) -> Result<(), Error<B::Error>>
    where
        for<'a> B::Context<'a>: Default,
    {
        let mut context: B::Context<'_> = Default::default();
        self.handle_event_with(&mut context, host::Event::Started)
    }

    pub fn poll(&mut self) -> Result<(), Error<B::Error>>
    where
        for<'a> B::Context<'a>: Default,
    {
        let mut context: B::Context<'_> = Default::default();
        self.handle_event_with(&mut context, host::Event::Poll)
    }

    pub fn drain(&mut self) -> Result<(), Error<B::Error>>
    where
        for<'a> B::Context<'a>: Default,
    {
        let mut context: B::Context<'_> = Default::default();
        self.drain_with(&mut context)
    }

    pub fn handle_event(&mut self, event: host::Event) -> Result<(), Error<B::Error>>
    where
        for<'a> B::Context<'a>: Default,
    {
        let mut context: B::Context<'_> = Default::default();
        self.handle_event_with(&mut context, event)
    }

    pub fn start_with(&mut self, context: &mut B::Context<'_>) -> Result<(), Error<B::Error>> {
        self.handle_event_with(context, host::Event::Started)
    }

    pub fn poll_with(&mut self, context: &mut B::Context<'_>) -> Result<(), Error<B::Error>> {
        self.handle_event_with(context, host::Event::Poll)
    }

    pub fn drain_with(&mut self, context: &mut B::Context<'_>) -> Result<(), Error<B::Error>> {
        self.sync_overlay_capabilities();
        let work = self.host.drain();
        self.apply_work(context, &work).map_err(Error::Backend)
    }

    pub fn handle_event_with(
        &mut self,
        context: &mut B::Context<'_>,
        event: host::Event,
    ) -> Result<(), Error<B::Error>> {
        if matches!(&event, host::Event::Poll) {
            self.poll_scheduled = false;
        }

        self.sync_overlay_capabilities();
        let work = self.host.handle_event(event).map_err(Error::Framework)?;
        self.apply_work(context, &work).map_err(Error::Backend)
    }

    fn apply_work(
        &mut self,
        context: &mut B::Context<'_>,
        work: &shell::Work,
    ) -> Result<(), B::Error> {
        for window in work.closed_windows() {
            log::debug!("closing backend window: {window:?}");
            self.backend.close_window(context, *window)?;
        }
        self.active_cursors
            .retain(|update| !work.closed_windows().contains(&update.window()));

        for window in work.opened_windows() {
            log::debug!(
                "opening backend window {:?}: title={:?}, size={:?}",
                window.id(),
                window.title(),
                window.size()
            );
            self.backend
                .open_window(context, &Window::from_shell(window))?;
        }

        let synchronized_popup_parents = work
            .presentations()
            .iter()
            .map(shell::Presentation::window)
            .collect::<Vec<_>>();
        for presentation in work.presentations() {
            let report = self.backend.present(context, presentation)?;
            self.host.shell_mut().runtime_mut().record_render_report(
                presentation.window(),
                presentation.revision(),
                report,
            );
        }
        if let Some(popup_presentations) = work.popup_presentations() {
            self.backend.present_overlay_popups(
                context,
                &synchronized_popup_parents,
                popup_presentations,
            )?;
        }

        for update in work.ime_updates() {
            self.backend.set_ime(context, *update)?;
        }

        self.sync_cursors(context, work.cursor_updates())?;
        self.sync_requests(context, work.requests())?;
        self.sync_poll(context, work.needs_poll())?;
        self.animation_schedule = work.animation_schedule();
        self.backend.maintain(context)?;

        Ok(())
    }

    fn sync_overlay_capabilities(&mut self) {
        let capabilities = self.backend.overlay_capabilities();
        self.host
            .shell_mut()
            .runtime_mut()
            .set_overlay_capabilities(capabilities);
    }

    fn sync_requests(
        &mut self,
        context: &mut B::Context<'_>,
        requests: &[session::Request],
    ) -> Result<(), B::Error> {
        self.active_requests
            .retain(|request| requests.contains(request));

        for request in requests {
            if self.active_requests.contains(request) {
                continue;
            }

            log::debug!("submitting backend request: {request:?}");
            self.backend.request(context, *request)?;
            self.active_requests.push(*request);
        }

        Ok(())
    }

    fn sync_cursors(
        &mut self,
        context: &mut B::Context<'_>,
        updates: &[pointer::Update],
    ) -> Result<(), B::Error> {
        for update in updates {
            if self.active_cursors.iter().any(|active| {
                active.window() == update.window() && active.cursor() == update.cursor()
            }) {
                continue;
            }

            log::debug!(
                "setting backend cursor for window {:?}: {:?}",
                update.window(),
                update.cursor()
            );
            self.backend
                .set_cursor(context, update.window(), update.cursor())?;
            self.active_cursors
                .retain(|active| active.window() != update.window());
            self.active_cursors.push(*update);
        }

        Ok(())
    }

    fn sync_poll(
        &mut self,
        context: &mut B::Context<'_>,
        needs_poll: bool,
    ) -> Result<(), B::Error> {
        if !needs_poll {
            self.poll_scheduled = false;
            return Ok(());
        }

        if self.poll_scheduled {
            return Ok(());
        }

        log::debug!("scheduling backend poll");
        self.backend.schedule_poll(context)?;
        self.poll_scheduled = true;
        Ok(())
    }
}
