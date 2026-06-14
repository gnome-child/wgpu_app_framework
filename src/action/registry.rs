use std::collections::{HashMap, HashSet};

use crate::window;

use super::{Action, Context, Effect, Id, Invocation, Scope, Shortcut, State};
use super::{definition, state};

/// Stores command definitions and context-scoped command state.
///
/// Execution is intentionally synchronous and effect-producing. Runtime scheduling, task
/// lifecycles, and background work belong above this registry.
#[derive(Debug)]
pub struct Registry<T = ()> {
    actions: HashMap<Id, Action<T>>,
    states: HashMap<(Id, Context), State>,
    busy: HashSet<(Id, Context)>,
}

impl<T> Registry<T> {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn register(&mut self, action: Action<T>) {
        self.actions.insert(action.id(), action);
    }

    pub fn action(&self, id: Id) -> Option<&Action<T>> {
        self.actions.get(&id)
    }

    pub fn shortcut_action(&self, shortcut: Shortcut) -> Option<Id> {
        self.actions
            .values()
            .find(|action| action.shortcuts().contains(&shortcut))
            .map(Action::id)
    }

    pub fn set_state(&mut self, id: Id, context: Context, state: State) -> bool {
        if self.states.get(&(id, context.clone())) == Some(&state) {
            return false;
        }

        self.states.insert((id, context), state);
        true
    }

    pub fn clear_context_states(&mut self, window: window::Id) {
        self.states.retain(|(_, context), _| {
            context.window_id() != window || matches!(context.scope(), Scope::Window)
        });
    }

    pub fn state(&self, id: Id, context: Context) -> State {
        state::with_busy_overlay(
            self.configured_state(id, context.clone()),
            self.is_busy(id, &context),
        )
    }

    pub fn configured_state(&self, id: Id, context: Context) -> State {
        if !self.actions.contains_key(&id) {
            return State::disabled();
        }

        if let Some(state) = self.states.get(&(id, context.clone())) {
            return *state;
        }

        if matches!(context.scope(), Scope::Path(_)) {
            let fallback = Context::window(context.window_id());

            if let Some(state) = self.states.get(&(id, fallback)) {
                return *state;
            }
        }

        State::enabled()
    }

    pub fn can_invoke(&self, id: Id, context: Context) -> bool {
        let state = self.state(id, context);

        state.is_enabled() && !state.is_busy()
    }

    pub fn set_busy(&mut self, id: Id, context: Context, busy: bool) -> bool {
        if busy {
            self.busy.insert((id, context))
        } else {
            self.busy.remove(&(id, context))
        }
    }

    pub fn execute(&mut self, invocation: Invocation) -> Option<Effect<T>> {
        let id = invocation.action();
        let context = invocation.context().clone();

        if !self.can_invoke(id, context.clone()) {
            return None;
        }

        self.actions
            .get(&id)
            .map(|action| definition::invoke(action, invocation))
    }

    fn is_busy(&self, id: Id, context: &Context) -> bool {
        if self.busy.contains(&(id, context.clone())) {
            return true;
        }

        if matches!(context.scope(), Scope::Path(_)) {
            return self
                .busy
                .contains(&(id, Context::window(context.window_id())));
        }

        false
    }
}

impl<T> Default for Registry<T> {
    fn default() -> Self {
        Self {
            actions: HashMap::new(),
            states: HashMap::new(),
            busy: HashSet::new(),
        }
    }
}
