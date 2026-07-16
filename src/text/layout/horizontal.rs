use std::ops::Range;

use super::constants::TEXT_AREA_RENDER_HORIZONTAL_OVERSCAN;

pub(super) const TEXT_AREA_HORIZONTAL_INDEX_MAX_SOURCE_SPAN: u32 = 4_096;
const TEXT_AREA_HORIZONTAL_INDEX_TARGET_SOURCE_SPAN: u32 = 256;
pub(super) const TEXT_AREA_HORIZONTAL_EXACT_BAND_MAX_SOURCE_SPAN: usize = 262_144;
const F32_EXACT_INTEGRAL_LIMIT: f64 = 16_777_216.0;

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
    source_bytes: Vec<u32>,
    xs: Vec<f64>,
    source_len: usize,
    width: f64,
    height: f32,
    stable_coordinates: bool,
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
    start_index: usize,
    end_index: usize,
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
        if band.source_bytes.first().copied() != Some(0)
            || band.source_bytes.last().copied() != Some(band_source_len)
            || band.source_bytes.len() != band.xs.len()
            || stable_xs.len() != band.xs.len()
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
        for (index, ((source_byte, x), stable_x)) in band
            .source_bytes
            .into_iter()
            .zip(band.xs)
            .zip(stable_xs)
            .enumerate()
        {
            if self.band_count > 0 && index == 0 {
                continue;
            }
            self.source_bytes
                .push(source_start.checked_add(source_byte)?);
            self.xs.push(self.width + x);
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
        Some((
            LineIndex {
                source_bytes: self.source_bytes,
                xs,
                source_len: self.source_len,
                width,
                height: self.height,
                stable_coordinates: use_stable,
            },
            self.band_count,
        ))
    }
}

impl LineIndex {
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

        let (source_bytes, xs) = split_checkpoints(checkpoints);
        Some(Self {
            source_bytes,
            xs,
            source_len: text.len(),
            width: f64::from(run.line_w),
            height: run.line_height.max(1.0),
            stable_coordinates: false,
        })
    }

    pub(super) fn stable_xs(&self, buffer: &glyphon::Buffer) -> Option<Vec<f64>> {
        let shape = buffer.lines.first()?.shape_opt()?;
        if shape.rtl || shape.spans.iter().any(|span| span.level.is_rtl()) {
            return None;
        }
        let font_size = buffer.metrics().font_size;
        let mut stable_xs = Vec::with_capacity(self.source_bytes.len());
        let mut checkpoint = 0_usize;
        let mut exact_x = 0.0_f64;
        for span in &shape.spans {
            for word in &span.words {
                for glyph in &word.glyphs {
                    while checkpoint < self.source_bytes.len()
                        && self.source_bytes[checkpoint] as usize <= glyph.start
                    {
                        stable_xs.push(exact_x);
                        checkpoint += 1;
                    }
                    exact_x += f64::from(glyph.width(font_size).max(0.0));
                }
            }
        }
        while checkpoint < self.source_bytes.len()
            && self.source_bytes[checkpoint] as usize == self.source_len
        {
            stable_xs.push(exact_x);
            checkpoint += 1;
        }
        (checkpoint == self.source_bytes.len()
            && stable_xs.len() == self.source_bytes.len()
            && exact_x.is_finite()
            && exact_x > 0.0)
            .then_some(stable_xs)
    }

    pub(super) fn with_stable_xs(mut self, stable_xs: Vec<f64>) -> Option<Self> {
        if stable_xs.len() != self.xs.len()
            || stable_xs.first().copied() != Some(0.0)
            || stable_xs.windows(2).any(|pair| pair[0] > pair[1])
        {
            return None;
        }
        self.width = stable_xs.last().copied()?;
        self.xs = stable_xs;
        self.stable_coordinates = true;
        Some(self)
    }

    pub(super) fn refine_exact_bands<F>(self, text: &str, mut shape: F) -> Option<(Self, usize)>
    where
        F: FnMut(&str) -> Option<Self>,
    {
        if self.width < F32_EXACT_INTEGRAL_LIMIT {
            return Some((self, 0));
        }

        let mut checkpoints = Vec::with_capacity(self.source_bytes.len());
        let mut band_start = 0_usize;
        let terminal = self.source_bytes.len().saturating_sub(1);
        let mut exact_x = 0.0_f64;
        let mut height = self.height;
        let mut band_count = 0_usize;

        while band_start < terminal {
            let source_start = self.source_bytes[band_start];
            let mut band_end = band_start + 1;
            while band_end < terminal
                && self.source_bytes[band_end + 1].saturating_sub(source_start)
                    <= TEXT_AREA_HORIZONTAL_EXACT_BAND_MAX_SOURCE_SPAN as u32
            {
                band_end += 1;
            }
            let source_end = self.source_bytes[band_end];
            let range = source_start as usize..source_end as usize;
            let band = shape(text.get(range.clone())?)?;
            if band.source_len != range.len() {
                return None;
            }

            for (index, (source_byte, x)) in band.source_bytes.into_iter().zip(band.xs).enumerate()
            {
                if band_count > 0 && index == 0 {
                    continue;
                }
                checkpoints.push(Checkpoint {
                    source_byte: source_start.checked_add(source_byte)?,
                    x: exact_x + x,
                });
            }
            exact_x += band.width;
            height = height.max(band.height);
            band_count += 1;
            band_start = band_end;
        }

        if checkpoints.len() != self.source_bytes.len()
            || checkpoints
                .last()
                .is_none_or(|checkpoint| checkpoint.source_byte as usize != text.len())
            || checkpoints
                .windows(2)
                .any(|pair| pair[0].source_byte >= pair[1].source_byte || pair[0].x > pair[1].x)
        {
            return None;
        }

        let (source_bytes, xs) = split_checkpoints(checkpoints);
        Some((
            Self {
                source_bytes,
                xs,
                source_len: text.len(),
                width: exact_x,
                height,
                stable_coordinates: false,
            },
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
        let terminal = self.source_bytes.len().checked_sub(1)?;
        let containing_start = self
            .source_bytes
            .partition_point(|source| *source <= old_edit.start as u32)
            .saturating_sub(1)
            .min(terminal.saturating_sub(1));
        let start_index = containing_start.saturating_sub(1);
        let containing_end = self
            .source_bytes
            .partition_point(|source| (*source as usize) < old_edit.end)
            .min(terminal);
        let end_index = containing_end.saturating_add(1).min(terminal);
        let old_source =
            self.source_bytes[start_index] as usize..self.source_bytes[end_index] as usize;
        let deleted_bytes = old_edit.end - old_edit.start;
        let new_end = old_source
            .end
            .checked_sub(deleted_bytes)?
            .checked_add(inserted_bytes)?;
        Some(EditWindow {
            new_source: old_source.start..new_end,
            old_source,
            start_index,
            end_index,
        })
    }

    pub(super) fn splice_edit(&self, window: EditWindow, replacement: LineIndex) -> Option<Self> {
        if replacement.source_len != window.new_source.len()
            || replacement.source_bytes.first().copied() != Some(0)
            || replacement.source_bytes.last().copied()? as usize != replacement.source_len
        {
            return None;
        }
        let old_segment_width = self.xs[window.end_index] - self.xs[window.start_index];
        let replacement_segment_width = replacement.xs.last().copied()? - replacement.xs[0];
        let x_shift = replacement_segment_width - old_segment_width;
        let byte_shift = window.new_source.len() as i64 - window.old_source.len() as i64;
        let mut source_bytes = Vec::with_capacity(
            self.source_bytes.len() - (window.end_index - window.start_index)
                + replacement.source_bytes.len().saturating_sub(1),
        );
        let mut xs = Vec::with_capacity(source_bytes.capacity());

        source_bytes.extend_from_slice(&self.source_bytes[..=window.start_index]);
        xs.extend_from_slice(&self.xs[..=window.start_index]);
        let source_origin = self.source_bytes[window.start_index];
        let x_origin = self.xs[window.start_index];
        for (&source, &x) in replacement.source_bytes.iter().zip(&replacement.xs).skip(1) {
            source_bytes.push(source_origin.checked_add(source)?);
            xs.push(x_origin + x);
        }
        for (&source, &x) in self.source_bytes[window.end_index + 1..]
            .iter()
            .zip(&self.xs[window.end_index + 1..])
        {
            let shifted = i64::from(source).checked_add(byte_shift)?;
            source_bytes.push(u32::try_from(shifted).ok()?);
            xs.push(x + x_shift);
        }

        let source_len = (self.source_len as i64).checked_add(byte_shift)?;
        let source_len = usize::try_from(source_len).ok()?;
        let width = self.width + x_shift;
        if source_bytes.len() < 3
            || source_bytes.len() != xs.len()
            || source_bytes.last().copied()? as usize != source_len
            || source_bytes.windows(2).any(|pair| {
                pair[0] >= pair[1] || pair[1] - pair[0] > TEXT_AREA_HORIZONTAL_INDEX_MAX_SOURCE_SPAN
            })
            || xs.windows(2).any(|pair| pair[0] > pair[1])
        {
            return None;
        }
        Some(Self {
            source_bytes,
            xs,
            source_len,
            width,
            height: self.height.max(replacement.height),
            stable_coordinates: true,
        })
    }

    pub(super) fn height(&self) -> f32 {
        self.height
    }

    pub(super) fn checkpoint_count(&self) -> usize {
        self.source_bytes.len()
    }

    pub(super) fn resident_bytes(&self) -> usize {
        self.source_bytes
            .len()
            .saturating_mul(std::mem::size_of::<u32>())
            .saturating_add(self.xs.len().saturating_mul(std::mem::size_of::<f64>()))
    }

    pub(super) fn windows_for_x(&self, origin: f64, width: f32) -> Vec<Window> {
        let origin = origin.max(0.0).min(self.width);
        let end = (origin + f64::from(width.max(1.0))).min(self.width);
        let start_index = self
            .xs
            .partition_point(|x| *x <= origin)
            .saturating_sub(1)
            .saturating_sub(1);
        let end_index = self
            .xs
            .partition_point(|x| *x < end)
            .saturating_add(1)
            .min(self.xs.len().saturating_sub(1));
        self.windows_between(start_index, end_index.max(start_index + 1))
    }

    pub(super) fn windows_for_source_byte(&self, source_byte: usize) -> Vec<Window> {
        let source_byte = source_byte.min(self.source_len);
        let source_byte = u32::try_from(source_byte).unwrap_or(u32::MAX);
        let containing = self
            .source_bytes
            .partition_point(|checkpoint| *checkpoint <= source_byte)
            .saturating_sub(1)
            .min(self.source_bytes.len().saturating_sub(2));
        let start = containing.saturating_sub(1);
        let end = containing
            .saturating_add(2)
            .min(self.source_bytes.len() - 1);
        self.windows_between(start, end.max(start + 1))
    }

    fn windows_between(&self, start: usize, end: usize) -> Vec<Window> {
        let start = start.min(self.source_bytes.len().saturating_sub(2));
        let end = end.clamp(start + 1, self.source_bytes.len() - 1);
        (start..end)
            .map(|index| self.window_between(index, index + 1))
            .collect()
    }

    fn window_between(&self, start: usize, end: usize) -> Window {
        let start = start.min(self.source_bytes.len().saturating_sub(2));
        let end = end.clamp(start + 1, self.source_bytes.len() - 1);
        Window {
            source: self.source_bytes[start] as usize..self.source_bytes[end] as usize,
            x: self.xs[start],
            width: (self.xs[end] - self.xs[start]).max(0.0),
        }
    }
}

fn split_checkpoints(checkpoints: Vec<Checkpoint>) -> (Vec<u32>, Vec<f64>) {
    let mut source_bytes = Vec::with_capacity(checkpoints.len());
    let mut xs = Vec::with_capacity(checkpoints.len());
    for checkpoint in checkpoints {
        source_bytes.push(checkpoint.source_byte);
        xs.push(checkpoint.x);
    }
    (source_bytes, xs)
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
        let (source_bytes, xs) = split_checkpoints(checkpoints);
        LineIndex {
            source_bytes,
            xs,
            source_len,
            width,
            height,
            stable_coordinates: false,
        }
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
}
