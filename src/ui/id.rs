use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Id(Repr);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum Repr {
    Named(&'static str),
    Structural { kind: &'static str, index: usize },
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Path {
    ids: Vec<Id>,
}

impl Id {
    pub const fn new(value: &'static str) -> Self {
        Self(Repr::Named(value))
    }

    pub(crate) const fn structural(kind: &'static str, index: usize) -> Self {
        Self(Repr::Structural { kind, index })
    }

    pub const fn as_str(self) -> &'static str {
        match self.0 {
            Repr::Named(value) => value,
            Repr::Structural { kind, .. } => kind,
        }
    }

    pub fn is_structural(self) -> bool {
        matches!(self.0, Repr::Structural { .. })
    }
}

impl Path {
    pub fn new(ids: impl Into<Vec<Id>>) -> Self {
        Self { ids: ids.into() }
    }

    pub fn root(id: Id) -> Self {
        Self { ids: vec![id] }
    }

    pub fn child(&self, id: Id) -> Self {
        let mut ids = self.ids.clone();
        ids.push(id);
        Self { ids }
    }

    pub fn push(&mut self, id: Id) {
        self.ids.push(id);
    }

    pub fn ids(&self) -> &[Id] {
        &self.ids
    }

    pub fn leaf(&self) -> Option<Id> {
        self.ids.last().copied()
    }

    pub fn is_descendant_of(&self, ancestor: &Self) -> bool {
        self.ids.starts_with(ancestor.ids())
    }
}

impl From<Id> for Path {
    fn from(value: Id) -> Self {
        Self::root(value)
    }
}

impl fmt::Display for Id {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.0 {
            Repr::Named(value) => formatter.write_str(value),
            Repr::Structural { kind, index } => write!(formatter, "{kind}_{index}"),
        }
    }
}
