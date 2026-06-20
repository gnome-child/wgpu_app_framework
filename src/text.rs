use std::cell::{Cell, RefCell};
use std::collections::{BTreeMap, HashMap, VecDeque};
use std::fmt;
use std::fs::File;
use std::io;
use std::num::NonZeroUsize;
use std::path::{Path, PathBuf};
use std::rc::Rc;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{Duration, Instant};

use crate::geometry::{area, point};
use crate::{paint, text_system};
use lru::LruCache;
use unicode_segmentation::UnicodeSegmentation;

pub mod buffer;
pub mod document;
pub mod edit;
pub mod layout;
pub mod projection;
pub mod unicode;
pub mod view;

pub use layout::CaretLayout;
pub use view::{RevealIntent, ScrollAnchor, Viewport, Visibility};

const MEASURE_CACHE_CAPACITY: usize = 2048;
const TEXT_AREA_LINE_DISPLAY_CACHE_CAPACITY: usize = 2048;
const TEXT_AREA_HEIGHT_INDEX_CACHE_CAPACITY: usize = 128;
const TEXT_AREA_HEIGHT_INDEX_BLOCK_LINES: usize = 128;
const TEXT_AREA_FRAME_MIN_OVERSCAN_LINES: usize = 16;
const TEXT_AREA_FRAME_MAX_LOGICAL_LINES: usize = 256;
const TEXT_LAYOUT_VISUAL_LINE_EPSILON: f32 = 0.5;
const DEFAULT_TEXT_FIELD_SIZE: f32 = 16.0;
const TEXT_FIELD_CARET_MARGIN: f32 = 5.0;
const TEXT_FIELD_CARET_BLINK_INTERVAL: Duration = Duration::from_millis(500);
const TYPING_UNDO_COALESCE_WINDOW: Duration = Duration::from_millis(1000);
static NEXT_BUFFER_ID: AtomicU64 = AtomicU64::new(1);
static NEXT_LINE_ID: AtomicU64 = AtomicU64::new(1);

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub(crate) struct HighlightStats {
    pub run_scans: usize,
    pub highlight_calls: usize,
    pub spans: usize,
    pub skips: usize,
    pub cache_hits: usize,
    pub cache_misses: usize,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct Diagnostics {
    pub text_area_metrics_layout_calls: usize,
    pub text_area_paint_layout_calls: usize,
    pub text_area_line_cache_hits: usize,
    pub text_area_line_cache_misses: usize,
    pub text_area_line_shape_calls: usize,
    pub text_area_shaped_logical_lines: usize,
    pub text_area_shaped_visual_lines: usize,
    pub text_area_visible_logical_lines: usize,
    pub text_area_hit_run_scans: usize,
    pub text_area_aggregate_buffer_fallbacks: usize,
    pub text_area_height_index_hits: usize,
    pub text_area_height_index_misses: usize,
    pub highlight_run_scans: usize,
    pub highlight_spans: usize,
    pub highlight_skips: usize,
}

impl Diagnostics {
    fn add_highlight_stats(&mut self, stats: HighlightStats) {
        self.highlight_run_scans += stats.run_scans;
        self.highlight_spans += stats.spans;
        self.highlight_skips += stats.skips;
    }
}

impl HighlightStats {
    fn record_run_scan(&mut self) {
        self.run_scans += 1;
    }

    fn record_span(&mut self) {
        self.spans += 1;
    }

    fn record_skip(&mut self) {
        self.skips += 1;
    }

    #[cfg(test)]
    fn add(&mut self, other: Self) {
        self.run_scans += other.run_scans;
        self.highlight_calls += other.highlight_calls;
        self.spans += other.spans;
        self.skips += other.skips;
        self.cache_hits += other.cache_hits;
        self.cache_misses += other.cache_misses;
    }
}

#[cfg(test)]
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
struct TextInteractionStats {
    text_area_shape_until_scroll_calls: usize,
    text_area_frame_cache_hits: usize,
    text_area_frame_cache_misses: usize,
    text_area_frame_shape_calls: usize,
    text_area_frame_shaped_logical_lines: usize,
    text_area_frame_shaped_visual_lines: usize,
    hit_run_scans: usize,
    aggregate_buffer_fallbacks: usize,
}
pub(crate) type Cursor = glyphon::Cursor;
type Selection = glyphon::cosmic_text::Selection;

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

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash)]
pub enum TextGravity {
    Upstream,
    #[default]
    Downstream,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct LineId(u64);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct TextAnchor {
    pub line_id: LineId,
    pub byte_offset: usize,
    pub affinity: TextAffinity,
    pub gravity: TextGravity,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct TextAnchorRange {
    pub start: TextAnchor,
    pub end: TextAnchor,
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

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash)]
pub enum TextDirection {
    #[default]
    Auto,
    Ltr,
    Rtl,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub(crate) enum ResolvedTextDirection {
    Ltr,
    Rtl,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Role {
    Body,
    Label,
    Control,
    Menu,
    Placeholder,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Document {
    blocks: Vec<Block>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Block {
    runs: Vec<Run>,
    align: Align,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Run {
    text: String,
    style: Style,
}

pub struct Engine {
    font_system: glyphon::FontSystem,
    cache: MeasureCache,
    text_area_line_displays: LruCache<TextAreaLineDisplayKey, CachedTextAreaLineDisplay>,
    text_area_height_indices: LruCache<TextAreaHeightKey, TextAreaHeightIndex>,
    diagnostics: Diagnostics,
    #[cfg(test)]
    highlight_stats: HighlightStats,
    #[cfg(test)]
    interaction_stats: TextInteractionStats,
    #[cfg(test)]
    uncached_measure_count: usize,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Measure {
    max: Option<area::Logical>,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Metrics {
    area: area::Logical,
    line_count: usize,
}

#[derive(Debug, Clone, PartialEq)]
pub struct TextFieldLayout {
    selection_spans: Vec<SelectionSpan>,
    preedit_underline_spans: Vec<SelectionSpan>,
    preedit_selection_spans: Vec<SelectionSpan>,
    caret: Option<Caret>,
    scroll_x: f32,
    scroll_y: f32,
    content_area: area::Logical,
}

pub struct TextAreaPaintLayout {
    layout: TextFieldLayout,
    surfaces: Vec<TextAreaSurface>,
}

#[derive(Clone)]
pub struct TextAreaSurface {
    x: f32,
    y: f32,
    width: f32,
    height: f32,
    buffer: Rc<RefCell<glyphon::Buffer>>,
    default_color: paint::Color,
}

impl fmt::Debug for TextAreaSurface {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("TextAreaSurface")
            .field("x", &self.x)
            .field("y", &self.y)
            .field("width", &self.width)
            .field("height", &self.height)
            .field("default_color", &self.default_color)
            .finish_non_exhaustive()
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SelectionSpan {
    x: f32,
    y: f32,
    width: f32,
    height: f32,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Caret {
    x: f32,
    y: f32,
    height: f32,
}

#[derive(Clone)]
pub struct Buffer {
    inner: Rc<RefCell<BufferInner>>,
    revision: u64,
}

#[derive(Debug)]
struct BufferInner {
    id: u64,
    revision: u64,
    document: TextDocument,
    cursor: TextAnchor,
    selection: Option<TextAnchorRange>,
    multiline: bool,
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
    revision: u64,
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

const TEXT_DOCUMENT_BLOCK_TARGET_LINES: usize = 128;
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
struct TextDocumentStatsSnapshot {
    full_materializations: usize,
    total_document_scans: usize,
    piece_tree_updates: usize,
    mapped_index_pages_scanned: usize,
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
struct TextDocument {
    #[allow(dead_code)]
    original: TextOriginal,
    add_buffer: String,
    tree: TextLineTree,
    revision: u64,
    stats: TextDocumentStats,
}

impl TextDocument {
    fn from_text(text: &str) -> Self {
        let original: Rc<str> = Rc::from(text);
        Self::from_source_text(
            text,
            TextOriginal::Owned(original),
            TextPieceSource::OriginalOwned,
            0,
        )
    }

    fn open_mapped(path: impl AsRef<Path>) -> io::Result<Self> {
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

    fn line_count(&self) -> usize {
        self.tree.line_count()
    }

    fn text_len(&self) -> usize {
        self.tree.text_len()
    }

    fn line_starts(&self) -> Rc<Vec<usize>> {
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

    fn line_start(&self, line: usize) -> usize {
        self.tree.line_start(line)
    }

    fn line_text_len(&self, line: usize) -> usize {
        self.tree
            .line(line)
            .map(|line| line.text.len())
            .unwrap_or(0)
    }

    #[allow(dead_code)]
    fn line_ending_len(&self, line: usize) -> usize {
        self.tree
            .line(line)
            .map(|line| line.ending.len())
            .unwrap_or(0)
    }

    fn text(&self) -> String {
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

    fn text_for_range(&self, range: std::ops::Range<usize>) -> String {
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

    fn text_for_line_range(&self, start: usize, end: usize) -> String {
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

    fn replace_range(
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
        let mut replacement = Self::lines_from_source(
            &replacement_text,
            end_ending,
            next_revision,
            TextPieceSource::Add,
            add_start,
        );
        if let (Some(id), Some(first)) = (old_start_id, replacement.first_mut()) {
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

    fn cursor_for_text_index(&self, index: usize) -> Cursor {
        let index = index.min(self.text_len());
        let (line, local) = self.line_and_local_for_index(index);
        let local = self
            .tree
            .line(line)
            .map(|line| line.floor_grapheme(local))
            .unwrap_or(0);
        Cursor::new(line, local)
    }

    fn text_index_for_cursor(&self, cursor: Cursor) -> usize {
        let line = cursor.line.min(self.line_count().saturating_sub(1));
        let local = self
            .tree
            .line(line)
            .map(|line| line.floor_grapheme(cursor.index))
            .unwrap_or(0);
        self.line_start(line) + local
    }

    fn anchor_for_position(&self, position: TextPosition) -> Option<TextAnchor> {
        let cursor = self.cursor_for_text_index(position.index);
        let line = self.tree.line(cursor.line)?;
        Some(TextAnchor {
            line_id: line.id,
            byte_offset: cursor.index,
            affinity: position.affinity,
            gravity: TextGravity::Downstream,
        })
    }

    fn anchor_range_for_selection(&self, selection: TextSelection) -> Option<TextAnchorRange> {
        Some(TextAnchorRange {
            start: self.anchor_for_position(selection.anchor)?,
            end: self.anchor_for_position(selection.focus)?,
        })
    }

    fn position_for_anchor(&self, anchor: TextAnchor) -> Option<TextPosition> {
        let (_, offset, line) = self.line_index_start_and_line_for_id(anchor.line_id)?;
        let local = line.floor_grapheme(anchor.byte_offset);
        Some(TextPosition::with_affinity(offset + local, anchor.affinity))
    }

    fn anchor_for_cursor(&self, cursor: Cursor) -> Option<TextAnchor> {
        let line_index = cursor.line.min(self.line_count().saturating_sub(1));
        let line = self.tree.line(line_index)?;
        Some(TextAnchor {
            line_id: line.id,
            byte_offset: line.floor_grapheme(cursor.index),
            affinity: text_affinity(cursor.affinity),
            gravity: TextGravity::Downstream,
        })
    }

    fn cursor_for_anchor(&self, anchor: TextAnchor) -> Option<Cursor> {
        let (line_index, _, line) = self.line_index_start_and_line_for_id(anchor.line_id)?;
        Some(Cursor::new_with_affinity(
            line_index,
            line.floor_grapheme(anchor.byte_offset),
            glyph_affinity(anchor.affinity),
        ))
    }

    fn selection_for_anchor_range(&self, range: TextAnchorRange) -> Option<TextSelection> {
        Some(TextSelection::new(
            self.position_for_anchor(range.start)?,
            self.position_for_anchor(range.end)?,
        ))
    }

    fn ordered_cursor_range_for_anchor_range(
        &self,
        range: TextAnchorRange,
    ) -> Option<(Cursor, Cursor)> {
        let start_position = self.position_for_anchor(range.start)?;
        let end_position = self.position_for_anchor(range.end)?;
        let start = self.cursor_for_anchor(range.start)?;
        let end = self.cursor_for_anchor(range.end)?;
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

    fn line_and_local_for_index(&self, index: usize) -> (usize, usize) {
        self.tree.line_and_local_for_index(index)
    }

    fn snap_range(&self, range: TextRange) -> std::ops::Range<usize> {
        let range = range.as_range();
        if range.is_empty() {
            let index = self.floor_grapheme_boundary(range.start);
            return index..index;
        }
        self.floor_grapheme_boundary(range.start)..self.ceil_grapheme_boundary(range.end)
    }

    fn floor_grapheme_boundary(&self, index: usize) -> usize {
        let (line_index, local) = self.line_and_local_for_index(index);
        let Some(line) = self.tree.line(line_index) else {
            return 0;
        };
        self.line_start(line_index) + line.floor_grapheme(local)
    }

    fn ceil_grapheme_boundary(&self, index: usize) -> usize {
        let (line_index, local) = self.line_and_local_for_index(index);
        let Some(line) = self.tree.line(line_index) else {
            return self.text_len();
        };
        self.line_start(line_index) + line.ceil_grapheme(local)
    }

    fn previous_grapheme_boundary_index(&self, index: usize) -> usize {
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

    fn next_grapheme_boundary_index(&self, index: usize) -> usize {
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

    fn previous_word_boundary_index(&self, index: usize) -> usize {
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

    fn next_word_boundary_index(&self, index: usize) -> usize {
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
    fn original_len(&self) -> usize {
        match &self.original {
            TextOriginal::Owned(text) => text.len(),
            TextOriginal::Mapped(mapped) => mapped.mmap.len(),
        }
    }

    #[cfg(test)]
    fn piece_source_lengths(&self) -> (usize, usize, usize) {
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
    fn reset_stats(&self) {
        self.stats.reset();
    }

    #[cfg(test)]
    fn stats(&self) -> TextDocumentStatsSnapshot {
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
#[derive(Debug, Clone, PartialEq)]
pub struct Field {
    buffer: Buffer,
    mode: FieldMode,
    obscuring: Obscuring,
    placeholder: Option<String>,
}
#[derive(Debug, Clone, PartialEq)]
pub struct Area {
    buffer: Buffer,
    mode: FieldMode,
    placeholder: Option<String>,
    wrap: AreaWrap,
}
#[derive(Debug, Clone, PartialEq)]
pub enum Surface {
    Field(Field),
    Area(Area),
}
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash)]
pub enum FieldMode {
    #[default]
    Editable,
    ReadOnly,
    Disabled,
}
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash)]
pub enum Obscuring {
    #[default]
    None,
    Dot,
}
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash)]
pub enum AreaWrap {
    None,
    #[default]
    WordOrGlyph,
}
#[derive(Debug, Clone, PartialEq)]
pub struct TextViewState {
    scroll_x: f32,
    scroll_y: f32,
    caret_epoch: Instant,
    preedit: Option<Preedit>,
    history: EditHistory,
    reveal_intent: RevealIntent,
    preferred_caret_x: Option<f32>,
}

pub type TextFieldState = TextViewState;
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Preedit {
    text: String,
    selection: Option<(usize, usize)>,
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Edit {
    Insert(String),
    ImeCommit(String),
    ReplaceRange {
        range: TextRange,
        text: String,
    },
    MoveRange {
        range: TextRange,
        to: TextPosition,
    },
    Backspace,
    Delete,
    InsertLineBreak,
    MovePosition(TextMotion),
    ExtendPosition(TextMotion),
    DeleteWordBackward,
    DeleteWordForward,
    SelectAll,
    SetPosition(TextPosition),
    Pointer {
        kind: PointerEditKind,
        position: TextPosition,
    },
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PointerEditKind {
    Click,
    DoubleClick,
    TripleClick,
    Drag,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Command {
    Copy,
    Cut,
    Paste,
    SelectAll,
    Undo,
    Redo,
}
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct CommandResult {
    pub text_changed: bool,
    pub selection_changed: bool,
    pub clipboard_changed: bool,
    pub unavailable: bool,
}
#[derive(Debug, Clone, Default)]
pub(crate) struct TextEditResult {
    pub text_changed: bool,
    pub selection_changed: bool,
    pub change: Option<TextChange>,
}
#[derive(Debug, Clone)]
pub(crate) struct TextCommandOutcome {
    pub result: CommandResult,
    pub change: Option<TextChange>,
}
#[derive(Debug, Clone)]
pub(crate) struct TextChange {
    before: BufferMarker,
    after: BufferMarker,
    transaction: TextTransaction,
}

#[derive(Debug, Clone, Default)]
pub(crate) struct TextTransaction {
    deltas: Vec<TextDelta>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct TextDelta {
    kind: TextEditKind,
    range: TextRange,
    deleted: String,
    inserted: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum TextEditKind {
    Insert,
    Delete,
    Replace,
    Move,
    ImeCommit,
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum HistoryKind {
    Typing(String),
    Boundary,
}
#[derive(Debug, Clone)]
struct HistoryEntry {
    before: BufferMarker,
    after: BufferMarker,
    transaction: TextTransaction,
    kind: HistoryKind,
    recorded_at: Instant,
}
#[derive(Debug, Clone, Default)]
struct EditHistory {
    undo: Vec<HistoryEntry>,
    redo: Vec<HistoryEntry>,
    current: Option<BufferMarker>,
}
struct FieldProjection {
    buffer: Buffer,
    source_boundaries: Option<Vec<usize>>,
}
struct PreeditProjection {
    buffer: Buffer,
    underline: Option<(Cursor, Cursor)>,
    selection: Option<(Cursor, Cursor)>,
}
#[derive(Clone)]
struct CachedTextAreaLineDisplay {
    buffer: Rc<RefCell<glyphon::Buffer>>,
    source_line: usize,
    source_start: usize,
    source_text_len: usize,
    height: f32,
    width: f32,
}
#[derive(Clone)]
struct TextAreaDisplaySegment {
    display: CachedTextAreaLineDisplay,
    y: f32,
}
#[derive(Debug, Clone, Default, PartialEq)]
struct HighlightSpans {
    selection: Vec<SelectionSpan>,
    preedit_underline: Vec<SelectionSpan>,
    preedit_selection: Vec<SelectionSpan>,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub(crate) struct AreaScrollKey {
    buffer_id: u64,
    revision: u64,
    style: StyleKey,
    viewport: BoundsKey,
    wrap: AreaWrap,
}
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct TextAreaLineDisplayKey {
    buffer_id: u64,
    revision: u64,
    style: StyleKey,
    width: u32,
    wrap: AreaWrap,
    direction: TextDirection,
    source_line: usize,
}
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct TextAreaHeightKey {
    buffer_id: u64,
    revision: u64,
    style: StyleKey,
    width: u32,
    wrap: AreaWrap,
    direction: TextDirection,
}
#[derive(Debug, Clone)]
struct TextAreaHeightIndex {
    line_count: usize,
    estimated_line_height: f32,
    measured: BTreeMap<usize, f32>,
    block_deltas: BTreeMap<usize, f32>,
    measured_delta: f32,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
struct StyleKey {
    size: u32,
    weight: Weight,
    direction: TextDirection,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ClipboardError {
    Unavailable,
}
pub type ClipboardResult<T> = Result<T, ClipboardError>;
pub trait Clipboard {
    fn read_text(&mut self) -> ClipboardResult<Option<String>>;
    fn write_text(&mut self, text: &str) -> ClipboardResult<()>;
}
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Style {
    size: f32,
    color: paint::Color,
    weight: Weight,
    direction: TextDirection,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Weight {
    Normal,
    Medium,
    Bold,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Align {
    Start,
    Center,
    End,
}
#[derive(Debug)]
struct MeasureCache {
    entries: HashMap<MeasureKey, Metrics>,
    order: VecDeque<MeasureKey>,
    capacity: usize,
}
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct MeasureKey {
    blocks: Vec<BlockKey>,
    max: Option<BoundsKey>,
}
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct BlockKey {
    align: Align,
    runs: Vec<RunKey>,
}
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct RunKey {
    text: String,
    size: u32,
    weight: Weight,
    direction: TextDirection,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
struct BoundsKey {
    width: u32,
    height: u32,
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
impl ResolvedTextDirection {
    pub(crate) fn is_rtl(self) -> bool {
        matches!(self, Self::Rtl)
    }
}
impl TextDirection {
    pub(crate) fn resolve_for_text(self, text: &str) -> ResolvedTextDirection {
        match self {
            Self::Ltr => ResolvedTextDirection::Ltr,
            Self::Rtl => ResolvedTextDirection::Rtl,
            Self::Auto => auto_text_direction(text),
        }
    }
}
impl CommandResult {
    pub fn buffer_changed(self) -> bool {
        self.text_changed || self.selection_changed
    }
    pub fn changed(self) -> bool {
        self.buffer_changed() || self.clipboard_changed
    }
}
impl TextEditResult {
    fn from_markers(
        before: BufferMarker,
        after: BufferMarker,
        transaction: TextTransaction,
    ) -> Self {
        let text_changed = !transaction.is_empty();
        let selection_changed =
            before.cursor != after.cursor || before.selection != after.selection;
        Self {
            text_changed,
            selection_changed,
            change: text_changed.then_some(TextChange {
                before,
                after,
                transaction,
            }),
        }
    }
    pub fn buffer_changed(&self) -> bool {
        self.text_changed || self.selection_changed
    }
}
impl TextTransaction {
    fn replace(range: TextRange, deleted: String, inserted: String, kind: TextEditKind) -> Self {
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

    fn is_empty(&self) -> bool {
        self.deltas.is_empty()
    }

    fn inverse(&self) -> Self {
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

    fn try_coalesce_typing(&mut self, next: &TextTransaction) -> bool {
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
    fn inserted_end(&self) -> usize {
        self.range.start + self.inserted.len()
    }
}
impl HistoryKind {
    fn typing_text(&self) -> Option<&str> {
        match self {
            Self::Typing(text) => Some(text),
            Self::Boundary => None,
        }
    }
}
impl AreaScrollKey {
    fn new(area_model: &Area, style: Style, viewport: area::Logical) -> Self {
        Self {
            buffer_id: area_model.buffer().id(),
            revision: area_model.buffer().revision(),
            style: StyleKey::new(style),
            viewport: BoundsKey::new(viewport),
            wrap: area_model.wrap(),
        }
    }
}
impl TextAreaLineDisplayKey {
    fn new(
        area_model: &Area,
        buffer: &Buffer,
        style: Style,
        width: f32,
        source_line: usize,
    ) -> Self {
        Self {
            buffer_id: buffer.id(),
            revision: buffer.revision(),
            style: StyleKey::new(style),
            width: finite_bits(width.max(0.0)),
            wrap: area_model.wrap(),
            direction: style.direction(),
            source_line,
        }
    }
}
impl TextAreaHeightKey {
    fn new(area_model: &Area, buffer: &Buffer, style: Style, width: f32) -> Self {
        Self {
            buffer_id: buffer.id(),
            revision: buffer.revision(),
            style: StyleKey::new(style),
            width: finite_bits(width.max(0.0)),
            wrap: area_model.wrap(),
            direction: style.direction(),
        }
    }
}
impl TextAreaHeightIndex {
    fn new(line_count: usize, estimated_line_height: f32) -> Self {
        Self {
            line_count: line_count.max(1),
            estimated_line_height: estimated_line_height.max(1.0),
            measured: BTreeMap::new(),
            block_deltas: BTreeMap::new(),
            measured_delta: 0.0,
        }
    }

    fn sync(&mut self, line_count: usize, estimated_line_height: f32) {
        let line_count = line_count.max(1);
        let estimated_line_height = estimated_line_height.max(1.0);
        if self.line_count != line_count
            || self.estimated_line_height.to_bits() != estimated_line_height.to_bits()
        {
            *self = Self::new(line_count, estimated_line_height);
        }
    }

    fn update_line(&mut self, line: usize, height: f32) {
        if line >= self.line_count {
            return;
        }
        let height = height.max(1.0);
        let old = self
            .measured
            .insert(line, height)
            .unwrap_or(self.estimated_line_height);
        let delta = height - old;
        if delta.abs() <= f32::EPSILON {
            return;
        }
        self.measured_delta += delta;
        let block = line / TEXT_AREA_HEIGHT_INDEX_BLOCK_LINES;
        let block_delta = self.block_deltas.entry(block).or_insert(0.0);
        *block_delta += delta;
        if block_delta.abs() <= f32::EPSILON {
            self.block_deltas.remove(&block);
        }
    }

    fn line_top(&self, line: usize) -> f32 {
        let line = line.min(self.line_count);
        line as f32 * self.estimated_line_height + self.measured_delta_before(line)
    }

    fn total_height(&self) -> f32 {
        (self.line_count as f32 * self.estimated_line_height + self.measured_delta).max(0.0)
    }

    fn line_at_y(&self, y: f32) -> usize {
        if self.line_count == 0 {
            return 0;
        }
        let y = y.max(0.0);
        let mut line = (y / self.estimated_line_height).floor() as usize;
        line = line.min(self.line_count.saturating_sub(1));
        while line + 1 < self.line_count && self.line_top(line + 1) <= y {
            line += 1;
        }
        while line > 0 && self.line_top(line) > y {
            line -= 1;
        }
        line
    }

    fn visible_line_count(&self, scroll_y: f32, viewport_height: f32) -> usize {
        let start = self.line_at_y(scroll_y);
        let limit = scroll_y.max(0.0) + viewport_height.max(self.estimated_line_height);
        let mut line = start;
        while line < self.line_count && self.line_top(line) <= limit {
            line += 1;
        }
        line.saturating_sub(start).max(1)
    }

    fn measured_delta_before(&self, line: usize) -> f32 {
        let block = line / TEXT_AREA_HEIGHT_INDEX_BLOCK_LINES;
        let block_start = block * TEXT_AREA_HEIGHT_INDEX_BLOCK_LINES;
        let block_delta = self
            .block_deltas
            .range(..block)
            .map(|(_, delta)| *delta)
            .sum::<f32>();
        let local_delta = self
            .measured
            .range(block_start..line)
            .map(|(_, height)| *height - self.estimated_line_height)
            .sum::<f32>();
        block_delta + local_delta
    }
}
impl StyleKey {
    fn new(style: Style) -> Self {
        Self {
            size: finite_bits(style.size().max(1.0)),
            weight: style.weight(),
            direction: style.direction(),
        }
    }
}

impl Document {
    pub fn new() -> Self {
        Self { blocks: Vec::new() }
    }
    pub fn plain(text: impl Into<String>) -> Self {
        Self {
            blocks: vec![Block::plain(text)],
        }
    }
    pub fn from_block(block: Block) -> Self {
        Self {
            blocks: vec![block],
        }
    }
    pub fn push_block(&mut self, block: Block) {
        self.blocks.push(block);
    }
    pub fn blocks(&self) -> &[Block] {
        &self.blocks
    }
    pub fn first_style(&self) -> Option<Style> {
        self.blocks
            .iter()
            .flat_map(Block::runs)
            .find(|run| !run.is_empty())
            .map(Run::style)
    }
    pub fn with_color(mut self, color: paint::Color) -> Self {
        for block in &mut self.blocks {
            for run in &mut block.runs {
                run.style = run.style.with_color(color);
            }
        }
        self
    }
    pub fn with_size(mut self, size: f32) -> Self {
        for block in &mut self.blocks {
            for run in &mut block.runs {
                run.style = run.style.with_size(size);
            }
        }
        self
    }
    pub fn is_empty(&self) -> bool {
        self.blocks.iter().all(Block::is_empty)
    }
}
impl Default for Document {
    fn default() -> Self {
        Self::new()
    }
}
impl From<String> for Document {
    fn from(value: String) -> Self {
        Self::plain(value)
    }
}
impl From<&str> for Document {
    fn from(value: &str) -> Self {
        Self::plain(value)
    }
}

impl Measure {
    pub fn unbounded() -> Self {
        Self { max: None }
    }
    pub fn bounded(max: area::Logical) -> Self {
        Self {
            max: Some(area::logical(max.width().max(0.0), max.height().max(0.0))),
        }
    }
    pub fn max(self) -> Option<area::Logical> {
        self.max
    }
}
impl Metrics {
    pub fn new(area: area::Logical, line_count: usize) -> Self {
        Self { area, line_count }
    }
    pub fn empty() -> Self {
        Self::new(area::logical(0.0, 0.0), 0)
    }
    pub fn area(self) -> area::Logical {
        self.area
    }
    pub fn width(self) -> f32 {
        self.area.width()
    }
    pub fn height(self) -> f32 {
        self.area.height()
    }
    pub fn line_count(self) -> usize {
        self.line_count
    }
}
impl TextFieldLayout {
    pub fn empty() -> Self {
        Self {
            selection_spans: Vec::new(),
            preedit_underline_spans: Vec::new(),
            preedit_selection_spans: Vec::new(),
            caret: None,
            scroll_x: 0.0,
            scroll_y: 0.0,
            content_area: area::logical(0.0, 0.0),
        }
    }
    pub fn selection_spans(&self) -> &[SelectionSpan] {
        &self.selection_spans
    }
    pub fn preedit_underline_spans(&self) -> &[SelectionSpan] {
        &self.preedit_underline_spans
    }
    pub fn preedit_selection_spans(&self) -> &[SelectionSpan] {
        &self.preedit_selection_spans
    }
    pub fn caret(&self) -> Option<Caret> {
        self.caret
    }
    pub fn scroll_x(&self) -> f32 {
        self.scroll_x
    }
    pub fn scroll_y(&self) -> f32 {
        self.scroll_y
    }
    pub fn content_area(&self) -> area::Logical {
        self.content_area
    }
}
impl TextAreaPaintLayout {
    pub fn layout(&self) -> &TextFieldLayout {
        &self.layout
    }
    pub fn surfaces(&self) -> &[TextAreaSurface] {
        &self.surfaces
    }
    pub fn into_parts(self) -> (TextFieldLayout, Vec<TextAreaSurface>) {
        (self.layout, self.surfaces)
    }
}

impl TextAreaSurface {
    pub fn x(&self) -> f32 {
        self.x
    }

    pub fn y(&self) -> f32 {
        self.y
    }

    pub fn width(&self) -> f32 {
        self.width
    }

    pub fn height(&self) -> f32 {
        self.height
    }

    pub fn buffer(&self) -> Rc<RefCell<glyphon::Buffer>> {
        self.buffer.clone()
    }

    pub fn default_color(&self) -> paint::Color {
        self.default_color
    }
}
impl SelectionSpan {
    pub fn new(x: f32, y: f32, width: f32, height: f32) -> Self {
        Self {
            x,
            y,
            width,
            height,
        }
    }
    pub fn x(self) -> f32 {
        self.x
    }
    pub fn y(self) -> f32 {
        self.y
    }
    pub fn width(self) -> f32 {
        self.width
    }
    pub fn height(self) -> f32 {
        self.height
    }
}
impl Caret {
    pub fn new(x: f32, y: f32, height: f32) -> Self {
        Self { x, y, height }
    }
    pub fn x(self) -> f32 {
        self.x
    }
    pub fn y(self) -> f32 {
        self.y
    }
    pub fn height(self) -> f32 {
        self.height
    }
}
impl Buffer {
    pub fn new() -> Self {
        Self::from_text("")
    }
    pub fn from_text(text: impl Into<String>) -> Self {
        Self::from_text_with_mode(text, false)
    }
    pub fn from_multiline_text(text: impl Into<String>) -> Self {
        Self::from_text_with_mode(text, true)
    }
    fn from_text_with_mode(text: impl Into<String>, multiline: bool) -> Self {
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
        let cursor = document_end_anchor(&document);
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
    fn reset_line_index_stats(&mut self) {
        self.inner.borrow().document.reset_stats();
    }

    #[cfg(test)]
    fn line_index_stats(&self) -> (usize, usize) {
        let stats = self.inner.borrow().document.stats();
        (stats.full_materializations, stats.piece_tree_updates)
    }

    #[cfg(test)]
    fn reset_document_stats(&self) {
        self.inner.borrow().document.reset_stats();
    }

    #[cfg(test)]
    fn document_stats(&self) -> TextDocumentStatsSnapshot {
        self.inner.borrow().document.stats()
    }

    #[cfg(test)]
    fn document_piece_source_lengths(&self) -> (usize, usize, usize) {
        self.inner.borrow().document.piece_source_lengths()
    }

    #[cfg(test)]
    #[cfg(test)]
    fn original_len(&self) -> usize {
        self.inner.borrow().document.original_len()
    }
    fn marker(&self) -> BufferMarker {
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
    pub fn logical_line_count(&self) -> usize {
        self.inner.borrow().document.line_count()
    }
    fn line_start_offsets(&self) -> Rc<Vec<usize>> {
        self.inner.borrow().document.line_starts()
    }
    pub fn position(&self) -> TextPosition {
        let inner = self.inner.borrow();
        inner
            .document
            .position_for_anchor(inner.cursor)
            .unwrap_or_else(|| TextPosition::new(inner.document.text_len()))
    }
    pub fn selection(&self) -> Option<TextSelection> {
        let inner = self.inner.borrow();
        inner
            .selection
            .and_then(|selection| inner.document.selection_for_anchor_range(selection))
    }
    pub fn anchor(&self) -> Option<TextAnchor> {
        Some(self.inner.borrow().cursor)
    }
    pub fn anchor_selection(&self) -> Option<TextAnchorRange> {
        self.inner.borrow().selection
    }
    pub fn position_for_anchor(&self, anchor: TextAnchor) -> Option<TextPosition> {
        self.inner.borrow().document.position_for_anchor(anchor)
    }
    pub fn cursor(&self) -> Cursor {
        let inner = self.inner.borrow();
        inner
            .document
            .cursor_for_anchor(inner.cursor)
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
            .ordered_cursor_range_for_anchor_range(selection)?;
        (inner.document.text_index_for_cursor(start) < inner.document.text_index_for_cursor(end))
            .then_some((start, end))
    }
    pub fn selected_range(&self) -> Option<TextRange> {
        let inner = self.inner.borrow();
        let selection = inner.selection?;
        let start = inner.document.position_for_anchor(selection.start)?.index;
        let end = inner.document.position_for_anchor(selection.end)?.index;
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
    fn cursor_for_text_index(&self, index: usize) -> Cursor {
        let inner = self.inner.borrow();
        inner.document.cursor_for_text_index(index)
    }
    fn text_index_for_cursor(&self, cursor: Cursor) -> usize {
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
        selection_anchor_for_document(&inner.document, selection)
            .and_then(|anchor| inner.document.cursor_for_anchor(anchor))
            .map(Selection::Normal)
            .unwrap_or(Selection::None)
    }
    fn cloned_cosmic_buffer(&self) -> glyphon::Buffer {
        cosmic_buffer_from_text(&self.text())
    }
    #[allow(dead_code)]
    fn clone_cosmic_buffer_with_attrs(&self, attrs: glyphon::AttrsList) -> glyphon::Buffer {
        let mut buffer = self.cloned_cosmic_buffer();
        for line in &mut buffer.lines {
            line.set_attrs_list(attrs.clone());
        }
        buffer
    }
    fn set_cursor_and_selection(&mut self, cursor: Cursor, selection: Selection) {
        let mut inner = self.inner.borrow_mut();
        let cursor = inner
            .document
            .anchor_for_cursor(cursor)
            .unwrap_or_else(|| document_end_anchor(&inner.document));
        let selection = selection_anchor_for_document(&inner.document, selection).map(|anchor| {
            TextAnchorRange {
                start: anchor,
                end: cursor,
            }
        });
        inner.cursor = cursor;
        inner.selection = selection;
    }
    #[allow(dead_code)]
    fn set_anchor_selection(&mut self, cursor: TextAnchor, selection: Option<TextAnchorRange>) {
        let mut inner = self.inner.borrow_mut();
        inner.cursor = if inner.document.position_for_anchor(cursor).is_some() {
            cursor
        } else {
            document_end_anchor(&inner.document)
        };
        inner.selection = selection.and_then(|range| {
            (inner.document.position_for_anchor(range.start).is_some()
                && inner.document.position_for_anchor(range.end).is_some())
            .then_some(range)
        });
    }
    fn replace_text_range(&mut self, range: TextRange, text: &str) -> TextTransaction {
        self.replace_text_range_with_kind(range, text, TextEditKind::Replace)
    }
    fn replace_text_range_with_kind(
        &mut self,
        range: TextRange,
        text: &str,
        kind: TextEditKind,
    ) -> TextTransaction {
        let inserted = normalize_for_mode(self.is_multiline(), text);
        let range = {
            let inner = self.inner.borrow();
            inner.document.snap_range(range)
        };
        if range.is_empty() && inserted.is_empty() {
            return TextTransaction::default();
        }

        let (range, deleted, inserted_text) = {
            let mut inner = self.inner.borrow_mut();
            let (range, removed, inserted, _, _, _) = inner
                .document
                .replace_range(TextRange::new(range.start, range.end), &inserted);
            inner.revision = inner.document.revision;
            self.revision = inner.revision;
            let cursor = inner
                .document
                .cursor_for_text_index(range.start + inserted.len());
            inner.cursor = inner
                .document
                .anchor_for_cursor(cursor)
                .unwrap_or_else(|| document_end_anchor(&inner.document));
            inner.selection = None;
            (range, removed, inserted)
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
        TextTransaction::replace(
            TextRange::new(range.start, range.end),
            deleted,
            inserted_text,
            delta_kind,
        )
    }
    fn move_text_range(&mut self, range: TextRange, to: TextPosition) -> TextTransaction {
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
    fn replace_all_text(&mut self, text: String) {
        let mut inner = self.inner.borrow_mut();
        inner.document = TextDocument::from_text(&text);
        inner.revision = inner.revision.saturating_add(1);
        inner.document.revision = inner.revision;
        self.revision = inner.revision;
        inner.cursor = document_end_anchor(&inner.document);
        inner.selection = None;
    }
    fn restore_marker(&mut self, marker: BufferMarker) {
        let mut inner = self.inner.borrow_mut();
        if inner.id == marker.buffer_id {
            inner.revision = marker.revision;
            self.revision = marker.revision;
            inner.cursor = marker.cursor_for(&inner.document);
            inner.selection = marker.selection_for(&inner.document);
        }
    }
    fn apply_transaction(&mut self, transaction: &TextTransaction) -> bool {
        for delta in &transaction.deltas {
            self.replace_text_range_with_kind(
                TextRange::new(delta.range.start, delta.range.end),
                &delta.inserted,
                delta.kind,
            );
        }
        true
    }
    fn text_for_line_range(&self, start: usize, end: usize) -> String {
        self.inner.borrow().document.text_for_line_range(start, end)
    }
}
impl Engine {
    pub fn new() -> Self {
        Self {
            font_system: text_system::font_system(),
            cache: MeasureCache::new(MEASURE_CACHE_CAPACITY),
            text_area_line_displays: text_area_line_display_cache(),
            text_area_height_indices: text_area_height_index_cache(),
            diagnostics: Diagnostics::default(),
            #[cfg(test)]
            highlight_stats: HighlightStats::default(),
            #[cfg(test)]
            interaction_stats: TextInteractionStats::default(),
            #[cfg(test)]
            uncached_measure_count: 0,
        }
    }
    pub fn measure(&mut self, document: &Document, measure: Measure) -> Metrics {
        if document.is_empty() {
            return Metrics::empty();
        }
        let key = MeasureKey::new(document, measure);
        if let Some(metrics) = self.cache.get(&key) {
            return metrics;
        }
        #[cfg(test)]
        {
            self.uncached_measure_count += 1;
        }
        let metrics = text_system::measure_document(&mut self.font_system, document, measure);
        self.cache.insert(key, metrics);
        metrics
    }
    pub fn apply_text_edit(&mut self, buffer: &mut Buffer, edit: Edit) -> bool {
        self.apply_text_edit_with_result(buffer, edit)
            .buffer_changed()
    }
    pub(crate) fn apply_text_edit_with_result(
        &mut self,
        buffer: &mut Buffer,
        edit: Edit,
    ) -> TextEditResult {
        let before = buffer.marker();
        let mut transaction = TextTransaction::default();
        match edit {
            Edit::Insert(text) => {
                let range = buffer
                    .selected_range()
                    .unwrap_or_else(|| TextRange::collapsed(buffer.position().index));
                transaction =
                    buffer.replace_text_range_with_kind(range, &text, TextEditKind::Insert);
            }
            Edit::ImeCommit(text) => {
                let range = buffer
                    .selected_range()
                    .unwrap_or_else(|| TextRange::collapsed(buffer.position().index));
                transaction =
                    buffer.replace_text_range_with_kind(range, &text, TextEditKind::ImeCommit);
            }
            Edit::ReplaceRange { range, text } => {
                transaction = buffer.replace_text_range(range, &text);
            }
            Edit::MoveRange { range, to } => transaction = buffer.move_text_range(range, to),
            Edit::Backspace => {
                if let Some(range) = buffer.selected_range() {
                    transaction =
                        buffer.replace_text_range_with_kind(range, "", TextEditKind::Delete);
                } else {
                    let end = buffer.position().index;
                    let start = buffer
                        .inner
                        .borrow()
                        .document
                        .previous_grapheme_boundary_index(end);
                    transaction = buffer.replace_text_range_with_kind(
                        TextRange::new(start, end),
                        "",
                        TextEditKind::Delete,
                    );
                }
            }
            Edit::Delete => {
                if let Some(range) = buffer.selected_range() {
                    transaction =
                        buffer.replace_text_range_with_kind(range, "", TextEditKind::Delete);
                } else {
                    let start = buffer.position().index;
                    let end = buffer
                        .inner
                        .borrow()
                        .document
                        .next_grapheme_boundary_index(start);
                    transaction = buffer.replace_text_range_with_kind(
                        TextRange::new(start, end),
                        "",
                        TextEditKind::Delete,
                    );
                }
            }
            Edit::InsertLineBreak => {
                if buffer.is_multiline() {
                    let range = buffer
                        .selected_range()
                        .unwrap_or_else(|| TextRange::collapsed(buffer.position().index));
                    transaction =
                        buffer.replace_text_range_with_kind(range, "\n", TextEditKind::Insert);
                }
            }
            Edit::DeleteWordBackward => {
                if let Some(range) = buffer.selected_range() {
                    transaction =
                        buffer.replace_text_range_with_kind(range, "", TextEditKind::Delete);
                } else {
                    let end = buffer.position().index;
                    let start = buffer
                        .inner
                        .borrow()
                        .document
                        .previous_word_boundary_index(end);
                    transaction = buffer.replace_text_range_with_kind(
                        TextRange::new(start, end),
                        "",
                        TextEditKind::Delete,
                    );
                }
            }
            Edit::DeleteWordForward => {
                if let Some(range) = buffer.selected_range() {
                    transaction =
                        buffer.replace_text_range_with_kind(range, "", TextEditKind::Delete);
                } else {
                    let start = buffer.position().index;
                    let end = buffer
                        .inner
                        .borrow()
                        .document
                        .next_word_boundary_index(start);
                    transaction = buffer.replace_text_range_with_kind(
                        TextRange::new(start, end),
                        "",
                        TextEditKind::Delete,
                    );
                }
            }
            Edit::MovePosition(motion) => self.move_position(buffer, motion, false),
            Edit::ExtendPosition(motion) => self.move_position(buffer, motion, true),
            Edit::SelectAll => {
                let end = buffer.len();
                let cursor = buffer.cursor_for_text_index(end);
                let selection = if end == 0 {
                    Selection::None
                } else {
                    Selection::Normal(buffer.cursor_for_text_index(0))
                };
                buffer.set_cursor_and_selection(cursor, selection);
            }
            Edit::SetPosition(position) => {
                let cursor = buffer.cursor_for_text_index(position.index);
                buffer.set_cursor_and_selection(
                    Cursor::new_with_affinity(
                        cursor.line,
                        cursor.index,
                        glyph_affinity(position.affinity),
                    ),
                    Selection::None,
                );
            }
            Edit::Pointer { kind, position } => {
                let cursor = buffer.cursor_for_text_index(position.index);
                match kind {
                    PointerEditKind::Click => {
                        buffer.set_cursor_and_selection(cursor, Selection::None)
                    }
                    PointerEditKind::DoubleClick => {
                        let line_text = buffer.text();
                        let range = word_range_at(&line_text, position.index);
                        buffer.set_cursor_and_selection(
                            buffer.cursor_for_text_index(range.end),
                            Selection::Normal(buffer.cursor_for_text_index(range.start)),
                        );
                    }
                    PointerEditKind::TripleClick => {
                        let end = buffer.len();
                        let cursor = buffer.cursor_for_text_index(end);
                        let selection = if end == 0 {
                            Selection::None
                        } else {
                            Selection::Normal(buffer.cursor_for_text_index(0))
                        };
                        buffer.set_cursor_and_selection(cursor, selection);
                    }
                    PointerEditKind::Drag => {
                        let anchor =
                            selection_anchor_from_buffer(buffer).unwrap_or_else(|| buffer.cursor());
                        buffer.set_cursor_and_selection(cursor, Selection::Normal(anchor));
                    }
                }
            }
        }
        if buffer.selected_range().is_none() {
            let cursor = buffer.cursor();
            buffer.set_cursor_and_selection(cursor, Selection::None);
        }
        let after = buffer.marker();
        let result = TextEditResult::from_markers(before, after, transaction);
        if result.text_changed {
            self.invalidate_text_area_surfaces_for(buffer);
        }
        result
    }
    fn move_position(&mut self, buffer: &mut Buffer, motion: TextMotion, extend: bool) {
        let anchor = if extend {
            selection_anchor_from_buffer(buffer).unwrap_or_else(|| buffer.cursor())
        } else {
            buffer.cursor()
        };
        if !extend && let Some((start, end)) = buffer.selection_bounds() {
            buffer.set_cursor_and_selection(
                collapsed_cursor_for_motion(motion, start, end),
                Selection::None,
            );
            return;
        }
        let next = self
            .motion_position(buffer, motion)
            .unwrap_or_else(|| buffer.position());
        let cursor = buffer.cursor_for_text_index(next.index);
        let cursor =
            Cursor::new_with_affinity(cursor.line, cursor.index, glyph_affinity(next.affinity));
        let selection = if extend {
            Selection::Normal(anchor)
        } else {
            Selection::None
        };
        buffer.set_cursor_and_selection(cursor, selection);
    }
    fn motion_position(&mut self, buffer: &Buffer, motion: TextMotion) -> Option<TextPosition> {
        if let Some(position) = text_position_for_motion_in_document(buffer, motion) {
            return Some(position);
        }
        let cosmic_motion = cosmic_motion_for_text_motion(motion)?;
        self.diagnostics.text_area_aggregate_buffer_fallbacks += 1;
        #[cfg(test)]
        {
            self.interaction_stats.aggregate_buffer_fallbacks += 1;
        }
        let mut prepared = buffer.cloned_cosmic_buffer();
        prepared.set_wrap(&mut self.font_system, glyphon::Wrap::None);
        prepared.shape_until_scroll(&mut self.font_system, false);
        let mut editor = glyphon::Editor::new(&mut prepared);
        glyphon::Edit::set_cursor(&mut editor, buffer.cursor());
        glyphon::Edit::set_selection(&mut editor, Selection::None);
        glyphon::Edit::action(
            &mut editor,
            &mut self.font_system,
            glyphon::Action::Motion(cosmic_motion),
        );
        let cursor = glyphon::Edit::cursor(&editor);
        drop(editor);
        Some(text_position_for_cursor_in_buffer(&prepared, cursor))
    }
    pub(crate) fn invalidate_text_area_surfaces_for(&mut self, _buffer: &Buffer) {
        // Display and overlay cache keys include buffer revisions. Retaining unrelated
        // entries keeps scrolling warm after edits while stale revisions age out via LRU.
    }
    pub fn apply_text_command(
        &mut self,
        buffer: &mut Buffer,
        command: Command,
        clipboard: &mut dyn Clipboard,
    ) -> CommandResult {
        self.apply_text_command_with_result(buffer, command, clipboard)
            .result
    }
    pub(crate) fn apply_text_command_with_result(
        &mut self,
        buffer: &mut Buffer,
        command: Command,
        clipboard: &mut dyn Clipboard,
    ) -> TextCommandOutcome {
        let before = buffer.marker();
        let mut result = CommandResult::default();
        let mut change = None;
        match command {
            Command::Copy => {
                let Some(selection) = buffer.selected_text() else {
                    return TextCommandOutcome { result, change };
                };
                match clipboard.write_text(&selection) {
                    Ok(()) => result.clipboard_changed = true,
                    Err(_) => result.unavailable = true,
                }
            }
            Command::Cut => {
                let Some(selection) = buffer.selected_text() else {
                    return TextCommandOutcome { result, change };
                };
                match clipboard.write_text(&selection) {
                    Ok(()) => {
                        result.clipboard_changed = true;
                        change = self
                            .apply_text_edit_with_result(buffer, Edit::insert(""))
                            .change;
                    }
                    Err(_) => result.unavailable = true,
                }
            }
            Command::Paste => match clipboard.read_text() {
                Ok(Some(text)) if !normalize_for_buffer(buffer, &text).is_empty() => {
                    change = self
                        .apply_text_edit_with_result(buffer, Edit::insert(text))
                        .change
                }
                Ok(_) => {}
                Err(_) => result.unavailable = true,
            },
            Command::SelectAll => {
                change = self
                    .apply_text_edit_with_result(buffer, Edit::SelectAll)
                    .change;
            }
            Command::Undo | Command::Redo => result.unavailable = true,
        }
        let after = buffer.marker();
        result.text_changed = before.revision != after.revision;
        result.selection_changed =
            before.cursor != after.cursor || before.selection != after.selection;
        TextCommandOutcome { result, change }
    }
    pub fn text_field_layout(
        &mut self,
        buffer: &Buffer,
        style: Style,
        area: area::Logical,
        state: TextFieldState,
    ) -> TextFieldLayout {
        self.text_field_layout_at(buffer, style, area, state, Instant::now())
    }
    pub fn text_field_layout_for_field(
        &mut self,
        field: &Field,
        style: Style,
        area: area::Logical,
        state: TextFieldState,
    ) -> TextFieldLayout {
        self.text_field_layout_for_field_at(field, style, area, state, Instant::now())
    }
    pub fn text_field_layout_at(
        &mut self,
        buffer: &Buffer,
        style: Style,
        area: area::Logical,
        state: TextFieldState,
        now: Instant,
    ) -> TextFieldLayout {
        let projection = PreeditProjection::new(buffer, &state);
        let (prepared, vertical_offset) =
            self.prepare_text_field_buffer(&projection.buffer, style, area);
        let ranges = projection.highlight_ranges();
        let (spans, stats) = highlight_spans_for_ranges(
            &prepared,
            projection.buffer.selection_bounds(),
            ranges.0,
            ranges.1,
            vertical_offset,
            state.scroll_x(),
            0.0,
        );
        self.add_highlight_stats(stats);
        let caret = (!projection.buffer.has_non_empty_selection() && state.caret_visible(now))
            .then(|| {
                cursor_position(&prepared, projection.buffer.cursor()).map(|(x, y)| Caret {
                    x: x as f32 - state.scroll_x(),
                    y: vertical_offset + y as f32,
                    height: prepared.metrics().line_height,
                })
            })
            .flatten();
        TextFieldLayout {
            selection_spans: spans.selection,
            preedit_underline_spans: spans.preedit_underline,
            preedit_selection_spans: spans.preedit_selection,
            caret,
            scroll_x: state.scroll_x(),
            scroll_y: 0.0,
            content_area: buffer_content_area(&prepared),
        }
    }
    pub fn text_field_layout_for_field_at(
        &mut self,
        field: &Field,
        style: Style,
        area: area::Logical,
        state: TextFieldState,
        now: Instant,
    ) -> TextFieldLayout {
        let projection = FieldProjection::new(field);
        let state = projected_state_for_field(field, state);
        let mut layout = self.text_field_layout_at(&projection.buffer, style, area, state, now);
        if !field.paints_caret() {
            layout.caret = None;
        }
        layout
    }
    pub fn text_layout_for_surface_at(
        &mut self,
        surface: &Surface,
        style: Style,
        area: area::Logical,
        state: TextFieldState,
        now: Instant,
    ) -> TextFieldLayout {
        match surface {
            Surface::Field(field) => {
                self.text_field_layout_for_field_at(field, style, area, state, now)
            }
            Surface::Area(area_model) => {
                self.text_area_paint_layout_for_area_at(area_model, style, area, state, now)
                    .into_parts()
                    .0
            }
        }
    }
    pub fn text_field_position_at(
        &mut self,
        buffer: &Buffer,
        style: Style,
        area: area::Logical,
        position: point::Logical,
        state: TextFieldState,
    ) -> Option<TextPosition> {
        let projection = PreeditProjection::new(buffer, &state);
        let (prepared, vertical_offset) =
            self.prepare_text_field_buffer(&projection.buffer, style, area);
        TextLayoutMap::from_line_starts(projection.buffer.line_start_offsets()).hit_with_observer(
            &prepared,
            position.x() + state.scroll_x(),
            position.y() - vertical_offset,
            |_| {},
        )
    }
    pub fn text_field_position_at_for_field(
        &mut self,
        field: &Field,
        style: Style,
        area: area::Logical,
        position: point::Logical,
        state: TextFieldState,
    ) -> Option<TextPosition> {
        let projection = FieldProjection::new(field);
        let state = projected_state_for_field(field, state);
        let display =
            self.text_field_position_at(&projection.buffer, style, area, position, state)?;
        Some(projection.source_position(display))
    }
    pub fn text_position_at_for_surface(
        &mut self,
        surface: &Surface,
        style: Style,
        area: area::Logical,
        position: point::Logical,
        state: TextFieldState,
    ) -> Option<TextPosition> {
        match surface {
            Surface::Field(field) => {
                self.text_field_position_at_for_field(field, style, area, position, state)
            }
            Surface::Area(area_model) => {
                self.text_area_position_at_for_area(area_model, style, area, position, state)
            }
        }
    }
    pub fn text_field_caret(
        &mut self,
        buffer: &Buffer,
        style: Style,
        area: area::Logical,
        state: TextFieldState,
    ) -> Option<Caret> {
        self.text_field_layout_at(buffer, style, area, state, Instant::now())
            .caret()
    }
    pub fn text_field_caret_for_field(
        &mut self,
        field: &Field,
        style: Style,
        area: area::Logical,
        state: TextFieldState,
    ) -> Option<Caret> {
        if !field.paints_caret() {
            None
        } else {
            self.text_field_layout_for_field_at(field, style, area, state, Instant::now())
                .caret()
        }
    }
    pub fn text_area_caret_for_area(
        &mut self,
        area_model: &Area,
        style: Style,
        area: area::Logical,
        state: TextFieldState,
    ) -> Option<Caret> {
        self.text_area_paint_layout_for_area_at(area_model, style, area, state, Instant::now())
            .into_parts()
            .0
            .caret()
    }
    pub fn text_field_reveal_scroll(
        &mut self,
        buffer: &Buffer,
        style: Style,
        area: area::Logical,
        state: TextFieldState,
    ) -> TextFieldState {
        let projection = PreeditProjection::new(buffer, &state);
        let (prepared, _) = self.prepare_text_field_buffer(&projection.buffer, style, area);
        let width = area.width().max(0.0);
        let max_scroll = (buffer_content_area(&prepared).width() - width).max(0.0);
        let Some((caret_x, _)) = cursor_position(&prepared, projection.buffer.cursor()) else {
            return state
                .clone()
                .with_scroll_x(state.scroll_x().clamp(0.0, max_scroll));
        };
        let caret_x = caret_x as f32;
        let mut scroll_x = state.scroll_x().clamp(0.0, max_scroll);
        if caret_x > scroll_x + width - TEXT_FIELD_CARET_MARGIN {
            scroll_x = caret_x + TEXT_FIELD_CARET_MARGIN - width;
        } else if caret_x < scroll_x + TEXT_FIELD_CARET_MARGIN {
            scroll_x = caret_x - TEXT_FIELD_CARET_MARGIN;
        }
        state.with_scroll_x(scroll_x.clamp(0.0, max_scroll))
    }
    pub fn text_field_reveal_scroll_for_field(
        &mut self,
        field: &Field,
        style: Style,
        area: area::Logical,
        state: TextFieldState,
    ) -> TextFieldState {
        let projection = FieldProjection::new(field);
        self.text_field_reveal_scroll(
            &projection.buffer,
            style,
            area,
            projected_state_for_field(field, state),
        )
    }
    pub fn text_area_reveal_scroll_for_area(
        &mut self,
        area_model: &Area,
        style: Style,
        viewport: area::Logical,
        state: TextFieldState,
    ) -> TextFieldState {
        let projection = PreeditProjection::new(area_model.buffer(), &state);
        let source = &projection.buffer;
        let committed = !projection.has_preedit();

        if state.reveal_intent().if_needed() {
            let segments = self
                .text_area_display_segments(area_model, source, committed, style, viewport, &state);
            if let Some(caret_layout) = self.text_area_caret_layout_from_segments(
                area_model,
                &projection,
                &state,
                &segments,
            ) {
                let viewport_state =
                    Viewport::new(viewport, point::logical(state.scroll_x(), state.scroll_y()));
                if caret_layout
                    .visibility_in(viewport_state, TEXT_FIELD_CARET_MARGIN)
                    .is_visible()
                {
                    let content_height = self
                        .text_area_content_height(area_model, source, committed, style, viewport);
                    let max_scroll = (content_height - viewport.height()).max(0.0);
                    let scroll_x = state.scroll_x();
                    let scroll_y = state.scroll_y().clamp(0.0, max_scroll);
                    return state.with_scroll(scroll_x, scroll_y);
                }
            }
        }

        let line_count = source.logical_line_count().max(1);
        let cursor_line = source.cursor().line.min(line_count.saturating_sub(1));
        let estimated_line_height = text_area_estimated_line_height(style);
        let height_key = TextAreaHeightKey::new(area_model, source, style, viewport.width());
        let mut height_index = if committed {
            self.text_area_height_indices
                .pop(&height_key)
                .unwrap_or_else(|| TextAreaHeightIndex::new(line_count, estimated_line_height))
        } else {
            TextAreaHeightIndex::new(line_count, estimated_line_height)
        };
        height_index.sync(line_count, estimated_line_height);

        let display = self.text_area_line_display(
            area_model,
            source,
            committed,
            style,
            viewport,
            cursor_line,
        );
        height_index.update_line(cursor_line, display.height.max(1.0));
        let caret_line_top = height_index.line_top(cursor_line);
        let content_height = height_index.total_height().max(viewport.height().max(0.0));
        if committed {
            self.text_area_height_indices.put(height_key, height_index);
        }

        let caret = {
            let buffer = display.buffer.borrow();
            cursor_position(&buffer, Cursor::new(0, source.cursor().index)).map(|(x, y)| {
                (
                    x as f32,
                    caret_line_top + y as f32,
                    buffer.metrics().line_height.max(1.0),
                )
            })
        };

        let max_scroll_x = (display.width.max(viewport.width()) - viewport.width()).max(0.0);
        let max_scroll = (content_height - viewport.height()).max(0.0);
        let mut scroll_x = state.scroll_x().clamp(0.0, max_scroll_x);
        let mut scroll_y = state.scroll_y().clamp(0.0, max_scroll);
        if let Some((caret_x, caret_y, caret_height)) = caret {
            if caret_x < scroll_x {
                scroll_x = caret_x;
            } else if caret_x + 1.0 > scroll_x + viewport.width() {
                scroll_x = caret_x + 1.0 - viewport.width();
            }

            if caret_y < scroll_y {
                scroll_y = caret_y;
            } else if caret_y + caret_height > scroll_y + viewport.height() {
                scroll_y = caret_y + caret_height - viewport.height();
            }
        }
        state.with_scroll(
            scroll_x.clamp(0.0, max_scroll_x),
            scroll_y.clamp(0.0, max_scroll),
        )
    }
    pub fn text_reveal_scroll_for_surface(
        &mut self,
        surface: &Surface,
        style: Style,
        area: area::Logical,
        state: TextFieldState,
    ) -> TextFieldState {
        match surface {
            Surface::Field(field) => {
                self.text_field_reveal_scroll_for_field(field, style, area, state)
            }
            Surface::Area(area_model) => {
                self.text_area_reveal_scroll_for_area(area_model, style, area, state)
            }
        }
    }
    fn prepare_text_field_buffer(
        &mut self,
        buffer: &Buffer,
        style: Style,
        area: area::Logical,
    ) -> (glyphon::Buffer, f32) {
        let font_size = style.size().max(1.0);
        let line_height = font_size * 1.25;
        let buffer_height = area.height().max(0.0).min(line_height);
        let vertical_offset = (area.height().max(0.0) - buffer_height).max(0.0) * 0.5;
        let attrs = text_system::attrs_for_style(style);
        let mut prepared = cosmic_buffer_from_text(&buffer.text_for_line_range(0, 1));
        for line in &mut prepared.lines {
            line.set_attrs_list(glyphon::AttrsList::new(&attrs));
        }
        prepared.set_wrap(&mut self.font_system, glyphon::Wrap::None);
        prepared.set_metrics_and_size(
            &mut self.font_system,
            glyphon::Metrics::relative(font_size, 1.25),
            Some(area.width().max(0.0)),
            Some(buffer_height),
        );
        prepared.shape_until_scroll(&mut self.font_system, false);
        (prepared, vertical_offset)
    }
    pub fn text_area_metrics_layout_for_area_at(
        &mut self,
        area_model: &Area,
        style: Style,
        viewport: area::Logical,
        state: TextFieldState,
        _now: Instant,
    ) -> TextFieldLayout {
        self.diagnostics.text_area_metrics_layout_calls += 1;
        let projection = PreeditProjection::new(area_model.buffer(), &state);
        let content_height = self.text_area_content_height(
            area_model,
            &projection.buffer,
            !projection.has_preedit(),
            style,
            viewport,
        );
        let content_area = area::logical(viewport.width().max(0.0), content_height);
        TextFieldLayout {
            selection_spans: Vec::new(),
            preedit_underline_spans: Vec::new(),
            preedit_selection_spans: Vec::new(),
            caret: None,
            scroll_x: state.scroll_x(),
            scroll_y: state.scroll_y(),
            content_area,
        }
    }
    pub fn text_area_paint_layout_for_area_at(
        &mut self,
        area_model: &Area,
        style: Style,
        viewport: area::Logical,
        state: TextFieldState,
        now: Instant,
    ) -> TextAreaPaintLayout {
        self.diagnostics.text_area_paint_layout_calls += 1;
        let projection = PreeditProjection::new(area_model.buffer(), &state);
        let committed = !projection.has_preedit();
        let segments = self.text_area_display_segments(
            area_model,
            &projection.buffer,
            committed,
            style,
            viewport,
            &state,
        );
        let layout = self.text_area_layout_from_segments(
            area_model,
            style,
            viewport,
            &state,
            now,
            &projection,
            &segments,
        );
        let surfaces = segments
            .into_iter()
            .map(|segment| TextAreaSurface {
                x: -state.scroll_x(),
                y: segment.y,
                width: segment.display.width.max(viewport.width()) + state.scroll_x().max(0.0),
                height: segment.display.height.max(1.0),
                buffer: segment.display.buffer,
                default_color: style.color(),
            })
            .collect();
        TextAreaPaintLayout { layout, surfaces }
    }

    #[allow(dead_code)]
    fn text_area_layout_for_area_at(
        &mut self,
        area_model: &Area,
        style: Style,
        viewport: area::Logical,
        state: TextFieldState,
        now: Instant,
    ) -> TextFieldLayout {
        let projection = PreeditProjection::new(area_model.buffer(), &state);
        let committed = !projection.has_preedit();
        let segments = self.text_area_display_segments(
            area_model,
            &projection.buffer,
            committed,
            style,
            viewport,
            &state,
        );
        self.text_area_layout_from_segments(
            area_model,
            style,
            viewport,
            &state,
            now,
            &projection,
            &segments,
        )
    }

    fn text_area_layout_from_segments(
        &mut self,
        area_model: &Area,
        style: Style,
        viewport: area::Logical,
        state: &TextFieldState,
        now: Instant,
        projection: &PreeditProjection,
        segments: &[TextAreaDisplaySegment],
    ) -> TextFieldLayout {
        let mut spans = HighlightSpans::default();
        #[allow(unused_mut)]
        let mut combined_stats = HighlightStats::default();
        let selection = projection.buffer.selection_bounds();
        let (preedit_underline, preedit_selection) = projection.highlight_ranges();
        let mut caret = None;
        let mut observed_width: f32 = 0.0;

        for segment in segments {
            observed_width = observed_width.max(segment.display.width);
            let buffer = segment.display.buffer.borrow();
            let selection = selection.and_then(|range| {
                local_cursor_range_for_source_line(
                    range,
                    segment.display.source_line,
                    segment.display.source_text_len,
                )
            });
            let preedit_underline = preedit_underline.and_then(|range| {
                local_cursor_range_for_source_line(
                    range,
                    segment.display.source_line,
                    segment.display.source_text_len,
                )
            });
            let preedit_selection = preedit_selection.and_then(|range| {
                local_cursor_range_for_source_line(
                    range,
                    segment.display.source_line,
                    segment.display.source_text_len,
                )
            });
            let (line_spans, stats) = highlight_spans_for_ranges(
                &buffer,
                selection,
                preedit_underline,
                preedit_selection,
                segment.y,
                state.scroll_x(),
                0.0,
            );
            spans.selection.extend(line_spans.selection);
            spans.preedit_underline.extend(line_spans.preedit_underline);
            spans.preedit_selection.extend(line_spans.preedit_selection);
            #[cfg(test)]
            combined_stats.add(stats);
            #[cfg(not(test))]
            let _ = stats;

            if caret.is_none()
                && !projection.buffer.has_non_empty_selection()
                && area_model.paints_caret()
                && state.caret_visible(now)
                && projection.buffer.cursor().line == segment.display.source_line
            {
                let cursor = Cursor::new(0, projection.buffer.cursor().index);
                caret = cursor_position(&buffer, cursor).map(|(x, y)| Caret {
                    x: x as f32 - state.scroll_x(),
                    y: segment.y + y as f32,
                    height: buffer.metrics().line_height,
                });
            }
        }

        self.add_highlight_stats(combined_stats);
        let content_area = area::logical(
            observed_width.max(viewport.width()),
            self.text_area_content_height(
                area_model,
                &projection.buffer,
                !projection.has_preedit(),
                style,
                viewport,
            ),
        );
        TextFieldLayout {
            selection_spans: spans.selection,
            preedit_underline_spans: spans.preedit_underline,
            preedit_selection_spans: spans.preedit_selection,
            caret,
            scroll_x: state.scroll_x(),
            scroll_y: state.scroll_y(),
            content_area,
        }
    }
    fn text_area_caret_layout_from_segments(
        &self,
        area_model: &Area,
        projection: &PreeditProjection,
        state: &TextFieldState,
        segments: &[TextAreaDisplaySegment],
    ) -> Option<CaretLayout> {
        if !area_model.paints_caret() {
            return None;
        }

        let source_cursor = projection.buffer.cursor();
        for segment in segments {
            if source_cursor.line != segment.display.source_line {
                continue;
            }

            let buffer = segment.display.buffer.borrow();
            let cursor = Cursor::new(0, source_cursor.index.min(segment.display.source_text_len));
            return cursor_position(&buffer, cursor).map(|(x, y)| {
                CaretLayout::new(Caret::new(
                    x as f32 - state.scroll_x(),
                    segment.y + y as f32,
                    buffer.metrics().line_height,
                ))
            });
        }

        None
    }
    fn text_area_content_height(
        &mut self,
        area_model: &Area,
        source: &Buffer,
        committed: bool,
        style: Style,
        viewport: area::Logical,
    ) -> f32 {
        let line_count = source.logical_line_count().max(1);
        let estimated_line_height = text_area_estimated_line_height(style);
        if committed {
            let key = TextAreaHeightKey::new(area_model, source, style, viewport.width());
            if let Some(index) = self.text_area_height_indices.get(&key) {
                self.diagnostics.text_area_height_index_hits += 1;
                return index.total_height().max(viewport.height().max(0.0));
            }
            self.diagnostics.text_area_height_index_misses += 1;
        }
        (line_count as f32 * estimated_line_height).max(viewport.height().max(0.0))
    }

    fn text_area_line_display(
        &mut self,
        area_model: &Area,
        source: &Buffer,
        committed: bool,
        style: Style,
        viewport: area::Logical,
        source_line: usize,
    ) -> CachedTextAreaLineDisplay {
        let key = TextAreaLineDisplayKey::new(
            area_model,
            source,
            style,
            viewport.width().max(0.0),
            source_line,
        );
        if committed && let Some(display) = self.text_area_line_displays.get(&key) {
            self.diagnostics.text_area_line_cache_hits += 1;
            #[cfg(test)]
            {
                self.interaction_stats.text_area_frame_cache_hits += 1;
            }
            return display.clone();
        }
        self.diagnostics.text_area_line_cache_misses += 1;
        #[cfg(test)]
        {
            self.interaction_stats.text_area_frame_cache_misses += 1;
        }
        let font_size = style.size().max(1.0);
        let metrics = glyphon::Metrics::relative(font_size, 1.25);
        let attrs = text_system::attrs_for_style(style);
        let text = source.text_for_line_range(source_line, source_line + 1);
        let mut buffer = cosmic_buffer_from_text(&text);
        buffer.set_wrap(&mut self.font_system, area_model.wrap().into());
        buffer.set_metrics_and_size(
            &mut self.font_system,
            metrics,
            Some(viewport.width().max(0.0)),
            None,
        );
        for line in &mut buffer.lines {
            line.set_attrs_list(glyphon::AttrsList::new(&attrs));
        }
        buffer.shape_until_scroll(&mut self.font_system, false);
        let content = buffer_content_area(&buffer);
        let visual_runs = buffer.layout_runs().count();
        self.diagnostics.text_area_line_shape_calls += 1;
        self.diagnostics.text_area_shaped_logical_lines += 1;
        self.diagnostics.text_area_shaped_visual_lines += visual_runs;
        #[cfg(test)]
        {
            self.interaction_stats.text_area_frame_shape_calls += 1;
            self.interaction_stats.text_area_frame_shaped_logical_lines += 1;
            self.interaction_stats.text_area_frame_shaped_visual_lines += visual_runs;
        }
        let inner = source.inner.borrow();
        let display = CachedTextAreaLineDisplay {
            buffer: Rc::new(RefCell::new(buffer)),
            source_line,
            source_start: inner.document.line_start(source_line),
            source_text_len: inner.document.line_text_len(source_line),
            height: content.height(),
            width: content.width(),
        };
        drop(inner);
        if committed {
            self.text_area_line_displays.put(key, display.clone());
        }
        display
    }
    fn text_area_display_segments(
        &mut self,
        area_model: &Area,
        source: &Buffer,
        committed: bool,
        style: Style,
        viewport: area::Logical,
        state: &TextFieldState,
    ) -> Vec<TextAreaDisplaySegment> {
        let estimated_line_height = text_area_estimated_line_height(style);
        let line_count = source.logical_line_count().max(1);
        let height_key = TextAreaHeightKey::new(area_model, source, style, viewport.width());
        let mut height_index = if committed {
            self.text_area_height_indices
                .pop(&height_key)
                .unwrap_or_else(|| TextAreaHeightIndex::new(line_count, estimated_line_height))
        } else {
            TextAreaHeightIndex::new(line_count, estimated_line_height)
        };
        height_index.sync(line_count, estimated_line_height);

        let scroll_y = state.scroll_y().max(0.0);
        let first_visible = height_index.line_at_y(scroll_y);
        let visible_lines = height_index.visible_line_count(scroll_y, viewport.height());
        let overscan = visible_lines.max(TEXT_AREA_FRAME_MIN_OVERSCAN_LINES);
        let source_line_start = first_visible
            .saturating_sub(overscan)
            .min(line_count.saturating_sub(1));
        let source_line_end = first_visible
            .saturating_add(visible_lines)
            .saturating_add(overscan)
            .saturating_add(1)
            .min(line_count)
            .min(source_line_start.saturating_add(TEXT_AREA_FRAME_MAX_LOGICAL_LINES))
            .max(source_line_start + 1)
            .min(line_count.max(1));

        let mut y = height_index.line_top(source_line_start) - scroll_y;
        let mut segments = Vec::with_capacity(source_line_end.saturating_sub(source_line_start));
        for line in source_line_start..source_line_end {
            let display =
                self.text_area_line_display(area_model, source, committed, style, viewport, line);
            let segment_y = y;
            let display_height = display.height.max(1.0);
            height_index.update_line(line, display_height);
            y += display_height;
            segments.push(TextAreaDisplaySegment {
                display,
                y: segment_y,
            });
        }
        self.diagnostics.text_area_visible_logical_lines += segments.len();

        if committed {
            self.text_area_height_indices.put(height_key, height_index);
        }

        segments
    }
    pub fn text_area_position_at_for_area(
        &mut self,
        area_model: &Area,
        style: Style,
        viewport: area::Logical,
        position: point::Logical,
        state: TextFieldState,
    ) -> Option<TextPosition> {
        let projection = PreeditProjection::new(area_model.buffer(), &state);
        let segments = self.text_area_display_segments(
            area_model,
            &projection.buffer,
            !projection.has_preedit(),
            style,
            viewport,
            &state,
        );
        if segments.is_empty() {
            return Some(TextPosition::new(0));
        }

        let mut nearest = None::<(f32, &TextAreaDisplaySegment)>;
        for segment in &segments {
            let top = segment.y;
            let bottom = segment.y + segment.display.height.max(1.0);
            if position.y() >= top && position.y() <= bottom {
                nearest = Some((0.0, segment));
                break;
            }
            let distance = if position.y() < top {
                top - position.y()
            } else {
                position.y() - bottom
            };
            if nearest
                .as_ref()
                .is_none_or(|(best_distance, _)| distance < *best_distance)
            {
                nearest = Some((distance, segment));
            }
        }

        let segment = nearest.map(|(_, segment)| segment)?;
        let buffer = segment.display.buffer.borrow();
        let map = TextLayoutMap::from_line_starts(Rc::new(vec![segment.display.source_start]));
        let local = map.hit_with_observer(
            &buffer,
            position.x() + state.scroll_x(),
            position.y() - segment.y,
            |runs| {
                self.diagnostics.text_area_hit_run_scans += runs;
                #[cfg(test)]
                {
                    self.interaction_stats.hit_run_scans += runs;
                }
            },
        )?;
        Some(TextPosition::with_affinity(
            local.index.min(projection.buffer.len()),
            local.affinity,
        ))
    }
    #[cfg(test)]
    pub fn uncached_measure_count(&self) -> usize {
        self.uncached_measure_count
    }
    #[cfg(test)]
    pub fn cache_len(&self) -> usize {
        self.cache.len()
    }
    #[cfg(test)]
    fn with_cache_capacity(capacity: usize) -> Self {
        Self {
            cache: MeasureCache::new(capacity),
            ..Self::new()
        }
    }
    #[cfg(test)]
    fn reset_highlight_stats(&mut self) {
        self.highlight_stats = HighlightStats::default();
    }
    #[cfg(test)]
    fn highlight_stats(&self) -> HighlightStats {
        self.highlight_stats
    }
    #[cfg(test)]
    fn reset_interaction_stats(&mut self) {
        self.interaction_stats = TextInteractionStats::default();
    }
    #[cfg(test)]
    fn interaction_stats(&self) -> TextInteractionStats {
        self.interaction_stats
    }
    pub fn diagnostics(&self) -> Diagnostics {
        self.diagnostics
    }
    pub fn reset_diagnostics(&mut self) {
        self.diagnostics = Diagnostics::default();
        #[cfg(test)]
        {
            self.highlight_stats = HighlightStats::default();
            self.interaction_stats = TextInteractionStats::default();
        }
    }
    fn add_highlight_stats(&mut self, stats: HighlightStats) {
        self.diagnostics.add_highlight_stats(stats);
        #[cfg(test)]
        self.highlight_stats.add(stats);
        #[cfg(not(test))]
        let _ = stats;
    }
}
impl Default for Engine {
    fn default() -> Self {
        Self::new()
    }
}
impl Field {
    pub fn new(buffer: impl Into<Buffer>) -> Self {
        Self {
            buffer: buffer.into(),
            mode: FieldMode::Editable,
            obscuring: Obscuring::None,
            placeholder: None,
        }
    }

    pub fn buffer(&self) -> &Buffer {
        &self.buffer
    }

    pub fn mode(&self) -> FieldMode {
        self.mode
    }

    pub fn obscuring(&self) -> Obscuring {
        self.obscuring
    }

    pub fn placeholder(&self) -> Option<&str> {
        self.placeholder.as_deref()
    }

    pub fn with_mode(mut self, mode: FieldMode) -> Self {
        self.mode = mode;
        self
    }

    pub fn read_only(self) -> Self {
        self.with_mode(FieldMode::ReadOnly)
    }

    pub fn disabled(self) -> Self {
        self.with_mode(FieldMode::Disabled)
    }

    pub fn with_obscuring(mut self, obscuring: Obscuring) -> Self {
        self.obscuring = obscuring;
        self
    }

    pub fn obscured_dot(self) -> Self {
        self.with_obscuring(Obscuring::Dot)
    }

    pub fn with_placeholder(mut self, placeholder: impl Into<String>) -> Self {
        self.placeholder = Some(placeholder.into());
        self
    }

    pub fn is_editable(&self) -> bool {
        self.mode == FieldMode::Editable
    }

    pub fn is_read_only(&self) -> bool {
        self.mode == FieldMode::ReadOnly
    }

    pub fn is_disabled(&self) -> bool {
        self.mode == FieldMode::Disabled
    }

    pub fn is_selectable(&self) -> bool {
        !self.is_disabled()
    }

    pub fn accepts_text_input(&self) -> bool {
        self.is_editable()
    }

    pub fn paints_caret(&self) -> bool {
        self.is_editable()
    }

    pub fn allows_text_mutation(&self) -> bool {
        self.is_editable()
    }

    pub fn allows_copy(&self) -> bool {
        self.is_selectable() && self.obscuring == Obscuring::None
    }

    pub fn allows_cut(&self) -> bool {
        self.is_editable() && self.obscuring == Obscuring::None
    }

    pub fn presentation_text(&self) -> String {
        match self.obscuring {
            Obscuring::None => self.buffer.text(),
            Obscuring::Dot => obscured_dot_text(&self.buffer.text()),
        }
    }

    pub fn presentation_text_for_state(&self, state: &TextFieldState) -> String {
        let source = self.presentation_text();
        let Some(preedit) = state.preedit() else {
            return source;
        };

        match self.obscuring {
            Obscuring::None => {
                let range = preedit_replacement_range(&self.buffer, &source);
                let preedit_text = normalize_for_buffer(&self.buffer, preedit.text());
                composed_presentation_text(&source, range, &preedit_text)
            }
            Obscuring::Dot => {
                let source_text = self.buffer.text();
                let source_boundaries = source_grapheme_boundaries(&source_text);
                let range = if let Some(range) = self.buffer.selected_range() {
                    display_index(&source_boundaries, range.start)
                        ..display_index(&source_boundaries, range.end)
                } else {
                    let index = self.buffer.text_index_for_cursor(self.buffer.cursor());
                    let index = display_index(&source_boundaries, index);
                    index..index
                };
                let preedit_text =
                    obscured_dot_text(&normalize_for_buffer(&self.buffer, preedit.text()));
                composed_presentation_text(&source, range, &preedit_text)
            }
        }
    }
}

impl Area {
    pub fn new(buffer: impl Into<Buffer>) -> Self {
        let mut buffer = buffer.into();
        if !buffer.is_multiline() {
            buffer = Buffer::from_multiline_text(buffer.text());
        }

        Self {
            buffer,
            mode: FieldMode::Editable,
            placeholder: None,
            wrap: AreaWrap::default(),
        }
    }

    pub fn buffer(&self) -> &Buffer {
        &self.buffer
    }

    pub fn mode(&self) -> FieldMode {
        self.mode
    }

    pub fn placeholder(&self) -> Option<&str> {
        self.placeholder.as_deref()
    }

    pub fn wrap(&self) -> AreaWrap {
        self.wrap
    }

    pub fn with_mode(mut self, mode: FieldMode) -> Self {
        self.mode = mode;
        self
    }

    pub fn read_only(self) -> Self {
        self.with_mode(FieldMode::ReadOnly)
    }

    pub fn disabled(self) -> Self {
        self.with_mode(FieldMode::Disabled)
    }

    pub fn with_placeholder(mut self, placeholder: impl Into<String>) -> Self {
        self.placeholder = Some(placeholder.into());
        self
    }

    pub fn with_wrap(mut self, wrap: AreaWrap) -> Self {
        self.wrap = wrap;
        self
    }

    pub fn no_wrap(self) -> Self {
        self.with_wrap(AreaWrap::None)
    }

    pub fn is_editable(&self) -> bool {
        self.mode == FieldMode::Editable
    }

    pub fn is_read_only(&self) -> bool {
        self.mode == FieldMode::ReadOnly
    }

    pub fn is_disabled(&self) -> bool {
        self.mode == FieldMode::Disabled
    }

    pub fn is_selectable(&self) -> bool {
        !self.is_disabled()
    }

    pub fn accepts_text_input(&self) -> bool {
        self.is_editable()
    }

    pub fn paints_caret(&self) -> bool {
        self.is_editable()
    }

    pub fn allows_text_mutation(&self) -> bool {
        self.is_editable()
    }

    pub fn allows_copy(&self) -> bool {
        self.is_selectable()
    }

    pub fn allows_cut(&self) -> bool {
        self.is_editable()
    }

    pub fn presentation_text(&self) -> String {
        self.buffer.text()
    }

    pub fn presentation_text_for_state(&self, state: &TextFieldState) -> String {
        let source = self.presentation_text();
        let Some(preedit) = state.preedit() else {
            return source;
        };

        let range = preedit_replacement_range(&self.buffer, &source);
        let preedit_text = normalize_for_buffer(&self.buffer, preedit.text());
        composed_presentation_text(&source, range, &preedit_text)
    }
}

impl From<Buffer> for Field {
    fn from(value: Buffer) -> Self {
        Self::new(value)
    }
}

impl From<String> for Field {
    fn from(value: String) -> Self {
        Self::new(Buffer::from(value))
    }
}

impl From<&str> for Field {
    fn from(value: &str) -> Self {
        Self::new(Buffer::from(value))
    }
}

impl From<Buffer> for Area {
    fn from(value: Buffer) -> Self {
        Self::new(value)
    }
}

impl From<String> for Area {
    fn from(value: String) -> Self {
        Self::new(Buffer::from_multiline_text(value))
    }
}

impl From<&str> for Area {
    fn from(value: &str) -> Self {
        Self::new(Buffer::from_multiline_text(value))
    }
}

impl Surface {
    pub fn buffer(&self) -> &Buffer {
        match self {
            Self::Field(field) => field.buffer(),
            Self::Area(area) => area.buffer(),
        }
    }

    pub fn is_field(&self) -> bool {
        matches!(self, Self::Field(_))
    }

    pub fn is_area(&self) -> bool {
        matches!(self, Self::Area(_))
    }

    pub fn as_field(&self) -> Option<&Field> {
        match self {
            Self::Field(field) => Some(field),
            Self::Area(_) => None,
        }
    }

    pub fn as_area(&self) -> Option<&Area> {
        match self {
            Self::Field(_) => None,
            Self::Area(area) => Some(area),
        }
    }

    pub fn placeholder(&self) -> Option<&str> {
        match self {
            Self::Field(field) => field.placeholder(),
            Self::Area(area) => area.placeholder(),
        }
    }

    pub fn is_editable(&self) -> bool {
        match self {
            Self::Field(field) => field.is_editable(),
            Self::Area(area) => area.is_editable(),
        }
    }

    pub fn is_read_only(&self) -> bool {
        match self {
            Self::Field(field) => field.is_read_only(),
            Self::Area(area) => area.is_read_only(),
        }
    }

    pub fn is_disabled(&self) -> bool {
        match self {
            Self::Field(field) => field.is_disabled(),
            Self::Area(area) => area.is_disabled(),
        }
    }

    pub fn is_selectable(&self) -> bool {
        match self {
            Self::Field(field) => field.is_selectable(),
            Self::Area(area) => area.is_selectable(),
        }
    }

    pub fn accepts_text_input(&self) -> bool {
        match self {
            Self::Field(field) => field.accepts_text_input(),
            Self::Area(area) => area.accepts_text_input(),
        }
    }

    pub fn paints_caret(&self) -> bool {
        match self {
            Self::Field(field) => field.paints_caret(),
            Self::Area(area) => area.paints_caret(),
        }
    }

    pub fn allows_text_mutation(&self) -> bool {
        match self {
            Self::Field(field) => field.allows_text_mutation(),
            Self::Area(area) => area.allows_text_mutation(),
        }
    }

    pub fn allows_copy(&self) -> bool {
        match self {
            Self::Field(field) => field.allows_copy(),
            Self::Area(area) => area.allows_copy(),
        }
    }

    pub fn allows_cut(&self) -> bool {
        match self {
            Self::Field(field) => field.allows_cut(),
            Self::Area(area) => area.allows_cut(),
        }
    }

    pub fn presentation_text(&self) -> String {
        match self {
            Self::Field(field) => field.presentation_text(),
            Self::Area(area) => area.presentation_text(),
        }
    }

    pub fn presentation_text_for_state(&self, state: &TextFieldState) -> String {
        match self {
            Self::Field(field) => field.presentation_text_for_state(state),
            Self::Area(area) => area.presentation_text_for_state(state),
        }
    }
}

impl From<Field> for Surface {
    fn from(value: Field) -> Self {
        Self::Field(value)
    }
}

impl From<Area> for Surface {
    fn from(value: Area) -> Self {
        Self::Area(value)
    }
}

impl From<AreaWrap> for glyphon::Wrap {
    fn from(value: AreaWrap) -> Self {
        match value {
            AreaWrap::None => glyphon::Wrap::None,
            AreaWrap::WordOrGlyph => glyphon::Wrap::WordOrGlyph,
        }
    }
}

impl TextViewState {
    pub fn new(scroll_x: f32) -> Self {
        Self::new_at(scroll_x, Instant::now())
    }

    pub fn new_at(scroll_x: f32, caret_epoch: Instant) -> Self {
        Self {
            scroll_x: scroll_x.max(0.0),
            scroll_y: 0.0,
            caret_epoch,
            preedit: None,
            history: EditHistory::default(),
            reveal_intent: RevealIntent::None,
            preferred_caret_x: None,
        }
    }

    pub fn scroll_x(&self) -> f32 {
        self.scroll_x
    }

    pub fn scroll_y(&self) -> f32 {
        self.scroll_y
    }

    pub fn with_scroll_x(mut self, scroll_x: f32) -> Self {
        self.scroll_x = scroll_x.max(0.0);
        self
    }

    pub fn with_scroll_y(mut self, scroll_y: f32) -> Self {
        self.scroll_y = scroll_y.max(0.0);
        self
    }

    pub fn with_scroll(mut self, scroll_x: f32, scroll_y: f32) -> Self {
        self.scroll_x = scroll_x.max(0.0);
        self.scroll_y = scroll_y.max(0.0);
        self
    }

    pub fn reset_caret_blink(mut self, now: Instant) -> Self {
        self.caret_epoch = now;
        self.reveal_intent = RevealIntent::CaretForce;
        self
    }

    pub(crate) fn reset_caret_blink_if_needed(mut self, now: Instant) -> Self {
        self.caret_epoch = now;
        self.reveal_intent = RevealIntent::CaretIfNeeded;
        self
    }

    pub(crate) fn reset_caret_blink_without_reveal(mut self, now: Instant) -> Self {
        self.caret_epoch = now;
        self.reveal_intent = RevealIntent::None;
        self
    }

    pub(crate) fn reveal_intent(&self) -> RevealIntent {
        self.reveal_intent
    }

    pub(crate) fn reveal_pending(&self) -> bool {
        self.reveal_intent.should_reveal()
    }

    pub(crate) fn clear_reveal_pending(mut self) -> Self {
        self.reveal_intent = RevealIntent::None;
        self
    }

    pub fn with_preedit(mut self, preedit: Option<Preedit>) -> Self {
        self.preedit = preedit;
        self
    }

    pub fn preedit(&self) -> Option<&Preedit> {
        self.preedit.as_ref()
    }

    pub(crate) fn sync_history(&mut self, buffer: &Buffer) -> bool {
        self.history.sync(buffer.marker())
    }

    pub(crate) fn record_history_at(
        &mut self,
        change: TextChange,
        kind: HistoryKind,
        now: Instant,
    ) {
        self.history.record(change, kind, now);
    }

    pub(crate) fn can_undo(&self) -> bool {
        self.history.can_undo()
    }

    pub(crate) fn can_redo(&self) -> bool {
        self.history.can_redo()
    }

    pub(crate) fn apply_undo(&mut self, buffer: &mut Buffer) -> CommandResult {
        self.history.undo(buffer)
    }

    pub(crate) fn apply_redo(&mut self, buffer: &mut Buffer) -> CommandResult {
        self.history.redo(buffer)
    }

    pub fn caret_visible(&self, now: Instant) -> bool {
        let elapsed = now.saturating_duration_since(self.caret_epoch);
        let interval = TEXT_FIELD_CARET_BLINK_INTERVAL.as_millis();

        if interval == 0 {
            return true;
        }

        (elapsed.as_millis() / interval) % 2 == 0
    }

    pub fn next_caret_deadline(&self, now: Instant) -> Instant {
        let elapsed = now.saturating_duration_since(self.caret_epoch);
        let interval_ms = TEXT_FIELD_CARET_BLINK_INTERVAL.as_millis();
        let remainder = elapsed.as_millis() % interval_ms;
        let wait_ms = if remainder == 0 {
            interval_ms
        } else {
            interval_ms - remainder
        };

        now.checked_add(Duration::from_millis(wait_ms.min(u64::MAX as u128) as u64))
            .unwrap_or(now)
    }
}

impl FieldProjection {
    fn new(field: &Field) -> Self {
        match field.obscuring {
            Obscuring::None => Self {
                buffer: field.buffer.clone(),
                source_boundaries: None,
            },
            Obscuring::Dot => {
                let source_text = field.buffer.text();
                let source_boundaries = source_grapheme_boundaries(&source_text);
                let mut buffer = Buffer::from_text(obscured_dot_text(&source_text));

                if let Some((start, end)) = field.buffer.selection_bounds() {
                    buffer.set_cursor_and_selection(
                        Self::display_cursor(&source_boundaries, end),
                        Selection::Normal(Self::display_cursor(&source_boundaries, start)),
                    );
                } else {
                    buffer.set_cursor_and_selection(
                        Self::display_cursor(&source_boundaries, field.buffer.cursor()),
                        Selection::None,
                    );
                }

                Self {
                    buffer,
                    source_boundaries: Some(source_boundaries),
                }
            }
        }
    }

    fn source_position(&self, position: TextPosition) -> TextPosition {
        let Some(source_boundaries) = self.source_boundaries.as_ref() else {
            return position;
        };

        TextPosition::with_affinity(
            self.source_index(source_boundaries, position.index),
            position.affinity,
        )
    }

    fn display_cursor(source_boundaries: &[usize], cursor: Cursor) -> Cursor {
        Cursor::new(0, display_index(source_boundaries, cursor.index))
    }

    fn source_index(&self, source_boundaries: &[usize], display_index: usize) -> usize {
        let text = self.buffer.text();
        let display_index = floor_boundary(&text, display_index);
        let character = text[..display_index].chars().count();

        source_boundaries
            .get(character.min(source_boundaries.len().saturating_sub(1)))
            .copied()
            .unwrap_or(0)
    }
}

impl PreeditProjection {
    fn new(buffer: &Buffer, state: &TextFieldState) -> Self {
        let Some(preedit) = state.preedit() else {
            return Self::committed(buffer);
        };

        let source = buffer.text();
        let range = preedit_replacement_range(buffer, &source);
        let preedit_text = normalize_for_buffer(buffer, preedit.text());
        let preedit_start = range.start;
        let preedit_end = preedit_start + preedit_text.len();
        let mut text =
            String::with_capacity(source.len() - (range.end - range.start) + preedit_text.len());
        text.push_str(&source[..range.start]);
        text.push_str(&preedit_text);
        text.push_str(&source[range.end..]);

        let mut buffer = Buffer::from_text_with_mode(text, buffer.is_multiline());
        let selection_range = preedit
            .selection()
            .map(|(start, end)| preedit_selection_range(&preedit_text, start, end));
        let cursor_index = selection_range
            .as_ref()
            .map(|range| preedit_start + range.end)
            .unwrap_or(preedit_end);
        let cursor = buffer.cursor_for_text_index(cursor_index);
        buffer.set_cursor_and_selection(cursor, Selection::None);

        let underline = (preedit_start < preedit_end).then(|| {
            (
                buffer.cursor_for_text_index(preedit_start),
                buffer.cursor_for_text_index(preedit_end),
            )
        });
        let selection = selection_range.and_then(|range| {
            (range.start < range.end).then(|| {
                (
                    buffer.cursor_for_text_index(preedit_start + range.start),
                    buffer.cursor_for_text_index(preedit_start + range.end),
                )
            })
        });

        Self {
            buffer,
            underline,
            selection,
        }
    }

    fn committed(buffer: &Buffer) -> Self {
        Self {
            buffer: buffer.clone(),
            underline: None,
            selection: None,
        }
    }

    fn has_preedit(&self) -> bool {
        self.underline.is_some()
    }

    fn highlight_ranges(&self) -> (Option<(Cursor, Cursor)>, Option<(Cursor, Cursor)>) {
        (self.underline, self.selection)
    }
}

fn typing_history_kind(text: &str) -> HistoryKind {
    let mut graphemes = text.graphemes(true);
    let Some(first) = graphemes.next() else {
        return HistoryKind::Boundary;
    };
    if graphemes.next().is_some() {
        return HistoryKind::Boundary;
    }
    if first
        .chars()
        .any(|ch| ch.is_whitespace() || ch.is_ascii_punctuation())
    {
        return HistoryKind::Boundary;
    }
    HistoryKind::Typing(first.to_owned())
}

fn document_end_anchor(document: &TextDocument) -> TextAnchor {
    document
        .anchor_for_cursor(document.cursor_for_text_index(document.text_len()))
        .expect("text documents always contain at least one line")
}

fn selection_anchor_for_document(
    document: &TextDocument,
    selection: Selection,
) -> Option<TextAnchor> {
    match selection {
        Selection::None => None,
        Selection::Normal(cursor) | Selection::Line(cursor) | Selection::Word(cursor) => {
            document.anchor_for_cursor(cursor)
        }
    }
}

fn selection_anchor_from_buffer(buffer: &Buffer) -> Option<Cursor> {
    let inner = buffer.inner.borrow();
    inner
        .selection
        .and_then(|selection| inner.document.cursor_for_anchor(selection.start))
}

fn local_cursor_range_for_source_line(
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
fn preedit_replacement_range(buffer: &Buffer, source: &str) -> std::ops::Range<usize> {
    if let Some(range) = buffer.selected_range() {
        return grapheme_range_in_text(source, range.as_range());
    }

    let index = floor_grapheme_boundary(source, buffer.text_index_for_cursor(buffer.cursor()));
    index..index
}

fn preedit_selection_range(text: &str, start: usize, end: usize) -> std::ops::Range<usize> {
    if start == end {
        let index = floor_grapheme_boundary(text, start);
        return index..index;
    }

    grapheme_range_in_text(text, start.min(end)..start.max(end))
}

fn projected_state_for_field(field: &Field, state: TextFieldState) -> TextFieldState {
    if field.obscuring != Obscuring::Dot {
        return state;
    }

    let Some(preedit) = state.preedit().cloned() else {
        return state;
    };

    state.with_preedit(Some(obscured_preedit(&preedit)))
}

fn obscured_preedit(preedit: &Preedit) -> Preedit {
    let boundaries = source_grapheme_boundaries(preedit.text());
    let text = obscured_dot_text(preedit.text());
    let selection = preedit.selection().map(|(start, end)| {
        (
            display_index(&boundaries, start),
            display_index(&boundaries, end),
        )
    });

    Preedit::new(text, selection)
}

fn composed_presentation_text(
    source: &str,
    replace_range: std::ops::Range<usize>,
    preedit_text: &str,
) -> String {
    let mut text = String::with_capacity(
        source.len() - (replace_range.end - replace_range.start) + preedit_text.len(),
    );
    text.push_str(&source[..replace_range.start]);
    text.push_str(preedit_text);
    text.push_str(&source[replace_range.end..]);
    text
}

impl Preedit {
    pub fn new(text: impl Into<String>, selection: Option<(usize, usize)>) -> Self {
        Self {
            text: text.into(),
            selection,
        }
    }

    pub fn text(&self) -> &str {
        &self.text
    }

    pub fn selection(&self) -> Option<(usize, usize)> {
        self.selection
    }
}

fn obscured_dot_text(text: &str) -> String {
    "•".repeat(text.graphemes(true).count())
}

fn source_grapheme_boundaries(text: &str) -> Vec<usize> {
    let mut boundaries = vec![0];

    for (index, _) in text.grapheme_indices(true).skip(1) {
        boundaries.push(index);
    }

    boundaries.push(text.len());
    boundaries
}

fn display_index(source_boundaries: &[usize], source_index: usize) -> usize {
    let source_index = floor_boundary_for_boundaries(source_boundaries, source_index);
    let character = source_boundaries
        .partition_point(|boundary| *boundary <= source_index)
        .saturating_sub(1);

    "•".len() * character
}

fn floor_boundary_for_boundaries(boundaries: &[usize], index: usize) -> usize {
    boundaries
        .iter()
        .copied()
        .take_while(|boundary| *boundary <= index)
        .last()
        .unwrap_or(0)
}

impl Default for TextViewState {
    fn default() -> Self {
        Self::new(0.0)
    }
}

impl EditHistory {
    fn sync(&mut self, marker: BufferMarker) -> bool {
        if self.current.as_ref() == Some(&marker) {
            return false;
        }

        if self.current.as_ref().is_some_and(|current| {
            current.buffer_id == marker.buffer_id && current.revision == marker.revision
        }) {
            self.current = Some(marker);
            return false;
        }

        let changed = self.current.is_some() || !self.undo.is_empty() || !self.redo.is_empty();
        self.undo.clear();
        self.redo.clear();
        self.current = Some(marker);
        changed
    }

    fn record(&mut self, change: TextChange, kind: HistoryKind, now: Instant) {
        if change.before == change.after {
            self.current = Some(change.after);
            return;
        }

        if let Some(current) = self.current.as_ref()
            && current != &change.before
            && (current.buffer_id != change.before.buffer_id
                || current.revision != change.before.revision)
        {
            self.undo.clear();
            self.redo.clear();
        }

        if kind.typing_text().is_some()
            && let Some(last) = self.undo.last_mut()
            && last.kind.typing_text().is_some()
            && last.after == change.before
            && now.saturating_duration_since(last.recorded_at) <= TYPING_UNDO_COALESCE_WINDOW
            && last.transaction.try_coalesce_typing(&change.transaction)
        {
            last.after = change.after.clone();
            last.kind = kind;
            last.recorded_at = now;
            self.redo.clear();
            self.current = Some(change.after);
            return;
        }

        self.undo.push(HistoryEntry {
            before: change.before,
            after: change.after.clone(),
            transaction: change.transaction,
            kind,
            recorded_at: now,
        });
        self.redo.clear();
        self.current = Some(change.after);
    }

    fn can_undo(&self) -> bool {
        !self.undo.is_empty()
    }

    fn can_redo(&self) -> bool {
        !self.redo.is_empty()
    }

    fn undo(&mut self, buffer: &mut Buffer) -> CommandResult {
        let Some(entry) = self.undo.pop() else {
            return CommandResult {
                unavailable: true,
                ..CommandResult::default()
            };
        };

        let before = buffer.marker();
        let reverse = entry.transaction.inverse();

        if !buffer.apply_transaction(&reverse) {
            self.undo.push(entry);
            return CommandResult {
                unavailable: true,
                ..CommandResult::default()
            };
        }

        buffer.restore_marker(entry.before.clone());
        let after = buffer.marker();
        self.current = Some(after.clone());
        self.redo.push(entry);
        command_result_from_markers(before, after)
    }

    fn redo(&mut self, buffer: &mut Buffer) -> CommandResult {
        let Some(entry) = self.redo.pop() else {
            return CommandResult {
                unavailable: true,
                ..CommandResult::default()
            };
        };

        let before = buffer.marker();

        if !buffer.apply_transaction(&entry.transaction) {
            self.redo.push(entry);
            return CommandResult {
                unavailable: true,
                ..CommandResult::default()
            };
        }

        buffer.restore_marker(entry.after.clone());
        let after = buffer.marker();
        self.current = Some(after.clone());
        self.undo.push(entry);
        command_result_from_markers(before, after)
    }
}

fn command_result_from_markers(before: BufferMarker, after: BufferMarker) -> CommandResult {
    CommandResult {
        text_changed: before.revision != after.revision,
        selection_changed: before.cursor != after.cursor || before.selection != after.selection,
        clipboard_changed: false,
        unavailable: false,
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct BufferMarker {
    buffer_id: u64,
    revision: u64,
    cursor: TextAnchor,
    selection: Option<TextAnchorRange>,
    cursor_position: TextPosition,
    selection_positions: Option<TextSelection>,
}

impl BufferMarker {
    fn new(inner: &BufferInner) -> Self {
        Self {
            buffer_id: inner.id,
            revision: inner.revision,
            cursor: inner.cursor,
            selection: inner.selection,
            cursor_position: inner
                .document
                .position_for_anchor(inner.cursor)
                .unwrap_or_else(|| TextPosition::new(inner.document.text_len())),
            selection_positions: inner
                .selection
                .and_then(|selection| inner.document.selection_for_anchor_range(selection)),
        }
    }

    fn cursor_for(&self, document: &TextDocument) -> TextAnchor {
        if document.position_for_anchor(self.cursor).is_some() {
            self.cursor
        } else {
            document
                .anchor_for_position(self.cursor_position)
                .unwrap_or_else(|| document_end_anchor(document))
        }
    }

    fn selection_for(&self, document: &TextDocument) -> Option<TextAnchorRange> {
        if let Some(selection) = self.selection
            && document.position_for_anchor(selection.start).is_some()
            && document.position_for_anchor(selection.end).is_some()
        {
            return Some(selection);
        }
        self.selection_positions
            .and_then(|selection| document.anchor_range_for_selection(selection))
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

impl PartialEq for EditHistory {
    fn eq(&self, other: &Self) -> bool {
        self.current == other.current
            && self.undo.len() == other.undo.len()
            && self.redo.len() == other.redo.len()
            && self
                .undo
                .last()
                .map(|entry| (&entry.before, &entry.after, &entry.kind))
                == other
                    .undo
                    .last()
                    .map(|entry| (&entry.before, &entry.after, &entry.kind))
            && self
                .redo
                .last()
                .map(|entry| (&entry.before, &entry.after, &entry.kind))
                == other
                    .redo
                    .last()
                    .map(|entry| (&entry.before, &entry.after, &entry.kind))
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

impl Edit {
    pub fn insert(text: impl Into<String>) -> Self {
        Self::Insert(text.into())
    }

    pub fn ime_commit(text: impl Into<String>) -> Self {
        Self::ImeCommit(text.into())
    }

    pub fn replace_range(range: impl Into<TextRange>, text: impl Into<String>) -> Self {
        Self::ReplaceRange {
            range: range.into(),
            text: text.into(),
        }
    }

    pub fn insert_at(position: impl Into<TextPosition>, text: impl Into<String>) -> Self {
        let position = position.into();
        Self::replace_range(TextRange::collapsed(position.index), text)
    }

    pub fn move_range(range: impl Into<TextRange>, to: impl Into<TextPosition>) -> Self {
        Self::MoveRange {
            range: range.into(),
            to: to.into(),
        }
    }

    pub fn backspace() -> Self {
        Self::Backspace
    }

    pub fn delete() -> Self {
        Self::Delete
    }

    pub fn insert_line_break() -> Self {
        Self::InsertLineBreak
    }

    pub fn move_position(motion: TextMotion) -> Self {
        Self::MovePosition(motion)
    }

    pub fn extend_position(motion: TextMotion) -> Self {
        Self::ExtendPosition(motion)
    }

    #[cfg(test)]
    pub(crate) fn action(action: glyphon::Action) -> Self {
        match action {
            glyphon::Action::Backspace => Self::Backspace,
            glyphon::Action::Delete => Self::Delete,
            glyphon::Action::Enter => Self::InsertLineBreak,
            glyphon::Action::Insert(character) => Self::insert(character.to_string()),
            glyphon::Action::Motion(motion) => Self::motion(motion),
            _ => Self::MovePosition(TextMotion::LogicalNext),
        }
    }

    #[cfg(test)]
    pub(crate) fn motion(motion: glyphon::cosmic_text::Motion) -> Self {
        Self::MovePosition(text_motion_from_cosmic_motion(motion))
    }

    #[cfg(test)]
    pub(crate) fn extend_motion(motion: glyphon::cosmic_text::Motion) -> Self {
        Self::ExtendPosition(text_motion_from_cosmic_motion(motion))
    }

    #[cfg(test)]
    pub(crate) fn set_cursor(cursor: Cursor) -> Self {
        Self::SetPosition(cursor.into())
    }
    pub fn delete_word_backward() -> Self {
        Self::DeleteWordBackward
    }

    pub fn delete_word_forward() -> Self {
        Self::DeleteWordForward
    }

    pub fn set_position(position: impl Into<TextPosition>) -> Self {
        Self::SetPosition(position.into())
    }

    pub fn pointer(kind: PointerEditKind, position: impl Into<TextPosition>) -> Self {
        Self::Pointer {
            kind,
            position: position.into(),
        }
    }

    pub(crate) fn history_kind(&self) -> HistoryKind {
        match self {
            Self::Insert(text) => typing_history_kind(text),
            Self::ImeCommit(_)
            | Self::ReplaceRange { .. }
            | Self::MoveRange { .. }
            | Self::Backspace
            | Self::Delete
            | Self::InsertLineBreak
            | Self::DeleteWordBackward
            | Self::DeleteWordForward => HistoryKind::Boundary,
            Self::MovePosition(_)
            | Self::ExtendPosition(_)
            | Self::SelectAll
            | Self::SetPosition(_)
            | Self::Pointer { .. } => HistoryKind::Boundary,
        }
    }

    pub(crate) fn mutates_text(&self) -> bool {
        matches!(
            self,
            Self::Insert(_)
                | Self::ImeCommit(_)
                | Self::ReplaceRange { .. }
                | Self::MoveRange { .. }
                | Self::Backspace
                | Self::Delete
                | Self::InsertLineBreak
                | Self::DeleteWordBackward
                | Self::DeleteWordForward
        )
    }
}

impl Block {
    pub fn new(align: Align) -> Self {
        Self {
            runs: Vec::new(),
            align,
        }
    }

    pub fn plain(text: impl Into<String>) -> Self {
        Self {
            runs: vec![Run::new(text, Style::default())],
            align: Align::Start,
        }
    }

    pub fn push_run(&mut self, run: Run) {
        self.runs.push(run);
    }

    pub fn runs(&self) -> &[Run] {
        &self.runs
    }

    pub fn align(&self) -> Align {
        self.align
    }

    pub fn set_align(&mut self, align: Align) {
        self.align = align;
    }

    pub fn with_align(mut self, align: Align) -> Self {
        self.align = align;
        self
    }

    pub fn is_empty(&self) -> bool {
        self.runs.iter().all(Run::is_empty)
    }
}

impl Run {
    pub fn new(text: impl Into<String>, style: Style) -> Self {
        Self {
            text: text.into(),
            style,
        }
    }

    pub fn text(&self) -> &str {
        &self.text
    }

    pub fn style(&self) -> Style {
        self.style
    }

    pub fn is_empty(&self) -> bool {
        self.text.is_empty()
    }
}

impl Style {
    pub fn size(self) -> f32 {
        self.size
    }

    pub fn color(self) -> paint::Color {
        self.color
    }

    pub fn weight(self) -> Weight {
        self.weight
    }

    pub fn direction(self) -> TextDirection {
        self.direction
    }

    pub fn with_color(mut self, color: paint::Color) -> Self {
        self.color = color;
        self
    }

    pub fn with_size(mut self, size: f32) -> Self {
        self.size = size;
        self
    }

    pub fn with_weight(mut self, weight: Weight) -> Self {
        self.weight = weight;
        self
    }

    pub fn with_direction(mut self, direction: TextDirection) -> Self {
        self.direction = direction;
        self
    }
}

impl Default for Style {
    fn default() -> Self {
        Self {
            size: 16.0,
            color: paint::Color::rgb(0.92, 0.94, 0.98),
            weight: Weight::Normal,
            direction: TextDirection::Auto,
        }
    }
}

impl MeasureCache {
    fn new(capacity: usize) -> Self {
        Self {
            entries: HashMap::new(),
            order: VecDeque::new(),
            capacity,
        }
    }

    fn get(&self, key: &MeasureKey) -> Option<Metrics> {
        self.entries.get(key).copied()
    }

    fn insert(&mut self, key: MeasureKey, metrics: Metrics) {
        if self.capacity == 0 {
            return;
        }

        if let Some(entry) = self.entries.get_mut(&key) {
            *entry = metrics;
            return;
        }

        while self.entries.len() >= self.capacity {
            if let Some(oldest) = self.order.pop_front() {
                self.entries.remove(&oldest);
            } else {
                break;
            }
        }

        self.order.push_back(key.clone());
        self.entries.insert(key, metrics);
    }

    #[cfg(test)]
    fn len(&self) -> usize {
        self.entries.len()
    }
}

impl MeasureKey {
    fn new(document: &Document, measure: Measure) -> Self {
        Self {
            blocks: document
                .blocks()
                .iter()
                .filter(|block| !block.is_empty())
                .map(BlockKey::new)
                .collect(),
            max: measure.max().map(BoundsKey::new),
        }
    }
}

impl BlockKey {
    fn new(block: &Block) -> Self {
        Self {
            align: block.align(),
            runs: block.runs().iter().map(RunKey::new).collect(),
        }
    }
}

impl RunKey {
    fn new(run: &Run) -> Self {
        let style = run.style();

        Self {
            text: run.text().to_owned(),
            size: finite_bits(style.size().max(1.0)),
            weight: style.weight(),
            direction: style.direction(),
        }
    }
}

impl BoundsKey {
    fn new(bounds: area::Logical) -> Self {
        Self {
            width: finite_bits(bounds.width().max(0.0)),
            height: finite_bits(bounds.height().max(0.0)),
        }
    }
}

fn finite_bits(value: f32) -> u32 {
    if value.is_finite() {
        value.to_bits()
    } else if value.is_sign_negative() {
        0.0_f32.to_bits()
    } else {
        f32::INFINITY.to_bits()
    }
}

fn text_area_line_display_cache() -> LruCache<TextAreaLineDisplayKey, CachedTextAreaLineDisplay> {
    LruCache::new(
        NonZeroUsize::new(TEXT_AREA_LINE_DISPLAY_CACHE_CAPACITY)
            .expect("text area line display cache capacity must be non-zero"),
    )
}

fn text_area_height_index_cache() -> LruCache<TextAreaHeightKey, TextAreaHeightIndex> {
    LruCache::new(
        NonZeroUsize::new(TEXT_AREA_HEIGHT_INDEX_CACHE_CAPACITY)
            .expect("text area height index cache capacity must be non-zero"),
    )
}

fn text_area_estimated_line_height(style: Style) -> f32 {
    glyphon::Metrics::relative(style.size().max(1.0), 1.25)
        .line_height
        .max(1.0)
}

fn default_text_field_metrics() -> glyphon::Metrics {
    glyphon::Metrics::relative(DEFAULT_TEXT_FIELD_SIZE, 1.25)
}

fn normalize_single_line(text: &str) -> String {
    text.chars()
        .map(|character| match character {
            '\r' | '\n' => ' ',
            _ => character,
        })
        .collect()
}

fn normalize_multiline(text: &str) -> String {
    let text = text.replace("\r\n", "\n").replace("\n\r", "\n");
    text.replace('\r', "\n")
}

fn normalize_for_mode(multiline: bool, text: &str) -> String {
    if multiline {
        normalize_multiline(text)
    } else {
        normalize_single_line(text)
    }
}

fn normalize_for_buffer(buffer: &Buffer, text: &str) -> String {
    normalize_for_mode(buffer.is_multiline(), text)
}

fn cosmic_buffer_from_text(text: &str) -> glyphon::Buffer {
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

#[cfg(test)]
fn text_motion_from_cosmic_motion(motion: glyphon::cosmic_text::Motion) -> TextMotion {
    match motion {
        glyphon::cosmic_text::Motion::Left => TextMotion::VisualLeft,
        glyphon::cosmic_text::Motion::Right => TextMotion::VisualRight,
        glyphon::cosmic_text::Motion::Up => TextMotion::VisualUp,
        glyphon::cosmic_text::Motion::Down => TextMotion::VisualDown,
        glyphon::cosmic_text::Motion::PageUp => TextMotion::PageUp,
        glyphon::cosmic_text::Motion::PageDown => TextMotion::PageDown,
        glyphon::cosmic_text::Motion::Previous => TextMotion::LogicalPrevious,
        glyphon::cosmic_text::Motion::Next => TextMotion::LogicalNext,
        glyphon::cosmic_text::Motion::LeftWord | glyphon::cosmic_text::Motion::PreviousWord => {
            TextMotion::WordPrevious
        }
        glyphon::cosmic_text::Motion::RightWord | glyphon::cosmic_text::Motion::NextWord => {
            TextMotion::WordNext
        }
        glyphon::cosmic_text::Motion::Home | glyphon::cosmic_text::Motion::SoftHome => {
            TextMotion::LineStart
        }
        glyphon::cosmic_text::Motion::End => TextMotion::LineEnd,
        glyphon::cosmic_text::Motion::ParagraphStart => TextMotion::ParagraphStart,
        glyphon::cosmic_text::Motion::ParagraphEnd => TextMotion::ParagraphEnd,
        glyphon::cosmic_text::Motion::BufferStart => TextMotion::DocumentStart,
        glyphon::cosmic_text::Motion::BufferEnd => TextMotion::DocumentEnd,
        _ => TextMotion::LogicalNext,
    }
}
fn glyph_affinity(affinity: TextAffinity) -> glyphon::cosmic_text::Affinity {
    match affinity {
        TextAffinity::Upstream => glyphon::cosmic_text::Affinity::Before,
        TextAffinity::Downstream => glyphon::cosmic_text::Affinity::After,
    }
}

fn text_affinity(affinity: glyphon::cosmic_text::Affinity) -> TextAffinity {
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

fn text_position_for_cursor_in_buffer(buffer: &glyphon::Buffer, cursor: Cursor) -> TextPosition {
    let cursor = clamp_cursor_in_buffer(buffer, cursor);
    TextPosition::with_affinity(
        text_index_for_cursor_in_buffer(buffer, cursor),
        text_affinity(cursor.affinity),
    )
}

#[allow(dead_code)]
fn selection_anchor(buffer: &glyphon::Buffer, selection: Selection) -> Option<Cursor> {
    match clamp_selection_in_buffer(buffer, selection) {
        Selection::None => None,
        Selection::Normal(cursor) | Selection::Line(cursor) | Selection::Word(cursor) => {
            Some(cursor)
        }
    }
}

#[allow(dead_code)]
fn fast_selection_bounds_in_buffer(
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
fn has_non_empty_selection_in_buffer(
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
fn ordered_cursors(first: Cursor, second: Cursor) -> (Cursor, Cursor) {
    if first <= second {
        (first, second)
    } else {
        (second, first)
    }
}
fn auto_text_direction(text: &str) -> ResolvedTextDirection {
    unicode_bidi::BidiInfo::new(text, None)
        .paragraphs
        .first()
        .map(|paragraph| {
            if paragraph.level.is_rtl() {
                ResolvedTextDirection::Rtl
            } else {
                ResolvedTextDirection::Ltr
            }
        })
        .unwrap_or(ResolvedTextDirection::Ltr)
}

pub(crate) fn block_direction(block: &Block) -> ResolvedTextDirection {
    let text = block.runs().iter().map(Run::text).collect::<String>();
    block
        .runs()
        .iter()
        .find(|run| !run.is_empty())
        .map(|run| run.style().direction().resolve_for_text(&text))
        .unwrap_or(ResolvedTextDirection::Ltr)
}

fn cosmic_motion_for_text_motion(motion: TextMotion) -> Option<glyphon::cosmic_text::Motion> {
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

fn text_position_for_motion_in_document(
    buffer: &Buffer,
    motion: TextMotion,
) -> Option<TextPosition> {
    let inner = buffer.inner.borrow();
    let index = inner
        .document
        .position_for_anchor(inner.cursor)
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
fn text_position_for_motion_in_buffer(
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
fn word_selection_cursors(buffer: &glyphon::Buffer, index: usize) -> (Cursor, Cursor) {
    let text = cosmic_buffer_text(buffer);
    let range = word_range_at(&text, index);
    (
        cursor_for_text_index_in_buffer(buffer, range.start),
        cursor_for_text_index_in_buffer(buffer, range.end),
    )
}
#[allow(dead_code)]
fn normalized_range_in_buffer(
    buffer: &glyphon::Buffer,
    range: std::ops::Range<usize>,
) -> std::ops::Range<usize> {
    let text = cosmic_buffer_text(buffer);
    grapheme_range_in_text(&text, range)
}

#[allow(dead_code)]
fn floor_text_index_in_buffer(buffer: &glyphon::Buffer, index: usize) -> usize {
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
fn line_start_offsets_for_buffer(buffer: &glyphon::Buffer) -> Vec<usize> {
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
fn cursor_for_text_index(text: &str, index: usize) -> Cursor {
    let index = floor_grapheme_boundary(text, index);
    let starts = line_start_offsets(text);
    let line = starts
        .partition_point(|start| *start <= index)
        .saturating_sub(1);
    let line_start = starts.get(line).copied().unwrap_or(0);
    Cursor::new(line, index.saturating_sub(line_start))
}

fn buffer_text_len(buffer: &glyphon::Buffer) -> usize {
    buffer
        .lines
        .iter()
        .map(|line| line.text().len() + line.ending().as_str().len())
        .sum()
}

fn cursor_for_text_index_in_buffer(buffer: &glyphon::Buffer, index: usize) -> Cursor {
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

fn text_index_for_cursor_in_buffer(buffer: &glyphon::Buffer, cursor: Cursor) -> usize {
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
fn text_range_for_cursors(buffer: &glyphon::Buffer, start: Cursor, end: Cursor) -> String {
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

fn clamp_cursor_in_buffer(buffer: &glyphon::Buffer, cursor: Cursor) -> Cursor {
    let line = cursor.line.min(buffer.lines.len().saturating_sub(1));
    let line_text = buffer
        .lines
        .get(line)
        .map(glyphon::BufferLine::text)
        .unwrap_or("");

    Cursor::new(line, floor_grapheme_boundary(line_text, cursor.index))
}

fn clamp_selection_in_buffer(buffer: &glyphon::Buffer, selection: Selection) -> Selection {
    match selection {
        Selection::None => Selection::None,
        Selection::Normal(cursor) => Selection::Normal(clamp_cursor_in_buffer(buffer, cursor)),
        Selection::Line(cursor) => Selection::Line(clamp_cursor_in_buffer(buffer, cursor)),
        Selection::Word(cursor) => Selection::Word(clamp_cursor_in_buffer(buffer, cursor)),
    }
}

struct TextLayoutMap {
    line_starts: Rc<Vec<usize>>,
}

#[derive(Debug, Clone, Copy, PartialEq)]
struct VisualLineGroup {
    start: usize,
    end: usize,
    top: f32,
    bottom: f32,
}

impl TextLayoutMap {
    #[cfg(test)]
    fn new(buffer: &glyphon::Buffer) -> Self {
        Self {
            line_starts: Rc::new(line_start_offsets_for_buffer(buffer)),
        }
    }

    fn from_line_starts(line_starts: Rc<Vec<usize>>) -> Self {
        Self { line_starts }
    }

    #[cfg(test)]
    fn hit(&self, buffer: &glyphon::Buffer, x: f32, y: f32) -> Option<TextPosition> {
        self.hit_with_observer(buffer, x, y, |_| {})
    }

    fn hit_with_observer(
        &self,
        buffer: &glyphon::Buffer,
        x: f32,
        y: f32,
        mut observe_run_scans: impl FnMut(usize),
    ) -> Option<TextPosition> {
        let runs: Vec<_> = buffer.layout_runs().collect();
        observe_run_scans(runs.len());

        if runs.is_empty() {
            return Some(TextPosition::new(buffer_text_len(buffer)));
        }

        let groups = Self::visual_line_groups(&runs);
        let mut nearest_line = None::<(f32, VisualLineGroup)>;
        for group in groups {
            if y >= group.top - TEXT_LAYOUT_VISUAL_LINE_EPSILON
                && y <= group.bottom + TEXT_LAYOUT_VISUAL_LINE_EPSILON
            {
                return self.hit_line(&runs[group.start..group.end], x);
            }

            let distance = if y < group.top {
                group.top - y
            } else {
                y - group.bottom
            };
            if nearest_line
                .as_ref()
                .is_none_or(|(best_distance, _)| distance < *best_distance)
            {
                nearest_line = Some((distance, group));
            }
        }

        nearest_line
            .and_then(|(_, group)| self.hit_line(&runs[group.start..group.end], x))
            .or_else(|| Some(TextPosition::new(buffer_text_len(buffer))))
    }

    fn visual_line_groups(runs: &[glyphon::cosmic_text::LayoutRun<'_>]) -> Vec<VisualLineGroup> {
        let mut groups = Vec::new();
        for (index, run) in runs.iter().enumerate() {
            let top = run.line_top;
            let bottom = run.line_top + run.line_height;
            let same_visual_line = groups.last().is_some_and(|group: &VisualLineGroup| {
                let first = &runs[group.start];
                first.line_i == run.line_i
                    && (group.top - top).abs() <= TEXT_LAYOUT_VISUAL_LINE_EPSILON
                    && (group.bottom - bottom).abs() <= TEXT_LAYOUT_VISUAL_LINE_EPSILON
            });

            if same_visual_line {
                let group = groups.last_mut().expect("last group should exist");
                group.end = index + 1;
                group.top = group.top.min(top);
                group.bottom = group.bottom.max(bottom);
            } else {
                groups.push(VisualLineGroup {
                    start: index,
                    end: index + 1,
                    top,
                    bottom,
                });
            }
        }
        groups
    }

    #[allow(dead_code)]
    fn line_vertical_bounds(runs: &[glyphon::cosmic_text::LayoutRun<'_>]) -> (f32, f32) {
        let mut top = f32::INFINITY;
        let mut bottom = f32::NEG_INFINITY;

        for run in runs {
            top = top.min(run.line_top);
            bottom = bottom.max(run.line_top + run.line_height);
        }

        if top.is_finite() && bottom.is_finite() {
            (top, bottom)
        } else {
            (0.0, 0.0)
        }
    }

    fn hit_line(
        &self,
        runs: &[glyphon::cosmic_text::LayoutRun<'_>],
        x: f32,
    ) -> Option<TextPosition> {
        let mut nearest = None::<(f32, TextPosition)>;
        let fallback_line = runs
            .first()
            .and_then(|run| self.line_starts.get(run.line_i).copied())
            .unwrap_or(0);

        for run in runs {
            let Some((left, right)) = Self::run_visual_bounds(run) else {
                continue;
            };

            if x >= left
                && x <= right
                && let Some(position) = self.hit_run(run, x)
            {
                return Some(position);
            }

            for (edge_x, visual_left) in [(left, true), (right, false)] {
                let Some(position) = self.run_edge_position(run, visual_left) else {
                    continue;
                };
                let distance = (x - edge_x).abs();
                if nearest
                    .as_ref()
                    .is_none_or(|(best_distance, _)| distance < *best_distance)
                {
                    nearest = Some((distance, position));
                }
            }
        }

        nearest
            .map(|(_, position)| position)
            .or_else(|| Some(TextPosition::new(fallback_line)))
    }

    fn hit_run(&self, run: &glyphon::cosmic_text::LayoutRun<'_>, x: f32) -> Option<TextPosition> {
        let line_start = self.line_starts.get(run.line_i).copied().unwrap_or(0);
        if run.glyphs.is_empty() {
            return Some(TextPosition::new(line_start));
        }

        let (left, right) = Self::run_visual_bounds(run)?;
        if x <= left {
            return self.run_edge_position(run, true);
        }
        if x >= right {
            return self.run_edge_position(run, false);
        }
        for glyph in run.glyphs {
            let left = glyph.x;
            let right = glyph.x + glyph.w;
            if x >= left && x <= right {
                return Some(self.visual_position(
                    run.rtl,
                    line_start,
                    glyph,
                    x <= left + glyph.w * 0.5,
                ));
            }
        }

        None
    }

    fn run_visual_bounds(run: &glyphon::cosmic_text::LayoutRun<'_>) -> Option<(f32, f32)> {
        let mut glyphs = run.glyphs.iter();
        let first = glyphs.next()?;
        let mut left = first.x;
        let mut right = first.x + first.w;

        for glyph in glyphs {
            left = left.min(glyph.x);
            right = right.max(glyph.x + glyph.w);
        }

        Some((left, right))
    }

    fn run_edge_position(
        &self,
        run: &glyphon::cosmic_text::LayoutRun<'_>,
        visual_left: bool,
    ) -> Option<TextPosition> {
        let line_start = self.line_starts.get(run.line_i).copied().unwrap_or(0);
        let mut glyph = run.glyphs.first()?;

        for candidate in run.glyphs {
            if visual_left {
                if candidate.x < glyph.x {
                    glyph = candidate;
                }
            } else if candidate.x + candidate.w > glyph.x + glyph.w {
                glyph = candidate;
            }
        }

        Some(self.visual_position(run.rtl, line_start, glyph, visual_left))
    }

    fn visual_position(
        &self,
        rtl: bool,
        line_start: usize,
        glyph: &glyphon::LayoutGlyph,
        visual_left: bool,
    ) -> TextPosition {
        let glyph_rtl = glyph.level.is_rtl();
        match (rtl || glyph_rtl, visual_left) {
            (false, true) => {
                TextPosition::with_affinity(line_start + glyph.start, TextAffinity::Downstream)
            }
            (false, false) => {
                TextPosition::with_affinity(line_start + glyph.end, TextAffinity::Upstream)
            }
            (true, true) => {
                TextPosition::with_affinity(line_start + glyph.end, TextAffinity::Upstream)
            }
            (true, false) => {
                TextPosition::with_affinity(line_start + glyph.start, TextAffinity::Downstream)
            }
        }
    }
}
fn highlight_spans_for_ranges(
    buffer: &glyphon::Buffer,
    selection: Option<(Cursor, Cursor)>,
    preedit_underline: Option<(Cursor, Cursor)>,
    preedit_selection: Option<(Cursor, Cursor)>,
    vertical_offset: f32,
    scroll_x: f32,
    scroll_y: f32,
) -> (HighlightSpans, HighlightStats) {
    let mut spans = HighlightSpans::default();
    let mut stats = HighlightStats::default();

    for run in buffer.layout_runs() {
        stats.record_run_scan();
        let ranges = [
            line_highlight_range_for_run(&run, selection),
            line_highlight_range_for_run(&run, preedit_underline),
            line_highlight_range_for_run(&run, preedit_selection),
        ];
        if ranges.iter().all(Option::is_none) {
            continue;
        }

        let mut bounds = [(f32::INFINITY, f32::NEG_INFINITY); 3];
        for glyph in run.glyphs {
            let glyph_start = glyph.start.min(glyph.end);
            let glyph_end = glyph.start.max(glyph.end);
            for (role, range) in ranges.iter().enumerate() {
                let Some((line_start, line_end)) = range else {
                    continue;
                };
                if glyph_end <= *line_start || glyph_start >= *line_end {
                    continue;
                }
                bounds[role].0 = bounds[role].0.min(glyph.x);
                bounds[role].1 = bounds[role].1.max(glyph.x + glyph.w);
            }
        }

        push_highlight_span_from_bounds(
            &mut spans.selection,
            &mut stats,
            ranges[0],
            bounds[0],
            &run,
            vertical_offset,
            scroll_x,
            scroll_y,
        );
        push_highlight_span_from_bounds(
            &mut spans.preedit_underline,
            &mut stats,
            ranges[1],
            bounds[1],
            &run,
            vertical_offset,
            scroll_x,
            scroll_y,
        );
        push_highlight_span_from_bounds(
            &mut spans.preedit_selection,
            &mut stats,
            ranges[2],
            bounds[2],
            &run,
            vertical_offset,
            scroll_x,
            scroll_y,
        );
    }

    (spans, stats)
}

fn line_highlight_range_for_run(
    run: &glyphon::cosmic_text::LayoutRun<'_>,
    range: Option<(Cursor, Cursor)>,
) -> Option<(usize, usize)> {
    let Some((start, end)) = range else {
        return None;
    };
    let (start, end) = if end < start {
        (end, start)
    } else {
        (start, end)
    };
    if run.line_i < start.line || run.line_i > end.line || run.glyphs.is_empty() {
        return None;
    }
    let line_start = if run.line_i == start.line {
        start.index
    } else {
        0
    };
    let line_end = if run.line_i == end.line {
        end.index
    } else {
        usize::MAX
    };
    (line_start < line_end).then_some((line_start, line_end))
}

fn push_highlight_span_from_bounds(
    spans: &mut Vec<SelectionSpan>,
    stats: &mut HighlightStats,
    range: Option<(usize, usize)>,
    bounds: (f32, f32),
    run: &glyphon::cosmic_text::LayoutRun<'_>,
    vertical_offset: f32,
    scroll_x: f32,
    scroll_y: f32,
) {
    if range.is_none() {
        return;
    }
    let (left, right) = bounds;
    if !left.is_finite() || !right.is_finite() || right <= left {
        stats.record_skip();
        return;
    }

    spans.push(SelectionSpan {
        x: left - scroll_x,
        y: vertical_offset + run.line_top - scroll_y,
        width: right - left,
        height: run.line_height,
    });
    stats.record_span();
}

fn buffer_content_area(buffer: &glyphon::Buffer) -> area::Logical {
    let mut width: f32 = 0.0;
    let mut height: f32 = 0.0;

    for run in buffer.layout_runs() {
        let run_width = run
            .glyphs
            .iter()
            .map(|glyph| glyph.x + glyph.w)
            .fold(0.0_f32, f32::max);
        width = width.max(run_width);
        height = height.max(run.line_top + run.line_height);
    }

    if height == 0.0 {
        height = buffer.metrics().line_height;
    }

    area::logical(width, height)
}

pub(crate) fn text_area_scroll_base_content_area(
    area_model: &Area,
    style: Style,
    viewport: area::Logical,
) -> (AreaScrollKey, area::Logical) {
    let key = AreaScrollKey::new(area_model, style, viewport);
    let line_height = text_area_estimated_line_height(style);
    let height = (area_model.buffer().logical_line_count() as f32 * line_height)
        .max(viewport.height().max(0.0));

    (key, area::logical(viewport.width().max(0.0), height))
}

pub(crate) fn stable_text_area_content_area(
    base: area::Logical,
    hint: Option<area::Logical>,
    observed: area::Logical,
    viewport: area::Logical,
) -> area::Logical {
    let hint = hint.unwrap_or(base);
    area::logical(
        viewport
            .width()
            .max(base.width())
            .max(hint.width())
            .max(observed.width()),
        viewport
            .height()
            .max(base.height())
            .max(hint.height())
            .max(observed.height()),
    )
}

#[allow(dead_code)]
fn selection_bounds(
    buffer: &glyphon::Buffer,
    cursor: Cursor,
    selection: Selection,
) -> Option<(Cursor, Cursor)> {
    let mut buffer = buffer.clone();
    let cursor = clamp_cursor_in_buffer(&buffer, cursor);
    let selection = clamp_selection_in_buffer(&buffer, selection);
    let mut editor = glyphon::Editor::new(&mut buffer);
    glyphon::Edit::set_cursor(&mut editor, cursor);
    glyphon::Edit::set_selection(&mut editor, selection);
    let bounds = glyphon::Edit::selection_bounds(&editor);

    drop(editor);

    bounds.filter(|(start, end)| {
        text_index_for_cursor_in_buffer(&buffer, *start)
            < text_index_for_cursor_in_buffer(&buffer, *end)
    })
}

fn collapsed_cursor_for_motion(motion: TextMotion, start: Cursor, end: Cursor) -> Cursor {
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

fn cursor_position(buffer: &glyphon::Buffer, cursor: Cursor) -> Option<(i32, i32)> {
    let mut buffer = buffer.clone();
    let mut editor = glyphon::Editor::new(&mut buffer);
    glyphon::Edit::set_cursor(&mut editor, cursor);

    glyphon::Edit::cursor_position(&editor)
}

fn grapheme_range_in_text(text: &str, range: std::ops::Range<usize>) -> std::ops::Range<usize> {
    let start = range.start.min(range.end).min(text.len());
    let end = range.start.max(range.end).min(text.len());

    if start == end {
        let index = floor_grapheme_boundary(text, start);
        return index..index;
    }

    floor_grapheme_boundary(text, start)..ceil_grapheme_boundary(text, end)
}

fn floor_grapheme_boundary(text: &str, index: usize) -> usize {
    let index = floor_boundary(text, index);
    floor_boundary_for_boundaries(&source_grapheme_boundaries(text), index)
}

fn ceil_grapheme_boundary(text: &str, index: usize) -> usize {
    let index = floor_boundary(text, index);
    let boundaries = source_grapheme_boundaries(text);
    boundaries
        .iter()
        .copied()
        .find(|boundary| *boundary >= index)
        .unwrap_or(text.len())
}

fn previous_grapheme_boundary(text: &str, index: usize) -> usize {
    let index = floor_boundary(text, index);
    source_grapheme_boundaries(text)
        .into_iter()
        .take_while(|boundary| *boundary < index)
        .last()
        .unwrap_or(0)
}

fn next_grapheme_boundary(text: &str, index: usize) -> usize {
    let index = floor_boundary(text, index);
    source_grapheme_boundaries(text)
        .into_iter()
        .find(|boundary| *boundary > index)
        .unwrap_or(text.len())
}

fn word_boundaries(text: &str) -> Vec<usize> {
    let mut boundaries = vec![0];
    for (index, word) in text.split_word_bound_indices() {
        boundaries.push(index);
        boundaries.push(index + word.len());
    }
    for (index, _) in unicode_linebreak::linebreaks(text) {
        boundaries.push(index);
    }
    boundaries.push(text.len());
    boundaries.sort_unstable();
    boundaries.dedup();
    boundaries
}

fn previous_word_boundary(text: &str, index: usize) -> usize {
    let index = floor_boundary(text, index);
    word_boundaries(text)
        .into_iter()
        .take_while(|boundary| *boundary < index)
        .last()
        .unwrap_or(0)
}

fn next_word_boundary(text: &str, index: usize) -> usize {
    let index = floor_boundary(text, index);
    word_boundaries(text)
        .into_iter()
        .find(|boundary| *boundary > index)
        .unwrap_or(text.len())
}

fn word_range_at(text: &str, index: usize) -> std::ops::Range<usize> {
    let index = floor_boundary(text, index);
    for (start, word) in text.unicode_word_indices() {
        let end = start + word.len();
        if start <= index && index <= end {
            return start..end;
        }
    }
    let start = previous_word_boundary(text, index);
    let end = next_word_boundary(text, index);
    start..end
}

#[allow(dead_code)]
fn paragraph_start_boundary(text: &str, index: usize) -> usize {
    let index = floor_boundary(text, index);
    text[..index].rfind('\n').map(|line| line + 1).unwrap_or(0)
}

#[allow(dead_code)]
fn paragraph_end_boundary(text: &str, index: usize) -> usize {
    let index = floor_boundary(text, index);
    text[index..]
        .find('\n')
        .map(|line| index + line)
        .unwrap_or(text.len())
}
fn floor_boundary(text: &str, index: usize) -> usize {
    let mut index = index.min(text.len());
    while index > 0 && !text.is_char_boundary(index) {
        index -= 1;
    }

    index
}

#[cfg(test)]
mod tests {
    use std::rc::Rc;
    use std::time::{Duration, Instant};

    use super::*;

    fn surface_line_text(surfaces: &[TextAreaSurface], line: usize) -> String {
        surfaces
            .get(line)
            .and_then(|surface| {
                let buffer = surface.buffer.borrow();
                buffer.lines.first().map(|line| line.text().to_owned())
            })
            .unwrap_or_default()
    }

    fn surface_visual_runs(surfaces: &[TextAreaSurface]) -> usize {
        surfaces
            .iter()
            .map(|surface| surface.buffer.borrow().layout_runs().count())
            .sum()
    }

    fn visual_group_source_range(
        runs: &[glyphon::cosmic_text::LayoutRun<'_>],
        group: VisualLineGroup,
        source_start: usize,
    ) -> Option<std::ops::Range<usize>> {
        let mut start = usize::MAX;
        let mut end = 0usize;
        for run in &runs[group.start..group.end] {
            for glyph in run.glyphs {
                start = start.min(source_start + glyph.start.min(glyph.end));
                end = end.max(source_start + glyph.start.max(glyph.end));
            }
        }
        (start < end).then_some(start..end)
    }
    #[derive(Debug, Default)]
    struct MockClipboard {
        text: Option<String>,
        unavailable: bool,
    }

    impl MockClipboard {
        fn with_text(text: &str) -> Self {
            Self {
                text: Some(text.to_owned()),
                unavailable: false,
            }
        }

        fn unavailable() -> Self {
            Self {
                text: None,
                unavailable: true,
            }
        }
    }

    impl Clipboard for MockClipboard {
        fn read_text(&mut self) -> ClipboardResult<Option<String>> {
            if self.unavailable {
                Err(ClipboardError::Unavailable)
            } else {
                Ok(self.text.clone())
            }
        }

        fn write_text(&mut self, text: &str) -> ClipboardResult<()> {
            if self.unavailable {
                Err(ClipboardError::Unavailable)
            } else {
                self.text = Some(text.to_owned());
                Ok(())
            }
        }
    }

    fn record_edit(
        engine: &mut Engine,
        state: &mut TextFieldState,
        buffer: &mut Buffer,
        edit: Edit,
    ) -> TextEditResult {
        record_edit_at(engine, state, buffer, edit, Instant::now())
    }

    fn record_edit_at(
        engine: &mut Engine,
        state: &mut TextFieldState,
        buffer: &mut Buffer,
        edit: Edit,
        now: Instant,
    ) -> TextEditResult {
        state.sync_history(buffer);
        let kind = edit.history_kind();
        let result = engine.apply_text_edit_with_result(buffer, edit);
        if let Some(change) = result.change.clone() {
            state.record_history_at(change, kind, now);
        }
        result
    }

    fn record_command(
        engine: &mut Engine,
        state: &mut TextFieldState,
        buffer: &mut Buffer,
        command: Command,
        clipboard: &mut dyn Clipboard,
    ) -> CommandResult {
        state.sync_history(buffer);
        let outcome = engine.apply_text_command_with_result(buffer, command, clipboard);
        if let Some(change) = outcome.change.clone() {
            state.record_history_at(change, HistoryKind::Boundary, Instant::now());
        }
        outcome.result
    }

    #[test]
    fn document_stores_block_run_and_style_data() {
        let style = Style::default()
            .with_size(18.0)
            .with_color(paint::Color::RED)
            .with_weight(Weight::Bold);
        let mut block = Block::new(Align::Center);
        block.push_run(Run::new("Label", style));
        let document = Document::from_block(block);

        assert_eq!(document.blocks().len(), 1);
        assert_eq!(document.blocks()[0].align(), Align::Center);
        assert_eq!(document.blocks()[0].runs()[0].text(), "Label");
        assert_eq!(document.blocks()[0].runs()[0].style(), style);
    }

    #[test]
    fn empty_document_is_empty() {
        assert!(Document::new().is_empty());
        assert!(Document::plain("").is_empty());
        assert!(!Document::plain("x").is_empty());
    }

    #[test]
    fn document_color_can_be_overridden() {
        let document = Document::plain("Label").with_color(paint::Color::BLACK);

        assert_eq!(
            document.blocks()[0].runs()[0].style().color(),
            paint::Color::BLACK
        );
    }

    #[test]
    fn document_size_can_be_overridden() {
        let document = Document::plain("Label").with_size(12.5);

        assert_eq!(document.blocks()[0].runs()[0].style().size(), 12.5);
    }

    #[test]
    fn plain_document_keeps_raw_default_style() {
        let document = Document::plain("Label");

        assert_eq!(document.blocks()[0].runs()[0].style(), Style::default());
    }

    #[test]
    fn engine_returns_non_zero_metrics_for_non_empty_text() {
        let mut engine = Engine::new();
        let metrics = engine.measure(&Document::plain("Label"), Measure::unbounded());

        assert!(metrics.width() > 0.0);
        assert!(metrics.height() > 0.0);
        assert_eq!(metrics.line_count(), 1);
    }

    #[test]
    fn longer_text_measures_wider_than_shorter_text() {
        let mut engine = Engine::new();
        let short = engine.measure(&Document::plain("Run"), Measure::unbounded());
        let long = engine.measure(&Document::plain("Run workspace task"), Measure::unbounded());

        assert!(long.width() > short.width());
        assert!(long.height() >= short.height());
    }

    #[test]
    fn cloning_buffer_preserves_identity_without_copying_text_state() {
        let buffer = Buffer::from_multiline_text("one\ntwo\nthree");
        let clone = buffer.clone();

        assert!(Rc::ptr_eq(&buffer.inner, &clone.inner));
        assert_eq!(buffer.id(), clone.id());
        assert_eq!(buffer.revision(), clone.revision());
    }

    #[test]
    fn typing_edit_records_transaction_delta() {
        let mut engine = Engine::new();
        let mut buffer = Buffer::from_multiline_text("one\ntwo\nthree");
        let before_revision = buffer.revision();
        let result = engine.apply_text_edit_with_result(&mut buffer, Edit::insert("!"));
        let change = result.change.expect("typing should produce an undo delta");

        assert!(result.text_changed);
        assert!(buffer.revision() > before_revision);
        assert_eq!(change.transaction.deltas.len(), 1);
        assert_eq!(change.transaction.deltas[0].kind, TextEditKind::Insert);
        assert_eq!(change.transaction.deltas[0].inserted, "!");
    }

    #[test]
    fn text_area_frame_cache_reuses_unchanged_frame_and_rebuilds_after_typing() {
        let mut engine = Engine::new();
        let mut buffer = Buffer::from_multiline_text("one\ntwo\nthree");
        let style = Style::default().with_size(13.0);
        let viewport = area::logical(240.0, 120.0);
        let state = TextFieldState::default();
        let now = Instant::now();

        let first = engine
            .text_area_paint_layout_for_area_at(
                &Area::new(buffer.clone()),
                style,
                viewport,
                state.clone(),
                now,
            )
            .into_parts()
            .1;
        let second = engine
            .text_area_paint_layout_for_area_at(
                &Area::new(buffer.clone()),
                style,
                viewport,
                state.clone(),
                now,
            )
            .into_parts()
            .1;
        assert_eq!(surface_line_text(&first, 2), surface_line_text(&second, 2));
        assert!(engine.text_area_line_displays.len() > 0);

        engine.apply_text_edit(&mut buffer, Edit::insert("!"));
        let third = engine
            .text_area_paint_layout_for_area_at(&Area::new(buffer), style, viewport, state, now)
            .into_parts()
            .1;
        assert_eq!(surface_line_text(&third, 2), "three!");
    }

    #[test]
    fn text_diagnostics_record_visible_text_area_cache_work() {
        let mut engine = Engine::new();
        let buffer = Buffer::from_multiline_text("one\ntwo\nthree");
        let area_model = Area::new(buffer);
        let style = Style::default().with_size(13.0);
        let viewport = area::logical(240.0, 120.0);
        let state = TextFieldState::default();
        let now = Instant::now();

        engine.reset_diagnostics();
        engine.text_area_paint_layout_for_area_at(&area_model, style, viewport, state.clone(), now);
        let first = engine.diagnostics();
        assert_eq!(first.text_area_paint_layout_calls, 1);
        assert!(first.text_area_line_cache_misses > 0);
        assert!(first.text_area_line_shape_calls > 0);
        assert!(first.text_area_visible_logical_lines > 0);

        engine.reset_diagnostics();
        engine.text_area_paint_layout_for_area_at(&area_model, style, viewport, state, now);
        let cached = engine.diagnostics();
        assert_eq!(cached.text_area_paint_layout_calls, 1);
        assert!(cached.text_area_line_cache_hits > 0);
        assert_eq!(cached.text_area_line_shape_calls, 0);
    }

    #[test]
    fn text_area_frame_cache_is_bounded() {
        let mut engine = Engine::new();
        let style = Style::default().with_size(13.0);
        let viewport = area::logical(240.0, 80.0);
        let state = TextFieldState::default();
        let now = Instant::now();

        for index in 0..(TEXT_AREA_LINE_DISPLAY_CACHE_CAPACITY + 16) {
            let buffer = Buffer::from_multiline_text(format!("line {index}\nnext"));
            engine.text_area_paint_layout_for_area_at(
                &Area::new(buffer),
                style,
                viewport,
                state.clone(),
                now,
            );
        }

        assert_eq!(
            engine.text_area_line_displays.len(),
            TEXT_AREA_LINE_DISPLAY_CACHE_CAPACITY
        );
    }

    #[test]
    fn text_area_preedit_projection_is_not_cached() {
        let mut engine = Engine::new();
        let buffer = Buffer::from_multiline_text("hello");
        let area_model = Area::new(buffer.clone());
        let style = Style::default().with_size(13.0);
        let viewport = area::logical(240.0, 80.0);
        let state = TextFieldState::default();
        let now = Instant::now();
        let committed = engine
            .text_area_paint_layout_for_area_at(&area_model, style, viewport, state.clone(), now)
            .into_parts()
            .1;
        let preedit_state = state.with_preedit(Some(Preedit::new("x", None)));
        let preedit = engine
            .text_area_paint_layout_for_area_at(&area_model, style, viewport, preedit_state, now)
            .into_parts()
            .1;
        let after = engine
            .text_area_paint_layout_for_area_at(
                &Area::new(buffer),
                style,
                viewport,
                TextFieldState::default(),
                now,
            )
            .into_parts()
            .1;

        assert_eq!(surface_line_text(&preedit, 0), "hellox");
        assert_eq!(
            surface_line_text(&committed, 0),
            surface_line_text(&after, 0)
        );
        assert_eq!(surface_line_text(&after, 0), "hello");
        assert!(engine.text_area_line_displays.len() > 0);
    }

    #[test]
    fn text_area_prepared_frame_is_bounded_to_viewport_window() {
        let mut engine = Engine::new();
        let text = (0..1_000)
            .map(|index| format!("line {index}"))
            .collect::<Vec<_>>()
            .join("\n");
        let buffer = Buffer::from_multiline_text(text);
        let style = Style::default().with_size(13.0);
        let viewport = area::logical(240.0, 52.0);
        let state = TextFieldState::default();
        let now = Instant::now();
        let (layout, surfaces) = engine
            .text_area_paint_layout_for_area_at(&Area::new(buffer), style, viewport, state, now)
            .into_parts();

        assert!(surfaces.len() <= TEXT_AREA_FRAME_MAX_LOGICAL_LINES);
        assert!(surfaces.len() < 1_000);
        assert!(layout.content_area().height() > viewport.height());
    }
    #[test]
    fn large_text_area_scroll_and_highlight_work_are_viewport_bounded() {
        let mut engine = Engine::new();
        let text = (0..100_000)
            .map(|index| format!("line {index}"))
            .collect::<Vec<_>>()
            .join("\n");
        let mut buffer = Buffer::from_multiline_text(text);
        engine.apply_text_edit(&mut buffer, Edit::SelectAll);
        let area_model = Area::new(buffer);
        let style = Style::default().with_size(13.0);
        let viewport = area::logical(240.0, 52.0);
        let state = TextFieldState::default().with_scroll_y(13.0 * 1.25 * 50_000.0);

        engine.reset_interaction_stats();
        engine.reset_highlight_stats();
        let (layout, surfaces) = engine
            .text_area_paint_layout_for_area_at(&area_model, style, viewport, state, Instant::now())
            .into_parts();
        let interaction_stats = engine.interaction_stats();
        let highlight_stats = engine.highlight_stats();
        let visible_runs = surface_visual_runs(&surfaces);

        assert!(!layout.selection_spans().is_empty());
        assert!(surfaces.len() <= TEXT_AREA_FRAME_MAX_LOGICAL_LINES);
        assert!(interaction_stats.text_area_frame_shape_calls <= TEXT_AREA_FRAME_MAX_LOGICAL_LINES);
        assert!(
            interaction_stats.text_area_frame_shaped_logical_lines
                <= TEXT_AREA_FRAME_MAX_LOGICAL_LINES
        );
        assert_eq!(interaction_stats.text_area_shape_until_scroll_calls, 0);
        assert_eq!(highlight_stats.run_scans, visible_runs);
        assert_eq!(highlight_stats.highlight_calls, 0);
    }
    #[test]
    fn piece_tree_insert_updates_touched_storage_without_full_materialization() {
        let mut engine = Engine::new();
        let text = (0..100_000)
            .map(|index| format!("line {index}"))
            .collect::<Vec<_>>()
            .join("\n");
        let mut buffer = Buffer::from_multiline_text(text);
        let paste = (0..512)
            .map(|index| format!("paste {index}"))
            .collect::<Vec<_>>()
            .join("\n");

        buffer.reset_document_stats();
        engine.apply_text_edit(&mut buffer, Edit::insert(paste.clone()));
        let stats = buffer.document_stats();
        let (_owned, _mapped, add) = buffer.document_piece_source_lengths();

        assert_eq!(stats.full_materializations, 0);
        assert_eq!(stats.total_document_scans, 0);
        assert_eq!(stats.piece_tree_updates, 1);
        assert!(add >= paste.lines().map(str::len).sum::<usize>());
    }

    #[test]
    fn anchors_round_trip_through_line_identity() {
        let mut engine = Engine::new();
        let mut buffer = Buffer::from_multiline_text("alpha\nbeta\ngamma");
        let beta = "alpha\nbe".len();

        engine.apply_text_edit(&mut buffer, Edit::set_position(TextPosition::new(beta)));
        let anchor = buffer
            .anchor()
            .expect("cursor should map to a stable anchor");
        engine.apply_text_edit(&mut buffer, Edit::insert("X"));
        let after = buffer
            .anchor()
            .expect("edited cursor should still map to an anchor");

        assert_eq!(anchor.line_id, after.line_id);
        assert!(after.byte_offset > anchor.byte_offset);
    }

    #[test]
    fn mapped_file_buffer_uses_original_mapped_pieces() {
        let path = std::env::temp_dir().join(format!(
            "wgpu_l3_text_mapped_{}_{}.txt",
            std::process::id(),
            Instant::now().elapsed().as_nanos()
        ));
        std::fs::write(&path, "one\ntwo\nthree").expect("temp mapped text should be writable");

        let buffer = Buffer::from_mapped_file(&path).expect("mapped text buffer should open");
        let stats = buffer.document_stats();
        let (owned, mapped, add) = buffer.document_piece_source_lengths();

        assert_eq!(buffer.to_plain_text(), "one\ntwo\nthree");
        assert_eq!(buffer.original_len(), "one\ntwo\nthree".len());
        assert_eq!(owned, 0);
        assert!(mapped >= "onetwothree".len());
        assert_eq!(add, 0);
        assert!(stats.mapped_index_pages_scanned >= 1);

        let _ = std::fs::remove_file(path);
    }
    #[test]
    fn piece_tree_seek_handles_summary_block_boundaries() {
        let lines = (0..300)
            .map(|index| format!("line-{index:03}"))
            .collect::<Vec<_>>();
        let text = lines.join("\n");
        let buffer = Buffer::from_multiline_text(text);
        let boundary_line = TEXT_DOCUMENT_BLOCK_TARGET_LINES;
        let boundary_index = lines
            .iter()
            .take(boundary_line)
            .map(|line| line.len() + 1)
            .sum::<usize>();

        let cursor = buffer.cursor_for_text_index(boundary_index);
        let position = buffer.position_for_text_index(boundary_index);

        assert_eq!(cursor.line, boundary_line);
        assert_eq!(cursor.index, 0);
        assert_eq!(position.index, boundary_index);
    }
    #[test]
    fn larger_font_measures_taller_than_smaller_font() {
        let mut engine = Engine::new();
        let small = Document::from_block({
            let mut block = Block::new(Align::Start);
            block.push_run(Run::new("Label", Style::default().with_size(10.0)));
            block
        });
        let large = Document::from_block({
            let mut block = Block::new(Align::Start);
            block.push_run(Run::new("Label", Style::default().with_size(24.0)));
            block
        });

        let small = engine.measure(&small, Measure::unbounded());
        let large = engine.measure(&large, Measure::unbounded());

        assert!(large.height() > small.height());
    }

    #[test]
    fn repeated_measurement_reuses_cached_metrics() {
        let mut engine = Engine::new();
        let document = Document::plain("Cached Label");

        let first = engine.measure(&document, Measure::unbounded());
        let second = engine.measure(&document, Measure::unbounded());

        assert_eq!(first, second);
        assert_eq!(engine.uncached_measure_count(), 1);
        assert_eq!(engine.cache_len(), 1);
    }

    #[test]
    fn color_only_changes_reuse_cached_metrics() {
        let mut engine = Engine::new();
        let red = Document::plain("Cached Label").with_color(paint::Color::RED);
        let black = Document::plain("Cached Label").with_color(paint::Color::BLACK);

        let red = engine.measure(&red, Measure::unbounded());
        let black = engine.measure(&black, Measure::unbounded());

        assert_eq!(red, black);
        assert_eq!(engine.uncached_measure_count(), 1);
    }

    #[test]
    fn shaping_relevant_document_and_bounds_changes_use_distinct_cache_keys() {
        let mut engine = Engine::new();
        let base = styled_document("Cached Label", Align::Start, 16.0, Weight::Normal);
        let text = styled_document("Different Label", Align::Start, 16.0, Weight::Normal);
        let size = styled_document("Cached Label", Align::Start, 20.0, Weight::Normal);
        let weight = styled_document("Cached Label", Align::Start, 16.0, Weight::Bold);
        let align = styled_document("Cached Label", Align::End, 16.0, Weight::Normal);

        engine.measure(&base, Measure::unbounded());
        engine.measure(&text, Measure::unbounded());
        engine.measure(&size, Measure::unbounded());
        engine.measure(&weight, Measure::unbounded());
        engine.measure(&align, Measure::unbounded());
        engine.measure(&base, Measure::bounded(area::logical(40.0, 100.0)));

        assert_eq!(engine.uncached_measure_count(), 6);
        assert_eq!(engine.cache_len(), 6);
    }

    #[test]
    fn bounded_fifo_cache_evicts_oldest_entries() {
        let mut engine = Engine::with_cache_capacity(2);
        let first = Document::plain("First");
        let second = Document::plain("Second");
        let third = Document::plain("Third");

        engine.measure(&first, Measure::unbounded());
        engine.measure(&second, Measure::unbounded());
        engine.measure(&third, Measure::unbounded());
        engine.measure(&first, Measure::unbounded());

        assert_eq!(engine.cache_len(), 2);
        assert_eq!(engine.uncached_measure_count(), 4);
    }

    #[test]
    fn buffer_inserts_and_deletes_text() {
        let mut engine = Engine::new();
        let mut buffer = Buffer::from_text("ab");

        engine.apply_text_edit(&mut buffer, Edit::insert("c"));
        engine.apply_text_edit(
            &mut buffer,
            Edit::motion(glyphon::cosmic_text::Motion::Left),
        );
        engine.apply_text_edit(&mut buffer, Edit::action(glyphon::Action::Backspace));

        assert_eq!(buffer.text(), "ac");
        assert_eq!(buffer.cursor().index, 1);

        engine.apply_text_edit(&mut buffer, Edit::action(glyphon::Action::Delete));

        assert_eq!(buffer.text(), "a");
        assert_eq!(buffer.cursor().index, 1);
    }

    #[test]
    fn buffer_select_all_replaces_selection() {
        let mut engine = Engine::new();
        let mut buffer = Buffer::from_text("hello");

        engine.apply_text_edit(&mut buffer, Edit::SelectAll);
        assert_eq!(buffer.selected_range(), Some(TextRange::new(0, 5)));

        engine.apply_text_edit(&mut buffer, Edit::insert("hi"));

        assert_eq!(buffer.text(), "hi");
        assert_eq!(buffer.cursor().index, 2);
        assert_eq!(buffer.selected_range(), None);
    }

    #[test]
    fn replace_range_normalizes_inserted_text_and_restores_caret() {
        let mut engine = Engine::new();
        let mut buffer = Buffer::from_text("hello world");

        assert!(engine.apply_text_edit(&mut buffer, Edit::replace_range(6..11, "there\nfriend")));

        assert_eq!(buffer.text(), "hello there friend");
        assert_eq!(buffer.cursor(), Cursor::new(0, "hello there friend".len()));
        assert_eq!(buffer.selected_range(), None);
    }

    #[test]
    fn move_range_adjusts_forward_drop_position() {
        let mut engine = Engine::new();
        let mut buffer = Buffer::from_text("abcdef");

        assert!(engine.apply_text_edit(&mut buffer, Edit::move_range(1..3, 5)));

        assert_eq!(buffer.text(), "adebcf");
        assert_eq!(buffer.cursor(), Cursor::new(0, 5));
        assert_eq!(buffer.selected_range(), None);
    }

    #[test]
    fn text_command_copy_writes_selection_without_mutating_buffer() {
        let mut engine = Engine::new();
        let mut buffer = Buffer::from_text("hello");
        let mut clipboard = MockClipboard::default();

        engine.apply_text_edit(&mut buffer, Edit::SelectAll);
        let result = engine.apply_text_command(&mut buffer, Command::Copy, &mut clipboard);

        assert_eq!(clipboard.text.as_deref(), Some("hello"));
        assert_eq!(buffer.text(), "hello");
        assert_eq!(buffer.selected_range(), Some(TextRange::new(0, 5)));
        assert!(result.clipboard_changed);
        assert!(!result.buffer_changed());
        assert!(!result.unavailable);
    }

    #[test]
    fn text_command_cut_copies_and_deletes_selection() {
        let mut engine = Engine::new();
        let mut buffer = Buffer::from_text("hello");
        let mut clipboard = MockClipboard::default();

        engine.apply_text_edit(&mut buffer, Edit::SelectAll);
        let result = engine.apply_text_command(&mut buffer, Command::Cut, &mut clipboard);

        assert_eq!(clipboard.text.as_deref(), Some("hello"));
        assert_eq!(buffer.text(), "");
        assert_eq!(buffer.selected_range(), None);
        assert!(result.clipboard_changed);
        assert!(result.text_changed);
        assert!(result.selection_changed);
        assert!(!result.unavailable);
    }

    #[test]
    fn text_command_paste_replaces_selection_and_normalizes_line_endings() {
        let mut engine = Engine::new();
        let mut buffer = Buffer::from_text("hello");
        let mut clipboard = MockClipboard::with_text("a\nb\rc");

        engine.apply_text_edit(&mut buffer, Edit::SelectAll);
        let result = engine.apply_text_command(&mut buffer, Command::Paste, &mut clipboard);

        assert_eq!(buffer.text(), "a b c");
        assert_eq!(buffer.selected_range(), None);
        assert!(result.text_changed);
        assert!(result.selection_changed);
        assert!(!result.clipboard_changed);
        assert!(!result.unavailable);
    }

    #[test]
    fn repeated_large_paste_updates_line_index_without_full_rebuild() {
        let mut engine = Engine::new();
        let text = (0..100_000)
            .map(|line| format!("line {line}"))
            .collect::<Vec<_>>()
            .join("\n");
        let mut buffer = Buffer::from_multiline_text(text);
        let block = (0..64)
            .map(|line| format!("paste {line}"))
            .collect::<Vec<_>>()
            .join("\n");
        let end = TextPosition::new(buffer.len());

        engine.apply_text_edit(&mut buffer, Edit::set_position(end));
        buffer.reset_line_index_stats();

        assert!(engine.apply_text_edit(&mut buffer, Edit::insert(format!("\n{block}"))));
        assert!(engine.apply_text_edit(&mut buffer, Edit::insert(format!("\n{block}"))));

        let (full_rebuilds, splice_updates) = buffer.line_index_stats();
        assert_eq!(full_rebuilds, 0);
        assert_eq!(splice_updates, 2);
        assert_eq!(buffer.logical_line_count(), 100_000 + 128);
        assert!(buffer.text().ends_with("paste 63"));
    }
    #[test]
    fn text_command_paste_without_text_or_clipboard_does_not_mutate() {
        let mut engine = Engine::new();
        let mut buffer = Buffer::from_text("hello");
        let mut empty_clipboard = MockClipboard::default();

        let empty = engine.apply_text_command(&mut buffer, Command::Paste, &mut empty_clipboard);

        assert_eq!(buffer.text(), "hello");
        assert!(!empty.changed());
        assert!(!empty.unavailable);

        let mut unavailable_clipboard = MockClipboard::unavailable();
        let unavailable =
            engine.apply_text_command(&mut buffer, Command::Paste, &mut unavailable_clipboard);

        assert_eq!(buffer.text(), "hello");
        assert!(!unavailable.changed());
        assert!(unavailable.unavailable);
    }

    #[test]
    fn text_history_coalesces_typing_into_one_undo_step() {
        let mut engine = Engine::new();
        let mut state = TextFieldState::default();
        let mut buffer = Buffer::new();

        record_edit(&mut engine, &mut state, &mut buffer, Edit::insert("a"));
        record_edit(&mut engine, &mut state, &mut buffer, Edit::insert("b"));
        record_edit(&mut engine, &mut state, &mut buffer, Edit::insert("c"));

        assert_eq!(buffer.text(), "abc");
        assert_eq!(state.history.undo.len(), 1);
        assert!(state.can_undo());

        let undo = state.apply_undo(&mut buffer);
        assert_eq!(buffer.text(), "");
        assert!(undo.text_changed);
        assert!(state.can_redo());

        let redo = state.apply_redo(&mut buffer);
        assert_eq!(buffer.text(), "abc");
        assert!(redo.text_changed);
    }
    #[test]
    fn text_history_splits_typing_after_coalesce_timeout() {
        let mut engine = Engine::new();
        let mut state = TextFieldState::default();
        let mut buffer = Buffer::new();
        let start = Instant::now();

        record_edit_at(
            &mut engine,
            &mut state,
            &mut buffer,
            Edit::insert("a"),
            start,
        );
        record_edit_at(
            &mut engine,
            &mut state,
            &mut buffer,
            Edit::insert("b"),
            start + TYPING_UNDO_COALESCE_WINDOW + Duration::from_millis(1),
        );

        assert_eq!(buffer.text(), "ab");
        assert_eq!(state.history.undo.len(), 2);
    }

    #[test]
    fn text_history_splits_typing_at_whitespace_and_punctuation() {
        let mut engine = Engine::new();
        let mut state = TextFieldState::default();
        let mut buffer = Buffer::new();

        record_edit(&mut engine, &mut state, &mut buffer, Edit::insert("a"));
        record_edit(&mut engine, &mut state, &mut buffer, Edit::insert(" "));
        record_edit(&mut engine, &mut state, &mut buffer, Edit::insert("b"));
        record_edit(&mut engine, &mut state, &mut buffer, Edit::insert("."));

        assert_eq!(buffer.text(), "a b.");
        assert_eq!(state.history.undo.len(), 4);
    }

    #[test]
    fn text_history_splits_typing_after_cursor_movement() {
        let mut engine = Engine::new();
        let mut state = TextFieldState::default();
        let mut buffer = Buffer::new();

        record_edit(&mut engine, &mut state, &mut buffer, Edit::insert("a"));
        engine.apply_text_edit(&mut buffer, Edit::set_cursor(Cursor::new(0, 0)));
        record_edit(&mut engine, &mut state, &mut buffer, Edit::insert("b"));

        assert_eq!(buffer.text(), "ba");
        assert_eq!(state.history.undo.len(), 2);
        state.apply_undo(&mut buffer);
        assert_eq!(buffer.text(), "a");
    }

    #[test]
    fn text_history_keeps_paste_cut_delete_word_delete_and_ime_as_separate_steps() {
        let mut engine = Engine::new();
        let mut state = TextFieldState::default();
        let mut buffer = Buffer::from_text("hello");
        let mut clipboard = MockClipboard::with_text(" pasted");

        record_command(
            &mut engine,
            &mut state,
            &mut buffer,
            Command::Paste,
            &mut clipboard,
        );
        record_edit(
            &mut engine,
            &mut state,
            &mut buffer,
            Edit::action(glyphon::Action::Backspace),
        );
        record_edit(
            &mut engine,
            &mut state,
            &mut buffer,
            Edit::delete_word_backward(),
        );
        record_edit(&mut engine, &mut state, &mut buffer, Edit::ime_commit("x"));

        engine.apply_text_edit(&mut buffer, Edit::SelectAll);
        let mut clipboard = MockClipboard::default();
        record_command(
            &mut engine,
            &mut state,
            &mut buffer,
            Command::Cut,
            &mut clipboard,
        );

        assert_eq!(state.history.undo.len(), 5);
    }

    #[test]
    fn text_history_undo_restores_text_cursor_and_selection() {
        let mut engine = Engine::new();
        let mut state = TextFieldState::default();
        let mut buffer = Buffer::from_text("hello");

        state.sync_history(&buffer);
        engine.apply_text_edit(&mut buffer, Edit::SelectAll);
        record_edit(&mut engine, &mut state, &mut buffer, Edit::insert("x"));

        assert_eq!(buffer.text(), "x");
        assert!(!buffer.has_selection());

        let undo = state.apply_undo(&mut buffer);
        assert_eq!(buffer.text(), "hello");
        assert_eq!(buffer.selected_text().as_deref(), Some("hello"));
        assert!(undo.text_changed);
        assert!(undo.selection_changed);

        let redo = state.apply_redo(&mut buffer);
        assert_eq!(buffer.text(), "x");
        assert!(!buffer.has_selection());
        assert!(redo.text_changed);
    }

    #[test]
    fn text_history_new_edit_after_undo_clears_redo() {
        let mut engine = Engine::new();
        let mut state = TextFieldState::default();
        let mut buffer = Buffer::new();

        record_edit(&mut engine, &mut state, &mut buffer, Edit::insert("a"));
        state.apply_undo(&mut buffer);
        assert!(state.can_redo());

        record_edit(&mut engine, &mut state, &mut buffer, Edit::insert("b"));

        assert_eq!(buffer.text(), "b");
        assert!(!state.can_redo());
        assert!(state.can_undo());
    }

    #[test]
    fn text_history_external_buffer_replacement_clears_stale_history() {
        let mut engine = Engine::new();
        let mut state = TextFieldState::default();
        let mut buffer = Buffer::new();

        record_edit(&mut engine, &mut state, &mut buffer, Edit::insert("a"));
        assert!(state.can_undo());

        let external = Buffer::from_text("external");
        assert!(state.sync_history(&external));
        assert!(!state.can_undo());
        assert!(!state.can_redo());
    }

    #[test]
    fn buffer_shift_motion_extends_selection() {
        let mut engine = Engine::new();
        let mut buffer = Buffer::from_text("hello");

        engine.apply_text_edit(
            &mut buffer,
            Edit::extend_motion(glyphon::cosmic_text::Motion::Left),
        );

        assert_eq!(buffer.cursor().index, 4);
        assert_eq!(buffer.selected_range(), Some(TextRange::new(4, 5)));

        engine.apply_text_edit(
            &mut buffer,
            Edit::extend_motion(glyphon::cosmic_text::Motion::Home),
        );

        assert_eq!(buffer.cursor().index, 0);
        assert_eq!(buffer.selected_range(), Some(TextRange::new(0, 5)));
    }

    #[test]
    fn buffer_plain_motion_collapses_selection() {
        let mut engine = Engine::new();
        let mut buffer = Buffer::from_text("hello");

        engine.apply_text_edit(&mut buffer, Edit::SelectAll);
        engine.apply_text_edit(
            &mut buffer,
            Edit::motion(glyphon::cosmic_text::Motion::Left),
        );

        assert_eq!(buffer.cursor().index, 0);
        assert_eq!(buffer.selected_range(), None);

        engine.apply_text_edit(&mut buffer, Edit::SelectAll);
        engine.apply_text_edit(
            &mut buffer,
            Edit::motion(glyphon::cosmic_text::Motion::Right),
        );

        assert_eq!(buffer.cursor().index, 5);
        assert_eq!(buffer.selected_range(), None);
    }

    #[test]
    fn buffer_word_delete_uses_cosmic_word_motion() {
        let mut engine = Engine::new();
        let mut buffer = Buffer::from_text("hello world again");

        engine.apply_text_edit(&mut buffer, Edit::delete_word_backward());

        assert_eq!(buffer.text(), "hello world ");
        assert_eq!(buffer.cursor().index, "hello world ".len());

        engine.apply_text_edit(&mut buffer, Edit::set_cursor(Cursor::new(0, 0)));
        engine.apply_text_edit(&mut buffer, Edit::delete_word_forward());

        assert_eq!(buffer.text(), " world ");
        assert_eq!(buffer.cursor().index, 0);
    }

    #[test]
    fn buffer_pointer_double_click_selects_word_and_triple_click_selects_all() {
        let mut engine = Engine::new();
        let mut buffer = Buffer::from_text("hello world");

        engine.apply_text_edit(
            &mut buffer,
            Edit::pointer(PointerEditKind::DoubleClick, Cursor::new(0, 1)),
        );

        assert_eq!(buffer.selected_range(), Some(TextRange::new(0, 5)));

        engine.apply_text_edit(
            &mut buffer,
            Edit::pointer(PointerEditKind::TripleClick, Cursor::new(0, 7)),
        );

        assert_eq!(
            buffer.selected_range(),
            Some(TextRange::new(0, "hello world".len()))
        );
    }

    #[test]
    fn buffer_pointer_drag_extends_from_click_anchor() {
        let mut engine = Engine::new();
        let mut buffer = Buffer::from_text("hello world");

        engine.apply_text_edit(
            &mut buffer,
            Edit::pointer(PointerEditKind::Click, Cursor::new(0, 0)),
        );
        engine.apply_text_edit(
            &mut buffer,
            Edit::pointer(PointerEditKind::Drag, Cursor::new(0, 5)),
        );

        assert_eq!(buffer.selected_range(), Some(TextRange::new(0, 5)));
    }

    #[test]
    fn buffer_edits_preserve_unicode_boundaries() {
        let mut engine = Engine::new();
        let mut buffer = Buffer::from_text("aé🙂");

        engine.apply_text_edit(&mut buffer, Edit::set_cursor(Cursor::new(0, 3)));
        assert_eq!(buffer.cursor().index, "aé".len());

        engine.apply_text_edit(&mut buffer, Edit::action(glyphon::Action::Backspace));
        assert_eq!(buffer.text(), "a🙂");

        engine.apply_text_edit(&mut buffer, Edit::motion(glyphon::cosmic_text::Motion::End));
        engine.apply_text_edit(&mut buffer, Edit::action(glyphon::Action::Backspace));
        assert_eq!(buffer.text(), "a");
        assert!(buffer.text().is_char_boundary(buffer.cursor().index));
    }
    #[test]
    fn byte_index_edits_snap_to_grapheme_boundaries() {
        let mut engine = Engine::new();
        let combining = "e\u{301}";
        let family = "👨‍👩‍👧‍👦";
        let flag = "🇺🇸";

        let mut replace = Buffer::from_text(format!("a{combining}b"));
        assert!(engine.apply_text_edit(&mut replace, Edit::replace_range(2..3, "X")));
        assert_eq!(replace.text(), "aXb");

        let mut insert = Buffer::from_text(format!("a{family}b"));
        let inside_family = 1 + "👨".len();
        assert!(engine.apply_text_edit(&mut insert, Edit::insert_at(inside_family, "X")));
        assert_eq!(insert.text(), format!("aX{family}b"));

        let flag_source = format!("a{flag}bc");
        let mut moved = Buffer::from_text(flag_source.clone());
        assert!(engine.apply_text_edit(&mut moved, Edit::move_range(3..6, flag_source.len())));
        assert_eq!(moved.text(), format!("abc{flag}"));

        let cursor_buffer = Buffer::from_text(format!("a{family}b"));
        let cursor = cursor_buffer.cursor_for_text_index(inside_family);
        assert_eq!(cursor_buffer.text_index_for_cursor(cursor), 1);
        assert_eq!(
            Field::new(format!("{combining}{family}{flag}"))
                .obscured_dot()
                .presentation_text(),
            "•••"
        );
    }

    #[test]
    fn logical_motion_respects_grapheme_boundaries() {
        let mut engine = Engine::new();
        let family = "👨‍👩‍👧‍👦";
        let mut buffer = Buffer::from_text(format!("a{family}b"));

        let end = buffer.text().len();
        engine.apply_text_edit(&mut buffer, Edit::set_position(end));
        engine.reset_interaction_stats();
        engine.apply_text_edit(
            &mut buffer,
            Edit::move_position(TextMotion::LogicalPrevious),
        );
        assert_eq!(buffer.position().index, 1 + family.len());

        engine.apply_text_edit(
            &mut buffer,
            Edit::move_position(TextMotion::LogicalPrevious),
        );
        assert_eq!(buffer.position().index, 1);
        assert_eq!(engine.interaction_stats().aggregate_buffer_fallbacks, 0);
    }

    #[test]
    fn unicode_word_boundaries_drive_selection_and_delete() {
        let mut engine = Engine::new();
        let mut buffer = Buffer::from_text("hello שלום again");
        let hebrew_start = "hello ".len();

        engine.apply_text_edit(
            &mut buffer,
            Edit::pointer(
                PointerEditKind::DoubleClick,
                TextPosition::new(hebrew_start + 2),
            ),
        );
        assert_eq!(buffer.selected_text().as_deref(), Some("שלום"));
        assert_eq!(
            buffer.selected_range(),
            Some(TextRange::new(hebrew_start, hebrew_start + "שלום".len()))
        );

        let end = buffer.text().len();
        engine.apply_text_edit(&mut buffer, Edit::set_position(end));
        engine.reset_interaction_stats();
        engine.apply_text_edit(&mut buffer, Edit::delete_word_backward());
        assert_eq!(buffer.text(), "hello שלום ");
    }

    #[test]
    fn bidi_hit_testing_preserves_visual_affinity() {
        let mut engine = Engine::new();
        let buffer = Buffer::from_text("abc אבג");
        let prepared = engine.prepare_text_field_buffer(
            &buffer,
            Style::default().with_size(18.0),
            area::logical(400.0, 32.0),
        );
        let prepared = prepared.0;
        let map = TextLayoutMap::new(&prepared);
        let rtl_glyph = prepared
            .layout_runs()
            .flat_map(|run| {
                let line_start = map.line_starts.get(run.line_i).copied().unwrap_or(0);
                run.glyphs
                    .iter()
                    .map(move |glyph| (run.line_top, run.line_height, line_start, glyph))
            })
            .find(|(_, _, _, glyph)| glyph.level.is_rtl())
            .expect("mixed Hebrew text should produce an RTL glyph");
        let (line_top, line_height, line_start, glyph) = rtl_glyph;
        let y = line_top + line_height * 0.5;

        let left = map
            .hit(&prepared, glyph.x + glyph.w * 0.25, y)
            .expect("left half should hit the RTL glyph");
        let right = map
            .hit(&prepared, glyph.x + glyph.w * 0.75, y)
            .expect("right half should hit the RTL glyph");

        assert_eq!(
            left,
            TextPosition::with_affinity(line_start + glyph.end, TextAffinity::Upstream)
        );
        assert_eq!(
            right,
            TextPosition::with_affinity(line_start + glyph.start, TextAffinity::Downstream)
        );
    }

    #[test]
    fn start_end_alignment_resolves_against_base_direction() {
        assert_eq!(
            text_system::align(Align::Start, ResolvedTextDirection::Ltr),
            glyphon::cosmic_text::Align::Left
        );
        assert_eq!(
            text_system::align(Align::Start, ResolvedTextDirection::Rtl),
            glyphon::cosmic_text::Align::Right
        );
        assert_eq!(
            text_system::align(Align::End, ResolvedTextDirection::Rtl),
            glyphon::cosmic_text::Align::Left
        );
    }

    #[test]
    fn mixed_direction_preedit_spans_are_projected_inline() {
        let mut engine = Engine::new();
        let buffer = Buffer::from_text("abc אבג");
        let state = TextFieldState::default()
            .with_preedit(Some(Preedit::new("שלום", Some((0, "של".len())))));
        let layout = engine.text_field_layout_at(
            &buffer,
            Style::default().with_size(18.0),
            area::logical(400.0, 32.0),
            state,
            Instant::now(),
        );

        assert!(layout.caret().is_some());
        assert!(!layout.preedit_underline_spans().is_empty());
        assert!(!layout.preedit_selection_spans().is_empty());
    }
    #[test]
    fn buffer_normalizes_inserted_line_endings_to_spaces() {
        let mut engine = Engine::new();
        let mut buffer = Buffer::from_text("a\nb");

        assert_eq!(buffer.text(), "a b");

        engine.apply_text_edit(&mut buffer, Edit::insert("\nc\r"));

        assert_eq!(buffer.text(), "a b c ");
    }

    #[test]
    fn text_field_selection_layout_uses_shaped_text_span() {
        let mut engine = Engine::new();
        let mut buffer = Buffer::from_text("hello");

        engine.apply_text_edit(&mut buffer, Edit::SelectAll);

        let layout = engine.text_field_layout(
            &buffer,
            Style::default().with_size(16.0),
            area::logical(240.0, 32.0),
            TextFieldState::default(),
        );
        let span = layout
            .selection_spans()
            .first()
            .expect("select all should create a highlight span");

        assert!(span.width() > 0.0);
        assert!(span.width() < 240.0);
        assert!(span.x() >= 0.0);
    }
    #[test]
    fn text_field_preedit_renders_inline_text_spans_and_commit_clears_projection() {
        let mut engine = Engine::new();
        let mut buffer = Buffer::from_text("hello");
        engine.apply_text_edit(&mut buffer, Edit::SelectAll);
        let state = TextFieldState::default().with_preedit(Some(Preedit::new("xy", Some((0, 1)))));
        let field = Field::new(buffer.clone());

        assert_eq!(field.presentation_text_for_state(&state), "xy");

        let layout = engine.text_field_layout_at(
            &buffer,
            Style::default().with_size(16.0),
            area::logical(240.0, 32.0),
            state,
            Instant::now(),
        );

        assert!(layout.caret().is_some());
        assert!(!layout.preedit_underline_spans().is_empty());
        assert!(!layout.preedit_selection_spans().is_empty());

        engine.apply_text_edit(&mut buffer, Edit::ime_commit("xy"));
        let committed = engine.text_field_layout(
            &buffer,
            Style::default().with_size(16.0),
            area::logical(240.0, 32.0),
            TextFieldState::default(),
        );

        assert_eq!(buffer.text(), "xy");
        assert!(committed.preedit_underline_spans().is_empty());
        assert!(committed.preedit_selection_spans().is_empty());
    }

    #[test]
    fn text_field_preedit_caret_uses_composed_projection() {
        let mut engine = Engine::new();
        let buffer = Buffer::from_text("hello");
        let style = Style::default().with_size(16.0);
        let viewport = area::logical(240.0, 32.0);
        let now = Instant::now();
        let committed = engine
            .text_field_layout_at(&buffer, style, viewport, TextFieldState::default(), now)
            .caret()
            .expect("committed caret should be visible");
        let composed = engine
            .text_field_layout_at(
                &buffer,
                style,
                viewport,
                TextFieldState::default().with_preedit(Some(Preedit::new(" world", None))),
                now,
            )
            .caret()
            .expect("preedit caret should be visible");

        assert!(composed.x() > committed.x());
    }

    #[test]
    fn text_area_metrics_layout_skips_highlight_overlay_work() {
        let mut engine = Engine::new();
        let text = (0..1_000)
            .map(|index| format!("line {index}"))
            .collect::<Vec<_>>()
            .join("\n");
        let mut buffer = Buffer::from_multiline_text(text);
        engine.apply_text_edit(&mut buffer, Edit::SelectAll);
        let area_model = Area::new(buffer);
        let style = Style::default().with_size(13.0);
        let viewport = area::logical(240.0, 52.0);

        engine.reset_highlight_stats();
        engine.reset_interaction_stats();
        let layout = engine.text_area_metrics_layout_for_area_at(
            &area_model,
            style,
            viewport,
            TextFieldState::default(),
            Instant::now(),
        );

        assert_eq!(engine.highlight_stats(), HighlightStats::default());
        let interaction_stats = engine.interaction_stats();
        assert_eq!(interaction_stats.text_area_frame_shape_calls, 0);
        assert_eq!(interaction_stats.text_area_shape_until_scroll_calls, 0);
        assert!(layout.selection_spans().is_empty());
        assert!(layout.preedit_underline_spans().is_empty());
        assert!(layout.preedit_selection_spans().is_empty());
    }

    #[test]
    fn text_area_paint_layout_computes_highlight_overlays_from_visible_surfaces() {
        let mut engine = Engine::new();
        let text = (0..1_000)
            .map(|index| format!("line {index}"))
            .collect::<Vec<_>>()
            .join("\n");
        let mut buffer = Buffer::from_multiline_text(text);
        engine.apply_text_edit(&mut buffer, Edit::SelectAll);
        let area_model = Area::new(buffer);
        let style = Style::default().with_size(13.0);
        let viewport = area::logical(240.0, 52.0);
        let state = TextFieldState::default();
        let now = Instant::now();

        engine.reset_highlight_stats();
        let (layout, surfaces) = engine
            .text_area_paint_layout_for_area_at(&area_model, style, viewport, state.clone(), now)
            .into_parts();
        let stats = engine.highlight_stats();
        let visible_runs = surface_visual_runs(&surfaces);

        assert!(!layout.selection_spans().is_empty());
        assert!(visible_runs <= TEXT_AREA_FRAME_MAX_LOGICAL_LINES);
        assert!(visible_runs < 1_000);
        assert_eq!(stats.run_scans, visible_runs);
        assert_eq!(stats.highlight_calls, 0);
        assert_eq!(stats.spans, layout.selection_spans().len());

        engine.reset_highlight_stats();
        let cached =
            engine.text_area_paint_layout_for_area_at(&area_model, style, viewport, state, now);
        let cached_stats = engine.highlight_stats();

        assert!(!cached.layout().selection_spans().is_empty());
        assert_eq!(cached_stats.run_scans, visible_runs);
        assert_eq!(cached_stats.highlight_calls, 0);
        assert_eq!(cached_stats.spans, cached.layout().selection_spans().len());
    }

    #[test]
    fn wrapped_text_area_line_displays_do_not_overlap() {
        let mut engine = Engine::new();
        let long = "wrap ".repeat(40);
        let area_model = Area::new(Buffer::from_multiline_text(format!("{long}\nnext")));
        let style = Style::default().with_size(16.0);
        let viewport = area::logical(72.0, 220.0);

        let paint_layout = engine.text_area_paint_layout_for_area_at(
            &area_model,
            style,
            viewport,
            TextFieldState::default(),
            Instant::now(),
        );
        let surfaces = paint_layout.surfaces();

        assert!(surfaces.len() >= 2);
        assert_eq!(surface_line_text(surfaces, 1), "next");
        let first_bottom = surfaces[0].y() + surfaces[0].height();
        assert!(
            surfaces[1].y() >= first_bottom - 0.5,
            "next line started at {}, before wrapped first line bottom {}",
            surfaces[1].y(),
            first_bottom
        );
    }

    #[test]
    fn wrapped_text_area_hit_testing_uses_clicked_visual_row() {
        let mut engine = Engine::new();
        let text = "alpha beta gamma delta epsilon zeta eta theta iota kappa lambda mu";
        let buffer = Buffer::from_multiline_text(text);
        let area_model = Area::new(buffer);
        let style = Style::default().with_size(16.0);
        let viewport = area::logical(86.0, 180.0);

        let (x, y, first_row_end, second_row_start, second_row_end) = {
            let display = engine.text_area_line_display(
                &area_model,
                area_model.buffer(),
                true,
                style,
                viewport,
                0,
            );
            let prepared = display.buffer.borrow();
            let runs = prepared.layout_runs().collect::<Vec<_>>();
            let groups = TextLayoutMap::visual_line_groups(&runs);
            assert!(
                groups.len() >= 2,
                "test text should wrap into at least two visual rows"
            );
            let first_range = visual_group_source_range(&runs, groups[0], display.source_start)
                .expect("first visual row should have glyphs");
            let second_range = visual_group_source_range(&runs, groups[1], display.source_start)
                .expect("second visual row should have glyphs");
            let first_run = &runs[groups[1].start];
            let first_glyph = first_run
                .glyphs
                .first()
                .expect("second visual row should have a first glyph");
            (
                first_glyph.x + first_glyph.w * 0.25,
                (groups[1].top + groups[1].bottom) * 0.5,
                first_range.end,
                second_range.start,
                second_range.end,
            )
        };

        assert!(second_row_start >= first_row_end);
        let hit = engine
            .text_area_position_at_for_area(
                &area_model,
                style,
                viewport,
                point::logical(x, y),
                TextFieldState::default(),
            )
            .expect("wrapped visual row hit should resolve to a caret");

        assert!(
            hit.index >= second_row_start && hit.index <= second_row_end,
            "hit index {} should be inside second visual row range {}..{} instead of first row ending at {}",
            hit.index,
            second_row_start,
            second_row_end,
            first_row_end
        );
    }

    #[test]
    fn wrapped_text_area_drag_selection_extends_into_lower_visual_row() {
        let mut engine = Engine::new();
        let text = "alpha beta gamma delta epsilon zeta eta theta iota kappa lambda mu";
        let mut buffer = Buffer::from_multiline_text(text);
        let area_model = Area::new(buffer.clone());
        let style = Style::default().with_size(16.0);
        let viewport = area::logical(86.0, 180.0);
        let (start_point, end_point, first_row_end, second_row_top) = {
            let display = engine.text_area_line_display(
                &area_model,
                area_model.buffer(),
                true,
                style,
                viewport,
                0,
            );
            let prepared = display.buffer.borrow();
            let runs = prepared.layout_runs().collect::<Vec<_>>();
            let groups = TextLayoutMap::visual_line_groups(&runs);
            assert!(
                groups.len() >= 2,
                "test text should wrap into at least two visual rows"
            );
            let first_range = visual_group_source_range(&runs, groups[0], display.source_start)
                .expect("first visual row should have glyphs");
            let first_run = &runs[groups[0].start];
            let second_run = &runs[groups[1].start];
            let start_glyph = first_run
                .glyphs
                .first()
                .expect("first visual row should have a first glyph");
            let end_glyph = second_run
                .glyphs
                .last()
                .expect("second visual row should have a last glyph");
            (
                point::logical(
                    start_glyph.x + start_glyph.w * 0.25,
                    (groups[0].top + groups[0].bottom) * 0.5,
                ),
                point::logical(
                    end_glyph.x + end_glyph.w * 0.75,
                    (groups[1].top + groups[1].bottom) * 0.5,
                ),
                first_range.end,
                groups[1].top,
            )
        };

        let start = engine
            .text_area_position_at_for_area(
                &area_model,
                style,
                viewport,
                start_point,
                TextFieldState::default(),
            )
            .expect("drag start should resolve to a caret");
        let end = engine
            .text_area_position_at_for_area(
                &area_model,
                style,
                viewport,
                end_point,
                TextFieldState::default(),
            )
            .expect("drag end should resolve to a caret");

        engine.apply_text_edit(&mut buffer, Edit::pointer(PointerEditKind::Click, start));
        engine.apply_text_edit(&mut buffer, Edit::pointer(PointerEditKind::Drag, end));

        let selected = buffer
            .selected_range()
            .expect("drag across wrapped rows should create a selection");
        assert!(
            selected.end > first_row_end,
            "selection {:?} should extend beyond first visual row ending at {}",
            selected,
            first_row_end
        );

        let layout = engine.text_area_paint_layout_for_area_at(
            &area_model,
            style,
            viewport,
            TextFieldState::default(),
            Instant::now(),
        );
        assert!(
            layout
                .layout()
                .selection_spans()
                .iter()
                .any(|span| (span.y() - second_row_top).abs() <= TEXT_LAYOUT_VISUAL_LINE_EPSILON),
            "selection highlight should include the lower wrapped visual row"
        );
    }
    #[test]
    fn text_area_metrics_reuse_measured_wrapped_heights_after_paint() {
        let mut engine = Engine::new();
        let long = "wrap ".repeat(40);
        let area_model = Area::new(Buffer::from_multiline_text(format!("{long}\nnext")));
        let style = Style::default().with_size(16.0);
        let viewport = area::logical(72.0, 24.0);
        let state = TextFieldState::default();

        let cold = engine.text_area_metrics_layout_for_area_at(
            &area_model,
            style,
            viewport,
            state.clone(),
            Instant::now(),
        );
        let _paint = engine.text_area_paint_layout_for_area_at(
            &area_model,
            style,
            viewport,
            state.clone(),
            Instant::now(),
        );
        let warm = engine.text_area_metrics_layout_for_area_at(
            &area_model,
            style,
            viewport,
            state,
            Instant::now(),
        );

        assert!(
            warm.content_area().height() > cold.content_area().height(),
            "painted wrapped line measurements should refine content height from {} to more than it, got {}",
            cold.content_area().height(),
            warm.content_area().height()
        );
    }
    #[test]
    fn text_area_overlay_cache_key_tracks_scroll_window() {
        let mut engine = Engine::new();
        let text = (0..1_000)
            .map(|index| format!("line {index}"))
            .collect::<Vec<_>>()
            .join("\n");
        let mut buffer = Buffer::from_multiline_text(text);
        engine.apply_text_edit(&mut buffer, Edit::SelectAll);
        let area_model = Area::new(buffer);
        let style = Style::default().with_size(13.0);
        let viewport = area::logical(240.0, 52.0);
        let line_height = 13.0 * 1.25;
        let now = Instant::now();

        engine.reset_highlight_stats();
        engine.text_area_paint_layout_for_area_at(
            &area_model,
            style,
            viewport,
            TextFieldState::default(),
            now,
        );
        let first = engine.highlight_stats();
        assert!(first.run_scans > 0);
        assert_eq!(first.highlight_calls, 0);

        engine.reset_highlight_stats();
        let scrolled_state = TextFieldState::default().with_scroll_y(line_height * 100.0);
        engine.text_area_paint_layout_for_area_at(
            &area_model,
            style,
            viewport,
            scrolled_state.clone(),
            now,
        );
        let scrolled = engine.highlight_stats();
        assert!(scrolled.run_scans > 0);
        assert_eq!(scrolled.highlight_calls, 0);

        engine.reset_highlight_stats();
        engine.text_area_paint_layout_for_area_at(
            &area_model,
            style,
            viewport,
            scrolled_state,
            now,
        );
        let cached = engine.highlight_stats();
        assert!(cached.run_scans > 0);
        assert_eq!(cached.highlight_calls, 0);
    }
    #[test]
    fn offscreen_text_area_selection_skips_run_highlight_calls() {
        let mut engine = Engine::new();
        let text = (0..1_000)
            .map(|index| format!("line {index}"))
            .collect::<Vec<_>>()
            .join("\n");
        let mut buffer = Buffer::from_multiline_text(text);
        engine.apply_text_edit(&mut buffer, Edit::set_position(TextPosition::new(0)));
        engine.apply_text_edit(&mut buffer, Edit::extend_position(TextMotion::WordNext));
        assert!(buffer.has_selection());
        let area_model = Area::new(buffer);
        let style = Style::default().with_size(13.0);
        let viewport = area::logical(240.0, 52.0);
        let state = TextFieldState::default().with_scroll_y(13.0 * 1.25 * 500.0);

        engine.reset_highlight_stats();
        let layout = engine
            .text_area_paint_layout_for_area_at(&area_model, style, viewport, state, Instant::now())
            .into_parts()
            .0;
        let stats = engine.highlight_stats();

        assert!(layout.selection_spans().is_empty());
        assert!(stats.run_scans <= TEXT_AREA_FRAME_MAX_LOGICAL_LINES);
        assert_eq!(stats.highlight_calls, 0);
        assert_eq!(stats.spans, 0);
    }

    #[test]
    fn fast_selection_check_matches_canonical_selected_range() {
        fn assert_matches(buffer: &Buffer) {
            assert_eq!(
                buffer.has_non_empty_selection(),
                buffer.selected_range().is_some()
            );
        }

        let mut engine = Engine::new();

        let mut collapsed = Buffer::from_text("abc");
        let cursor = collapsed.cursor_for_text_index(1);
        collapsed.set_cursor_and_selection(cursor, Selection::Normal(cursor));
        assert_matches(&collapsed);

        let mut single = Buffer::from_text("hello world");
        assert_matches(&single);
        engine.apply_text_edit(&mut single, Edit::SelectAll);
        assert_matches(&single);
        engine.apply_text_edit(&mut single, Edit::insert("x"));
        assert_matches(&single);

        let mut multiline = Buffer::from_multiline_text("one\ntwo\nthree");
        engine.apply_text_edit(&mut multiline, Edit::set_position(TextPosition::new(0)));
        engine.apply_text_edit(
            &mut multiline,
            Edit::extend_position(TextMotion::DocumentEnd),
        );
        assert_matches(&multiline);

        let mut word = Buffer::from_text("hello world");
        engine.apply_text_edit(
            &mut word,
            Edit::pointer(PointerEditKind::DoubleClick, TextPosition::new(1)),
        );
        assert_matches(&word);
    }
    #[test]
    fn selection_only_pointer_edits_do_not_bump_revision_or_invalidate_surfaces() {
        let mut engine = Engine::new();
        let text = (0..200)
            .map(|index| format!("line {index}"))
            .collect::<Vec<_>>()
            .join("\n");
        let mut buffer = Buffer::from_multiline_text(text);
        let area_model = Area::new(buffer.clone());
        let style = Style::default().with_size(13.0);
        let viewport = area::logical(240.0, 52.0);
        engine.text_area_paint_layout_for_area_at(
            &area_model,
            style,
            viewport,
            TextFieldState::default(),
            Instant::now(),
        );
        let revision = buffer.revision();
        let cached_frames = engine.text_area_line_displays.len();

        let set = engine.apply_text_edit_with_result(&mut buffer, Edit::set_position(0));
        assert!(set.selection_changed);
        assert!(!set.text_changed);
        assert!(set.change.is_none());
        assert_eq!(buffer.revision(), revision);
        assert_eq!(engine.text_area_line_displays.len(), cached_frames);

        let drag = engine.apply_text_edit_with_result(
            &mut buffer,
            Edit::pointer(PointerEditKind::Drag, TextPosition::new(20)),
        );
        assert!(drag.selection_changed);
        assert!(!drag.text_changed);
        assert!(drag.change.is_none());
        assert!(buffer.selected_range().is_some());
        assert_eq!(buffer.revision(), revision);
        assert_eq!(engine.text_area_line_displays.len(), cached_frames);
    }

    #[test]
    fn text_area_hit_testing_uses_nearest_caret_in_empty_space() {
        let mut engine = Engine::new();
        let buffer = Buffer::from_multiline_text("one\ntwo");
        let area_model = Area::new(buffer.clone());
        let style = Style::default().with_size(16.0);
        let viewport = area::logical(240.0, 120.0);

        let below_near_start = engine
            .text_area_position_at_for_area(
                &area_model,
                style,
                viewport,
                point::logical(-4.0, 100.0),
                TextFieldState::default(),
            )
            .expect("click below short text should resolve on the nearest line");
        assert_eq!(below_near_start.index, "one\n".len());

        let below_far_right = engine
            .text_area_position_at_for_area(
                &area_model,
                style,
                viewport,
                point::logical(220.0, 100.0),
                TextFieldState::default(),
            )
            .expect("click below short text should still honor x on the nearest line");
        assert_eq!(below_far_right.index, buffer.text().len());

        let right_of_first_line = engine
            .text_area_position_at_for_area(
                &area_model,
                style,
                viewport,
                point::logical(220.0, 8.0),
                TextFieldState::default(),
            )
            .expect("click to the right of a line should resolve to a caret");
        assert_eq!(right_of_first_line.index, "one".len());

        let above_near_start = engine
            .text_area_position_at_for_area(
                &area_model,
                style,
                viewport,
                point::logical(-4.0, -8.0),
                TextFieldState::default(),
            )
            .expect("click above text should resolve on the nearest line");
        assert_eq!(above_near_start.index, 0);

        let above_far_right = engine
            .text_area_position_at_for_area(
                &area_model,
                style,
                viewport,
                point::logical(220.0, -8.0),
                TextFieldState::default(),
            )
            .expect("click above text should still honor x on the nearest line");
        assert_eq!(above_far_right.index, "one".len());

        let empty = Area::new(Buffer::from_multiline_text(""));
        let empty_hit = engine
            .text_area_position_at_for_area(
                &empty,
                style,
                viewport,
                point::logical(12.0, 80.0),
                TextFieldState::default(),
            )
            .expect("empty text area should still resolve to a caret");
        assert_eq!(empty_hit.index, 0);
    }

    #[test]
    fn mixed_direction_line_edges_preserve_affinity_for_nearest_line_hits() {
        let mut engine = Engine::new();
        let buffer = Buffer::from_multiline_text("abc אבג\nxyz");
        let area_model = Area::new(buffer);
        let style = Style::default().with_size(18.0);
        let viewport = area::logical(280.0, 120.0);
        let display = engine.text_area_line_display(
            &area_model,
            area_model.buffer(),
            true,
            style,
            viewport,
            0,
        );
        let prepared = display.buffer.borrow();
        let map = TextLayoutMap::from_line_starts(Rc::new(vec![display.source_start]));
        let runs = prepared.layout_runs().collect::<Vec<_>>();
        assert!(
            runs.iter()
                .any(|run| run.glyphs.iter().any(|glyph| glyph.level.is_rtl()))
        );

        let mut left_edge = None::<(f32, &glyphon::cosmic_text::LayoutRun<'_>)>;
        let mut right_edge = None::<(f32, &glyphon::cosmic_text::LayoutRun<'_>)>;
        for run in &runs {
            let Some((left, right)) = TextLayoutMap::run_visual_bounds(run) else {
                continue;
            };
            if left_edge.is_none_or(|(best, _)| left < best) {
                left_edge = Some((left, run));
            }
            if right_edge.is_none_or(|(best, _)| right > best) {
                right_edge = Some((right, run));
            }
        }
        let (left, left_run) = left_edge.expect("mixed line should have a left visual edge");
        let (right, right_run) = right_edge.expect("mixed line should have a right visual edge");
        let y = runs[0].line_top - runs[0].line_height;

        let left_hit = map
            .hit(&prepared, left - 8.0, y)
            .expect("above-line left edge should resolve to a caret");
        let right_hit = map
            .hit(&prepared, right + 8.0, y)
            .expect("above-line right edge should resolve to a caret");

        assert_eq!(left_hit, map.run_edge_position(left_run, true).unwrap());
        assert_eq!(right_hit, map.run_edge_position(right_run, false).unwrap());
    }

    #[test]
    fn repeated_large_text_area_hit_tests_reuse_cached_frame() {
        let mut engine = Engine::new();
        let text = (0..5_000)
            .map(|index| format!("line {index}"))
            .collect::<Vec<_>>()
            .join("\n");
        let area_model = Area::new(Buffer::from_multiline_text(text));
        let style = Style::default().with_size(13.0);
        let viewport = area::logical(240.0, 52.0);
        let state = TextFieldState::default();

        engine.reset_interaction_stats();
        let first = engine.text_area_position_at_for_area(
            &area_model,
            style,
            viewport,
            point::logical(16.0, 18.0),
            state.clone(),
        );
        let second = engine.text_area_position_at_for_area(
            &area_model,
            style,
            viewport,
            point::logical(18.0, 18.0),
            state.clone(),
        );
        let stats = engine.interaction_stats();

        assert!(first.is_some());
        assert!(second.is_some());
        assert!(stats.text_area_frame_cache_misses <= TEXT_AREA_FRAME_MAX_LOGICAL_LINES);
        assert!(stats.text_area_frame_cache_hits > 0);
        assert!(stats.text_area_frame_shape_calls <= TEXT_AREA_FRAME_MAX_LOGICAL_LINES);
        assert!(stats.text_area_frame_shaped_logical_lines <= TEXT_AREA_FRAME_MAX_LOGICAL_LINES);
        assert_eq!(stats.text_area_shape_until_scroll_calls, 0);
        assert!(stats.hit_run_scans <= stats.text_area_frame_shaped_visual_lines * 2);
    }

    #[test]
    fn warmed_large_text_area_hit_test_does_not_reshape_visible_window() {
        let mut engine = Engine::new();
        let text = (0..5_000)
            .map(|index| format!("line {index}"))
            .collect::<Vec<_>>()
            .join("\n");
        let area_model = Area::new(Buffer::from_multiline_text(text));
        let style = Style::default().with_size(13.0);
        let viewport = area::logical(240.0, 52.0);
        let state = TextFieldState::default();

        let _ = engine.text_area_position_at_for_area(
            &area_model,
            style,
            viewport,
            point::logical(16.0, 18.0),
            state.clone(),
        );
        engine.reset_interaction_stats();
        let hit = engine.text_area_position_at_for_area(
            &area_model,
            style,
            viewport,
            point::logical(20.0, 18.0),
            state,
        );
        let stats = engine.interaction_stats();

        assert!(hit.is_some());
        assert_eq!(stats.text_area_shape_until_scroll_calls, 0);
        assert!(stats.text_area_frame_cache_hits > 0);
        assert_eq!(stats.text_area_frame_cache_misses, 0);
        assert_eq!(stats.text_area_frame_shape_calls, 0);
        assert!(stats.hit_run_scans <= TEXT_AREA_FRAME_MAX_LOGICAL_LINES);
    }

    #[test]
    fn text_area_preedit_reveal_scroll_uses_composed_projection() {
        let mut engine = Engine::new();
        let mut buffer = Buffer::from_multiline_text("one\ntwo");
        let end = buffer.position_for_text_index(buffer.text().len());
        engine.apply_text_edit(&mut buffer, Edit::set_position(end));
        let area_model = Area::new(buffer);
        let style = Style::default().with_size(16.0);
        let viewport = area::logical(120.0, 36.0);
        let state = TextFieldState::default()
            .with_preedit(Some(Preedit::new("\nthree\nfour\nfive\nsix", None)));

        let revealed =
            engine.text_area_reveal_scroll_for_area(&area_model, style, viewport, state.clone());
        let layout = engine
            .text_area_paint_layout_for_area_at(
                &area_model,
                style,
                viewport,
                revealed.clone(),
                Instant::now(),
            )
            .into_parts()
            .0;

        assert!(revealed.scroll_y() > 0.0);
        assert!(!layout.preedit_underline_spans().is_empty());
    }

    #[test]
    fn obscured_text_field_hit_testing_maps_display_cursor_to_source_cursor() {
        let mut engine = Engine::new();
        let field = Field::new("åb").obscured_dot();
        let position = engine
            .text_field_position_at_for_field(
                &field,
                Style::default().with_size(16.0),
                area::logical(200.0, 24.0),
                point::logical(200.0, 8.0),
                TextFieldState::default(),
            )
            .expect("hit testing should return a position");

        assert_eq!(field.presentation_text(), "••");
        assert_eq!(field.buffer().text(), "åb");
        assert_eq!(position.index, field.buffer().text().len());
    }

    #[test]
    fn text_field_reveal_scroll_keeps_caret_inside_content_rect() {
        let mut engine = Engine::new();
        let buffer = Buffer::from_text("hello world this is a long single-line field");
        let area = area::logical(80.0, 32.0);
        let state = engine.text_field_reveal_scroll(
            &buffer,
            Style::default().with_size(16.0),
            area,
            TextFieldState::default(),
        );

        assert!(state.scroll_x() > 0.0);

        let layout =
            engine.text_field_layout(&buffer, Style::default().with_size(16.0), area, state);
        let caret = layout.caret().expect("focused long text should have caret");

        assert!(caret.x() >= 0.0);
        assert!(caret.x() <= area.width());
    }

    #[test]
    fn text_field_caret_visibility_follows_blink_phase() {
        let mut engine = Engine::new();
        let buffer = Buffer::from_text("hello");
        let area = area::logical(100.0, 24.0);
        let epoch = Instant::now();
        let state = TextFieldState::new_at(0.0, epoch);

        let visible = engine.text_field_layout_at(
            &buffer,
            Style::default().with_size(16.0),
            area,
            state.clone(),
            epoch,
        );
        let hidden = engine.text_field_layout_at(
            &buffer,
            Style::default().with_size(16.0),
            area,
            state.clone(),
            epoch + Duration::from_millis(500),
        );
        let visible_again = engine.text_field_layout_at(
            &buffer,
            Style::default().with_size(16.0),
            area,
            state,
            epoch + Duration::from_millis(1000),
        );

        assert!(visible.caret().is_some());
        assert_eq!(hidden.caret(), None);
        assert!(visible_again.caret().is_some());
    }

    #[test]
    fn text_field_selection_suppresses_caret_layout() {
        let mut engine = Engine::new();
        let mut buffer = Buffer::from_text("hello");
        let area = area::logical(100.0, 24.0);
        let epoch = Instant::now();

        engine.apply_text_edit(&mut buffer, Edit::SelectAll);

        let layout = engine.text_field_layout_at(
            &buffer,
            Style::default().with_size(16.0),
            area,
            TextFieldState::new_at(0.0, epoch),
            epoch,
        );

        assert_eq!(layout.caret(), None);
        assert!(!layout.selection_spans().is_empty());
    }

    #[test]
    fn multiline_buffer_preserves_line_breaks_and_enter_inserts_newline() {
        let mut engine = Engine::new();
        let mut buffer = Buffer::from_multiline_text("one\r\ntwo\rthree");

        assert_eq!(buffer.text(), "one\ntwo\nthree");

        let end = buffer.position_for_text_index(buffer.text().len());
        engine.apply_text_edit(&mut buffer, Edit::set_position(end));
        engine.apply_text_edit(&mut buffer, Edit::insert_line_break());
        engine.apply_text_edit(&mut buffer, Edit::insert("four\nfive"));

        assert_eq!(buffer.text(), "one\ntwo\nthree\nfour\nfive");
    }

    #[test]
    fn multiline_select_all_selects_the_entire_document() {
        let mut engine = Engine::new();
        let mut buffer = Buffer::from_multiline_text("alpha\nbeta\ngamma");

        engine.apply_text_edit(&mut buffer, Edit::SelectAll);

        assert_eq!(
            buffer.selected_text(),
            Some("alpha\nbeta\ngamma".to_owned())
        );
        assert_eq!(
            buffer.selected_range(),
            Some(TextRange::new(0, "alpha\nbeta\ngamma".len()))
        );
    }

    #[test]
    fn text_area_reveal_scroll_uses_wrapped_visual_caret_row() {
        let mut engine = Engine::new();
        let text = "alpha beta gamma delta epsilon zeta eta theta iota kappa lambda mu";
        let mut buffer = Buffer::from_multiline_text(text);
        let area_model = Area::new(buffer.clone());
        let style = Style::default().with_size(16.0);
        let viewport = area::logical(86.0, 24.0);
        let (cursor_index, second_row_top, row_height) = {
            let display = engine.text_area_line_display(
                &area_model,
                area_model.buffer(),
                true,
                style,
                viewport,
                0,
            );
            let prepared = display.buffer.borrow();
            let runs = prepared.layout_runs().collect::<Vec<_>>();
            let groups = TextLayoutMap::visual_line_groups(&runs);
            assert!(
                groups.len() >= 2,
                "test text should wrap into at least two visual rows"
            );
            let second_run = &runs[groups[1].start];
            let first_glyph = second_run
                .glyphs
                .first()
                .expect("second visual row should have a first glyph");
            (
                display.source_start + first_glyph.start,
                groups[1].top,
                second_run.line_height,
            )
        };

        engine.apply_text_edit(
            &mut buffer,
            Edit::set_position(TextPosition::new(cursor_index)),
        );
        let area_model = Area::new(buffer);
        let scroll_y = (second_row_top - 2.0).max(0.0);
        let state = TextFieldState::default().with_scroll_y(scroll_y);
        let revealed = engine.text_area_reveal_scroll_for_area(&area_model, style, viewport, state);

        assert!(
            (revealed.scroll_y() - scroll_y).abs() <= TEXT_LAYOUT_VISUAL_LINE_EPSILON,
            "visible wrapped-row caret should not reveal to hard-line top: before {scroll_y}, after {}",
            revealed.scroll_y()
        );

        let layout = engine
            .text_area_paint_layout_for_area_at(
                &area_model,
                style,
                area::logical(viewport.width(), row_height + 4.0),
                revealed,
                Instant::now(),
            )
            .into_parts()
            .0;
        let caret = layout.caret().expect("wrapped row caret should be visible");
        assert!(caret.y() >= 0.0);
    }
    #[test]
    fn text_area_reveal_scroll_keeps_caret_inside_vertical_viewport() {
        let mut engine = Engine::new();
        let mut buffer = Buffer::from_multiline_text("one\ntwo\nthree\nfour\nfive\nsix");
        let end = buffer.position_for_text_index(buffer.text().len());
        engine.apply_text_edit(&mut buffer, Edit::set_position(end));

        let area_model = Area::new(buffer);
        let viewport = area::logical(120.0, 36.0);
        let state = engine.text_area_reveal_scroll_for_area(
            &area_model,
            Style::default().with_size(16.0),
            viewport,
            TextFieldState::default(),
        );

        assert!(state.scroll_y() > 0.0);

        let paint_layout = engine.text_area_paint_layout_for_area_at(
            &area_model,
            Style::default().with_size(16.0),
            viewport,
            state,
            Instant::now(),
        );
        let caret = paint_layout
            .layout()
            .caret()
            .expect("area caret should be visible");

        assert!(caret.y() >= 0.0);
        assert!(caret.y() + caret.height() <= viewport.height() + TEXT_FIELD_CARET_MARGIN);
    }

    #[test]
    fn text_area_if_needed_reveal_preserves_visible_caret_scroll_after_backspace() {
        let mut engine = Engine::new();
        let text = (0..40)
            .map(|line| format!("line {line:02} abc"))
            .collect::<Vec<_>>()
            .join("\n");
        let mut buffer = Buffer::from_multiline_text(text.clone());
        let cursor_index = text.find("line 20 abc").unwrap() + "line 20 abc".len();
        engine.apply_text_edit(
            &mut buffer,
            Edit::set_position(TextPosition::new(cursor_index)),
        );
        engine.apply_text_edit(&mut buffer, Edit::backspace());

        let area_model = Area::new(buffer);
        let style = Style::default().with_size(16.0);
        let viewport = area::logical(200.0, 64.0);
        let scroll_y = text_area_estimated_line_height(style) * 18.0;
        let state = TextFieldState::default()
            .with_scroll_y(scroll_y)
            .reset_caret_blink_if_needed(Instant::now());

        let revealed = engine.text_area_reveal_scroll_for_area(&area_model, style, viewport, state);

        assert!(
            (revealed.scroll_y() - scroll_y).abs() <= TEXT_LAYOUT_VISUAL_LINE_EPSILON,
            "visible caret should preserve scroll after backspace: before {scroll_y}, after {}",
            revealed.scroll_y()
        );

        let layout = engine.text_area_paint_layout_for_area_at(
            &area_model,
            style,
            viewport,
            revealed,
            Instant::now(),
        );
        let caret = layout
            .layout()
            .caret()
            .expect("caret should remain visible after preserving scroll");
        assert!(caret.y() >= -TEXT_FIELD_CARET_MARGIN);
        assert!(caret.y() + caret.height() <= viewport.height() + TEXT_FIELD_CARET_MARGIN);
    }

    #[test]
    fn text_area_if_needed_reveal_scrolls_hidden_caret_into_view() {
        let mut engine = Engine::new();
        let text = (0..40)
            .map(|line| format!("line {line:02}"))
            .collect::<Vec<_>>()
            .join("\n");
        let mut buffer = Buffer::from_multiline_text(text.clone());
        let cursor_index = text.find("line 30").unwrap() + "line 30".len();
        engine.apply_text_edit(
            &mut buffer,
            Edit::set_position(TextPosition::new(cursor_index)),
        );

        let area_model = Area::new(buffer);
        let style = Style::default().with_size(16.0);
        let viewport = area::logical(200.0, 64.0);
        let state = TextFieldState::default().reset_caret_blink_if_needed(Instant::now());

        let revealed = engine.text_area_reveal_scroll_for_area(&area_model, style, viewport, state);

        assert!(revealed.scroll_y() > 0.0);
        let layout = engine.text_area_paint_layout_for_area_at(
            &area_model,
            style,
            viewport,
            revealed,
            Instant::now(),
        );
        let caret = layout
            .layout()
            .caret()
            .expect("hidden caret should be revealed into the painted viewport");
        assert!(caret.y() >= -TEXT_FIELD_CARET_MARGIN);
        assert!(caret.y() + caret.height() <= viewport.height() + TEXT_FIELD_CARET_MARGIN);
    }
    fn styled_document(text: &str, align: Align, size: f32, weight: Weight) -> Document {
        let mut block = Block::new(align);
        block.push_run(Run::new(
            text,
            Style::default().with_size(size).with_weight(weight),
        ));

        Document::from_block(block)
    }
}
