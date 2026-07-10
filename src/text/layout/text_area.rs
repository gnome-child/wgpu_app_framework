use std::cell::RefCell;
use std::num::NonZeroUsize;
use std::rc::Rc;

use lru::LruCache;

use super::super::buffer::{Buffer, LineId, LineLayoutIdentity};
use super::super::document::{Style, TextDirection};
use super::super::edit::{Area, AreaWrap, ViewState};
use super::constants::{
    TEXT_AREA_LINE_DISPLAY_CACHE_CAPACITY, TEXT_AREA_RENDER_BUFFER_CACHE_CAPACITY,
    TEXT_AREA_RENDER_GUARD_LINES, TEXT_AREA_RENDER_MAX_WINDOW_LINES,
};
use super::key::{StyleKey, finite_bits};
use super::output::TextAreaSurface;
use super::shaping_cache::ShapingCache;
use crate::paint;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum Observation {
    RenderOnly,
    Observe,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub(super) struct RenderAnchor {
    pub(super) source_line: usize,
    pub(super) source_line_end: usize,
    pub(super) y: f32,
    pub(super) height: f32,
}

#[derive(Clone)]
pub(in crate::text) struct CachedLineDisplay {
    pub(in crate::text) buffer: Rc<RefCell<glyphon::Buffer>>,
    pub(in crate::text) height: f32,
    pub(in crate::text) width: f32,
}

#[derive(Clone)]
pub(in crate::text) struct LineDisplay {
    pub(in crate::text) buffer: Rc<RefCell<glyphon::Buffer>>,
    pub(in crate::text) source_line: usize,
    pub(in crate::text) source_line_id: Option<LineId>,
    pub(in crate::text) source_start: usize,
    pub(in crate::text) source_text_len: usize,
    pub(in crate::text) height: f32,
    pub(in crate::text) width: f32,
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

#[derive(Clone)]
pub(in crate::text) struct CachedRenderBuffer {
    pub(in crate::text) buffer: Rc<RefCell<glyphon::Buffer>>,
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
pub(in crate::text) struct RenderBufferKey {
    lines: Vec<LineLayoutIdentity>,
    style: StyleKey,
    width: u32,
    wrap: AreaWrap,
    direction: TextDirection,
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

impl RenderBufferKey {
    pub(super) fn new(
        area_model: &Area,
        buffer: &Buffer,
        style: Style,
        width: f32,
        source_line_start: usize,
        source_line_end: usize,
    ) -> Option<Self> {
        let source_line_end = source_line_end.min(buffer.logical_line_count());
        let lines = (source_line_start.min(source_line_end)..source_line_end)
            .map(|line| buffer.line_layout_identity(line))
            .collect::<Option<Vec<_>>>()?;

        Some(Self {
            lines,
            style: StyleKey::new(style),
            width: finite_bits(width.max(0.0)),
            wrap: area_model.wrap(),
            direction: style.direction(),
        })
    }
}

impl LineDisplay {
    pub(super) fn from_cached(
        source: &Buffer,
        source_line: usize,
        cached: CachedLineDisplay,
    ) -> Self {
        let (source_start, source_text_len) = line_source_metrics(source, source_line);
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

pub(super) fn line_display_cache() -> ShapingCache<LineDisplayKey, CachedLineDisplay> {
    ShapingCache::new(
        TEXT_AREA_LINE_DISPLAY_CACHE_CAPACITY,
        "text area line display",
    )
}

pub(super) fn render_buffer_cache() -> LruCache<RenderBufferKey, CachedRenderBuffer> {
    LruCache::new(
        NonZeroUsize::new(TEXT_AREA_RENDER_BUFFER_CACHE_CAPACITY)
            .expect("text area render buffer cache capacity must be non-zero"),
    )
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
    viewport: paint::area::Logical,
    state: &ViewState,
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
