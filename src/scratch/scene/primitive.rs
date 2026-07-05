use std::{cell::RefCell, fmt, rc::Rc};

use super::super::geometry;
use super::Color;

#[derive(Debug, Clone, PartialEq)]
pub enum Primitive {
    Quad(Quad),
    Text(Text),
    TextViewport(TextViewport),
    Outline(Outline),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Quad {
    rect: geometry::Rect,
    fill: Color,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Text {
    rect: geometry::Rect,
    value: String,
    color: Color,
    wrap: TextWrap,
}

#[derive(Debug, Clone, PartialEq)]
pub struct TextViewport {
    rect: geometry::Rect,
    surfaces: Vec<TextSurface>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Outline {
    rect: geometry::Rect,
    color: Color,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TextWrap {
    None,
    WordOrGlyph,
}

#[derive(Clone)]
pub struct TextSurface {
    rect: geometry::Rect,
    buffer: Rc<RefCell<glyphon::Buffer>>,
    default_color: TextColor,
}

#[derive(Clone, Copy, PartialEq)]
pub(in crate::scratch) struct TextColor {
    r: f32,
    g: f32,
    b: f32,
    a: f32,
}

impl Quad {
    pub(super) fn new(rect: geometry::Rect, fill: Color) -> Self {
        Self { rect, fill }
    }

    pub fn rect(&self) -> geometry::Rect {
        self.rect
    }

    pub fn fill(&self) -> Color {
        self.fill
    }
}

impl Text {
    pub(super) fn new(
        rect: geometry::Rect,
        value: impl Into<String>,
        color: Color,
        wrap: TextWrap,
    ) -> Self {
        Self {
            rect,
            value: value.into(),
            color,
            wrap,
        }
    }

    pub fn rect(&self) -> geometry::Rect {
        self.rect
    }

    pub fn value(&self) -> &str {
        &self.value
    }

    pub fn color(&self) -> Color {
        self.color
    }

    pub fn wrap(&self) -> TextWrap {
        self.wrap
    }
}

impl TextViewport {
    pub(super) fn new(rect: geometry::Rect, surfaces: Vec<TextSurface>) -> Self {
        Self { rect, surfaces }
    }

    pub fn rect(&self) -> geometry::Rect {
        self.rect
    }

    pub fn surfaces(&self) -> &[TextSurface] {
        &self.surfaces
    }
}

impl TextSurface {
    pub(super) fn new(
        rect: geometry::Rect,
        buffer: Rc<RefCell<glyphon::Buffer>>,
        default_color: TextColor,
    ) -> Self {
        Self {
            rect,
            buffer,
            default_color,
        }
    }

    pub fn rect(&self) -> geometry::Rect {
        self.rect
    }

    pub(in crate::scratch) fn buffer(&self) -> Rc<RefCell<glyphon::Buffer>> {
        self.buffer.clone()
    }

    pub(in crate::scratch) fn default_color(&self) -> TextColor {
        self.default_color
    }
}

impl fmt::Debug for TextSurface {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("TextSurface")
            .field("rect", &self.rect)
            .field("default_color", &self.default_color)
            .finish_non_exhaustive()
    }
}

impl PartialEq for TextSurface {
    fn eq(&self, other: &Self) -> bool {
        self.rect == other.rect && self.default_color == other.default_color
    }
}

impl fmt::Debug for TextColor {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("TextColor")
            .field("r", &self.r)
            .field("g", &self.g)
            .field("b", &self.b)
            .field("a", &self.a)
            .finish()
    }
}

impl TextColor {
    pub(super) const fn rgba(r: f32, g: f32, b: f32, a: f32) -> Self {
        Self { r, g, b, a }
    }

    pub(in crate::scratch) fn channels(self) -> (f32, f32, f32, f32) {
        (self.r, self.g, self.b, self.a)
    }
}

impl Outline {
    pub(super) fn new(rect: geometry::Rect, color: Color) -> Self {
        Self { rect, color }
    }

    pub fn rect(&self) -> geometry::Rect {
        self.rect
    }

    pub fn color(&self) -> Color {
        self.color
    }
}
