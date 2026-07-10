use std::{
    cell::RefCell,
    collections::{HashMap, VecDeque},
    rc::Rc,
};

use super::{Id, Sink, Status, Task};

pub(crate) struct Queue<E: Send + 'static> {
    inner: Rc<RefCell<Inner<E>>>,
}

struct Inner<E: Send + 'static> {
    next_id: u64,
    tasks: VecDeque<Entry<E>>,
    completions: VecDeque<Completion<E>>,
    statuses: HashMap<Id, Status>,
}

struct Entry<E: Send + 'static> {
    id: Id,
    task: Task<E>,
}

struct Completion<E: Send + 'static> {
    id: Id,
    event: E,
}

impl<E: Send + 'static> Default for Queue<E> {
    fn default() -> Self {
        Self {
            inner: Rc::new(RefCell::new(Inner::default())),
        }
    }
}

impl<E: Send + 'static> Queue<E> {
    pub(crate) fn sink(&self) -> Sink {
        let inner = Rc::clone(&self.inner);
        Sink::new(move |task| {
            let task = task.into_task::<E>()?;
            Some(inner.borrow_mut().push(task))
        })
    }

    pub(crate) fn pop(&mut self) -> Option<(Id, Task<E>)> {
        self.inner.borrow_mut().pop()
    }

    pub(crate) fn run_next(&mut self) -> Option<Id> {
        let (id, task) = self.pop()?;
        let event = task.run();
        self.accept_completion(id, event);
        Some(id)
    }

    pub(crate) fn accept_completion(&mut self, id: Id, event: E) -> bool {
        self.inner.borrow_mut().accept_completion(id, event)
    }

    pub(crate) fn pop_completion(&mut self) -> Option<(Id, E)> {
        self.inner.borrow_mut().pop_completion()
    }

    pub(crate) fn len(&self) -> usize {
        self.inner.borrow().len()
    }

    pub(crate) fn completions_len(&self) -> usize {
        self.inner.borrow().completions_len()
    }

    pub(crate) fn clear(&mut self) {
        self.inner.borrow_mut().clear();
    }

    pub(crate) fn cancel(&mut self, id: Id) -> bool {
        self.inner.borrow_mut().cancel(id)
    }

    pub(crate) fn status(&self, id: Id) -> Option<Status> {
        self.inner.borrow().status(id)
    }
}

impl<E: Send + 'static> Default for Inner<E> {
    fn default() -> Self {
        Self {
            next_id: 0,
            tasks: VecDeque::new(),
            completions: VecDeque::new(),
            statuses: HashMap::new(),
        }
    }
}

impl<E: Send + 'static> Inner<E> {
    fn push(&mut self, task: Task<E>) -> Id {
        let id = self.allocate_id();
        self.statuses.insert(id, Status::Pending);
        self.tasks.push_back(Entry { id, task });
        log::debug!("queued task {id:?}");
        id
    }

    fn pop(&mut self) -> Option<(Id, Task<E>)> {
        while let Some(entry) = self.tasks.pop_front() {
            if self.status(entry.id) == Some(Status::Pending) {
                log::debug!("running task {:?}", entry.id);
                return Some((entry.id, entry.task));
            }
            log::debug!(
                "discarding queued task {:?} with status {:?}",
                entry.id,
                self.status(entry.id)
            );
        }

        None
    }

    fn push_completion(&mut self, id: Id, event: E) {
        log::debug!("queued task completion {id:?}");
        self.completions.push_back(Completion { id, event });
    }

    fn pop_completion(&mut self) -> Option<(Id, E)> {
        self.completions.pop_front().map(|completion| {
            log::debug!("dispatching task completion {:?}", completion.id);
            (completion.id, completion.event)
        })
    }

    fn len(&self) -> usize {
        self.statuses
            .values()
            .filter(|status| **status == Status::Pending)
            .count()
    }

    fn completions_len(&self) -> usize {
        self.completions.len()
    }

    fn clear(&mut self) {
        let pending = self.len();
        for status in self.statuses.values_mut() {
            if *status == Status::Pending {
                *status = Status::Canceled;
            }
        }
        self.tasks.clear();
        self.completions.clear();
        if pending > 0 {
            log::debug!("cleared task queue; canceled {pending} pending tasks");
        }
    }

    fn cancel(&mut self, id: Id) -> bool {
        if self.status(id) != Some(Status::Pending) {
            log::debug!(
                "ignored task cancel for {:?}; status={:?}",
                id,
                self.status(id)
            );
            return false;
        }

        self.statuses.insert(id, Status::Canceled);
        log::debug!("canceled task {id:?}");
        true
    }

    fn accept_completion(&mut self, id: Id, event: E) -> bool {
        if self.status(id) != Some(Status::Pending) {
            log::debug!(
                "discarding task completion {id:?}; status={:?}",
                self.status(id)
            );
            return false;
        }
        self.statuses.insert(id, Status::Completed);
        log::debug!("completed task {id:?}");
        self.push_completion(id, event);
        true
    }

    fn status(&self, id: Id) -> Option<Status> {
        self.statuses.get(&id).copied()
    }

    fn allocate_id(&mut self) -> Id {
        let id = Id(self.next_id);
        self.next_id += 1;
        id
    }
}
