use std::any::{TypeId, type_name};
use std::fmt;

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub struct Key {
    type_id: TypeId,
    name: &'static str,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Target(TargetRepr);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum TargetRepr {
    Action(Key),
    Trait { id: TypeId, name: &'static str },
    Type { id: TypeId, name: &'static str },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Route {
    key: Key,
    target: Target,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Binding {
    key: Key,
    state: Option<State>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct State {
    available: bool,
    active: bool,
    running: bool,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum Subject {
    #[default]
    Origin,
    Current,
    Captured,
    Window,
}

impl Key {
    pub(crate) const fn typed<T: 'static>(name: &'static str) -> Self {
        Self {
            type_id: TypeId::of::<T>(),
            name,
        }
    }

    pub const fn as_str(self) -> &'static str {
        self.name
    }
}

impl fmt::Debug for Key {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.debug_tuple("Key").field(&self.name).finish()
    }
}

impl fmt::Display for Key {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.name)
    }
}

impl Target {
    pub(crate) const fn action(key: Key) -> Self {
        Self(TargetRepr::Action(key))
    }

    pub(crate) fn of<T: ?Sized + 'static>() -> Self {
        Self(TargetRepr::Trait {
            id: TypeId::of::<T>(),
            name: type_name::<T>(),
        })
    }

    pub(crate) fn of_type<T: 'static>() -> Self {
        Self(TargetRepr::Type {
            id: TypeId::of::<T>(),
            name: type_name::<T>(),
        })
    }

    pub const fn name(self) -> &'static str {
        match self.0 {
            TargetRepr::Action(key) => key.as_str(),
            TargetRepr::Trait { name, .. } | TargetRepr::Type { name, .. } => name,
        }
    }
}

impl Default for Target {
    fn default() -> Self {
        Self::action(Key::typed::<UnknownAction>(type_name::<UnknownAction>()))
    }
}

impl fmt::Display for Target {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.name())
    }
}

struct UnknownAction;

impl Route {
    pub(crate) const fn new(key: Key, target: Target) -> Self {
        Self { key, target }
    }

    pub const fn key(self) -> Key {
        self.key
    }

    pub const fn target(self) -> Target {
        self.target
    }
}

impl Binding {
    pub(crate) const fn new(key: Key) -> Self {
        Self { key, state: None }
    }

    pub const fn key(&self) -> Key {
        self.key
    }

    pub fn state(&self) -> Option<&State> {
        self.state.as_ref()
    }

    pub fn available(mut self, available: bool) -> Self {
        self.state = Some(self.current_state().with_available(available));
        self
    }

    pub fn active(mut self, active: bool) -> Self {
        self.state = Some(self.current_state().with_active(active));
        self
    }

    pub fn running(mut self, running: bool) -> Self {
        self.state = Some(self.current_state().with_running(running));
        self
    }

    fn current_state(&self) -> State {
        self.state.unwrap_or_default()
    }
}

impl State {
    pub const fn available() -> Self {
        Self {
            available: true,
            active: false,
            running: false,
        }
    }

    pub const fn available_if(available: bool) -> Self {
        Self::available().with_available(available)
    }

    pub const fn active() -> Self {
        Self::available().with_active(true)
    }

    pub const fn running() -> Self {
        Self::available().with_running(true)
    }

    pub const fn is_available(self) -> bool {
        self.available
    }

    pub const fn is_active(self) -> bool {
        self.active
    }

    pub const fn is_running(self) -> bool {
        self.running
    }

    pub const fn with_available(mut self, available: bool) -> Self {
        self.available = available;
        self
    }

    pub const fn with_active(mut self, active: bool) -> Self {
        self.active = active;
        self
    }

    pub const fn with_running(mut self, running: bool) -> Self {
        self.running = running;
        self
    }
}

impl Default for State {
    fn default() -> Self {
        Self::available()
    }
}
