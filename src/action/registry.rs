use std::collections::{HashMap, HashSet};

use crate::window;

use super::{Action, Context, Effect, Id, Invocation, Scope, State};
use super::{definition, state};

#[derive(Debug)]
pub struct Registry<T = ()> {
    actions: HashMap<Id, Action<T>>,
    states: HashMap<(Id, Context), State>,
    executing: HashSet<(Id, Context)>,
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
        self.executing.retain(|(_, context)| {
            context.window_id() != window || matches!(context.scope(), Scope::Window)
        });
    }

    pub fn state(&self, id: Id, context: Context) -> State {
        if !self.actions.contains_key(&id) {
            return State::disabled();
        }

        let executing = self.executing.contains(&(id, context.clone()));
        if let Some(state) = self.states.get(&(id, context.clone())) {
            return state::with_active_overlay(*state, executing);
        }

        if matches!(context.scope(), Scope::Path(_)) {
            let fallback = Context::window(context.window_id());

            if let Some(state) = self.states.get(&(id, fallback)) {
                return state::with_active_overlay(*state, executing);
            }
        }

        state::with_active_overlay(State::enabled(), executing)
    }

    pub fn can_invoke(&self, id: Id, context: Context) -> bool {
        self.state(id, context).is_enabled()
    }

    pub fn execute(&mut self, invocation: Invocation) -> Option<Effect<T>> {
        let id = invocation.action();
        let context = invocation.context().clone();

        if !self.can_invoke(id, context.clone()) {
            return None;
        }

        self.executing.insert((id, context.clone()));
        let effect = self
            .actions
            .get(&id)
            .map(|action| definition::invoke(action, invocation));
        self.executing.remove(&(id, context));

        effect
    }
}

impl<T> Default for Registry<T> {
    fn default() -> Self {
        Self {
            actions: HashMap::new(),
            states: HashMap::new(),
            executing: HashSet::new(),
        }
    }
}
