use crate::response::{self, AnyResponse};

pub(in crate::runtime) struct Outcome {
    pub(in crate::runtime) response: AnyResponse,
    pub(in crate::runtime) changed_state: bool,
    pub(in crate::runtime) effect: response::Effect,
}

impl Outcome {
    pub(in crate::runtime::transaction) fn new(
        response: AnyResponse,
        changed_state: bool,
        effect: response::Effect,
    ) -> Self {
        Self {
            response,
            changed_state,
            effect,
        }
    }
}
