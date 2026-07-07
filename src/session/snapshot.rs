use super::WindowSnapshot;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Snapshot {
    windows: Vec<WindowSnapshot>,
}

impl Snapshot {
    pub(super) fn new(windows: Vec<WindowSnapshot>) -> Self {
        Self { windows }
    }

    pub fn from_windows(windows: impl IntoIterator<Item = WindowSnapshot>) -> Self {
        Self::new(windows.into_iter().collect())
    }

    pub fn windows(&self) -> &[WindowSnapshot] {
        &self.windows
    }

    pub(super) fn into_windows(self) -> Vec<WindowSnapshot> {
        self.windows
    }
}
