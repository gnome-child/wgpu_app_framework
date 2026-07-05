use super::{host, session, shell, state::State};

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
pub(super) use runner::file_dialog_selected;
pub use runner::{Runner, run};

pub struct Platform<M: State, E: Send + 'static = (), B = ()> {
    host: host::Host<M, E>,
    backend: B,
    active_requests: Vec<session::Request>,
    poll_scheduled: bool,
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
            poll_scheduled: false,
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

        let work = self.host.handle_event(event).map_err(Error::Framework)?;
        self.apply_work(context, &work).map_err(Error::Backend)
    }

    fn apply_work(
        &mut self,
        context: &mut B::Context<'_>,
        work: &shell::Work,
    ) -> Result<(), B::Error> {
        for window in work.closed_windows() {
            self.backend.close_window(context, *window)?;
        }

        for window in work.opened_windows() {
            self.backend
                .open_window(context, &Window::from_shell(window))?;
        }

        for presentation in work.presentations() {
            self.backend.present(context, presentation)?;
        }

        self.sync_requests(context, work.requests())?;
        self.sync_poll(context, work.needs_poll())?;

        Ok(())
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

            self.backend.request(context, *request)?;
            self.active_requests.push(*request);
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

        self.backend.schedule_poll(context)?;
        self.poll_scheduled = true;
        Ok(())
    }
}
