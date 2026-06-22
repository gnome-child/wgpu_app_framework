use std::cell::RefCell;
use std::collections::{BTreeMap, HashMap, VecDeque};
use std::fmt;
use std::num::NonZeroUsize;
use std::rc::Rc;
use std::time::Instant;

#[cfg(test)]
use super::buffer::line_start_offsets_for_buffer;
use super::buffer::{
    Buffer, Cursor, LineId, LineLayoutIdentity, Selection, TextAffinity, TextEditImpact,
    TextPosition, buffer_text_len, clamp_cursor_in_buffer, clamp_selection_in_buffer,
    cosmic_buffer_from_text, cursor_position, local_cursor_range_for_source_line,
    set_cosmic_buffer_text, text_index_for_cursor_in_buffer,
};
use super::document::{Align, Block, Document, Run, Style, TextDirection, Weight};
use super::surface::{
    Area, AreaWrap, Field, FieldProjection, PreeditProjection, Surface, projected_state_for_field,
};
use super::view::{ScrollAnchor, TextViewState, Viewport, Visibility};
use crate::geometry::{area, point};
use crate::paint;
use crate::text_system;
use lru::LruCache;

const MEASURE_CACHE_CAPACITY: usize = 2048;
pub(super) const TEXT_AREA_LINE_DISPLAY_CACHE_CAPACITY: usize = 2048;
const TEXT_AREA_RENDER_BUFFER_CACHE_CAPACITY: usize = 32;
const TEXT_AREA_HEIGHT_INDEX_CACHE_CAPACITY: usize = 128;
const TEXT_AREA_HEIGHT_INDEX_BLOCK_LINES: usize = 128;
pub(super) const TEXT_AREA_FRAME_MIN_OVERSCAN_LINES: usize = 16;
pub(super) const TEXT_AREA_RENDER_GUARD_LINES: usize = 12;
pub(super) const TEXT_AREA_RENDER_MAX_WINDOW_LINES: usize = 128;
const TEXT_AREA_RENDER_HORIZONTAL_OVERSCAN: f32 = 256.0;
pub(super) const TEXT_AREA_FRAME_MAX_LOGICAL_LINES: usize = 256;
pub(super) const TEXT_LAYOUT_VISUAL_LINE_EPSILON: f32 = 0.5;
pub(super) const TEXT_FIELD_CARET_MARGIN: f32 = 5.0;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TextAreaObservation {
    RenderOnly,
    Observe,
}

#[derive(Clone)]
pub(super) struct CachedTextAreaLineDisplay {
    pub(super) buffer: Rc<RefCell<glyphon::Buffer>>,
    pub(super) height: f32,
    pub(super) width: f32,
}
#[derive(Clone)]
pub(super) struct TextAreaLineDisplay {
    pub(super) buffer: Rc<RefCell<glyphon::Buffer>>,
    pub(super) source_line: usize,
    pub(super) source_line_id: Option<LineId>,
    pub(super) source_start: usize,
    pub(super) source_text_len: usize,
    pub(super) height: f32,
    pub(super) width: f32,
}
#[derive(Clone)]
pub(super) struct TextAreaDisplaySegment {
    display: TextAreaLineDisplay,
    y: f32,
}
#[derive(Debug, Clone, Copy, PartialEq)]
struct TextAreaRenderAnchor {
    source_line: usize,
    source_line_end: usize,
    y: f32,
    height: f32,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct TextAreaRenderLineWindow {
    start: usize,
    end: usize,
}
#[derive(Clone)]
pub(super) struct CachedTextAreaRenderBuffer {
    pub(super) buffer: Rc<RefCell<glyphon::Buffer>>,
}
#[derive(Debug, Clone, Default, PartialEq)]
pub(super) struct HighlightSpans {
    selection: Vec<SelectionSpan>,
    preedit_underline: Vec<SelectionSpan>,
    preedit_selection: Vec<SelectionSpan>,
}
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub(super) struct TextAreaLineDisplayKey {
    buffer_id: u64,
    line: LineLayoutIdentity,
    style: StyleKey,
    width: u32,
    wrap: AreaWrap,
    direction: TextDirection,
}
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub(super) struct TextAreaRenderBufferKey {
    buffer_id: u64,
    revision: u64,
    source_line_start: usize,
    source_line_end: usize,
    style: StyleKey,
    width: u32,
    wrap: AreaWrap,
    direction: TextDirection,
}
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub(super) struct TextAreaHeightKey {
    buffer_id: u64,
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
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub(super) struct StyleKey {
    size: u32,
    weight: Weight,
    direction: TextDirection,
}
#[derive(Debug)]
pub(super) struct MeasureCache {
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
pub(super) struct BoundsKey {
    width: u32,
    height: u32,
}
impl TextAreaLineDisplayKey {
    fn new(
        area_model: &Area,
        buffer: &Buffer,
        style: Style,
        width: f32,
        source_line: usize,
    ) -> Option<Self> {
        Some(Self {
            buffer_id: buffer.id(),
            line: buffer.line_layout_identity(source_line)?,
            style: StyleKey::new(style),
            width: finite_bits(width.max(0.0)),
            wrap: area_model.wrap(),
            direction: style.direction(),
        })
    }
}
impl TextAreaRenderBufferKey {
    fn new(
        area_model: &Area,
        buffer: &Buffer,
        style: Style,
        width: f32,
        source_line_start: usize,
        source_line_end: usize,
    ) -> Self {
        Self {
            buffer_id: buffer.id(),
            revision: buffer.revision(),
            source_line_start,
            source_line_end,
            style: StyleKey::new(style),
            width: finite_bits(width.max(0.0)),
            wrap: area_model.wrap(),
            direction: style.direction(),
        }
    }
}
impl TextAreaRenderLineWindow {
    fn new(visible_start: usize, visible_end: usize, line_count: usize) -> Self {
        let line_count = line_count.max(1);
        let visible_start = visible_start.min(line_count.saturating_sub(1));
        let visible_end = visible_end.min(line_count).max(visible_start + 1);
        let visible_lines = visible_end.saturating_sub(visible_start).max(1);
        let guard_lines = text_area_render_guard_lines(visible_lines);
        let window_lines = visible_lines
            .saturating_add(guard_lines.saturating_mul(2))
            .min(TEXT_AREA_RENDER_MAX_WINDOW_LINES)
            .min(line_count)
            .max(visible_lines.min(line_count));

        if window_lines >= line_count {
            return Self {
                start: 0,
                end: line_count,
            };
        }

        let leading_guard = window_lines.saturating_sub(visible_lines) / 2;
        let stride = leading_guard.max(1);
        let desired_start = visible_start.saturating_sub(leading_guard);
        let mut start = desired_start / stride * stride;
        if visible_end > start.saturating_add(window_lines) {
            start = visible_end.saturating_sub(window_lines);
        }
        start = start.min(line_count.saturating_sub(window_lines));

        Self {
            start,
            end: start + window_lines,
        }
    }
}
impl TextAreaHeightKey {
    fn new(area_model: &Area, buffer: &Buffer, style: Style, width: f32) -> Self {
        Self {
            buffer_id: buffer.id(),
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
            measured: HashMap::new(),
            resolved: BTreeMap::new(),
            block_deltas: BTreeMap::new(),
            measured_delta: 0.0,
            resolved_dirty: false,
        }
    }

    fn sync(&mut self, source: &Buffer, line_count: usize, estimated_line_height: f32) {
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

    fn update_line(&mut self, source: &Buffer, line: usize, height: f32) {
        if line >= self.line_count {
            return;
        }
        let height = height.max(1.0);
        if let Some(identity) = source.line_layout_identity(line) {
            self.measured.insert(identity, height);
        }
        self.update_resolved_line(line, height);
    }

    fn line_height(&self, line: usize) -> f32 {
        self.resolved
            .get(&line)
            .copied()
            .unwrap_or(self.estimated_line_height)
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

    fn invalidate_line_range(&mut self, start: usize, count: usize) {
        let start = start.min(self.line_count);
        let end = start.saturating_add(count.max(1)).min(self.line_count);
        let lines = self
            .resolved
            .range(start..end)
            .map(|(line, _)| *line)
            .collect::<Vec<_>>();
        for line in lines {
            if let Some(height) = self.resolved.remove(&line) {
                self.add_measured_delta(line, self.estimated_line_height - height);
            }
        }
    }

    fn apply_edit_impact(&mut self, impact: &TextEditImpact, line_count: usize) {
        if let Some(line_id) = impact.affected_start_line_id {
            self.measured.retain(|identity, _| identity.id != line_id);
        }
        if impact.line_count_delta() == 0 {
            self.invalidate_line_range(impact.affected_start_line, impact.affected_line_count());
            return;
        }

        self.line_count = line_count.max(1);
        self.resolved_dirty = true;
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
            .resolved
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
    pub text_area_render_surface_calls: usize,
    pub text_area_render_surface_cache_hits: usize,
    pub text_area_render_surface_cache_misses: usize,
    pub text_area_render_surface_source_lines: usize,
    pub text_area_render_surface_source_bytes: usize,
    pub text_area_render_surface_anchor_us: u128,
    pub text_area_render_surface_text_us: u128,
    pub text_area_render_surface_buffer_us: u128,
    pub text_area_render_surface_attrs_us: u128,
    pub text_area_render_surface_size_us: u128,
    pub text_area_render_surface_shape_us: u128,
    pub text_area_render_surface_metadata_us: u128,
    pub text_area_render_surface_total_us: u128,
    pub text_area_line_cache_hits: usize,
    pub text_area_line_cache_misses: usize,
    pub text_area_line_shape_calls: usize,
    pub text_area_shaped_logical_lines: usize,
    pub text_area_shaped_visual_lines: usize,
    pub text_area_visible_logical_lines: usize,
    pub text_area_layout_segments: usize,
    pub text_area_overscan_segments: usize,
    pub text_area_interaction_surfaces: usize,
    pub text_area_hit_run_scans: usize,
    pub text_area_height_index_hits: usize,
    pub text_area_height_index_misses: usize,
    pub highlight_run_scans: usize,
    pub highlight_spans: usize,
    pub highlight_skips: usize,
}

impl Diagnostics {
    pub(super) fn add_highlight_stats(&mut self, stats: HighlightStats) {
        self.highlight_run_scans += stats.run_scans;
        self.highlight_spans += stats.spans;
        self.highlight_skips += stats.skips;
    }
}

impl HighlightStats {
    pub(super) fn record_run_scan(&mut self) {
        self.run_scans += 1;
    }

    pub(super) fn record_span(&mut self) {
        self.spans += 1;
    }

    pub(super) fn record_skip(&mut self) {
        self.skips += 1;
    }

    #[cfg(test)]
    pub(super) fn add(&mut self, other: Self) {
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
pub(super) struct TextInteractionStats {
    pub(super) text_area_shape_until_scroll_calls: usize,
    pub(super) text_area_frame_cache_hits: usize,
    pub(super) text_area_frame_cache_misses: usize,
    pub(super) text_area_frame_shape_calls: usize,
    pub(super) text_area_frame_shaped_logical_lines: usize,
    pub(super) text_area_frame_shaped_visual_lines: usize,
    pub(super) hit_run_scans: usize,
    pub(super) aggregate_buffer_fallbacks: usize,
}

pub struct Engine {
    pub(super) font_system: glyphon::FontSystem,
    pub(super) cache: MeasureCache,
    pub(super) text_area_line_displays: LruCache<TextAreaLineDisplayKey, CachedTextAreaLineDisplay>,
    pub(super) text_area_render_buffers:
        LruCache<TextAreaRenderBufferKey, CachedTextAreaRenderBuffer>,
    pub(super) text_area_height_indices: LruCache<TextAreaHeightKey, TextAreaHeightIndex>,
    pub(super) diagnostics: Diagnostics,
    #[cfg(test)]
    pub(super) highlight_stats: HighlightStats,
    #[cfg(test)]
    pub(super) interaction_stats: TextInteractionStats,
    #[cfg(test)]
    pub(super) uncached_measure_count: usize,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Measure {
    pub(super) max: Option<area::Logical>,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Metrics {
    pub(super) area: area::Logical,
    pub(super) line_count: usize,
}

#[derive(Debug, Clone, PartialEq)]
pub struct TextFieldLayout {
    pub(super) selection_spans: Vec<SelectionSpan>,
    pub(super) preedit_underline_spans: Vec<SelectionSpan>,
    pub(super) preedit_selection_spans: Vec<SelectionSpan>,
    pub(super) caret: Option<Caret>,
    pub(super) scroll_x: f32,
    pub(super) scroll_y: f32,
    pub(super) content_area: area::Logical,
}

pub struct TextAreaPaintLayout {
    pub(super) layout: TextFieldLayout,
    pub(super) interaction_surfaces: Vec<TextAreaSurface>,
    pub(super) render_surfaces: Vec<TextAreaSurface>,
}

#[derive(Clone)]
pub struct TextAreaSurface {
    pub(super) x: f32,
    pub(super) y: f32,
    pub(super) width: f32,
    pub(super) height: f32,
    pub(super) source_line: usize,
    pub(super) source_line_id: Option<LineId>,
    pub(super) source_start: usize,
    pub(super) source_text_len: usize,
    pub(super) buffer: Rc<RefCell<glyphon::Buffer>>,
    pub(super) default_color: paint::Color,
}

impl fmt::Debug for TextAreaSurface {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("TextAreaSurface")
            .field("x", &self.x)
            .field("y", &self.y)
            .field("width", &self.width)
            .field("height", &self.height)
            .field("source_line", &self.source_line)
            .field("source_line_id", &self.source_line_id)
            .field("source_start", &self.source_start)
            .field("source_text_len", &self.source_text_len)
            .field("default_color", &self.default_color)
            .finish_non_exhaustive()
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SelectionSpan {
    pub(super) x: f32,
    pub(super) y: f32,
    pub(super) width: f32,
    pub(super) height: f32,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Caret {
    pub(super) x: f32,
    pub(super) y: f32,
    pub(super) height: f32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub(crate) struct AreaScrollKey {
    pub(super) buffer_id: u64,
    pub(super) revision: u64,
    pub(super) style: StyleKey,
    pub(super) viewport: BoundsKey,
    pub(super) wrap: AreaWrap,
}

impl AreaScrollKey {
    pub(super) fn new(area_model: &Area, style: Style, viewport: area::Logical) -> Self {
        Self {
            buffer_id: area_model.buffer().id(),
            revision: area_model.buffer().revision(),
            style: StyleKey::new(style),
            viewport: BoundsKey::new(viewport),
            wrap: area_model.wrap(),
        }
    }
}

impl TextAreaLineDisplay {
    fn from_cached(source: &Buffer, source_line: usize, cached: CachedTextAreaLineDisplay) -> Self {
        let (source_start, source_text_len) = text_area_line_source_metrics(source, source_line);
        let source_line_id = source
            .line_layout_identity(source_line)
            .map(|identity| identity.id);
        Self {
            buffer: cached.buffer,
            source_line,
            source_line_id,
            source_start,
            source_text_len,
            height: cached.height,
            width: cached.width,
        }
    }
}

fn text_area_line_source_metrics(source: &Buffer, source_line: usize) -> (usize, usize) {
    let inner = source.inner.borrow();
    (
        inner.document.line_start(source_line),
        inner.document.line_text_len(source_line),
    )
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
    pub fn caret_layout(&self) -> Option<CaretLayout> {
        self.caret.map(CaretLayout::new)
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

    pub(crate) fn translated_for_scroll(&self, scroll_x: f32, scroll_y: f32) -> Self {
        let dx = self.scroll_x - scroll_x;
        let dy = self.scroll_y - scroll_y;
        let translate_span = |span: &SelectionSpan| {
            SelectionSpan::new(span.x + dx, span.y + dy, span.width, span.height)
        };

        Self {
            selection_spans: self.selection_spans.iter().map(translate_span).collect(),
            preedit_underline_spans: self
                .preedit_underline_spans
                .iter()
                .map(translate_span)
                .collect(),
            preedit_selection_spans: self
                .preedit_selection_spans
                .iter()
                .map(translate_span)
                .collect(),
            caret: self
                .caret
                .map(|caret| Caret::new(caret.x + dx, caret.y + dy, caret.height)),
            scroll_x,
            scroll_y,
            content_area: self.content_area,
        }
    }
}
impl TextAreaPaintLayout {
    pub fn layout(&self) -> &TextFieldLayout {
        &self.layout
    }
    pub fn interaction_surfaces(&self) -> &[TextAreaSurface] {
        &self.interaction_surfaces
    }
    pub fn render_surfaces(&self) -> &[TextAreaSurface] {
        &self.render_surfaces
    }
    pub fn into_interaction_parts(self) -> (TextFieldLayout, Vec<TextAreaSurface>) {
        (self.layout, self.interaction_surfaces)
    }
    pub fn into_projection_parts(
        self,
    ) -> (TextFieldLayout, Vec<TextAreaSurface>, Vec<TextAreaSurface>) {
        (self.layout, self.interaction_surfaces, self.render_surfaces)
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

    pub fn source_line(&self) -> usize {
        self.source_line
    }

    pub fn source_line_id(&self) -> Option<LineId> {
        self.source_line_id
    }

    pub fn source_start(&self) -> usize {
        self.source_start
    }

    pub fn source_text_len(&self) -> usize {
        self.source_text_len
    }

    pub fn buffer(&self) -> Rc<RefCell<glyphon::Buffer>> {
        self.buffer.clone()
    }

    pub fn default_color(&self) -> paint::Color {
        self.default_color
    }

    pub(crate) fn translated_for_scroll(
        &self,
        old_scroll: point::Logical,
        new_scroll: point::Logical,
        _new_viewport: area::Logical,
    ) -> Self {
        let dx = old_scroll.x() - new_scroll.x();
        let dy = old_scroll.y() - new_scroll.y();
        self.translated_by(dx, dy)
    }

    pub(crate) fn translated_by(&self, dx: f32, dy: f32) -> Self {
        Self {
            x: self.x + dx,
            y: self.y + dy,
            width: self.width,
            height: self.height,
            source_line: self.source_line,
            source_line_id: self.source_line_id,
            source_start: self.source_start,
            source_text_len: self.source_text_len,
            buffer: self.buffer.clone(),
            default_color: self.default_color,
        }
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

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct CaretLayout {
    caret: Caret,
}

impl CaretLayout {
    pub fn new(caret: Caret) -> Self {
        Self { caret }
    }

    pub fn caret(self) -> Caret {
        self.caret
    }

    pub fn visibility_in(self, viewport: Viewport, margin: f32) -> Visibility {
        viewport.visibility_of_local_caret(self.caret, margin)
    }
}

fn ensure_caret_visible_from_layout(
    state: TextViewState,
    viewport: area::Logical,
    caret_layout: CaretLayout,
    content_area: Option<area::Logical>,
) -> Option<TextViewState> {
    let viewport_state =
        Viewport::new(viewport, point::logical(state.scroll_x(), state.scroll_y()));
    let visibility = caret_layout.visibility_in(viewport_state, TEXT_FIELD_CARET_MARGIN);
    if visibility.is_visible() {
        return Some(state);
    }
    if matches!(visibility, Visibility::Unknown) {
        return None;
    }

    let caret = caret_layout.caret();
    let mut scroll_x = state.scroll_x();
    let mut scroll_y = state.scroll_y();
    match visibility {
        Visibility::Above => {
            scroll_y = scroll_y + caret.y() - TEXT_FIELD_CARET_MARGIN;
        }
        Visibility::Below => {
            scroll_y =
                scroll_y + caret.y() + caret.height() + TEXT_FIELD_CARET_MARGIN - viewport.height();
        }
        Visibility::Before => {
            scroll_x = scroll_x + caret.x() - TEXT_FIELD_CARET_MARGIN;
        }
        Visibility::After => {
            scroll_x = scroll_x + caret.x() + 1.0 + TEXT_FIELD_CARET_MARGIN - viewport.width();
        }
        Visibility::Visible | Visibility::Unknown => {}
    }

    if let Some(content_area) = content_area {
        let max_scroll_x = (content_area.width() - viewport.width()).max(0.0);
        let max_scroll_y = (content_area.height() - viewport.height()).max(0.0);
        scroll_x = scroll_x.clamp(0.0, max_scroll_x);
        scroll_y = scroll_y.clamp(0.0, max_scroll_y);
    }

    Some(state.with_scroll(scroll_x, scroll_y))
}
impl Engine {
    pub fn new() -> Self {
        Self {
            font_system: text_system::font_system(),
            cache: MeasureCache::new(MEASURE_CACHE_CAPACITY),
            text_area_line_displays: text_area_line_display_cache(),
            text_area_render_buffers: text_area_render_buffer_cache(),
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
    fn invalidate_text_area_height_indices_for_impacts(
        &mut self,
        buffer: &Buffer,
        impacts: &[TextEditImpact],
    ) {
        if impacts.is_empty() {
            return;
        }
        let buffer_id = buffer.id();
        let keys = self
            .text_area_height_indices
            .iter()
            .filter(|(key, _)| key.buffer_id == buffer_id)
            .map(|(key, _)| key.clone())
            .collect::<Vec<_>>();
        let line_count = buffer.logical_line_count();
        for key in keys {
            if let Some(index) = self.text_area_height_indices.get_mut(&key) {
                for impact in impacts {
                    index.apply_edit_impact(impact, line_count);
                }
            }
        }
    }

    pub(crate) fn invalidate_text_area_for_edit(
        &mut self,
        buffer: &Buffer,
        impacts: &[TextEditImpact],
    ) {
        self.invalidate_text_area_height_indices_for_impacts(buffer, impacts);
        self.invalidate_text_area_surfaces_for(buffer);
    }

    pub(crate) fn invalidate_text_area_surfaces_for(&mut self, _buffer: &Buffer) {
        // Display cache keys use per-line layout identity. Retaining entries keeps
        // unrelated lines warm after edits while stale line revisions age out via LRU.
    }
    pub fn text_field_layout(
        &mut self,
        buffer: &Buffer,
        style: Style,
        area: area::Logical,
        state: TextViewState,
    ) -> TextFieldLayout {
        self.text_field_layout_at(buffer, style, area, state, Instant::now())
    }
    pub fn text_field_layout_for_field(
        &mut self,
        field: &Field,
        style: Style,
        area: area::Logical,
        state: TextViewState,
    ) -> TextFieldLayout {
        self.text_field_layout_for_field_at(field, style, area, state, Instant::now())
    }
    pub fn text_field_layout_at(
        &mut self,
        buffer: &Buffer,
        style: Style,
        area: area::Logical,
        state: TextViewState,
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
            state.field_scroll_x(),
            0.0,
        );
        self.add_highlight_stats(stats);
        let caret = (!projection.buffer.has_non_empty_selection() && state.caret_visible(now))
            .then(|| {
                cursor_position(&prepared, projection.buffer.cursor()).map(|(x, y)| Caret {
                    x: x as f32 - state.field_scroll_x(),
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
            scroll_x: state.field_scroll_x(),
            scroll_y: 0.0,
            content_area: buffer_content_area(&prepared),
        }
    }
    pub fn text_field_layout_for_field_at(
        &mut self,
        field: &Field,
        style: Style,
        area: area::Logical,
        state: TextViewState,
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
        state: TextViewState,
        now: Instant,
    ) -> TextFieldLayout {
        match surface {
            Surface::Field(field) => {
                self.text_field_layout_for_field_at(field, style, area, state, now)
            }
            Surface::Area(area_model) => {
                self.text_area_paint_layout_for_area_at(area_model, style, area, state, now)
                    .into_interaction_parts()
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
        state: TextViewState,
    ) -> Option<TextPosition> {
        let projection = PreeditProjection::new(buffer, &state);
        let (prepared, vertical_offset) =
            self.prepare_text_field_buffer(&projection.buffer, style, area);
        TextLayoutMap::from_line_starts(projection.buffer.line_start_offsets()).hit_with_observer(
            &prepared,
            position.x() + state.field_scroll_x(),
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
        state: TextViewState,
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
        state: TextViewState,
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
        state: TextViewState,
    ) -> Option<Caret> {
        self.text_field_layout_at(buffer, style, area, state, Instant::now())
            .caret()
    }
    pub fn text_field_caret_for_field(
        &mut self,
        field: &Field,
        style: Style,
        area: area::Logical,
        state: TextViewState,
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
        state: TextViewState,
    ) -> Option<Caret> {
        self.text_area_paint_layout_for_area_at(area_model, style, area, state, Instant::now())
            .into_interaction_parts()
            .0
            .caret()
    }
    pub fn ensure_caret_visible(
        &mut self,
        buffer: &Buffer,
        style: Style,
        area: area::Logical,
        state: TextViewState,
    ) -> TextViewState {
        let projection = PreeditProjection::new(buffer, &state);
        let (prepared, vertical_offset) =
            self.prepare_text_field_buffer(&projection.buffer, style, area);
        let content_area = buffer_content_area(&prepared);
        let max_scroll_x = (content_area.width() - area.width().max(0.0)).max(0.0);
        let Some((caret_x, caret_y)) = cursor_position(&prepared, projection.buffer.cursor())
        else {
            return state
                .clone()
                .with_field_scroll_x(state.field_scroll_x().clamp(0.0, max_scroll_x));
        };
        let caret_layout = CaretLayout::new(Caret::new(
            caret_x as f32 - state.field_scroll_x(),
            vertical_offset + caret_y as f32,
            prepared.metrics().line_height,
        ));
        ensure_caret_visible_from_layout(state.clone(), area, caret_layout, Some(content_area))
            .unwrap_or(state)
    }
    pub fn ensure_caret_visible_for_field(
        &mut self,
        field: &Field,
        style: Style,
        area: area::Logical,
        state: TextViewState,
    ) -> TextViewState {
        let projection = FieldProjection::new(field);
        self.ensure_caret_visible(
            &projection.buffer,
            style,
            area,
            projected_state_for_field(field, state),
        )
    }
    pub fn ensure_caret_visible_for_area(
        &mut self,
        area_model: &Area,
        style: Style,
        viewport: area::Logical,
        state: TextViewState,
        observed_layout: Option<&TextFieldLayout>,
    ) -> TextViewState {
        if state.reveal_intent().should_ensure_caret_visible()
            && let Some(layout) = observed_layout
            && let Some(caret_layout) = layout.caret_layout()
            && let Some(next) = ensure_caret_visible_from_layout(
                state.clone(),
                viewport,
                caret_layout,
                Some(layout.content_area()),
            )
        {
            return next;
        }

        let projection = PreeditProjection::new(area_model.buffer(), &state);
        let source = &projection.buffer;
        let committed = !projection.has_preedit();

        if state.reveal_intent().should_ensure_caret_visible() {
            let segments = self
                .text_area_display_segments(area_model, source, committed, style, viewport, &state);
            if let Some(caret_layout) = self.text_area_caret_layout_from_segments(
                area_model,
                &projection,
                &state,
                &segments,
            ) && let Some(next) =
                ensure_caret_visible_from_layout(state.clone(), viewport, caret_layout, None)
            {
                return next;
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
        height_index.sync(source, line_count, estimated_line_height);

        let display = self.text_area_line_display(
            area_model,
            source,
            committed,
            style,
            viewport,
            cursor_line,
        );
        height_index.update_line(source, cursor_line, display.height.max(1.0));
        let caret_line_top = height_index.line_top(cursor_line);
        let content_height = height_index.total_height().max(viewport.height().max(0.0));
        if committed {
            self.text_area_height_indices.put(height_key, height_index);
        }

        let caret_layout = {
            let buffer = display.buffer.borrow();
            cursor_position(&buffer, Cursor::new(0, source.cursor().index)).map(|(x, y)| {
                CaretLayout::new(Caret::new(
                    x as f32 - state.scroll_x(),
                    caret_line_top + y as f32 - state.scroll_y(),
                    buffer.metrics().line_height.max(1.0),
                ))
            })
        };

        let content_area = area::logical(display.width.max(viewport.width()), content_height);
        if let Some(caret_layout) = caret_layout
            && let Some(next) = ensure_caret_visible_from_layout(
                state.clone(),
                viewport,
                caret_layout,
                Some(content_area),
            )
        {
            return next;
        }

        let max_scroll_x = (content_area.width() - viewport.width()).max(0.0);
        let max_scroll_y = (content_area.height() - viewport.height()).max(0.0);
        let scroll_x = state.scroll_x().clamp(0.0, max_scroll_x);
        let scroll_y = state.scroll_y().clamp(0.0, max_scroll_y);
        state.with_scroll(scroll_x, scroll_y)
    }
    pub fn text_area_scroll_y_for_anchor(
        &mut self,
        area_model: &Area,
        style: Style,
        viewport: area::Logical,
        _state: TextViewState,
        anchor: ScrollAnchor,
    ) -> Option<f32> {
        let source = area_model.buffer();
        let anchor_position = source.position_for_mark(anchor.mark())?;
        let line_count = source.logical_line_count().max(1);
        let anchor_line = source
            .cursor_for_text_index(anchor_position.index)
            .line
            .min(line_count.saturating_sub(1));
        let estimated_line_height = text_area_estimated_line_height(style);
        let height_key = TextAreaHeightKey::new(area_model, source, style, viewport.width());
        let mut height_index = self
            .text_area_height_indices
            .pop(&height_key)
            .unwrap_or_else(|| TextAreaHeightIndex::new(line_count, estimated_line_height));
        height_index.sync(source, line_count, estimated_line_height);

        let display =
            self.text_area_line_display(area_model, source, true, style, viewport, anchor_line);
        height_index.update_line(source, anchor_line, display.height.max(1.0));
        let scroll_y = (height_index.line_top(anchor_line) + anchor.offset_y()).max(0.0);
        self.text_area_height_indices.put(height_key, height_index);

        Some(scroll_y)
    }

    pub fn ensure_caret_visible_for_surface(
        &mut self,
        surface: &Surface,
        style: Style,
        area: area::Logical,
        state: TextViewState,
    ) -> TextViewState {
        match surface {
            Surface::Field(field) => self.ensure_caret_visible_for_field(field, style, area, state),
            Surface::Area(area_model) => {
                self.ensure_caret_visible_for_area(area_model, style, area, state, None)
            }
        }
    }
    pub(super) fn prepare_text_field_buffer(
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
        state: TextViewState,
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
        let content_width = match area_model.wrap() {
            AreaWrap::None => self
                .text_area_display_segments(
                    area_model,
                    &projection.buffer,
                    !projection.has_preedit(),
                    style,
                    viewport,
                    &state,
                )
                .iter()
                .map(|segment| segment.display.width)
                .fold(viewport.width().max(0.0), f32::max),
            AreaWrap::WordOrGlyph => viewport.width().max(0.0),
        };
        let content_area = area::logical(content_width, content_height);
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
        state: TextViewState,
        now: Instant,
    ) -> TextAreaPaintLayout {
        self.text_area_layout_for_area_at_with_observation(
            area_model,
            style,
            viewport,
            state,
            now,
            TextAreaObservation::Observe,
            None,
        )
    }

    pub fn text_area_render_layout_for_area_at(
        &mut self,
        area_model: &Area,
        style: Style,
        viewport: area::Logical,
        state: TextViewState,
        now: Instant,
        content_area: area::Logical,
    ) -> TextAreaPaintLayout {
        self.text_area_layout_for_area_at_with_observation(
            area_model,
            style,
            viewport,
            state,
            now,
            TextAreaObservation::RenderOnly,
            Some(content_area),
        )
    }

    pub fn text_area_overlay_layout_for_area_at(
        &mut self,
        area_model: &Area,
        style: Style,
        viewport: area::Logical,
        state: TextViewState,
        now: Instant,
        content_area: area::Logical,
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
        let layout = self.text_area_layout_from_segments(
            area_model,
            style,
            viewport,
            &state,
            now,
            &projection,
            &segments,
            Some(content_area),
        );
        self.diagnostics.text_area_interaction_surfaces += segments.len();
        layout
    }

    pub fn text_area_overlay_layout_for_surfaces_at(
        &mut self,
        area_model: &Area,
        state: TextViewState,
        now: Instant,
        content_area: area::Logical,
        surfaces: &[TextAreaSurface],
    ) -> TextFieldLayout {
        let projection = PreeditProjection::new(area_model.buffer(), &state);
        let selection = projection.buffer.selection_bounds();
        let (preedit_underline, preedit_selection) = projection.highlight_ranges();
        let mut spans = HighlightSpans::default();
        #[allow(unused_mut)]
        let mut combined_stats = HighlightStats::default();
        let mut caret = None;

        for surface in surfaces {
            let buffer = surface.buffer.borrow();
            let selection = selection.and_then(|range| {
                local_cursor_range_for_source_line(
                    range,
                    surface.source_line,
                    surface.source_text_len,
                )
            });
            let preedit_underline = preedit_underline.and_then(|range| {
                local_cursor_range_for_source_line(
                    range,
                    surface.source_line,
                    surface.source_text_len,
                )
            });
            let preedit_selection = preedit_selection.and_then(|range| {
                local_cursor_range_for_source_line(
                    range,
                    surface.source_line,
                    surface.source_text_len,
                )
            });
            let (line_spans, stats) = highlight_spans_for_ranges(
                &buffer,
                selection,
                preedit_underline,
                preedit_selection,
                surface.y,
                -surface.x,
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
                && projection.buffer.cursor().line == surface.source_line
            {
                let cursor = Cursor::new(0, projection.buffer.cursor().index);
                caret = cursor_position(&buffer, cursor).map(|(x, y)| Caret {
                    x: x as f32 + surface.x,
                    y: surface.y + y as f32,
                    height: buffer.metrics().line_height,
                });
            }
        }

        self.add_highlight_stats(combined_stats);
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

    fn text_area_layout_for_area_at_with_observation(
        &mut self,
        area_model: &Area,
        style: Style,
        viewport: area::Logical,
        state: TextViewState,
        now: Instant,
        observation: TextAreaObservation,
        content_area: Option<area::Logical>,
    ) -> TextAreaPaintLayout {
        self.diagnostics.text_area_paint_layout_calls += 1;
        let projection = PreeditProjection::new(area_model.buffer(), &state);
        let committed = !projection.has_preedit();

        let observe = observation == TextAreaObservation::Observe;

        let segments = if observe {
            self.text_area_display_segments(
                area_model,
                &projection.buffer,
                committed,
                style,
                viewport,
                &state,
            )
        } else {
            self.record_text_area_render_window(
                area_model,
                &projection.buffer,
                committed,
                style,
                viewport,
                &state,
            );
            Vec::new()
        };
        let layout = self.text_area_layout_from_segments(
            area_model,
            style,
            viewport,
            &state,
            now,
            &projection,
            &segments,
            content_area,
        );
        self.diagnostics.text_area_interaction_surfaces += segments.len();
        let surfaces = segments
            .iter()
            .map(|segment| text_area_surface_for_segment(segment, style, viewport, &state))
            .collect();
        let render_surfaces = self.text_area_render_surfaces(
            area_model,
            &projection.buffer,
            committed,
            style,
            viewport,
            &state,
        );
        TextAreaPaintLayout {
            layout,
            interaction_surfaces: surfaces,
            render_surfaces,
        }
    }

    #[allow(dead_code)]
    fn text_area_layout_for_area_at(
        &mut self,
        area_model: &Area,
        style: Style,
        viewport: area::Logical,
        state: TextViewState,
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
            None,
        )
    }

    fn text_area_layout_from_segments(
        &mut self,
        area_model: &Area,
        style: Style,
        viewport: area::Logical,
        state: &TextViewState,
        now: Instant,
        projection: &PreeditProjection,
        segments: &[TextAreaDisplaySegment],
        content_area: Option<area::Logical>,
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
        let content_area = content_area.unwrap_or_else(|| {
            area::logical(
                text_area_content_width(area_model.wrap(), viewport, observed_width),
                self.text_area_content_height(
                    area_model,
                    &projection.buffer,
                    !projection.has_preedit(),
                    style,
                    viewport,
                ),
            )
        });
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
        state: &TextViewState,
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
            if let Some(index) = self.text_area_height_indices.get_mut(&key) {
                index.sync(source, line_count, estimated_line_height);
                self.diagnostics.text_area_height_index_hits += 1;
                return index.total_height().max(viewport.height().max(0.0));
            }
            self.diagnostics.text_area_height_index_misses += 1;
        }
        (line_count as f32 * estimated_line_height).max(viewport.height().max(0.0))
    }

    fn text_area_render_surfaces(
        &mut self,
        area_model: &Area,
        source: &Buffer,
        committed: bool,
        style: Style,
        viewport: area::Logical,
        state: &TextViewState,
    ) -> Vec<TextAreaSurface> {
        let total_started = Instant::now();
        self.diagnostics.text_area_render_surface_calls += 1;

        let anchor_started = Instant::now();
        let Some(anchor) =
            self.text_area_render_anchor(area_model, source, committed, style, viewport, state)
        else {
            self.diagnostics.text_area_render_surface_anchor_us += elapsed_micros(anchor_started);
            self.diagnostics.text_area_render_surface_total_us += elapsed_micros(total_started);
            return Vec::new();
        };
        let anchor_us = elapsed_micros(anchor_started);
        self.diagnostics.text_area_render_surface_anchor_us += anchor_us;

        let font_size = style.size().max(1.0);
        let metrics = glyphon::Metrics::relative(font_size, 1.25);
        let surface_height = anchor.height.max(viewport.height() - anchor.y).max(1.0);
        let surface_width = viewport.width().max(1.0)
            + state.scroll_x().max(0.0)
            + TEXT_AREA_RENDER_HORIZONTAL_OVERSCAN;
        let layout_width = match area_model.wrap() {
            AreaWrap::None => None,
            AreaWrap::WordOrGlyph => Some(viewport.width().max(0.0)),
        };
        let key_width = layout_width.unwrap_or(0.0);
        let key = TextAreaRenderBufferKey::new(
            area_model,
            source,
            style,
            key_width,
            anchor.source_line,
            anchor.source_line_end,
        );
        let source_lines = anchor.source_line_end.saturating_sub(anchor.source_line);
        self.diagnostics.text_area_render_surface_source_lines += source_lines;

        let mut cache_hit = false;
        let mut text_us = 0;
        let mut buffer_us = 0;
        let mut attrs_us = 0;
        let mut size_us = 0;
        let mut shape_us = 0;
        let buffer = if committed && let Some(cached) = self.text_area_render_buffers.get(&key) {
            cache_hit = true;
            self.diagnostics.text_area_render_surface_cache_hits += 1;
            cached.buffer.clone()
        } else {
            self.diagnostics.text_area_render_surface_cache_misses += 1;
            let attrs = text_system::attrs_for_style(style);

            let text_started = Instant::now();
            let text = source.text_for_line_range(anchor.source_line, anchor.source_line_end);
            text_us = elapsed_micros(text_started);
            self.diagnostics.text_area_render_surface_text_us += text_us;

            let buffer_started = Instant::now();
            let mut buffer = glyphon::Buffer::new_empty(metrics);
            buffer_us = elapsed_micros(buffer_started);

            let size_started = Instant::now();
            buffer.set_wrap(&mut self.font_system, area_model.wrap().into());
            buffer.set_size(&mut self.font_system, layout_width, None);
            size_us = elapsed_micros(size_started);
            self.diagnostics.text_area_render_surface_size_us += size_us;

            let attrs_started = Instant::now();
            let attrs = glyphon::AttrsList::new(&attrs);
            attrs_us = elapsed_micros(attrs_started);
            self.diagnostics.text_area_render_surface_attrs_us += attrs_us;

            let buffer_started = Instant::now();
            let shaping = text_area_shaping_for_text(style, &text);
            set_cosmic_buffer_text(&mut buffer, &text, attrs, shaping);
            buffer_us += elapsed_micros(buffer_started);
            self.diagnostics.text_area_render_surface_buffer_us += buffer_us;

            let shape_started = Instant::now();
            buffer.shape_until_scroll(&mut self.font_system, false);
            shape_us = elapsed_micros(shape_started);
            self.diagnostics.text_area_render_surface_shape_us += shape_us;

            let buffer = Rc::new(RefCell::new(buffer));
            if committed {
                self.text_area_render_buffers.put(
                    key,
                    CachedTextAreaRenderBuffer {
                        buffer: buffer.clone(),
                    },
                );
            }
            buffer
        };
        let metadata_started = Instant::now();
        let (source_start, source_text_len) = {
            let inner = source.inner.borrow();
            let start = inner.document.line_start(anchor.source_line);
            let end = inner
                .document
                .line_start(anchor.source_line_end.min(source.logical_line_count()));
            (start, end.saturating_sub(start))
        };
        let metadata_us = elapsed_micros(metadata_started);
        self.diagnostics.text_area_render_surface_metadata_us += metadata_us;
        self.diagnostics.text_area_render_surface_source_bytes += source_text_len;
        let total_us = elapsed_micros(total_started);
        self.diagnostics.text_area_render_surface_total_us += total_us;

        if std::env::var_os("WGPU_L3_SCROLL_TRACE").is_some() {
            eprintln!(
                concat!(
                    "[wgpu_l3 text-surface] ",
                    "lines={}..{} count={} bytes={} cache_hit={} ",
                    "viewport={:.1}x{:.1} scroll={:.1},{:.1} surface={:.1}x{:.1} ",
                    "anchor={}us text={}us buffer={}us attrs={}us size={}us shape={}us meta={}us total={}us"
                ),
                anchor.source_line,
                anchor.source_line_end,
                source_lines,
                source_text_len,
                cache_hit,
                viewport.width(),
                viewport.height(),
                state.scroll_x(),
                state.scroll_y(),
                surface_width,
                surface_height,
                anchor_us,
                text_us,
                buffer_us,
                attrs_us,
                size_us,
                shape_us,
                metadata_us,
                total_us,
            );
        }

        vec![TextAreaSurface {
            x: -state.scroll_x(),
            y: anchor.y,
            width: surface_width,
            height: surface_height,
            source_line: anchor.source_line,
            source_line_id: source
                .line_layout_identity(anchor.source_line)
                .map(|identity| identity.id),
            source_start,
            source_text_len,
            buffer,
            default_color: style.color(),
        }]
    }

    fn text_area_render_anchor(
        &mut self,
        area_model: &Area,
        source: &Buffer,
        committed: bool,
        style: Style,
        viewport: area::Logical,
        state: &TextViewState,
    ) -> Option<TextAreaRenderAnchor> {
        let line_count = source.logical_line_count().max(1);
        let estimated_line_height = text_area_estimated_line_height(style);
        let height_key = TextAreaHeightKey::new(area_model, source, style, viewport.width());
        let mut height_index = if committed {
            self.text_area_height_indices
                .pop(&height_key)
                .unwrap_or_else(|| TextAreaHeightIndex::new(line_count, estimated_line_height))
        } else {
            TextAreaHeightIndex::new(line_count, estimated_line_height)
        };
        height_index.sync(source, line_count, estimated_line_height);

        let scroll_y = state.scroll_y().max(0.0);
        let visible_line = height_index.line_at_y(scroll_y);
        let visible_lines = height_index.visible_line_count(scroll_y, viewport.height());
        let visible_line_end = visible_line.saturating_add(visible_lines).min(line_count);
        let window = text_area_render_line_window(visible_line, visible_line_end, line_count);
        let source_line = window.start;
        let source_line_end = window.end;
        let y = height_index.line_top(source_line) - scroll_y;
        let height =
            (height_index.line_top(source_line_end) - height_index.line_top(source_line)).max(1.0);

        if committed {
            self.text_area_height_indices.put(height_key, height_index);
        }

        Some(TextAreaRenderAnchor {
            source_line,
            source_line_end,
            y,
            height,
        })
    }

    fn record_text_area_render_window(
        &mut self,
        area_model: &Area,
        source: &Buffer,
        committed: bool,
        style: Style,
        viewport: area::Logical,
        state: &TextViewState,
    ) {
        let line_count = source.logical_line_count().max(1);
        let estimated_line_height = text_area_estimated_line_height(style);
        let height_key = TextAreaHeightKey::new(area_model, source, style, viewport.width());
        let mut height_index = if committed {
            self.text_area_height_indices
                .pop(&height_key)
                .unwrap_or_else(|| TextAreaHeightIndex::new(line_count, estimated_line_height))
        } else {
            TextAreaHeightIndex::new(line_count, estimated_line_height)
        };
        height_index.sync(source, line_count, estimated_line_height);
        let scroll_y = state.scroll_y().max(0.0);
        let visible_lines = height_index.visible_line_count(scroll_y, viewport.height());
        self.diagnostics.text_area_visible_logical_lines += visible_lines;

        if committed {
            self.text_area_height_indices.put(height_key, height_index);
        }
    }

    pub(super) fn text_area_line_display(
        &mut self,
        area_model: &Area,
        source: &Buffer,
        committed: bool,
        style: Style,
        viewport: area::Logical,
        source_line: usize,
    ) -> TextAreaLineDisplay {
        let key = TextAreaLineDisplayKey::new(
            area_model,
            source,
            style,
            viewport.width().max(0.0),
            source_line,
        );
        if committed
            && let Some(key) = key.as_ref()
            && let Some(cached) = self.text_area_line_displays.get(key)
        {
            self.diagnostics.text_area_line_cache_hits += 1;
            #[cfg(test)]
            {
                self.interaction_stats.text_area_frame_cache_hits += 1;
            }
            return TextAreaLineDisplay::from_cached(source, source_line, cached.clone());
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
        let mut buffer = glyphon::Buffer::new_empty(metrics);
        buffer.set_wrap(&mut self.font_system, area_model.wrap().into());
        buffer.set_size(
            &mut self.font_system,
            match area_model.wrap() {
                AreaWrap::None => None,
                AreaWrap::WordOrGlyph => Some(viewport.width().max(0.0)),
            },
            None,
        );
        let shaping = text_area_shaping_for_text(style, &text);
        set_cosmic_buffer_text(&mut buffer, &text, glyphon::AttrsList::new(&attrs), shaping);
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
        let cached = CachedTextAreaLineDisplay {
            buffer: Rc::new(RefCell::new(buffer)),
            height: content.height(),
            width: content.width(),
        };
        let display = TextAreaLineDisplay::from_cached(source, source_line, cached.clone());
        if committed && let Some(key) = key {
            self.text_area_line_displays.put(key, cached);
        }
        display
    }

    fn cached_text_area_line_display(
        &mut self,
        area_model: &Area,
        source: &Buffer,
        committed: bool,
        style: Style,
        viewport: area::Logical,
        source_line: usize,
    ) -> Option<TextAreaLineDisplay> {
        if !committed {
            return None;
        }
        let key = TextAreaLineDisplayKey::new(
            area_model,
            source,
            style,
            viewport.width().max(0.0),
            source_line,
        )?;
        let cached = self.text_area_line_displays.get(&key)?;
        self.diagnostics.text_area_line_cache_hits += 1;
        #[cfg(test)]
        {
            self.interaction_stats.text_area_frame_cache_hits += 1;
        }
        Some(TextAreaLineDisplay::from_cached(
            source,
            source_line,
            cached.clone(),
        ))
    }

    fn text_area_display_segments(
        &mut self,
        area_model: &Area,
        source: &Buffer,
        committed: bool,
        style: Style,
        viewport: area::Logical,
        state: &TextViewState,
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
        height_index.sync(source, line_count, estimated_line_height);

        let scroll_y = state.scroll_y().max(0.0);
        let first_visible = height_index.line_at_y(scroll_y);
        let visible_lines = height_index.visible_line_count(scroll_y, viewport.height());
        let visible_line_end = first_visible
            .saturating_add(visible_lines)
            .saturating_add(1)
            .min(line_count);
        let overscan = TEXT_AREA_FRAME_MIN_OVERSCAN_LINES;
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
        let shape_overscan =
            state.caret_visibility_pending() || source.has_non_empty_selection() || !committed;
        for line in source_line_start..source_line_end {
            let visible = line >= first_visible && line < visible_line_end;
            let display =
                if visible || shape_overscan {
                    Some(self.text_area_line_display(
                        area_model, source, committed, style, viewport, line,
                    ))
                } else {
                    self.cached_text_area_line_display(
                        area_model, source, committed, style, viewport, line,
                    )
                };
            let segment_y = y;
            let display_height = display
                .as_ref()
                .map(|display| display.height.max(1.0))
                .unwrap_or_else(|| height_index.line_height(line));
            if let Some(display) = display {
                height_index.update_line(source, line, display_height);
                segments.push(TextAreaDisplaySegment {
                    display,
                    y: segment_y,
                });
            }
            y += display_height;
        }
        self.diagnostics.text_area_visible_logical_lines += visible_lines;
        self.diagnostics.text_area_layout_segments += segments.len();
        self.diagnostics.text_area_overscan_segments +=
            segments.len().saturating_sub(visible_lines);

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
        state: TextViewState,
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
        self.text_area_position_at_for_segments(
            &segments,
            position,
            state.scroll_x(),
            projection.buffer.len(),
        )
    }

    pub fn text_area_position_at_for_paint_layout(
        &mut self,
        area_model: &Area,
        position: point::Logical,
        state: TextViewState,
        observed_layout: &TextAreaPaintLayout,
    ) -> Option<TextPosition> {
        let projection = PreeditProjection::new(area_model.buffer(), &state);
        self.text_area_position_at_for_surfaces(
            observed_layout.interaction_surfaces(),
            position,
            observed_layout.layout.scroll_x(),
            projection.buffer.len(),
        )
    }

    pub fn text_area_position_at_for_observed_surfaces(
        &mut self,
        area_model: &Area,
        position: point::Logical,
        state: TextViewState,
        scroll_x: f32,
        observed_surfaces: &[TextAreaSurface],
    ) -> Option<TextPosition> {
        let projection = PreeditProjection::new(area_model.buffer(), &state);
        self.text_area_position_at_for_surfaces(
            observed_surfaces,
            position,
            scroll_x,
            projection.buffer.len(),
        )
    }

    fn text_area_position_at_for_segments(
        &mut self,
        segments: &[TextAreaDisplaySegment],
        position: point::Logical,
        scroll_x: f32,
        text_len: usize,
    ) -> Option<TextPosition> {
        if segments.is_empty() {
            return Some(TextPosition::new(0));
        }

        let mut nearest = None::<(f32, &TextAreaDisplaySegment)>;
        for segment in segments {
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
        self.text_area_position_in_line_buffer(
            &buffer,
            segment.display.source_start,
            position.x() + scroll_x,
            position.y() - segment.y,
            text_len,
        )
    }

    fn text_area_position_at_for_surfaces(
        &mut self,
        surfaces: &[TextAreaSurface],
        position: point::Logical,
        scroll_x: f32,
        text_len: usize,
    ) -> Option<TextPosition> {
        if surfaces.is_empty() {
            return Some(TextPosition::new(0));
        }

        let mut nearest = None::<(f32, &TextAreaSurface)>;
        for surface in surfaces {
            let top = surface.y;
            let bottom = surface.y + surface.height.max(1.0);
            if position.y() >= top && position.y() <= bottom {
                nearest = Some((0.0, surface));
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
                nearest = Some((distance, surface));
            }
        }

        let surface = nearest.map(|(_, surface)| surface)?;
        let buffer = surface.buffer.borrow();
        self.text_area_position_in_line_buffer(
            &buffer,
            surface.source_start,
            position.x() + scroll_x,
            position.y() - surface.y,
            text_len,
        )
    }

    fn text_area_position_in_line_buffer(
        &mut self,
        buffer: &glyphon::Buffer,
        source_start: usize,
        x: f32,
        y: f32,
        text_len: usize,
    ) -> Option<TextPosition> {
        let map = TextLayoutMap::from_line_starts(Rc::new(vec![source_start]));
        let local = map.hit_with_observer(buffer, x, y, |runs| {
            self.diagnostics.text_area_hit_run_scans += runs;
            #[cfg(test)]
            {
                self.interaction_stats.hit_run_scans += runs;
            }
        })?;
        Some(TextPosition::with_affinity(
            local.index.min(text_len),
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
    pub(super) fn with_cache_capacity(capacity: usize) -> Self {
        Self {
            cache: MeasureCache::new(capacity),
            ..Self::new()
        }
    }
    #[cfg(test)]
    pub(super) fn reset_highlight_stats(&mut self) {
        self.highlight_stats = HighlightStats::default();
    }
    #[cfg(test)]
    pub(super) fn highlight_stats(&self) -> HighlightStats {
        self.highlight_stats
    }
    #[cfg(test)]
    pub(super) fn reset_interaction_stats(&mut self) {
        self.interaction_stats = TextInteractionStats::default();
    }
    #[cfg(test)]
    pub(super) fn interaction_stats(&self) -> TextInteractionStats {
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

fn text_area_render_buffer_cache() -> LruCache<TextAreaRenderBufferKey, CachedTextAreaRenderBuffer>
{
    LruCache::new(
        NonZeroUsize::new(TEXT_AREA_RENDER_BUFFER_CACHE_CAPACITY)
            .expect("text area render buffer cache capacity must be non-zero"),
    )
}

fn text_area_height_index_cache() -> LruCache<TextAreaHeightKey, TextAreaHeightIndex> {
    LruCache::new(
        NonZeroUsize::new(TEXT_AREA_HEIGHT_INDEX_CACHE_CAPACITY)
            .expect("text area height index cache capacity must be non-zero"),
    )
}

fn text_area_surface_for_segment(
    segment: &TextAreaDisplaySegment,
    style: Style,
    viewport: area::Logical,
    state: &TextViewState,
) -> TextAreaSurface {
    TextAreaSurface {
        x: -state.scroll_x(),
        y: segment.y,
        width: segment.display.width.max(viewport.width()) + state.scroll_x().max(0.0),
        height: segment.display.height.max(1.0),
        source_line: segment.display.source_line,
        source_line_id: segment.display.source_line_id,
        source_start: segment.display.source_start,
        source_text_len: segment.display.source_text_len,
        buffer: segment.display.buffer.clone(),
        default_color: style.color(),
    }
}

fn text_area_render_line_window(
    visible_start: usize,
    visible_end: usize,
    line_count: usize,
) -> TextAreaRenderLineWindow {
    TextAreaRenderLineWindow::new(visible_start, visible_end, line_count)
}

fn text_area_render_guard_lines(_visible_lines: usize) -> usize {
    TEXT_AREA_RENDER_GUARD_LINES
}

pub(super) fn text_area_estimated_line_height(style: Style) -> f32 {
    glyphon::Metrics::relative(style.size().max(1.0), 1.25)
        .line_height
        .max(1.0)
}

fn text_area_shaping_for_text(_style: Style, _text: &str) -> glyphon::Shaping {
    glyphon::Shaping::Advanced
}

pub(super) struct TextLayoutMap {
    pub(super) line_starts: Rc<Vec<usize>>,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub(super) struct VisualLineGroup {
    pub(super) start: usize,
    pub(super) end: usize,
    pub(super) top: f32,
    pub(super) bottom: f32,
}

impl TextLayoutMap {
    #[cfg(test)]
    pub(super) fn new(buffer: &glyphon::Buffer) -> Self {
        Self {
            line_starts: Rc::new(line_start_offsets_for_buffer(buffer)),
        }
    }

    pub(super) fn from_line_starts(line_starts: Rc<Vec<usize>>) -> Self {
        Self { line_starts }
    }

    #[cfg(test)]
    pub(super) fn hit(&self, buffer: &glyphon::Buffer, x: f32, y: f32) -> Option<TextPosition> {
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

    pub(super) fn visual_line_groups(
        runs: &[glyphon::cosmic_text::LayoutRun<'_>],
    ) -> Vec<VisualLineGroup> {
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

    pub(super) fn run_visual_bounds(
        run: &glyphon::cosmic_text::LayoutRun<'_>,
    ) -> Option<(f32, f32)> {
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

    pub(super) fn run_edge_position(
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
    wrap: AreaWrap,
    base: area::Logical,
    hint: Option<area::Logical>,
    observed: area::Logical,
    viewport: area::Logical,
) -> area::Logical {
    let hint = hint.unwrap_or(base);
    let width = match wrap {
        AreaWrap::None => viewport
            .width()
            .max(base.width())
            .max(hint.width())
            .max(observed.width()),
        AreaWrap::WordOrGlyph => viewport.width().max(0.0),
    };

    area::logical(
        width,
        viewport
            .height()
            .max(base.height())
            .max(hint.height())
            .max(observed.height()),
    )
}

fn text_area_content_width(wrap: AreaWrap, viewport: area::Logical, observed_width: f32) -> f32 {
    match wrap {
        AreaWrap::None => observed_width.max(viewport.width().max(0.0)),
        AreaWrap::WordOrGlyph => viewport.width().max(0.0),
    }
}

fn elapsed_micros(start: Instant) -> u128 {
    start.elapsed().as_micros()
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
