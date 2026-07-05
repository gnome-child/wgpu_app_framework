use std::fmt;

use crate::action;

use super::{Command, Key, Response, call::Invocation, state::State};

pub trait Target<C: Command> {
    fn state(&self, _context: &super::call::Context) -> State {
        State::available()
    }

    fn invoke(&mut self, args: C::Args, invocation: Invocation<C>) -> Response<C::Output>;
}

/// A runtime target category advertised by responder implementations.
///
/// Categories are capability labels for scope/focus resolution. They do not invoke commands by
/// themselves; concrete command execution is still provided by [`Target<C>`].
pub trait Category: 'static {
    fn kind() -> Kind {
        Kind::of::<Self>()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Kind(action::Target);

impl Kind {
    pub(crate) const fn command(command: Key) -> Self {
        Self(action::Target::action(command.action()))
    }

    pub(crate) fn is_command(self, command: Key) -> bool {
        self == Self::command(command)
    }

    pub fn of<T: ?Sized + Category>() -> Self {
        Self(action::Target::of::<T>())
    }

    pub fn of_type<T: 'static>() -> Self {
        Self(action::Target::of_type::<T>())
    }

    pub(crate) const fn from_action(target: action::Target) -> Self {
        Self(target)
    }

    pub(crate) const fn action(self) -> action::Target {
        self.0
    }

    pub const fn name(self) -> &'static str {
        self.0.name()
    }
}

impl Default for Kind {
    fn default() -> Self {
        Self::command(Key::unknown())
    }
}

impl fmt::Display for Kind {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(&self.0, formatter)
    }
}
