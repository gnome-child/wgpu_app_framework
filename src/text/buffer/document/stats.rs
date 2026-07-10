use std::cell::Cell;

#[cfg(test)]
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub(in crate::text) struct TextDocumentStatsSnapshot {
    pub(in crate::text) full_materializations: usize,
    pub(in crate::text) total_document_scans: usize,
    pub(in crate::text) piece_tree_updates: usize,
}

#[derive(Debug, Default)]
pub(super) struct TextDocumentStats {
    pub(super) full_materializations: Cell<usize>,
    pub(super) total_document_scans: Cell<usize>,
    pub(super) piece_tree_updates: Cell<usize>,
}

impl Clone for TextDocumentStats {
    fn clone(&self) -> Self {
        Self {
            full_materializations: Cell::new(self.full_materializations.get()),
            total_document_scans: Cell::new(self.total_document_scans.get()),
            piece_tree_updates: Cell::new(self.piece_tree_updates.get()),
        }
    }
}

impl TextDocumentStats {
    #[cfg(test)]
    pub(super) fn snapshot(&self) -> TextDocumentStatsSnapshot {
        TextDocumentStatsSnapshot {
            full_materializations: self.full_materializations.get(),
            total_document_scans: self.total_document_scans.get(),
            piece_tree_updates: self.piece_tree_updates.get(),
        }
    }

    #[cfg(test)]
    pub(super) fn reset(&self) {
        self.full_materializations.set(0);
        self.total_document_scans.set(0);
        self.piece_tree_updates.set(0);
        // Keep mapped indexing progress; reset only transient materialization/edit counters.
    }
}
