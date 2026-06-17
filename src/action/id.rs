use super::PayloadKind;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Id(&'static str);

pub const SELECT_ALL: Id = Id::new("select_all");
pub const COPY: Id = Id::new("copy");
pub const CUT: Id = Id::new("cut");
pub const PASTE: Id = Id::new("paste");
pub const INSERT_TEXT: Id = Id::new("insert_text");
pub const UNDO: Id = Id::new("undo");
pub const REDO: Id = Id::new("redo");

impl Id {
    pub const fn new(value: &'static str) -> Self {
        Self(value)
    }

    pub const fn as_str(self) -> &'static str {
        self.0
    }

    pub fn payload_kind(self) -> PayloadKind {
        if self.0 == INSERT_TEXT.0 {
            PayloadKind::Text
        } else {
            PayloadKind::None
        }
    }
}
