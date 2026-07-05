use std::{
    any::{Any, TypeId},
    marker::PhantomData,
    rc::Rc,
};

use super::{
    command::{self, Command, State},
    context::Context,
    error::{Error, Result},
    interaction,
    response::{AnyResponse, Response},
    session, state,
    target::{AnyTarget, Target},
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Kind {
    Captured,
    Transient,
    Focused,
    Ancestor,
    Window,
    Workspace,
    App,
}

impl Kind {
    fn rank(self) -> usize {
        match self {
            Self::Captured => 0,
            Self::Transient => 1,
            Self::Focused => 2,
            Self::Ancestor => 3,
            Self::Window => 4,
            Self::Workspace => 5,
            Self::App => 6,
        }
    }
}

pub(super) trait Framework<M: state::State> {
    fn state(
        &mut self,
        store: &mut state::Store<M>,
        command_type: TypeId,
        command_name: &'static str,
        args: &dyn Any,
        cx: &Context,
    ) -> Result<Option<State>>;

    fn invoke(
        &mut self,
        store: &mut state::Store<M>,
        command_type: TypeId,
        command_name: &'static str,
        args: Box<dyn Any + Send>,
        cx: &mut Context,
    ) -> Option<AnyResponse>;
}

/// Runtime command resolution is an explicit chain built from focus/capture state.
pub struct Chain<'a, M: state::State> {
    store: &'a mut state::Store<M>,
    responders: Vec<&'a Responder<M>>,
    framework: Option<Box<dyn Framework<M> + 'a>>,
}

impl<'a, M: state::State> Chain<'a, M> {
    pub fn nearest_first(
        store: &'a mut state::Store<M>,
        responders: Vec<&'a Responder<M>>,
    ) -> Self {
        Self {
            store,
            responders,
            framework: None,
        }
    }

    pub(super) fn with_framework(mut self, framework: impl Framework<M> + 'a) -> Self {
        self.framework = Some(Box::new(framework));
        self
    }

    pub(super) fn state<C: Command>(
        &mut self,
        args: &C::Args,
        cx: &Context,
    ) -> Result<Option<State>> {
        for responder in &self.responders {
            let mut claim = None;

            for target in responder
                .targets
                .iter()
                .filter(|target| target.handles::<C>())
            {
                let state = target.state::<C>(self.store.model_mut(), args, cx)?;
                if !state.is_hidden() {
                    if claim.is_some() {
                        return Err(Error::AmbiguousTarget {
                            command: C::NAME,
                            responder: responder.name,
                        });
                    }

                    claim = Some(state);
                }
            }

            if claim.is_some() {
                return Ok(claim);
            }
        }

        match self.framework.as_mut() {
            Some(framework) => framework.state(self.store, TypeId::of::<C>(), C::NAME, args, cx),
            None => Ok(None),
        }
    }

    pub(super) fn state_any(
        &mut self,
        command_type: TypeId,
        command_name: &'static str,
        args: &dyn Any,
        cx: &Context,
    ) -> Result<Option<State>> {
        for responder in &self.responders {
            let mut claim = None;

            for target in responder
                .targets
                .iter()
                .filter(|target| target.handles_type(command_type))
            {
                let state = target.state_any(self.store.model_mut(), args, cx)?;
                if !state.is_hidden() {
                    if claim.is_some() {
                        return Err(Error::AmbiguousTarget {
                            command: command_name,
                            responder: responder.name,
                        });
                    }

                    claim = Some(state);
                }
            }

            if claim.is_some() {
                return Ok(claim);
            }
        }

        match self.framework.as_mut() {
            Some(framework) => framework.state(self.store, command_type, command_name, args, cx),
            None => Ok(None),
        }
    }

    pub(super) fn invoke<C: Command>(
        &mut self,
        args: C::Args,
        cx: &mut Context,
    ) -> Option<Response<C::Output>> {
        for responder in &self.responders {
            let mut claim = None;

            for (index, target) in responder
                .targets
                .iter()
                .enumerate()
                .filter(|(_, target)| target.handles::<C>())
            {
                match target.state::<C>(self.store.model_mut(), &args, cx) {
                    Ok(state) if state.is_hidden() => {}
                    Ok(state) => {
                        if claim.is_some() {
                            return Some(Response::failed(Error::AmbiguousTarget {
                                command: C::NAME,
                                responder: responder.name,
                            }));
                        }

                        claim = Some((index, state));
                    }
                    Err(error) => return Some(Response::failed(error)),
                }
            }

            let Some((index, state)) = claim else {
                continue;
            };

            match state.availability {
                command::Availability::Hidden => unreachable!("hidden targets are not claims"),
                command::Availability::Disabled => {
                    return Some(Response::failed(Error::Disabled { command: C::NAME }));
                }
                command::Availability::Enabled => {
                    return Some(responder.targets[index].invoke::<C>(
                        self.store.model_mut(),
                        args,
                        cx,
                    ));
                }
            }
        }

        self.framework.as_mut().and_then(|framework| {
            framework
                .invoke(self.store, TypeId::of::<C>(), C::NAME, Box::new(args), cx)
                .map(|response| response.into_response(C::NAME))
        })
    }

    pub(super) fn invoke_any(
        &mut self,
        command_type: TypeId,
        command_name: &'static str,
        args: Box<dyn Any + Send>,
        cx: &mut Context,
    ) -> Option<AnyResponse> {
        for responder in &self.responders {
            let mut claim = None;

            for (index, target) in responder
                .targets
                .iter()
                .enumerate()
                .filter(|(_, target)| target.handles_type(command_type))
            {
                match target.state_any(self.store.model_mut(), args.as_ref(), cx) {
                    Ok(state) if state.is_hidden() => {}
                    Ok(state) => {
                        if claim.is_some() {
                            return Some(AnyResponse::failed(Error::AmbiguousTarget {
                                command: command_name,
                                responder: responder.name,
                            }));
                        }

                        claim = Some((index, state));
                    }
                    Err(error) => return Some(AnyResponse::failed(error)),
                }
            }

            let Some((index, state)) = claim else {
                continue;
            };

            match state.availability {
                command::Availability::Hidden => unreachable!("hidden targets are not claims"),
                command::Availability::Disabled => {
                    return Some(AnyResponse::failed(Error::Disabled {
                        command: command_name,
                    }));
                }
                command::Availability::Enabled => {
                    return Some(responder.targets[index].invoke_any(
                        self.store.model_mut(),
                        args,
                        cx,
                    ));
                }
            }
        }

        self.framework.as_mut().and_then(|framework| {
            framework.invoke(self.store, command_type, command_name, args, cx)
        })
    }
}

pub struct Responder<M: state::State> {
    kind: Kind,
    name: &'static str,
    identity: interaction::Id,
    targets: Vec<AnyTarget<M>>,
}

impl<M: state::State> Responder<M> {
    pub(super) fn new(kind: Kind, name: &'static str) -> Self {
        Self {
            kind,
            name,
            identity: interaction::Id::new(name),
            targets: Vec::new(),
        }
    }

    fn matches_focus(&self, focus: Option<session::Focus>) -> bool {
        match self.kind {
            Kind::Focused => focus.is_some_and(|focus| self.identity == focus.target()),
            _ => true,
        }
    }
}

pub struct Builder<M: state::State> {
    specs: Vec<Responder<M>>,
}

pub struct ObjectBuilder<'builder, M: state::State, T: 'static> {
    spec: &'builder mut Responder<M>,
    selector: Rc<dyn for<'a> Fn(&'a mut M) -> &'a mut T>,
    _target: PhantomData<T>,
}

impl<M: state::State> Default for Builder<M> {
    fn default() -> Self {
        Self { specs: Vec::new() }
    }
}

impl<M: state::State> Builder<M> {
    pub fn app(&mut self) -> ObjectBuilder<'_, M, M> {
        self.object_with_kind(Kind::App, "app", |model| model)
    }

    pub fn object<T>(
        &mut self,
        name: &'static str,
        selector: impl for<'a> Fn(&'a mut M) -> &'a mut T + 'static,
    ) -> ObjectBuilder<'_, M, T>
    where
        T: 'static,
    {
        self.object_with_kind(Kind::Focused, name, selector)
    }

    fn object_with_kind<T>(
        &mut self,
        kind: Kind,
        name: &'static str,
        selector: impl for<'a> Fn(&'a mut M) -> &'a mut T + 'static,
    ) -> ObjectBuilder<'_, M, T>
    where
        T: 'static,
    {
        self.specs.push(Responder::new(kind, name));

        ObjectBuilder {
            spec: self
                .specs
                .last_mut()
                .expect("a responder spec was just pushed"),
            selector: Rc::new(selector),
            _target: PhantomData,
        }
    }

    pub(super) fn chain<'a>(&'a self, store: &'a mut state::Store<M>) -> Chain<'a, M> {
        self.chain_for(store, None)
    }

    pub(super) fn chain_for<'a>(
        &'a self,
        store: &'a mut state::Store<M>,
        focus: Option<session::Focus>,
    ) -> Chain<'a, M> {
        let mut responders = self
            .specs
            .iter()
            .enumerate()
            .filter(|(_, spec)| spec.matches_focus(focus))
            .map(|(index, spec)| (spec.kind.rank(), index, spec))
            .collect::<Vec<_>>();

        responders.sort_by_key(|(rank, index, _)| (*rank, *index));

        let responders = responders
            .into_iter()
            .map(|(_, _, responder)| responder)
            .collect();

        Chain::nearest_first(store, responders)
    }
}

impl<M, T> ObjectBuilder<'_, M, T>
where
    M: state::State,
    T: 'static,
{
    pub fn target<C: Command>(&mut self) -> &mut Self
    where
        T: Target<C>,
    {
        self.spec
            .targets
            .push(AnyTarget::new::<C, T>(Rc::clone(&self.selector)));
        self
    }
}
