use std::any::{TypeId, type_name};
use std::fmt;

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
pub struct Kind(Repr);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum Repr {
    Command(Key),
    Trait { id: TypeId, name: &'static str },
    Type { id: TypeId, name: &'static str },
}

impl Kind {
    pub(crate) const fn command(command: Key) -> Self {
        Self(Repr::Command(command))
    }

    pub fn of<T: ?Sized + Category>() -> Self {
        Self(Repr::Trait {
            id: TypeId::of::<T>(),
            name: type_name::<T>(),
        })
    }

    pub fn of_type<T: 'static>() -> Self {
        Self(Repr::Type {
            id: TypeId::of::<T>(),
            name: type_name::<T>(),
        })
    }

    pub const fn name(self) -> &'static str {
        match self.0 {
            Repr::Command(command) => command.as_str(),
            Repr::Trait { name, .. } | Repr::Type { name, .. } => name,
        }
    }
}

impl Default for Kind {
    fn default() -> Self {
        Self::command(Key::unknown())
    }
}

impl fmt::Display for Kind {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.name())
    }
}
