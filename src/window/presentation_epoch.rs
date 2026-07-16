#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub(crate) struct PresentationEpoch(u64);

impl PresentationEpoch {
    pub(crate) fn initial() -> Self {
        Self(0)
    }

    pub(crate) fn next(self) -> Self {
        Self(self.0 + 1)
    }

    pub(crate) fn value(self) -> u64 {
        self.0
    }
}
