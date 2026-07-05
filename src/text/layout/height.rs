use std::collections::{BTreeMap, HashMap};
use std::num::NonZeroUsize;

use lru::LruCache;

use super::super::buffer::{Buffer, LineLayoutIdentity};
use super::super::document::{Style, TextDirection};
use super::super::edit::{Area, AreaWrap};
use super::constants::{TEXT_AREA_HEIGHT_INDEX_BLOCK_LINES, TEXT_AREA_HEIGHT_INDEX_CACHE_CAPACITY};
use super::key::{StyleKey, finite_bits};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub(super) struct TextAreaHeightKey {
    style: StyleKey,
    width: u32,
    wrap: AreaWrap,
    direction: TextDirection,
}

#[derive(Debug, Clone)]
pub(super) struct TextAreaHeightIndex {
    line_count: usize,
    estimated_line_height: f32,
    measured: HashMap<LineLayoutIdentity, f32>,
    resolved: BTreeMap<usize, f32>,
    block_deltas: BTreeMap<usize, f32>,
    measured_delta: f32,
    resolved_dirty: bool,
}

impl TextAreaHeightKey {
    pub(super) fn new(area_model: &Area, style: Style, width: f32) -> Self {
        Self {
            style: StyleKey::new(style),
            width: finite_bits(width.max(0.0)),
            wrap: area_model.wrap(),
            direction: style.direction(),
        }
    }
}

impl TextAreaHeightIndex {
    pub(super) fn new(line_count: usize, estimated_line_height: f32) -> Self {
        Self {
            line_count: line_count.max(1),
            estimated_line_height: estimated_line_height.max(1.0),
            measured: HashMap::new(),
            resolved: BTreeMap::new(),
            block_deltas: BTreeMap::new(),
            measured_delta: 0.0,
            resolved_dirty: false,
        }
    }

    pub(super) fn sync(&mut self, source: &Buffer, line_count: usize, estimated_line_height: f32) {
        let line_count = line_count.max(1);
        let estimated_line_height = estimated_line_height.max(1.0);
        if self.estimated_line_height.to_bits() != estimated_line_height.to_bits() {
            *self = Self::new(line_count, estimated_line_height);
            return;
        }
        if self.line_count != line_count {
            self.line_count = line_count;
            self.resolved_dirty = true;
        }
        if self.resolved_dirty {
            self.rebuild_resolved(source);
        }
    }

    pub(super) fn update_line(&mut self, source: &Buffer, line: usize, height: f32) {
        if line >= self.line_count {
            return;
        }
        let height = height.max(1.0);
        if let Some(identity) = source.line_layout_identity(line) {
            self.measured.insert(identity, height);
        }
        self.update_resolved_line(line, height);
    }

    pub(super) fn line_height(&self, line: usize) -> f32 {
        self.resolved
            .get(&line)
            .copied()
            .unwrap_or(self.estimated_line_height)
    }

    pub(super) fn line_top(&self, line: usize) -> f32 {
        let line = line.min(self.line_count);
        line as f32 * self.estimated_line_height + self.measured_delta_before(line)
    }

    pub(super) fn total_height(&self) -> f32 {
        (self.line_count as f32 * self.estimated_line_height + self.measured_delta).max(0.0)
    }

    pub(super) fn line_at_y(&self, y: f32) -> usize {
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

    pub(super) fn visible_line_count(&self, scroll_y: f32, viewport_height: f32) -> usize {
        let start = self.line_at_y(scroll_y);
        let limit = scroll_y.max(0.0) + viewport_height.max(self.estimated_line_height);
        let mut line = start;
        while line < self.line_count && self.line_top(line) <= limit {
            line += 1;
        }
        line.saturating_sub(start).max(1)
    }

    fn update_resolved_line(&mut self, line: usize, height: f32) {
        let old = self
            .resolved
            .insert(line, height)
            .unwrap_or(self.estimated_line_height);
        let delta = height - old;
        if delta.abs() <= f32::EPSILON {
            return;
        }
        self.add_measured_delta(line, delta);
    }

    fn rebuild_resolved(&mut self, source: &Buffer) {
        let measured = std::mem::take(&mut self.measured);
        self.resolved.clear();
        self.block_deltas.clear();
        self.measured_delta = 0.0;
        self.line_count = source.logical_line_count().max(1);

        for line in 0..self.line_count {
            let Some(identity) = source.line_layout_identity(line) else {
                continue;
            };
            let Some(height) = measured.get(&identity).copied() else {
                continue;
            };
            self.measured.insert(identity, height);
            self.update_resolved_line(line, height);
        }

        self.resolved_dirty = false;
    }

    fn add_measured_delta(&mut self, line: usize, delta: f32) {
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

    fn measured_delta_before(&self, line: usize) -> f32 {
        let block = line / TEXT_AREA_HEIGHT_INDEX_BLOCK_LINES;
        let block_start = block * TEXT_AREA_HEIGHT_INDEX_BLOCK_LINES;
        let block_delta = self
            .block_deltas
            .range(..block)
            .map(|(_, delta)| *delta)
            .sum::<f32>();
        let local_delta = self
            .resolved
            .range(block_start..line)
            .map(|(_, height)| *height - self.estimated_line_height)
            .sum::<f32>();
        block_delta + local_delta
    }
}

pub(super) fn cache() -> LruCache<TextAreaHeightKey, TextAreaHeightIndex> {
    LruCache::new(
        NonZeroUsize::new(TEXT_AREA_HEIGHT_INDEX_CACHE_CAPACITY)
            .expect("text area height index cache capacity must be non-zero"),
    )
}
