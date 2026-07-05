use std::any::{Any, TypeId};

use super::Responder;
use crate::scratch::{
    command::{self, Command, State},
    context::Context,
    error::{Error, Result},
    response::{AnyResponse, Response},
    state,
};

pub(in crate::scratch) trait Framework<M: state::State> {
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

    pub(in crate::scratch) fn with_framework(mut self, framework: impl Framework<M> + 'a) -> Self {
        self.framework = Some(Box::new(framework));
        self
    }

    pub(in crate::scratch) fn state<C: Command>(
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

    pub(in crate::scratch) fn state_any(
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

    pub(in crate::scratch) fn invoke<C: Command>(
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

    pub(in crate::scratch) fn invoke_any(
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
