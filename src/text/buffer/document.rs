use std::{fs, io, path::Path, rc::Rc, sync::Arc};

use super::super::unicode::{
    ceil_grapheme_boundary, floor_grapheme_boundary, next_grapheme_boundary, next_word_boundary,
    previous_grapheme_boundary, previous_word_boundary,
};
use super::{Cursor, LineLayoutIdentity, Mark, MarkGravity, MarkRange, Position, Range, Selection};

mod line_index;
mod span_tree;
mod stats;

use line_index::{LineIndex, LineMeta};
use span_tree::SpanTree;
#[cfg(test)]
pub(in crate::text) use span_tree::TARGET_LEAF_BYTES as TEXT_DOCUMENT_TARGET_LEAF_BYTES;
use stats::TextDocumentStats;
#[cfg(test)]
pub(in crate::text) use stats::TextDocumentStatsSnapshot;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum DocumentLineEnding {
    Lf,
    CrLf,
}

impl DocumentLineEnding {
    fn detect(text: &str) -> Self {
        let bytes = text.as_bytes();
        let mut lf = 0usize;
        let mut crlf = 0usize;
        let mut first = None;
        for (index, byte) in bytes.iter().copied().enumerate() {
            if byte != b'\n' {
                continue;
            }
            let ending = if index > 0 && bytes[index - 1] == b'\r' {
                crlf += 1;
                Self::CrLf
            } else {
                lf += 1;
                Self::Lf
            };
            first.get_or_insert(ending);
        }
        match crlf.cmp(&lf) {
            std::cmp::Ordering::Greater => Self::CrLf,
            std::cmp::Ordering::Less => Self::Lf,
            std::cmp::Ordering::Equal => first.unwrap_or(Self::Lf),
        }
    }

    fn as_str(self) -> &'static str {
        match self {
            Self::Lf => "\n",
            Self::CrLf => "\r\n",
        }
    }
}

#[derive(Debug, Clone)]
pub(in crate::text) struct TextDocument {
    tree: SpanTree,
    lines: LineIndex,
    line_ending: DocumentLineEnding,
    pub(in crate::text) revision: u64,
    stats: TextDocumentStats,
}

impl TextDocument {
    pub(in crate::text) fn from_text(text: &str) -> Self {
        Self::from_owned(Arc::from(text))
    }

    pub(in crate::text) fn open_file(path: impl AsRef<Path>) -> io::Result<Self> {
        let bytes = fs::read(path)?;
        let text = String::from_utf8(bytes).map_err(|error| {
            io::Error::new(
                io::ErrorKind::InvalidData,
                format!("text file is not valid UTF-8: {error}"),
            )
        })?;
        Ok(Self::from_owned(Arc::from(text)))
    }

    fn from_owned(text: Arc<str>) -> Self {
        let line_ending = DocumentLineEnding::detect(&text);
        let tree = SpanTree::from_original(text);
        let lines = LineIndex::new(tree.line_count(), 0);
        Self {
            tree,
            lines,
            line_ending,
            revision: 0,
            stats: TextDocumentStats::default(),
        }
    }

    pub(in crate::text) fn line_count(&self) -> usize {
        self.tree.line_count()
    }

    pub(in crate::text) fn text_len(&self) -> usize {
        self.tree.len()
    }

    pub(in crate::text) fn line_starts(&self) -> Rc<Vec<usize>> {
        self.stats
            .total_document_scans
            .set(self.stats.total_document_scans.get() + 1);
        Rc::new(self.tree.line_starts())
    }

    pub(in crate::text) fn line_start(&self, line: usize) -> usize {
        self.tree.line_start(line)
    }

    pub(in crate::text) fn line_text_len(&self, line: usize) -> usize {
        let bounds = self.tree.line_bounds(line);
        bounds.end - bounds.start
    }

    pub(in crate::text) fn line_ending(&self) -> &'static str {
        self.line_ending.as_str()
    }

    pub(in crate::text) fn text(&self) -> String {
        self.stats
            .full_materializations
            .set(self.stats.full_materializations.get() + 1);
        self.tree.text()
    }

    pub(in crate::text) fn text_for_range(&self, range: std::ops::Range<usize>) -> String {
        self.tree.text_for_range(range)
    }

    pub(in crate::text) fn text_for_line_range(&self, start: usize, end: usize) -> String {
        let end = end.min(self.line_count());
        let start = start.min(end);
        if start == end {
            return String::new();
        }
        let first = self.tree.line_bounds(start);
        let last = self.tree.line_bounds(end - 1);
        self.tree.text_for_range(first.start..last.end)
    }

    pub(in crate::text) fn line_layout_identity(&self, line: usize) -> Option<LineLayoutIdentity> {
        self.lines.get(line).map(LineMeta::identity)
    }

    pub(in crate::text) fn replace_range(
        &mut self,
        range: Range,
        inserted: &str,
    ) -> (std::ops::Range<usize>, String, String, usize, usize, usize) {
        let range = self.snap_range(range);
        let deleted = self.text_for_range(range.clone());
        if range.is_empty() && inserted.is_empty() {
            return (range, deleted, String::new(), 0, 0, 0);
        }

        let (start_line, start_local) = self.line_and_local_for_index(range.start);
        let (end_line, end_local) = self.line_and_local_for_index(range.end);
        let old_line_count = end_line.saturating_sub(start_line) + 1;
        let old_start = self.lines.get(start_line);
        let old_end = self.lines.get(end_line);
        let next_revision = self.revision.saturating_add(1);
        let inserted_line_count = inserted.bytes().filter(|byte| *byte == b'\n').count() + 1;
        let mut replacement_lines = (0..inserted_line_count)
            .map(|_| LineMeta::new(next_revision))
            .collect::<Vec<_>>();

        if start_local == 0 && end_local == 0 {
            if let (Some(old_end), Some(suffix)) = (old_end, replacement_lines.last_mut()) {
                *suffix = old_end.with_revision(next_revision);
            }
        } else if let (Some(old_start), Some(first)) = (old_start, replacement_lines.first_mut()) {
            *first = old_start.with_revision(next_revision);
        }

        let inserted_tree = if inserted.is_empty() {
            SpanTree::default()
        } else {
            SpanTree::from_addition(Arc::from(inserted))
        };
        self.tree = self.tree.replace(range.clone(), inserted_tree);
        self.lines = self
            .lines
            .replace(start_line, old_line_count, replacement_lines);
        self.revision = next_revision;
        self.stats
            .piece_tree_updates
            .set(self.stats.piece_tree_updates.get() + 1);
        debug_assert_eq!(self.tree.line_count(), self.lines.len());
        #[cfg(debug_assertions)]
        {
            self.tree.assert_invariants();
            self.lines.assert_invariants();
        }

        (
            range,
            deleted,
            inserted.to_owned(),
            start_line,
            old_line_count,
            inserted_line_count,
        )
    }

    pub(in crate::text) fn cursor_for_text_index(&self, index: usize) -> Cursor {
        let index = index.min(self.text_len());
        let (line, local) = self.line_and_local_for_index(index);
        Cursor::new(line, self.floor_grapheme_in_line(line, local))
    }

    pub(in crate::text) fn cursor_for_position(&self, position: Position) -> Cursor {
        let cursor = self.cursor_for_text_index(position.index);
        Cursor::new_with_affinity(cursor.line, cursor.index, position.affinity)
    }

    pub(in crate::text) fn text_index_for_cursor(&self, cursor: Cursor) -> usize {
        let line = cursor.line.min(self.line_count().saturating_sub(1));
        self.line_start(line) + self.floor_grapheme_in_line(line, cursor.index)
    }

    pub(in crate::text) fn mark_for_position(&self, position: Position) -> Option<Mark> {
        let cursor = self.cursor_for_position(position);
        self.mark_for_cursor(cursor)
    }

    pub(in crate::text) fn mark_range_for_selection(
        &self,
        selection: Selection,
    ) -> Option<MarkRange> {
        Some(MarkRange {
            start: self.mark_for_position(selection.anchor)?,
            end: self.mark_for_position(selection.focus)?,
        })
    }

    pub(in crate::text) fn position_for_mark(&self, anchor: Mark) -> Option<Position> {
        let (line, _) = self.lines.find(anchor.line_id)?;
        let local = self.floor_grapheme_in_line(line, anchor.byte_offset);
        Some(Position::with_affinity(
            self.line_start(line) + local,
            anchor.affinity,
        ))
    }

    pub(in crate::text) fn mark_for_cursor(&self, cursor: Cursor) -> Option<Mark> {
        let line = cursor.line.min(self.line_count().saturating_sub(1));
        let meta = self.lines.get(line)?;
        Some(Mark {
            line_id: meta.id,
            byte_offset: self.floor_grapheme_in_line(line, cursor.index),
            affinity: cursor.affinity,
            gravity: MarkGravity::Downstream,
        })
    }

    pub(in crate::text) fn cursor_for_mark(&self, anchor: Mark) -> Option<Cursor> {
        let (line, _) = self.lines.find(anchor.line_id)?;
        Some(Cursor::new_with_affinity(
            line,
            self.floor_grapheme_in_line(line, anchor.byte_offset),
            anchor.affinity,
        ))
    }

    pub(in crate::text) fn selection_for_mark_range(&self, range: MarkRange) -> Option<Selection> {
        Some(Selection::new(
            self.position_for_mark(range.start)?,
            self.position_for_mark(range.end)?,
        ))
    }

    pub(in crate::text) fn ordered_cursor_range_for_mark_range(
        &self,
        range: MarkRange,
    ) -> Option<(Cursor, Cursor)> {
        let start_position = self.position_for_mark(range.start)?;
        let end_position = self.position_for_mark(range.end)?;
        let start = self.cursor_for_mark(range.start)?;
        let end = self.cursor_for_mark(range.end)?;
        if start_position.index <= end_position.index {
            Some((start, end))
        } else {
            Some((end, start))
        }
    }

    pub(in crate::text) fn line_and_local_for_index(&self, index: usize) -> (usize, usize) {
        self.tree.line_and_local_for_index(index)
    }

    pub(in crate::text) fn snap_range(&self, range: Range) -> std::ops::Range<usize> {
        let range = range.as_range();
        if range.is_empty() {
            let index = self.floor_grapheme_boundary(range.start);
            return index..index;
        }
        self.floor_grapheme_boundary(range.start)..self.ceil_grapheme_boundary(range.end)
    }

    pub(in crate::text) fn floor_grapheme_boundary(&self, index: usize) -> usize {
        let (line, local) = self.line_and_local_for_index(index);
        self.line_start(line) + self.floor_grapheme_in_line(line, local)
    }

    pub(in crate::text) fn ceil_grapheme_boundary(&self, index: usize) -> usize {
        let (line, local) = self.line_and_local_for_index(index);
        self.line_start(line) + self.ceil_grapheme_in_line(line, local)
    }

    pub(in crate::text) fn previous_grapheme_boundary_index(&self, index: usize) -> usize {
        let index = index.min(self.text_len());
        if index == 0 {
            return 0;
        }
        let (line, local) = self.line_and_local_for_index(index);
        if local > 0 {
            let text = self.line_text(line);
            self.line_start(line) + previous_grapheme_boundary(&text, local)
        } else if line == 0 {
            0
        } else {
            self.line_start(line - 1) + self.line_text_len(line - 1)
        }
    }

    pub(in crate::text) fn next_grapheme_boundary_index(&self, index: usize) -> usize {
        let index = index.min(self.text_len());
        let (line, local) = self.line_and_local_for_index(index);
        let text = self.line_text(line);
        if local < text.len() {
            self.line_start(line) + next_grapheme_boundary(&text, local)
        } else if line + 1 < self.line_count() {
            self.line_start(line + 1)
        } else {
            self.text_len()
        }
    }

    pub(in crate::text) fn previous_word_boundary_index(&self, index: usize) -> usize {
        let index = index.min(self.text_len());
        if index == 0 {
            return 0;
        }
        let (line, local) = self.line_and_local_for_index(index);
        if local > 0 {
            let text = self.line_text(line);
            self.line_start(line) + previous_word_boundary(&text, local)
        } else if line == 0 {
            0
        } else {
            self.line_start(line - 1) + self.line_text_len(line - 1)
        }
    }

    pub(in crate::text) fn next_word_boundary_index(&self, index: usize) -> usize {
        let index = index.min(self.text_len());
        let (line, local) = self.line_and_local_for_index(index);
        let text = self.line_text(line);
        if local < text.len() {
            self.line_start(line) + next_word_boundary(&text, local)
        } else if line + 1 < self.line_count() {
            self.line_start(line + 1)
        } else {
            self.text_len()
        }
    }

    fn line_text(&self, line: usize) -> String {
        let bounds = self.tree.line_bounds(line);
        self.tree.text_for_range(bounds.start..bounds.end)
    }

    fn floor_grapheme_in_line(&self, line: usize, local: usize) -> usize {
        let text = self.line_text(line);
        floor_grapheme_boundary(&text, local)
    }

    fn ceil_grapheme_in_line(&self, line: usize, local: usize) -> usize {
        let text = self.line_text(line);
        ceil_grapheme_boundary(&text, local)
    }

    pub(in crate::text) fn write_to(&self, writer: &mut dyn io::Write) -> io::Result<()> {
        self.tree.write_to(writer)
    }

    #[cfg(test)]
    pub(super) fn original_len(&self) -> usize {
        self.tree.source_lengths().0
    }

    #[cfg(test)]
    pub(super) fn piece_source_lengths(&self) -> (usize, usize, usize) {
        let (original, add) = self.tree.source_lengths();
        (original, 0, add)
    }

    #[cfg(test)]
    pub(super) fn shares_text_root_with(&self, other: &Self) -> bool {
        self.tree.shares_root_with(&other.tree)
    }

    #[cfg(test)]
    pub(super) fn shared_text_leaf_count(&self, other: &Self) -> usize {
        self.tree.shared_leaf_count(&other.tree)
    }

    #[cfg(test)]
    pub(super) fn shares_line_index_root_with(&self, other: &Self) -> bool {
        self.lines.shares_root_with(&other.lines)
    }

    #[cfg(test)]
    pub(super) fn shared_line_index_leaf_count(&self, other: &Self) -> usize {
        self.lines.shared_leaf_count(&other.lines)
    }

    #[cfg(test)]
    pub(super) fn reset_stats(&self) {
        self.stats.reset();
    }

    #[cfg(test)]
    pub(super) fn stats(&self) -> TextDocumentStatsSnapshot {
        self.stats.snapshot()
    }
}

#[cfg(test)]
mod tests {
    use super::DocumentLineEnding;

    #[test]
    fn dominant_line_ending_uses_counts_then_first_seen_tie_break() {
        assert_eq!(DocumentLineEnding::detect("plain"), DocumentLineEnding::Lf);
        assert_eq!(
            DocumentLineEnding::detect("a\r\nb\r\nc\n"),
            DocumentLineEnding::CrLf
        );
        assert_eq!(
            DocumentLineEnding::detect("a\nb\r\n"),
            DocumentLineEnding::Lf
        );
        assert_eq!(
            DocumentLineEnding::detect("a\r\nb\n"),
            DocumentLineEnding::CrLf
        );
    }
}
