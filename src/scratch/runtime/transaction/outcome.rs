use crate::scratch::response::{self, AnyResponse};

pub(in crate::scratch::runtime) struct Outcome {
    pub(in crate::scratch::runtime) response: AnyResponse,
    pub(in crate::scratch::runtime) changed_state: bool,
    pub(in crate::scratch::runtime) effect: response::Effect,
}

impl Outcome {
    pub(in crate::scratch::runtime::transaction) fn new(
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
