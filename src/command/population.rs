use std::{any::TypeId, collections::HashMap, marker::PhantomData};

use super::{AnyTrigger, HistoryGroup, Listing, Registry, Standard, State};
use crate::{context as command_context, responder, state};

/// Registry-wide discovery for the captured task described by the palette.
pub(crate) enum Palette {}

/// Nearest-owner discovery for a captured inspection or editing path.
pub(crate) enum Context {}

/// Stable registered-standard discovery for a live task chain.
pub(crate) enum Bar {}

/// The single command-population owner. Surface markers select policy without
/// conflating the different membership, traversal, and ordering contracts.
pub(crate) struct Population<'a> {
    registry: &'a Registry,
}

pub(crate) struct Candidates<P> {
    entries: Vec<Candidate>,
    policy: PhantomData<fn() -> P>,
}

pub(in crate::command) struct Candidate {
    registration_index: usize,
    trigger: AnyTrigger,
    listing: Listing,
    route: responder::Route,
}

pub(crate) struct ResolvedActions<P> {
    entries: Vec<ResolvedAction>,
    policy: PhantomData<fn() -> P>,
}

#[derive(Clone)]
pub(crate) struct ResolvedAction {
    registration_index: usize,
    command_type: TypeId,
    command_name: &'static str,
    trigger: AnyTrigger,
    state: State,
    claim: responder::Claim,
    listing: Listing,
    route: responder::Route,
}

pub(crate) struct BarActions {
    entries: Vec<BarAction>,
}

#[derive(Clone)]
pub(crate) struct BarAction {
    command_type: TypeId,
    standard: Option<Standard>,
    trigger: AnyTrigger,
    state: State,
}

/// Complete conventional-bar projection. It contains ordinary command
/// actions, grouped by the platform topology, but no view vocabulary.
pub(crate) struct BarProjection {
    categories: Vec<BarCategory>,
}

pub(crate) struct BarCategory {
    id: &'static str,
    label: &'static str,
    sections: Vec<Vec<BarEntry>>,
}

#[derive(Clone)]
pub(crate) struct BarEntry {
    action: BarAction,
    show_shortcut: bool,
}

impl<'a> Population<'a> {
    pub(in crate::command) fn new(registry: &'a Registry) -> Self {
        Self { registry }
    }

    pub(crate) fn palette_candidates(&self) -> Candidates<Palette> {
        Candidates::new(
            self.registry
                .order
                .iter()
                .enumerate()
                .filter_map(|(registration_index, command_type)| {
                    let command = self.registry.commands.get(command_type)?;
                    command.accepts_shortcut_args().then(|| {
                        Candidate::new(
                            registration_index,
                            command.unit_trigger(),
                            command.spec.listing,
                            responder::Route::Chain,
                        )
                    })
                })
                .collect(),
        )
    }

    pub(crate) fn context_candidates(
        &self,
        binding: Option<AnyTrigger>,
        targets: impl IntoIterator<Item = (TypeId, responder::Route)>,
    ) -> Candidates<Context> {
        let mut entries = Vec::new();
        let mut seen = Vec::new();

        if let Some(trigger) = binding
            && let Some(command) = self.registry.commands.get(&trigger.command_type())
        {
            seen.push(trigger.command_type());
            entries.push(Candidate::new(
                entries.len(),
                trigger,
                command.spec.listing,
                responder::Route::Chain,
            ));
        }

        for (command_type, route) in targets {
            if seen.contains(&command_type) {
                continue;
            }
            let Some(command) = self.registry.commands.get(&command_type) else {
                continue;
            };
            if !command.accepts_shortcut_args() {
                continue;
            }
            seen.push(command_type);
            entries.push(Candidate::new(
                entries.len(),
                command.unit_trigger(),
                command.spec.listing,
                route,
            ));
        }

        Candidates::new(entries)
    }

    pub(crate) fn bar_candidates(&self) -> Candidates<Bar> {
        Candidates::new(
            self.registry
                .order
                .iter()
                .enumerate()
                .filter_map(|(registration_index, command_type)| {
                    let command = self.registry.commands.get(command_type)?;
                    (command.accepts_shortcut_args()
                        && command.spec.participates_in_standard_menu())
                    .then(|| {
                        Candidate::new(
                            registration_index,
                            command.unit_trigger(),
                            command.spec.listing,
                            responder::Route::Chain,
                        )
                    })
                })
                .collect(),
        )
    }

    pub(in crate::command) fn menu_topology(
        &self,
        platform: crate::keymap::Platform,
    ) -> super::menu::Topology {
        let items = self
            .bar_candidates()
            .into_entries()
            .into_iter()
            .filter_map(|candidate| {
                let command = self
                    .registry
                    .commands
                    .get(&candidate.trigger().command_type())?;
                Some(super::menu::Item {
                    command_type: command.command_type,
                    command_name: command.command_name,
                    command_type_name: command.command_type_name,
                    standard: command.spec.standard,
                    placement: command.spec.menu_placement,
                    suppressed: command.spec.menu_suppressed,
                    shortcut_visibility: command.spec.menu_shortcut_visibility,
                })
            })
            .collect();
        super::menu::Topology::resolve(platform, &self.registry.menu_categories, items)
    }

    pub(crate) fn resolve_claimed<P>(
        &self,
        candidates: Candidates<P>,
        chain: &mut responder::Chain<'_, impl state::State>,
        cx: &command_context::Context,
    ) -> ResolvedActions<P> {
        let entries = candidates
            .into_entries()
            .into_iter()
            .filter_map(|candidate| {
                let command = self
                    .registry
                    .commands
                    .get(&candidate.trigger().command_type())?;
                let route = candidate.route();
                let claim = match chain.claim_on(
                    route,
                    command.command_type,
                    command.command_name,
                    candidate.trigger().args(),
                    cx,
                ) {
                    Ok(Some(claim)) => claim,
                    Ok(None) | Err(_) => return None,
                };
                let state = claim.state().clone().with_command(command);
                let registration_index = candidate.registration_index();
                let listing = candidate.listing();
                let trigger = candidate.into_trigger();

                Some(ResolvedAction::new(
                    registration_index,
                    trigger,
                    state,
                    claim,
                    listing,
                    route,
                ))
            })
            .collect();

        ResolvedActions::new(entries)
    }

    pub(crate) fn resolve_bar(
        &self,
        candidates: Candidates<Bar>,
        chain: &mut responder::Chain<'_, impl state::State>,
        cx: &command_context::Context,
    ) -> BarActions {
        let entries = candidates
            .into_entries()
            .into_iter()
            .filter_map(|candidate| {
                let command = self
                    .registry
                    .commands
                    .get(&candidate.trigger().command_type())?;
                let standard = command.spec.standard;
                let state = self.registry.state_any_on(
                    candidate.route(),
                    command.command_type,
                    command.command_name,
                    candidate.trigger().args(),
                    chain,
                    cx,
                );
                Some(BarAction {
                    command_type: command.command_type,
                    standard,
                    trigger: candidate.into_trigger(),
                    state,
                })
            })
            .collect();

        BarActions { entries }
    }

    pub(crate) fn standard_bar(
        &self,
        platform: crate::keymap::Platform,
        chain: &mut responder::Chain<'_, impl state::State>,
        cx: &command_context::Context,
    ) -> BarProjection {
        let topology = self.menu_topology(platform);
        let actions = self
            .resolve_bar(self.bar_candidates(), chain, cx)
            .into_iter()
            .map(|action| (action.command_type(), action))
            .collect::<HashMap<_, _>>();

        let categories = topology
            .categories()
            .iter()
            .filter_map(|category| {
                let sections = category
                    .sections()
                    .iter()
                    .filter_map(|section| {
                        let entries = section
                            .iter()
                            .filter_map(|entry| {
                                actions.get(&entry.command_type()).cloned().map(|action| {
                                    debug_assert_eq!(entry.standard(), action.standard());
                                    BarEntry {
                                        action,
                                        show_shortcut: entry.show_shortcut(),
                                    }
                                })
                            })
                            .collect::<Vec<_>>();
                        (!entries.is_empty()).then_some(entries)
                    })
                    .collect::<Vec<_>>();
                (!sections.is_empty()).then_some(BarCategory {
                    id: category.id(),
                    label: category.label(),
                    sections,
                })
            })
            .collect();

        BarProjection { categories }
    }
}

impl<P> Candidates<P> {
    fn new(entries: Vec<Candidate>) -> Self {
        Self {
            entries,
            policy: PhantomData,
        }
    }

    pub(in crate::command) fn into_entries(self) -> Vec<Candidate> {
        self.entries
    }
}

impl Candidate {
    fn new(
        registration_index: usize,
        trigger: AnyTrigger,
        listing: Listing,
        route: responder::Route,
    ) -> Self {
        Self {
            registration_index,
            trigger,
            listing,
            route,
        }
    }

    fn registration_index(&self) -> usize {
        self.registration_index
    }

    pub(in crate::command) fn trigger(&self) -> &AnyTrigger {
        &self.trigger
    }

    fn into_trigger(self) -> AnyTrigger {
        self.trigger
    }

    fn listing(&self) -> Listing {
        self.listing
    }

    fn route(&self) -> responder::Route {
        self.route
    }
}

impl<P> ResolvedActions<P> {
    fn new(entries: Vec<ResolvedAction>) -> Self {
        Self {
            entries,
            policy: PhantomData,
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
    fn new(
        registration_index: usize,
        trigger: AnyTrigger,
        state: State,
        claim: responder::Claim,
        listing: Listing,
        route: responder::Route,
    ) -> Self {
        Self {
            registration_index,
            command_type: trigger.command_type(),
            command_name: trigger.command_name(),
            trigger,
            state,
            claim,
            listing,
            route,
        }
    }

    pub(crate) fn registration_index(&self) -> usize {
        self.registration_index
    }

    pub(crate) fn command_type(&self) -> TypeId {
        self.command_type
    }

    pub(crate) fn command_name(&self) -> &'static str {
        self.command_name
    }

    pub(crate) fn trigger(&self) -> AnyTrigger {
        self.trigger.clone()
    }

    pub(crate) fn history_group(&self) -> Option<HistoryGroup> {
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

    pub(crate) fn route(&self) -> responder::Route {
        self.route
    }
}

impl IntoIterator for BarActions {
    type Item = BarAction;
    type IntoIter = std::vec::IntoIter<BarAction>;

    fn into_iter(self) -> Self::IntoIter {
        self.entries.into_iter()
    }
}

impl BarAction {
    pub(crate) fn command_type(&self) -> TypeId {
        self.command_type
    }

    pub(crate) fn standard(&self) -> Option<Standard> {
        self.standard
    }

    pub(crate) fn trigger(&self) -> AnyTrigger {
        self.trigger.clone()
    }

    pub(crate) fn state(&self) -> &State {
        &self.state
    }
}

impl BarProjection {
    pub(crate) fn categories(&self) -> &[BarCategory] {
        &self.categories
    }
}

impl BarCategory {
    pub(crate) fn id(&self) -> &'static str {
        self.id
    }

    pub(crate) fn label(&self) -> &'static str {
        self.label
    }

    pub(crate) fn sections(&self) -> &[Vec<BarEntry>] {
        &self.sections
    }
}

impl BarEntry {
    pub(crate) fn action(&self) -> &BarAction {
        &self.action
    }

    pub(crate) fn show_shortcut(&self) -> bool {
        self.show_shortcut
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{command::Command, context::Context as CommandContext, response::Response, target};

    #[derive(Clone)]
    struct Model;

    impl state::State for Model {}

    struct CopyValue;

    impl Command for CopyValue {
        type Args = ();
        type Output = ();

        const NAME: &'static str = "test.copy_value";
    }

    impl target::Target<CopyValue> for Model {
        fn state(&self, _args: &(), _cx: &CommandContext) -> State {
            State::enabled()
        }

        fn invoke(&mut self, _args: (), _cx: &mut CommandContext) -> Response<()> {
            Response::output(())
        }
    }

    #[test]
    fn bar_population_retains_unclaimed_roles_and_reads_the_live_chain() {
        let mut registry = Registry::default();
        registry.register::<CopyValue>(super::super::Spec::standard(Standard::Copy));
        let mut store = state::Store::new(Model);
        let no_targets = responder::Builder::default();
        let mut chain = no_targets.chain(&mut store);
        let population = registry.population();
        let actions = population.resolve_bar(
            population.bar_candidates(),
            &mut chain,
            &CommandContext::default(),
        );
        let action = actions.into_iter().next().expect("registered role remains");
        assert_eq!(action.standard(), Some(Standard::Copy));
        assert!(!action.state().is_enabled());
        drop(chain);

        let mut live_targets = responder::Builder::default();
        live_targets.app().target::<CopyValue>();
        let mut chain = live_targets.chain(&mut store);
        let population = registry.population();
        let actions = population.resolve_bar(
            population.bar_candidates(),
            &mut chain,
            &CommandContext::default(),
        );
        let action = actions.into_iter().next().expect("registered role remains");
        assert!(action.state().is_enabled());
    }
}
