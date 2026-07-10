use std::fmt;
use std::io;
use std::path::Path;
use std::rc::Rc;
use std::sync::atomic::{AtomicU64, Ordering};

use super::unicode::normalize_for_mode;

mod document;
mod mark;
#[cfg(test)]
pub(super) use document::TEXT_DOCUMENT_BLOCK_TARGET_LINES;
use document::TextDocument;
#[cfg(test)]
pub(super) use document::TextDocumentStatsSnapshot;
pub use mark::{Mark, MarkGravity, MarkRange};

static NEXT_BUFFER_ID: AtomicU64 = AtomicU64::new(1);

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash)]
pub enum Affinity {
    Upstream,
    #[default]
    Downstream,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash)]
pub struct Cursor {
    pub line: usize,
    pub index: usize,
    pub affinity: Affinity,
}

impl Cursor {
    pub fn new(line: usize, index: usize) -> Self {
        Self::new_with_affinity(line, index, Affinity::Upstream)
    }

    pub fn new_with_affinity(line: usize, index: usize, affinity: Affinity) -> Self {
        Self {
            line,
            index,
            affinity,
        }
    }
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash)]
pub(crate) enum CursorSelection {
    #[default]
    None,
    Normal(Cursor),
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash)]
pub struct Position {
    pub index: usize,
    pub affinity: Affinity,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash)]
pub struct Range {
    pub start: usize,
    pub end: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Selection {
    pub anchor: Position,
    pub focus: Position,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct LineId(u64);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub(crate) struct LineLayoutIdentity {
    pub(crate) id: LineId,
    pub(crate) revision: u64,
}

pub struct Buffer {
    pub(super) inner: BufferInner,
}

#[derive(Debug)]
pub(super) struct BufferInner {
    pub(super) id: u64,
    pub(super) revision: u64,
    pub(super) document: TextDocument,
    pub(super) multiline: bool,
}

impl Buffer {
    pub fn new() -> Self {
        Self::from_text("")
    }
    pub fn new_multiline() -> Self {
        Self::from_multiline_text("")
    }
    pub fn from_text(text: impl Into<String>) -> Self {
        Self::from_text_with_mode(text, false)
    }
    pub fn from_multiline_text(text: impl Into<String>) -> Self {
        Self::from_text_with_mode(text, true)
    }
    pub(crate) fn from_text_with_mode(text: impl Into<String>, multiline: bool) -> Self {
        let text = normalize_for_mode(multiline, &text.into());
        Self::from_document_with_mode(TextDocument::from_text(&text), multiline)
    }

    pub fn from_mapped_file(path: impl AsRef<Path>) -> io::Result<Self> {
        Self::from_mapped_file_with_mode(path, true)
    }

    fn from_mapped_file_with_mode(path: impl AsRef<Path>, multiline: bool) -> io::Result<Self> {
        let document = TextDocument::open_mapped(path)?;
        Ok(Self::from_document_with_mode(document, multiline))
    }

    fn from_document_with_mode(document: TextDocument, multiline: bool) -> Self {
        let inner = BufferInner {
            id: NEXT_BUFFER_ID.fetch_add(1, Ordering::Relaxed),
            revision: document.revision,
            document,
            multiline,
        };
        Self { inner }
    }
    pub fn id(&self) -> u64 {
        self.inner.id
    }
    pub fn revision(&self) -> u64 {
        self.inner.revision
    }

    #[cfg(test)]
    pub(super) fn reset_line_index_stats(&mut self) {
        self.inner.document.reset_stats();
    }

    #[cfg(test)]
    pub(super) fn line_index_stats(&self) -> (usize, usize) {
        let stats = self.inner.document.stats();
        (stats.full_materializations, stats.piece_tree_updates)
    }

    #[cfg(test)]
    pub(super) fn reset_document_stats(&self) {
        self.inner.document.reset_stats();
    }

    #[cfg(test)]
    pub(super) fn document_stats(&self) -> TextDocumentStatsSnapshot {
        self.inner.document.stats()
    }

    #[cfg(test)]
    pub(super) fn document_piece_source_lengths(&self) -> (usize, usize, usize) {
        self.inner.document.piece_source_lengths()
    }

    #[cfg(test)]
    pub(super) fn original_len(&self) -> usize {
        self.inner.document.original_len()
    }
    pub fn text(&self) -> String {
        self.inner.document.text()
    }

    #[cfg(test)]
    pub(super) fn shares_add_buffer_with(&self, other: &Self) -> bool {
        std::sync::Arc::ptr_eq(
            &self.inner.document.add_buffer,
            &other.inner.document.add_buffer,
        )
    }

    #[cfg(test)]
    pub(super) fn shares_line_text_with(&self, other: &Self, line: usize) -> bool {
        let left = &self.inner;
        let right = &other.inner;
        let Some(left_text) = left.document.tree.line(line).map(|line| line.text.clone()) else {
            return false;
        };
        let Some(right_text) = right.document.tree.line(line).map(|line| line.text.clone()) else {
            return false;
        };

        std::sync::Arc::ptr_eq(&left_text, &right_text)
    }

    #[cfg(test)]
    pub(super) fn shares_line_block_with(&self, other: &Self, line: usize) -> bool {
        let left = &self.inner;
        let right = &other.inner;
        let (left_block, _) = left.document.tree.locate_line(line);
        let (right_block, _) = right.document.tree.locate_line(line);
        let Some(left_block) = left.document.tree.blocks.get(left_block) else {
            return false;
        };
        let Some(right_block) = right.document.tree.blocks.get(right_block) else {
            return false;
        };

        std::sync::Arc::ptr_eq(&left_block.lines, &right_block.lines)
    }

    pub fn to_plain_text(&self) -> String {
        self.text()
    }
    pub fn len(&self) -> usize {
        self.text_len()
    }
    fn text_len(&self) -> usize {
        self.inner.document.text_len()
    }
    pub fn is_empty(&self) -> bool {
        self.text_len() == 0
    }
    pub fn is_multiline(&self) -> bool {
        self.inner.multiline
    }
    pub(crate) fn promote_to_multiline(&mut self) {
        self.inner.multiline = true;
    }
    pub fn logical_line_count(&self) -> usize {
        self.inner.document.line_count()
    }
    pub(crate) fn line_start_offsets(&self) -> Rc<Vec<usize>> {
        self.inner.document.line_starts()
    }
    pub fn position_for_mark(&self, mark: Mark) -> Option<Position> {
        self.inner.document.position_for_mark(mark)
    }
    pub fn mark_for_position(&self, position: Position) -> Option<Mark> {
        self.inner.document.mark_for_position(position)
    }
    pub fn position_for_text_index(&self, index: usize) -> Position {
        let inner = &self.inner;
        let cursor = inner.document.cursor_for_text_index(index);
        Position::with_affinity(
            inner.document.text_index_for_cursor(cursor),
            cursor.affinity,
        )
    }
    pub fn text_index_for_position(&self, position: Position) -> usize {
        let inner = &self.inner;
        inner
            .document
            .text_index_for_cursor(inner.document.cursor_for_position(position))
    }
    pub(crate) fn cursor_for_text_index(&self, index: usize) -> Cursor {
        let inner = &self.inner;
        inner.document.cursor_for_text_index(index)
    }
    pub(crate) fn cursor_for_position(&self, position: Position) -> Cursor {
        let inner = &self.inner;
        inner.document.cursor_for_position(position)
    }
    pub(crate) fn text_index_for_cursor(&self, cursor: Cursor) -> usize {
        let inner = &self.inner;
        inner.document.text_index_for_cursor(cursor)
    }
    pub(crate) fn text_for_line_range(&self, start: usize, end: usize) -> String {
        self.inner.document.text_for_line_range(start, end)
    }
    pub(crate) fn line_layout_identity(&self, line: usize) -> Option<LineLayoutIdentity> {
        self.inner.document.line_layout_identity(line)
    }
}
pub(crate) fn local_cursor_range_for_source_line(
    range: (Cursor, Cursor),
    source_line: usize,
    source_text_len: usize,
) -> Option<(Cursor, Cursor)> {
    if range.1.line < source_line || range.0.line > source_line {
        return None;
    }
    let start = if range.0.line < source_line {
        0
    } else {
        range.0.index.min(source_text_len)
    };
    let end = if range.1.line > source_line {
        source_text_len
    } else {
        range.1.index.min(source_text_len)
    };
    (start < end).then(|| (Cursor::new(0, start), Cursor::new(0, end)))
}

impl PartialEq for Buffer {
    fn eq(&self, other: &Self) -> bool {
        self.is_multiline() == other.is_multiline() && self.to_plain_text() == other.to_plain_text()
    }
}

impl fmt::Debug for Buffer {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Buffer")
            .field("id", &self.id())
            .field("revision", &self.revision())
            .field("multiline", &self.is_multiline())
            .finish()
    }
}

impl Default for Buffer {
    fn default() -> Self {
        Self::new()
    }
}

impl From<String> for Buffer {
    fn from(value: String) -> Self {
        Self::from_text(value)
    }
}

impl From<&str> for Buffer {
    fn from(value: &str) -> Self {
        Self::from_text(value)
    }
}

pub(crate) fn normalize_for_buffer(buffer: &Buffer, text: &str) -> String {
    normalize_for_mode(buffer.is_multiline(), text)
}

impl Clone for Buffer {
    fn clone(&self) -> Self {
        let inner = &self.inner;
        let cloned = BufferInner {
            id: NEXT_BUFFER_ID.fetch_add(1, Ordering::Relaxed),
            revision: inner.revision,
            document: inner.document.clone(),
            multiline: inner.multiline,
        };

        Self { inner: cloned }
    }
}

impl Position {
    pub fn new(index: usize) -> Self {
        Self::with_affinity(index, Affinity::Downstream)
    }
    pub fn with_affinity(index: usize, affinity: Affinity) -> Self {
        Self { index, affinity }
    }
}
impl From<usize> for Position {
    fn from(value: usize) -> Self {
        Self::new(value)
    }
}
impl From<Cursor> for Position {
    fn from(value: Cursor) -> Self {
        Self::with_affinity(value.index, value.affinity)
    }
}
impl PartialEq<std::ops::Range<usize>> for Range {
    fn eq(&self, other: &std::ops::Range<usize>) -> bool {
        self.start == other.start && self.end == other.end
    }
}
impl PartialEq<Range> for std::ops::Range<usize> {
    fn eq(&self, other: &Range) -> bool {
        self.start == other.start && self.end == other.end
    }
}
impl Range {
    pub fn new(start: usize, end: usize) -> Self {
        Self { start, end }
    }
    pub fn collapsed(index: usize) -> Self {
        Self::new(index, index)
    }
    pub fn as_range(self) -> std::ops::Range<usize> {
        self.start.min(self.end)..self.start.max(self.end)
    }
    pub fn is_empty(self) -> bool {
        self.start == self.end
    }
}
impl From<std::ops::Range<usize>> for Range {
    fn from(value: std::ops::Range<usize>) -> Self {
        Self::new(value.start, value.end)
    }
}
impl Selection {
    pub fn new(anchor: Position, focus: Position) -> Self {
        Self { anchor, focus }
    }
    pub fn range(self) -> Range {
        Range::new(self.anchor.index, self.focus.index)
    }
}
