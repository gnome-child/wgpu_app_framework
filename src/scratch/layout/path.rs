#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Path(Vec<usize>);

impl Path {
    pub(super) fn root() -> Self {
        Self(Vec::new())
    }

    pub(super) fn child(&self, index: usize) -> Self {
        let mut path = self.0.clone();
        path.push(index);
        Self(path)
    }

    pub fn indexes(&self) -> &[usize] {
        &self.0
    }

    pub(in crate::scratch) fn is_descendant_of(&self, ancestor: &Self) -> bool {
        self.0.len() > ancestor.0.len() && self.0.starts_with(&ancestor.0)
    }
}
