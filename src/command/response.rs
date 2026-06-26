use super::{
    Command, Target,
    call::{Call, Scope, Source},
    effect::{Effect, RuntimeEffect, Task},
    output::Output,
    registry::Rejection,
};

#[derive(Debug, PartialEq, Eq)]
pub struct Response<O: Output> {
    output: O,
    effects: Vec<Effect>,
}

impl<O: Output> Response<O> {
    pub fn output(output: O) -> Self {
        Self {
            output,
            effects: Vec::new(),
        }
    }

    pub fn runtime(output: O, effect: RuntimeEffect) -> Self {
        Self {
            output,
            effects: vec![Effect::Runtime(effect)],
        }
    }

    pub fn with_effect(mut self, effect: Effect) -> Self {
        self.effects.push(effect);
        self
    }

    pub fn then_call<C: Command>(self, call: Call<C>) -> Self {
        self.with_effect(Effect::call(call))
    }

    pub fn then_task(self, task: Task) -> Self {
        self.with_effect(Effect::Task(task))
    }

    pub fn pipe<C, TTarget>(self, source: Source, scope: Scope) -> Result<Response<()>, Rejection>
    where
        C: Command,
        TTarget: Target<C> + 'static,
        C::Args: From<O>,
    {
        self.pipe_with::<C, TTarget>(source, scope, C::Args::from)
    }

    pub fn pipe_with<C, TTarget>(
        self,
        source: Source,
        scope: Scope,
        args: impl FnOnce(O) -> C::Args,
    ) -> Result<Response<()>, Rejection>
    where
        C: Command,
        TTarget: Target<C> + 'static,
    {
        let call = Call::<C>::new::<TTarget>(args(self.output))
            .map_err(|error| Rejection::InvalidArgs {
                command: C::NAME,
                error,
            })?
            .with_source(source)
            .with_scope(scope);

        Ok(Response {
            output: (),
            effects: self
                .effects
                .into_iter()
                .chain([Effect::Call(super::call::Any::new(call))])
                .collect(),
        })
    }

    pub fn effects(&self) -> &[Effect] {
        &self.effects
    }

    pub fn into_output(self) -> O {
        self.output
    }

    pub fn into_parts(self) -> (O, Vec<Effect>) {
        (self.output, self.effects)
    }
}

impl Response<()> {
    pub fn none() -> Self {
        Self::output(())
    }

    pub fn task(task: Task) -> Self {
        Self::none().then_task(task)
    }
}

impl<O: Output> From<O> for Response<O> {
    fn from(output: O) -> Self {
        Self::output(output)
    }
}
