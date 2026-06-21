use super::{LineId, TextAffinity};

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash)]
pub enum Gravity {
    Upstream,
    #[default]
    Downstream,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Mark {
    pub line_id: LineId,
    pub byte_offset: usize,
    pub affinity: TextAffinity,
    pub gravity: Gravity,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Range {
    pub start: Mark,
    pub end: Mark,
}
