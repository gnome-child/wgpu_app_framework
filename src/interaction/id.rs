#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Id(&'static str);

impl Id {
    pub const fn new(value: &'static str) -> Self {
        Self(value)
    }

    pub fn as_str(self) -> &'static str {
        self.0
    }
}

impl From<&'static str> for Id {
    fn from(value: &'static str) -> Self {
        Self::new(value)
    }
}
