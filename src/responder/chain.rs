use std::any::{Any, TypeId};

use super::{Kind, Responder};
use crate::{
    command::{self, Command, Error, Result, State},
    context::Context,
    interaction,
    notification::{Notification, Reaction},
    response::{AnyResponse, Response},
    state,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Route {
    Chain,
    Responder(interaction::Id),
    Service(&'static str),
}

pub(crate) trait Service<M: state::State> {
    fn claim(
        &mut self,
        store: &mut state::Store<M>,
        command_type: TypeId,
        command_name: &'static str,
        args: &dyn Any,
        cx: &Context,
    ) -> Result<Option<Claim>>;

    fn invoke(
        &mut self,
        store: &mut state::Store<M>,
        command_type: TypeId,
        command_name: &'static str,
        args: Box<dyn Any + Send>,
        cx: &mut Context,
    ) -> Option<AnyResponse>;

    fn claim_exact(
        &mut self,
        _store: &mut state::Store<M>,
        _service: &'static str,
        _command_type: TypeId,
        _command_name: &'static str,
        _args: &dyn Any,
        _cx: &Context,
    ) -> Result<Option<Claim>> {
        Ok(None)
    }

    fn invoke_exact(
        &mut self,
        _store: &mut state::Store<M>,
        _service: &'static str,
        _command_type: TypeId,
        _command_name: &'static str,
        _args: Box<dyn Any + Send>,
        _cx: &mut Context,
    ) -> Option<AnyResponse> {
        None
    }
}

/// Runtime command resolution is an explicit chain built from focus/capture state.
pub struct Chain<'a, M: state::State> {
    store: &'a mut state::Store<M>,
    responders: Vec<&'a Responder<M>>,
    services: Vec<Box<dyn Service<M> + 'a>>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct Provenance {
    kind: Kind,
    name: &'static str,
    order: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct Claim {
    provenance: Provenance,
    state: State,
}

struct TargetClaim {
    responder: usize,
    target: usize,
    claim: Claim,
}

impl Provenance {
    pub(crate) fn new(kind: Kind, name: &'static str, order: usize) -> Self {
        Self { kind, name, order }
    }

    pub(crate) fn kind(&self) -> Kind {
        self.kind
    }

    pub(crate) fn sort_key(&self) -> (usize, usize, &'static str) {
        (self.order, self.kind.structural_order(), self.name)
    }
}

impl Claim {
    pub(crate) fn new(provenance: Provenance, state: State) -> Self {
        Self { provenance, state }
    }

    pub(crate) fn service(kind: Kind, name: &'static str, state: State) -> Self {
        Self::new(Provenance::new(kind, name, 0), state)
    }

    pub(crate) fn with_order(mut self, order: usize) -> Self {
        self.provenance.order = order;
        self
    }

    pub(crate) fn provenance(&self) -> &Provenance {
        &self.provenance
    }

    pub(crate) fn state(&self) -> &State {
        &self.state
    }

    pub(crate) fn into_state(self) -> State {
        self.state
    }
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

    pub(crate) fn with_service(mut self, service: impl Service<M> + 'a) -> Self {
        self.services.push(Box::new(service));
        self
    }

    pub(crate) fn state<C: Command>(
        &mut self,
        args: &C::Args,
        cx: &Context,
    ) -> Result<Option<State>> {
        self.state_any(TypeId::of::<C>(), C::NAME, args, cx)
    }

    pub(crate) fn state_any(
        &mut self,
        command_type: TypeId,
        command_name: &'static str,
        args: &dyn Any,
        cx: &Context,
    ) -> Result<Option<State>> {
        Ok(self
            .claim_any(command_type, command_name, args, cx)?
            .map(Claim::into_state))
    }

    pub(crate) fn claim_any(
        &mut self,
        command_type: TypeId,
        command_name: &'static str,
        args: &dyn Any,
        cx: &Context,
    ) -> Result<Option<Claim>> {
        if let Some(claim) = self.responder_claim(command_type, command_name, args, cx)? {
            return Ok(Some(claim.claim));
        }

        let service_order_base = self.responders.len();
        for (service_index, service) in self.services.iter_mut().enumerate() {
            if let Some(claim) = service.claim(self.store, command_type, command_name, args, cx)? {
                return Ok(Some(claim.with_order(service_order_base + service_index)));
            }
        }

        Ok(None)
    }

    pub(crate) fn claim_on(
        &mut self,
        route: Route,
        command_type: TypeId,
        command_name: &'static str,
        args: &dyn Any,
        cx: &Context,
    ) -> Result<Option<Claim>> {
        match route {
            Route::Chain => self.claim_any(command_type, command_name, args, cx),
            Route::Responder(identity) => Ok(self
                .responder_claim_exact(identity, command_type, command_name, args, cx)?
                .map(|claim| claim.claim)),
            Route::Service(name) => {
                for (service_index, service) in self.services.iter_mut().enumerate() {
                    if let Some(claim) = service.claim_exact(
                        self.store,
                        name,
                        command_type,
                        command_name,
                        args,
                        cx,
                    )? {
                        return Ok(Some(
                            claim.with_order(self.responders.len() + service_index),
                        ));
                    }
                }
                Ok(None)
            }
        }
    }

    pub(crate) fn invoke<C: Command>(
        &mut self,
        args: C::Args,
        cx: &mut Context,
    ) -> Option<Response<C::Output>> {
        self.invoke_any(TypeId::of::<C>(), C::NAME, Box::new(args), cx)
            .map(|response| response.into_response(C::NAME))
    }

    pub(crate) fn invoke_any(
        &mut self,
        command_type: TypeId,
        command_name: &'static str,
        args: Box<dyn Any + Send>,
        cx: &mut Context,
    ) -> Option<AnyResponse> {
        match self.responder_claim(command_type, command_name, args.as_ref(), cx) {
            Ok(Some(claim)) => match claim.claim.state.availability {
                command::Availability::Hidden => unreachable!("hidden targets are not claims"),
                command::Availability::Disabled => {
                    return Some(AnyResponse::failed(Error::Disabled {
                        command: command_name,
                    }));
                }
                command::Availability::Enabled => {
                    return Some(
                        self.responders[claim.responder].targets[claim.target].invoke_any(
                            self.store.model_mut(),
                            args,
                            cx,
                        ),
                    );
                }
            },
            Ok(None) => {}
            Err(error) => return Some(AnyResponse::failed(error)),
        }

        for service in &mut self.services {
            match service.claim(self.store, command_type, command_name, args.as_ref(), cx) {
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

    pub(crate) fn invoke_on(
        &mut self,
        route: Route,
        command_type: TypeId,
        command_name: &'static str,
        args: Box<dyn Any + Send>,
        cx: &mut Context,
    ) -> Option<AnyResponse> {
        match route {
            Route::Chain => self.invoke_any(command_type, command_name, args, cx),
            Route::Responder(identity) => {
                let claim = match self.responder_claim_exact(
                    identity,
                    command_type,
                    command_name,
                    args.as_ref(),
                    cx,
                ) {
                    Ok(claim) => claim?,
                    Err(error) => return Some(AnyResponse::failed(error)),
                };
                match claim.claim.state.availability {
                    command::Availability::Hidden => unreachable!("hidden targets are not claims"),
                    command::Availability::Disabled => Some(AnyResponse::failed(Error::Disabled {
                        command: command_name,
                    })),
                    command::Availability::Enabled => Some(
                        self.responders[claim.responder].targets[claim.target].invoke_any(
                            self.store.model_mut(),
                            args,
                            cx,
                        ),
                    ),
                }
            }
            Route::Service(name) => {
                for service in &mut self.services {
                    match service.claim_exact(
                        self.store,
                        name,
                        command_type,
                        command_name,
                        args.as_ref(),
                        cx,
                    ) {
                        Ok(Some(claim)) => {
                            if !claim.state().is_enabled() {
                                return Some(AnyResponse::failed(Error::Disabled {
                                    command: command_name,
                                }));
                            }
                            return Some(
                                service
                                    .invoke_exact(
                                        self.store,
                                        name,
                                        command_type,
                                        command_name,
                                        args,
                                        cx,
                                    )
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
    }

    pub(crate) fn notify<N: Notification>(&mut self, payload: &N::Payload) -> Reaction {
        self.notify_any(TypeId::of::<N>(), payload)
    }

    fn notify_any(&mut self, notification_type: TypeId, payload: &dyn Any) -> Reaction {
        let mut reaction = Reaction::ignored();

        for responder in &self.responders {
            for listener in responder
                .listeners
                .iter()
                .filter(|listener| listener.handles_type(notification_type))
            {
                reaction = reaction.then(listener.notify_any(self.store.model_mut(), payload));
            }
        }

        reaction
    }

    fn responder_claim(
        &mut self,
        command_type: TypeId,
        command_name: &'static str,
        args: &dyn Any,
        cx: &Context,
    ) -> Result<Option<TargetClaim>> {
        for (responder_index, responder) in self.responders.iter().enumerate() {
            let mut claim = None;

            for (target_index, target) in responder
                .targets
                .iter()
                .enumerate()
                .filter(|(_, target)| target.handles_type(command_type))
            {
                let state = target.state_any(self.store.model_mut(), args, cx)?;
                if state.is_hidden() {
                    continue;
                }

                if claim.is_some() {
                    return Err(Error::AmbiguousTarget {
                        command: command_name,
                        responder: responder.name,
                    });
                }

                let provenance = Provenance::new(responder.kind, responder.name, responder_index);
                claim = Some(TargetClaim {
                    responder: responder_index,
                    target: target_index,
                    claim: Claim::new(provenance, state),
                });
            }

            if claim.is_some() {
                return Ok(claim);
            }
        }

        Ok(None)
    }

    fn responder_claim_exact(
        &mut self,
        identity: interaction::Id,
        command_type: TypeId,
        command_name: &'static str,
        args: &dyn Any,
        cx: &Context,
    ) -> Result<Option<TargetClaim>> {
        let Some(responder_index) = self
            .responders
            .iter()
            .position(|responder| responder.identity() == identity)
        else {
            return Ok(None);
        };
        let responder = self.responders[responder_index];
        let mut claim = None;

        for (target_index, target) in responder
            .targets
            .iter()
            .enumerate()
            .filter(|(_, target)| target.handles_type(command_type))
        {
            let state = target.state_any(self.store.model_mut(), args, cx)?;
            if state.is_hidden() {
                continue;
            }
            if claim.is_some() {
                return Err(Error::AmbiguousTarget {
                    command: command_name,
                    responder: responder.name,
                });
            }
            claim = Some(TargetClaim {
                responder: responder_index,
                target: target_index,
                claim: Claim::new(
                    Provenance::new(responder.kind, responder.name, responder_index),
                    state,
                ),
            });
        }

        Ok(claim)
    }
}
