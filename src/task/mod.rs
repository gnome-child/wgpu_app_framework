use std::future::Future;

mod executor;
mod id;
mod outcome;
mod queue;
mod sink;

pub use id::{Id, Status};
pub use outcome::Outcome;

pub(crate) use executor::Executor;
pub(crate) use queue::Queue;
pub(crate) use sink::{AnyTask, Sink};

pub struct Task<E: Send + 'static> {
    run: Box<dyn FnOnce() -> E + Send>,
}

impl<E: Send + 'static> Task<E> {
    pub fn new(run: impl FnOnce() -> E + Send + 'static) -> Self {
        Self { run: Box::new(run) }
    }

    pub fn ready(event: E) -> Self {
        Self::new(move || event)
    }

    pub fn future(future: impl Future<Output = E> + Send + 'static) -> Self {
        Self::new(move || pollster::block_on(future))
    }

    pub(crate) fn run(self) -> E {
        (self.run)()
    }

    pub(crate) fn into_any(self) -> AnyTask {
        AnyTask::new(self)
    }
}
