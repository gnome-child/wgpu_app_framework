use crate::{state::State, task};

use super::Shell;

impl<M: State, E: Send + 'static> Shell<M, E> {
    pub fn run_next_task(&mut self) -> Option<task::Outcome> {
        self.runtime.run_next_task()
    }

    pub fn complete_next_task(&mut self) -> Option<task::Id> {
        self.runtime.complete_next_task()
    }

    pub fn dispatch_next_task_completion(&mut self) -> Option<task::Outcome> {
        self.runtime.dispatch_next_task_completion()
    }
}
