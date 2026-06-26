use std::any::{TypeId, type_name};
use std::fmt;

use super::Command;

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub struct Key {
    type_id: TypeId,
    name: &'static str,
}

struct UnknownCommand;

impl Key {
    pub(crate) const fn of<C: Command>() -> Self {
        Self {
            type_id: TypeId::of::<C>(),
            name: C::NAME,
        }
    }

    pub(crate) fn unknown() -> Self {
        Self {
            type_id: TypeId::of::<UnknownCommand>(),
            name: type_name::<UnknownCommand>(),
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
