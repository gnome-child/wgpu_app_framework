use std::collections::HashMap;

use crate::{ui, window};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Action {
    id: Id,
    label: String,
}

impl Action {
    pub fn new(id: Id, label: impl Into<String>) -> Self {
        Self {
            id,
            label: label.into(),
        }
    }

    pub fn id(&self) -> Id {
        self.id
    }

    pub fn label(&self) -> &str {
        &self.label
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
    pub enabled: bool,
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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Context {
    pub window: window::Id,
    pub target: Option<ui::Id>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Source {
    Pointer,
    Programmatic,
}

#[derive(Debug, Default)]
pub struct Registry {
    actions: HashMap<Id, Action>,
    states: HashMap<(Id, Context), State>,
}

impl Registry {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn register(&mut self, action: Action) {
        self.actions.insert(action.id(), action);
    }

    pub fn action(&self, id: Id) -> Option<&Action> {
        self.actions.get(&id)
    }

    pub fn set_state(&mut self, id: Id, context: Context, state: State) {
        self.states.insert((id, context), state);
    }

    pub fn state(&self, id: Id, context: Context) -> State {
        if !self.actions.contains_key(&id) {
            return State::disabled();
        }

        if let Some(state) = self.states.get(&(id, context)) {
            return *state;
        }

        if context.target.is_some() {
            let fallback = Context {
                target: None,
                ..context
            };

            if let Some(state) = self.states.get(&(id, fallback)) {
                return *state;
            }
        }

        State::enabled()
    }

    pub fn can_invoke(&self, id: Id, context: Context) -> bool {
        self.state(id, context).enabled
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const SELECT_ALL: Id = Id::new("select_all");
    const TEXT_BOX: ui::Id = ui::Id::new("text_box");

    #[test]
    fn unregistered_action_is_disabled() {
        let registry = Registry::new();
        let context = Context {
            window: window::Id::new(1),
            target: Some(TEXT_BOX),
        };

        assert_eq!(registry.state(SELECT_ALL, context), State::disabled());
    }

    #[test]
    fn context_specific_state_wins_over_window_fallback() {
        let mut registry = Registry::new();
        let window = window::Id::new(1);

        registry.register(Action::new(SELECT_ALL, "Select All"));
        registry.set_state(
            SELECT_ALL,
            Context {
                window,
                target: None,
            },
            State::disabled(),
        );
        registry.set_state(
            SELECT_ALL,
            Context {
                window,
                target: Some(TEXT_BOX),
            },
            State::active(),
        );

        assert_eq!(
            registry.state(
                SELECT_ALL,
                Context {
                    window,
                    target: Some(TEXT_BOX),
                }
            ),
            State::active()
        );
    }

    #[test]
    fn disabled_action_cannot_invoke() {
        let mut registry = Registry::new();
        let context = Context {
            window: window::Id::new(1),
            target: Some(TEXT_BOX),
        };

        registry.register(Action::new(SELECT_ALL, "Select All"));
        registry.set_state(SELECT_ALL, context, State::disabled());

        assert!(!registry.can_invoke(SELECT_ALL, context));
    }
}
