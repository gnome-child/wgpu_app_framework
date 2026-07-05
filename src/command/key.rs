use std::fmt;

use crate::action;

use super::Command;

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub struct Key(action::Key);

struct UnknownCommand;

impl Key {
    pub(crate) const fn of<C: Command>() -> Self {
        Self(action::Key::typed::<C>(C::NAME))
    }

    pub(crate) fn unknown() -> Self {
        Self(action::Key::typed::<UnknownCommand>(std::any::type_name::<
            UnknownCommand,
        >()))
    }

    pub(crate) const fn from_action(key: action::Key) -> Self {
        Self(key)
    }

    pub(crate) const fn action(self) -> action::Key {
        self.0
    }

    pub const fn as_str(self) -> &'static str {
        self.0.as_str()
    }
}

impl fmt::Debug for Key {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Debug::fmt(&self.0, formatter)
    }
}

impl fmt::Display for Key {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(&self.0, formatter)
    }
}
