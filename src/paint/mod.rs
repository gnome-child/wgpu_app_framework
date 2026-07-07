use crate::icon;
use crate::paint_geometry::{self, Rect};
use crate::text;
use std::cell::RefCell;
use std::fmt;
use std::rc::Rc;

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

    pub fn push_clip(&mut self, clip: Clip) {
        self.items.push(Item::Clip(clip));
    }

    pub fn pop_clip(&mut self) {
        self.items.push(Item::PopClip);
    }

    pub fn items(&self) -> &[Item] {
        &self.items
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
    TextViewport(TextViewport),
    Icon(Icon),
    Shadow(Shadow),
    Outline(Outline),
    Filter(Filter),
    Clip(Clip),
    PopClip,
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
    pub origin: paint_geometry::LogicalPoint,
    pub translate: paint_geometry::LogicalPoint,
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
    pub offset: paint_geometry::LogicalPoint,
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
    #[cfg(test)]
    pub fn blur(rect: Rect, amount: f32) -> Self {
        Self::stack(rect, [FilterOp::blur(amount)])
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
            origin: paint_geometry::logical_point(0.0, 0.0),
            translate: paint_geometry::logical_point(0.0, 0.0),
            scale_x: 1.0,
            scale_y: 1.0,
        }
    }

    #[cfg(test)]
    pub fn scale_about(origin: paint_geometry::LogicalPoint, scale_x: f32, scale_y: f32) -> Self {
        Self {
            origin,
            scale_x: sanitized_scale(scale_x),
            scale_y: sanitized_scale(scale_y),
            ..Self::identity()
        }
    }

    #[cfg(test)]
    pub fn scale_y_about(origin: paint_geometry::LogicalPoint, scale_y: f32) -> Self {
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
            paint_geometry::logical_point(left, top),
            paint_geometry::logical_area((right - left).max(0.0), (bottom - top).max(0.0)),
            rect.rounding,
        )
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

#[cfg(test)]
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
    pub fn clamped(self) -> Self {
        Self {
            sigma: self.sigma.max(0.0),
            edge_mode: self.edge_mode,
        }
    }
}

impl Refraction {
    const MAX_DISPLACEMENT: f32 = 4.0;

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
    pub fn clamped(self) -> Self {
        Self {
            color: self.color,
            opacity: self.opacity.clamp(0.0, 1.0),
        }
    }
}

impl Noise {
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

    #[cfg(test)]
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

    #[cfg(test)]
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

    #[cfg(test)]
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

    #[cfg(test)]
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
    pub const fn rgba(r: f32, g: f32, b: f32, a: f32) -> Self {
        Self { r, g, b, a }
    }

    #[cfg(test)]
    pub const BLACK: Self = Self {
        r: 0.0,
        g: 0.0,
        b: 0.0,
        a: 1.0,
    };

    #[cfg(test)]
    pub const RED: Self = Self {
        r: 1.0,
        b: 0.0,
        g: 0.0,
        a: 1.0,
    };

    pub fn to_array(self) -> [f32; 4] {
        [self.r, self.g, self.b, self.a]
    }

    #[cfg(test)]
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
    use crate::icon;
    use crate::paint_geometry;

    use super::*;

    fn solid_quad(x: f32) -> Quad {
        Quad {
            rect: Rect::new(
                paint_geometry::logical_point(x, 0.0),
                paint_geometry::logical_area(10.0, 10.0),
            ),
            rasterization: Rasterization::default(),
            transform: Transform::identity(),
            style: Style {
                fill: Some(Fill::Brush(Brush::solid(Color::RED))),
                stroke: None,
                tint: None,
            },
        }
    }

    #[test]
    fn new_scene_is_empty() {
        let scene = Scene::new();

        assert_eq!(scene.clear_color(), None);
        assert!(scene.items().is_empty());
    }

    #[test]
    fn clear_color_is_stored() {
        let mut scene = Scene::new();

        scene.clear(Color::BLACK);

        assert_eq!(scene.clear_color(), Some(Color::BLACK));
        assert!(scene.items().is_empty());
    }

    #[test]
    fn pushed_items_preserve_order() {
        let mut scene = Scene::new();
        let first = solid_quad(1.0);
        let text = Text {
            rect: Rect::new(
                paint_geometry::logical_point(1.5, 0.0),
                paint_geometry::logical_area(10.0, 10.0),
            ),
            document: text::document::Document::plain("Label"),
            wrap: TextWrap::WordOrGlyph,
            vertical_align: TextVerticalAlign::Center,
        };
        let icon = Icon {
            rect: Rect::new(
                paint_geometry::logical_point(1.6, 0.0),
                paint_geometry::logical_area(10.0, 10.0),
            ),
            icon: icon::Icon::phosphor(icon::Id::new("check")),
            color: Color::BLACK,
            size: 16.0,
        };
        let shadow = Shadow {
            rect: Rect::new(
                paint_geometry::logical_point(1.7, 0.0),
                paint_geometry::logical_area(10.0, 10.0),
            ),
            brush: Brush::solid(Color::rgba(0.0, 0.0, 0.0, 0.35)),
            blur: 16.0,
            spread: 1.0,
            offset: paint_geometry::logical_point(0.0, 4.0),
        };
        let filter = Filter::blur(
            Rect::new(
                paint_geometry::logical_point(1.72, 0.0),
                paint_geometry::logical_area(10.0, 10.0),
            ),
            0.5,
        );
        let clip = Clip {
            rect: Rect::new(
                paint_geometry::logical_point(1.73, 0.0),
                paint_geometry::logical_area(10.0, 10.0),
            ),
        };
        let outline = Outline {
            rect: Rect::new(
                paint_geometry::logical_point(1.75, 0.0),
                paint_geometry::logical_area(10.0, 10.0),
            ),
            brush: Brush::solid(Color::BLACK),
            width: 2.0,
            offset: 1.0,
        };
        let second = Quad {
            rect: Rect::rounded(
                paint_geometry::logical_point(2.0, 0.0),
                paint_geometry::logical_area(10.0, 10.0),
                paint_geometry::Rounding::none(),
            ),
            ..solid_quad(2.0)
        };

        scene.push_quad(first);
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
    fn shadow_item_preserves_shape_and_cutout_data() {
        let mut scene = Scene::new();
        let shadow = Shadow {
            rect: Rect::rounded(
                paint_geometry::logical_point(0.0, 0.0),
                paint_geometry::logical_area(20.0, 10.0),
                paint_geometry::Rounding::relative(1.0),
            ),
            brush: Brush::solid(Color::rgba(0.0, 0.0, 0.0, 0.3)),
            blur: 18.0,
            spread: 1.0,
            offset: paint_geometry::logical_point(0.0, 6.0),
        };

        scene.push_shadow(shadow);

        assert_eq!(scene.items(), &[Item::Shadow(shadow)]);
    }

    #[test]
    fn filter_item_is_stored() {
        let mut scene = Scene::new();
        let filter = Filter::stack(
            Rect::new(
                paint_geometry::logical_point(0.0, 0.0),
                paint_geometry::logical_area(10.0, 10.0),
            ),
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
                paint_geometry::logical_point(0.0, 0.0),
                paint_geometry::logical_area(20.0, 10.0),
                paint_geometry::Rounding::relative(1.0),
            ),
            0.5,
        );

        scene.push_filter(filter.clone());

        assert_eq!(scene.items(), &[Item::Filter(filter)]);
    }

    #[test]
    fn empty_and_zero_size_filters_are_skipped() {
        let mut scene = Scene::new();
        let rect = Rect::new(
            paint_geometry::logical_point(0.0, 0.0),
            paint_geometry::logical_area(10.0, 10.0),
        );

        scene.push_filter(Filter::stack(rect, []));
        scene.push_filter(Filter::blur(
            Rect::new(
                paint_geometry::logical_point(0.0, 0.0),
                paint_geometry::logical_area(0.0, 10.0),
            ),
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
        let rect = Rect::new(
            paint_geometry::logical_point(10.0, 20.0),
            paint_geometry::logical_area(40.0, 10.0),
        );
        let transform = Transform::scale_y_about(paint_geometry::logical_point(30.0, 25.0), 1.5);

        assert_eq!(
            transform.transformed_rect(rect),
            Rect::new(
                paint_geometry::logical_point(10.0, 17.5),
                paint_geometry::logical_area(40.0, 15.0)
            )
        );
    }

    #[test]
    fn clip_commands_preserve_order_and_shape() {
        let mut scene = Scene::new();
        let clip = Clip {
            rect: Rect::rounded(
                paint_geometry::logical_point(0.0, 0.0),
                paint_geometry::logical_area(20.0, 10.0),
                paint_geometry::Rounding::relative(0.5),
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
            rect: Rect::new(
                paint_geometry::logical_point(0.0, 0.0),
                paint_geometry::logical_area(10.0, 10.0),
            ),
            document: text::document::Document::plain(""),
            wrap: TextWrap::WordOrGlyph,
            vertical_align: TextVerticalAlign::Center,
        });

        assert!(scene.items().is_empty());
    }
}
