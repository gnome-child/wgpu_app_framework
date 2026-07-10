use std::{fs::File, io, path::Path, rc::Rc, sync::Arc};

use super::super::unicode::{
    next_grapheme_boundary, next_word_boundary, previous_grapheme_boundary, previous_word_boundary,
};
use super::{
    Cursor, LineId, LineLayoutIdentity, Mark, MarkGravity, MarkRange, Position, Range, Selection,
};

mod line;
mod source;
mod stats;

#[cfg(test)]
pub(in crate::text) use line::TEXT_DOCUMENT_BLOCK_TARGET_LINES;
use line::{TextLine, TextLineEnding, TextLineTree};
use source::{MappedTextSource, TextOriginal, TextPieceSource};
use stats::TextDocumentStats;
#[cfg(test)]
pub(in crate::text) use stats::TextDocumentStatsSnapshot;

const TEXT_MAPPED_INDEX_PAGE_BYTES: usize = 64 * 1024;

#[derive(Debug, Clone)]
pub(in crate::text) struct TextDocument {
    // Owns the original text bytes, and keeps mmap-backed source slices alive.
    #[allow(dead_code)]
    original: TextOriginal,
    pub(super) add_buffer: Arc<String>,
    pub(super) tree: TextLineTree,
    pub(in crate::text) revision: u64,
    stats: TextDocumentStats,
}

impl TextDocument {
    pub(in crate::text) fn from_text(text: &str) -> Self {
        let original: Arc<str> = Arc::from(text);
        Self::from_source_text(
            text,
            TextOriginal::Owned(original),
            TextPieceSource::OriginalOwned,
            0,
        )
    }

    pub(in crate::text) fn open_mapped(path: impl AsRef<Path>) -> io::Result<Self> {
        let path = path.as_ref();
        let file = File::open(path)?;
        let mmap = Arc::new(unsafe { memmap2::Mmap::map(&file)? });
        let text = std::str::from_utf8(&mmap[..]).map_err(|error| {
            io::Error::new(
                io::ErrorKind::InvalidData,
                format!("mapped text is not valid UTF-8: {error}"),
            )
        })?;
        let mapped = Arc::new(MappedTextSource {
            path: path.to_path_buf(),
            mmap: mmap.clone(),
        });
        let document = Self::from_source_text(
            text,
            TextOriginal::Mapped(mapped),
            TextPieceSource::OriginalMapped,
            0,
        );
        let pages = text.len().saturating_add(TEXT_MAPPED_INDEX_PAGE_BYTES - 1)
            / TEXT_MAPPED_INDEX_PAGE_BYTES;
        document.stats.mapped_index_pages_scanned.set(pages.max(1));
        Ok(document)
    }

    fn from_source_text(
        text: &str,
        original: TextOriginal,
        source: TextPieceSource,
        source_base: usize,
    ) -> Self {
        let lines = Self::lines_from_source(text, TextLineEnding::None, 0, source, source_base);
        Self {
            original,
            add_buffer: Arc::new(String::new()),
            tree: TextLineTree::from_lines(lines),
            revision: 0,
            stats: TextDocumentStats::default(),
        }
    }

    pub(in crate::text) fn line_count(&self) -> usize {
        self.tree.line_count()
    }

    pub(in crate::text) fn text_len(&self) -> usize {
        self.tree.text_len()
    }

    pub(in crate::text) fn line_starts(&self) -> Rc<Vec<usize>> {
        self.stats
            .total_document_scans
            .set(self.stats.total_document_scans.get() + 1);
        let mut starts = Vec::with_capacity(self.line_count());
        let mut offset = 0usize;
        for block in &self.tree.blocks {
            for line in block.lines.iter() {
                starts.push(offset);
                offset = offset.saturating_add(line.total_len());
            }
        }
        if starts.is_empty() {
            starts.push(0);
        }
        Rc::new(starts)
    }

    pub(in crate::text) fn line_start(&self, line: usize) -> usize {
        self.tree.line_start(line)
    }

    pub(in crate::text) fn line_text_len(&self, line: usize) -> usize {
        self.tree
            .line(line)
            .map(|line| line.text.len())
            .unwrap_or(0)
    }

    pub(in crate::text) fn text(&self) -> String {
        self.stats
            .full_materializations
            .set(self.stats.full_materializations.get() + 1);
        let mut text = String::with_capacity(self.text_len());
        for block in &self.tree.blocks {
            for line in block.lines.iter() {
                text.push_str(&line.text);
                text.push_str(line.ending.as_str());
            }
        }
        text
    }

    pub(in crate::text) fn text_for_range(&self, range: std::ops::Range<usize>) -> String {
        let range = range.start.min(self.text_len())..range.end.min(self.text_len());
        if range.is_empty() {
            return String::new();
        }
        let (start_line, start_local) = self.line_and_local_for_index(range.start);
        let (end_line, end_local) = self.line_and_local_for_index(range.end);
        if start_line == end_line {
            let Some(line) = self.tree.line(start_line) else {
                return String::new();
            };
            return line.text[start_local.min(line.text.len())..end_local.min(line.text.len())]
                .to_owned();
        }
        let mut text = String::new();
        if let Some(line) = self.tree.line(start_line) {
            text.push_str(&line.text[start_local.min(line.text.len())..]);
            text.push_str(line.ending.as_str());
        }
        for line_index in start_line + 1..end_line {
            if let Some(line) = self.tree.line(line_index) {
                text.push_str(&line.text);
                text.push_str(line.ending.as_str());
            }
        }
        if let Some(line) = self.tree.line(end_line) {
            text.push_str(&line.text[..end_local.min(line.text.len())]);
        }
        text
    }

    pub(in crate::text) fn text_for_line_range(&self, start: usize, end: usize) -> String {
        let mut text = String::new();
        let end = end.min(self.line_count());
        for line_index in start.min(end)..end {
            if let Some(line) = self.tree.line(line_index) {
                text.push_str(&line.text);
                if line_index + 1 < end {
                    text.push_str(line.ending.as_str());
                }
            }
        }
        text
    }

    pub(in crate::text) fn line_layout_identity(&self, line: usize) -> Option<LineLayoutIdentity> {
        self.tree.line(line).map(|line| LineLayoutIdentity {
            id: line.id,
            revision: line.revision,
        })
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
        let old_start_id = self.tree.line(start_line).map(|line| line.id);
        let old_end_id = self.tree.line(end_line).map(|line| line.id);
        let end_ending = self
            .tree
            .line(end_line)
            .map(|line| line.ending)
            .unwrap_or(TextLineEnding::None);
        let prefix = self
            .tree
            .line(start_line)
            .map(|line| line.text[..start_local.min(line.text.len())].to_owned())
            .unwrap_or_default();
        let suffix = self
            .tree
            .line(end_line)
            .map(|line| line.text[end_local.min(line.text.len())..].to_owned())
            .unwrap_or_default();

        let mut replacement_text =
            String::with_capacity(prefix.len() + inserted.len() + suffix.len());
        replacement_text.push_str(&prefix);
        replacement_text.push_str(inserted);
        replacement_text.push_str(&suffix);

        let next_revision = self.revision.saturating_add(1);
        let add_buffer = Arc::make_mut(&mut self.add_buffer);
        let add_start = add_buffer.len();
        add_buffer.push_str(&replacement_text);
        let removes_whole_lines =
            start_local == 0 && replacement_text.is_empty() && deleted.ends_with('\n');
        let mut replacement = if removes_whole_lines {
            Vec::new()
        } else {
            Self::lines_from_source(
                &replacement_text,
                end_ending,
                next_revision,
                TextPieceSource::Add,
                add_start,
            )
        };
        if start_local == 0 && end_local == 0 {
            let suffix_line = inserted.match_indices('\n').count();
            if let Some(id) = old_end_id
                && let Some(line) = replacement.get_mut(suffix_line)
            {
                line.id = id;
            }
        } else if let (Some(id), Some(first)) = (old_start_id, replacement.first_mut()) {
            first.id = id;
        }
        let new_line_count = replacement.len();

        self.tree
            .splice_lines(start_line, old_line_count, replacement);
        self.revision = next_revision;
        self.stats
            .piece_tree_updates
            .set(self.stats.piece_tree_updates.get() + 1);
        (
            range,
            deleted,
            inserted.to_owned(),
            start_line,
            old_line_count,
            new_line_count,
        )
    }

    pub(in crate::text) fn cursor_for_text_index(&self, index: usize) -> Cursor {
        let index = index.min(self.text_len());
        let (line, local) = self.line_and_local_for_index(index);
        let local = self
            .tree
            .line(line)
            .map(|line| line.floor_grapheme(local))
            .unwrap_or(0);
        Cursor::new(line, local)
    }

    pub(in crate::text) fn cursor_for_position(&self, position: Position) -> Cursor {
        let cursor = self.cursor_for_text_index(position.index);
        Cursor::new_with_affinity(cursor.line, cursor.index, position.affinity)
    }

    pub(in crate::text) fn text_index_for_cursor(&self, cursor: Cursor) -> usize {
        let line = cursor.line.min(self.line_count().saturating_sub(1));
        let local = self
            .tree
            .line(line)
            .map(|line| line.floor_grapheme(cursor.index))
            .unwrap_or(0);
        self.line_start(line) + local
    }

    pub(in crate::text) fn mark_for_position(&self, position: Position) -> Option<Mark> {
        let cursor = self.cursor_for_position(position);
        let line = self.tree.line(cursor.line)?;
        Some(Mark {
            line_id: line.id,
            byte_offset: cursor.index,
            affinity: cursor.affinity,
            gravity: MarkGravity::Downstream,
        })
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
        let (_, offset, line) = self.line_index_start_and_line_for_id(anchor.line_id)?;
        let local = line.floor_grapheme(anchor.byte_offset);
        Some(Position::with_affinity(offset + local, anchor.affinity))
    }

    pub(in crate::text) fn mark_for_cursor(&self, cursor: Cursor) -> Option<Mark> {
        let line_index = cursor.line.min(self.line_count().saturating_sub(1));
        let line = self.tree.line(line_index)?;
        Some(Mark {
            line_id: line.id,
            byte_offset: line.floor_grapheme(cursor.index),
            affinity: cursor.affinity,
            gravity: MarkGravity::Downstream,
        })
    }

    pub(in crate::text) fn cursor_for_mark(&self, anchor: Mark) -> Option<Cursor> {
        let (line_index, _, line) = self.line_index_start_and_line_for_id(anchor.line_id)?;
        Some(Cursor::new_with_affinity(
            line_index,
            line.floor_grapheme(anchor.byte_offset),
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

    fn line_index_start_and_line_for_id(
        &self,
        line_id: LineId,
    ) -> Option<(usize, usize, &TextLine)> {
        let mut line_index = 0usize;
        let mut offset = 0usize;
        for block in &self.tree.blocks {
            for line in block.lines.iter() {
                if line.id == line_id {
                    return Some((line_index, offset, line));
                }
                line_index = line_index.saturating_add(1);
                offset = offset.saturating_add(line.total_len());
            }
        }
        None
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
        let (line_index, local) = self.line_and_local_for_index(index);
        let Some(line) = self.tree.line(line_index) else {
            return 0;
        };
        self.line_start(line_index) + line.floor_grapheme(local)
    }

    pub(in crate::text) fn ceil_grapheme_boundary(&self, index: usize) -> usize {
        let (line_index, local) = self.line_and_local_for_index(index);
        let Some(line) = self.tree.line(line_index) else {
            return self.text_len();
        };
        self.line_start(line_index) + line.ceil_grapheme(local)
    }

    pub(in crate::text) fn previous_grapheme_boundary_index(&self, index: usize) -> usize {
        let index = index.min(self.text_len());
        if index == 0 {
            return 0;
        }
        let (line_index, local) = self.line_and_local_for_index(index);
        let Some(line) = self.tree.line(line_index) else {
            return 0;
        };
        if local > 0 {
            return self.line_start(line_index) + previous_grapheme_boundary(&line.text, local);
        }
        if line_index == 0 {
            0
        } else {
            self.line_start(line_index - 1) + self.line_text_len(line_index - 1)
        }
    }

    pub(in crate::text) fn next_grapheme_boundary_index(&self, index: usize) -> usize {
        let index = index.min(self.text_len());
        let (line_index, local) = self.line_and_local_for_index(index);
        let Some(line) = self.tree.line(line_index) else {
            return self.text_len();
        };
        if local < line.text.len() {
            return self.line_start(line_index) + next_grapheme_boundary(&line.text, local);
        }
        if line_index + 1 < self.line_count() {
            self.line_start(line_index + 1)
        } else {
            self.text_len()
        }
    }

    pub(in crate::text) fn previous_word_boundary_index(&self, index: usize) -> usize {
        let index = index.min(self.text_len());
        if index == 0 {
            return 0;
        }
        let (line_index, local) = self.line_and_local_for_index(index);
        let Some(line) = self.tree.line(line_index) else {
            return 0;
        };
        if local > 0 {
            return self.line_start(line_index) + previous_word_boundary(&line.text, local);
        }
        if line_index == 0 {
            0
        } else {
            self.line_start(line_index - 1) + self.line_text_len(line_index - 1)
        }
    }

    pub(in crate::text) fn next_word_boundary_index(&self, index: usize) -> usize {
        let index = index.min(self.text_len());
        let (line_index, local) = self.line_and_local_for_index(index);
        let Some(line) = self.tree.line(line_index) else {
            return self.text_len();
        };
        if local < line.text.len() {
            return self.line_start(line_index) + next_word_boundary(&line.text, local);
        }
        if line_index + 1 < self.line_count() {
            self.line_start(line_index + 1)
        } else {
            self.text_len()
        }
    }

    fn lines_from_source(
        text: &str,
        last_ending: TextLineEnding,
        revision: u64,
        source: TextPieceSource,
        source_base: usize,
    ) -> Vec<TextLine> {
        let mut lines = Vec::new();
        let mut start = 0;
        for (index, _) in text.match_indices('\n') {
            lines.push(
                TextLine::from_piece(
                    &text[start..index],
                    TextLineEnding::Lf,
                    source,
                    source_base + start,
                    revision,
                )
                .with_revision(revision),
            );
            start = index + 1;
        }
        lines.push(
            TextLine::from_piece(
                &text[start..],
                last_ending,
                source,
                source_base + start,
                revision,
            )
            .with_revision(revision),
        );
        if lines.is_empty() {
            lines.push(TextLine::new("", TextLineEnding::None).with_revision(revision));
        }
        lines
    }

    #[cfg(test)]
    pub(super) fn original_len(&self) -> usize {
        match &self.original {
            TextOriginal::Owned(text) => text.len(),
            TextOriginal::Mapped(mapped) => mapped.mmap.len(),
        }
    }

    #[cfg(test)]
    pub(super) fn piece_source_lengths(&self) -> (usize, usize, usize) {
        let mut owned = 0usize;
        let mut mapped = 0usize;
        let mut add = 0usize;
        for block in &self.tree.blocks {
            for line in block.lines.iter() {
                let (line_owned, line_mapped, line_add) = line.piece_source_lengths();
                owned = owned.saturating_add(line_owned);
                mapped = mapped.saturating_add(line_mapped);
                add = add.saturating_add(line_add);
            }
        }
        (owned, mapped, add)
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
