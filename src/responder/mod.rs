mod builder;
mod chain;
mod kind;

pub use builder::Builder;
#[allow(unused_imports)]
pub use builder::Object;
pub use chain::Chain;
pub(crate) use chain::Service;
pub(crate) use chain::{Claim, Provenance};
pub use kind::Kind;

use super::{interaction, session, state, target::AnyTarget};

pub struct Responder<M: state::State> {
    pub(super) kind: Kind,
    pub(super) name: &'static str,
    identity: interaction::Id,
    pub(super) targets: Vec<AnyTarget<M>>,
}

impl<M: state::State> Responder<M> {
    pub(super) fn new(kind: Kind, name: &'static str) -> Self {
        Self {
            kind,
            name,
            identity: interaction::Id::new(name),
            targets: Vec::new(),
        }
    }

    pub(super) fn matches_focus(&self, focus: Option<session::Focus>) -> bool {
        match self.kind {
            Kind::Focused => focus
                .as_ref()
                .and_then(session::Focus::target_id)
                .is_some_and(|target| self.identity == target),
            _ => true,
        }
    }
}
