use std::{
    any::{Any, TypeId},
    collections::HashMap,
};

use super::super::{
    context::Context,
    error::{Error, Result},
    input, responder,
    response::{AnyResponse, Response},
    state,
};
use super::{Command, History, KeyChord, Spec, State};
#[derive(Default)]
pub struct Registry {
    commands: HashMap<TypeId, AnyCommand>,
    shortcuts: HashMap<KeyChord, Vec<TypeId>>,
}

pub(in crate::scratch) struct AnyCommand {
    pub(in crate::scratch::command) command_name: &'static str,
    pub(in crate::scratch::command) command_type: TypeId,
    args_type: TypeId,
    history: History,
    pub(in crate::scratch::command) spec: Spec,
}

impl AnyCommand {
    pub(in crate::scratch) fn command_name(&self) -> &'static str {
        self.command_name
    }

    pub(in crate::scratch) fn command_type(&self) -> TypeId {
        self.command_type
    }

    pub(in crate::scratch) fn history(&self) -> History {
        self.history
    }

    fn accepts_shortcut_args(&self) -> bool {
        self.args_type == TypeId::of::<()>()
    }

    pub(in crate::scratch::command) fn shortcut(&self) -> Option<KeyChord> {
        self.accepts_shortcut_args().then_some(self.spec.shortcut?)
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

    pub(in crate::scratch) fn state_any(
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

    pub(in crate::scratch) fn invoke_any(
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

    pub(in crate::scratch) fn shortcut_command(
        &self,
        shortcut: KeyChord,
    ) -> Result<Option<&AnyCommand>> {
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

    pub(in crate::scratch) fn history_for(&self, command_type: TypeId) -> Option<History> {
        self.commands.get(&command_type).map(AnyCommand::history)
    }

    pub(in crate::scratch) fn shortcut_for_key(
        &self,
        key: input::Key,
        modifiers: input::Modifiers,
    ) -> Option<KeyChord> {
        self.shortcuts
            .keys()
            .find(|shortcut| shortcut.matches_key(key, modifiers))
            .copied()
    }

    pub(in crate::scratch) fn invoke_shortcut(
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

    pub(in crate::scratch) fn command<C: Command>(&self) -> Result<&AnyCommand> {
        self.commands
            .get(&TypeId::of::<C>())
            .ok_or(Error::UnknownCommand { command: C::NAME })
    }

    pub(in crate::scratch) fn apply_spec<C: Command>(&self, state: State) -> State {
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
