use std::any::Any as StdAny;
use std::fmt;

use super::call::{Any, Raw};
use super::shortcut::Shortcut;
use super::{Command, Key, Target, state::State, target};

pub struct Definition {
    key: Key,
    display: String,
    hint: Option<String>,
    contract: Contract,
    shortcuts: Vec<Shortcut>,
    call: CallFactory,
    target_invoker: Option<TargetInvoker>,
    target_state: Option<TargetState>,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash)]
pub struct Contract {
    target: target::Kind,
    repeatable: bool,
}

struct CallFactory {
    build: fn(Raw) -> Result<Any, super::args::Error>,
}

#[derive(Debug)]
pub(crate) struct ErasedResponse {
    pub(crate) command: Key,
    pub(crate) context: super::call::Context,
    pub(crate) effects: Vec<super::Effect>,
}

#[derive(Clone, Copy)]
struct TargetInvoker {
    target: target::Kind,
    invoke: fn(
        &super::Registry,
        &mut dyn StdAny,
        Any,
    ) -> Result<ErasedResponse, super::registry::Rejection>,
}

#[derive(Clone, Copy)]
struct TargetState {
    target: target::Kind,
    read: fn(&dyn StdAny, &super::call::Context) -> Option<State>,
}

impl Definition {
    pub fn for_command<C, TTarget>() -> Self
    where
        C: Command,
        TTarget: Target<C> + 'static,
    {
        Self::for_target::<C>(target::Kind::of_type::<TTarget>())
            .with_target_invoker::<C, TTarget>()
    }

    pub(crate) fn for_target<C: Command>(target: target::Kind) -> Self {
        Self {
            key: Key::of::<C>(),
            display: C::DISPLAY.to_owned(),
            hint: C::hint().map(str::to_owned),
            contract: Contract {
                target,
                repeatable: C::repeatable(),
            },
            shortcuts: Vec::new(),
            call: CallFactory::of::<C>(),
            target_invoker: None,
            target_state: None,
        }
    }

    pub(crate) fn key(&self) -> Key {
        self.key
    }

    pub fn display(&self) -> &str {
        &self.display
    }

    pub fn hint(&self) -> Option<&str> {
        self.hint.as_deref()
    }

    pub fn contract(&self) -> Contract {
        self.contract
    }

    pub fn target(&self) -> target::Kind {
        self.contract.target
    }

    pub fn shortcuts(&self) -> &[Shortcut] {
        &self.shortcuts
    }

    pub fn accepts_repeat(&self) -> bool {
        self.contract.repeatable
    }

    pub(crate) fn prepare_call(&self, request: Raw) -> Result<Any, super::args::Error> {
        self.call.build(request)
    }

    pub fn with_display(mut self, display: impl Into<String>) -> Self {
        self.display = display.into();
        self
    }

    pub fn with_hint(mut self, hint: impl Into<String>) -> Self {
        self.hint = Some(hint.into());
        self
    }

    pub fn repeatable(mut self) -> Self {
        self.contract.repeatable = true;
        self
    }

    pub fn shortcut(mut self, shortcut: Shortcut) -> Self {
        self.shortcuts.push(shortcut);
        self
    }

    pub(crate) fn invoke_target(
        &self,
        registry: &super::Registry,
        target: &mut dyn StdAny,
        call: Any,
    ) -> Result<ErasedResponse, super::registry::Rejection> {
        let Some(invoker) = self.target_invoker else {
            return Err(super::registry::Rejection::UnresolvedTarget {
                command: self.key.as_str(),
            });
        };

        if call.target() != invoker.target {
            return Err(super::registry::Rejection::UnresolvedTarget {
                command: self.key.as_str(),
            });
        }

        (invoker.invoke)(registry, target, call)
    }

    pub(crate) fn state_for_target(
        &self,
        target_kind: target::Kind,
        target: &dyn StdAny,
        context: &super::call::Context,
    ) -> Option<State> {
        let reader = self.target_state?;
        if target_kind != reader.target || self.target() != reader.target {
            return None;
        }

        (reader.read)(target, context)
    }

    pub(crate) fn fallback_state(&self) -> State {
        if self.target_state.is_some() {
            State::available()
        } else {
            State::unavailable()
        }
    }

    fn with_target_invoker<C, TTarget>(mut self) -> Self
    where
        C: Command,
        TTarget: Target<C> + 'static,
    {
        self.target_invoker = Some(TargetInvoker {
            target: target::Kind::of_type::<TTarget>(),
            invoke: invoke_target::<C, TTarget>,
        });
        self.target_state = Some(TargetState {
            target: target::Kind::of_type::<TTarget>(),
            read: target_state::<C, TTarget>,
        });
        self
    }
}

impl Contract {
    pub fn target(self) -> target::Kind {
        self.target
    }

    pub fn repeatable(self) -> bool {
        self.repeatable
    }
}

impl CallFactory {
    fn of<C: Command>() -> Self {
        Self {
            build: Any::from_raw::<C>,
        }
    }

    fn build(&self, request: Raw) -> Result<Any, super::args::Error> {
        (self.build)(request)
    }
}

fn invoke_target<C, TTarget>(
    registry: &super::Registry,
    target: &mut dyn StdAny,
    call: Any,
) -> Result<ErasedResponse, super::registry::Rejection>
where
    C: Command,
    TTarget: Target<C> + 'static,
{
    let target = target
        .downcast_mut::<TTarget>()
        .ok_or(super::registry::Rejection::UnresolvedTarget { command: C::NAME })?;
    let call = call
        .into_call::<C>()
        .map_err(|_| super::registry::Rejection::UnknownCommand { command: C::NAME })?;
    let context = call
        .requested_context()
        .ok_or(super::registry::Rejection::UnresolvedTarget { command: C::NAME })?;
    let response = registry.invoke_on(target, call)?;
    let (_, effects) = response.into_parts();

    Ok(ErasedResponse {
        command: Key::of::<C>(),
        context,
        effects,
    })
}

fn target_state<C, TTarget>(target: &dyn StdAny, context: &super::call::Context) -> Option<State>
where
    C: Command,
    TTarget: Target<C> + 'static,
{
    target
        .downcast_ref::<TTarget>()
        .map(|target| <TTarget as Target<C>>::state(target, context))
}

impl fmt::Debug for Definition {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("Definition")
            .field("key", &self.key)
            .field("display", &self.display)
            .field("hint", &self.hint)
            .field("contract", &self.contract)
            .field("shortcuts", &self.shortcuts)
            .field("target_invoker", &self.target_invoker.is_some())
            .field("target_state", &self.target_state.is_some())
            .finish_non_exhaustive()
    }
}
