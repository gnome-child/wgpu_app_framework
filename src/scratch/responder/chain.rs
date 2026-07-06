use std::any::{Any, TypeId};

use super::Responder;
use crate::scratch::{
    command::{self, Command, State},
    context::Context,
    error::{Error, Result},
    response::{AnyResponse, Response},
    state,
};

pub(in crate::scratch) trait Service<M: state::State> {
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
    services: Vec<Box<dyn Service<M> + 'a>>,
}

impl<'a, M: state::State> Chain<'a, M> {
    pub fn nearest_first(
        store: &'a mut state::Store<M>,
        responders: Vec<&'a Responder<M>>,
    ) -> Self {
        Self {
            store,
            responders,
            services: Vec::new(),
        }
    }

    pub(in crate::scratch) fn with_service(mut self, service: impl Service<M> + 'a) -> Self {
        self.services.push(Box::new(service));
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

        for service in &mut self.services {
            if let Some(state) = service.state(self.store, TypeId::of::<C>(), C::NAME, args, cx)? {
                return Ok(Some(state));
            }
        }

        Ok(None)
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

        for service in &mut self.services {
            if let Some(state) = service.state(self.store, command_type, command_name, args, cx)? {
                return Ok(Some(state));
            }
        }

        Ok(None)
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

        for service in &mut self.services {
            match service.state(self.store, TypeId::of::<C>(), C::NAME, &args, cx) {
                Ok(Some(_)) => {
                    return Some(
                        service
                            .invoke(self.store, TypeId::of::<C>(), C::NAME, Box::new(args), cx)
                            .map(|response| response.into_response(C::NAME))
                            .unwrap_or_else(|| {
                                Response::failed(Error::MissingTarget { command: C::NAME })
                            }),
                    );
                }
                Ok(None) => {}
                Err(error) => return Some(Response::failed(error)),
            }
        }

        None
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

        for service in &mut self.services {
            match service.state(self.store, command_type, command_name, args.as_ref(), cx) {
                Ok(Some(_)) => {
                    return Some(
                        service
                            .invoke(self.store, command_type, command_name, args, cx)
                            .unwrap_or_else(|| {
                                AnyResponse::failed(Error::MissingTarget {
                                    command: command_name,
                                })
                            }),
                    );
                }
                Ok(None) => {}
                Err(error) => return Some(AnyResponse::failed(error)),
            }
        }

        None
    }
}
