use std::{
    any::{Any, TypeId},
    marker::PhantomData,
    sync::Arc,
};

use super::super::{
    context::Context,
    error::Error,
    responder,
    response::{AnyResponse, Response},
    state,
};
use super::{Command, HistoryGroup, Registry, State};
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

    pub(crate) fn args(&self) -> &C::Args {
        &self.args
    }

    pub(crate) fn into_args(self) -> C::Args {
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

pub(crate) struct AnyTrigger {
    command_name: &'static str,
    command_type: TypeId,
    args: Box<dyn AnyArgs>,
}

pub(crate) struct AnyValueTrigger<I> {
    command_name: &'static str,
    command_type: TypeId,
    build_args: Arc<dyn Fn(I) -> Box<dyn AnyArgs> + Send + Sync>,
}

trait AnyArgs {
    fn clone_box(&self) -> Box<dyn AnyArgs>;

    fn as_any(&self) -> &dyn Any;

    fn clone_any(&self) -> Box<dyn Any + Send>;

    fn history_group(&self) -> Option<HistoryGroup>;
}

struct TypedArgs<C: Command> {
    args: C::Args,
    _command: PhantomData<C>,
}

struct UnitArgs {
    history_group: fn(&dyn Any) -> Option<HistoryGroup>,
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
    pub(crate) fn command<C>(args: C::Args) -> Self
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

    pub(in crate::command) fn unit(
        command_type: TypeId,
        command_name: &'static str,
        history_group: fn(&dyn Any) -> Option<HistoryGroup>,
    ) -> Self {
        Self {
            command_name,
            command_type,
            args: Box::new(UnitArgs { history_group }),
        }
    }

    pub(crate) fn command_name(&self) -> &'static str {
        self.command_name
    }

    pub(crate) fn command_type(&self) -> TypeId {
        self.command_type
    }

    pub(crate) fn history_group(&self) -> Option<HistoryGroup> {
        self.args.history_group()
    }

    pub(in crate::command) fn args(&self) -> &dyn Any {
        self.args.as_any()
    }

    pub(crate) fn state_on(
        &self,
        route: responder::Route,
        registry: &Registry,
        chain: &mut responder::Chain<'_, impl state::State>,
        cx: &Context,
    ) -> State {
        registry.state_any_on(
            route,
            self.command_type,
            self.command_name,
            self.args.as_any(),
            chain,
            cx,
        )
    }

    pub(crate) fn invoke_on(
        &self,
        route: responder::Route,
        registry: &Registry,
        chain: &mut responder::Chain<'_, impl state::State>,
        cx: &mut Context,
    ) -> AnyResponse {
        registry
            .invoke_any_on(
                route,
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

impl AnyArgs for UnitArgs {
    fn clone_box(&self) -> Box<dyn AnyArgs> {
        Box::new(UnitArgs {
            history_group: self.history_group,
        })
    }

    fn as_any(&self) -> &dyn Any {
        &()
    }

    fn clone_any(&self) -> Box<dyn Any + Send> {
        Box::new(())
    }

    fn history_group(&self) -> Option<HistoryGroup> {
        (self.history_group)(&())
    }
}

impl<I> AnyValueTrigger<I> {
    pub(crate) fn command<C>(map: impl Fn(I) -> C::Args + Send + Sync + 'static) -> Self
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

    pub(crate) fn trigger(&self, input: I) -> AnyTrigger {
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

    fn history_group(&self) -> Option<HistoryGroup> {
        C::history_group(&self.args)
    }
}
