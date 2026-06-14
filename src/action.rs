use std::collections::{HashMap, HashSet};
use std::fmt;

use crate::{ui, window};

pub struct Action<T = ()> {
    id: Id,
    label: String,
    handler: Box<dyn Fn(Invocation) -> Effect<T>>,
}

impl<T> Action<T> {
    pub fn new(id: Id, label: impl Into<String>) -> Self {
        Self {
            id,
            label: label.into(),
            handler: Box::new(|_| Effect::None),
        }
    }

    pub fn id(&self) -> Id {
        self.id
    }

    pub fn label(&self) -> &str {
        &self.label
    }

    pub fn on_invoke(mut self, handler: impl Fn(Invocation) -> Effect<T> + 'static) -> Self {
        self.handler = Box::new(handler);
        self
    }

    pub fn emit(self, handler: impl Fn(Invocation) -> T + 'static) -> Self {
        self.on_invoke(move |invocation| Effect::Emit(handler(invocation)))
    }

    fn invoke(&self, invocation: Invocation) -> Effect<T> {
        (self.handler)(invocation)
    }
}

impl<T> fmt::Debug for Action<T> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("Action")
            .field("id", &self.id)
            .field("label", &self.label)
            .finish_non_exhaustive()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Id(&'static str);

impl Id {
    pub const fn new(value: &'static str) -> Self {
        Self(value)
    }

    pub const fn as_str(self) -> &'static str {
        self.0
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct State {
    /// Whether the action can be invoked in the resolved context.
    pub enabled: bool,
    /// Whether the action is currently on, selected, or running in the resolved context.
    ///
    /// Completed or historical work should stay in application state unless it represents a
    /// persistent current state.
    pub active: bool,
}

impl State {
    pub const fn enabled() -> Self {
        Self {
            enabled: true,
            active: false,
        }
    }

    pub const fn disabled() -> Self {
        Self {
            enabled: false,
            active: false,
        }
    }

    pub const fn active() -> Self {
        Self {
            enabled: true,
            active: true,
        }
    }
}

impl Default for State {
    fn default() -> Self {
        Self::enabled()
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Context {
    pub window: window::Id,
    pub scope: Scope,
}

impl Context {
    pub fn window(window: window::Id) -> Self {
        Self {
            window,
            scope: Scope::Window,
        }
    }

    pub fn path(window: window::Id, path: ui::Path) -> Self {
        Self {
            window,
            scope: Scope::Path(path),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Scope {
    Path(ui::Path),
    Window,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Source {
    Pointer,
    Programmatic,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Invocation {
    pub action: Id,
    pub source: Source,
    pub context: Context,
}

impl Invocation {
    pub fn new(action: Id, source: Source, context: Context) -> Self {
        Self {
            action,
            source,
            context,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Effect<T> {
    None,
    Emit(T),
    Batch(Vec<T>),
}

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
        self.states
            .retain(|(_, context), _| context.window != window || context.scope == Scope::Window);
        self.executing
            .retain(|(_, context)| context.window != window || context.scope == Scope::Window);
    }

    pub fn state(&self, id: Id, context: Context) -> State {
        if !self.actions.contains_key(&id) {
            return State::disabled();
        }

        let executing = self.executing.contains(&(id, context.clone()));
        if let Some(state) = self.states.get(&(id, context.clone())) {
            return state.with_active_override(executing);
        }

        if matches!(context.scope, Scope::Path(_)) {
            let fallback = Context {
                window: context.window,
                scope: Scope::Window,
            };

            if let Some(state) = self.states.get(&(id, fallback)) {
                return state.with_active_override(executing);
            }
        }

        State::enabled().with_active_override(executing)
    }

    pub fn can_invoke(&self, id: Id, context: Context) -> bool {
        self.state(id, context).enabled
    }

    pub(crate) fn invoke(&self, invocation: Invocation) -> Option<Effect<T>> {
        self.actions
            .get(&invocation.action)
            .map(|action| action.invoke(invocation))
    }

    pub(crate) fn begin_execution(&mut self, id: Id, context: Context) -> bool {
        if !self.actions.contains_key(&id) {
            return false;
        }

        self.executing.insert((id, context))
    }

    pub(crate) fn end_execution(&mut self, id: Id, context: &Context) -> bool {
        self.executing.remove(&(id, context.clone()))
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

impl State {
    fn with_active_override(mut self, active: bool) -> Self {
        self.active |= active;
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const SELECT_ALL: Id = Id::new("select_all");
    const TEXT_BOX: ui::Id = ui::Id::new("text_box");

    #[test]
    fn unregistered_action_is_disabled() {
        let registry = Registry::<()>::new();
        let context = Context::path(window::Id::new(1), ui::Path::from(TEXT_BOX));

        assert_eq!(registry.state(SELECT_ALL, context), State::disabled());
    }

    #[test]
    fn context_specific_state_wins_over_window_fallback() {
        let mut registry = Registry::<()>::new();
        let window = window::Id::new(1);

        registry.register(Action::new(SELECT_ALL, "Select All"));
        registry.set_state(SELECT_ALL, Context::window(window), State::disabled());
        registry.set_state(
            SELECT_ALL,
            Context::path(window, ui::Path::from(TEXT_BOX)),
            State::active(),
        );

        assert_eq!(
            registry.state(SELECT_ALL, Context::path(window, ui::Path::from(TEXT_BOX))),
            State::active()
        );
    }

    #[test]
    fn disabled_action_cannot_invoke() {
        let mut registry = Registry::<()>::new();
        let context = Context::path(window::Id::new(1), ui::Path::from(TEXT_BOX));

        registry.register(Action::new(SELECT_ALL, "Select All"));
        registry.set_state(SELECT_ALL, context.clone(), State::disabled());

        assert!(!registry.can_invoke(SELECT_ALL, context));
    }

    #[test]
    fn clearing_context_states_keeps_window_fallback() {
        let mut registry = Registry::<()>::new();
        let window = window::Id::new(1);

        registry.register(Action::new(SELECT_ALL, "Select All"));
        registry.set_state(SELECT_ALL, Context::window(window), State::disabled());
        registry.set_state(
            SELECT_ALL,
            Context::path(window, ui::Path::from(TEXT_BOX)),
            State::active(),
        );

        registry.clear_context_states(window);

        assert_eq!(
            registry.state(SELECT_ALL, Context::path(window, ui::Path::from(TEXT_BOX))),
            State::disabled()
        );
    }

    #[test]
    fn enabled_action_invokes_behavior_and_emits_event() {
        let mut registry = Registry::<i32>::new();
        let window = window::Id::new(1);

        registry.register(Action::new(SELECT_ALL, "Select All").emit(|_| 7));

        assert_eq!(
            registry.invoke(Invocation::new(
                SELECT_ALL,
                Source::Programmatic,
                Context::window(window)
            )),
            Some(Effect::Emit(7))
        );
    }

    #[test]
    fn batched_action_effect_preserves_event_order() {
        let mut registry = Registry::<&'static str>::new();
        let window = window::Id::new(1);

        registry.register(
            Action::new(SELECT_ALL, "Select All")
                .on_invoke(|_| Effect::Batch(vec!["first", "second"])),
        );

        assert_eq!(
            registry.invoke(Invocation::new(
                SELECT_ALL,
                Source::Programmatic,
                Context::window(window)
            )),
            Some(Effect::Batch(vec!["first", "second"]))
        );
    }

    #[test]
    fn execution_state_overlays_active_state() {
        let mut registry = Registry::<()>::new();
        let window = window::Id::new(1);
        let context = Context::path(window, ui::Path::from(TEXT_BOX));

        registry.register(Action::new(SELECT_ALL, "Select All"));
        registry.begin_execution(SELECT_ALL, context.clone());

        assert!(registry.state(SELECT_ALL, context.clone()).active);

        registry.end_execution(SELECT_ALL, &context);

        assert!(!registry.state(SELECT_ALL, context).active);
    }
}
