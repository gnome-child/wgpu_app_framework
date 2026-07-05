use super::{
    super::buffer::{LineId, Mark, Range},
    Marker,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct Impact {
    pub(crate) range: Range,
    pub(crate) affected_start_line: usize,
    pub(crate) affected_start_line_id: Option<LineId>,
    pub(crate) removed_line_count: usize,
    pub(crate) inserted_line_count: usize,
    pub(crate) deleted_bytes: usize,
    pub(crate) inserted_bytes: usize,
    pub(crate) caret_mark: Mark,
}

#[derive(Debug, Clone)]
pub(crate) struct Change {
    pub(crate) before: Marker,
    pub(crate) after: Marker,
    pub(crate) transaction: Transaction,
}

#[derive(Debug, Clone, Default)]
pub(crate) struct Transaction {
    pub(crate) deltas: Vec<Delta>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct Delta {
    pub(crate) kind: Kind,
    pub(crate) range: Range,
    pub(crate) deleted: String,
    pub(crate) inserted: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Kind {
    Insert,
    Delete,
    Replace,
    Move,
    ImeCommit,
}

impl Impact {
    pub(crate) fn affected_line_count(&self) -> usize {
        self.removed_line_count.max(self.inserted_line_count).max(1)
    }
}

impl Transaction {
    pub(crate) fn replace(range: Range, deleted: String, inserted: String, kind: Kind) -> Self {
        let mut transaction = Self::default();
        transaction.push_replace(range, deleted, inserted, kind);
        transaction
    }

    fn push_replace(&mut self, range: Range, deleted: String, inserted: String, kind: Kind) {
        if range.start == range.end && deleted.is_empty() && inserted.is_empty() {
            return;
        }
        self.deltas.push(Delta {
            kind,
            range,
            deleted,
            inserted,
        });
    }

    pub(crate) fn is_empty(&self) -> bool {
        self.deltas.is_empty()
    }

    pub(crate) fn inverse(&self) -> Self {
        let mut inverse = Self::default();
        for delta in self.deltas.iter().rev() {
            inverse.push_replace(
                Range::new(delta.range.start, delta.range.start + delta.inserted.len()),
                delta.inserted.clone(),
                delta.deleted.clone(),
                delta.kind,
            );
        }
        inverse
    }

    pub(crate) fn try_coalesce_typing(&mut self, next: &Transaction) -> bool {
        if self.deltas.len() != 1 || next.deltas.len() != 1 {
            return false;
        }
        let current = &mut self.deltas[0];
        let next = &next.deltas[0];
        if current.kind != Kind::Insert || next.kind != Kind::Insert {
            return false;
        }
        if !current.deleted.is_empty() || !next.deleted.is_empty() {
            return false;
        }
        if current.range.start + current.inserted.len() != next.range.start {
            return false;
        }
        current.inserted.push_str(&next.inserted);
        true
    }
}

impl Delta {
    #[allow(dead_code)]
    pub(crate) fn inserted_end(&self) -> usize {
        self.range.start + self.inserted.len()
    }
}
