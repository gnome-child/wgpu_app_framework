use crate::{ui, window};

use super::{Id, Payload};

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
    Shortcut,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Request {
    action: Id,
    source: Source,
    target: Context,
    payload: Payload,
    origin: Option<ui::Path>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Invocation {
    action: Id,
    source: Source,
    context: Context,
    payload: Payload,
    origin: Option<ui::Path>,
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

impl Request {
    pub fn new(action: Id, source: Source, target: Context) -> Self {
        Self {
            action,
            source,
            target,
            payload: Payload::None,
            origin: None,
        }
    }

    pub fn with_payload(mut self, payload: Payload) -> Self {
        self.payload = payload;
        self
    }

    pub fn with_origin(mut self, origin: ui::Path) -> Self {
        self.origin = Some(origin);
        self
    }

    pub fn with_target(mut self, target: Context) -> Self {
        self.target = target;
        self
    }

    pub fn action(&self) -> Id {
        self.action
    }

    pub fn source(&self) -> Source {
        self.source
    }

    pub fn target(&self) -> &Context {
        &self.target
    }

    pub fn payload(&self) -> &Payload {
        &self.payload
    }

    pub fn origin(&self) -> Option<&ui::Path> {
        self.origin.as_ref()
    }
}

impl Invocation {
    pub fn new(action: Id, source: Source, context: Context) -> Self {
        Self {
            action,
            source,
            context,
            payload: Payload::None,
            origin: None,
        }
    }

    pub fn with_payload(mut self, payload: Payload) -> Self {
        self.payload = payload;
        self
    }

    pub fn with_origin(mut self, origin: ui::Path) -> Self {
        self.origin = Some(origin);
        self
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

    pub fn payload(&self) -> &Payload {
        &self.payload
    }

    pub fn origin(&self) -> Option<&ui::Path> {
        self.origin.as_ref()
    }
}

impl From<Request> for Invocation {
    fn from(request: Request) -> Self {
        Self {
            action: request.action,
            source: request.source,
            context: request.target,
            payload: request.payload,
            origin: request.origin,
        }
    }
}
