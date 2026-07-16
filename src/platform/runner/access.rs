use super::super::{Backend, Error, Events, Platform};
use super::Runner;
use crate::{host, state::State};

impl<M: State, E: Send + 'static, B: Backend> Runner<M, E, B> {
    pub fn with_platform(platform: Platform<M, E, B>) -> Self {
        Self {
            platform,
            events: Events::new(),
            started: false,
            error: None,
            executor: None,
            task_proxy: None,
        }
    }

    pub fn platform(&self) -> &Platform<M, E, B> {
        &self.platform
    }

    pub fn platform_mut(&mut self) -> &mut Platform<M, E, B> {
        &mut self.platform
    }

    pub fn events(&self) -> &Events {
        &self.events
    }

    pub fn events_mut(&mut self) -> &mut Events {
        &mut self.events
    }

    pub fn started(&self) -> bool {
        self.started
    }

    pub fn error(&self) -> Option<&Error<B::Error>> {
        self.error.as_ref()
    }

    pub fn take_error(&mut self) -> Option<Error<B::Error>> {
        self.error.take()
    }

    pub fn into_platform(self) -> Platform<M, E, B> {
        self.platform
    }

    pub fn start(&mut self) -> Result<(), Error<B::Error>>
    where
        for<'a> B::Context<'a>: Default,
    {
        if self.started {
            return Ok(());
        }

        self.platform.start()?;
        self.started = true;
        Ok(())
    }

    pub fn handle_event(&mut self, event: host::Event) -> Result<(), Error<B::Error>>
    where
        for<'a> B::Context<'a>: Default,
    {
        self.platform.handle_event(event)
    }

    pub fn emit(&mut self, event: E) -> Result<(), Error<B::Error>>
    where
        for<'a> B::Context<'a>: Default,
    {
        self.platform
            .host_mut()
            .shell_mut()
            .runtime_mut()
            .emit(event);
        self.platform.drain()
    }

    pub fn poll(&mut self) -> Result<(), Error<B::Error>>
    where
        for<'a> B::Context<'a>: Default,
    {
        self.platform.poll()
    }
}
