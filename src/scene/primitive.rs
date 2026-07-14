use std::fmt;

use crate::icon as icons;
use crate::text as text_model;

use super::super::geometry;
use super::Color;
use super::material::Material;

#[derive(Debug, Clone, PartialEq)]
pub enum Primitive {
    Quad(Quad),
    Rule(Rule),
    Text(Text),
    TextViewport(TextViewport),
    Icon(Icon),
    Shadow(Shadow),
    Pane(Pane),
    Clip(Clip),
    PopClip,
    Outline(Outline),
    Group(Group),
}

#[derive(Debug, Clone, PartialEq)]
pub struct Group {
    primitives: Vec<Primitive>,
    opacity: f32,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Quad {
    rect: geometry::Rect,
    style: Style,
    rounding: Rounding,
    rasterization: Rasterization,
    transform: Transform,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Rule {
    axis: Axis,
    rect: geometry::Rect,
    color: Color,
    thickness_px: u32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Axis {
    Horizontal,
    Vertical,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Transform {
    origin_x: f32,
    origin_y: f32,
    translate_x: f32,
    translate_y: f32,
    scale_x: f32,
    scale_y: f32,
    motion: Motion,
    scale_motion: Option<ScaleMotion>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Motion {
    Moving,
    Resting,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ScaleMotion {
    from_x: f32,
    from_y: f32,
    to_x: f32,
    to_y: f32,
    progress: f32,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Text {
    rect: geometry::Rect,
    value: String,
    color: Color,
    style: TextStyle,
    wrap: TextWrap,
    align: TextAlign,
    overflow: text_model::Overflow,
}

#[derive(Debug, Clone, PartialEq)]
pub struct TextViewport {
    rect: geometry::Rect,
    surfaces: Vec<TextSurface>,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Icon {
    rect: geometry::Rect,
    icon: icons::Icon,
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

#[derive(Debug, Clone, PartialEq)]
pub struct Pane {
    rect: geometry::Rect,
    rounding: Rounding,
    material: Material,
    region_id: Option<crate::composition::NodeId>,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Outline {
    rect: geometry::Rect,
    color: Color,
    width: f32,
    offset: f32,
    rounding: Rounding,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Clip {
    rect: geometry::Rect,
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
pub struct TextStyle {
    size: f32,
    weight: text_model::document::Weight,
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
    LinearGradient { from: Color, to: Color },
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
    edge_mode: EdgeMode,
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
    buffer: text_model::layout::ShapedBuffer,
    default_color: TextColor,
}

#[derive(Clone, Copy, PartialEq)]
pub(crate) struct TextColor {
    r: f32,
    g: f32,
    b: f32,
    a: f32,
}

impl Primitive {
    pub(crate) fn translated(&self, dx: i32, dy: i32) -> Self {
        match self {
            Self::Quad(quad) => Self::Quad(quad.translated(dx, dy)),
            Self::Rule(rule) => Self::Rule(rule.translated(dx, dy)),
            Self::Text(text) => Self::Text(text.translated(dx, dy)),
            Self::TextViewport(viewport) => Self::TextViewport(viewport.translated(dx, dy)),
            Self::Icon(icon) => Self::Icon(icon.translated(dx, dy)),
            Self::Shadow(shadow) => Self::Shadow(shadow.translated(dx, dy)),
            Self::Pane(pane) => Self::Pane(pane.translated(dx, dy)),
            Self::Clip(clip) => Self::Clip(clip.translated(dx, dy)),
            Self::PopClip => Self::PopClip,
            Self::Outline(outline) => Self::Outline(outline.translated(dx, dy)),
            Self::Group(group) => Self::Group(group.translated(dx, dy)),
        }
    }

    pub(in crate::scene) fn without_backdrop_sampling(&self) -> Self {
        match self {
            Self::Pane(pane) => Self::Pane(pane.without_backdrop_sampling()),
            Self::Group(group) => Self::Group(group.without_backdrop_sampling()),
            _ => self.clone(),
        }
    }
}

fn translate_rect(rect: geometry::Rect, dx: i32, dy: i32) -> geometry::Rect {
    geometry::Rect::new(
        rect.x().saturating_add(dx),
        rect.y().saturating_add(dy),
        rect.width(),
        rect.height(),
    )
}

impl Group {
    pub(crate) fn new(primitives: Vec<Primitive>, opacity: f32) -> Option<Self> {
        let opacity = opacity.clamp(0.0, 1.0);
        if opacity <= 0.0 || primitives.is_empty() {
            return None;
        }

        Some(Self {
            primitives,
            opacity,
        })
    }

    pub(crate) fn primitives(&self) -> &[Primitive] {
        &self.primitives
    }

    pub(crate) fn opacity(&self) -> f32 {
        self.opacity
    }

    pub(crate) fn translated(&self, dx: i32, dy: i32) -> Self {
        Self {
            primitives: self
                .primitives
                .iter()
                .map(|primitive| primitive.translated(dx, dy))
                .collect(),
            opacity: self.opacity,
        }
    }

    fn without_backdrop_sampling(&self) -> Self {
        Self {
            primitives: self
                .primitives
                .iter()
                .map(Primitive::without_backdrop_sampling)
                .collect(),
            opacity: self.opacity,
        }
    }
}

impl Quad {
    pub(in crate::scene) fn new(rect: geometry::Rect, fill: Color) -> Self {
        Self {
            rect,
            style: Style::filled(fill),
            rounding: Rounding::none(),
            rasterization: Rasterization::default(),
            transform: Transform::identity(),
        }
    }

    pub(in crate::scene) fn styled(rect: geometry::Rect, style: Style) -> Self {
        Self {
            rect,
            style,
            rounding: Rounding::none(),
            rasterization: Rasterization::default(),
            transform: Transform::identity(),
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

    pub fn with_transform(mut self, transform: Transform) -> Self {
        self.transform = transform;
        self
    }

    pub fn rect(&self) -> geometry::Rect {
        self.rect
    }

    pub fn fill(&self) -> Color {
        match self.style.fill {
            Some(Brush::Solid(color)) => color,
            Some(Brush::LinearGradient { .. }) => Color::rgba(0, 0, 0, 0),
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

    pub fn transform(&self) -> Transform {
        self.transform
    }

    pub(crate) fn translated(&self, dx: i32, dy: i32) -> Self {
        Self {
            rect: translate_rect(self.rect, dx, dy),
            transform: self.transform.translated(dx, dy),
            ..*self
        }
    }
}

impl Rule {
    pub(in crate::scene) fn horizontal(
        rect: geometry::Rect,
        color: Color,
        thickness_px: u32,
    ) -> Self {
        Self {
            axis: Axis::Horizontal,
            rect,
            color,
            thickness_px: thickness_px.max(1),
        }
    }

    pub(in crate::scene) fn vertical(
        rect: geometry::Rect,
        color: Color,
        thickness_px: u32,
    ) -> Self {
        Self {
            axis: Axis::Vertical,
            rect,
            color,
            thickness_px: thickness_px.max(1),
        }
    }

    pub fn axis(&self) -> Axis {
        self.axis
    }

    pub fn rect(&self) -> geometry::Rect {
        self.rect
    }

    pub fn color(&self) -> Color {
        self.color
    }

    pub fn thickness_px(&self) -> u32 {
        self.thickness_px
    }

    pub(crate) fn translated(&self, dx: i32, dy: i32) -> Self {
        Self {
            rect: translate_rect(self.rect, dx, dy),
            ..*self
        }
    }
}

impl Transform {
    pub const fn identity() -> Self {
        Self {
            origin_x: 0.0,
            origin_y: 0.0,
            translate_x: 0.0,
            translate_y: 0.0,
            scale_x: 1.0,
            scale_y: 1.0,
            motion: Motion::Resting,
            scale_motion: None,
        }
    }

    pub fn translate(x: f32, y: f32) -> Self {
        Self {
            translate_x: sanitized_transform_value(x, 0.0),
            translate_y: sanitized_transform_value(y, 0.0),
            ..Self::identity()
        }
    }

    pub fn scale_about(origin_x: f32, origin_y: f32, scale_x: f32, scale_y: f32) -> Self {
        Self {
            origin_x: sanitized_transform_value(origin_x, 0.0),
            origin_y: sanitized_transform_value(origin_y, 0.0),
            scale_x: sanitized_transform_value(scale_x, 1.0),
            scale_y: sanitized_transform_value(scale_y, 1.0),
            ..Self::identity()
        }
    }

    pub fn scale_y_about_rect_center(rect: geometry::Rect, scale_y: f32) -> Self {
        let origin_x = rect.x() as f32 + rect.width() as f32 / 2.0;
        let origin_y = rect.y() as f32 + rect.height() as f32 / 2.0;

        Self::scale_about(origin_x, origin_y, 1.0, scale_y)
    }

    pub fn with_motion(mut self, motion: Motion) -> Self {
        self.motion = motion;
        self
    }

    pub fn with_scale_motion(
        mut self,
        from_x: f32,
        from_y: f32,
        to_x: f32,
        to_y: f32,
        progress: f32,
    ) -> Self {
        self.scale_motion = Some(ScaleMotion::new(from_x, from_y, to_x, to_y, progress));
        self
    }

    pub const fn origin_x(self) -> f32 {
        self.origin_x
    }

    pub const fn origin_y(self) -> f32 {
        self.origin_y
    }

    pub const fn translate_x(self) -> f32 {
        self.translate_x
    }

    pub const fn translate_y(self) -> f32 {
        self.translate_y
    }

    pub const fn scale_x(self) -> f32 {
        self.scale_x
    }

    pub const fn scale_y(self) -> f32 {
        self.scale_y
    }

    pub const fn motion(self) -> Motion {
        self.motion
    }

    pub const fn scale_motion(self) -> Option<ScaleMotion> {
        self.scale_motion
    }

    pub(crate) fn translated(mut self, dx: i32, dy: i32) -> Self {
        self.origin_x += dx as f32;
        self.origin_y += dy as f32;
        self
    }
}

impl Default for Transform {
    fn default() -> Self {
        Self::identity()
    }
}

fn sanitized_transform_value(value: f32, fallback: f32) -> f32 {
    if value.is_finite() { value } else { fallback }
}

impl ScaleMotion {
    fn new(from_x: f32, from_y: f32, to_x: f32, to_y: f32, progress: f32) -> Self {
        Self {
            from_x: sanitize_scale(from_x),
            from_y: sanitize_scale(from_y),
            to_x: sanitize_scale(to_x),
            to_y: sanitize_scale(to_y),
            progress: sanitize_progress(progress),
        }
    }

    pub const fn from_x(self) -> f32 {
        self.from_x
    }

    pub const fn from_y(self) -> f32 {
        self.from_y
    }

    pub const fn to_x(self) -> f32 {
        self.to_x
    }

    pub const fn to_y(self) -> f32 {
        self.to_y
    }

    pub const fn progress(self) -> f32 {
        self.progress
    }
}

fn sanitize_scale(scale: f32) -> f32 {
    if scale.is_finite() {
        scale.max(0.0)
    } else {
        1.0
    }
}

fn sanitize_progress(progress: f32) -> f32 {
    if progress.is_finite() {
        progress.clamp(0.0, 1.0)
    } else {
        1.0
    }
}

impl Text {
    pub(in crate::scene) fn new(
        rect: geometry::Rect,
        value: impl Into<String>,
        color: Color,
        wrap: TextWrap,
    ) -> Self {
        Self {
            rect,
            value: value.into(),
            color,
            style: TextStyle::default(),
            wrap,
            align: TextAlign::Start,
            overflow: text_model::Overflow::Clip,
        }
    }

    pub(in crate::scene) fn with_style(mut self, style: TextStyle) -> Self {
        self.style = style;
        self
    }

    pub(in crate::scene) fn with_align(mut self, align: TextAlign) -> Self {
        self.align = align;
        self
    }

    pub(in crate::scene) fn with_overflow(mut self, overflow: text_model::Overflow) -> Self {
        self.overflow = overflow;
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

    pub fn style(&self) -> TextStyle {
        self.style
    }

    pub fn wrap(&self) -> TextWrap {
        self.wrap
    }

    pub fn align(&self) -> TextAlign {
        self.align
    }

    pub fn overflow(&self) -> text_model::Overflow {
        self.overflow
    }

    pub(crate) fn translated(&self, dx: i32, dy: i32) -> Self {
        Self {
            rect: translate_rect(self.rect, dx, dy),
            value: self.value.clone(),
            color: self.color,
            style: self.style,
            wrap: self.wrap,
            align: self.align,
            overflow: self.overflow,
        }
    }
}

impl TextStyle {
    pub(crate) const fn new(size: f32, weight: text_model::document::Weight) -> Self {
        Self { size, weight }
    }

    pub fn size(self) -> f32 {
        self.size
    }

    pub fn weight(self) -> text_model::document::Weight {
        self.weight
    }
}

impl Default for TextStyle {
    fn default() -> Self {
        Self::new(16.0, text_model::document::Weight::Normal)
    }
}

impl TextViewport {
    pub(in crate::scene) fn new(rect: geometry::Rect, surfaces: Vec<TextSurface>) -> Self {
        Self { rect, surfaces }
    }

    pub fn rect(&self) -> geometry::Rect {
        self.rect
    }

    pub fn surfaces(&self) -> &[TextSurface] {
        &self.surfaces
    }

    pub(crate) fn translated(&self, dx: i32, dy: i32) -> Self {
        Self {
            rect: translate_rect(self.rect, dx, dy),
            surfaces: self
                .surfaces
                .iter()
                .map(|surface| surface.translated(dx, dy))
                .collect(),
        }
    }
}

impl TextSurface {
    pub(in crate::scene) fn new(
        rect: geometry::Rect,
        buffer: text_model::layout::ShapedBuffer,
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

    pub(crate) fn buffer(&self) -> text_model::layout::ShapedBuffer {
        self.buffer.clone()
    }

    pub(crate) fn default_color(&self) -> TextColor {
        self.default_color
    }

    pub(crate) fn translated(&self, dx: i32, dy: i32) -> Self {
        Self {
            rect: translate_rect(self.rect, dx, dy),
            buffer: self.buffer.clone(),
            default_color: self.default_color,
        }
    }
}

impl Clip {
    pub(crate) fn new(rect: geometry::Rect) -> Self {
        Self {
            rect,
            rounding: Rounding::none(),
        }
    }

    pub(crate) fn with_rounding(mut self, rounding: Rounding) -> Self {
        self.rounding = rounding;
        self
    }

    pub fn rect(self) -> geometry::Rect {
        self.rect
    }

    pub fn rounding(self) -> Rounding {
        self.rounding
    }

    pub(crate) fn translated(&self, dx: i32, dy: i32) -> Self {
        Self {
            rect: translate_rect(self.rect, dx, dy),
            rounding: self.rounding,
        }
    }
}

impl Icon {
    pub(in crate::scene) fn new(
        rect: geometry::Rect,
        icon: icons::Icon,
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

    pub fn icon(&self) -> icons::Icon {
        self.icon
    }

    pub fn color(&self) -> Color {
        self.color
    }

    pub fn size(&self) -> f32 {
        self.size
    }

    pub(crate) fn translated(&self, dx: i32, dy: i32) -> Self {
        Self {
            rect: translate_rect(self.rect, dx, dy),
            icon: self.icon,
            color: self.color,
            size: self.size,
        }
    }
}

impl Shadow {
    pub(in crate::scene) fn new(
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

    pub(in crate::scene) fn with_rounding(mut self, rounding: Rounding) -> Self {
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

    pub(crate) fn translated(&self, dx: i32, dy: i32) -> Self {
        Self {
            rect: translate_rect(self.rect, dx, dy),
            color: self.color,
            blur: self.blur,
            spread: self.spread,
            offset: self.offset,
            rounding: self.rounding,
        }
    }
}

impl Pane {
    pub(in crate::scene) fn new(rect: geometry::Rect, material: Material) -> Self {
        Self {
            rect,
            rounding: Rounding::none(),
            material,
            region_id: None,
        }
    }

    pub(in crate::scene) fn with_rounding(mut self, rounding: Rounding) -> Self {
        self.rounding = rounding;
        self
    }

    pub(in crate::scene) fn with_region_id(mut self, id: crate::composition::NodeId) -> Self {
        self.region_id = Some(id);
        self
    }

    pub(in crate::scene) fn with_material(mut self, material: Material) -> Self {
        self.material = material;
        self
    }

    pub(crate) fn without_backdrop_sampling(&self) -> Self {
        Self {
            rect: self.rect,
            rounding: self.rounding,
            material: self.material.without_backdrop_sampling(),
            region_id: None,
        }
    }

    pub fn rect(&self) -> geometry::Rect {
        self.rect
    }

    pub fn rounding(&self) -> Rounding {
        self.rounding
    }

    pub fn material(&self) -> &Material {
        &self.material
    }

    pub(crate) fn region_id(&self) -> Option<crate::composition::NodeId> {
        self.region_id
    }

    pub(crate) fn translated(&self, dx: i32, dy: i32) -> Self {
        Self {
            rect: translate_rect(self.rect, dx, dy),
            rounding: self.rounding,
            material: self.material.clone(),
            region_id: self.region_id,
        }
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

    pub(crate) fn channels(self) -> (f32, f32, f32, f32) {
        (self.r, self.g, self.b, self.a)
    }
}

impl Outline {
    pub(in crate::scene) fn new(rect: geometry::Rect, color: Color) -> Self {
        Self {
            rect,
            color,
            width: 1.0,
            offset: 0.0,
            rounding: Rounding::none(),
        }
    }

    pub(in crate::scene) fn with_width(mut self, width: f32) -> Self {
        self.width = width.max(0.0);
        self
    }

    pub(in crate::scene) fn with_offset(mut self, offset: f32) -> Self {
        self.offset = offset.max(0.0);
        self
    }

    pub(in crate::scene) fn with_rounding(mut self, rounding: Rounding) -> Self {
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

    pub(crate) fn translated(&self, dx: i32, dy: i32) -> Self {
        Self {
            rect: translate_rect(self.rect, dx, dy),
            color: self.color,
            width: self.width,
            offset: self.offset,
            rounding: self.rounding,
        }
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

    pub const fn filled_with(brush: Brush) -> Self {
        Self {
            fill: Some(brush),
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

    pub const fn linear_gradient(from: Color, to: Color) -> Self {
        Self::LinearGradient { from, to }
    }

    pub fn is_visible(self) -> bool {
        match self {
            Self::Solid(color) => color.channels().3 > 0,
            Self::LinearGradient { from, to } => from.channels().3 > 0 || to.channels().3 > 0,
        }
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
    pub const fn new(edge_mode: EdgeMode) -> Self {
        Self { edge_mode }
    }

    pub const fn edge_mode(self) -> EdgeMode {
        self.edge_mode
    }
}

impl Default for Rasterization {
    fn default() -> Self {
        Self {
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
