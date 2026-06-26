use std::any::TypeId;
use std::fmt;
use std::marker::PhantomData;

use crate::{ui, window};

use super::{
    Command, Key, Target,
    args::{self, Args, Error as ArgsError},
    binding, target,
};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Context {
    window: window::Id,
    scope: Scope,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Scope {
    Current,
    Focused,
    Captured,
    Window,
    Path(ui::Path),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Source {
    Pointer,
    Keyboard,
    Programmatic,
    Shortcut,
}

#[derive(Debug, Clone, PartialEq)]
pub(crate) struct Raw {
    command: Key,
    target: target::Kind,
    source: Source,
    context: Context,
    args: args::Raw,
    origin: Option<ui::Path>,
    repeated: bool,
}

pub struct Call<C: Command> {
    args: C::Args,
    target: target::Kind,
    source: Source,
    window: Option<window::Id>,
    scope: Scope,
    origin: Option<ui::Path>,
    repeated: bool,
    _command: PhantomData<fn() -> C>,
}

pub struct Any {
    call: Box<dyn ErasedCall>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Invocation<C = Any> {
    command: Key,
    source: Source,
    context: Context,
    origin: Option<ui::Path>,
    repeated: bool,
    _command: PhantomData<fn() -> C>,
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

impl Raw {
    pub(crate) fn from_route(route: binding::Route, source: Source, context: Context) -> Self {
        Self {
            command: route.command(),
            target: route.target(),
            source,
            context,
            args: args::Raw::None,
            origin: None,
            repeated: false,
        }
    }

    #[cfg(test)]
    pub(crate) fn from_key(command: Key, source: Source, context: Context) -> Self {
        Self {
            command,
            target: target::Kind::command(command),
            source,
            context,
            args: args::Raw::None,
            origin: None,
            repeated: false,
        }
    }

    pub(crate) fn with_args(mut self, args: args::Raw) -> Self {
        self.args = args;
        self
    }

    pub(crate) fn with_origin(mut self, origin: ui::Path) -> Self {
        self.origin = Some(origin);
        self
    }

    pub(crate) fn with_repeated(mut self, repeated: bool) -> Self {
        self.repeated = repeated;
        self
    }

    pub(crate) fn with_context(mut self, context: Context) -> Self {
        self.context = context;
        self
    }

    pub(crate) fn command(&self) -> Key {
        self.command
    }

    pub(crate) fn target(&self) -> target::Kind {
        self.target
    }

    pub(crate) fn source(&self) -> Source {
        self.source
    }

    pub(crate) fn context(&self) -> &Context {
        &self.context
    }

    pub(crate) fn args(&self) -> &args::Raw {
        &self.args
    }

    pub(crate) fn origin(&self) -> Option<&ui::Path> {
        self.origin.as_ref()
    }

    pub(crate) fn repeated(&self) -> bool {
        self.repeated
    }
}

impl<C: Command> Call<C> {
    pub fn new<TTarget>(args: C::Args) -> Result<Self, ArgsError>
    where
        TTarget: Target<C> + 'static,
    {
        Self::new_with_target(args, target::Kind::of_type::<TTarget>())
    }

    pub(crate) fn new_with_target(args: C::Args, target: target::Kind) -> Result<Self, ArgsError> {
        args.validate()?;

        Ok(Self {
            args,
            target,
            source: Source::Programmatic,
            window: None,
            scope: Scope::Current,
            origin: None,
            repeated: false,
            _command: PhantomData,
        })
    }

    pub fn for_window<TTarget>(args: C::Args, window: window::Id) -> Result<Self, ArgsError>
    where
        TTarget: Target<C> + 'static,
    {
        Self::new::<TTarget>(args).map(|call| call.with_window(window).with_scope(Scope::Window))
    }

    pub fn for_context<TTarget>(args: C::Args, context: Context) -> Result<Self, ArgsError>
    where
        TTarget: Target<C> + 'static,
    {
        Self::for_context_with_target(args, context, target::Kind::of_type::<TTarget>())
    }

    pub(crate) fn for_context_with_target(
        args: C::Args,
        context: Context,
        target: target::Kind,
    ) -> Result<Self, ArgsError> {
        let window = context.window;
        let scope = context.scope;

        Self::new_with_target(args, target).map(|call| call.with_window(window).with_scope(scope))
    }

    pub fn for_invocation<TTarget, T>(
        args: C::Args,
        invocation: &Invocation<T>,
    ) -> Result<Self, ArgsError>
    where
        TTarget: Target<C> + 'static,
    {
        let mut call = Self::for_context::<TTarget>(args, invocation.context().clone())?
            .with_source(invocation.source());

        if let Some(origin) = invocation.origin().cloned() {
            call = call.with_origin(origin);
        }
        call = call.with_repeated(invocation.repeated());

        Ok(call)
    }

    pub fn with_source(mut self, source: Source) -> Self {
        self.source = source;
        self
    }

    pub fn with_scope(mut self, scope: Scope) -> Self {
        self.scope = scope;
        self
    }

    pub fn with_window(mut self, window: window::Id) -> Self {
        self.window = Some(window);
        self
    }

    pub fn with_origin(mut self, origin: ui::Path) -> Self {
        self.origin = Some(origin);
        self
    }

    pub fn with_repeated(mut self, repeated: bool) -> Self {
        self.repeated = repeated;
        self
    }

    pub fn args(&self) -> &C::Args {
        &self.args
    }

    pub fn target(&self) -> target::Kind {
        self.target
    }

    pub fn source(&self) -> Source {
        self.source
    }

    pub fn window_id(&self) -> Option<window::Id> {
        self.window
    }

    pub fn scope(&self) -> &Scope {
        &self.scope
    }

    pub fn requested_context(&self) -> Option<Context> {
        self.window
            .map(|window| Context::with_scope(window, self.scope.clone()))
    }

    pub fn origin(&self) -> Option<&ui::Path> {
        self.origin.as_ref()
    }

    pub(crate) fn into_raw(self) -> Raw {
        let context = self
            .requested_context()
            .expect("command call must be resolved before converting to raw call");
        let mut request = Raw::from_route(
            binding::Route::new(Key::of::<C>(), self.target),
            self.source,
            context,
        )
        .with_args(self.args.into_raw());

        if let Some(origin) = self.origin {
            request = request.with_origin(origin);
        }
        request = request.with_repeated(self.repeated);

        request
    }

    pub(crate) fn into_parts(
        self,
        fallback_window: window::Id,
    ) -> (
        C::Args,
        target::Kind,
        Source,
        Context,
        Option<ui::Path>,
        bool,
    ) {
        (
            self.args,
            self.target,
            self.source,
            Context::with_scope(self.window.unwrap_or(fallback_window), self.scope),
            self.origin,
            self.repeated,
        )
    }
}

impl Any {
    pub(crate) fn new<C: Command>(call: Call<C>) -> Self {
        Self {
            call: Box::new(call),
        }
    }

    pub(crate) fn from_raw<C: Command>(raw: Raw) -> Result<Self, ArgsError> {
        let Raw {
            command,
            target,
            source,
            context,
            args,
            origin,
            repeated,
        } = raw;

        debug_assert_eq!(command, Key::of::<C>());

        let args = C::Args::from_raw(args)?;
        let mut call =
            Call::<C>::for_context_with_target(args, context, target)?.with_source(source);
        if let Some(origin) = origin {
            call = call.with_origin(origin);
        }
        call = call.with_repeated(repeated);

        Ok(Self::new(call))
    }

    pub(crate) fn is<C: Command>(&self) -> bool {
        self.call.command_type() == TypeId::of::<C>()
    }

    pub(crate) fn command(&self) -> Key {
        self.call.command()
    }

    pub(crate) fn target(&self) -> target::Kind {
        self.call.target()
    }

    pub(crate) fn source(&self) -> Source {
        self.call.source()
    }

    pub(crate) fn context(&self) -> Option<Context> {
        self.call.context()
    }

    pub(crate) fn origin(&self) -> Option<&ui::Path> {
        self.call.origin()
    }

    pub(crate) fn repeated(&self) -> bool {
        self.call.repeated()
    }

    pub(crate) fn with_fallback_window(self, window: window::Id) -> Self {
        Self {
            call: self.call.with_fallback_window(window),
        }
    }

    pub(crate) fn into_call<C: Command>(self) -> Result<Call<C>, Self> {
        if !self.is::<C>() {
            return Err(self);
        }

        let target = self.target();
        let source = self.source();
        let window = self.call.window_id();
        let scope = self.call.scope().clone();
        let origin = self.origin().cloned();
        let repeated = self.repeated();
        let args = self
            .call
            .into_args()
            .downcast::<C::Args>()
            .expect("erased call type id should match command args type");

        Ok(Call {
            args: *args,
            target,
            source,
            window,
            scope,
            origin,
            repeated,
            _command: PhantomData,
        })
    }
}

trait ErasedCall: Send {
    fn command_type(&self) -> TypeId;
    fn command(&self) -> Key;
    fn target(&self) -> target::Kind;
    fn source(&self) -> Source;
    fn window_id(&self) -> Option<window::Id>;
    fn scope(&self) -> &Scope;
    fn context(&self) -> Option<Context>;
    fn origin(&self) -> Option<&ui::Path>;
    fn repeated(&self) -> bool;
    fn with_fallback_window(self: Box<Self>, window: window::Id) -> Box<dyn ErasedCall>;
    fn into_args(self: Box<Self>) -> Box<dyn std::any::Any + Send>;
}

impl<C: Command> ErasedCall for Call<C> {
    fn command_type(&self) -> TypeId {
        TypeId::of::<C>()
    }

    fn command(&self) -> Key {
        Key::of::<C>()
    }

    fn target(&self) -> target::Kind {
        self.target
    }

    fn source(&self) -> Source {
        self.source
    }

    fn window_id(&self) -> Option<window::Id> {
        self.window
    }

    fn scope(&self) -> &Scope {
        &self.scope
    }

    fn context(&self) -> Option<Context> {
        self.requested_context()
    }

    fn origin(&self) -> Option<&ui::Path> {
        self.origin.as_ref()
    }

    fn repeated(&self) -> bool {
        self.repeated
    }

    fn with_fallback_window(mut self: Box<Self>, window: window::Id) -> Box<dyn ErasedCall> {
        if self.window.is_none() {
            self.window = Some(window);
        }
        self
    }

    fn into_args(self: Box<Self>) -> Box<dyn std::any::Any + Send> {
        Box::new(self.args)
    }
}

impl fmt::Debug for Any {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("Any")
            .field("command", &self.command())
            .field("target", &self.target())
            .field("source", &self.source())
            .field("context", &self.context())
            .field("origin", &self.origin())
            .field("repeated", &self.repeated())
            .finish()
    }
}

impl PartialEq for Any {
    fn eq(&self, other: &Self) -> bool {
        self.command() == other.command()
            && self.target() == other.target()
            && self.source() == other.source()
            && self.context() == other.context()
            && self.origin() == other.origin()
            && self.repeated() == other.repeated()
    }
}

impl Eq for Any {}

impl Invocation {
    pub fn new<C: Command>(source: Source, context: Context) -> Invocation<C> {
        Invocation::for_command(source, context)
    }
}

impl<C> Invocation<C> {
    pub fn with_origin(mut self, origin: ui::Path) -> Self {
        self.origin = Some(origin);
        self
    }

    pub fn with_repeated(mut self, repeated: bool) -> Self {
        self.repeated = repeated;
        self
    }

    #[cfg(test)]
    pub(crate) fn command(&self) -> Key {
        self.command
    }

    pub fn source(&self) -> Source {
        self.source
    }

    pub fn context(&self) -> &Context {
        &self.context
    }

    pub fn origin(&self) -> Option<&ui::Path> {
        self.origin.as_ref()
    }

    pub fn repeated(&self) -> bool {
        self.repeated
    }
}

impl<C: Command> Invocation<C> {
    pub fn for_command(source: Source, context: Context) -> Self {
        Self {
            command: Key::of::<C>(),
            source,
            context,
            origin: None,
            repeated: false,
            _command: PhantomData,
        }
    }
}

impl From<Raw> for Invocation {
    fn from(request: Raw) -> Self {
        Self {
            command: request.command,
            source: request.source,
            context: request.context,
            origin: request.origin,
            repeated: request.repeated,
            _command: PhantomData,
        }
    }
}

impl<C: Command> From<Call<C>> for Raw {
    fn from(call: Call<C>) -> Self {
        call.into_raw()
    }
}
