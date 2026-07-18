#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct Path(Vec<Segment>);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Segment {
    Index(usize),
    VirtualRow(crate::list::Key),
}

impl Path {
    pub(super) fn root() -> Self {
        Self(Vec::new())
    }

    pub(super) fn child(&self, index: usize) -> Self {
        let mut path = self.0.clone();
        path.push(Segment::Index(index));
        Self(path)
    }

    pub(super) fn virtual_row(&self, key: crate::list::Key) -> Self {
        let mut path = self.0.clone();
        path.push(Segment::VirtualRow(key));
        Self(path)
    }

    pub(super) fn len(&self) -> usize {
        self.0.len()
    }

    pub(super) fn is_descendant_of(&self, ancestor: &Self) -> bool {
        self.0.len() > ancestor.0.len() && self.0.starts_with(&ancestor.0)
    }

    pub(super) fn rebased(&self, from: &Self, to: &Self) -> Option<Self> {
        let suffix = self.0.strip_prefix(from.0.as_slice())?;
        let mut path = to.0.clone();
        path.extend_from_slice(suffix);
        Some(Self(path))
    }
}
