use std::{
    any::{Any, TypeId},
    collections::HashMap,
    marker::PhantomData,
    sync::Arc,
};

use super::{
    context::{Context, Source},
    error::{Error, Result},
    input, responder,
    response::{AnyResponse, Effect, Response},
    state,
};

/// App code dispatches by this type, not by an action id.
pub trait Command: 'static + Sized {
    type Args: Send + 'static;
    type Output: Send + 'static;

    /// Stable metadata for keymaps, debugging, settings, plugins, etc.
    /// Normal compiled app code should still use the command type.
    const NAME: &'static str;
    const HISTORY: History = History::Automatic;

    fn history_group(_args: &Self::Args) -> Option<HistoryGroup> {
        None
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum History {
    /// Runtime snapshots the model before dispatch and records changed state in undo history.
    Automatic,
    /// Handling target commits through framework services; runtime repairs changed user overrides.
    Committed,
    /// Command is not undoable; changed responses still advance revision but do not snapshot.
    Ignored,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct HistoryGroup {
    key: &'static str,
}

impl HistoryGroup {
    pub fn new(key: &'static str) -> Self {
        Self { key }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum Availability {
    Enabled,
    Disabled,
    Hidden,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct State {
    pub(super) availability: Availability,
    pub(super) checked: Option<bool>,
    pub(super) label: Option<String>,
    pub(super) shortcut: Option<KeyChord>,
    pub(super) tooltip: Option<String>,
}

impl State {
    pub(super) fn enabled() -> Self {
        Self {
            availability: Availability::Enabled,
            checked: None,
            label: None,
            shortcut: None,
            tooltip: None,
        }
    }

    pub(super) fn disabled() -> Self {
        Self {
            availability: Availability::Disabled,
            checked: None,
            label: None,
            shortcut: None,
            tooltip: None,
        }
    }

    /// Means "this target does not claim the command in this state; keep resolving".
    pub(super) fn hidden() -> Self {
        Self {
            availability: Availability::Hidden,
            checked: None,
            label: None,
            shortcut: None,
            tooltip: None,
        }
    }

    pub(super) fn is_enabled(&self) -> bool {
        self.availability == Availability::Enabled
    }

    pub(super) fn is_hidden(&self) -> bool {
        self.availability == Availability::Hidden
    }

    pub(super) fn with_label(mut self, label: impl Into<String>) -> Self {
        self.label = Some(label.into());
        self
    }

    pub(super) fn with_shortcut(mut self, shortcut: KeyChord) -> Self {
        self.shortcut = Some(shortcut);
        self
    }

    pub(super) fn with_tooltip(mut self, tooltip: impl Into<String>) -> Self {
        self.tooltip = Some(tooltip.into());
        self
    }

    pub(super) fn checked(mut self, checked: bool) -> Self {
        self.checked = Some(checked);
        self
    }

    fn with_command(mut self, command: &AnyCommand) -> Self {
        if self.label.is_none() {
            self = self.with_label(command.spec.display_name);
        }

        if let Some(shortcut) = command.shortcut()
            && self.shortcut.is_none()
        {
            self = self.with_shortcut(shortcut);
        }

        self
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct KeyChord(&'static str);

impl KeyChord {
    pub fn new(chord: &'static str) -> Self {
        Self(chord)
    }

    pub fn as_str(self) -> &'static str {
        self.0
    }

    fn matches_key(self, key: input::Key, modifiers: input::Modifiers) -> bool {
        let Some(chord) = ParsedKeyChord::parse(self.0) else {
            return false;
        };

        chord.matches_key(key, modifiers)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct ParsedKeyChord {
    key: ParsedKey,
    modifiers: input::Modifiers,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ParsedKey {
    Character(char),
    F4,
}

#[derive(Debug, Clone)]
pub struct Spec {
    display_name: &'static str,
    shortcut: Option<KeyChord>,
}

impl Spec {
    pub(super) fn new(display_name: &'static str) -> Self {
        Self {
            display_name,
            shortcut: None,
        }
    }

    pub(super) fn shortcut(mut self, shortcut: &'static str) -> Self {
        self.shortcut = Some(KeyChord(shortcut));
        self
    }
}

#[derive(Default)]
pub struct Registry {
    commands: HashMap<TypeId, AnyCommand>,
    shortcuts: HashMap<KeyChord, Vec<TypeId>>,
}

pub(super) struct AnyCommand {
    command_name: &'static str,
    command_type: TypeId,
    args_type: TypeId,
    history: History,
    spec: Spec,
}

impl AnyCommand {
    pub(super) fn command_name(&self) -> &'static str {
        self.command_name
    }

    pub(super) fn command_type(&self) -> TypeId {
        self.command_type
    }

    pub(super) fn history(&self) -> History {
        self.history
    }

    fn accepts_shortcut_args(&self) -> bool {
        self.args_type == TypeId::of::<()>()
    }

    fn shortcut(&self) -> Option<KeyChord> {
        self.accepts_shortcut_args().then_some(self.spec.shortcut?)
    }
}

#[derive(Debug, Clone)]
pub struct Observation {
    source: Source,
    effect: Effect,
    command_changed: bool,
    changed: bool,
}

impl Observation {
    pub(super) fn new(source: Source, effect: Effect, command_changed: bool) -> Self {
        Self {
            source,
            effect,
            command_changed,
            changed: false,
        }
    }

    pub fn source(&self) -> Source {
        self.source
    }

    pub fn effect(&self) -> &Effect {
        &self.effect
    }

    pub fn command_changed_state(&self) -> bool {
        self.command_changed
    }

    pub fn changed_state(&self) -> bool {
        self.changed
    }

    pub fn mark_changed(&mut self) {
        self.changed = true;
    }
}

pub struct Observers<M: state::State> {
    observers: HashMap<TypeId, Vec<Observer<M>>>,
}

struct Observer<M: state::State> {
    observe: ObserverFn<M>,
}

type ObserverFn<M> = Box<dyn FnMut(&mut M, &(dyn Any + Send), &mut Observation) -> Result<()>>;

impl<M: state::State> Default for Observers<M> {
    fn default() -> Self {
        Self {
            observers: HashMap::new(),
        }
    }
}

impl<M: state::State> Observers<M> {
    pub fn observe<C>(
        &mut self,
        mut callback: impl FnMut(&mut M, &C::Output, &mut Observation) + 'static,
    ) -> &mut Self
    where
        C: Command,
    {
        self.observers
            .entry(TypeId::of::<C>())
            .or_default()
            .push(Observer {
                observe: Box::new(move |model, output, observation| {
                    let output = output
                        .downcast_ref::<C::Output>()
                        .ok_or(Error::OutputMismatch { command: C::NAME })?;
                    callback(model, output, observation);
                    Ok(())
                }),
            });

        self
    }

    pub(super) fn observe_response<C>(
        &mut self,
        model: &mut M,
        response: &Response<C::Output>,
        source: Source,
    ) -> Result<bool>
    where
        C: Command,
    {
        let Some(output) = response.output_ref() else {
            return Ok(false);
        };

        let mut observation =
            Observation::new(source, response.effect.clone(), response.changed_state());
        self.observe_output(TypeId::of::<C>(), model, output, &mut observation)?;

        Ok(observation.changed_state())
    }

    pub(super) fn observe_any(
        &mut self,
        command_type: TypeId,
        model: &mut M,
        response: &AnyResponse,
        source: Source,
    ) -> Result<bool> {
        let Some(output) = response.output_any() else {
            return Ok(false);
        };

        let mut observation = Observation::new(source, response.effect(), response.changed_state());
        self.observe_output(command_type, model, output, &mut observation)?;

        Ok(observation.changed_state())
    }

    fn observe_output(
        &mut self,
        command_type: TypeId,
        model: &mut M,
        output: &(dyn Any + Send),
        observation: &mut Observation,
    ) -> Result<()> {
        let Some(observers) = self.observers.get_mut(&command_type) else {
            return Ok(());
        };

        for observer in observers {
            (observer.observe)(model, output, observation)?;
        }

        Ok(())
    }
}

impl Registry {
    pub fn register<C>(&mut self, spec: Spec) -> &mut Self
    where
        C: Command,
    {
        let shortcut = spec.shortcut;
        let command_type = TypeId::of::<C>();
        self.remove_shortcuts_for(command_type);
        self.commands.insert(
            command_type,
            AnyCommand {
                command_name: C::NAME,
                command_type,
                args_type: TypeId::of::<C::Args>(),
                history: C::HISTORY,
                spec,
            },
        );
        if let Some(shortcut) = shortcut {
            self.bind_shortcut(shortcut, command_type);
        }

        self
    }

    pub fn state<C: Command>(
        &self,
        chain: &mut responder::Chain<'_, impl state::State>,
        args: &C::Args,
        cx: &Context,
    ) -> State {
        let Ok(command) = self.command::<C>() else {
            return State::hidden();
        };

        let state = match chain.state::<C>(args, cx) {
            Ok(Some(state)) => state,
            Ok(None) => State::disabled(),
            Err(error) => State::disabled().with_tooltip(error.to_string()),
        };

        state.with_command(command)
    }

    pub(super) fn state_any(
        &self,
        command_type: TypeId,
        command_name: &'static str,
        args: &dyn Any,
        chain: &mut responder::Chain<'_, impl state::State>,
        cx: &Context,
    ) -> State {
        let Some(command) = self.commands.get(&command_type) else {
            return State::hidden();
        };

        let state = match chain.state_any(command_type, command_name, args, cx) {
            Ok(Some(state)) => state,
            Ok(None) => State::disabled(),
            Err(error) => State::disabled().with_tooltip(error.to_string()),
        };

        state.with_command(command)
    }

    pub fn invoke<C: Command>(
        &self,
        chain: &mut responder::Chain<'_, impl state::State>,
        args: C::Args,
        cx: &mut Context,
    ) -> Response<C::Output> {
        if let Err(error) = self.command::<C>() {
            return Response::failed(error);
        }

        chain
            .invoke::<C>(args, cx)
            .unwrap_or_else(|| Response::failed(Error::MissingTarget { command: C::NAME }))
    }

    pub(super) fn invoke_any(
        &self,
        command_type: TypeId,
        command_name: &'static str,
        args: Box<dyn Any + Send>,
        chain: &mut responder::Chain<'_, impl state::State>,
        cx: &mut Context,
    ) -> Option<AnyResponse> {
        self.commands.get(&command_type)?;

        Some(
            chain
                .invoke_any(command_type, command_name, args, cx)
                .unwrap_or_else(|| {
                    AnyResponse::failed(Error::MissingTarget {
                        command: command_name,
                    })
                }),
        )
    }

    pub(super) fn shortcut_command(&self, shortcut: KeyChord) -> Result<Option<&AnyCommand>> {
        let Some(command_types) = self.shortcuts.get(&shortcut) else {
            return Ok(None);
        };

        if command_types.len() > 1 {
            return Err(Error::AmbiguousShortcut {
                shortcut: shortcut.as_str(),
                commands: self.shortcut_command_names(command_types),
            });
        }

        let command = command_types
            .first()
            .and_then(|command_type| self.commands.get(command_type));
        let Some(command) = command else {
            return Ok(None);
        };

        if !command.accepts_shortcut_args() {
            return Err(Error::ShortcutRequiresArgs {
                shortcut: shortcut.as_str(),
                command: command.command_name,
            });
        }

        Ok(Some(command))
    }

    pub(super) fn history_for(&self, command_type: TypeId) -> Option<History> {
        self.commands.get(&command_type).map(AnyCommand::history)
    }

    pub(super) fn shortcut_for_key(
        &self,
        key: input::Key,
        modifiers: input::Modifiers,
    ) -> Option<KeyChord> {
        self.shortcuts
            .keys()
            .find(|shortcut| shortcut.matches_key(key, modifiers))
            .copied()
    }

    pub(super) fn invoke_shortcut(
        &self,
        shortcut: KeyChord,
        chain: &mut responder::Chain<'_, impl state::State>,
        cx: &mut Context,
    ) -> Result<Option<AnyResponse>> {
        let Some(command) = self.shortcut_command(shortcut)? else {
            return Ok(None);
        };

        Ok(Some(
            chain
                .invoke_any(command.command_type, command.command_name, Box::new(()), cx)
                .unwrap_or_else(|| {
                    AnyResponse::failed(Error::MissingTarget {
                        command: command.command_name,
                    })
                }),
        ))
    }

    fn command<C: Command>(&self) -> Result<&AnyCommand> {
        self.commands
            .get(&TypeId::of::<C>())
            .ok_or(Error::UnknownCommand { command: C::NAME })
    }

    pub(super) fn apply_spec<C: Command>(&self, state: State) -> State {
        let Ok(command) = self.command::<C>() else {
            return State::hidden();
        };

        state.with_command(command)
    }

    fn bind_shortcut(&mut self, shortcut: KeyChord, command_type: TypeId) {
        let command_types = self.shortcuts.entry(shortcut).or_default();
        if !command_types.contains(&command_type) {
            command_types.push(command_type);
        }
    }

    fn remove_shortcuts_for(&mut self, command_type: TypeId) {
        self.shortcuts.retain(|_, command_types| {
            command_types.retain(|registered| *registered != command_type);
            !command_types.is_empty()
        });
    }

    fn shortcut_command_names(&self, command_types: &[TypeId]) -> Vec<&'static str> {
        command_types
            .iter()
            .filter_map(|command_type| self.commands.get(command_type))
            .map(|command| command.command_name)
            .collect()
    }
}

impl ParsedKeyChord {
    fn parse(chord: &'static str) -> Option<Self> {
        let mut control = false;
        let mut shift = false;
        let mut alt = false;
        let mut super_key = false;
        let mut key = None;

        for part in chord.split('+') {
            if part.eq_ignore_ascii_case("ctrl") || part.eq_ignore_ascii_case("control") {
                control = true;
            } else if part.eq_ignore_ascii_case("shift") {
                shift = true;
            } else if part.eq_ignore_ascii_case("alt") {
                alt = true;
            } else if part.eq_ignore_ascii_case("super")
                || part.eq_ignore_ascii_case("cmd")
                || part.eq_ignore_ascii_case("meta")
            {
                super_key = true;
            } else if part.eq_ignore_ascii_case("f4") {
                key = Some(ParsedKey::F4);
            } else {
                let mut chars = part.chars();
                let value = chars.next()?;
                if chars.next().is_some() {
                    return None;
                }

                key = Some(ParsedKey::Character(value.to_ascii_lowercase()));
            }
        }

        Some(Self {
            key: key?,
            modifiers: input::Modifiers::new(shift, control, alt, super_key),
        })
    }

    fn matches_key(self, key: input::Key, modifiers: input::Modifiers) -> bool {
        let key = match key.normalized() {
            input::Key::Character(value) => ParsedKey::Character(value),
            input::Key::F4 => ParsedKey::F4,
            _ => return false,
        };

        self.key == key && self.modifiers == modifiers
    }
}

pub struct Trigger<C: Command> {
    args: C::Args,
    _command: PhantomData<C>,
}

impl<C: Command> Trigger<C> {
    pub fn command(args: C::Args) -> Self {
        Self {
            args,
            _command: PhantomData,
        }
    }

    pub fn state<M: state::State>(
        &self,
        registry: &Registry,
        chain: &mut responder::Chain<'_, M>,
        cx: &Context,
    ) -> State {
        registry.state::<C>(chain, &self.args, cx)
    }

    pub(super) fn args(&self) -> &C::Args {
        &self.args
    }

    pub(super) fn into_args(self) -> C::Args {
        self.args
    }

    pub fn invoke(
        &self,
        registry: &Registry,
        chain: &mut responder::Chain<'_, impl state::State>,
        cx: &mut Context,
    ) -> Response<C::Output>
    where
        C::Args: Clone,
    {
        registry.invoke::<C>(chain, self.args.clone(), cx)
    }
}

pub(super) struct AnyTrigger {
    command_name: &'static str,
    command_type: TypeId,
    args: Box<dyn AnyArgs>,
}

pub(super) struct AnyValueTrigger<I> {
    command_name: &'static str,
    command_type: TypeId,
    build_args: Arc<dyn Fn(I) -> Box<dyn AnyArgs> + Send + Sync>,
}

trait AnyArgs {
    fn clone_box(&self) -> Box<dyn AnyArgs>;

    fn as_any(&self) -> &dyn Any;

    fn clone_any(&self) -> Box<dyn Any + Send>;
}

struct TypedArgs<C: Command> {
    args: C::Args,
    _command: PhantomData<C>,
}

impl Clone for AnyTrigger {
    fn clone(&self) -> Self {
        Self {
            command_name: self.command_name,
            command_type: self.command_type,
            args: self.args.clone_box(),
        }
    }
}

impl<I> Clone for AnyValueTrigger<I> {
    fn clone(&self) -> Self {
        Self {
            command_name: self.command_name,
            command_type: self.command_type,
            build_args: Arc::clone(&self.build_args),
        }
    }
}

impl AnyTrigger {
    pub(super) fn command<C>(args: C::Args) -> Self
    where
        C: Command,
        C::Args: Clone,
    {
        Self {
            command_name: C::NAME,
            command_type: TypeId::of::<C>(),
            args: Box::new(TypedArgs::<C> {
                args,
                _command: PhantomData,
            }),
        }
    }

    pub(super) fn command_name(&self) -> &'static str {
        self.command_name
    }

    pub(super) fn command_type(&self) -> TypeId {
        self.command_type
    }

    pub(super) fn state(
        &self,
        registry: &Registry,
        chain: &mut responder::Chain<'_, impl state::State>,
        cx: &Context,
    ) -> State {
        registry.state_any(
            self.command_type,
            self.command_name,
            self.args.as_any(),
            chain,
            cx,
        )
    }

    pub(super) fn invoke(
        &self,
        registry: &Registry,
        chain: &mut responder::Chain<'_, impl state::State>,
        cx: &mut Context,
    ) -> AnyResponse {
        registry
            .invoke_any(
                self.command_type,
                self.command_name,
                self.args.clone_any(),
                chain,
                cx,
            )
            .unwrap_or_else(|| {
                AnyResponse::failed(Error::MissingTarget {
                    command: self.command_name,
                })
            })
    }
}

impl<I> AnyValueTrigger<I> {
    pub(super) fn command<C>(map: impl Fn(I) -> C::Args + Send + Sync + 'static) -> Self
    where
        C: Command,
        C::Args: Clone,
    {
        Self {
            command_name: C::NAME,
            command_type: TypeId::of::<C>(),
            build_args: Arc::new(move |input| {
                Box::new(TypedArgs::<C> {
                    args: map(input),
                    _command: PhantomData,
                })
            }),
        }
    }

    pub(super) fn trigger(&self, input: I) -> AnyTrigger {
        AnyTrigger {
            command_name: self.command_name,
            command_type: self.command_type,
            args: (self.build_args)(input),
        }
    }
}

impl<C> AnyArgs for TypedArgs<C>
where
    C: Command,
    C::Args: Clone,
{
    fn clone_box(&self) -> Box<dyn AnyArgs> {
        Box::new(TypedArgs::<C> {
            args: self.args.clone(),
            _command: PhantomData,
        })
    }

    fn as_any(&self) -> &dyn Any {
        &self.args
    }

    fn clone_any(&self) -> Box<dyn Any + Send> {
        Box::new(self.args.clone())
    }
}
