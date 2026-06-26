use std::fmt;
use std::future::Future;

use super::{Command, Response, call, registry};

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash)]
pub enum Kind {
    #[default]
    None,
    Runtime,
    Batch,
    Call,
    Task,
}

pub enum Effect {
    None,
    Runtime(RuntimeEffect),
    Batch(Vec<Effect>),
    Call(call::Any),
    Task(Task),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RuntimeEffect {
    Notify(&'static str),
    RequestRedraw,
    ClipboardWrite(String),
}

pub struct Task {
    work: Box<dyn FnOnce() -> Result<Response<()>, registry::Rejection> + Send + 'static>,
}

impl Effect {
    pub fn call<C: Command>(call: call::Call<C>) -> Self {
        Self::Call(call::Any::new(call))
    }

    pub fn kind(&self) -> Kind {
        match self {
            Self::None => Kind::None,
            Self::Runtime(_) => Kind::Runtime,
            Self::Batch(_) => Kind::Batch,
            Self::Call(_) => Kind::Call,
            Self::Task(_) => Kind::Task,
        }
    }
}

impl Task {
    pub fn new(
        work: impl FnOnce() -> Result<Response<()>, registry::Rejection> + Send + 'static,
    ) -> Self {
        Self {
            work: Box::new(work),
        }
    }

    pub fn future(
        future: impl Future<Output = Result<Response<()>, registry::Rejection>> + Send + 'static,
    ) -> Self {
        Self::new(|| pollster::block_on(future))
    }

    pub fn run(self) -> Result<Response<()>, registry::Rejection> {
        (self.work)()
    }
}

impl fmt::Debug for Effect {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::None => formatter.write_str("None"),
            Self::Runtime(effect) => formatter.debug_tuple("Runtime").field(effect).finish(),
            Self::Batch(effects) => formatter.debug_tuple("Batch").field(effects).finish(),
            Self::Call(call) => formatter.debug_tuple("Call").field(call).finish(),
            Self::Task(task) => formatter.debug_tuple("Task").field(task).finish(),
        }
    }
}

impl fmt::Debug for Task {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.debug_struct("Task").finish_non_exhaustive()
    }
}

impl PartialEq for Effect {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::None, Self::None) => true,
            (Self::Runtime(left), Self::Runtime(right)) => left == right,
            (Self::Batch(left), Self::Batch(right)) => left == right,
            (Self::Call(left), Self::Call(right)) => left == right,
            (Self::Task(_), Self::Task(_)) => false,
            _ => false,
        }
    }
}

impl Eq for Effect {}
