use std::{cell::RefCell, fmt, rc::Rc};

use crate::icon as framework_icon;

use super::super::geometry;
use super::Color;

#[derive(Debug, Clone, PartialEq)]
pub enum Primitive {
    Quad(Quad),
    Text(Text),
    TextViewport(TextViewport),
    Icon(Icon),
    Shadow(Shadow),
    Outline(Outline),
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Quad {
    rect: geometry::Rect,
    style: Style,
    rounding: Rounding,
    rasterization: Rasterization,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Text {
    rect: geometry::Rect,
    value: String,
    color: Color,
    wrap: TextWrap,
    align: TextAlign,
}

#[derive(Debug, Clone, PartialEq)]
pub struct TextViewport {
    rect: geometry::Rect,
    surfaces: Vec<TextSurface>,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Icon {
    rect: geometry::Rect,
    icon: framework_icon::Icon,
    color: Color,
    size: f32,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Shadow {
    rect: geometry::Rect,
    color: Color,
    blur: f32,
    spread: f32,
    offset: Offset,
    rounding: Rounding,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Outline {
    rect: geometry::Rect,
    color: Color,
    width: f32,
    offset: f32,
    rounding: Rounding,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TextWrap {
    None,
    WordOrGlyph,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TextAlign {
    Start,
    Center,
    End,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Style {
    fill: Option<Brush>,
    stroke: Option<Stroke>,
    tint: Option<Brush>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Brush {
    Solid(Color),
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Stroke {
    brush: Brush,
    width: f32,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Rounding {
    top_left: Radius,
    top_right: Radius,
    bottom_right: Radius,
    bottom_left: Radius,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Radius {
    Relative(f32),
    Fixed(f32),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Rasterization {
    snapping: Snapping,
    edge_mode: EdgeMode,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Snapping {
    Disabled,
    Rect,
    FixedWidth { width_px: u32 },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EdgeMode {
    Antialiased,
    Hard,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Offset {
    x: f32,
    y: f32,
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
    pub(in crate::scratch::scene) fn new(rect: geometry::Rect, fill: Color) -> Self {
        Self {
            rect,
            style: Style::filled(fill),
            rounding: Rounding::none(),
            rasterization: Rasterization::default(),
        }
    }

    pub(in crate::scratch::scene) fn styled(rect: geometry::Rect, style: Style) -> Self {
        Self {
            rect,
            style,
            rounding: Rounding::none(),
            rasterization: Rasterization::default(),
        }
    }

    pub fn with_rounding(mut self, rounding: Rounding) -> Self {
        self.rounding = rounding;
        self
    }

    pub fn with_stroke(mut self, stroke: Stroke) -> Self {
        self.style = self.style.with_stroke(stroke);
        self
    }

    pub fn with_rasterization(mut self, rasterization: Rasterization) -> Self {
        self.rasterization = rasterization;
        self
    }

    pub fn rect(&self) -> geometry::Rect {
        self.rect
    }

    pub fn fill(&self) -> Color {
        match self.style.fill {
            Some(Brush::Solid(color)) => color,
            None => Color::rgba(0, 0, 0, 0),
        }
    }

    pub fn style(&self) -> Style {
        self.style
    }

    pub fn rounding(&self) -> Rounding {
        self.rounding
    }

    pub fn rasterization(&self) -> Rasterization {
        self.rasterization
    }
}

impl Text {
    pub(in crate::scratch::scene) fn new(
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
            align: TextAlign::Start,
        }
    }

    pub(in crate::scratch::scene) fn with_align(mut self, align: TextAlign) -> Self {
        self.align = align;
        self
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

    pub fn align(&self) -> TextAlign {
        self.align
    }
}

impl TextViewport {
    pub(in crate::scratch::scene) fn new(rect: geometry::Rect, surfaces: Vec<TextSurface>) -> Self {
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
    pub(in crate::scratch::scene) fn new(
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

impl Icon {
    pub(in crate::scratch::scene) fn new(
        rect: geometry::Rect,
        icon: framework_icon::Icon,
        color: Color,
        size: f32,
    ) -> Self {
        Self {
            rect,
            icon,
            color,
            size,
        }
    }

    pub fn rect(&self) -> geometry::Rect {
        self.rect
    }

    pub fn icon(&self) -> framework_icon::Icon {
        self.icon
    }

    pub fn color(&self) -> Color {
        self.color
    }

    pub fn size(&self) -> f32 {
        self.size
    }
}

impl Shadow {
    pub(in crate::scratch::scene) fn new(
        rect: geometry::Rect,
        color: Color,
        blur: f32,
        spread: f32,
        offset: Offset,
    ) -> Self {
        Self {
            rect,
            color,
            blur,
            spread,
            offset,
            rounding: Rounding::none(),
        }
    }

    pub(in crate::scratch::scene) fn with_rounding(mut self, rounding: Rounding) -> Self {
        self.rounding = rounding;
        self
    }

    pub fn rect(&self) -> geometry::Rect {
        self.rect
    }

    pub fn color(&self) -> Color {
        self.color
    }

    pub fn blur(&self) -> f32 {
        self.blur
    }

    pub fn spread(&self) -> f32 {
        self.spread
    }

    pub fn offset(&self) -> Offset {
        self.offset
    }

    pub fn rounding(&self) -> Rounding {
        self.rounding
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
    pub(in crate::scratch::scene) fn new(rect: geometry::Rect, color: Color) -> Self {
        Self {
            rect,
            color,
            width: 1.0,
            offset: 0.0,
            rounding: Rounding::none(),
        }
    }

    pub(in crate::scratch::scene) fn with_width(mut self, width: f32) -> Self {
        self.width = width.max(0.0);
        self
    }

    pub(in crate::scratch::scene) fn with_rounding(mut self, rounding: Rounding) -> Self {
        self.rounding = rounding;
        self
    }

    pub fn rect(&self) -> geometry::Rect {
        self.rect
    }

    pub fn color(&self) -> Color {
        self.color
    }

    pub fn width(&self) -> f32 {
        self.width
    }

    pub fn offset(&self) -> f32 {
        self.offset
    }

    pub fn rounding(&self) -> Rounding {
        self.rounding
    }
}

impl Style {
    pub const fn filled(color: Color) -> Self {
        Self {
            fill: Some(Brush::Solid(color)),
            stroke: None,
            tint: None,
        }
    }

    pub const fn stroked(stroke: Stroke) -> Self {
        Self {
            fill: None,
            stroke: Some(stroke),
            tint: None,
        }
    }

    pub const fn with_stroke(mut self, stroke: Stroke) -> Self {
        self.stroke = Some(stroke);
        self
    }

    pub const fn fill(self) -> Option<Brush> {
        self.fill
    }

    pub const fn stroke(self) -> Option<Stroke> {
        self.stroke
    }

    pub const fn tint(self) -> Option<Brush> {
        self.tint
    }
}

impl Stroke {
    pub const fn new(brush: Brush, width: f32) -> Self {
        Self { brush, width }
    }

    pub const fn brush(self) -> Brush {
        self.brush
    }

    pub const fn width(self) -> f32 {
        self.width
    }
}

impl Brush {
    pub const fn solid(color: Color) -> Self {
        Self::Solid(color)
    }
}

impl Rounding {
    pub const fn new(
        top_left: Radius,
        top_right: Radius,
        bottom_right: Radius,
        bottom_left: Radius,
    ) -> Self {
        Self {
            top_left,
            top_right,
            bottom_right,
            bottom_left,
        }
    }

    pub const fn none() -> Self {
        Self::fixed(0.0)
    }

    pub const fn fixed(value: f32) -> Self {
        Self::all(Radius::Fixed(value))
    }

    pub const fn relative(value: f32) -> Self {
        Self::all(Radius::Relative(value))
    }

    pub const fn top_left(self) -> Radius {
        self.top_left
    }

    pub const fn top_right(self) -> Radius {
        self.top_right
    }

    pub const fn bottom_right(self) -> Radius {
        self.bottom_right
    }

    pub const fn bottom_left(self) -> Radius {
        self.bottom_left
    }

    const fn all(radius: Radius) -> Self {
        Self::new(radius, radius, radius, radius)
    }
}

impl Default for Rounding {
    fn default() -> Self {
        Self::none()
    }
}

impl Rasterization {
    pub const fn new(snapping: Snapping, edge_mode: EdgeMode) -> Self {
        Self {
            snapping,
            edge_mode,
        }
    }

    pub const fn snapping(self) -> Snapping {
        self.snapping
    }

    pub const fn edge_mode(self) -> EdgeMode {
        self.edge_mode
    }
}

impl Default for Rasterization {
    fn default() -> Self {
        Self {
            snapping: Snapping::Disabled,
            edge_mode: EdgeMode::Antialiased,
        }
    }
}

impl Offset {
    pub const fn new(x: f32, y: f32) -> Self {
        Self { x, y }
    }

    pub const fn x(self) -> f32 {
        self.x
    }

    pub const fn y(self) -> f32 {
        self.y
    }
}
