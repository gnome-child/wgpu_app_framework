use std::collections::{HashMap, HashSet};
use std::fmt;

use crate::window;

use super::args;
use super::binding;
use super::call::{Any, Call, Context, Raw, Scope};
use super::definition::{Definition, ErasedResponse};
use super::shortcut::Shortcut;
use super::state::State;
use super::{Command, Key, Response, Target};
use super::{state, target};

/// Contributes command definitions to a registry.
///
/// Catalogs define static command metadata. They do not choose invocation context, run command
/// behavior, project command state, run tasks, or manage busy state; those decisions stay in the
/// runtime and target invocation paths.
pub trait Catalog {
    fn register(self, commands: &mut Commands);
}

/// Definition-only registration surface passed to [`Registry::commands`].
pub struct Commands {
    commands: Vec<Definition>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RegisterError {
    DuplicateCommand {
        command: &'static str,
    },
    DuplicateShortcut {
        shortcut: Shortcut,
        existing: &'static str,
        duplicate: &'static str,
    },
}

/// Why a command request was rejected by the registry.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Rejection {
    UnknownCommand {
        command: &'static str,
    },
    InvalidArgs {
        command: &'static str,
        error: args::Error,
    },
    Disabled {
        command: &'static str,
        context: Context,
    },
    Running {
        command: &'static str,
        context: Context,
    },
    UnresolvedTarget {
        command: &'static str,
    },
}

impl Commands {
    /// Adds one command definition.
    pub fn command(&mut self, command: Definition) -> &mut Self {
        self.commands.push(command);
        self
    }

    pub fn define<C, TTarget>(
        &mut self,
        configure: impl FnOnce(Definition) -> Definition,
    ) -> &mut Self
    where
        C: Command,
        TTarget: Target<C> + 'static,
    {
        self.command(configure(Definition::for_command::<C, TTarget>()))
    }

    #[cfg(test)]
    pub(crate) fn define_with_target<C: Command>(
        &mut self,
        target: target::Kind,
        configure: impl FnOnce(Definition) -> Definition,
    ) -> &mut Self {
        self.command(configure(Definition::for_target::<C>(target)))
    }

    /// Adds a collection of command definitions in iteration order.
    pub fn commands(&mut self, commands: impl IntoIterator<Item = Definition>) -> &mut Self {
        for command in commands {
            self.command(command);
        }

        self
    }
}

impl Default for Commands {
    fn default() -> Self {
        Self {
            commands: Vec::new(),
        }
    }
}

impl RegisterError {
    pub fn command_name(&self) -> &'static str {
        match self {
            Self::DuplicateCommand { command } => command,
            Self::DuplicateShortcut { duplicate, .. } => duplicate,
        }
    }
}

impl fmt::Display for RegisterError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::DuplicateCommand { command } => {
                write!(formatter, "duplicate command registration: {}", command)
            }
            Self::DuplicateShortcut {
                shortcut,
                existing,
                duplicate,
            } => write!(
                formatter,
                "duplicate shortcut registration: {} is already bound to {}; {} cannot reuse it",
                shortcut.display_label(),
                existing,
                duplicate
            ),
        }
    }
}

impl std::error::Error for RegisterError {}

impl Rejection {
    pub fn command_name(&self) -> &'static str {
        match self {
            Self::UnknownCommand { command }
            | Self::InvalidArgs { command, .. }
            | Self::Disabled { command, .. }
            | Self::Running { command, .. }
            | Self::UnresolvedTarget { command } => command,
        }
    }
}

impl fmt::Display for Rejection {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::UnknownCommand { command } => {
                write!(formatter, "unknown command: {command}")
            }
            Self::InvalidArgs { command, error } => {
                write!(
                    formatter,
                    "command {} rejected arguments: {:?}",
                    command, error
                )
            }
            Self::Disabled { command, context } => write!(
                formatter,
                "command {} is disabled for {:?}",
                command, context
            ),
            Self::Running { command, context } => {
                write!(
                    formatter,
                    "command {} is running for {:?}",
                    command, context
                )
            }
            Self::UnresolvedTarget { command } => {
                write!(formatter, "command {} has no resolved target", command)
            }
        }
    }
}

impl std::error::Error for Rejection {}

/// Stores command definitions and context-scoped command state.
#[derive(Debug)]
pub struct Registry {
    commands: HashMap<Key, Definition>,
    shortcuts: HashMap<Shortcut, binding::Route>,
    states: HashMap<(Key, Context), State>,
    running: HashSet<(Key, Context)>,
}

impl Registry {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn register(&mut self, catalog: impl Catalog) {
        self.try_register(catalog)
            .unwrap_or_else(|error| panic!("{error}"));
    }

    pub fn commands(&mut self, configure: impl FnOnce(&mut Commands)) {
        self.try_commands(configure)
            .unwrap_or_else(|error| panic!("{error}"));
    }

    pub fn define<C, TTarget>(&mut self, configure: impl FnOnce(Definition) -> Definition)
    where
        C: Command,
        TTarget: Target<C> + 'static,
    {
        self.try_define::<C, TTarget>(configure)
            .unwrap_or_else(|error| panic!("{error}"));
    }

    pub fn try_commands(
        &mut self,
        configure: impl FnOnce(&mut Commands),
    ) -> Result<(), RegisterError> {
        let mut commands = Commands::default();

        configure(&mut commands);

        self.insert_all(commands.commands)
    }

    pub fn try_define<C, TTarget>(
        &mut self,
        configure: impl FnOnce(Definition) -> Definition,
    ) -> Result<(), RegisterError>
    where
        C: Command,
        TTarget: Target<C> + 'static,
    {
        self.insert_all(vec![configure(Definition::for_command::<C, TTarget>())])
    }

    pub fn try_register(&mut self, catalog: impl Catalog) -> Result<(), RegisterError> {
        let mut commands = Commands::default();

        catalog.register(&mut commands);

        self.insert_all(commands.commands)
    }

    pub fn command<C: Command>(&self) -> Option<&Definition> {
        self.command_key(Key::of::<C>())
    }

    pub(crate) fn command_key(&self, key: Key) -> Option<&Definition> {
        self.commands.get(&key)
    }

    pub fn target<C: Command>(&self) -> target::Kind {
        self.target_key(Key::of::<C>())
    }

    pub(crate) fn target_key(&self, key: Key) -> target::Kind {
        self.commands
            .get(&key)
            .map(Definition::target)
            .unwrap_or_else(|| target::Kind::command(key))
    }

    pub(crate) fn shortcut_command(&self, shortcut: Shortcut) -> Option<binding::Route> {
        self.shortcuts.get(&shortcut).copied()
    }

    pub fn accepts_repeat<C: Command>(&self) -> bool {
        self.accepts_repeat_key(Key::of::<C>())
    }

    pub(crate) fn accepts_repeat_key(&self, key: Key) -> bool {
        self.commands
            .get(&key)
            .is_some_and(Definition::accepts_repeat)
    }

    pub fn set_state<C: Command>(&mut self, context: Context, state: State) -> bool {
        self.set_state_key(Key::of::<C>(), context, state)
    }

    pub(crate) fn set_state_key(&mut self, key: Key, context: Context, state: State) -> bool {
        if self.states.get(&(key, context.clone())) == Some(&state) {
            return false;
        }

        self.states.insert((key, context), state);
        true
    }

    pub fn clear_context_states(&mut self, window: window::Id) {
        self.states.retain(|(_, context), _| {
            context.window_id() != window || matches!(context.scope(), Scope::Window)
        });
    }

    pub fn state<C: Command>(&self, context: Context) -> State {
        self.state_key(Key::of::<C>(), context)
    }

    pub(crate) fn state_key(&self, key: Key, context: Context) -> State {
        state::with_running_overlay(
            self.configured_state_key(key, context.clone()),
            self.is_running(key, &context),
        )
    }

    pub fn configured_state<C: Command>(&self, context: Context) -> State {
        self.configured_state_key(Key::of::<C>(), context)
    }

    pub(crate) fn configured_state_key(&self, key: Key, context: Context) -> State {
        let Some(definition) = self.commands.get(&key) else {
            return State::unavailable();
        };

        self.configured_state_override_key(key, &context)
            .unwrap_or_else(|| definition.fallback_state())
    }

    pub(crate) fn configured_state_override_key(
        &self,
        key: Key,
        context: &Context,
    ) -> Option<State> {
        if let Some(state) = self.states.get(&(key, context.clone())) {
            return Some(state.clone());
        }

        if matches!(context.scope(), Scope::Path(_)) {
            let fallback = Context::window(context.window_id());

            if let Some(state) = self.states.get(&(key, fallback)) {
                return Some(state.clone());
            }
        }

        None
    }

    pub fn can_invoke<C: Command>(&self, context: Context) -> bool {
        self.can_invoke_key(Key::of::<C>(), context)
    }

    pub(crate) fn can_invoke_key(&self, key: Key, context: Context) -> bool {
        let state = self.state_key(key, context);

        state.is_available() && !state.is_running()
    }

    pub(crate) fn project_target_states<TTarget: 'static>(
        &mut self,
        target: &TTarget,
        context: Context,
    ) -> bool {
        let target_kind = target::Kind::of_type::<TTarget>();
        let states = self
            .commands
            .iter()
            .filter_map(|(key, definition)| {
                definition
                    .state_for_target(target_kind, target, &context)
                    .map(|state| (*key, state))
            })
            .collect::<Vec<_>>();

        let mut changed = false;
        for (command, state) in states {
            changed |= self.set_state_key(command, context.clone(), state);
        }

        changed
    }

    pub(crate) fn project_command_state<C, TTarget>(
        &mut self,
        target: &TTarget,
        context: Context,
    ) -> bool
    where
        C: Command,
        TTarget: Target<C>,
    {
        let command = Key::of::<C>();
        if !self.commands.contains_key(&command) {
            return false;
        }

        self.set_state_key(command, context.clone(), target.state(&context))
    }

    pub fn presentation<C: Command>(&self, context: Context) -> Option<state::Presentation> {
        self.presentation_key(Key::of::<C>(), context)
    }

    pub(crate) fn presentation_key(
        &self,
        key: Key,
        context: Context,
    ) -> Option<state::Presentation> {
        let definition = self.commands.get(&key)?;
        let state = self.state_key(key, context);
        let display = state
            .display()
            .map(str::to_owned)
            .unwrap_or_else(|| definition.display().to_owned());
        let hint = state
            .hint()
            .map(str::to_owned)
            .or_else(|| definition.hint().map(str::to_owned));

        Some(state::Presentation::new(display, hint, state))
    }

    pub(crate) fn can_execute(&self, request: &Raw) -> bool {
        self.validate(request).is_ok()
    }

    pub(crate) fn validate(&self, request: &Raw) -> Result<(), Rejection> {
        let definition = self.validate_request(request)?;
        definition
            .prepare_call(request.clone())
            .map(|_| ())
            .map_err(|error| Rejection::InvalidArgs {
                command: request.command().as_str(),
                error,
            })
    }

    pub(crate) fn prepare_call(&self, request: Raw) -> Result<Any, Rejection> {
        let command = request.command();
        let target = request.target();
        let source = request.source();
        let context = request.context().clone();
        let args = request.args().kind();
        let repeated = request.repeated();

        log::debug!(
            "command prepare_call requested command={} target={target} source={source:?} context={context:?} args={args:?} repeated={repeated}",
            command.as_str()
        );

        let definition = match self.validate_request(&request) {
            Ok(definition) => definition,
            Err(error) => {
                log::debug!(
                    "command prepare_call rejected command={} source={source:?} context={context:?}: {error}",
                    command.as_str()
                );
                return Err(error);
            }
        };

        let call = definition
            .prepare_call(request)
            .map_err(|error| Rejection::InvalidArgs {
                command: command.as_str(),
                error,
            })?;

        log::debug!(
            "command prepare_call accepted command={} source={source:?} context={context:?}",
            command.as_str()
        );

        Ok(call)
    }

    pub fn set_running<C: Command>(&mut self, context: Context, running: bool) -> bool {
        self.set_running_key(Key::of::<C>(), context, running)
    }

    pub(crate) fn set_running_key(&mut self, key: Key, context: Context, running: bool) -> bool {
        if running {
            self.running.insert((key, context))
        } else {
            self.running.remove(&(key, context))
        }
    }

    pub fn invoke_on<C, TTarget>(
        &self,
        target: &mut TTarget,
        call: Call<C>,
    ) -> Result<Response<C::Output>, Rejection>
    where
        C: Command,
        TTarget: Target<C>,
    {
        let command = Key::of::<C>();
        let Some(context) = call.requested_context() else {
            let error = Rejection::UnresolvedTarget { command: C::NAME };
            log::debug!(
                "command invoke_on rejected command={} source={:?}: {error}",
                command.as_str(),
                call.source()
            );
            return Err(error);
        };
        let (args, route, source, context, origin, repeated) = call.into_parts(context.window_id());

        log::debug!(
            "command invoke_on requested command={} source={source:?} route={route} target={context:?}",
            command.as_str()
        );

        let Some(definition) = self.commands.get(&command) else {
            let error = Rejection::UnknownCommand {
                command: command.as_str(),
            };
            log::debug!(
                "command invoke_on rejected command={} source={source:?} target={context:?}: {error}",
                command.as_str()
            );
            return Err(error);
        };

        if route != definition.target() {
            let error = Rejection::UnresolvedTarget {
                command: command.as_str(),
            };
            log::debug!(
                "command invoke_on rejected command={} source={source:?} route={route} expected_route={}: {error}",
                command.as_str(),
                definition.target()
            );
            return Err(error);
        }

        let state = self
            .configured_state_override_key(command, &context)
            .unwrap_or_else(|| target.state(&context));
        let state = state::with_running_overlay(state, self.is_running(command, &context));
        if !state.is_available() {
            let error = Rejection::Disabled {
                command: command.as_str(),
                context: context.clone(),
            };
            log::debug!(
                "command invoke_on rejected command={} source={source:?} target={context:?}: {error}",
                command.as_str()
            );
            return Err(error);
        }
        if state.is_running() {
            let error = Rejection::Running {
                command: command.as_str(),
                context: context.clone(),
            };
            log::debug!(
                "command invoke_on rejected command={} source={source:?} target={context:?}: {error}",
                command.as_str()
            );
            return Err(error);
        }

        let mut invocation = super::call::Invocation::<C>::for_command(source, context.clone())
            .with_repeated(repeated);
        if let Some(origin) = origin {
            invocation = invocation.with_origin(origin);
        }

        let response = target.invoke(args, invocation);

        log::debug!(
            "command invoke_on accepted command={} source={source:?} target={context:?}",
            command.as_str()
        );

        Ok(response)
    }

    pub(crate) fn invoke_any_on<TTarget: 'static>(
        &self,
        target: &mut TTarget,
        call: Any,
    ) -> Result<ErasedResponse, Rejection> {
        let command = call.command();
        let route = call.target();
        let target_kind = target::Kind::of_type::<TTarget>();
        let Some(definition) = self.commands.get(&command) else {
            return Err(Rejection::UnknownCommand {
                command: command.as_str(),
            });
        };

        if !definition.can_invoke_target(route, target_kind) {
            return Err(Rejection::UnresolvedTarget {
                command: command.as_str(),
            });
        }

        log::debug!(
            "command invoke_any_on requested command={} target={route}",
            command.as_str()
        );

        definition.invoke_target(self, target_kind, target, call)
    }

    pub(crate) fn can_invoke_any_on<TTarget: 'static>(&self, call: &Any) -> bool {
        let Some(definition) = self.commands.get(&call.command()) else {
            return false;
        };

        definition.can_invoke_target(call.target(), target::Kind::of_type::<TTarget>())
    }

    fn validate_request(&self, request: &Raw) -> Result<&Definition, Rejection> {
        let Some(command) = self.commands.get(&request.command()) else {
            return Err(Rejection::UnknownCommand {
                command: request.command().as_str(),
            });
        };

        if command.target() != request.target() {
            return Err(Rejection::UnresolvedTarget {
                command: request.command().as_str(),
            });
        }

        if self.is_running(request.command(), request.context()) {
            return Err(Rejection::Running {
                command: request.command().as_str(),
                context: request.context().clone(),
            });
        }

        if let Some(state) =
            self.configured_state_override_key(request.command(), request.context())
        {
            if !state.is_available() {
                return Err(Rejection::Disabled {
                    command: request.command().as_str(),
                    context: request.context().clone(),
                });
            }
            if state.is_running() {
                return Err(Rejection::Running {
                    command: request.command().as_str(),
                    context: request.context().clone(),
                });
            }
        }

        Ok(command)
    }

    fn is_running(&self, key: Key, context: &Context) -> bool {
        if self.running.contains(&(key, context.clone())) {
            return true;
        }

        if matches!(context.scope(), Scope::Path(_)) {
            return self
                .running
                .contains(&(key, Context::window(context.window_id())));
        }

        false
    }

    fn insert(&mut self, command: Definition) {
        let key = command.key();
        let target = command.target();
        let shortcuts = command.shortcuts().to_vec();

        if let Some(existing) = self.commands.get_mut(&key) {
            existing.merge(command);
        } else {
            self.commands.insert(key, command);
        }

        for shortcut in shortcuts {
            self.shortcuts
                .insert(shortcut, binding::Route::new(key, target));
        }
    }

    fn insert_all(&mut self, commands: Vec<Definition>) -> Result<(), RegisterError> {
        let mut staged_commands: HashMap<Key, Definition> = HashMap::new();
        let mut staged_shortcuts: HashMap<Shortcut, Key> = HashMap::new();

        for command in commands {
            let key = command.key();

            if let Some(existing) = self.commands.get(&key)
                && !existing.can_merge(&command)
            {
                return Err(RegisterError::DuplicateCommand {
                    command: key.as_str(),
                });
            }

            if let Some(existing) = staged_commands.get(&key)
                && !existing.can_merge(&command)
            {
                return Err(RegisterError::DuplicateCommand {
                    command: key.as_str(),
                });
            }

            for shortcut in command.shortcuts().iter().copied() {
                if let Some(existing) = self.shortcuts.get(&shortcut).copied() {
                    if existing.command() != key {
                        return Err(RegisterError::DuplicateShortcut {
                            shortcut,
                            existing: existing.command().as_str(),
                            duplicate: key.as_str(),
                        });
                    }
                }

                if let Some(existing) = staged_shortcuts.get(&shortcut).copied()
                    && existing != key
                {
                    return Err(RegisterError::DuplicateShortcut {
                        shortcut,
                        existing: existing.as_str(),
                        duplicate: key.as_str(),
                    });
                }

                staged_shortcuts.insert(shortcut, key);
            }

            if let Some(existing) = staged_commands.get_mut(&key) {
                existing.merge(command);
            } else {
                staged_commands.insert(key, command);
            }
        }

        for command in staged_commands.into_values() {
            self.insert(command);
        }

        Ok(())
    }
}

impl Default for Registry {
    fn default() -> Self {
        Self {
            commands: HashMap::new(),
            shortcuts: HashMap::new(),
            states: HashMap::new(),
            running: HashSet::new(),
        }
    }
}

impl Catalog for Definition {
    fn register(self, commands: &mut Commands) {
        commands.command(self);
    }
}

impl<I> Catalog for I
where
    I: IntoIterator<Item = Definition>,
{
    fn register(self, commands: &mut Commands) {
        commands.commands(self);
    }
}
