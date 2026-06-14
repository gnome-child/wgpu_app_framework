#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Id(&'static str);

pub const SELECT_ALL: Id = Id::new("select_all");
pub const COPY: Id = Id::new("copy");
pub const CUT: Id = Id::new("cut");
pub const PASTE: Id = Id::new("paste");

impl Id {
    pub const fn new(value: &'static str) -> Self {
        Self(value)
    }

    pub const fn as_str(self) -> &'static str {
        self.0
    }
}
