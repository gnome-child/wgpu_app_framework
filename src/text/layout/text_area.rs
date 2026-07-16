use crate::geometry::area;
use std::cell::RefCell;
use std::rc::Rc;

use lru::LruCache;

use super::super::buffer::{Buffer, LineId, LineLayoutIdentity};
use super::super::document::{Style, TextDirection};
use super::super::{
    surface::{Area, AreaWrap},
    view::ViewState,
};
use super::constants::{
    TEXT_AREA_HORIZONTAL_INDEX_CACHE_CAPACITY, TEXT_AREA_LINE_DISPLAY_CACHE_CAPACITY,
    TEXT_AREA_RENDER_GUARD_LINES, TEXT_AREA_RENDER_MAX_WINDOW_LINES,
};
use super::horizontal::LineIndex as HorizontalLineIndex;
use super::key::{StyleKey, finite_bits};
use super::output::TextAreaSurface;
use super::shaping_cache::ShapingCache;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum Observation {
    RenderOnly,
    Observe,
}

#[derive(Clone)]
pub(in crate::text) struct CachedLineDisplay {
    pub(in crate::text) buffer: Rc<RefCell<glyphon::Buffer>>,
    pub(in crate::text) height: f32,
    pub(in crate::text) width: f32,
    pub(in crate::text) source_byte_start: usize,
    pub(in crate::text) source_text_len: usize,
    pub(in crate::text) source_x: f32,
    pub(in crate::text) glyph_count: usize,
    pub(in crate::text) resident_bytes: usize,
}

#[derive(Clone)]
pub(in crate::text) struct LineDisplay {
    pub(in crate::text) buffer: Rc<RefCell<glyphon::Buffer>>,
    pub(in crate::text) source_line: usize,
    pub(in crate::text) source_line_id: Option<LineId>,
    pub(in crate::text) source_start: usize,
    pub(in crate::text) source_line_byte_start: usize,
    pub(in crate::text) source_text_len: usize,
    pub(in crate::text) height: f32,
    pub(in crate::text) width: f32,
    pub(in crate::text) surface_x: f32,
    pub(in crate::text) surface_width: f32,
    pub(in crate::text) text_x: f32,
    pub(in crate::text) cache_hit: bool,
}

#[derive(Clone)]
pub(in crate::text) struct DisplaySegment {
    pub(super) display: LineDisplay,
    pub(super) y: f32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(in crate::text) struct RenderLineWindow {
    pub(super) start: usize,
    pub(super) end: usize,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub(in crate::text) struct LineDisplayKey {
    line: LineLayoutIdentity,
    style: StyleKey,
    width: u32,
    wrap: AreaWrap,
    direction: TextDirection,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub(in crate::text) struct LineWindowKey {
    line: LineDisplayKey,
    source_start: usize,
    source_end: usize,
}

impl LineDisplayKey {
    pub(super) fn new(
        area_model: &Area,
        buffer: &Buffer,
        style: Style,
        width: f32,
        source_line: usize,
    ) -> Option<Self> {
        Some(Self {
            line: buffer.line_layout_identity(source_line)?,
            style: StyleKey::new(style),
            width: finite_bits(width.max(0.0)),
            wrap: area_model.wrap(),
            direction: style.direction(),
        })
    }
}

impl LineWindowKey {
    pub(super) fn new(line: LineDisplayKey, source_start: usize, source_end: usize) -> Self {
        Self {
            line,
            source_start,
            source_end,
        }
    }
}

impl LineDisplay {
    pub(super) fn from_cached(
        source: &Buffer,
        source_line: usize,
        cached: CachedLineDisplay,
        surface_x: f32,
        surface_width: f32,
        scroll_x: f32,
        cache_hit: bool,
    ) -> Self {
        let (line_start, _) = line_source_metrics(source, source_line);
        let source_line_id = source
            .line_layout_identity(source_line)
            .map(|identity| identity.id);
        Self {
            buffer: cached.buffer,
            source_line,
            source_line_id,
            source_start: line_start.saturating_add(cached.source_byte_start),
            source_line_byte_start: cached.source_byte_start,
            source_text_len: cached.source_text_len,
            height: cached.height,
            width: cached.width,
            surface_x: surface_x - scroll_x,
            surface_width,
            text_x: cached.source_x - scroll_x,
            cache_hit,
        }
    }
}

pub(super) fn line_display_cache() -> ShapingCache<LineWindowKey, CachedLineDisplay> {
    ShapingCache::new(TEXT_AREA_LINE_DISPLAY_CACHE_CAPACITY)
}

pub(super) fn horizontal_index_cache() -> LruCache<LineDisplayKey, Rc<HorizontalLineIndex>> {
    LruCache::new(TEXT_AREA_HORIZONTAL_INDEX_CACHE_CAPACITY)
}

pub(super) fn render_line_window(
    visible_start: usize,
    visible_end: usize,
    line_count: usize,
) -> RenderLineWindow {
    RenderLineWindow::new(visible_start, visible_end, line_count)
}

pub(super) fn surface_for_segment(
    segment: &DisplaySegment,
    style: Style,
    _viewport: area::Logical,
    _state: &ViewState,
) -> TextAreaSurface {
    TextAreaSurface {
        x: segment.display.surface_x,
        y: segment.y,
        text_x: segment.display.text_x,
        width: segment.display.surface_width,
        height: segment.display.height.max(1.0),
        source_line: segment.display.source_line,
        source_line_id: segment.display.source_line_id,
        source_start: segment.display.source_start,
        source_line_byte_start: segment.display.source_line_byte_start,
        source_text_len: segment.display.source_text_len,
        buffer: segment.display.buffer.clone(),
        default_color: style.color(),
    }
}

impl RenderLineWindow {
    fn new(visible_start: usize, visible_end: usize, line_count: usize) -> Self {
        let line_count = line_count.max(1);
        let visible_start = visible_start.min(line_count.saturating_sub(1));
        let visible_end = visible_end.min(line_count).max(visible_start + 1);
        let visible_lines = visible_end.saturating_sub(visible_start).max(1);
        let guard_lines = render_guard_lines(visible_lines);
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

fn render_guard_lines(_visible_lines: usize) -> usize {
    TEXT_AREA_RENDER_GUARD_LINES
}

fn line_source_metrics(source: &Buffer, source_line: usize) -> (usize, usize) {
    let inner = &source.inner;
    (
        inner.document.line_start(source_line),
        inner.document.line_text_len(source_line),
    )
}
