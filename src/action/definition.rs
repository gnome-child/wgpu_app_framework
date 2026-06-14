use std::fmt;

use crate::Task;

use super::{Id, Invocation};

pub struct Action<T = ()> {
    id: Id,
    label: String,
    handler: Box<dyn Fn(Invocation) -> Effect<T>>,
}

pub enum Effect<T> {
    None,
    Emit(T),
    Batch(Vec<T>),
    Task(Task<T>),
}

impl<T> Action<T> {
    pub fn new(id: Id, label: impl Into<String>) -> Self {
        Self {
            id,
            label: label.into(),
            handler: Box::new(|_| Effect::None),
        }
    }

    pub fn id(&self) -> Id {
        self.id
    }

    pub fn label(&self) -> &str {
        &self.label
    }

    pub fn on_invoke(mut self, handler: impl Fn(Invocation) -> Effect<T> + 'static) -> Self {
        self.handler = Box::new(handler);
        self
    }

    pub fn emit(self, handler: impl Fn(Invocation) -> T + 'static) -> Self {
        self.on_invoke(move |invocation| Effect::Emit(handler(invocation)))
    }

    fn invoke(&self, invocation: Invocation) -> Effect<T> {
        (self.handler)(invocation)
    }
}

pub fn invoke<T>(action: &Action<T>, invocation: Invocation) -> Effect<T> {
    action.invoke(invocation)
}

impl<T: Send + 'static> Action<T> {
    pub fn task(self, handler: impl Fn(Invocation) -> Task<T> + 'static) -> Self {
        self.on_invoke(move |invocation| Effect::Task(handler(invocation)))
    }
}

impl<T> fmt::Debug for Action<T> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("Action")
            .field("id", &self.id)
            .field("label", &self.label)
            .finish_non_exhaustive()
    }
}

impl<T: fmt::Debug> fmt::Debug for Effect<T> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::None => formatter.write_str("None"),
            Self::Emit(event) => formatter.debug_tuple("Emit").field(event).finish(),
            Self::Batch(events) => formatter.debug_tuple("Batch").field(events).finish(),
            Self::Task(task) => formatter.debug_tuple("Task").field(task).finish(),
        }
    }
}

impl<T: PartialEq> PartialEq for Effect<T> {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::None, Self::None) => true,
            (Self::Emit(left), Self::Emit(right)) => left == right,
            (Self::Batch(left), Self::Batch(right)) => left == right,
            (Self::Task(_), Self::Task(_)) => false,
            _ => false,
        }
    }
}

impl<T: Eq> Eq for Effect<T> {}
