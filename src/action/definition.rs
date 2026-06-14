use std::fmt;

use super::{Id, Invocation};

pub struct Action<T = ()> {
    id: Id,
    label: String,
    handler: Box<dyn Fn(Invocation) -> Effect<T>>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Effect<T> {
    None,
    Emit(T),
    Batch(Vec<T>),
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

impl<T> fmt::Debug for Action<T> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("Action")
            .field("id", &self.id)
            .field("label", &self.label)
            .finish_non_exhaustive()
    }
}
