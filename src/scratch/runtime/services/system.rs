use std::any::{Any, TypeId};

use crate::scratch::{
    context, error::Result, responder, response::AnyResponse, session, state, timeline,
};

use super::{Services, target};

const RESPONDER_NAME: &str = "system";

struct ServiceContext<'a, 'services, M: state::State> {
    services: &'a mut Services<'services, M>,
    store: &'a mut state::Store<M>,
}

impl<M: state::State> target::Provider<timeline::Undo> for ServiceContext<'_, '_, M> {
    type Target<'target>
        = timeline::Service<'target, M>
    where
        Self: 'target;

    fn target(&mut self) -> Self::Target<'_> {
        timeline::Service::new(self.store, &mut *self.services.timeline)
    }
}

impl<M: state::State> target::Provider<timeline::Redo> for ServiceContext<'_, '_, M> {
    type Target<'target>
        = timeline::Service<'target, M>
    where
        Self: 'target;

    fn target(&mut self) -> Self::Target<'_> {
        timeline::Service::new(self.store, &mut *self.services.timeline)
    }
}

impl<M: state::State> target::Provider<session::CloseWindow> for ServiceContext<'_, '_, M> {
    type Target<'target>
        = session::Service<'target>
    where
        Self: 'target;

    fn target(&mut self) -> Self::Target<'_> {
        session::Service::new(
            &mut *self.services.session,
            &mut *self.services.composition,
            &mut *self.services.diagnostics,
            self.services.window,
        )
    }
}

impl<M: state::State> target::Provider<session::OpenCommandPalette> for ServiceContext<'_, '_, M> {
    type Target<'target>
        = session::Service<'target>
    where
        Self: 'target;

    fn target(&mut self) -> Self::Target<'_> {
        session::Service::new(
            &mut *self.services.session,
            &mut *self.services.composition,
            &mut *self.services.diagnostics,
            self.services.window,
        )
    }
}

pub(super) fn claim<M: state::State>(
    services: &mut Services<'_, M>,
    store: &mut state::Store<M>,
    command_type: TypeId,
    command_name: &'static str,
    args: &dyn Any,
    cx: &context::Context,
) -> Result<Option<responder::Claim>> {
    let targets = targets();
    let mut service = ServiceContext { services, store };

    Ok(target::claim(
        RESPONDER_NAME,
        &targets,
        &mut service,
        command_type,
        command_name,
        args,
        cx,
    )?
    .map(|claim| {
        responder::Claim::service(responder::Kind::Framework, RESPONDER_NAME, claim.state)
    }))
}

pub(super) fn invoke<M: state::State>(
    services: &mut Services<'_, M>,
    store: &mut state::Store<M>,
    command_type: TypeId,
    command_name: &'static str,
    args: Box<dyn Any + Send>,
    cx: &mut context::Context,
) -> Option<AnyResponse> {
    let targets = targets();
    let mut service = ServiceContext { services, store };

    target::invoke(
        RESPONDER_NAME,
        &targets,
        &mut service,
        command_type,
        command_name,
        args,
        cx,
    )
}

fn targets<'a, 'services, M: state::State>()
-> [target::AnyTarget<ServiceContext<'a, 'services, M>>; 4] {
    [
        target::AnyTarget::for_provider::<timeline::Undo>(),
        target::AnyTarget::for_provider::<timeline::Redo>(),
        target::AnyTarget::for_provider::<session::CloseWindow>(),
        target::AnyTarget::for_provider::<session::OpenCommandPalette>(),
    ]
}
