use std::{
    any::{Any, TypeId},
    collections::HashMap,
};

use super::super::{
    context::Context,
    error::{Error, Result},
    input, keymap, responder,
    response::{AnyResponse, Response},
    state,
};
use super::{AnyTrigger, Command, History, HistoryGroup, KeyChord, Spec, State};
#[derive(Default)]
pub struct Registry {
    commands: HashMap<TypeId, AnyCommand>,
    shortcuts: HashMap<KeyChord, Vec<TypeId>>,
    order: Vec<TypeId>,
}

#[derive(Clone)]
pub(crate) struct ResolvedCommand {
    registration_index: usize,
    command_type: TypeId,
    command_name: &'static str,
    trigger: AnyTrigger,
    state: State,
    claim: responder::Claim,
}

pub(crate) struct AnyCommand {
    pub(in crate::command) command_name: &'static str,
    pub(in crate::command) command_type: TypeId,
    args_type: TypeId,
    history: History,
    history_group: fn(&dyn Any) -> Option<HistoryGroup>,
    pub(in crate::command) spec: Spec,
}

impl AnyCommand {
    pub(crate) fn command_name(&self) -> &'static str {
        self.command_name
    }

    pub(crate) fn command_type(&self) -> TypeId {
        self.command_type
    }

    pub(crate) fn history(&self) -> History {
        self.history
    }

    pub(crate) fn history_group(&self, args: &dyn Any) -> Option<HistoryGroup> {
        (self.history_group)(args)
    }

    fn accepts_shortcut_args(&self) -> bool {
        self.args_type == TypeId::of::<()>()
    }

    pub(in crate::command) fn shortcut(&self) -> Option<KeyChord> {
        self.accepts_shortcut_args().then_some(self.spec.shortcut?)
    }

    fn unit_trigger(&self) -> AnyTrigger {
        AnyTrigger::unit(self.command_type, self.command_name, self.history_group)
    }
}

impl ResolvedCommand {
    pub(crate) fn registration_index(&self) -> usize {
        self.registration_index
    }

    pub(crate) fn command_type(&self) -> TypeId {
        self.command_type
    }

    pub(crate) fn command_name(&self) -> &'static str {
        self.command_name
    }

    pub(crate) fn trigger(&self) -> AnyTrigger {
        self.trigger.clone()
    }

    pub(crate) fn history_group(&self) -> Option<HistoryGroup> {
        self.trigger.history_group()
    }

    pub(crate) fn state(&self) -> &State {
        &self.state
    }

    pub(crate) fn claim(&self) -> &responder::Claim {
        &self.claim
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
        if !self.order.contains(&command_type) {
            self.order.push(command_type);
        }
        self.commands.insert(
            command_type,
            AnyCommand {
                command_name: C::NAME,
                command_type,
                args_type: TypeId::of::<C::Args>(),
                history: C::HISTORY,
                history_group: history_group_for::<C>,
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

    pub(crate) fn state_any(
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

    pub(crate) fn resolved_unit_commands(
        &self,
        chain: &mut responder::Chain<'_, impl state::State>,
        cx: &Context,
    ) -> Vec<ResolvedCommand> {
        let args = ();
        self.order
            .iter()
            .enumerate()
            .filter_map(|(registration_index, command_type)| {
                let command = self.commands.get(command_type)?;
                if !command.accepts_shortcut_args() {
                    return None;
                }
                if command.spec.listing == super::Listing::Describer {
                    return None;
                }
                let claim =
                    match chain.claim_any(command.command_type, command.command_name, &args, cx) {
                        Ok(Some(claim)) => claim,
                        Ok(None) | Err(_) => return None,
                    };
                let state = claim.state().clone().with_command(command);
                if !state.is_enabled() {
                    return None;
                }

                Some(ResolvedCommand {
                    registration_index,
                    command_type: command.command_type,
                    command_name: command.command_name,
                    trigger: command.unit_trigger(),
                    state,
                    claim,
                })
            })
            .collect()
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

    pub(crate) fn invoke_any(
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

    pub(crate) fn shortcut_command(
        &self,
        shortcut: KeyChord,
        profile: keymap::Profile,
    ) -> Result<Option<&AnyCommand>> {
        let command_types = self.command_types_matching_shortcut(shortcut, profile);
        if command_types.is_empty() {
            return Ok(None);
        }

        if command_types.len() > 1 {
            return Err(Error::AmbiguousShortcut {
                shortcut: shortcut.as_str(),
                commands: self.shortcut_command_names(&command_types),
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

    pub(crate) fn history_for(&self, command_type: TypeId) -> Option<History> {
        self.commands.get(&command_type).map(AnyCommand::history)
    }

    pub(crate) fn shortcut_for_key(
        &self,
        key: input::Key,
        modifiers: input::Modifiers,
        profile: keymap::Profile,
    ) -> Result<Option<KeyChord>> {
        let matching = self
            .registered_shortcuts_in_order()
            .filter(|shortcut| shortcut.matches_key(key, modifiers, profile))
            .collect::<Vec<_>>();
        if matching.is_empty() {
            return Ok(None);
        }

        let mut command_types = Vec::new();
        for shortcut in &matching {
            if let Some(types) = self.shortcuts.get(shortcut) {
                push_unique_command_types(&mut command_types, types);
            }
        }

        let shortcut = matching[0];
        if command_types.len() > 1 {
            return Err(Error::AmbiguousShortcut {
                shortcut: shortcut.as_str(),
                commands: self.shortcut_command_names(&command_types),
            });
        }

        let Some(command) = command_types
            .first()
            .and_then(|command_type| self.commands.get(command_type))
        else {
            return Ok(None);
        };
        if !command.accepts_shortcut_args() {
            return Err(Error::ShortcutRequiresArgs {
                shortcut: shortcut.as_str(),
                command: command.command_name,
            });
        }

        Ok(Some(shortcut))
    }

    pub(crate) fn invoke_shortcut(
        &self,
        shortcut: KeyChord,
        profile: keymap::Profile,
        chain: &mut responder::Chain<'_, impl state::State>,
        cx: &mut Context,
    ) -> Result<Option<AnyResponse>> {
        let Some(command) = self.shortcut_command(shortcut, profile)? else {
            return Ok(None);
        };
        let args = ();
        let state = self.state_any(command.command_type, command.command_name, &args, chain, cx);
        if !state.is_enabled() {
            return Ok(None);
        }

        Ok(Some(
            chain
                .invoke_any(
                    command.command_type,
                    command.command_name,
                    Box::new(args),
                    cx,
                )
                .unwrap_or_else(|| {
                    AnyResponse::failed(Error::MissingTarget {
                        command: command.command_name,
                    })
                }),
        ))
    }

    pub(crate) fn command<C: Command>(&self) -> Result<&AnyCommand> {
        self.commands
            .get(&TypeId::of::<C>())
            .ok_or(Error::UnknownCommand { command: C::NAME })
    }

    pub(crate) fn apply_spec<C: Command>(&self, state: State) -> State {
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

    fn command_types_matching_shortcut(
        &self,
        shortcut: KeyChord,
        profile: keymap::Profile,
    ) -> Vec<TypeId> {
        let requested = profile.chords(shortcut);
        let mut command_types = Vec::new();
        for registered in self.registered_shortcuts_in_order() {
            let Some(registered_command_types) = self.shortcuts.get(&registered) else {
                continue;
            };
            let registered = profile.chords(registered);
            if requested
                .iter()
                .any(|requested| registered.iter().any(|registered| requested == registered))
            {
                push_unique_command_types(&mut command_types, registered_command_types);
            }
        }

        command_types
    }

    fn registered_shortcuts_in_order(&self) -> impl Iterator<Item = KeyChord> + '_ {
        self.order.iter().filter_map(|command_type| {
            self.commands
                .get(command_type)
                .and_then(|command| command.spec.shortcut)
        })
    }
}

fn history_group_for<C: Command>(args: &dyn Any) -> Option<HistoryGroup> {
    args.downcast_ref::<C::Args>().and_then(C::history_group)
}

fn push_unique_command_types(target: &mut Vec<TypeId>, source: &[TypeId]) {
    for command_type in source {
        if !target.contains(command_type) {
            target.push(*command_type);
        }
    }
}
