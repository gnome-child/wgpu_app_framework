use std::{
    any::{Any, TypeId},
    fmt,
    rc::Rc,
};

use super::{Id, Task};

pub(in crate::scratch) struct AnyTask {
    event_type: TypeId,
    run: Box<dyn FnOnce() -> Box<dyn Any + Send> + Send>,
}

#[derive(Clone)]
pub(in crate::scratch) struct Sink {
    spawn: Rc<dyn Fn(AnyTask) -> Option<Id>>,
}

impl AnyTask {
    pub(in crate::scratch) fn new<E: Send + 'static>(task: Task<E>) -> Self {
        Self {
            event_type: TypeId::of::<E>(),
            run: Box::new(move || Box::new(task.run()) as Box<dyn Any + Send>),
        }
    }

    pub(super) fn into_task<E: Send + 'static>(self) -> Option<Task<E>> {
        if self.event_type != TypeId::of::<E>() {
            return None;
        }

        Some(Task {
            run: Box::new(move || {
                *(self.run)()
                    .downcast::<E>()
                    .expect("task event type was checked before downcast")
            }),
        })
    }
}

impl Sink {
    pub(super) fn new(spawn: impl Fn(AnyTask) -> Option<Id> + 'static) -> Self {
        Self {
            spawn: Rc::new(spawn),
        }
    }

    pub(in crate::scratch) fn spawn(&mut self, task: AnyTask) -> Option<Id> {
        (self.spawn)(task)
    }
}

impl fmt::Debug for Sink {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.debug_struct("Sink").finish_non_exhaustive()
    }
}
