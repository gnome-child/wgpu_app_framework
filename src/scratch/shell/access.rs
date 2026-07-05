use crate::scratch::{runtime, state::State, view};

use super::Shell;

impl<M: State, E: Send + 'static> Shell<M, E> {
    pub fn new(runtime: runtime::Runtime<M, E, view::View>) -> Self {
        Self {
            runtime,
            windows: Vec::new(),
        }
    }

    pub fn runtime(&self) -> &runtime::Runtime<M, E, view::View> {
        &self.runtime
    }

    pub fn runtime_mut(&mut self) -> &mut runtime::Runtime<M, E, view::View> {
        &mut self.runtime
    }

    pub fn into_runtime(self) -> runtime::Runtime<M, E, view::View> {
        self.runtime
    }
}
