use std::marker::PhantomData;

use super::{AnyTrigger, Listing, State};
use crate::responder;

/// Global registry discovery, used by command-world surfaces such as the
/// palette. The uninhabited marker keeps discovery policy in the type.
pub(crate) enum Global {}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(in crate::command) enum Route {
    Chain,
}

pub(crate) struct Candidates<P> {
    entries: Vec<Candidate>,
    provider: PhantomData<fn() -> P>,
}

pub(in crate::command) struct Candidate {
    registration_index: usize,
    trigger: AnyTrigger,
    listing: Listing,
    route: Route,
}

pub(crate) struct ResolvedActions<P> {
    entries: Vec<ResolvedAction>,
    provider: PhantomData<fn() -> P>,
}

#[derive(Clone)]
pub(crate) struct ResolvedAction {
    registration_index: usize,
    command_type: std::any::TypeId,
    command_name: &'static str,
    trigger: AnyTrigger,
    state: State,
    claim: responder::Claim,
    listing: Listing,
}

impl<P> Candidates<P> {
    pub(in crate::command) fn new(entries: Vec<Candidate>) -> Self {
        Self {
            entries,
            provider: PhantomData,
        }
    }

    pub(in crate::command) fn into_entries(self) -> Vec<Candidate> {
        self.entries
    }
}

impl Candidate {
    pub(in crate::command) fn new(
        registration_index: usize,
        trigger: AnyTrigger,
        listing: Listing,
        route: Route,
    ) -> Self {
        Self {
            registration_index,
            trigger,
            listing,
            route,
        }
    }

    pub(in crate::command) fn registration_index(&self) -> usize {
        self.registration_index
    }

    pub(in crate::command) fn trigger(&self) -> &AnyTrigger {
        &self.trigger
    }

    pub(in crate::command) fn into_trigger(self) -> AnyTrigger {
        self.trigger
    }

    pub(in crate::command) fn listing(&self) -> Listing {
        self.listing
    }

    pub(in crate::command) fn route(&self) -> Route {
        self.route
    }
}

impl<P> ResolvedActions<P> {
    pub(in crate::command) fn new(entries: Vec<ResolvedAction>) -> Self {
        Self {
            entries,
            provider: PhantomData,
        }
    }
}

impl<P> IntoIterator for ResolvedActions<P> {
    type Item = ResolvedAction;
    type IntoIter = std::vec::IntoIter<ResolvedAction>;

    fn into_iter(self) -> Self::IntoIter {
        self.entries.into_iter()
    }
}

impl ResolvedAction {
    pub(in crate::command) fn new(
        registration_index: usize,
        trigger: AnyTrigger,
        state: State,
        claim: responder::Claim,
        listing: Listing,
    ) -> Self {
        Self {
            registration_index,
            command_type: trigger.command_type(),
            command_name: trigger.command_name(),
            trigger,
            state,
            claim,
            listing,
        }
    }

    pub(crate) fn registration_index(&self) -> usize {
        self.registration_index
    }

    pub(crate) fn command_type(&self) -> std::any::TypeId {
        self.command_type
    }

    pub(crate) fn command_name(&self) -> &'static str {
        self.command_name
    }

    pub(crate) fn trigger(&self) -> AnyTrigger {
        self.trigger.clone()
    }

    pub(crate) fn history_group(&self) -> Option<super::HistoryGroup> {
        self.trigger.history_group()
    }

    pub(crate) fn state(&self) -> &State {
        &self.state
    }

    pub(crate) fn claim(&self) -> &responder::Claim {
        &self.claim
    }

    pub(crate) fn listing(&self) -> Listing {
        self.listing
    }
}
