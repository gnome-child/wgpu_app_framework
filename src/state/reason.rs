use crate::command::Command;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Reason {
    Command(&'static str),
    Event(&'static str),
    Load,
    Save,
    Restore,
    Undo,
    Redo,
    Programmatic(&'static str),
}

impl Reason {
    pub fn command<C: Command>() -> Self {
        Self::Command(C::NAME)
    }

    pub fn event(label: &'static str) -> Self {
        Self::Event(label)
    }

    pub fn programmatic(label: &'static str) -> Self {
        Self::Programmatic(label)
    }
}
