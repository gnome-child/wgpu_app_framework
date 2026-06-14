use crate::{ui, window};

use super::Id;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Context {
    window: window::Id,
    scope: Scope,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Scope {
    Path(ui::Path),
    Window,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Source {
    Pointer,
    Keyboard,
    Programmatic,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Invocation {
    action: Id,
    source: Source,
    context: Context,
}

impl Context {
    pub fn window(window: window::Id) -> Self {
        Self {
            window,
            scope: Scope::Window,
        }
    }

    pub fn path(window: window::Id, path: ui::Path) -> Self {
        Self {
            window,
            scope: Scope::Path(path),
        }
    }

    pub fn with_scope(window: window::Id, scope: Scope) -> Self {
        Self { window, scope }
    }

    pub fn window_id(&self) -> window::Id {
        self.window
    }

    pub fn scope(&self) -> &Scope {
        &self.scope
    }
}

impl Invocation {
    pub fn new(action: Id, source: Source, context: Context) -> Self {
        Self {
            action,
            source,
            context,
        }
    }

    pub fn action(&self) -> Id {
        self.action
    }

    pub fn source(&self) -> Source {
        self.source
    }

    pub fn context(&self) -> &Context {
        &self.context
    }
}
