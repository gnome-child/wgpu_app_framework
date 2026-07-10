use std::{rc::Rc, sync::Arc};

use unicode_segmentation::UnicodeSegmentation;

use super::super::{LineId, next_line_id};
use super::source::{TextPiece, TextPieceSource};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum TextLineEnding {
    None,
    Lf,
}

impl TextLineEnding {
    pub(super) fn as_str(self) -> &'static str {
        match self {
            Self::None => "",
            Self::Lf => "\n",
        }
    }

    pub(super) fn len(self) -> usize {
        self.as_str().len()
    }
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
pub(in crate::text::buffer) struct TextLine {
    pub(super) id: LineId,
    pub(super) revision: u64,
    pieces: Vec<TextPiece>,
    pub(in crate::text::buffer) text: Arc<str>,
    pub(super) ending: TextLineEnding,
    grapheme_boundaries: Rc<Vec<usize>>,
    dirty: bool,
}

impl TextLine {
    pub(super) fn new(text: impl Into<String>, ending: TextLineEnding) -> Self {
        Self::from_cached_text(text.into(), ending, Vec::new(), 0)
    }

    pub(super) fn from_piece(
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
        let text: Arc<str> = Arc::from(text);
        let mut line = Self {
            id: next_line_id(),
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

    pub(super) fn with_revision(mut self, revision: u64) -> Self {
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

    pub(super) fn total_len(&self) -> usize {
        self.text.len() + self.ending.len()
    }

    pub(super) fn floor_grapheme(&self, local: usize) -> usize {
        let local = local.min(self.text.len());
        let index = self
            .grapheme_boundaries
            .partition_point(|boundary| *boundary <= local)
            .saturating_sub(1);
        self.grapheme_boundaries.get(index).copied().unwrap_or(0)
    }

    pub(super) fn ceil_grapheme(&self, local: usize) -> usize {
        let local = local.min(self.text.len());
        self.grapheme_boundaries
            .iter()
            .copied()
            .find(|boundary| *boundary >= local)
            .unwrap_or(self.text.len())
    }

    #[cfg(test)]
    pub(super) fn piece_source_lengths(&self) -> (usize, usize, usize) {
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
pub(in crate::text::buffer) struct TextLineBlock {
    pub(in crate::text::buffer) lines: Arc<[TextLine]>,
    summary: TextSummary,
}

impl TextLineBlock {
    fn new(lines: Vec<TextLine>) -> Self {
        let summary = TextSummary::from_lines(&lines);
        Self {
            lines: Arc::from(lines),
            summary,
        }
    }

    fn line_count(&self) -> usize {
        self.lines.len()
    }
}

pub(in crate::text) const TEXT_DOCUMENT_BLOCK_TARGET_LINES: usize = 128;
const TEXT_DOCUMENT_BLOCK_MAX_LINES: usize = 256;

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub(in crate::text::buffer) struct TextLineTree {
    pub(in crate::text::buffer) blocks: Vec<TextLineBlock>,
    summary: TextSummary,
}

impl TextLineTree {
    pub(super) fn from_lines(lines: Vec<TextLine>) -> Self {
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

    pub(super) fn line_count(&self) -> usize {
        self.summary.line_count.max(1)
    }

    pub(super) fn text_len(&self) -> usize {
        self.summary.utf8_len
    }

    pub(in crate::text::buffer) fn line(&self, line: usize) -> Option<&TextLine> {
        let (block, local) = self.locate_line(line);
        self.blocks
            .get(block)
            .and_then(|block| block.lines.get(local))
    }

    pub(super) fn line_start(&self, line: usize) -> usize {
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

    pub(super) fn line_and_local_for_index(&self, index: usize) -> (usize, usize) {
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
            for line in block.lines.iter() {
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

    pub(super) fn splice_lines(
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

    pub(in crate::text::buffer) fn locate_line(&self, line: usize) -> (usize, usize) {
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
            let mut lines = self.blocks[index].lines.to_vec();
            let tail = lines.split_off(TEXT_DOCUMENT_BLOCK_TARGET_LINES);
            let head = lines;
            self.blocks[index] = TextLineBlock::new(head);
            self.blocks.insert(index + 1, TextLineBlock::new(tail));
            index += 1;
        }
    }
}
