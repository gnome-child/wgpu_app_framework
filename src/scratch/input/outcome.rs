use super::super::response;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Outcome {
    handled: bool,
    changed_state: bool,
    effect: response::Effect,
}

impl Outcome {
    pub(in crate::scratch) fn handled(changed_state: bool, effect: response::Effect) -> Self {
        Self {
            handled: true,
            changed_state,
            effect,
        }
    }

    pub(in crate::scratch) fn ignored() -> Self {
        Self {
            handled: false,
            changed_state: false,
            effect: response::Effect::None,
        }
    }

    pub fn is_handled(&self) -> bool {
        self.handled
    }

    pub fn changed_state(&self) -> bool {
        self.changed_state
    }

    pub fn effect(&self) -> &response::Effect {
        &self.effect
    }
}
