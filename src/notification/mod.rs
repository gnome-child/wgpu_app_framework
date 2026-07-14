use std::any::{Any, TypeId};

use crate::{response, state, target::Selector};

mod window;

/// A past-tense framework fact.
///
/// Notifications are not commands: they have no availability, no undo policy,
/// no registry spec, and no advertised shortcut/menu/palette surface.
pub trait Notification: 'static + Sized {
    type Payload: Send + 'static;

    const NAME: &'static str;
}

/// A value that wants to hear a notification.
pub trait Listener<N: Notification> {
    fn notify(&mut self, payload: &N::Payload) -> Reaction;
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Reaction {
    changed_state: bool,
    effect: response::Effect,
}

type NotifyThunk<M> = dyn Fn(&mut M, &dyn Any) -> Reaction;

pub(crate) struct AnyListener<M: state::State> {
    notification_type: TypeId,
    notify: Box<NotifyThunk<M>>,
}

impl Reaction {
    pub fn ignored() -> Self {
        Self {
            changed_state: false,
            effect: response::Effect::None,
        }
    }

    pub fn changed() -> Self {
        Self {
            changed_state: true,
            effect: response::Effect::None,
        }
    }

    pub fn with_effect(mut self, effect: response::Effect) -> Self {
        self.effect = effect;
        self
    }

    pub fn then(self, next: Self) -> Self {
        Self {
            changed_state: self.changed_state || next.changed_state,
            effect: self.effect.then(next.effect),
        }
    }

    pub fn changed_state(&self) -> bool {
        self.changed_state
    }

    pub fn effect(&self) -> &response::Effect {
        &self.effect
    }
}

impl<M: state::State> AnyListener<M> {
    pub(crate) fn new<N, T>(selector: Selector<M, T>) -> Self
    where
        N: Notification,
        T: Listener<N> + 'static,
    {
        Self {
            notification_type: TypeId::of::<N>(),
            notify: Box::new(move |model, payload| {
                let payload = payload
                    .downcast_ref::<N::Payload>()
                    .expect("notification payload type matched listener registration");
                let listener = selector(model);

                <T as Listener<N>>::notify(listener, payload)
            }),
        }
    }

    pub(crate) fn handles_type(&self, notification_type: TypeId) -> bool {
        self.notification_type == notification_type
    }

    pub(crate) fn notify_any(&self, model: &mut M, payload: &dyn Any) -> Reaction {
        (self.notify)(model, payload)
    }
}
