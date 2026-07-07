use crate::icon;
use crate::paint_geometry::{Rect, area, point};
use crate::text;
use std::cell::RefCell;
use std::fmt;
use std::rc::Rc;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct LayerId(pub u64);

#[derive(Debug, Clone, PartialEq)]
pub struct Scene {
    clear_color: Option<Color>,
    items: Vec<Item>,
}

impl Scene {
    pub fn new() -> Self {
        Self {
            clear_color: None,
            items: Vec::new(),
        }
    }

    pub fn clear(&mut self, color: Color) {
        self.clear_color = Some(color);
    }

    pub fn clear_color(&self) -> Option<Color> {
        self.clear_color
    }

    pub fn push_quad(&mut self, quad: Quad) {
        self.items.push(Item::Quad(quad));
    }

    pub fn push_text(&mut self, text: Text) {
        if !text.document.is_empty() {
            self.items.push(Item::Text(text));
        }
    }

    pub fn push_text_surface(&mut self, text: TextSurface) {
        self.items.push(Item::TextSurface(text));
    }

    pub fn push_text_viewport(&mut self, text: TextViewport) {
        if !text.surfaces.is_empty() {
            self.items.push(Item::TextViewport(text));
        }
    }

    pub fn push_icon(&mut self, icon: Icon) {
        self.items.push(Item::Icon(icon));
    }

    pub fn push_shadow(&mut self, shadow: Shadow) {
        self.items.push(Item::Shadow(shadow));
    }

    pub fn push_tint(&mut self, tint: Tint) {
        self.items.push(Item::Tint(tint));
    }

    pub fn push_outline(&mut self, outline: Outline) {
        self.items.push(Item::Outline(outline));
    }

    pub fn push_filter(&mut self, filter: Filter) {
        if filter.rect.area.width() > 0.0
            && filter.rect.area.height() > 0.0
            && !filter.ops.is_empty()
        {
            self.items.push(Item::Filter(filter));
        }
    }

    pub fn push_layer(&mut self, layer: Layer) {
        self.items.push(Item::Layer(layer));
    }

    pub fn push_clip(&mut self, clip: Clip) {
        self.items.push(Item::Clip(clip));
    }

    pub fn pop_clip(&mut self) {
        self.items.push(Item::PopClip);
    }

    pub fn items(&self) -> &[Item] {
        &self.items
    }

    pub fn len(&self) -> usize {
        self.items.len()
    }

    pub fn is_empty(&self) -> bool {
        self.clear_color.is_none() && self.items.is_empty()
    }

    pub(crate) fn replace_items(
        &mut self,
        range: std::ops::Range<usize>,
        items: impl IntoIterator<Item = Item>,
    ) {
        self.items.splice(range, items);
    }

    pub(crate) fn translate_items(
        &mut self,
        range: std::ops::Range<usize>,
        delta: point::Logical,
    ) -> usize {
        let mut translated = 0;
        for item in self.items[range].iter_mut() {
            if item.translate(delta) {
                translated += 1;
            }
        }
        translated
    }

    pub(crate) fn translated(mut self, delta: point::Logical) -> Self {
        self.translate_items(0..self.items.len(), delta);
        self
    }
}

impl Default for Scene {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum Item {
    Quad(Quad),
    Text(Text),
    TextSurface(TextSurface),
    TextViewport(TextViewport),
    Icon(Icon),
    Shadow(Shadow),
    Tint(Tint),
    Outline(Outline),
    Filter(Filter),
    Layer(Layer),
    Clip(Clip),
    PopClip,
}

impl Item {
    pub(crate) fn translate(&mut self, delta: point::Logical) -> bool {
        match self {
            Self::Quad(quad) => {
                quad.rect = translate_rect(quad.rect, delta);
                quad.transform = quad.transform.translated(delta);
                true
            }
            Self::Text(text) => {
                text.rect = translate_rect(text.rect, delta);
                true
            }
            Self::TextSurface(text) => {
                text.rect = translate_rect(text.rect, delta);
                true
            }
            Self::TextViewport(text) => {
                text.rect = translate_rect(text.rect, delta);
                for surface in &mut text.surfaces {
                    surface.rect = translate_rect(surface.rect, delta);
                }
                true
            }
            Self::Icon(icon) => {
                icon.rect = translate_rect(icon.rect, delta);
                true
            }
            Self::Shadow(shadow) => {
                shadow.rect = translate_rect(shadow.rect, delta);
                true
            }
            Self::Tint(tint) => {
                tint.rect = translate_rect(tint.rect, delta);
                true
            }
            Self::Outline(outline) => {
                outline.rect = translate_rect(outline.rect, delta);
                true
            }
            Self::Filter(filter) => {
                filter.rect = translate_rect(filter.rect, delta);
                true
            }
            Self::Layer(layer) => {
                layer.rect = translate_rect(layer.rect, delta);
                true
            }
            Self::Clip(clip) => {
                clip.rect = translate_rect(clip.rect, delta);
                true
            }
            Self::PopClip => false,
        }
    }
}

fn translate_rect(rect: Rect, delta: point::Logical) -> Rect {
    Rect::rounded(
        point::logical(rect.origin.x() + delta.x(), rect.origin.y() + delta.y()),
        rect.area,
        rect.rounding,
    )
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Quad {
    pub rect: Rect,
    pub style: Style,
    pub rasterization: Rasterization,
    pub transform: Transform,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Transform {
    pub origin: point::Logical,
    pub translate: point::Logical,
    pub scale_x: f32,
    pub scale_y: f32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Rasterization {
    pub snapping: Snapping,
    pub edge_mode: EdgeMode,
}

impl Default for Rasterization {
    fn default() -> Self {
        Self {
            snapping: Snapping::Disabled,
            edge_mode: EdgeMode::Antialiased,
        }
    }
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

#[derive(Debug, Clone, PartialEq)]
pub struct Text {
    pub rect: Rect,
    pub document: text::document::Document,
    pub wrap: TextWrap,
    pub vertical_align: TextVerticalAlign,
}

#[derive(Clone)]
pub struct TextSurface {
    pub rect: Rect,
    pub buffer: Rc<RefCell<glyphon::Buffer>>,
    pub default_color: Color,
}

#[derive(Clone)]
pub struct TextViewport {
    pub rect: Rect,
    pub surfaces: Vec<TextSurface>,
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

impl fmt::Debug for TextViewport {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("TextViewport")
            .field("rect", &self.rect)
            .field("surfaces", &self.surfaces)
            .finish()
    }
}

impl PartialEq for TextViewport {
    fn eq(&self, other: &Self) -> bool {
        self.rect == other.rect && self.surfaces == other.surfaces
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LayerUpdate {
    pub id: LayerId,
    pub coverage: Rect,
    pub scene: Scene,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Layer {
    pub id: LayerId,
    pub rect: Rect,
    pub source: Rect,
    pub sampling: LayerSampling,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum LayerSampling {
    #[default]
    Filtered,
    PixelAligned,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TextWrap {
    WordOrGlyph,
    None,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TextVerticalAlign {
    Start,
    Center,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Icon {
    pub rect: Rect,
    pub icon: icon::Icon,
    pub color: Color,
    pub size: f32,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Shadow {
    pub rect: Rect,
    pub brush: Brush,
    pub blur: f32,
    pub spread: f32,
    pub offset: point::Logical,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Tint {
    pub rect: Rect,
    pub brush: Brush,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Outline {
    pub rect: Rect,
    pub brush: Brush,
    pub width: f32,
    pub offset: f32,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Filter {
    pub rect: Rect,
    pub ops: Vec<FilterOp>,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Clip {
    pub rect: Rect,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum FilterOp {
    Blur {
        amount: f32,
    },
    BackdropBlur(BackdropBlur),
    Liquid {
        depth: f32,
        splay: f32,
        feather: f32,
        curve: f32,
    },
    Refraction(Refraction),
    Luminosity(Luminosity),
    Noise(Noise),
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct BackdropBlur {
    pub sigma: f32,
    pub edge_mode: BackdropEdgeMode,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BackdropEdgeMode {
    Mirror,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct LiquidFilter {
    pub depth: f32,
    pub splay: f32,
    pub feather: f32,
    pub curve: f32,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Refraction {
    pub displacement: f32,
    pub splay: f32,
    pub feather: f32,
    pub curve: f32,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Luminosity {
    pub color: Color,
    pub opacity: f32,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Noise {
    pub opacity: f32,
}

impl Filter {
    pub fn blur(rect: Rect, amount: f32) -> Self {
        Self::stack(rect, [FilterOp::blur(amount)])
    }

    pub fn liquid(rect: Rect, params: LiquidFilter) -> Self {
        Self::stack(rect, [FilterOp::liquid(params)])
    }

    pub fn stack(rect: Rect, ops: impl IntoIterator<Item = FilterOp>) -> Self {
        Self {
            rect,
            ops: ops.into_iter().map(FilterOp::clamped).collect(),
        }
    }
}

impl Transform {
    pub fn identity() -> Self {
        Self {
            origin: point::logical(0.0, 0.0),
            translate: point::logical(0.0, 0.0),
            scale_x: 1.0,
            scale_y: 1.0,
        }
    }

    pub fn translate(delta: point::Logical) -> Self {
        Self {
            translate: delta,
            ..Self::identity()
        }
    }

    pub fn scale_about(origin: point::Logical, scale_x: f32, scale_y: f32) -> Self {
        Self {
            origin,
            scale_x: sanitized_scale(scale_x),
            scale_y: sanitized_scale(scale_y),
            ..Self::identity()
        }
    }

    pub fn scale_y_about(origin: point::Logical, scale_y: f32) -> Self {
        Self::scale_about(origin, 1.0, scale_y)
    }

    pub fn is_identity(self) -> bool {
        self.translate.x() == 0.0
            && self.translate.y() == 0.0
            && self.scale_x == 1.0
            && self.scale_y == 1.0
    }

    pub fn transformed_rect(self, rect: Rect) -> Rect {
        if self.is_identity() {
            return rect;
        }

        let left = rect.origin.x();
        let top = rect.origin.y();
        let right = left + rect.area.width();
        let bottom = top + rect.area.height();
        let x0 = self.transform_x(left);
        let x1 = self.transform_x(right);
        let y0 = self.transform_y(top);
        let y1 = self.transform_y(bottom);
        let left = x0.min(x1);
        let top = y0.min(y1);
        let right = x0.max(x1);
        let bottom = y0.max(y1);

        Rect::rounded(
            point::logical(left, top),
            area::logical((right - left).max(0.0), (bottom - top).max(0.0)),
            rect.rounding,
        )
    }

    pub fn translated(self, delta: point::Logical) -> Self {
        Self {
            origin: point::logical(self.origin.x() + delta.x(), self.origin.y() + delta.y()),
            ..self
        }
    }

    fn transform_x(self, x: f32) -> f32 {
        ((x - self.origin.x()) * self.scale_x) + self.origin.x() + self.translate.x()
    }

    fn transform_y(self, y: f32) -> f32 {
        ((y - self.origin.y()) * self.scale_y) + self.origin.y() + self.translate.y()
    }
}

impl Default for Transform {
    fn default() -> Self {
        Self::identity()
    }
}

fn sanitized_scale(scale: f32) -> f32 {
    if scale.is_finite() { scale } else { 1.0 }
}

impl FilterOp {
    pub fn blur(amount: f32) -> Self {
        Self::Blur {
            amount: amount.clamp(0.0, 1.0),
        }
    }

    pub fn backdrop_blur(params: BackdropBlur) -> Self {
        Self::BackdropBlur(params.clamped())
    }

    pub fn liquid(params: LiquidFilter) -> Self {
        Self::Liquid {
            depth: params.depth.clamp(0.0, 1.0),
            splay: params.splay.max(0.0),
            feather: params.feather.max(0.0),
            curve: params.curve.max(0.1),
        }
    }

    pub fn refraction(params: Refraction) -> Self {
        Self::Refraction(params.clamped())
    }

    pub fn luminosity(params: Luminosity) -> Self {
        Self::Luminosity(params.clamped())
    }

    pub fn noise(params: Noise) -> Self {
        Self::Noise(params.clamped())
    }

    pub fn clamped(self) -> Self {
        match self {
            Self::Blur { amount } => Self::blur(amount),
            Self::BackdropBlur(params) => Self::backdrop_blur(params),
            Self::Liquid {
                depth,
                splay,
                feather,
                curve,
            } => Self::liquid(LiquidFilter {
                depth,
                splay,
                feather,
                curve,
            }),
            Self::Refraction(params) => Self::refraction(params),
            Self::Luminosity(params) => Self::luminosity(params),
            Self::Noise(params) => Self::noise(params),
        }
    }
}

impl BackdropBlur {
    pub fn new(sigma: f32) -> Self {
        Self {
            sigma,
            edge_mode: BackdropEdgeMode::Mirror,
        }
    }

    pub fn clamped(self) -> Self {
        Self {
            sigma: self.sigma.max(0.0),
            edge_mode: self.edge_mode,
        }
    }
}

impl Refraction {
    const MAX_DISPLACEMENT: f32 = 4.0;

    pub fn new(displacement: f32, splay: f32, feather: f32, curve: f32) -> Self {
        Self {
            displacement,
            splay,
            feather,
            curve,
        }
    }

    pub fn clamped(self) -> Self {
        Self {
            displacement: self.displacement.clamp(0.0, Self::MAX_DISPLACEMENT),
            splay: self.splay.max(0.0),
            feather: self.feather.max(0.0),
            curve: self.curve.max(0.1),
        }
    }
}

impl Luminosity {
    pub fn new(color: Color, opacity: f32) -> Self {
        Self { color, opacity }
    }

    pub fn clamped(self) -> Self {
        Self {
            color: self.color,
            opacity: self.opacity.clamp(0.0, 1.0),
        }
    }
}

impl Noise {
    pub fn new(opacity: f32) -> Self {
        Self { opacity }
    }

    pub fn clamped(self) -> Self {
        Self {
            opacity: self.opacity.clamp(0.0, 1.0),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Style {
    pub fill: Option<Fill>,
    pub stroke: Option<Stroke>,
    pub tint: Option<Brush>,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Fill {
    Brush(Brush),
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Stroke {
    pub brush: Brush,
    pub width: f32,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Brush {
    Solid(Color),
    Gradient(Gradient),
}

impl Brush {
    pub const fn solid(color: Color) -> Self {
        Self::Solid(color)
    }

    pub fn linear_gradient(from: Color, to: Color) -> Self {
        Self::Gradient(Gradient::linear(from, to))
    }

    pub fn dimmed(self, factor: f32) -> Self {
        match self {
            Self::Solid(color) => Self::Solid(color.dimmed(factor)),
            Self::Gradient(gradient) => Self::Gradient(gradient.dimmed(factor)),
        }
    }

    pub fn is_visible(self) -> bool {
        match self {
            Self::Solid(color) => color.a > 0.0,
            Self::Gradient(gradient) => gradient.is_visible(),
        }
    }
}

impl From<Color> for Brush {
    fn from(color: Color) -> Self {
        Self::solid(color)
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Gradient {
    Linear(LinearGradient),
}

impl Gradient {
    pub fn linear(from: Color, to: Color) -> Self {
        Self::Linear(LinearGradient::new(
            UnitPoint::TOP_LEFT,
            UnitPoint::BOTTOM_RIGHT,
            from,
            to,
        ))
    }

    pub fn dimmed(self, factor: f32) -> Self {
        match self {
            Self::Linear(gradient) => Self::Linear(gradient.dimmed(factor)),
        }
    }

    pub fn is_visible(self) -> bool {
        match self {
            Self::Linear(gradient) => gradient.is_visible(),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct LinearGradient {
    start: UnitPoint,
    end: UnitPoint,
    from: Color,
    to: Color,
}

impl LinearGradient {
    pub const fn new(start: UnitPoint, end: UnitPoint, from: Color, to: Color) -> Self {
        Self {
            start,
            end,
            from,
            to,
        }
    }

    pub fn start(self) -> UnitPoint {
        self.start
    }

    pub fn end(self) -> UnitPoint {
        self.end
    }

    pub fn from(self) -> Color {
        self.from
    }

    pub fn to(self) -> Color {
        self.to
    }

    pub fn dimmed(self, factor: f32) -> Self {
        Self {
            start: self.start,
            end: self.end,
            from: self.from.dimmed(factor),
            to: self.to.dimmed(factor),
        }
    }

    pub fn is_visible(self) -> bool {
        self.from.a > 0.0 || self.to.a > 0.0
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct UnitPoint {
    x: f32,
    y: f32,
}

impl UnitPoint {
    pub const TOP_LEFT: Self = Self { x: 0.0, y: 0.0 };
    pub const BOTTOM_RIGHT: Self = Self { x: 1.0, y: 1.0 };

    pub fn new(x: f32, y: f32) -> Self {
        Self {
            x: x.clamp(0.0, 1.0),
            y: y.clamp(0.0, 1.0),
        }
    }

    pub fn x(self) -> f32 {
        self.x
    }

    pub fn y(self) -> f32 {
        self.y
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Color {
    pub r: f32,
    pub g: f32,
    pub b: f32,
    pub a: f32,
}

impl Color {
    pub const fn rgb(r: f32, g: f32, b: f32) -> Self {
        Self { r, g, b, a: 1.0 }
    }

    pub const fn rgba(r: f32, g: f32, b: f32, a: f32) -> Self {
        Self { r, g, b, a }
    }

    pub const BLACK: Self = Self {
        r: 0.0,
        g: 0.0,
        b: 0.0,
        a: 1.0,
    };

    pub const RED: Self = Self {
        r: 1.0,
        b: 0.0,
        g: 0.0,
        a: 1.0,
    };

    pub fn to_array(self) -> [f32; 4] {
        [self.r, self.g, self.b, self.a]
    }

    pub fn dimmed(self, factor: f32) -> Self {
        let factor = factor.max(0.0);

        Self {
            r: self.r * factor,
            g: self.g * factor,
            b: self.b * factor,
            a: self.a,
        }
    }
}

#[cfg(test)]
mod tests {
    use std::cell::RefCell;
    use std::rc::Rc;

    use crate::icon;
    use crate::paint_geometry::{area, point, rect};

    use super::*;

    fn solid_quad(x: f32) -> Quad {
        Quad {
            rect: Rect::new(point::logical(x, 0.0), area::logical(10.0, 10.0)),
            rasterization: Rasterization::default(),
            transform: Transform::identity(),
            style: Style {
                fill: Some(Fill::Brush(Brush::solid(Color::RED))),
                stroke: None,
                tint: None,
            },
        }
    }

    fn item_origin(item: &Item) -> Option<point::Logical> {
        match item {
            Item::Quad(item) => Some(item.rect.origin),
            Item::Text(item) => Some(item.rect.origin),
            Item::TextSurface(item) => Some(item.rect.origin),
            Item::TextViewport(item) => Some(item.rect.origin),
            Item::Icon(item) => Some(item.rect.origin),
            Item::Shadow(item) => Some(item.rect.origin),
            Item::Tint(item) => Some(item.rect.origin),
            Item::Outline(item) => Some(item.rect.origin),
            Item::Filter(item) => Some(item.rect.origin),
            Item::Layer(item) => Some(item.rect.origin),
            Item::Clip(item) => Some(item.rect.origin),
            Item::PopClip => None,
        }
    }

    #[test]
    fn new_scene_is_empty() {
        let scene = Scene::new();

        assert!(scene.is_empty());
        assert_eq!(scene.clear_color(), None);
        assert!(scene.items().is_empty());
    }

    #[test]
    fn clear_color_is_stored() {
        let mut scene = Scene::new();

        scene.clear(Color::BLACK);

        assert_eq!(scene.clear_color(), Some(Color::BLACK));
        assert!(!scene.is_empty());
    }

    #[test]
    fn pushed_items_preserve_order() {
        let mut scene = Scene::new();
        let first = solid_quad(1.0);
        let tint = Tint {
            rect: Rect::new(point::logical(1.25, 0.0), area::logical(10.0, 10.0)),
            brush: Brush::solid(Color::rgba(1.0, 1.0, 1.0, 0.2)),
        };
        let text = Text {
            rect: Rect::new(point::logical(1.5, 0.0), area::logical(10.0, 10.0)),
            document: text::document::Document::plain("Label"),
            wrap: TextWrap::WordOrGlyph,
            vertical_align: TextVerticalAlign::Center,
        };
        let icon = Icon {
            rect: Rect::new(point::logical(1.6, 0.0), area::logical(10.0, 10.0)),
            icon: icon::Icon::phosphor(icon::Id::new("check")),
            color: Color::BLACK,
            size: 16.0,
        };
        let shadow = Shadow {
            rect: Rect::new(point::logical(1.7, 0.0), area::logical(10.0, 10.0)),
            brush: Brush::solid(Color::rgba(0.0, 0.0, 0.0, 0.35)),
            blur: 16.0,
            spread: 1.0,
            offset: point::logical(0.0, 4.0),
        };
        let filter = Filter::blur(
            Rect::new(point::logical(1.72, 0.0), area::logical(10.0, 10.0)),
            0.5,
        );
        let clip = Clip {
            rect: Rect::new(point::logical(1.73, 0.0), area::logical(10.0, 10.0)),
        };
        let outline = Outline {
            rect: Rect::new(point::logical(1.75, 0.0), area::logical(10.0, 10.0)),
            brush: Brush::solid(Color::BLACK),
            width: 2.0,
            offset: 1.0,
        };
        let second = Quad {
            rect: Rect::rounded(
                point::logical(2.0, 0.0),
                area::logical(10.0, 10.0),
                rect::Rounding::none(),
            ),
            ..solid_quad(2.0)
        };

        scene.push_quad(first);
        scene.push_tint(tint);
        scene.push_icon(icon);
        scene.push_text(text.clone());
        scene.push_shadow(shadow);
        scene.push_filter(filter.clone());
        scene.push_clip(clip);
        scene.push_outline(outline);
        scene.pop_clip();
        scene.push_quad(second);

        assert_eq!(
            scene.items(),
            &[
                Item::Quad(first),
                Item::Tint(tint),
                Item::Icon(icon),
                Item::Text(text),
                Item::Shadow(shadow),
                Item::Filter(filter),
                Item::Clip(clip),
                Item::Outline(outline),
                Item::PopClip,
                Item::Quad(second)
            ]
        );
    }

    #[test]
    fn translate_items_moves_geometry_and_preserves_pop_clip() {
        let mut scene = Scene::new();
        let rect = Rect::new(point::logical(1.0, 2.0), area::logical(10.0, 10.0));
        scene.push_quad(Quad {
            rect,
            rasterization: Rasterization::default(),
            transform: Transform::identity(),
            style: Style {
                fill: None,
                stroke: None,
                tint: None,
            },
        });
        scene.push_text(Text {
            rect,
            document: text::document::Document::plain("Label"),
            wrap: TextWrap::WordOrGlyph,
            vertical_align: TextVerticalAlign::Start,
        });
        scene.push_text_surface(TextSurface {
            rect,
            buffer: Rc::new(RefCell::new(glyphon::Buffer::new_empty(
                glyphon::Metrics::relative(12.0, 1.25),
            ))),
            default_color: Color::BLACK,
        });
        scene.push_icon(Icon {
            rect,
            icon: icon::Icon::phosphor(icon::Id::new("check")),
            color: Color::BLACK,
            size: 16.0,
        });
        scene.push_shadow(Shadow {
            rect,
            brush: Brush::solid(Color::BLACK),
            blur: 4.0,
            spread: 1.0,
            offset: point::logical(0.0, 1.0),
        });
        scene.push_tint(Tint {
            rect,
            brush: Brush::solid(Color::BLACK),
        });
        scene.push_outline(Outline {
            rect,
            brush: Brush::solid(Color::BLACK),
            width: 1.0,
            offset: 0.0,
        });
        scene.push_filter(Filter::blur(rect, 1.0));
        scene.push_clip(Clip { rect });
        scene.pop_clip();

        let translated = scene.translate_items(0..scene.items().len(), point::logical(3.0, -1.0));

        assert_eq!(translated, 9);
        for item in &scene.items()[..9] {
            assert_eq!(item_origin(item), Some(point::logical(4.0, 1.0)));
        }
        assert_eq!(scene.items()[9], Item::PopClip);
    }

    #[test]
    fn shadow_item_preserves_shape_and_cutout_data() {
        let mut scene = Scene::new();
        let shadow = Shadow {
            rect: Rect::rounded(
                point::logical(0.0, 0.0),
                area::logical(20.0, 10.0),
                rect::Rounding::relative(1.0),
            ),
            brush: Brush::solid(Color::rgba(0.0, 0.0, 0.0, 0.3)),
            blur: 18.0,
            spread: 1.0,
            offset: point::logical(0.0, 6.0),
        };

        scene.push_shadow(shadow);

        assert_eq!(scene.items(), &[Item::Shadow(shadow)]);
    }

    #[test]
    fn filter_item_is_stored() {
        let mut scene = Scene::new();
        let filter = Filter::stack(
            Rect::new(point::logical(0.0, 0.0), area::logical(10.0, 10.0)),
            [
                FilterOp::Blur { amount: 0.5 },
                FilterOp::Liquid {
                    depth: 0.2,
                    splay: 2.0,
                    feather: 18.0,
                    curve: 2.0,
                },
            ],
        );

        scene.push_filter(filter.clone());

        assert_eq!(scene.items(), &[Item::Filter(filter)]);
    }

    #[test]
    fn filter_preserves_rounded_rect_shape() {
        let mut scene = Scene::new();
        let filter = Filter::blur(
            Rect::rounded(
                point::logical(0.0, 0.0),
                area::logical(20.0, 10.0),
                rect::Rounding::relative(1.0),
            ),
            0.5,
        );

        scene.push_filter(filter.clone());

        assert_eq!(scene.items(), &[Item::Filter(filter)]);
    }

    #[test]
    fn empty_and_zero_size_filters_are_skipped() {
        let mut scene = Scene::new();
        let rect = Rect::new(point::logical(0.0, 0.0), area::logical(10.0, 10.0));

        scene.push_filter(Filter::stack(rect, []));
        scene.push_filter(Filter::blur(
            Rect::new(point::logical(0.0, 0.0), area::logical(0.0, 10.0)),
            0.5,
        ));

        assert!(scene.items().is_empty());
    }

    #[test]
    fn filter_ops_clamp_parameters() {
        assert_eq!(FilterOp::blur(2.0), FilterOp::Blur { amount: 1.0 });
        assert_eq!(
            FilterOp::liquid(LiquidFilter {
                depth: 2.0,
                splay: -2.0,
                feather: -4.0,
                curve: 0.0,
            }),
            FilterOp::Liquid {
                depth: 1.0,
                splay: 0.0,
                feather: 0.0,
                curve: 0.1,
            }
        );
    }

    #[test]
    fn transform_scales_rect_about_origin() {
        let rect = Rect::new(point::logical(10.0, 20.0), area::logical(40.0, 10.0));
        let transform = Transform::scale_y_about(point::logical(30.0, 25.0), 1.5);

        assert_eq!(
            transform.transformed_rect(rect),
            Rect::new(point::logical(10.0, 17.5), area::logical(40.0, 15.0))
        );
    }

    #[test]
    fn translated_items_move_quad_transform_origin() {
        let mut scene = Scene::new();
        let mut quad = solid_quad(10.0);
        quad.transform = Transform::scale_y_about(point::logical(15.0, 5.0), 1.5);

        scene.push_quad(quad);
        scene.translate_items(0..scene.items().len(), point::logical(3.0, 4.0));

        let Item::Quad(translated) = scene.items()[0] else {
            panic!("expected translated quad");
        };
        assert_eq!(translated.rect.origin, point::logical(13.0, 4.0));
        assert_eq!(translated.transform.origin, point::logical(18.0, 9.0));
    }

    #[test]
    fn clip_commands_preserve_order_and_shape() {
        let mut scene = Scene::new();
        let clip = Clip {
            rect: Rect::rounded(
                point::logical(0.0, 0.0),
                area::logical(20.0, 10.0),
                rect::Rounding::relative(0.5),
            ),
        };
        let quad = solid_quad(1.0);

        scene.push_clip(clip);
        scene.push_quad(quad);
        scene.pop_clip();

        assert_eq!(
            scene.items(),
            &[Item::Clip(clip), Item::Quad(quad), Item::PopClip]
        );
    }

    #[test]
    fn replace_items_splices_scene_items_in_place() {
        let mut scene = Scene::new();
        let first = solid_quad(1.0);
        let second = solid_quad(2.0);
        let third = solid_quad(3.0);
        let replacement = solid_quad(4.0);

        scene.push_quad(first);
        scene.push_quad(second);
        scene.push_quad(third);

        scene.replace_items(1..2, [Item::Quad(replacement)]);

        assert_eq!(
            scene.items(),
            &[
                Item::Quad(first),
                Item::Quad(replacement),
                Item::Quad(third)
            ]
        );
    }

    #[test]
    fn color_converts_into_solid_brush() {
        let brush: Brush = Color::RED.into();

        assert_eq!(brush, Brush::Solid(Color::RED));
    }

    #[test]
    fn linear_gradient_brush_preserves_rgba_stops() {
        let from = Color::rgba(0.1, 0.2, 0.3, 0.4);
        let to = Color::rgba(0.5, 0.6, 0.7, 0.8);
        let Brush::Gradient(Gradient::Linear(gradient)) = Brush::linear_gradient(from, to) else {
            panic!("expected linear gradient brush");
        };

        assert_eq!(gradient.start(), UnitPoint::TOP_LEFT);
        assert_eq!(gradient.end(), UnitPoint::BOTTOM_RIGHT);
        assert_eq!(gradient.from(), from);
        assert_eq!(gradient.to(), to);
    }

    #[test]
    fn unit_point_clamps_to_normalized_range() {
        let point = UnitPoint::new(-1.0, 2.0);

        assert_eq!(point.x(), 0.0);
        assert_eq!(point.y(), 1.0);
    }

    #[test]
    fn brush_dim_preserves_alpha_and_dims_gradient_stops() {
        let brush = Brush::linear_gradient(
            Color::rgba(1.0, 0.5, 0.25, 0.4),
            Color::rgba(0.5, 0.25, 0.125, 0.8),
        )
        .dimmed(0.5);
        let Brush::Gradient(Gradient::Linear(gradient)) = brush else {
            panic!("expected linear gradient brush");
        };

        assert_eq!(gradient.from(), Color::rgba(0.5, 0.25, 0.125, 0.4));
        assert_eq!(gradient.to(), Color::rgba(0.25, 0.125, 0.0625, 0.8));
    }

    #[test]
    fn empty_text_is_not_pushed() {
        let mut scene = Scene::new();

        scene.push_text(Text {
            rect: Rect::new(point::logical(0.0, 0.0), area::logical(10.0, 10.0)),
            document: text::document::Document::plain(""),
            wrap: TextWrap::WordOrGlyph,
            vertical_align: TextVerticalAlign::Center,
        });

        assert!(scene.items().is_empty());
        assert!(scene.is_empty());
    }
}
