use std::cell::{Cell, RefCell};
use std::fmt;
use std::fs::File;
use std::io;
use std::path::{Path, PathBuf};
use std::rc::Rc;
use std::sync::atomic::{AtomicU64, Ordering};

use unicode_segmentation::UnicodeSegmentation;

use super::unicode::{
    floor_grapheme_boundary, grapheme_range_in_text, next_grapheme_boundary, next_word_boundary,
    normalize_for_mode, normalize_multiline, paragraph_end_boundary, paragraph_start_boundary,
    previous_grapheme_boundary, previous_word_boundary, word_range_at,
};

pub mod mark;
pub use mark::Mark;
use mark::{Gravity as MarkGravity, Range as MarkRange};

static NEXT_BUFFER_ID: AtomicU64 = AtomicU64::new(1);
static NEXT_LINE_ID: AtomicU64 = AtomicU64::new(1);

pub(crate) type Cursor = glyphon::Cursor;
pub(super) type Selection = glyphon::cosmic_text::Selection;

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash)]
pub enum TextAffinity {
    Upstream,
    #[default]
    Downstream,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash)]
pub struct TextPosition {
    pub index: usize,
    pub affinity: TextAffinity,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash)]
pub struct TextRange {
    pub start: usize,
    pub end: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct TextSelection {
    pub anchor: TextPosition,
    pub focus: TextPosition,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct LineId(u64);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub(crate) struct LineLayoutIdentity {
    pub(crate) id: LineId,
    pub(crate) revision: u64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TextMotion {
    VisualLeft,
    VisualRight,
    VisualUp,
    VisualDown,
    PageUp,
    PageDown,
    LogicalPrevious,
    LogicalNext,
    WordPrevious,
    WordNext,
    LineStart,
    LineEnd,
    ParagraphStart,
    ParagraphEnd,
    DocumentStart,
    DocumentEnd,
}

#[derive(Clone)]
pub struct Buffer {
    pub(super) inner: Rc<RefCell<BufferInner>>,
    pub(super) revision: u64,
}

#[derive(Debug)]
pub(super) struct BufferInner {
    pub(super) id: u64,
    pub(super) revision: u64,
    pub(super) document: TextDocument,
    pub(super) cursor: Mark,
    pub(super) selection: Option<MarkRange>,
    pub(super) multiline: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum TextLineEnding {
    None,
    Lf,
}

impl TextLineEnding {
    fn as_str(self) -> &'static str {
        match self {
            Self::None => "",
            Self::Lf => "\n",
        }
    }

    fn len(self) -> usize {
        self.as_str().len()
    }

    fn cosmic(self) -> glyphon::cosmic_text::LineEnding {
        match self {
            Self::None => glyphon::cosmic_text::LineEnding::None,
            Self::Lf => glyphon::cosmic_text::LineEnding::Lf,
        }
    }
}

impl From<glyphon::cosmic_text::LineEnding> for TextLineEnding {
    fn from(value: glyphon::cosmic_text::LineEnding) -> Self {
        match value {
            glyphon::cosmic_text::LineEnding::None => Self::None,
            _ => Self::Lf,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum TextPieceSource {
    OriginalOwned,
    OriginalMapped,
    Add,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct TextPiece {
    source: TextPieceSource,
    start: usize,
    len: usize,
}

#[derive(Clone)]
pub struct MappedTextSource {
    path: PathBuf,
    mmap: Rc<memmap2::Mmap>,
}

impl fmt::Debug for MappedTextSource {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("MappedTextSource")
            .field("path", &self.path)
            .field("len", &self.mmap.len())
            .finish()
    }
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
enum TextOriginal {
    Owned(Rc<str>),
    Mapped(Rc<MappedTextSource>),
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
struct TextSummary {
    utf8_len: usize,
    utf16_len: usize,
    line_count: usize,
    grapheme_count: usize,
    first_line_len: usize,
    last_line_len: usize,
    max_line_len: usize,
    dirty_ranges: usize,
}

impl TextSummary {
    fn from_lines(lines: &[TextLine]) -> Self {
        let mut summary = Self::default();
        for line in lines {
            summary.add_line(line);
        }
        summary
    }

    fn add_summary(&mut self, other: Self) {
        if other.line_count == 0 {
            return;
        }
        if self.line_count == 0 {
            self.first_line_len = other.first_line_len;
        }
        self.utf8_len = self.utf8_len.saturating_add(other.utf8_len);
        self.utf16_len = self.utf16_len.saturating_add(other.utf16_len);
        self.line_count = self.line_count.saturating_add(other.line_count);
        self.grapheme_count = self.grapheme_count.saturating_add(other.grapheme_count);
        self.last_line_len = other.last_line_len;
        self.max_line_len = self.max_line_len.max(other.max_line_len);
        self.dirty_ranges = self.dirty_ranges.saturating_add(other.dirty_ranges);
    }

    fn add_line(&mut self, line: &TextLine) {
        let text_len = line.text.len();
        if self.line_count == 0 {
            self.first_line_len = text_len;
        }
        self.utf8_len = self.utf8_len.saturating_add(text_len + line.ending.len());
        self.utf16_len = self
            .utf16_len
            .saturating_add(line.text.encode_utf16().count() + line.ending.len());
        self.line_count += 1;
        self.grapheme_count = self
            .grapheme_count
            .saturating_add(line.grapheme_boundaries.len().saturating_sub(1));
        self.last_line_len = text_len;
        self.max_line_len = self.max_line_len.max(text_len);
        self.dirty_ranges += usize::from(line.dirty);
    }
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
struct TextLine {
    id: LineId,
    pub(super) revision: u64,
    pieces: Vec<TextPiece>,
    text: String,
    ending: TextLineEnding,
    grapheme_boundaries: Rc<Vec<usize>>,
    dirty: bool,
}

impl TextLine {
    fn new(text: impl Into<String>, ending: TextLineEnding) -> Self {
        Self::from_cached_text(text.into(), ending, Vec::new(), 0)
    }

    fn from_piece(
        text: impl Into<String>,
        ending: TextLineEnding,
        source: TextPieceSource,
        source_start: usize,
        revision: u64,
    ) -> Self {
        let text = text.into();
        let len = text.len();
        let pieces = (len > 0)
            .then_some(TextPiece {
                source,
                start: source_start,
                len,
            })
            .into_iter()
            .collect();
        Self::from_cached_text(text, ending, pieces, revision)
    }

    fn from_cached_text(
        text: String,
        ending: TextLineEnding,
        pieces: Vec<TextPiece>,
        revision: u64,
    ) -> Self {
        let mut line = Self {
            id: LineId(NEXT_LINE_ID.fetch_add(1, Ordering::Relaxed)),
            revision,
            pieces,
            text,
            ending,
            grapheme_boundaries: Rc::new(Vec::new()),
            dirty: false,
        };
        line.rebuild_boundaries();
        line
    }

    fn with_revision(mut self, revision: u64) -> Self {
        self.revision = revision;
        self
    }

    fn rebuild_boundaries(&mut self) {
        let mut boundaries = Vec::new();
        boundaries.push(0);
        for (index, _) in self.text.grapheme_indices(true) {
            if index != 0 {
                boundaries.push(index);
            }
        }
        if boundaries.last().copied() != Some(self.text.len()) {
            boundaries.push(self.text.len());
        }
        self.grapheme_boundaries = Rc::new(boundaries);
    }

    fn total_len(&self) -> usize {
        self.text.len() + self.ending.len()
    }

    fn floor_grapheme(&self, local: usize) -> usize {
        let local = local.min(self.text.len());
        let index = self
            .grapheme_boundaries
            .partition_point(|boundary| *boundary <= local)
            .saturating_sub(1);
        self.grapheme_boundaries.get(index).copied().unwrap_or(0)
    }

    fn ceil_grapheme(&self, local: usize) -> usize {
        let local = local.min(self.text.len());
        self.grapheme_boundaries
            .iter()
            .copied()
            .find(|boundary| *boundary >= local)
            .unwrap_or(self.text.len())
    }

    #[allow(dead_code)]
    fn to_buffer_line(&self, attrs: glyphon::AttrsList) -> glyphon::BufferLine {
        glyphon::BufferLine::new(
            &self.text,
            self.ending.cosmic(),
            attrs,
            glyphon::Shaping::Advanced,
        )
    }

    #[cfg(test)]
    fn piece_source_lengths(&self) -> (usize, usize, usize) {
        let mut owned = 0usize;
        let mut mapped = 0usize;
        let mut add = 0usize;
        for piece in &self.pieces {
            let _start = piece.start;
            match piece.source {
                TextPieceSource::OriginalOwned => owned = owned.saturating_add(piece.len),
                TextPieceSource::OriginalMapped => mapped = mapped.saturating_add(piece.len),
                TextPieceSource::Add => add = add.saturating_add(piece.len),
            }
        }
        (owned, mapped, add)
    }
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
struct TextLineBlock {
    lines: Vec<TextLine>,
    summary: TextSummary,
}

impl TextLineBlock {
    fn new(lines: Vec<TextLine>) -> Self {
        let summary = TextSummary::from_lines(&lines);
        Self { lines, summary }
    }

    fn line_count(&self) -> usize {
        self.lines.len()
    }
}

pub(crate) const TEXT_DOCUMENT_BLOCK_TARGET_LINES: usize = 128;
const TEXT_DOCUMENT_BLOCK_MAX_LINES: usize = 256;
const TEXT_MAPPED_INDEX_PAGE_BYTES: usize = 64 * 1024;

#[allow(dead_code)]
#[derive(Debug, Clone)]
struct TextLineTree {
    blocks: Vec<TextLineBlock>,
    summary: TextSummary,
}

impl TextLineTree {
    fn from_lines(lines: Vec<TextLine>) -> Self {
        let mut tree = Self {
            blocks: Self::blocks_from_lines(lines),
            summary: TextSummary::default(),
        };
        tree.rebuild_summary();
        tree
    }

    fn blocks_from_lines(lines: Vec<TextLine>) -> Vec<TextLineBlock> {
        let mut blocks = Vec::new();
        let mut chunk = Vec::with_capacity(TEXT_DOCUMENT_BLOCK_TARGET_LINES);
        for line in lines {
            chunk.push(line);
            if chunk.len() >= TEXT_DOCUMENT_BLOCK_TARGET_LINES {
                blocks.push(TextLineBlock::new(std::mem::take(&mut chunk)));
            }
        }
        if !chunk.is_empty() {
            blocks.push(TextLineBlock::new(chunk));
        }
        if blocks.is_empty() {
            blocks.push(TextLineBlock::new(vec![TextLine::new(
                "",
                TextLineEnding::None,
            )]));
        }
        blocks
    }

    fn rebuild_summary(&mut self) {
        let mut summary = TextSummary::default();
        for block in &self.blocks {
            summary.add_summary(block.summary);
        }
        self.summary = summary;
    }

    fn line_count(&self) -> usize {
        self.summary.line_count.max(1)
    }

    fn text_len(&self) -> usize {
        self.summary.utf8_len
    }

    fn line(&self, line: usize) -> Option<&TextLine> {
        let (block, local) = self.locate_line(line);
        self.blocks
            .get(block)
            .and_then(|block| block.lines.get(local))
    }

    fn line_start(&self, line: usize) -> usize {
        let target = line.min(self.line_count().saturating_sub(1));
        let mut offset = 0usize;
        let mut remaining = target;
        for block in &self.blocks {
            if remaining >= block.line_count() {
                offset = offset.saturating_add(block.summary.utf8_len);
                remaining -= block.line_count();
                continue;
            }
            for line in block.lines.iter().take(remaining) {
                offset = offset.saturating_add(line.total_len());
            }
            return offset;
        }
        offset
    }

    fn line_and_local_for_index(&self, index: usize) -> (usize, usize) {
        let mut remaining = index.min(self.text_len());
        let mut line_index = 0usize;
        for (block_index, block) in self.blocks.iter().enumerate() {
            if remaining > block.summary.utf8_len
                || (remaining == block.summary.utf8_len && block_index + 1 < self.blocks.len())
            {
                remaining -= block.summary.utf8_len;
                line_index += block.line_count();
                continue;
            }
            for line in &block.lines {
                let text_len = line.text.len();
                let total = line.total_len();
                if remaining <= text_len {
                    return (line_index, remaining);
                }
                if remaining < total {
                    return (line_index, text_len);
                }
                remaining = remaining.saturating_sub(total);
                line_index += 1;
            }
        }
        let last = self.line_count().saturating_sub(1);
        let local = self.line(last).map(|line| line.text.len()).unwrap_or(0);
        (last, local)
    }

    fn splice_lines(
        &mut self,
        start_line: usize,
        old_line_count: usize,
        replacement: Vec<TextLine>,
    ) {
        let start_line = start_line.min(self.line_count().saturating_sub(1));
        let old_line_count = old_line_count.max(1);
        let end_line = start_line
            .saturating_add(old_line_count)
            .saturating_sub(1)
            .min(self.line_count().saturating_sub(1));
        let (start_block, start_local) = self.locate_line(start_line);
        let (end_block, end_local) = self.locate_line(end_line);
        let mut merged = Vec::new();
        if let Some(block) = self.blocks.get(start_block) {
            merged.extend(block.lines.iter().take(start_local).cloned());
        }
        merged.extend(replacement);
        if let Some(block) = self.blocks.get(end_block) {
            merged.extend(block.lines.iter().skip(end_local + 1).cloned());
        }
        if merged.is_empty() {
            merged.push(TextLine::new("", TextLineEnding::None));
        }
        let new_blocks = Self::blocks_from_lines(merged);
        self.blocks.splice(start_block..=end_block, new_blocks);
        if self.blocks.is_empty() {
            self.blocks = Self::blocks_from_lines(vec![TextLine::new("", TextLineEnding::None)]);
        }
        self.rebalance_oversized_blocks();
        self.rebuild_summary();
    }

    fn locate_line(&self, line: usize) -> (usize, usize) {
        let mut remaining = line.min(self.line_count().saturating_sub(1));
        for (block_index, block) in self.blocks.iter().enumerate() {
            if remaining < block.line_count() {
                return (block_index, remaining);
            }
            remaining -= block.line_count();
        }
        let block_index = self.blocks.len().saturating_sub(1);
        let local = self
            .blocks
            .get(block_index)
            .map(|block| block.line_count().saturating_sub(1))
            .unwrap_or(0);
        (block_index, local)
    }

    fn rebalance_oversized_blocks(&mut self) {
        let mut index = 0;
        while index < self.blocks.len() {
            if self.blocks[index].line_count() <= TEXT_DOCUMENT_BLOCK_MAX_LINES {
                index += 1;
                continue;
            }
            let tail = self.blocks[index]
                .lines
                .split_off(TEXT_DOCUMENT_BLOCK_TARGET_LINES);
            let head = std::mem::take(&mut self.blocks[index].lines);
            self.blocks[index] = TextLineBlock::new(head);
            self.blocks.insert(index + 1, TextLineBlock::new(tail));
            index += 1;
        }
    }
}

#[cfg(test)]
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub(super) struct TextDocumentStatsSnapshot {
    pub(super) full_materializations: usize,
    pub(super) total_document_scans: usize,
    pub(super) piece_tree_updates: usize,
    pub(super) mapped_index_pages_scanned: usize,
}

#[derive(Debug, Default)]
struct TextDocumentStats {
    full_materializations: Cell<usize>,
    total_document_scans: Cell<usize>,
    piece_tree_updates: Cell<usize>,
    mapped_index_pages_scanned: Cell<usize>,
}

impl Clone for TextDocumentStats {
    fn clone(&self) -> Self {
        Self {
            full_materializations: Cell::new(self.full_materializations.get()),
            total_document_scans: Cell::new(self.total_document_scans.get()),
            piece_tree_updates: Cell::new(self.piece_tree_updates.get()),
            mapped_index_pages_scanned: Cell::new(self.mapped_index_pages_scanned.get()),
        }
    }
}

impl TextDocumentStats {
    #[cfg(test)]
    fn snapshot(&self) -> TextDocumentStatsSnapshot {
        TextDocumentStatsSnapshot {
            full_materializations: self.full_materializations.get(),
            total_document_scans: self.total_document_scans.get(),
            piece_tree_updates: self.piece_tree_updates.get(),
            mapped_index_pages_scanned: self.mapped_index_pages_scanned.get(),
        }
    }

    #[cfg(test)]
    fn reset(&self) {
        self.full_materializations.set(0);
        self.total_document_scans.set(0);
        self.piece_tree_updates.set(0);
        // Keep mapped indexing progress; reset only transient materialization/edit counters.
    }
}

#[derive(Debug, Clone)]
pub(super) struct TextDocument {
    #[allow(dead_code)]
    original: TextOriginal,
    add_buffer: String,
    tree: TextLineTree,
    pub(super) revision: u64,
    stats: TextDocumentStats,
}

impl TextDocument {
    pub(super) fn from_text(text: &str) -> Self {
        let original: Rc<str> = Rc::from(text);
        Self::from_source_text(
            text,
            TextOriginal::Owned(original),
            TextPieceSource::OriginalOwned,
            0,
        )
    }

    pub(super) fn open_mapped(path: impl AsRef<Path>) -> io::Result<Self> {
        let path = path.as_ref();
        let file = File::open(path)?;
        let mmap = Rc::new(unsafe { memmap2::Mmap::map(&file)? });
        let text = std::str::from_utf8(&mmap[..]).map_err(|error| {
            io::Error::new(
                io::ErrorKind::InvalidData,
                format!("mapped text is not valid UTF-8: {error}"),
            )
        })?;
        let mapped = Rc::new(MappedTextSource {
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
            add_buffer: String::new(),
            tree: TextLineTree::from_lines(lines),
            revision: 0,
            stats: TextDocumentStats::default(),
        }
    }

    pub(super) fn line_count(&self) -> usize {
        self.tree.line_count()
    }

    pub(super) fn text_len(&self) -> usize {
        self.tree.text_len()
    }

    pub(super) fn line_starts(&self) -> Rc<Vec<usize>> {
        self.stats
            .total_document_scans
            .set(self.stats.total_document_scans.get() + 1);
        let mut starts = Vec::with_capacity(self.line_count());
        let mut offset = 0usize;
        for block in &self.tree.blocks {
            for line in &block.lines {
                starts.push(offset);
                offset = offset.saturating_add(line.total_len());
            }
        }
        if starts.is_empty() {
            starts.push(0);
        }
        Rc::new(starts)
    }

    pub(super) fn line_start(&self, line: usize) -> usize {
        self.tree.line_start(line)
    }

    pub(super) fn line_text_len(&self, line: usize) -> usize {
        self.tree
            .line(line)
            .map(|line| line.text.len())
            .unwrap_or(0)
    }

    #[allow(dead_code)]
    pub(super) fn line_ending_len(&self, line: usize) -> usize {
        self.tree
            .line(line)
            .map(|line| line.ending.len())
            .unwrap_or(0)
    }

    pub(super) fn text(&self) -> String {
        self.stats
            .full_materializations
            .set(self.stats.full_materializations.get() + 1);
        let mut text = String::with_capacity(self.text_len());
        for block in &self.tree.blocks {
            for line in &block.lines {
                text.push_str(&line.text);
                text.push_str(line.ending.as_str());
            }
        }
        text
    }

    pub(super) fn text_for_range(&self, range: std::ops::Range<usize>) -> String {
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

    pub(super) fn text_for_line_range(&self, start: usize, end: usize) -> String {
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

    pub(super) fn line_layout_identity(&self, line: usize) -> Option<LineLayoutIdentity> {
        self.tree.line(line).map(|line| LineLayoutIdentity {
            id: line.id,
            revision: line.revision,
        })
    }

    pub(super) fn replace_range(
        &mut self,
        range: TextRange,
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
        let add_start = self.add_buffer.len();
        self.add_buffer.push_str(&replacement_text);
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

    pub(super) fn cursor_for_text_index(&self, index: usize) -> Cursor {
        let index = index.min(self.text_len());
        let (line, local) = self.line_and_local_for_index(index);
        let local = self
            .tree
            .line(line)
            .map(|line| line.floor_grapheme(local))
            .unwrap_or(0);
        Cursor::new(line, local)
    }

    pub(super) fn text_index_for_cursor(&self, cursor: Cursor) -> usize {
        let line = cursor.line.min(self.line_count().saturating_sub(1));
        let local = self
            .tree
            .line(line)
            .map(|line| line.floor_grapheme(cursor.index))
            .unwrap_or(0);
        self.line_start(line) + local
    }

    pub(super) fn mark_for_position(&self, position: TextPosition) -> Option<Mark> {
        let cursor = self.cursor_for_text_index(position.index);
        let line = self.tree.line(cursor.line)?;
        Some(Mark {
            line_id: line.id,
            byte_offset: cursor.index,
            affinity: position.affinity,
            gravity: MarkGravity::Downstream,
        })
    }

    pub(super) fn mark_range_for_selection(&self, selection: TextSelection) -> Option<MarkRange> {
        Some(MarkRange {
            start: self.mark_for_position(selection.anchor)?,
            end: self.mark_for_position(selection.focus)?,
        })
    }

    pub(super) fn position_for_mark(&self, anchor: Mark) -> Option<TextPosition> {
        let (_, offset, line) = self.line_index_start_and_line_for_id(anchor.line_id)?;
        let local = line.floor_grapheme(anchor.byte_offset);
        Some(TextPosition::with_affinity(offset + local, anchor.affinity))
    }

    pub(super) fn mark_for_cursor(&self, cursor: Cursor) -> Option<Mark> {
        let line_index = cursor.line.min(self.line_count().saturating_sub(1));
        let line = self.tree.line(line_index)?;
        Some(Mark {
            line_id: line.id,
            byte_offset: line.floor_grapheme(cursor.index),
            affinity: text_affinity(cursor.affinity),
            gravity: MarkGravity::Downstream,
        })
    }

    pub(super) fn cursor_for_mark(&self, anchor: Mark) -> Option<Cursor> {
        let (line_index, _, line) = self.line_index_start_and_line_for_id(anchor.line_id)?;
        Some(Cursor::new_with_affinity(
            line_index,
            line.floor_grapheme(anchor.byte_offset),
            glyph_affinity(anchor.affinity),
        ))
    }

    pub(super) fn selection_for_mark_range(&self, range: MarkRange) -> Option<TextSelection> {
        Some(TextSelection::new(
            self.position_for_mark(range.start)?,
            self.position_for_mark(range.end)?,
        ))
    }

    pub(super) fn ordered_cursor_range_for_mark_range(
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
            for line in &block.lines {
                if line.id == line_id {
                    return Some((line_index, offset, line));
                }
                line_index = line_index.saturating_add(1);
                offset = offset.saturating_add(line.total_len());
            }
        }
        None
    }

    pub(super) fn line_and_local_for_index(&self, index: usize) -> (usize, usize) {
        self.tree.line_and_local_for_index(index)
    }

    pub(super) fn snap_range(&self, range: TextRange) -> std::ops::Range<usize> {
        let range = range.as_range();
        if range.is_empty() {
            let index = self.floor_grapheme_boundary(range.start);
            return index..index;
        }
        self.floor_grapheme_boundary(range.start)..self.ceil_grapheme_boundary(range.end)
    }

    pub(super) fn floor_grapheme_boundary(&self, index: usize) -> usize {
        let (line_index, local) = self.line_and_local_for_index(index);
        let Some(line) = self.tree.line(line_index) else {
            return 0;
        };
        self.line_start(line_index) + line.floor_grapheme(local)
    }

    pub(super) fn ceil_grapheme_boundary(&self, index: usize) -> usize {
        let (line_index, local) = self.line_and_local_for_index(index);
        let Some(line) = self.tree.line(line_index) else {
            return self.text_len();
        };
        self.line_start(line_index) + line.ceil_grapheme(local)
    }

    pub(super) fn previous_grapheme_boundary_index(&self, index: usize) -> usize {
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

    pub(super) fn next_grapheme_boundary_index(&self, index: usize) -> usize {
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

    pub(super) fn previous_word_boundary_index(&self, index: usize) -> usize {
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

    pub(super) fn next_word_boundary_index(&self, index: usize) -> usize {
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
            for line in &block.lines {
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

#[allow(dead_code)]
#[derive(Debug, Clone)]
struct BufferLineIndex {
    text_lens: Vec<usize>,
    ending_lens: Vec<usize>,
}

#[allow(dead_code)]
impl BufferLineIndex {
    fn from_document(document: &TextDocument) -> Self {
        Self {
            text_lens: (0..document.line_count())
                .map(|line| document.line_text_len(line))
                .collect(),
            ending_lens: (0..document.line_count())
                .map(|line| document.line_ending_len(line))
                .collect(),
        }
    }

    fn replace_from_document(
        &mut self,
        document: &TextDocument,
        start_line: usize,
        old_line_count: usize,
        new_line_count: usize,
    ) {
        let start = start_line.min(self.line_count());
        let end = start
            .saturating_add(old_line_count)
            .min(self.text_lens.len());
        let new_end = start
            .saturating_add(new_line_count)
            .min(document.line_count());
        self.text_lens.splice(
            start..end,
            (start..new_end).map(|line| document.line_text_len(line)),
        );
        let end = start
            .saturating_add(old_line_count)
            .min(self.ending_lens.len());
        self.ending_lens.splice(
            start..end,
            (start..new_end).map(|line| document.line_ending_len(line)),
        );
    }

    fn line_count(&self) -> usize {
        self.text_lens.len().max(1)
    }

    fn line_text_len(&self, line: usize) -> usize {
        self.text_lens
            .get(line.min(self.line_count().saturating_sub(1)))
            .copied()
            .unwrap_or(0)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct TextEditImpact {
    pub(crate) range: TextRange,
    pub(crate) affected_start_line: usize,
    pub(crate) affected_start_line_id: Option<LineId>,
    pub(crate) removed_line_count: usize,
    pub(crate) inserted_line_count: usize,
    pub(crate) deleted_bytes: usize,
    pub(crate) inserted_bytes: usize,
    pub(crate) caret_mark: Mark,
}

#[derive(Debug, Clone)]
pub(crate) struct TextChange {
    pub(super) before: BufferMarker,
    pub(super) after: BufferMarker,
    pub(super) transaction: TextTransaction,
}

#[derive(Debug, Clone, Default)]
pub(crate) struct TextTransaction {
    pub(crate) deltas: Vec<TextDelta>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct TextDelta {
    pub(crate) kind: TextEditKind,
    pub(crate) range: TextRange,
    pub(crate) deleted: String,
    pub(crate) inserted: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum TextEditKind {
    Insert,
    Delete,
    Replace,
    Move,
    ImeCommit,
}

impl TextEditImpact {
    pub(crate) fn affected_line_count(&self) -> usize {
        self.removed_line_count.max(self.inserted_line_count).max(1)
    }

    pub(crate) fn line_count_delta(&self) -> isize {
        self.inserted_line_count as isize - self.removed_line_count as isize
    }
}

impl TextTransaction {
    pub(crate) fn replace(
        range: TextRange,
        deleted: String,
        inserted: String,
        kind: TextEditKind,
    ) -> Self {
        let mut transaction = Self::default();
        transaction.push_replace(range, deleted, inserted, kind);
        transaction
    }

    fn push_replace(
        &mut self,
        range: TextRange,
        deleted: String,
        inserted: String,
        kind: TextEditKind,
    ) {
        if range.start == range.end && deleted.is_empty() && inserted.is_empty() {
            return;
        }
        self.deltas.push(TextDelta {
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
                TextRange::new(delta.range.start, delta.range.start + delta.inserted.len()),
                delta.inserted.clone(),
                delta.deleted.clone(),
                delta.kind,
            );
        }
        inverse
    }

    pub(crate) fn try_coalesce_typing(&mut self, next: &TextTransaction) -> bool {
        if self.deltas.len() != 1 || next.deltas.len() != 1 {
            return false;
        }
        let current = &mut self.deltas[0];
        let next = &next.deltas[0];
        if current.kind != TextEditKind::Insert || next.kind != TextEditKind::Insert {
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

impl TextDelta {
    #[allow(dead_code)]
    pub(crate) fn inserted_end(&self) -> usize {
        self.range.start + self.inserted.len()
    }
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
        let cursor = document_end_mark(&document);
        let inner = BufferInner {
            id: NEXT_BUFFER_ID.fetch_add(1, Ordering::Relaxed),
            revision: document.revision,
            document,
            cursor,
            selection: None,
            multiline,
        };
        Self {
            revision: inner.revision,
            inner: Rc::new(RefCell::new(inner)),
        }
    }
    pub fn id(&self) -> u64 {
        self.inner.borrow().id
    }
    pub fn revision(&self) -> u64 {
        self.inner.borrow().revision
    }

    #[cfg(test)]
    pub(super) fn reset_line_index_stats(&mut self) {
        self.inner.borrow().document.reset_stats();
    }

    #[cfg(test)]
    pub(super) fn line_index_stats(&self) -> (usize, usize) {
        let stats = self.inner.borrow().document.stats();
        (stats.full_materializations, stats.piece_tree_updates)
    }

    #[cfg(test)]
    pub(super) fn reset_document_stats(&self) {
        self.inner.borrow().document.reset_stats();
    }

    #[cfg(test)]
    pub(super) fn document_stats(&self) -> TextDocumentStatsSnapshot {
        self.inner.borrow().document.stats()
    }

    #[cfg(test)]
    pub(super) fn document_piece_source_lengths(&self) -> (usize, usize, usize) {
        self.inner.borrow().document.piece_source_lengths()
    }

    #[cfg(test)]
    #[cfg(test)]
    pub(super) fn original_len(&self) -> usize {
        self.inner.borrow().document.original_len()
    }
    pub(super) fn marker(&self) -> BufferMarker {
        let inner = self.inner.borrow();
        BufferMarker::new(&inner)
    }
    pub fn text(&self) -> String {
        self.inner.borrow().document.text()
    }
    pub fn to_plain_text(&self) -> String {
        self.text()
    }
    pub fn len(&self) -> usize {
        self.text_len()
    }
    fn text_len(&self) -> usize {
        self.inner.borrow().document.text_len()
    }
    pub fn is_empty(&self) -> bool {
        self.text_len() == 0
    }
    pub fn is_multiline(&self) -> bool {
        self.inner.borrow().multiline
    }
    pub(crate) fn promote_to_multiline(&self) {
        self.inner.borrow_mut().multiline = true;
    }
    pub fn logical_line_count(&self) -> usize {
        self.inner.borrow().document.line_count()
    }
    pub(crate) fn line_start_offsets(&self) -> Rc<Vec<usize>> {
        self.inner.borrow().document.line_starts()
    }
    pub fn position(&self) -> TextPosition {
        let inner = self.inner.borrow();
        inner
            .document
            .position_for_mark(inner.cursor)
            .unwrap_or_else(|| TextPosition::new(inner.document.text_len()))
    }
    pub fn selection(&self) -> Option<TextSelection> {
        let inner = self.inner.borrow();
        inner
            .selection
            .and_then(|selection| inner.document.selection_for_mark_range(selection))
    }
    pub fn mark(&self) -> Option<Mark> {
        Some(self.inner.borrow().cursor)
    }
    pub fn mark_selection(&self) -> Option<MarkRange> {
        self.inner.borrow().selection
    }
    pub fn position_for_mark(&self, mark: Mark) -> Option<TextPosition> {
        self.inner.borrow().document.position_for_mark(mark)
    }
    pub fn mark_for_position(&self, position: TextPosition) -> Option<Mark> {
        self.inner.borrow().document.mark_for_position(position)
    }
    pub fn cursor(&self) -> Cursor {
        let inner = self.inner.borrow();
        inner
            .document
            .cursor_for_mark(inner.cursor)
            .unwrap_or_else(|| {
                inner
                    .document
                    .cursor_for_text_index(inner.document.text_len())
            })
    }
    pub fn selection_bounds(&self) -> Option<(Cursor, Cursor)> {
        let inner = self.inner.borrow();
        let selection = inner.selection?;
        let (start, end) = inner
            .document
            .ordered_cursor_range_for_mark_range(selection)?;
        (inner.document.text_index_for_cursor(start) < inner.document.text_index_for_cursor(end))
            .then_some((start, end))
    }
    pub fn selected_range(&self) -> Option<TextRange> {
        let inner = self.inner.borrow();
        let selection = inner.selection?;
        let start = inner.document.position_for_mark(selection.start)?.index;
        let end = inner.document.position_for_mark(selection.end)?.index;
        let (start, end) = if start <= end {
            (start, end)
        } else {
            (end, start)
        };
        (start < end).then_some(TextRange::new(start, end))
    }
    pub fn selected_text(&self) -> Option<String> {
        let range = self.selected_range()?.as_range();
        Some(self.inner.borrow().document.text_for_range(range))
    }
    pub fn has_selection(&self) -> bool {
        self.has_non_empty_selection()
    }
    pub(crate) fn has_non_empty_selection(&self) -> bool {
        self.selected_range().is_some()
    }
    pub fn position_for_text_index(&self, index: usize) -> TextPosition {
        let inner = self.inner.borrow();
        let cursor = inner.document.cursor_for_text_index(index);
        TextPosition::with_affinity(
            inner.document.text_index_for_cursor(cursor),
            text_affinity(cursor.affinity),
        )
    }
    pub fn text_index_for_position(&self, position: TextPosition) -> usize {
        let inner = self.inner.borrow();
        let cursor = inner.document.cursor_for_text_index(position.index);
        inner
            .document
            .text_index_for_cursor(Cursor::new_with_affinity(
                cursor.line,
                cursor.index,
                glyph_affinity(position.affinity),
            ))
    }
    pub(crate) fn cursor_for_text_index(&self, index: usize) -> Cursor {
        let inner = self.inner.borrow();
        inner.document.cursor_for_text_index(index)
    }
    pub(crate) fn text_index_for_cursor(&self, cursor: Cursor) -> usize {
        let inner = self.inner.borrow();
        inner.document.text_index_for_cursor(cursor)
    }
    #[allow(dead_code)]
    fn clamp_cursor(&self, cursor: Cursor) -> Cursor {
        let inner = self.inner.borrow();
        inner
            .document
            .cursor_for_text_index(inner.document.text_index_for_cursor(cursor))
    }
    #[allow(dead_code)]
    fn clamp_selection(&self, selection: Selection) -> Selection {
        let inner = self.inner.borrow();
        selection_mark_for_document(&inner.document, selection)
            .and_then(|anchor| inner.document.cursor_for_mark(anchor))
            .map(Selection::Normal)
            .unwrap_or(Selection::None)
    }
    pub(crate) fn cloned_cosmic_buffer(&self) -> glyphon::Buffer {
        cosmic_buffer_from_text(&self.text())
    }
    #[allow(dead_code)]
    pub(crate) fn clone_cosmic_buffer_with_attrs(
        &self,
        attrs: glyphon::AttrsList,
    ) -> glyphon::Buffer {
        let mut buffer = self.cloned_cosmic_buffer();
        for line in &mut buffer.lines {
            line.set_attrs_list(attrs.clone());
        }
        buffer
    }
    pub(crate) fn set_cursor_and_selection(&mut self, cursor: Cursor, selection: Selection) {
        let mut inner = self.inner.borrow_mut();
        let cursor = inner
            .document
            .mark_for_cursor(cursor)
            .unwrap_or_else(|| document_end_mark(&inner.document));
        let selection =
            selection_mark_for_document(&inner.document, selection).map(|anchor| MarkRange {
                start: anchor,
                end: cursor,
            });
        inner.cursor = cursor;
        inner.selection = selection;
    }
    #[allow(dead_code)]
    pub(crate) fn set_mark_selection(&mut self, cursor: Mark, selection: Option<MarkRange>) {
        let mut inner = self.inner.borrow_mut();
        inner.cursor = if inner.document.position_for_mark(cursor).is_some() {
            cursor
        } else {
            document_end_mark(&inner.document)
        };
        inner.selection = selection.and_then(|range| {
            (inner.document.position_for_mark(range.start).is_some()
                && inner.document.position_for_mark(range.end).is_some())
            .then_some(range)
        });
    }
    fn replace_text_range_with_kind(
        &mut self,
        range: TextRange,
        text: &str,
        kind: TextEditKind,
    ) -> TextTransaction {
        self.replace_text_range_with_kind_and_impact(range, text, kind)
            .0
    }
    pub(crate) fn replace_text_range_with_kind_and_impact(
        &mut self,
        range: TextRange,
        text: &str,
        kind: TextEditKind,
    ) -> (TextTransaction, Option<TextEditImpact>) {
        let inserted = normalize_for_mode(self.is_multiline(), text);
        let range = {
            let inner = self.inner.borrow();
            inner.document.snap_range(range)
        };
        if range.is_empty() && inserted.is_empty() {
            return (TextTransaction::default(), None);
        }

        let (range, deleted, inserted_text, impact) = {
            let mut inner = self.inner.borrow_mut();
            let (range, removed, inserted, start_line, old_line_count, new_line_count) = inner
                .document
                .replace_range(TextRange::new(range.start, range.end), &inserted);
            inner.revision = inner.document.revision;
            self.revision = inner.revision;
            let cursor = inner
                .document
                .cursor_for_text_index(range.start + inserted.len());
            inner.cursor = inner
                .document
                .mark_for_cursor(cursor)
                .unwrap_or_else(|| document_end_mark(&inner.document));
            inner.selection = None;
            let affected_start_line_id = inner
                .document
                .line_layout_identity(start_line)
                .map(|identity| identity.id);
            let impact = TextEditImpact {
                range: TextRange::new(range.start, range.end),
                affected_start_line: start_line,
                affected_start_line_id,
                removed_line_count: old_line_count,
                inserted_line_count: new_line_count,
                deleted_bytes: removed.len(),
                inserted_bytes: inserted.len(),
                caret_mark: inner.cursor,
            };
            (range, removed, inserted, impact)
        };

        let delta_kind = if deleted.is_empty() && !inserted_text.is_empty() {
            match kind {
                TextEditKind::ImeCommit => TextEditKind::ImeCommit,
                TextEditKind::Move => TextEditKind::Move,
                _ => TextEditKind::Insert,
            }
        } else if inserted_text.is_empty() && !deleted.is_empty() {
            TextEditKind::Delete
        } else {
            kind
        };
        (
            TextTransaction::replace(
                TextRange::new(range.start, range.end),
                deleted,
                inserted_text,
                delta_kind,
            ),
            Some(impact),
        )
    }
    pub(crate) fn move_text_range(
        &mut self,
        range: TextRange,
        to: TextPosition,
    ) -> TextTransaction {
        let (range, to, moved) = {
            let inner = self.inner.borrow();
            let range = inner.document.snap_range(range);
            let to = inner.document.floor_grapheme_boundary(to.index);
            let moved = inner.document.text_for_range(range.clone());
            (range, to, moved)
        };
        if range.is_empty() || (range.start..=range.end).contains(&to) {
            let cursor = self.cursor_for_text_index(to);
            self.set_cursor_and_selection(cursor, Selection::None);
            return TextTransaction::default();
        }
        let adjusted_to = if to > range.end {
            to - (range.end - range.start)
        } else {
            to
        };
        let mut transaction = self.replace_text_range_with_kind(
            TextRange::new(range.start, range.end),
            "",
            TextEditKind::Move,
        );
        let insert = self.replace_text_range_with_kind(
            TextRange::collapsed(adjusted_to),
            &moved,
            TextEditKind::Move,
        );
        transaction.deltas.extend(insert.deltas);
        transaction
    }
    #[allow(dead_code)]
    pub(crate) fn replace_all_text(&mut self, text: String) {
        let mut inner = self.inner.borrow_mut();
        inner.document = TextDocument::from_text(&text);
        inner.revision = inner.revision.saturating_add(1);
        inner.document.revision = inner.revision;
        self.revision = inner.revision;
        inner.cursor = document_end_mark(&inner.document);
        inner.selection = None;
    }
    pub(super) fn restore_marker(&mut self, marker: BufferMarker) {
        let mut inner = self.inner.borrow_mut();
        if inner.id == marker.buffer_id {
            inner.revision = marker.revision;
            self.revision = marker.revision;
            inner.cursor = marker.cursor_for(&inner.document);
            inner.selection = marker.selection_for(&inner.document);
        }
    }
    pub(crate) fn apply_transaction(&mut self, transaction: &TextTransaction) -> bool {
        for delta in &transaction.deltas {
            self.replace_text_range_with_kind(
                TextRange::new(delta.range.start, delta.range.end),
                &delta.inserted,
                delta.kind,
            );
        }
        true
    }
    pub(crate) fn text_for_line_range(&self, start: usize, end: usize) -> String {
        self.inner.borrow().document.text_for_line_range(start, end)
    }
    pub(crate) fn line_layout_identity(&self, line: usize) -> Option<LineLayoutIdentity> {
        self.inner.borrow().document.line_layout_identity(line)
    }
}
fn selection_mark_for_document(document: &TextDocument, selection: Selection) -> Option<Mark> {
    match selection {
        Selection::None => None,
        Selection::Normal(cursor) | Selection::Line(cursor) | Selection::Word(cursor) => {
            document.mark_for_cursor(cursor)
        }
    }
}

pub(crate) fn selection_mark_from_buffer(buffer: &Buffer) -> Option<Cursor> {
    let inner = buffer.inner.borrow();
    inner
        .selection
        .and_then(|selection| inner.document.cursor_for_mark(selection.start))
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

fn document_end_mark(document: &TextDocument) -> Mark {
    document
        .mark_for_cursor(document.cursor_for_text_index(document.text_len()))
        .expect("text documents always contain at least one line")
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct BufferMarker {
    pub(crate) buffer_id: u64,
    pub(crate) revision: u64,
    pub(crate) cursor: Mark,
    pub(crate) selection: Option<MarkRange>,
    cursor_position: TextPosition,
    selection_positions: Option<TextSelection>,
}

impl BufferMarker {
    pub(crate) fn new(inner: &BufferInner) -> Self {
        Self {
            buffer_id: inner.id,
            revision: inner.revision,
            cursor: inner.cursor,
            selection: inner.selection,
            cursor_position: inner
                .document
                .position_for_mark(inner.cursor)
                .unwrap_or_else(|| TextPosition::new(inner.document.text_len())),
            selection_positions: inner
                .selection
                .and_then(|selection| inner.document.selection_for_mark_range(selection)),
        }
    }

    pub(crate) fn cursor_for(&self, document: &TextDocument) -> Mark {
        if document.position_for_mark(self.cursor).is_some() {
            self.cursor
        } else {
            document
                .mark_for_position(self.cursor_position)
                .unwrap_or_else(|| document_end_mark(document))
        }
    }

    pub(crate) fn selection_for(&self, document: &TextDocument) -> Option<MarkRange> {
        if let Some(selection) = self.selection
            && document.position_for_mark(selection.start).is_some()
            && document.position_for_mark(selection.end).is_some()
        {
            return Some(selection);
        }
        self.selection_positions
            .and_then(|selection| document.mark_range_for_selection(selection))
    }
}

impl PartialEq for Buffer {
    fn eq(&self, other: &Self) -> bool {
        let self_marker = self.marker();
        let other_marker = other.marker();

        if self_marker.buffer_id == other_marker.buffer_id {
            return self_marker == other_marker;
        }

        self.is_multiline() == other.is_multiline()
            && self.cursor() == other.cursor()
            && self.selection() == other.selection()
            && self.to_plain_text() == other.to_plain_text()
    }
}

impl fmt::Debug for Buffer {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let marker = self.marker();
        f.debug_struct("Buffer")
            .field("id", &marker.buffer_id)
            .field("revision", &marker.revision)
            .field("cursor", &marker.cursor)
            .field("selection", &marker.selection)
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

fn default_text_field_metrics() -> glyphon::Metrics {
    glyphon::Metrics::relative(16.0, 1.25)
}

pub(crate) fn cosmic_buffer_from_text(text: &str) -> glyphon::Buffer {
    let mut buffer = glyphon::Buffer::new_empty(default_text_field_metrics());
    let attrs = glyphon::AttrsList::new(&glyphon::Attrs::new());

    buffer.lines.clear();

    if text.is_empty() {
        buffer.lines.push(glyphon::BufferLine::new(
            "",
            glyphon::cosmic_text::LineEnding::None,
            attrs,
            glyphon::Shaping::Advanced,
        ));
        return buffer;
    }

    let mut start = 0;
    for (index, _) in text.match_indices('\n') {
        buffer.lines.push(glyphon::BufferLine::new(
            &text[start..index],
            glyphon::cosmic_text::LineEnding::Lf,
            attrs.clone(),
            glyphon::Shaping::Advanced,
        ));
        start = index + 1;
    }

    buffer.lines.push(glyphon::BufferLine::new(
        &text[start..],
        glyphon::cosmic_text::LineEnding::None,
        attrs,
        glyphon::Shaping::Advanced,
    ));

    buffer
}

fn cosmic_buffer_text(buffer: &glyphon::Buffer) -> String {
    let mut text = String::new();

    for line in &buffer.lines {
        text.push_str(line.text());
        text.push_str(line.ending().as_str());
    }

    normalize_multiline(&text)
}

pub(crate) fn glyph_affinity(affinity: TextAffinity) -> glyphon::cosmic_text::Affinity {
    match affinity {
        TextAffinity::Upstream => glyphon::cosmic_text::Affinity::Before,
        TextAffinity::Downstream => glyphon::cosmic_text::Affinity::After,
    }
}

pub(crate) fn text_affinity(affinity: glyphon::cosmic_text::Affinity) -> TextAffinity {
    match affinity {
        glyphon::cosmic_text::Affinity::Before => TextAffinity::Upstream,
        glyphon::cosmic_text::Affinity::After => TextAffinity::Downstream,
    }
}

#[allow(dead_code)]
fn cursor_for_text_position_in_buffer(buffer: &glyphon::Buffer, position: TextPosition) -> Cursor {
    let cursor = cursor_for_text_index_in_buffer(buffer, position.index);
    Cursor::new_with_affinity(cursor.line, cursor.index, glyph_affinity(position.affinity))
}

pub(crate) fn text_position_for_cursor_in_buffer(
    buffer: &glyphon::Buffer,
    cursor: Cursor,
) -> TextPosition {
    let cursor = clamp_cursor_in_buffer(buffer, cursor);
    TextPosition::with_affinity(
        text_index_for_cursor_in_buffer(buffer, cursor),
        text_affinity(cursor.affinity),
    )
}

#[allow(dead_code)]
pub(crate) fn selection_anchor(buffer: &glyphon::Buffer, selection: Selection) -> Option<Cursor> {
    match clamp_selection_in_buffer(buffer, selection) {
        Selection::None => None,
        Selection::Normal(cursor) | Selection::Line(cursor) | Selection::Word(cursor) => {
            Some(cursor)
        }
    }
}

#[allow(dead_code)]
pub(crate) fn fast_selection_bounds_in_buffer(
    buffer: &glyphon::Buffer,
    cursor: Cursor,
    selection: Selection,
) -> Option<(Cursor, Cursor)> {
    let cursor = clamp_cursor_in_buffer(buffer, cursor);
    match clamp_selection_in_buffer(buffer, selection) {
        Selection::None => None,
        Selection::Normal(select) => Some(ordered_cursors(select, cursor)),
        Selection::Line(select) => {
            let start_line = select.line.min(cursor.line);
            let end_line = select.line.max(cursor.line);
            let end_index = buffer.lines.get(end_line)?.text().len();
            Some((Cursor::new(start_line, 0), Cursor::new(end_line, end_index)))
        }
        Selection::Word(select) => {
            let (mut start, mut end) = ordered_cursors(select, cursor);

            if let Some(line) = buffer.lines.get(start.line) {
                start.index = line
                    .text()
                    .unicode_word_indices()
                    .rev()
                    .map(|(index, _)| index)
                    .find(|index| *index < start.index)
                    .unwrap_or(0);
            }

            if let Some(line) = buffer.lines.get(end.line) {
                end.index = line
                    .text()
                    .unicode_word_indices()
                    .map(|(index, word)| index + word.len())
                    .find(|index| *index > end.index)
                    .unwrap_or_else(|| line.text().len());
            }

            Some((start, end))
        }
    }
}

#[allow(dead_code)]
pub(crate) fn has_non_empty_selection_in_buffer(
    buffer: &glyphon::Buffer,
    cursor: Cursor,
    selection: Selection,
) -> bool {
    fast_selection_bounds_in_buffer(buffer, cursor, selection).is_some_and(|(start, end)| {
        text_index_for_cursor_in_buffer(buffer, start)
            < text_index_for_cursor_in_buffer(buffer, end)
    })
}

#[allow(dead_code)]
pub(crate) fn ordered_cursors(first: Cursor, second: Cursor) -> (Cursor, Cursor) {
    if first <= second {
        (first, second)
    } else {
        (second, first)
    }
}
pub(crate) fn cosmic_motion_for_text_motion(
    motion: TextMotion,
) -> Option<glyphon::cosmic_text::Motion> {
    Some(match motion {
        TextMotion::VisualLeft => glyphon::cosmic_text::Motion::Left,
        TextMotion::VisualRight => glyphon::cosmic_text::Motion::Right,
        TextMotion::VisualUp => glyphon::cosmic_text::Motion::Up,
        TextMotion::VisualDown => glyphon::cosmic_text::Motion::Down,
        TextMotion::PageUp => glyphon::cosmic_text::Motion::PageUp,
        TextMotion::PageDown => glyphon::cosmic_text::Motion::PageDown,
        TextMotion::LineStart => glyphon::cosmic_text::Motion::Home,
        TextMotion::LineEnd => glyphon::cosmic_text::Motion::End,
        _ => return None,
    })
}

pub(crate) fn text_position_for_motion_in_document(
    buffer: &Buffer,
    motion: TextMotion,
) -> Option<TextPosition> {
    let inner = buffer.inner.borrow();
    let index = inner
        .document
        .position_for_mark(inner.cursor)
        .unwrap_or_else(|| TextPosition::new(inner.document.text_len()))
        .index;
    let next = match motion {
        TextMotion::LogicalPrevious => inner.document.previous_grapheme_boundary_index(index),
        TextMotion::LogicalNext => inner.document.next_grapheme_boundary_index(index),
        TextMotion::WordPrevious => inner.document.previous_word_boundary_index(index),
        TextMotion::WordNext => inner.document.next_word_boundary_index(index),
        TextMotion::ParagraphStart => {
            let (line, _) = inner.document.line_and_local_for_index(index);
            inner.document.line_start(line)
        }
        TextMotion::ParagraphEnd => {
            let (line, _) = inner.document.line_and_local_for_index(index);
            inner.document.line_start(line) + inner.document.line_text_len(line)
        }
        TextMotion::DocumentStart => 0,
        TextMotion::DocumentEnd => inner.document.text_len(),
        _ => return None,
    };

    Some(TextPosition::new(next))
}
#[allow(dead_code)]
pub(crate) fn text_position_for_motion_in_buffer(
    buffer: &glyphon::Buffer,
    cursor: Cursor,
    motion: TextMotion,
) -> Option<TextPosition> {
    let text = cosmic_buffer_text(buffer);
    let index = text_index_for_cursor_in_buffer(buffer, cursor);
    let next = match motion {
        TextMotion::LogicalPrevious => previous_grapheme_boundary(&text, index),
        TextMotion::LogicalNext => next_grapheme_boundary(&text, index),
        TextMotion::WordPrevious => previous_word_boundary(&text, index),
        TextMotion::WordNext => next_word_boundary(&text, index),
        TextMotion::ParagraphStart => paragraph_start_boundary(&text, index),
        TextMotion::ParagraphEnd => paragraph_end_boundary(&text, index),
        TextMotion::DocumentStart => 0,
        TextMotion::DocumentEnd => text.len(),
        _ => return None,
    };

    Some(TextPosition::new(next))
}

#[allow(dead_code)]
pub(crate) fn word_selection_cursors(buffer: &glyphon::Buffer, index: usize) -> (Cursor, Cursor) {
    let text = cosmic_buffer_text(buffer);
    let range = word_range_at(&text, index);
    (
        cursor_for_text_index_in_buffer(buffer, range.start),
        cursor_for_text_index_in_buffer(buffer, range.end),
    )
}
#[allow(dead_code)]
pub(crate) fn normalized_range_in_buffer(
    buffer: &glyphon::Buffer,
    range: std::ops::Range<usize>,
) -> std::ops::Range<usize> {
    let text = cosmic_buffer_text(buffer);
    grapheme_range_in_text(&text, range)
}

#[allow(dead_code)]
pub(crate) fn floor_text_index_in_buffer(buffer: &glyphon::Buffer, index: usize) -> usize {
    let text = cosmic_buffer_text(buffer);
    floor_grapheme_boundary(&text, index)
}

#[allow(dead_code)]
fn line_start_offsets(text: &str) -> Vec<usize> {
    let mut starts = vec![0];

    for (index, character) in text.char_indices() {
        if character == '\n' {
            starts.push(index + 1);
        }
    }

    starts
}

#[cfg(test)]
pub(crate) fn line_start_offsets_for_buffer(buffer: &glyphon::Buffer) -> Vec<usize> {
    let mut starts = Vec::with_capacity(buffer.lines.len().max(1));
    let mut offset = 0;

    for line in &buffer.lines {
        starts.push(offset);
        offset += line.text().len() + line.ending().as_str().len();
    }

    if starts.is_empty() {
        starts.push(0);
    }

    starts
}

#[allow(dead_code)]
pub(crate) fn cursor_for_text_index(text: &str, index: usize) -> Cursor {
    let index = floor_grapheme_boundary(text, index);
    let starts = line_start_offsets(text);
    let line = starts
        .partition_point(|start| *start <= index)
        .saturating_sub(1);
    let line_start = starts.get(line).copied().unwrap_or(0);
    Cursor::new(line, index.saturating_sub(line_start))
}

pub(crate) fn buffer_text_len(buffer: &glyphon::Buffer) -> usize {
    buffer
        .lines
        .iter()
        .map(|line| line.text().len() + line.ending().as_str().len())
        .sum()
}

pub(crate) fn cursor_for_text_index_in_buffer(buffer: &glyphon::Buffer, index: usize) -> Cursor {
    let mut remaining = index.min(buffer_text_len(buffer));

    for (line_index, line) in buffer.lines.iter().enumerate() {
        let text = line.text();
        if remaining <= text.len() {
            return Cursor::new(line_index, floor_grapheme_boundary(text, remaining));
        }

        remaining -= text.len();
        let ending_len = line.ending().as_str().len();
        if remaining < ending_len {
            return Cursor::new(line_index, text.len());
        }
        remaining = remaining.saturating_sub(ending_len);
    }

    let line = buffer.lines.len().saturating_sub(1);
    Cursor::new(
        line,
        buffer
            .lines
            .get(line)
            .map(glyphon::BufferLine::text)
            .map(str::len)
            .unwrap_or(0),
    )
}

pub(crate) fn text_index_for_cursor_in_buffer(buffer: &glyphon::Buffer, cursor: Cursor) -> usize {
    let cursor = clamp_cursor_in_buffer(buffer, cursor);
    let mut index = 0;

    for (line_index, line) in buffer.lines.iter().enumerate() {
        if line_index == cursor.line {
            return index + floor_grapheme_boundary(line.text(), cursor.index);
        }

        index += line.text().len() + line.ending().as_str().len();
    }

    index
}

#[allow(dead_code)]
pub(crate) fn text_range_for_cursors(
    buffer: &glyphon::Buffer,
    start: Cursor,
    end: Cursor,
) -> String {
    let start = clamp_cursor_in_buffer(buffer, start);
    let end = clamp_cursor_in_buffer(buffer, end);

    if start.line == end.line {
        let Some(line) = buffer.lines.get(start.line) else {
            return String::new();
        };

        return line.text()[start.index..end.index].to_owned();
    }

    let mut text = String::new();

    if let Some(line) = buffer.lines.get(start.line) {
        text.push_str(&line.text()[start.index..]);
        text.push_str(line.ending().as_str());
    }

    for line_index in start.line + 1..end.line {
        if let Some(line) = buffer.lines.get(line_index) {
            text.push_str(line.text());
            text.push_str(line.ending().as_str());
        }
    }

    if let Some(line) = buffer.lines.get(end.line) {
        text.push_str(&line.text()[..end.index]);
    }

    text
}

pub(crate) fn clamp_cursor_in_buffer(buffer: &glyphon::Buffer, cursor: Cursor) -> Cursor {
    let line = cursor.line.min(buffer.lines.len().saturating_sub(1));
    let line_text = buffer
        .lines
        .get(line)
        .map(glyphon::BufferLine::text)
        .unwrap_or("");

    Cursor::new(line, floor_grapheme_boundary(line_text, cursor.index))
}

pub(crate) fn clamp_selection_in_buffer(
    buffer: &glyphon::Buffer,
    selection: Selection,
) -> Selection {
    match selection {
        Selection::None => Selection::None,
        Selection::Normal(cursor) => Selection::Normal(clamp_cursor_in_buffer(buffer, cursor)),
        Selection::Line(cursor) => Selection::Line(clamp_cursor_in_buffer(buffer, cursor)),
        Selection::Word(cursor) => Selection::Word(clamp_cursor_in_buffer(buffer, cursor)),
    }
}

pub(crate) fn collapsed_cursor_for_motion(
    motion: TextMotion,
    start: Cursor,
    end: Cursor,
) -> Cursor {
    match motion {
        TextMotion::VisualLeft
        | TextMotion::LogicalPrevious
        | TextMotion::WordPrevious
        | TextMotion::LineStart
        | TextMotion::ParagraphStart
        | TextMotion::DocumentStart => start,
        TextMotion::VisualRight
        | TextMotion::LogicalNext
        | TextMotion::WordNext
        | TextMotion::LineEnd
        | TextMotion::ParagraphEnd
        | TextMotion::DocumentEnd => end,
        _ => end,
    }
}

pub(crate) fn cursor_position(buffer: &glyphon::Buffer, cursor: Cursor) -> Option<(i32, i32)> {
    let mut buffer = buffer.clone();
    let mut editor = glyphon::Editor::new(&mut buffer);
    glyphon::Edit::set_cursor(&mut editor, cursor);

    glyphon::Edit::cursor_position(&editor)
}

impl TextPosition {
    pub fn new(index: usize) -> Self {
        Self::with_affinity(index, TextAffinity::Downstream)
    }
    pub fn with_affinity(index: usize, affinity: TextAffinity) -> Self {
        Self { index, affinity }
    }
}
impl From<usize> for TextPosition {
    fn from(value: usize) -> Self {
        Self::new(value)
    }
}
impl From<Cursor> for TextPosition {
    fn from(value: Cursor) -> Self {
        Self::with_affinity(value.index, text_affinity(value.affinity))
    }
}
impl PartialEq<std::ops::Range<usize>> for TextRange {
    fn eq(&self, other: &std::ops::Range<usize>) -> bool {
        self.start == other.start && self.end == other.end
    }
}
impl PartialEq<TextRange> for std::ops::Range<usize> {
    fn eq(&self, other: &TextRange) -> bool {
        self.start == other.start && self.end == other.end
    }
}
impl TextRange {
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
impl From<std::ops::Range<usize>> for TextRange {
    fn from(value: std::ops::Range<usize>) -> Self {
        Self::new(value.start, value.end)
    }
}
impl TextSelection {
    pub fn new(anchor: TextPosition, focus: TextPosition) -> Self {
        Self { anchor, focus }
    }
    pub fn range(self) -> TextRange {
        TextRange::new(self.anchor.index, self.focus.index)
    }
}
