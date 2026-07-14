pub mod builder;
mod chain;
mod kind;
mod path;
mod scope;

pub use builder::Builder;
pub use chain::Chain;
pub(crate) use chain::Service;
pub(crate) use chain::{Claim, Provenance, Route};
pub use kind::Kind;
pub(crate) use path::{Path, Traversal};
pub(crate) use scope::Scope;

use super::{identity, notification, state, target::AnyTarget};

pub struct Responder<M: state::State> {
    pub(super) kind: Kind,
    pub(super) name: &'static str,
    identity: identity::Id,
    pub(super) targets: Vec<AnyTarget<M>>,
    pub(super) listeners: Vec<notification::AnyListener<M>>,
}

impl<M: state::State> Responder<M> {
    pub(super) fn new(kind: Kind, name: &'static str) -> Self {
        Self {
            kind,
            name,
            identity: identity::Id::new(name),
            targets: Vec::new(),
            listeners: Vec::new(),
        }
    }

    pub(super) fn identity(&self) -> identity::Id {
        self.identity
    }
}
