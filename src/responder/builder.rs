use std::{marker::PhantomData, rc::Rc};

use super::{Chain, Kind, Responder, Scope};
use crate::{
    command::Command,
    notification::{self, Notification},
    session, state,
    target::{AnyTarget, Target},
};

pub struct Builder<M: state::State> {
    specs: Vec<Responder<M>>,
}

pub struct Object<'builder, M: state::State, T: 'static> {
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
    pub fn app(&mut self) -> Object<'_, M, M> {
        self.object_with_kind(Kind::App, "app", |model| model)
    }

    pub fn object<T>(
        &mut self,
        name: &'static str,
        selector: impl for<'a> Fn(&'a mut M) -> &'a mut T + 'static,
    ) -> Object<'_, M, T>
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
    ) -> Object<'_, M, T>
    where
        T: 'static,
    {
        self.specs.push(Responder::new(kind, name));

        Object {
            spec: self
                .specs
                .last_mut()
                .expect("a responder spec was just pushed"),
            selector: Rc::new(selector),
            _target: PhantomData,
        }
    }

    pub(crate) fn chain<'a>(&'a self, store: &'a mut state::Store<M>) -> Chain<'a, M> {
        self.chain_for(store, None)
    }

    pub(crate) fn chain_for<'a>(
        &'a self,
        store: &'a mut state::Store<M>,
        focus: Option<session::Focus>,
    ) -> Chain<'a, M> {
        self.chain_for_scope(store, Scope::focused(focus))
    }

    pub(crate) fn chain_for_scope<'a>(
        &'a self,
        store: &'a mut state::Store<M>,
        scope: Scope,
    ) -> Chain<'a, M> {
        let mut responders = self
            .specs
            .iter()
            .enumerate()
            .filter(|(_, spec)| spec.matches_scope(scope))
            .map(|(index, spec)| (spec.kind.rank(), index, spec))
            .collect::<Vec<_>>();

        responders.sort_by_key(|(rank, index, _)| (*rank, *index));

        let responders = responders
            .into_iter()
            .map(|(_, _, responder)| responder)
            .collect();

        Chain::nearest_first(store, responders)
    }

    pub(crate) fn target_types_for(
        &self,
        identity: crate::interaction::Id,
    ) -> Vec<std::any::TypeId> {
        self.specs
            .iter()
            .find(|responder| responder.identity() == identity)
            .map(|responder| {
                responder
                    .targets
                    .iter()
                    .map(AnyTarget::command_type)
                    .collect()
            })
            .unwrap_or_default()
    }

    pub(crate) fn app_identity() -> crate::interaction::Id {
        crate::interaction::Id::new("app")
    }
}

impl<M, T> Object<'_, M, T>
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

    pub fn listen<N: Notification>(&mut self) -> &mut Self
    where
        T: notification::Listener<N>,
    {
        self.spec
            .listeners
            .push(notification::AnyListener::new::<N, T>(Rc::clone(
                &self.selector,
            )));
        self
    }
}
