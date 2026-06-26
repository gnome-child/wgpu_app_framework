//! Command model sketch.
//!
//! Goal:
//! - App code names commands with types, not public ids.
//! - A command is static identity plus default metadata.
//! - A target implements `Target<C>` to say it can handle command `C`.
//! - Runtime creates calls, resolves scope to a concrete target, validates state,
//!   then constructs an invocation and calls the target.
//! - The registry owns definitions, shortcut routing, and runtime state.
//! - Internal ids/keys are derived from command types and stay private.
//!
//! Vocabulary:
//! - Command: "what operation"
//! - Args: small invocation parameters supplied by the caller, like terminal command arguments
//! - Target<C>: "this thing can perform C"
//! - Call<C>: unresolved attempt to run C
//! - Raw: undecoded runtime call packet, usually from menu/shortcut/pointer routing
//! - Any: decoded type-erased `Call<C>` for heterogeneous runtime dispatch
//! - Scope: how a call should resolve
//! - Invocation<C>: resolved execution context for C
//! - Response: output plus runtime effects
//! - Effect: work requested from runtime, including calling another command
//! - Task: deferred work returned as an effect
//! - State: runtime presentation/availability owned by the registry
//! - Presentation: effective display/hint after applying runtime state to definition defaults
//! - Erased*: private runtime adapter/storage for a typed concept
//!
//! Rule of thumb:
//! - If the caller chooses it, it is probably Args.
//! - If the resolved target already owns it, it is not Args.
//! - Large target data should cross command boundaries by handle/range/id, not by value.
//! - Use standard owned types for simple args; use named structs when the shape
//!   carries domain meaning, has multiple fields, or needs command-specific validation.

#![allow(dead_code)]

use std::any::TypeId;
use std::collections::HashMap;
use std::marker::PhantomData;

// Core traits.

pub trait Command: 'static + Sized {
    type Args: Args;
    type Output: Output;

    const NAME: &'static str;
    const DISPLAY: &'static str;

    fn hint() -> Option<&'static str> {
        None
    }

    fn repeatable() -> bool {
        false
    }
}

pub const MAX_STRING_ARG_BYTES: usize = 64 * 1024;

pub trait Args: Send + 'static {
    fn validate(&self) -> Result<(), ArgsError> {
        Ok(())
    }

    fn size_hint(&self) -> usize {
        0
    }

    fn from_raw(raw: RawArgs) -> Result<Self, ArgsError>
    where
        Self: Sized;

    fn into_raw(self) -> RawArgs;
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ArgsError {
    InvalidRaw(&'static str),
    TooLarge { max: usize, actual: usize },
    Invalid(&'static str),
}

#[derive(Debug, Clone, Default, PartialEq)]
pub enum RawArgs {
    #[default]
    None,
    Text(String),
    Bool(bool),
    Number(f64),
    Path(std::path::PathBuf),
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash)]
pub enum RawArgsKind {
    #[default]
    None,
    Text,
    Bool,
    Number,
    Path,
}

impl RawArgs {
    pub fn kind(&self) -> RawArgsKind {
        match self {
            Self::None => RawArgsKind::None,
            Self::Text(_) => RawArgsKind::Text,
            Self::Bool(_) => RawArgsKind::Bool,
            Self::Number(_) => RawArgsKind::Number,
            Self::Path(_) => RawArgsKind::Path,
        }
    }
}

impl Args for () {
    fn from_raw(raw: RawArgs) -> Result<Self, ArgsError> {
        match raw {
            RawArgs::None => Ok(()),
            _ => Err(ArgsError::InvalidRaw("expected no args")),
        }
    }

    fn into_raw(self) -> RawArgs {
        RawArgs::None
    }
}

impl Args for bool {
    fn from_raw(raw: RawArgs) -> Result<Self, ArgsError> {
        match raw {
            RawArgs::Bool(value) => Ok(value),
            _ => Err(ArgsError::InvalidRaw("expected bool args")),
        }
    }

    fn into_raw(self) -> RawArgs {
        RawArgs::Bool(self)
    }
}

impl Args for f64 {
    fn from_raw(raw: RawArgs) -> Result<Self, ArgsError> {
        match raw {
            RawArgs::Number(value) => Ok(value),
            _ => Err(ArgsError::InvalidRaw("expected number args")),
        }
    }

    fn into_raw(self) -> RawArgs {
        RawArgs::Number(self)
    }
}

impl Args for String {
    fn from_raw(raw: RawArgs) -> Result<Self, ArgsError> {
        let value = match raw {
            RawArgs::Text(value) => value,
            _ => return Err(ArgsError::InvalidRaw("expected text args")),
        };

        value.validate()?;
        Ok(value)
    }

    fn validate(&self) -> Result<(), ArgsError> {
        if self.len() > MAX_STRING_ARG_BYTES {
            return Err(ArgsError::TooLarge {
                max: MAX_STRING_ARG_BYTES,
                actual: self.len(),
            });
        }

        Ok(())
    }

    fn size_hint(&self) -> usize {
        self.len()
    }

    fn into_raw(self) -> RawArgs {
        RawArgs::Text(self)
    }
}

impl Args for std::path::PathBuf {
    fn from_raw(raw: RawArgs) -> Result<Self, ArgsError> {
        let value = match raw {
            RawArgs::Path(value) => value,
            _ => return Err(ArgsError::InvalidRaw("expected path args")),
        };

        value.validate()?;
        Ok(value)
    }

    fn validate(&self) -> Result<(), ArgsError> {
        if self.as_os_str().is_empty() {
            return Err(ArgsError::Invalid("path cannot be empty"));
        }

        Ok(())
    }

    fn into_raw(self) -> RawArgs {
        RawArgs::Path(self)
    }
}

pub trait Output: Send + 'static {}
impl Output for () {}
impl Output for String {}
impl<T: Send + 'static> Output for Option<T> {}

pub trait Target<C: Command>: 'static {
    fn state(&self, _context: &Context) -> State {
        State::available()
    }

    fn invoke(&mut self, args: C::Args, invocation: Invocation<C>) -> Response<C::Output>;
}

// Runtime packets.

pub struct Call<C: Command> {
    args: C::Args,
    target: TargetKey,
    source: Source,
    origin: Option<Path>,
    scope: Scope,
}

impl<C: Command> Call<C> {
    pub fn new<T>(args: C::Args) -> Result<Self, Rejection>
    where
        T: Target<C>,
    {
        args.validate().map_err(Rejection::InvalidArgs)?;

        Ok(Self {
            args,
            target: TargetKey::of::<T>(),
            source: Source::Programmatic,
            origin: None,
            scope: Scope::Current,
        })
    }

    pub fn with_source(mut self, source: Source) -> Self {
        self.source = source;
        self
    }

    pub fn with_origin(mut self, origin: Path) -> Self {
        self.origin = Some(origin);
        self
    }

    pub fn with_scope(mut self, scope: Scope) -> Self {
        self.scope = scope;
        self
    }

    pub fn source(&self) -> Source {
        self.source
    }

    pub fn origin(&self) -> Option<&Path> {
        self.origin.as_ref()
    }

    pub fn scope(&self) -> Scope {
        self.scope
    }
}

pub struct Invocation<C: Command> {
    source: Source,
    origin: Option<Path>,
    context: Context,
    repeated: bool,
    _command: PhantomData<C>,
}

impl<C: Command> Invocation<C> {
    pub fn source(&self) -> Source {
        self.source
    }

    pub fn origin(&self) -> Option<&Path> {
        self.origin.as_ref()
    }

    pub fn context(&self) -> Context {
        self.context
    }

    pub fn repeated(&self) -> bool {
        self.repeated
    }
}

pub struct Response<O: Output> {
    output: O,
    effects: Vec<Effect>,
}

impl<O: Output> Response<O> {
    pub fn output(output: O) -> Self {
        Self {
            output,
            effects: Vec::new(),
        }
    }

    pub fn runtime(output: O, effect: RuntimeEffect) -> Self {
        Self {
            output,
            effects: vec![Effect::Runtime(effect)],
        }
    }

    pub fn with_effect(mut self, effect: Effect) -> Self {
        self.effects.push(effect);
        self
    }

    pub fn then_call<C: Command>(mut self, call: Call<C>) -> Self {
        self.effects.push(Effect::Call(Any::new(call)));
        self
    }

    pub fn then_task(mut self, task: Task) -> Self {
        self.effects.push(Effect::Task(task));
        self
    }

    pub fn pipe<C, T>(self, source: Source, scope: Scope) -> Result<Response<()>, Rejection>
    where
        C: Command,
        T: Target<C>,
        C::Args: From<O>,
    {
        self.pipe_with::<C, T>(source, scope, C::Args::from)
    }

    pub fn pipe_with<C, T>(
        self,
        source: Source,
        scope: Scope,
        args: impl FnOnce(O) -> C::Args,
    ) -> Result<Response<()>, Rejection>
    where
        C: Command,
        T: Target<C>,
    {
        let call = Call::<C>::new::<T>(args(self.output))?
            .with_source(source)
            .with_scope(scope);

        Ok(Response {
            output: (),
            effects: self
                .effects
                .into_iter()
                .chain([Effect::Call(Any::new(call))])
                .collect(),
        })
    }

    pub fn into_parts(self) -> (O, Vec<Effect>) {
        (self.output, self.effects)
    }
}

impl Response<()> {
    pub fn none() -> Self {
        Self::output(())
    }

    pub fn task(task: Task) -> Self {
        Self {
            output: (),
            effects: vec![Effect::Task(task)],
        }
    }
}

pub enum Effect {
    None,
    Runtime(RuntimeEffect),
    Batch(Vec<Effect>),
    Call(Any),
    Task(Task),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RuntimeEffect {
    Notify(&'static str),
    RequestRedraw,
    ClipboardWrite(String),
}

pub struct Task {
    run: Box<dyn FnOnce() -> Result<Response<()>, Rejection> + Send + 'static>,
}

impl Task {
    pub fn new(run: impl FnOnce() -> Result<Response<()>, Rejection> + Send + 'static) -> Self {
        Self { run: Box::new(run) }
    }

    pub fn run(self) -> Result<Response<()>, Rejection> {
        (self.run)()
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Rejection {
    UnknownCommand,
    InvalidArgs(ArgsError),
    Disabled,
    Running,
    UnresolvedTarget,
    TargetMismatch,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Source {
    Shortcut,
    Menu,
    Pointer,
    Programmatic,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Scope {
    Current,
    Focused,
    Captured,
    Window,
    Path(Path),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Context {
    Window(WindowId),
    Path(WindowId, Path),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Path;
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct WindowId;

// Type-erased runtime internals.
//
// App code should not touch these directly. The registry/runtime needs them
// because calls, target adapters, and responses become heterogeneous once a
// shortcut/menu/pointer source asks for "whatever command this resolved to".

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
struct CommandKey(TypeId);

impl CommandKey {
    fn of<C: Command>() -> Self {
        Self(TypeId::of::<C>())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
struct TargetKey(TypeId);

impl TargetKey {
    fn of<T: 'static>() -> Self {
        Self(TypeId::of::<T>())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
struct Route {
    command: CommandKey,
    target: TargetKey,
}

impl Route {
    fn of<C, T>() -> Self
    where
        C: Command,
        T: Target<C>,
    {
        Self {
            command: CommandKey::of::<C>(),
            target: TargetKey::of::<T>(),
        }
    }
}

pub struct Raw {
    command: CommandKey,
    target: TargetKey,
    args: RawArgs,
    source: Source,
    scope: Scope,
}

pub struct Any {
    call: Box<dyn ErasedCall>,
}

impl Any {
    pub fn new<C: Command>(call: Call<C>) -> Self {
        Self {
            call: Box::new(call),
        }
    }

    pub fn command_type_id(&self) -> TypeId {
        self.call.command_type_id()
    }
}

trait ErasedCall: Send {
    fn command_type_id(&self) -> TypeId;
    fn source(&self) -> Source;
    fn origin(&self) -> Option<&Path>;
    fn scope(&self) -> Scope;
    fn into_args(self: Box<Self>) -> Box<dyn std::any::Any + Send>;
}

impl<C: Command> ErasedCall for Call<C> {
    fn command_type_id(&self) -> TypeId {
        TypeId::of::<C>()
    }

    fn source(&self) -> Source {
        self.source
    }

    fn origin(&self) -> Option<&Path> {
        self.origin.as_ref()
    }

    fn scope(&self) -> Scope {
        self.scope
    }

    fn into_args(self: Box<Self>) -> Box<dyn std::any::Any + Send> {
        Box::new(self.args)
    }
}

trait AnyInvoker: Send + Sync {
    fn invoke_any(
        &self,
        target: &mut dyn std::any::Any,
        call: Any,
        context: Context,
    ) -> Result<AnyResponse, Rejection>;
}

struct AnyResponse {
    output: Box<dyn std::any::Any + Send>,
    effects: Vec<Effect>,
}

// Registry-owned state.
//
// Static command metadata gives defaults. Runtime state projects availability
// and presentation per window/path/context without mutating the command type.

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct State {
    available: bool,
    active: bool,
    running: bool,
    display: Option<String>,
    hint: Option<String>,
}

impl State {
    pub fn available() -> Self {
        Self {
            available: true,
            active: false,
            running: false,
            display: None,
            hint: None,
        }
    }

    pub fn unavailable() -> Self {
        Self::available().with_available(false)
    }

    pub fn active() -> Self {
        Self::available().with_active(true)
    }

    pub fn running() -> Self {
        Self::available().with_running(true)
    }

    pub fn available_if(available: bool) -> Self {
        Self::available().with_available(available)
    }

    pub fn active_if(active: bool) -> Self {
        Self::available().with_active(active)
    }

    pub fn with_available(mut self, available: bool) -> Self {
        self.available = available;
        self
    }

    pub fn with_active(mut self, active: bool) -> Self {
        self.active = active;
        self
    }

    pub fn with_running(mut self, running: bool) -> Self {
        self.running = running;
        self
    }

    pub fn with_display(mut self, display: impl Into<String>) -> Self {
        self.display = Some(display.into());
        self
    }

    pub fn with_hint(mut self, hint: impl Into<String>) -> Self {
        self.hint = Some(hint.into());
        self
    }

    pub fn set_available(&mut self, available: bool) {
        self.available = available;
    }

    pub fn set_active(&mut self, active: bool) {
        self.active = active;
    }

    pub fn set_running(&mut self, running: bool) {
        self.running = running;
    }

    pub fn set_display(&mut self, display: impl Into<String>) {
        self.display = Some(display.into());
    }

    pub fn set_hint(&mut self, hint: impl Into<String>) {
        self.hint = Some(hint.into());
    }

    pub fn is_available(&self) -> bool {
        self.available
    }

    pub fn is_active(&self) -> bool {
        self.active
    }

    pub fn is_running(&self) -> bool {
        self.running
    }

    pub fn display(&self) -> Option<&str> {
        self.display.as_deref()
    }

    pub fn hint(&self) -> Option<&str> {
        self.hint.as_deref()
    }
}

impl Default for State {
    fn default() -> Self {
        Self {
            available: true,
            active: false,
            running: false,
            display: None,
            hint: None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Presentation {
    display: String,
    hint: Option<String>,
}

impl Presentation {
    pub fn display(&self) -> &str {
        &self.display
    }

    pub fn hint(&self) -> Option<&str> {
        self.hint.as_deref()
    }

    fn for_definition(definition: &Definition, state: &State) -> Self {
        Self {
            display: state
                .display
                .clone()
                .unwrap_or_else(|| definition.display.to_owned()),
            hint: state
                .hint
                .clone()
                .or_else(|| definition.hint.map(str::to_owned)),
        }
    }
}

pub struct Registry {
    definitions: HashMap<CommandKey, Definition>,
    shortcuts: HashMap<Shortcut, Route>,
    states: HashMap<(CommandKey, Context), State>,
}

impl Registry {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn define<C, T>(
        &mut self,
        configure: impl FnOnce(&mut DefinitionBuilder),
    ) -> Result<(), DefinitionError>
    where
        C: Command,
        T: Target<C>,
    {
        let key = CommandKey::of::<C>();
        let route = Route::of::<C, T>();
        if self.definitions.contains_key(&key) {
            return Err(DefinitionError::DuplicateCommand);
        }

        let mut builder = DefinitionBuilder::for_command::<C>();
        configure(&mut builder);

        for shortcut in builder.shortcuts() {
            if self.shortcuts.contains_key(shortcut) {
                return Err(DefinitionError::ShortcutConflict(*shortcut));
            }
        }

        let definition = builder.finish::<C>(key, route.target);
        for shortcut in &definition.shortcuts {
            self.shortcuts.insert(*shortcut, route);
        }
        self.definitions.insert(key, definition);

        Ok(())
    }

    pub fn invoke_on<C, T>(
        &self,
        target: &mut T,
        call: Call<C>,
        context: Context,
    ) -> Result<Response<C::Output>, Rejection>
    where
        C: Command,
        T: Target<C>,
    {
        let key = CommandKey::of::<C>();
        if !self.definitions.contains_key(&key) {
            return Err(Rejection::UnknownCommand);
        }

        let state = self.state_for_key(key, context);
        if !state.is_available() {
            return Err(Rejection::Disabled);
        }
        if state.is_running() {
            return Err(Rejection::Running);
        }

        // Scope is consumed by runtime resolution. At this point the concrete
        // context is the resolved target location.
        let _resolved_scope = call.scope;
        let invocation = Invocation {
            source: call.source,
            origin: call.origin,
            context,
            repeated: false,
            _command: PhantomData,
        };

        Ok(target.invoke(call.args, invocation))
    }

    pub fn state<C: Command>(&self, context: Context) -> State {
        self.state_for_key(CommandKey::of::<C>(), context)
    }

    pub fn stored_state<C: Command>(&self, context: Context) -> Option<&State> {
        self.states.get(&(CommandKey::of::<C>(), context))
    }

    pub fn set_state<C: Command>(&mut self, context: Context, state: State) {
        self.states.insert((CommandKey::of::<C>(), context), state);
    }

    pub fn update_state<C: Command>(&mut self, context: Context, update: impl FnOnce(&mut State)) {
        let state = self
            .states
            .entry((CommandKey::of::<C>(), context))
            .or_default();
        update(state);
    }

    pub fn presentation<C: Command>(&self, context: Context) -> Option<Presentation> {
        let key = CommandKey::of::<C>();
        let definition = self.definitions.get(&key)?;
        let state = self.state_for_key(key, context);

        Some(Presentation::for_definition(definition, &state))
    }

    fn state_for_key(&self, key: CommandKey, context: Context) -> State {
        self.states
            .get(&(key, context))
            .cloned()
            .or_else(|| {
                let Context::Path(window, _) = context else {
                    return None;
                };

                self.states.get(&(key, Context::Window(window))).cloned()
            })
            .unwrap_or_default()
    }
}

impl Default for Registry {
    fn default() -> Self {
        Self {
            definitions: HashMap::new(),
            shortcuts: HashMap::new(),
            states: HashMap::new(),
        }
    }
}

struct Definition {
    command: CommandKey,
    target: TargetKey,
    name: &'static str,
    display: &'static str,
    hint: Option<&'static str>,
    shortcuts: Vec<Shortcut>,
    repeatable: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DefinitionError {
    DuplicateCommand,
    ShortcutConflict(Shortcut),
}

pub struct DefinitionBuilder {
    display: &'static str,
    hint: Option<&'static str>,
    shortcuts: Vec<Shortcut>,
    repeatable: bool,
}

impl DefinitionBuilder {
    fn for_command<C: Command>() -> Self {
        Self {
            display: C::DISPLAY,
            hint: C::hint(),
            shortcuts: Vec::new(),
            repeatable: C::repeatable(),
        }
    }

    pub fn display(&mut self, display: &'static str) -> &mut Self {
        self.display = display;
        self
    }

    pub fn hint(&mut self, hint: &'static str) -> &mut Self {
        self.hint = Some(hint);
        self
    }

    pub fn shortcut(&mut self, shortcut: Shortcut) -> &mut Self {
        if !self.shortcuts.contains(&shortcut) {
            self.shortcuts.push(shortcut);
        }
        self
    }

    pub fn repeatable(&mut self) -> &mut Self {
        self.repeatable = true;
        self
    }

    fn shortcuts(&self) -> &[Shortcut] {
        &self.shortcuts
    }

    fn finish<C: Command>(self, command: CommandKey, target: TargetKey) -> Definition {
        Definition {
            command,
            target,
            name: C::NAME,
            display: self.display,
            hint: self.hint,
            shortcuts: self.shortcuts,
            repeatable: self.repeatable,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Shortcut {
    key: char,
    modifiers: Modifiers,
}

impl Shortcut {
    pub fn ctrl(key: char) -> Self {
        Self::new(key, Modifiers::ctrl())
    }

    pub fn ctrl_shift(key: char) -> Self {
        Self::new(key, Modifiers::ctrl_shift())
    }

    pub fn new(key: char, modifiers: Modifiers) -> Self {
        Self {
            key: key.to_ascii_lowercase(),
            modifiers,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Modifiers {
    ctrl: bool,
    shift: bool,
    alt: bool,
    meta: bool,
}

impl Modifiers {
    pub fn ctrl() -> Self {
        Self {
            ctrl: true,
            ..Self::default()
        }
    }

    pub fn ctrl_shift() -> Self {
        Self {
            ctrl: true,
            shift: true,
            ..Self::default()
        }
    }
}

impl Default for Modifiers {
    fn default() -> Self {
        Self {
            ctrl: false,
            shift: false,
            alt: false,
            meta: false,
        }
    }
}

// Macro sketch.
//
// This keeps the plain trait model visible, but makes the common case cheap.
// A procedural macro could derive nicer default DISPLAY values like "Save As"
// from `SaveAs`. A macro_rules version can use stringify! and rely on builder
// overrides for polished labels.

macro_rules! command {
    ($vis:vis $name:ident) => {
        $vis struct $name;

        impl Command for $name {
            type Args = ();
            type Output = ();

            const NAME: &'static str = stringify!($name);
            const DISPLAY: &'static str = stringify!($name);
        }
    };

    ($vis:vis $name:ident($args:ty) -> $output:ty) => {
        $vis struct $name;

        impl Command for $name {
            type Args = $args;
            type Output = $output;

            const NAME: &'static str = stringify!($name);
            const DISPLAY: &'static str = stringify!($name);
        }
    };
}

// Example app concepts.

command!(pub Save);
command!(pub SaveAs(std::path::PathBuf) -> ());
command!(pub InsertText(String) -> ());
command!(pub SelectedText(()) -> Option<String>);

struct ReplaceRangeArgs {
    range: TextRange,
    replacement: String,
}

impl Args for ReplaceRangeArgs {
    fn validate(&self) -> Result<(), ArgsError> {
        self.replacement.validate()
    }
}

struct TextRange;

struct Document {
    dirty: bool,
}

impl Target<Save> for Document {
    fn invoke(&mut self, _args: (), _invocation: Invocation<Save>) -> Response<()> {
        self.dirty = false;
        Response::runtime((), RuntimeEffect::Notify("saved"))
    }
}

impl Target<InsertText> for Document {
    fn invoke(&mut self, _text: String, _invocation: Invocation<InsertText>) -> Response<()> {
        self.dirty = true;
        Response::runtime((), RuntimeEffect::Notify("text inserted"))
    }
}

fn register_commands(registry: &mut Registry) -> Result<(), DefinitionError> {
    registry.define::<Save, Document>(|command| {
        command.shortcut(Shortcut::ctrl('s'));
    })?;

    registry.define::<SaveAs, Document>(|command| {
        command.display("Save As...");
        command.shortcut(Shortcut::ctrl_shift('s'));
    })?;

    registry.define::<InsertText, Document>(|command| {
        command.display("Insert Text");
        command.repeatable();
    })?;

    Ok(())
}

fn project_command_state(registry: &mut Registry, window: WindowId, document: &Document) {
    registry.update_state::<Save>(Context::Window(window), |state| {
        state.set_available(document.dirty);
        state.set_display(if document.dirty { "Save *" } else { "Save" });
    });

    registry.update_state::<InsertText>(Context::Window(window), |state| {
        state.set_available(true);
        state.set_hint("Insert text at the focused caret");
    });
}

fn duplicate_selection(selected: Response<Option<String>>) -> Result<Response<()>, Rejection> {
    selected.pipe_with::<InsertText, Document>(Source::Programmatic, Scope::Focused, |text| {
        text.unwrap_or_default()
    })
}

fn schedule_autosave() -> Response<()> {
    Response::task(Task::new(|| {
        let save = Call::<Save>::new::<Document>(())?
            .with_source(Source::Programmatic)
            .with_scope(Scope::Focused);

        Ok(Response::none().then_call(save))
    }))
}
