use std::cell::RefCell;
use std::fmt;
use std::rc::Rc;

use super::super::Color;
use super::super::buffer::LineId;
use super::caret::{Caret, CaretLayout};
use super::highlight::SelectionSpan;
use crate::geometry::{area, point};

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Measure {
    pub(in crate::text) max: Option<area::Logical>,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Metrics {
    pub(in crate::text) area: area::Logical,
    pub(in crate::text) line_count: usize,
}

#[derive(Debug, Clone, PartialEq)]
pub struct TextFieldLayout {
    pub(in crate::text) selection_spans: Vec<SelectionSpan>,
    pub(in crate::text) preedit_underline_spans: Vec<SelectionSpan>,
    pub(in crate::text) preedit_selection_spans: Vec<SelectionSpan>,
    pub(in crate::text) caret: Option<Caret>,
    pub(in crate::text) scroll_x: f32,
    pub(in crate::text) scroll_y: f32,
    pub(in crate::text) content_area: area::Logical,
}

pub struct TextAreaPaintLayout {
    pub(in crate::text) layout: TextFieldLayout,
    pub(in crate::text) interaction_surfaces: Vec<TextAreaSurface>,
    pub(in crate::text) render_surfaces: Vec<TextAreaSurface>,
}

#[derive(Clone)]
pub struct TextAreaSurface {
    pub(in crate::text) x: f32,
    pub(in crate::text) y: f32,
    pub(in crate::text) width: f32,
    pub(in crate::text) height: f32,
    pub(in crate::text) source_line: usize,
    pub(in crate::text) source_line_id: Option<LineId>,
    pub(in crate::text) source_start: usize,
    pub(in crate::text) source_text_len: usize,
    pub(in crate::text) buffer: Rc<RefCell<glyphon::Buffer>>,
    pub(in crate::text) default_color: Color,
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

    pub fn default_color(&self) -> Color {
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
