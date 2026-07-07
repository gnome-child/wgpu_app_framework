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
        let mut inner = self.inner.borrow_mut();
        inner.complete(id);
        inner.push_completion(id, event);
        Some(id)
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
        id
    }

    fn pop(&mut self) -> Option<(Id, Task<E>)> {
        while let Some(entry) = self.tasks.pop_front() {
            if self.status(entry.id) == Some(Status::Pending) {
                return Some((entry.id, entry.task));
            }
        }

        None
    }

    fn push_completion(&mut self, id: Id, event: E) {
        self.completions.push_back(Completion { id, event });
    }

    fn pop_completion(&mut self) -> Option<(Id, E)> {
        self.completions
            .pop_front()
            .map(|completion| (completion.id, completion.event))
    }

    fn len(&self) -> usize {
        self.tasks
            .iter()
            .filter(|entry| self.status(entry.id) == Some(Status::Pending))
            .count()
    }

    fn completions_len(&self) -> usize {
        self.completions.len()
    }

    fn clear(&mut self) {
        for entry in &self.tasks {
            if self.status(entry.id) == Some(Status::Pending) {
                self.statuses.insert(entry.id, Status::Canceled);
            }
        }
        self.tasks.clear();
        self.completions.clear();
    }

    fn cancel(&mut self, id: Id) -> bool {
        if self.status(id) != Some(Status::Pending) {
            return false;
        }

        self.statuses.insert(id, Status::Canceled);
        true
    }

    fn complete(&mut self, id: Id) {
        if self.status(id) == Some(Status::Pending) {
            self.statuses.insert(id, Status::Completed);
        }
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
