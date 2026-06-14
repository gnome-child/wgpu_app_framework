use std::fmt;
use std::future::Future;

pub struct Task<T> {
    work: Box<dyn FnOnce() -> T + Send + 'static>,
}

impl<T> Task<T> {
    pub fn future(future: impl Future<Output = T> + Send + 'static) -> Self {
        Self {
            work: Box::new(|| pollster::block_on(future)),
        }
    }

    pub fn run(self) -> T {
        (self.work)()
    }
}

impl<T> fmt::Debug for Task<T> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.debug_struct("Task").finish_non_exhaustive()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn future_task_runs_to_app_event() {
        let task = Task::future(async { 7 });

        assert_eq!(task.run(), 7);
    }
}
