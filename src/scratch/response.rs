use std::any::Any;

use super::error::{Error, Result};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Effect {
    None,
    Repaint,
    ClosePopup,
    OpenFileDialog,
    SaveFileDialog,
    Batch(Vec<Effect>),
}

impl Default for Effect {
    fn default() -> Self {
        Self::None
    }
}

impl Effect {
    pub fn then(self, next: Self) -> Self {
        let mut effects = Vec::new();
        collect_effects(self, &mut effects);
        collect_effects(next, &mut effects);

        match effects.len() {
            0 => Self::None,
            1 => effects.pop().expect("length was checked"),
            _ => Self::Batch(effects),
        }
    }

    pub fn is_none(&self) -> bool {
        matches!(self, Self::None)
    }

    pub fn contains(&self, effect: &Effect) -> bool {
        match self {
            Self::Batch(effects) => effects.iter().any(|item| item.contains(effect)),
            _ => self == effect,
        }
    }
}

fn collect_effects(effect: Effect, effects: &mut Vec<Effect>) {
    match effect {
        Effect::None => {}
        Effect::Batch(batch) => {
            for effect in batch {
                collect_effects(effect, effects);
            }
        }
        effect => {
            if !effects.contains(&effect) {
                effects.push(effect);
            }
        }
    }
}

pub struct Response<O: Send + 'static> {
    pub(super) output: Result<O>,
    pub(super) effect: Effect,
    changed: bool,
}

impl<O: Send + 'static> Response<O> {
    pub(super) fn output(output: O) -> Self {
        Self {
            output: Ok(output),
            effect: Effect::None,
            changed: false,
        }
    }

    pub(super) fn changed(output: O) -> Self {
        Self {
            output: Ok(output),
            effect: Effect::None,
            changed: true,
        }
    }

    pub(super) fn failed(error: Error) -> Self {
        Self {
            output: Err(error),
            effect: Effect::None,
            changed: false,
        }
    }

    pub(super) fn with_effect(mut self, effect: Effect) -> Self {
        self.effect = effect;
        self
    }

    pub(super) fn is_ok(&self) -> bool {
        self.output.is_ok()
    }

    pub(super) fn changed_state(&self) -> bool {
        self.changed
    }

    pub(super) fn mark_changed(&mut self) {
        self.changed = true;
    }

    pub(super) fn output_ref(&self) -> Option<&O> {
        self.output.as_ref().ok()
    }
}

pub(super) struct AnyResponse {
    output: Result<Box<dyn Any + Send>>,
    effect: Effect,
    changed: bool,
}

impl AnyResponse {
    pub(super) fn from_response<O: Send + 'static>(response: Response<O>) -> Self {
        Self {
            output: response
                .output
                .map(|output| Box::new(output) as Box<dyn Any + Send>),
            effect: response.effect,
            changed: response.changed,
        }
    }

    pub(super) fn failed(error: Error) -> Self {
        Self {
            output: Err(error),
            effect: Effect::None,
            changed: false,
        }
    }

    pub(super) fn into_response<O: Send + 'static>(self, command: &'static str) -> Response<O> {
        let output = match self.output {
            Ok(output) => output
                .downcast::<O>()
                .map(|output| *output)
                .map_err(|_| Error::OutputMismatch { command }),
            Err(error) => Err(error),
        };

        Response {
            output,
            effect: self.effect,
            changed: self.changed,
        }
    }

    pub(super) fn effect(&self) -> Effect {
        self.effect.clone()
    }

    pub(super) fn changed_state(&self) -> bool {
        self.changed
    }

    pub(super) fn mark_changed(&mut self) {
        self.changed = true;
    }

    pub(super) fn is_ok(&self) -> bool {
        self.output.is_ok()
    }

    pub(super) fn output_any(&self) -> Option<&(dyn Any + Send)> {
        self.output.as_ref().ok().map(|output| output.as_ref())
    }

    pub(super) fn into_result(self) -> Result<()> {
        self.output.map(|_| ())
    }
}
