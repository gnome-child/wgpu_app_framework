use super::{Affinity, LineId};

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash)]
pub enum MarkGravity {
    Upstream,
    #[default]
    Downstream,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Mark {
    pub line_id: LineId,
    pub byte_offset: usize,
    pub affinity: Affinity,
    pub gravity: MarkGravity,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct MarkRange {
    pub start: Mark,
    pub end: Mark,
}
