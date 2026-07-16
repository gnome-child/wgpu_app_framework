use std::{ops::Range, sync::Arc};

use super::constants::TEXT_AREA_RENDER_HORIZONTAL_OVERSCAN;

pub(super) const TEXT_AREA_HORIZONTAL_INDEX_MAX_SOURCE_SPAN: u32 = 4_096;
const TEXT_AREA_HORIZONTAL_INDEX_TARGET_SOURCE_SPAN: u32 = 256;
pub(super) const TEXT_AREA_HORIZONTAL_EXACT_BAND_MAX_SOURCE_SPAN: usize = 262_144;
const F32_EXACT_INTEGRAL_LIMIT: f64 = 16_777_216.0;
const CHECKPOINT_BLOCK_SEGMENTS: usize = 256;

pub(super) fn prepared_window(viewport_width: f32, scroll_x: f64) -> (f64, f32) {
    let viewport_width = viewport_width.max(1.0);
    let overscan = TEXT_AREA_RENDER_HORIZONTAL_OVERSCAN.max(1.0);
    let width = viewport_width + overscan * 2.0;
    let overscan_exact = f64::from(overscan);
    let desired_origin = (scroll_x - overscan_exact).max(0.0);
    let origin = (desired_origin / overscan_exact).floor() * overscan_exact;
    (origin, width)
}

#[derive(Debug, Clone, Copy, PartialEq)]
struct Checkpoint {
    source_byte: u32,
    x: f64,
}

#[derive(Debug, Clone, PartialEq)]
pub(super) struct Window {
    pub(super) source: Range<usize>,
    pub(super) x: f64,
    pub(super) width: f64,
}

#[derive(Debug, Clone)]
pub(super) struct LineIndex {
    blocks: Vec<Arc<CheckpointBlock>>,
    checkpoint_count: usize,
    source_len: usize,
    width: f64,
    height: f32,
    stable_coordinates: bool,
}

#[derive(Debug)]
struct CheckpointBlock {
    source_bytes: Box<[u32]>,
    xs: Box<[f64]>,
}

pub(super) struct LineIndexBuilder {
    source_bytes: Vec<u32>,
    xs: Vec<f64>,
    source_len: usize,
    width: f64,
    height: f32,
    band_count: usize,
    stable_xs: Vec<f64>,
    stable_width: f64,
}

pub(super) struct EditWindow {
    pub(super) new_source: Range<usize>,
    old_source: Range<usize>,
    start_segment: usize,
    end_segment: usize,
}

impl LineIndexBuilder {
    pub(super) fn new() -> Self {
        Self {
            source_bytes: Vec::new(),
            xs: Vec::new(),
            source_len: 0,
            width: 0.0,
            height: 1.0,
            band_count: 0,
            stable_xs: Vec::new(),
            stable_width: 0.0,
        }
    }

    pub(super) fn push(&mut self, band: LineIndex, stable_xs: Vec<f64>) -> Option<()> {
        let band_source_len = u32::try_from(band.source_len).ok()?;
        let checkpoints = band.absolute_checkpoints();
        if checkpoints.first().map(|checkpoint| checkpoint.source_byte) != Some(0)
            || checkpoints.last().map(|checkpoint| checkpoint.source_byte) != Some(band_source_len)
            || stable_xs.len() != checkpoints.len()
        {
            return None;
        }
        let stable_width = stable_xs.last().copied()?;
        if stable_xs.first().copied() != Some(0.0)
            || stable_xs.windows(2).any(|pair| pair[0] > pair[1])
            || !stable_width.is_finite()
        {
            return None;
        }
        let source_start = u32::try_from(self.source_len).ok()?;
        for (index, (checkpoint, stable_x)) in checkpoints.into_iter().zip(stable_xs).enumerate() {
            if self.band_count > 0 && index == 0 {
                continue;
            }
            self.source_bytes
                .push(source_start.checked_add(checkpoint.source_byte)?);
            self.xs.push(self.width + checkpoint.x);
            self.stable_xs.push(self.stable_width + stable_x);
        }
        self.source_len = self.source_len.checked_add(band.source_len)?;
        self.width += band.width;
        self.height = self.height.max(band.height);
        self.band_count += 1;
        self.stable_width += stable_width;
        Some(())
    }

    pub(super) fn finish(self, expected_source_len: usize) -> Option<(LineIndex, usize)> {
        if self.source_len != expected_source_len
            || self.source_bytes.len() < 3
            || self.source_bytes.len() != self.xs.len()
            || self.source_bytes.windows(2).any(|pair| {
                pair[0] >= pair[1] || pair[1] - pair[0] > TEXT_AREA_HORIZONTAL_INDEX_MAX_SOURCE_SPAN
            })
            || self.xs.windows(2).any(|pair| pair[0] > pair[1])
        {
            return None;
        }
        let use_stable = self.stable_width >= F32_EXACT_INTEGRAL_LIMIT;
        let xs = if use_stable { self.stable_xs } else { self.xs };
        let width = if use_stable {
            self.stable_width
        } else {
            self.width
        };
        let index = LineIndex::from_absolute_parts(
            self.source_bytes,
            xs,
            self.source_len,
            width,
            self.height,
            use_stable,
        )?;
        Some((index, self.band_count))
    }
}

impl CheckpointBlock {
    fn from_absolute(checkpoints: &[Checkpoint]) -> Option<Self> {
        let first = *checkpoints.first()?;
        let last = *checkpoints.last()?;
        if checkpoints.len() < 2
            || checkpoints.windows(2).any(|pair| {
                pair[0].source_byte >= pair[1].source_byte
                    || pair[0].x > pair[1].x
                    || pair[1].source_byte - pair[0].source_byte
                        > TEXT_AREA_HORIZONTAL_INDEX_MAX_SOURCE_SPAN
            })
        {
            return None;
        }
        let source_bytes = checkpoints
            .iter()
            .map(|checkpoint| checkpoint.source_byte.checked_sub(first.source_byte))
            .collect::<Option<Vec<_>>>()?;
        let xs = checkpoints
            .iter()
            .map(|checkpoint| checkpoint.x - first.x)
            .collect::<Vec<_>>();
        if source_bytes.first().copied() != Some(0)
            || xs.first().copied() != Some(0.0)
            || source_bytes.last().copied()? != last.source_byte - first.source_byte
            || !xs.last().copied()?.is_finite()
        {
            return None;
        }
        Some(Self {
            source_bytes: source_bytes.into_boxed_slice(),
            xs: xs.into_boxed_slice(),
        })
    }

    fn segment_count(&self) -> usize {
        self.source_bytes.len().saturating_sub(1)
    }

    fn source_len(&self) -> usize {
        self.source_bytes.last().copied().unwrap_or_default() as usize
    }

    fn width(&self) -> f64 {
        self.xs.last().copied().unwrap_or_default()
    }

    fn resident_bytes(&self) -> usize {
        self.source_bytes
            .len()
            .saturating_mul(std::mem::size_of::<u32>())
            .saturating_add(self.xs.len().saturating_mul(std::mem::size_of::<f64>()))
    }

    fn checkpoint(&self, index: usize) -> Option<Checkpoint> {
        Some(Checkpoint {
            source_byte: *self.source_bytes.get(index)?,
            x: *self.xs.get(index)?,
        })
    }

    fn segment_slice(&self, segments: Range<usize>) -> Option<Arc<Self>> {
        if segments.start >= segments.end || segments.end > self.segment_count() {
            return None;
        }
        let checkpoints = (segments.start..=segments.end)
            .map(|index| self.checkpoint(index))
            .collect::<Option<Vec<_>>>()?;
        Some(Arc::new(Self::from_absolute(&checkpoints)?))
    }

    fn joined(left: &Self, right: &Self) -> Option<Vec<Arc<Self>>> {
        let mut checkpoints = (0..=left.segment_count())
            .map(|index| left.checkpoint(index))
            .collect::<Option<Vec<_>>>()?;
        let source_origin = u32::try_from(left.source_len()).ok()?;
        let x_origin = left.width();
        checkpoints.extend((1..=right.segment_count()).filter_map(|index| {
            let checkpoint = right.checkpoint(index)?;
            Some(Checkpoint {
                source_byte: source_origin.checked_add(checkpoint.source_byte)?,
                x: x_origin + checkpoint.x,
            })
        }));
        LineIndex::blocks_from_absolute(&checkpoints)
    }
}

impl LineIndex {
    fn from_absolute_parts(
        source_bytes: Vec<u32>,
        xs: Vec<f64>,
        source_len: usize,
        width: f64,
        height: f32,
        stable_coordinates: bool,
    ) -> Option<Self> {
        if source_bytes.len() != xs.len() {
            return None;
        }
        let checkpoints = source_bytes
            .into_iter()
            .zip(xs)
            .map(|(source_byte, x)| Checkpoint { source_byte, x })
            .collect::<Vec<_>>();
        Self::from_absolute_checkpoints(checkpoints, source_len, width, height, stable_coordinates)
    }

    fn from_absolute_checkpoints(
        checkpoints: Vec<Checkpoint>,
        source_len: usize,
        width: f64,
        height: f32,
        stable_coordinates: bool,
    ) -> Option<Self> {
        if checkpoints.len() < 3
            || checkpoints.first().copied()
                != Some(Checkpoint {
                    source_byte: 0,
                    x: 0.0,
                })
            || checkpoints.last()?.source_byte as usize != source_len
            || !width.is_finite()
            || checkpoints.windows(2).any(|pair| {
                pair[0].source_byte >= pair[1].source_byte
                    || pair[0].x > pair[1].x
                    || pair[1].source_byte - pair[0].source_byte
                        > TEXT_AREA_HORIZONTAL_INDEX_MAX_SOURCE_SPAN
            })
        {
            return None;
        }
        let blocks = Self::blocks_from_absolute(&checkpoints)?;
        Some(Self {
            blocks,
            checkpoint_count: checkpoints.len(),
            source_len,
            width,
            height: height.max(1.0),
            stable_coordinates,
        })
    }

    fn blocks_from_absolute(checkpoints: &[Checkpoint]) -> Option<Vec<Arc<CheckpointBlock>>> {
        let terminal = checkpoints.len().checked_sub(1)?;
        if terminal == 0 {
            return None;
        }
        let mut blocks = Vec::with_capacity(terminal.div_ceil(CHECKPOINT_BLOCK_SEGMENTS));
        let mut start = 0_usize;
        while start < terminal {
            let end = start
                .saturating_add(CHECKPOINT_BLOCK_SEGMENTS)
                .min(terminal);
            blocks.push(Arc::new(CheckpointBlock::from_absolute(
                &checkpoints[start..=end],
            )?));
            start = end;
        }
        Some(blocks)
    }

    fn absolute_checkpoints(&self) -> Vec<Checkpoint> {
        let mut checkpoints = Vec::with_capacity(self.checkpoint_count);
        let mut source_origin = 0_u32;
        let mut x_origin = 0.0_f64;
        for (block_index, block) in self.blocks.iter().enumerate() {
            for index in usize::from(block_index > 0)..=block.segment_count() {
                let Some(checkpoint) = block.checkpoint(index) else {
                    continue;
                };
                let Some(source_byte) = source_origin.checked_add(checkpoint.source_byte) else {
                    continue;
                };
                checkpoints.push(Checkpoint {
                    source_byte,
                    x: x_origin + checkpoint.x,
                });
            }
            source_origin = source_origin.saturating_add(block.source_len() as u32);
            x_origin += block.width();
        }
        checkpoints
    }

    fn segment_count(&self) -> usize {
        self.checkpoint_count.saturating_sub(1)
    }

    fn checkpoint(&self, index: usize) -> Option<Checkpoint> {
        if index >= self.checkpoint_count {
            return None;
        }
        let mut remaining = index;
        let mut source_origin = 0_u32;
        let mut x_origin = 0.0_f64;
        for block in &self.blocks {
            let segments = block.segment_count();
            if remaining <= segments {
                let checkpoint = block.checkpoint(remaining)?;
                return Some(Checkpoint {
                    source_byte: source_origin.checked_add(checkpoint.source_byte)?,
                    x: x_origin + checkpoint.x,
                });
            }
            remaining -= segments;
            source_origin = source_origin.checked_add(u32::try_from(block.source_len()).ok()?)?;
            x_origin += block.width();
        }
        None
    }

    fn segment_at_source(&self, source_byte: usize) -> usize {
        let source_byte = source_byte.min(self.source_len);
        let mut source_origin = 0_usize;
        let mut segment_origin = 0_usize;
        for (index, block) in self.blocks.iter().enumerate() {
            let block_end = source_origin.saturating_add(block.source_len());
            if source_byte < block_end || index + 1 == self.blocks.len() {
                let local =
                    u32::try_from(source_byte.saturating_sub(source_origin)).unwrap_or(u32::MAX);
                let segment = block
                    .source_bytes
                    .partition_point(|checkpoint| *checkpoint <= local)
                    .saturating_sub(1)
                    .min(block.segment_count().saturating_sub(1));
                return segment_origin.saturating_add(segment);
            }
            source_origin = block_end;
            segment_origin = segment_origin.saturating_add(block.segment_count());
        }
        self.segment_count().saturating_sub(1)
    }

    fn segment_at_x(&self, x: f64) -> usize {
        let x = x.max(0.0).min(self.width);
        let mut x_origin = 0.0_f64;
        let mut segment_origin = 0_usize;
        for (index, block) in self.blocks.iter().enumerate() {
            let block_end = x_origin + block.width();
            if x < block_end || index + 1 == self.blocks.len() {
                let local = x - x_origin;
                let segment = block
                    .xs
                    .partition_point(|checkpoint| *checkpoint <= local)
                    .saturating_sub(1)
                    .min(block.segment_count().saturating_sub(1));
                return segment_origin.saturating_add(segment);
            }
            x_origin = block_end;
            segment_origin = segment_origin.saturating_add(block.segment_count());
        }
        self.segment_count().saturating_sub(1)
    }

    fn segment_window(&self, segment: usize) -> Option<Window> {
        if segment >= self.segment_count() {
            return None;
        }
        let mut remaining = segment;
        let mut source_origin = 0_usize;
        let mut x_origin = 0.0_f64;
        for block in &self.blocks {
            let segments = block.segment_count();
            if remaining < segments {
                let start = block.checkpoint(remaining)?;
                let end = block.checkpoint(remaining + 1)?;
                return Some(Window {
                    source: source_origin.saturating_add(start.source_byte as usize)
                        ..source_origin.saturating_add(end.source_byte as usize),
                    x: x_origin + start.x,
                    width: (end.x - start.x).max(0.0),
                });
            }
            remaining -= segments;
            source_origin = source_origin.saturating_add(block.source_len());
            x_origin += block.width();
        }
        None
    }

    fn append_segment_range(
        &self,
        target: &mut Vec<Arc<CheckpointBlock>>,
        segments: Range<usize>,
    ) -> Option<()> {
        if segments.start > segments.end || segments.end > self.segment_count() {
            return None;
        }
        let mut block_start = 0_usize;
        for block in &self.blocks {
            let block_end = block_start.saturating_add(block.segment_count());
            let start = segments.start.max(block_start);
            let end = segments.end.min(block_end);
            if start < end {
                let projected = if start == block_start && end == block_end {
                    Arc::clone(block)
                } else {
                    block.segment_slice(start - block_start..end - block_start)?
                };
                Self::push_compact_block(target, projected)?;
            }
            if block_end >= segments.end {
                break;
            }
            block_start = block_end;
        }
        Some(())
    }

    fn push_compact_block(
        target: &mut Vec<Arc<CheckpointBlock>>,
        block: Arc<CheckpointBlock>,
    ) -> Option<()> {
        let Some(last) = target.last() else {
            target.push(block);
            return Some(());
        };
        if last.segment_count().saturating_add(block.segment_count()) > CHECKPOINT_BLOCK_SEGMENTS {
            target.push(block);
            return Some(());
        }
        let last = target.pop()?;
        target.extend(CheckpointBlock::joined(&last, &block)?);
        Some(())
    }

    pub(super) fn from_ltr_buffer(text: &str, buffer: &glyphon::Buffer) -> Option<Self> {
        Self::from_ltr_buffer_inner(text, buffer, true)
    }

    pub(super) fn from_ltr_fragment_buffer(text: &str, buffer: &glyphon::Buffer) -> Option<Self> {
        Self::from_ltr_buffer_inner(text, buffer, false)
    }

    fn from_ltr_buffer_inner(
        text: &str,
        buffer: &glyphon::Buffer,
        require_multiple_windows: bool,
    ) -> Option<Self> {
        if text.is_empty() {
            return None;
        }
        let source_len = u32::try_from(text.len()).ok()?;

        let mut runs = buffer.layout_runs();
        let run = runs.next()?;
        if runs.next().is_some() || run.rtl || run.text != text || run.line_w <= 0.0 {
            return None;
        }

        let mut checkpoints = vec![Checkpoint {
            source_byte: 0,
            x: 0.0,
        }];
        let mut last_x = 0.0_f32;
        let mut last_source_start = 0_usize;
        let mut terminal_x = run.line_w;
        for glyph in run.glyphs {
            if glyph.end < glyph.start
                || glyph.end > text.len()
                || glyph.x < last_x
                || glyph.start < last_source_start
            {
                return None;
            }
            last_x = glyph.x;
            last_source_start = glyph.start;
            terminal_x = terminal_x.max(glyph.x + glyph.w);
        }

        let mut previous_boundary = None::<Checkpoint>;
        let mut glyph_index = 0_usize;
        for (source_byte, _) in unicode_linebreak::linebreaks(text) {
            if source_byte == 0 || source_byte >= text.len() {
                continue;
            }
            while glyph_index < run.glyphs.len() && run.glyphs[glyph_index].end <= source_byte {
                glyph_index += 1;
            }
            let Some(glyph) = run.glyphs.get(glyph_index) else {
                continue;
            };
            if glyph.start < source_byte {
                continue;
            }
            let boundary = Checkpoint {
                source_byte: u32::try_from(source_byte).ok()?,
                x: f64::from(glyph.x),
            };
            let last = *checkpoints.last()?;
            if boundary.source_byte.saturating_sub(last.source_byte)
                > TEXT_AREA_HORIZONTAL_INDEX_MAX_SOURCE_SPAN
            {
                let candidate = previous_boundary?;
                if candidate.source_byte <= last.source_byte
                    || boundary.source_byte.saturating_sub(candidate.source_byte)
                        > TEXT_AREA_HORIZONTAL_INDEX_MAX_SOURCE_SPAN
                {
                    return None;
                }
                checkpoints.push(candidate);
            }
            if boundary
                .source_byte
                .saturating_sub(checkpoints.last()?.source_byte)
                >= TEXT_AREA_HORIZONTAL_INDEX_TARGET_SOURCE_SPAN
            {
                checkpoints.push(boundary);
            }
            previous_boundary = Some(boundary);
        }

        let terminal = Checkpoint {
            source_byte: source_len,
            x: f64::from(terminal_x).max(checkpoints.last()?.x),
        };
        if terminal
            .source_byte
            .saturating_sub(checkpoints.last()?.source_byte)
            > TEXT_AREA_HORIZONTAL_INDEX_MAX_SOURCE_SPAN
        {
            let candidate = previous_boundary?;
            if candidate.source_byte <= checkpoints.last()?.source_byte
                || terminal.source_byte.saturating_sub(candidate.source_byte)
                    > TEXT_AREA_HORIZONTAL_INDEX_MAX_SOURCE_SPAN
            {
                return None;
            }
            checkpoints.push(candidate);
        }
        if checkpoints
            .last()
            .is_none_or(|last| last.source_byte != terminal.source_byte || last.x != terminal.x)
        {
            checkpoints.push(terminal);
        }
        if checkpoints.len() < if require_multiple_windows { 3 } else { 2 }
            || checkpoints.windows(2).any(|pair| {
                pair[0].source_byte >= pair[1].source_byte
                    || pair[0].x > pair[1].x
                    || pair[1].source_byte - pair[0].source_byte
                        > TEXT_AREA_HORIZONTAL_INDEX_MAX_SOURCE_SPAN
            })
        {
            return None;
        }

        Self::from_absolute_checkpoints(
            checkpoints,
            text.len(),
            f64::from(run.line_w),
            run.line_height,
            false,
        )
    }

    pub(super) fn stable_xs(&self, buffer: &glyphon::Buffer) -> Option<Vec<f64>> {
        let shape = buffer.lines.first()?.shape_opt()?;
        if shape.rtl || shape.spans.iter().any(|span| span.level.is_rtl()) {
            return None;
        }
        let font_size = buffer.metrics().font_size;
        let source_bytes = self
            .absolute_checkpoints()
            .into_iter()
            .map(|checkpoint| checkpoint.source_byte)
            .collect::<Vec<_>>();
        let mut stable_xs = Vec::with_capacity(source_bytes.len());
        let mut checkpoint = 0_usize;
        let mut exact_x = 0.0_f64;
        for span in &shape.spans {
            for word in &span.words {
                for glyph in &word.glyphs {
                    while checkpoint < source_bytes.len()
                        && source_bytes[checkpoint] as usize <= glyph.start
                    {
                        stable_xs.push(exact_x);
                        checkpoint += 1;
                    }
                    exact_x += f64::from(glyph.width(font_size).max(0.0));
                }
            }
        }
        while checkpoint < source_bytes.len()
            && source_bytes[checkpoint] as usize == self.source_len
        {
            stable_xs.push(exact_x);
            checkpoint += 1;
        }
        (checkpoint == source_bytes.len()
            && stable_xs.len() == source_bytes.len()
            && exact_x.is_finite()
            && exact_x > 0.0)
            .then_some(stable_xs)
    }

    pub(super) fn with_stable_xs(self, stable_xs: Vec<f64>) -> Option<Self> {
        if stable_xs.len() != self.checkpoint_count
            || stable_xs.first().copied() != Some(0.0)
            || stable_xs.windows(2).any(|pair| pair[0] > pair[1])
        {
            return None;
        }
        let source_bytes = self
            .absolute_checkpoints()
            .into_iter()
            .map(|checkpoint| checkpoint.source_byte)
            .collect();
        let width = stable_xs.last().copied()?;
        Self::from_absolute_parts(
            source_bytes,
            stable_xs,
            self.source_len,
            width,
            self.height,
            true,
        )
    }

    pub(super) fn refine_exact_bands<F>(self, text: &str, mut shape: F) -> Option<(Self, usize)>
    where
        F: FnMut(&str) -> Option<Self>,
    {
        if self.width < F32_EXACT_INTEGRAL_LIMIT {
            return Some((self, 0));
        }

        let source_bytes = self
            .absolute_checkpoints()
            .into_iter()
            .map(|checkpoint| checkpoint.source_byte)
            .collect::<Vec<_>>();
        let mut checkpoints = Vec::with_capacity(source_bytes.len());
        let mut band_start = 0_usize;
        let terminal = source_bytes.len().saturating_sub(1);
        let mut exact_x = 0.0_f64;
        let mut height = self.height;
        let mut band_count = 0_usize;

        while band_start < terminal {
            let source_start = source_bytes[band_start];
            let mut band_end = band_start + 1;
            while band_end < terminal
                && source_bytes[band_end + 1].saturating_sub(source_start)
                    <= TEXT_AREA_HORIZONTAL_EXACT_BAND_MAX_SOURCE_SPAN as u32
            {
                band_end += 1;
            }
            let source_end = source_bytes[band_end];
            let range = source_start as usize..source_end as usize;
            let band = shape(text.get(range.clone())?)?;
            if band.source_len != range.len() {
                return None;
            }

            for (index, checkpoint) in band.absolute_checkpoints().into_iter().enumerate() {
                if band_count > 0 && index == 0 {
                    continue;
                }
                checkpoints.push(Checkpoint {
                    source_byte: source_start.checked_add(checkpoint.source_byte)?,
                    x: exact_x + checkpoint.x,
                });
            }
            exact_x += band.width;
            height = height.max(band.height);
            band_count += 1;
            band_start = band_end;
        }

        if checkpoints.len() != source_bytes.len()
            || checkpoints
                .last()
                .is_none_or(|checkpoint| checkpoint.source_byte as usize != text.len())
            || checkpoints
                .windows(2)
                .any(|pair| pair[0].source_byte >= pair[1].source_byte || pair[0].x > pair[1].x)
        {
            return None;
        }

        Some((
            Self::from_absolute_checkpoints(checkpoints, text.len(), exact_x, height, false)?,
            band_count,
        ))
    }

    pub(super) fn width(&self) -> f64 {
        self.width
    }

    pub(super) fn edit_window(
        &self,
        old_edit: Range<usize>,
        inserted_bytes: usize,
    ) -> Option<EditWindow> {
        if !self.stable_coordinates
            || old_edit.start > old_edit.end
            || old_edit.end > self.source_len
        {
            return None;
        }
        let terminal = self.segment_count();
        let containing_start = self.segment_at_source(old_edit.start);
        let start_segment = containing_start.saturating_sub(1);
        let containing_end = self.segment_at_source(old_edit.end);
        let containing_end_start = self.checkpoint(containing_end)?.source_byte as usize;
        let lower_bound = if containing_end_start < old_edit.end {
            containing_end.saturating_add(1)
        } else {
            containing_end
        };
        let end_segment = lower_bound.saturating_add(1).min(terminal);
        let old_source = self.checkpoint(start_segment)?.source_byte as usize
            ..self.checkpoint(end_segment)?.source_byte as usize;
        let deleted_bytes = old_edit.end - old_edit.start;
        let new_end = old_source
            .end
            .checked_sub(deleted_bytes)?
            .checked_add(inserted_bytes)?;
        Some(EditWindow {
            new_source: old_source.start..new_end,
            old_source,
            start_segment,
            end_segment,
        })
    }

    pub(super) fn splice_edit(&self, window: EditWindow, replacement: LineIndex) -> Option<Self> {
        if replacement.source_len != window.new_source.len()
            || replacement.checkpoint(0)?.source_byte != 0
            || replacement
                .checkpoint(replacement.segment_count())?
                .source_byte as usize
                != replacement.source_len
            || !replacement.stable_coordinates
        {
            return None;
        }
        let old_start = self.checkpoint(window.start_segment)?;
        let old_end = self.checkpoint(window.end_segment)?;
        let old_segment_width = old_end.x - old_start.x;
        let x_shift = replacement.width - old_segment_width;
        let byte_shift = window.new_source.len() as i64 - window.old_source.len() as i64;
        let source_len = (self.source_len as i64).checked_add(byte_shift)?;
        let source_len = usize::try_from(source_len).ok()?;
        let width = self.width + x_shift;
        let mut blocks = Vec::with_capacity(
            self.blocks
                .len()
                .saturating_add(replacement.blocks.len())
                .saturating_add(2),
        );
        self.append_segment_range(&mut blocks, 0..window.start_segment)?;
        replacement.append_segment_range(&mut blocks, 0..replacement.segment_count())?;
        self.append_segment_range(&mut blocks, window.end_segment..self.segment_count())?;
        let checkpoint_count = blocks
            .iter()
            .map(|block| block.segment_count())
            .sum::<usize>()
            .saturating_add(1);
        let projected_source_len = blocks.iter().map(|block| block.source_len()).sum::<usize>();
        let projected_width = blocks.iter().map(|block| block.width()).sum::<f64>();
        if checkpoint_count < 3
            || projected_source_len != source_len
            || (projected_width - width).abs() > f64::EPSILON * width.abs().max(1.0) * 8.0
        {
            return None;
        }
        Some(Self {
            blocks,
            checkpoint_count,
            source_len,
            width: projected_width,
            height: self.height.max(replacement.height),
            stable_coordinates: true,
        })
    }

    pub(super) fn height(&self) -> f32 {
        self.height
    }

    pub(super) fn checkpoint_count(&self) -> usize {
        self.checkpoint_count
    }

    pub(super) fn resident_bytes(&self) -> usize {
        self.blocks.iter().map(|block| block.resident_bytes()).sum()
    }

    pub(super) fn exclusive_resident_bytes(&self) -> usize {
        self.blocks
            .iter()
            .filter(|block| Arc::strong_count(block) == 1)
            .map(|block| block.resident_bytes())
            .sum()
    }

    pub(super) fn windows_for_x(&self, origin: f64, width: f32) -> Vec<Window> {
        let origin = origin.max(0.0).min(self.width);
        let end = (origin + f64::from(width.max(1.0))).min(self.width);
        let start = self.segment_at_x(origin).saturating_sub(1);
        let end = self
            .segment_at_x(end)
            .saturating_add(2)
            .min(self.segment_count());
        self.windows_between(start, end.max(start + 1))
    }

    pub(super) fn windows_for_source_byte(&self, source_byte: usize) -> Vec<Window> {
        let containing = self.segment_at_source(source_byte);
        let start = containing.saturating_sub(1);
        let end = containing.saturating_add(2).min(self.segment_count());
        self.windows_between(start, end.max(start + 1))
    }

    fn windows_between(&self, start: usize, end: usize) -> Vec<Window> {
        let start = start.min(self.segment_count().saturating_sub(1));
        let end = end.clamp(start + 1, self.segment_count());
        (start..end)
            .filter_map(|index| self.segment_window(index))
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn line_index(
        checkpoints: Vec<Checkpoint>,
        source_len: usize,
        width: f64,
        height: f32,
    ) -> LineIndex {
        LineIndex::from_absolute_checkpoints(checkpoints, source_len, width, height, false)
            .expect("test checkpoints should form an index")
    }

    fn index() -> LineIndex {
        line_index(
            vec![
                Checkpoint {
                    source_byte: 0,
                    x: 0.0,
                },
                Checkpoint {
                    source_byte: 100,
                    x: 1_000.0,
                },
                Checkpoint {
                    source_byte: 200,
                    x: 2_000.0,
                },
                Checkpoint {
                    source_byte: 300,
                    x: 3_000.0,
                },
                Checkpoint {
                    source_byte: 400,
                    x: 4_000.0,
                },
            ],
            400,
            4_000.0,
            20.0,
        )
    }

    #[test]
    fn x_window_is_bounded_by_neighboring_safe_checkpoints() {
        let windows = index().windows_for_x(1_450.0, 920.0);
        assert_eq!(windows.len(), 4);
        assert_eq!(windows[0].source, 0..100);
        assert_eq!(windows[3].source, 300..400);
        assert!(windows[0].x <= 1_450.0);
        assert!(windows[3].x + windows[3].width >= 2_370.0);
    }

    #[test]
    fn source_window_includes_a_guard_checkpoint_on_each_side() {
        let windows = index().windows_for_source_byte(250);
        assert_eq!(windows.len(), 3);
        assert_eq!(windows[0].source, 100..200);
        assert_eq!(windows[2].source, 300..400);
    }

    #[test]
    fn prepared_window_retains_unit_deltas_across_f32_precision_boundary() {
        let viewport = 920.0;
        let values = [16_777_215_u32, 16_777_216, 16_777_217, 24_000_001];
        let local_origins = values.map(|value| {
            let scroll = f64::from(value);
            let (origin, _) = prepared_window(viewport, scroll);
            (origin, origin - scroll)
        });

        for ((left_origin, left_local), (right_origin, right_local)) in local_origins
            .into_iter()
            .zip(local_origins.into_iter().skip(1))
        {
            assert!(right_origin >= left_origin);
            assert_ne!(
                (left_origin, left_local),
                (right_origin, right_local),
                "adjacent or odd integral offsets must not collapse before local projection"
            );
        }
        assert_eq!(local_origins[1].1 - local_origins[2].1, 1.0);
    }

    #[test]
    fn index_windows_keep_exact_checkpoint_coordinates_past_two_to_the_24th() {
        let index = line_index(
            vec![
                Checkpoint {
                    source_byte: 0,
                    x: 0.0,
                },
                Checkpoint {
                    source_byte: 100,
                    x: 16_777_215.0,
                },
                Checkpoint {
                    source_byte: 200,
                    x: 16_777_216.0,
                },
                Checkpoint {
                    source_byte: 300,
                    x: 16_777_217.0,
                },
                Checkpoint {
                    source_byte: 400,
                    x: 24_000_001.0,
                },
            ],
            400,
            24_000_001.0,
            20.0,
        );

        let windows = index.windows_for_x(16_777_217.0, 1.0);
        assert!(windows.iter().any(|window| window.x == 16_777_217.0));
        assert_eq!(index.width(), 24_000_001.0);
    }

    #[test]
    fn exact_band_refinement_accumulates_local_widths_without_losing_units() {
        const SOURCE_LEN: usize = 600_000;
        const STEP: usize = 4_000;
        const ADVANCE: f64 = 42.000_001;
        let checkpoints = (0..=SOURCE_LEN / STEP)
            .map(|index| Checkpoint {
                source_byte: (index * STEP) as u32,
                x: ((index * STEP) as f32 * ADVANCE as f32) as f64,
            })
            .collect::<Vec<_>>();
        let rounded = line_index(
            checkpoints,
            SOURCE_LEN,
            (SOURCE_LEN as f32 * ADVANCE as f32) as f64,
            20.0,
        );
        let source = "x".repeat(SOURCE_LEN);
        let (exact, bands) = rounded
            .refine_exact_bands(&source, |band| {
                let checkpoints = (0..=band.len() / STEP)
                    .map(|index| Checkpoint {
                        source_byte: (index * STEP) as u32,
                        x: index as f64 * STEP as f64 * ADVANCE,
                    })
                    .collect::<Vec<_>>();
                Some(line_index(
                    checkpoints,
                    band.len(),
                    band.len() as f64 * ADVANCE,
                    20.0,
                ))
            })
            .expect("bounded exact bands should merge");

        assert_eq!(bands, 3);
        assert_eq!(exact.width(), SOURCE_LEN as f64 * ADVANCE);
        assert_ne!(exact.width(), (SOURCE_LEN as f32 * ADVANCE as f32) as f64);
        assert_eq!(exact.checkpoint_count(), SOURCE_LEN / STEP + 1);
    }

    #[test]
    fn sixty_four_mib_edit_reuses_all_untouched_checkpoint_blocks() {
        const SOURCE_LEN: usize = 64 * 1024 * 1024;
        const CHECKPOINTS: usize = 246_041;
        let checkpoints = (0..CHECKPOINTS)
            .map(|index| {
                let source_byte = index * SOURCE_LEN / (CHECKPOINTS - 1);
                Checkpoint {
                    source_byte: source_byte as u32,
                    x: source_byte as f64 * 6.565,
                }
            })
            .collect::<Vec<_>>();
        let width = checkpoints.last().expect("terminal checkpoint").x;
        let index =
            LineIndex::from_absolute_checkpoints(checkpoints, SOURCE_LEN, width, 20.0, true)
                .expect("64 MiB synthetic index should be valid");
        let edit_at = SOURCE_LEN / 2 + 1;
        let window = index
            .edit_window(edit_at..edit_at, 1)
            .expect("interior insertion should have a guarded edit window");
        let replacement_len = window.new_source.len();
        let replacement_width = replacement_len as f64 * 6.565;
        let replacement = LineIndex::from_absolute_checkpoints(
            vec![
                Checkpoint {
                    source_byte: 0,
                    x: 0.0,
                },
                Checkpoint {
                    source_byte: (replacement_len / 2) as u32,
                    x: replacement_width / 2.0,
                },
                Checkpoint {
                    source_byte: replacement_len as u32,
                    x: replacement_width,
                },
            ],
            replacement_len,
            replacement_width,
            20.0,
            true,
        )
        .expect("replacement index should be valid");

        let updated = index
            .splice_edit(window, replacement)
            .expect("64 MiB local edit should splice");
        let shared_blocks = updated
            .blocks
            .iter()
            .filter(|updated| {
                index
                    .blocks
                    .iter()
                    .any(|original| Arc::ptr_eq(original, updated))
            })
            .count();

        assert_eq!(updated.source_len, SOURCE_LEN + 1);
        assert!(
            shared_blocks + 4 >= index.blocks.len(),
            "only blocks touching the guarded edit may be replaced: shared={shared_blocks} original={}",
            index.blocks.len(),
        );
        assert!(
            updated.exclusive_resident_bytes() <= 32 * 1024,
            "a 64 MiB edit must allocate only bounded checkpoint blocks, got {} bytes",
            updated.exclusive_resident_bytes(),
        );
    }
}
