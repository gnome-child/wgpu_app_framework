use super::super::{state, task};
use super::Runtime;

impl<M: state::State, E: Send + 'static, V> Runtime<M, E, V> {
    pub fn pending_tasks(&self) -> usize {
        self.tasks.len()
    }

    pub fn pending_task_completions(&self) -> usize {
        self.tasks.completions_len()
    }

    pub fn task_status(&self, id: task::Id) -> Option<task::Status> {
        self.tasks.status(id)
    }

    pub fn cancel_task(&mut self, id: task::Id) -> bool {
        self.tasks.cancel(id)
    }

    pub fn complete_next_task(&mut self) -> Option<task::Id> {
        self.tasks.run_next()
    }

    pub fn dispatch_next_task_completion(&mut self) -> Option<task::Outcome> {
        let (id, event) = self.tasks.pop_completion()?;
        let before = self.revision();
        self.emit(event);
        Some(task::Outcome::completed(id, self.revision() != before))
    }

    pub fn run_next_task(&mut self) -> Option<task::Outcome> {
        if self.pending_task_completions() == 0 {
            self.complete_next_task()?;
        }

        self.dispatch_next_task_completion()
    }
}
