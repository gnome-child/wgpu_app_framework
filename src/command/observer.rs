use std::{
    any::{Any, TypeId},
    collections::HashMap,
};

use super::super::{
    context::Source,
    error::{Error, Result},
    response::{AnyResponse, Effect, Response},
    state,
};
use super::Command;
#[derive(Debug, Clone)]
pub struct Observation {
    source: Source,
    effect: Effect,
    command_changed: bool,
    changed: bool,
}

impl Observation {
    pub(crate) fn new(source: Source, effect: Effect, command_changed: bool) -> Self {
        Self {
            source,
            effect,
            command_changed,
            changed: false,
        }
    }

    pub fn source(&self) -> Source {
        self.source
    }

    pub fn effect(&self) -> &Effect {
        &self.effect
    }

    pub fn command_changed_state(&self) -> bool {
        self.command_changed
    }

    pub fn changed_state(&self) -> bool {
        self.changed
    }

    pub fn mark_changed(&mut self) {
        self.changed = true;
    }
}

pub struct Observers<M: state::State> {
    observers: HashMap<TypeId, Vec<Observer<M>>>,
}

struct Observer<M: state::State> {
    observe: ObserverFn<M>,
}

type ObserverFn<M> = Box<dyn FnMut(&mut M, &(dyn Any + Send), &mut Observation) -> Result<()>>;

impl<M: state::State> Default for Observers<M> {
    fn default() -> Self {
        Self {
            observers: HashMap::new(),
        }
    }
}

impl<M: state::State> Observers<M> {
    pub fn observe<C>(
        &mut self,
        mut callback: impl FnMut(&mut M, &C::Output, &mut Observation) + 'static,
    ) -> &mut Self
    where
        C: Command,
    {
        self.observers
            .entry(TypeId::of::<C>())
            .or_default()
            .push(Observer {
                observe: Box::new(move |model, output, observation| {
                    let output = output
                        .downcast_ref::<C::Output>()
                        .ok_or(Error::OutputMismatch { command: C::NAME })?;
                    callback(model, output, observation);
                    Ok(())
                }),
            });

        self
    }

    pub(crate) fn observe_response<C>(
        &mut self,
        model: &mut M,
        response: &Response<C::Output>,
        source: Source,
    ) -> Result<bool>
    where
        C: Command,
    {
        let Some(output) = response.output_ref() else {
            return Ok(false);
        };

        let mut observation =
            Observation::new(source, response.effect.clone(), response.changed_state());
        self.observe_output(TypeId::of::<C>(), model, output, &mut observation)?;

        Ok(observation.changed_state())
    }

    pub(crate) fn observe_any(
        &mut self,
        command_type: TypeId,
        model: &mut M,
        response: &AnyResponse,
        source: Source,
    ) -> Result<bool> {
        let Some(output) = response.output_any() else {
            return Ok(false);
        };

        let mut observation = Observation::new(source, response.effect(), response.changed_state());
        self.observe_output(command_type, model, output, &mut observation)?;

        Ok(observation.changed_state())
    }

    fn observe_output(
        &mut self,
        command_type: TypeId,
        model: &mut M,
        output: &(dyn Any + Send),
        observation: &mut Observation,
    ) -> Result<()> {
        let Some(observers) = self.observers.get_mut(&command_type) else {
            return Ok(());
        };

        for observer in observers {
            (observer.observe)(model, output, observation)?;
        }

        Ok(())
    }
}
