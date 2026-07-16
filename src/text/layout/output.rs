use crate::geometry::{Rect, area};
use std::cell::{Ref, RefCell};
use std::fmt;
use std::rc::Rc;

use super::super::Color;
use super::super::buffer::LineId;
use super::caret::{Caret, CaretLayout};
use super::highlight::SelectionSpan;

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

pub struct TextFieldPaintLayout {
    pub(in crate::text) layout: TextFieldLayout,
    pub(in crate::text) surface: Option<TextAreaSurface>,
}

#[derive(Clone)]
pub struct TextAreaSurface {
    pub(in crate::text) x: f32,
    pub(in crate::text) y: f32,
    pub(in crate::text) text_x: f32,
    pub(in crate::text) width: f32,
    pub(in crate::text) height: f32,
    pub(in crate::text) source_line: usize,
    pub(in crate::text) source_line_id: Option<LineId>,
    pub(in crate::text) source_start: usize,
    pub(in crate::text) source_text_len: usize,
    pub(in crate::text) buffer: Rc<RefCell<glyphon::Buffer>>,
    pub(in crate::text) default_color: Color,
}

#[derive(Clone)]
pub(crate) struct ShapedBuffer(Rc<RefCell<glyphon::Buffer>>);

impl fmt::Debug for TextAreaSurface {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("TextAreaSurface")
            .field("x", &self.x)
            .field("y", &self.y)
            .field("text_x", &self.text_x)
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

impl TextFieldPaintLayout {
    pub fn layout(&self) -> &TextFieldLayout {
        &self.layout
    }

    pub fn surface(&self) -> Option<&TextAreaSurface> {
        self.surface.as_ref()
    }

    pub fn into_parts(self) -> (TextFieldLayout, Option<TextAreaSurface>) {
        (self.layout, self.surface)
    }
}

impl TextAreaSurface {
    pub(crate) fn pixel_rect(&self, viewport: Rect) -> Rect {
        Rect::new(
            viewport.x().saturating_add(self.x.round() as i32),
            viewport.y().saturating_add(self.y.round() as i32),
            self.width.ceil().max(0.0) as i32,
            self.height.ceil().max(0.0) as i32,
        )
    }

    pub(crate) fn text_origin(&self, viewport: Rect) -> crate::geometry::Point {
        crate::geometry::Point::new(
            viewport.x().saturating_add(self.text_x.round() as i32),
            viewport.y().saturating_add(self.y.round() as i32),
        )
    }

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

    pub(in crate::text) fn buffer(&self) -> Rc<RefCell<glyphon::Buffer>> {
        self.buffer.clone()
    }

    pub(crate) fn shaped_buffer(&self) -> ShapedBuffer {
        ShapedBuffer(self.buffer.clone())
    }

    pub fn default_color(&self) -> Color {
        self.default_color
    }
}

impl ShapedBuffer {
    pub(crate) fn borrow(&self) -> Ref<'_, glyphon::Buffer> {
        self.0.borrow()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn text_surface_has_one_integral_rectangle_for_paint_and_residency() {
        let surface = TextAreaSurface {
            x: 0.51,
            y: -3.49,
            text_x: 0.51,
            width: 20.01,
            height: 10.01,
            source_line: 0,
            source_line_id: None,
            source_start: 0,
            source_text_len: 0,
            buffer: Rc::new(RefCell::new(glyphon::Buffer::new_empty(
                glyphon::Metrics::new(14.0, 17.5),
            ))),
            default_color: Color::BLACK,
        };

        assert_eq!(
            surface.pixel_rect(Rect::new(100, 50, 80, 40)),
            Rect::new(101, 47, 21, 11)
        );
    }

    #[test]
    fn text_surface_keeps_prepared_runway_separate_from_buffer_origin() {
        let surface = TextAreaSurface {
            x: -193.0,
            y: 7.0,
            text_x: -5_157_889.0,
            width: 1_432.0,
            height: 24.0,
            source_line: 0,
            source_line_id: None,
            source_start: 0,
            source_text_len: 0,
            buffer: Rc::new(RefCell::new(glyphon::Buffer::new_empty(
                glyphon::Metrics::new(14.0, 18.0),
            ))),
            default_color: Color::BLACK,
        };
        let viewport = Rect::new(40, 60, 920, 640);

        assert_eq!(surface.pixel_rect(viewport), Rect::new(-153, 67, 1432, 24));
        assert_eq!(
            surface.text_origin(viewport),
            crate::geometry::Point::new(-5_157_849, 67)
        );
    }
}
