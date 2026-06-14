#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Id(&'static str);

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Path {
    ids: Vec<Id>,
}

impl Id {
    pub const fn new(value: &'static str) -> Self {
        Self(value)
    }

    pub const fn as_str(self) -> &'static str {
        self.0
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
}

impl From<Id> for Path {
    fn from(value: Id) -> Self {
        Self::root(value)
    }
}
