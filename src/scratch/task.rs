use std::{
    any::{Any, TypeId},
    cell::RefCell,
    collections::{HashMap, VecDeque},
    fmt,
    future::Future,
    rc::Rc,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Id(u64);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Status {
    Pending,
    Canceled,
    Completed,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Outcome {
    id: Id,
    status: Status,
    changed_state: bool,
}

pub struct Task<E: Send + 'static> {
    run: Box<dyn FnOnce() -> E + Send>,
}

pub(super) struct Queue<E: Send + 'static> {
    inner: Rc<RefCell<Inner<E>>>,
}

struct Inner<E: Send + 'static> {
    next_id: u64,
    tasks: VecDeque<Entry<E>>,
    completions: VecDeque<Completion<E>>,
    statuses: HashMap<Id, Status>,
}

pub(super) struct AnyTask {
    event_type: TypeId,
    run: Box<dyn FnOnce() -> Box<dyn Any + Send> + Send>,
}

#[derive(Clone)]
pub(super) struct Sink {
    spawn: Rc<dyn Fn(AnyTask) -> Option<Id>>,
}

struct Entry<E: Send + 'static> {
    id: Id,
    task: Task<E>,
}

struct Completion<E: Send + 'static> {
    id: Id,
    event: E,
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

    pub(super) fn run(self) -> E {
        (self.run)()
    }

    pub(super) fn into_any(self) -> AnyTask {
        AnyTask {
            event_type: TypeId::of::<E>(),
            run: Box::new(move || Box::new(self.run()) as Box<dyn Any + Send>),
        }
    }
}

impl Outcome {
    pub(super) fn completed(id: Id, changed_state: bool) -> Self {
        Self {
            id,
            status: Status::Completed,
            changed_state,
        }
    }

    pub fn id(self) -> Id {
        self.id
    }

    pub fn status(self) -> Status {
        self.status
    }

    pub fn changed_state(self) -> bool {
        self.changed_state
    }
}

impl<E: Send + 'static> Default for Queue<E> {
    fn default() -> Self {
        Self {
            inner: Rc::new(RefCell::new(Inner::default())),
        }
    }
}

impl<E: Send + 'static> Queue<E> {
    pub(super) fn sink(&self) -> Sink {
        let inner = Rc::clone(&self.inner);
        Sink {
            spawn: Rc::new(move |task| {
                let task = task.into_task::<E>()?;
                Some(inner.borrow_mut().push(task))
            }),
        }
    }

    pub(super) fn push(&mut self, task: Task<E>) -> Id {
        self.inner.borrow_mut().push(task)
    }

    pub(super) fn pop(&mut self) -> Option<(Id, Task<E>)> {
        self.inner.borrow_mut().pop()
    }

    pub(super) fn run_next(&mut self) -> Option<Id> {
        let (id, task) = self.pop()?;
        let event = task.run();
        let mut inner = self.inner.borrow_mut();
        inner.complete(id);
        inner.push_completion(id, event);
        Some(id)
    }

    pub(super) fn pop_completion(&mut self) -> Option<(Id, E)> {
        self.inner.borrow_mut().pop_completion()
    }

    pub(super) fn len(&self) -> usize {
        self.inner.borrow().len()
    }

    pub(super) fn completions_len(&self) -> usize {
        self.inner.borrow().completions_len()
    }

    pub(super) fn clear(&mut self) {
        self.inner.borrow_mut().clear();
    }

    pub(super) fn cancel(&mut self, id: Id) -> bool {
        self.inner.borrow_mut().cancel(id)
    }

    pub(super) fn complete(&mut self, id: Id) {
        self.inner.borrow_mut().complete(id);
    }

    pub(super) fn status(&self, id: Id) -> Option<Status> {
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

impl AnyTask {
    fn into_task<E: Send + 'static>(self) -> Option<Task<E>> {
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
    pub(super) fn spawn(&mut self, task: AnyTask) -> Option<Id> {
        (self.spawn)(task)
    }
}

impl fmt::Debug for Sink {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.debug_struct("Sink").finish_non_exhaustive()
    }
}
