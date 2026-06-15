use crate::geometry::{Rect, point};
use crate::icon;
use crate::text;

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

    pub fn push_backdrop(&mut self, backdrop: Backdrop) {
        self.items.push(Item::Backdrop(backdrop));
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

    pub fn is_empty(&self) -> bool {
        self.clear_color.is_none() && self.items.is_empty()
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
    Icon(Icon),
    Shadow(Shadow),
    Tint(Tint),
    Outline(Outline),
    Backdrop(Backdrop),
    Clip(Clip),
    PopClip,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Quad {
    pub rect: Rect,
    pub style: Style,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Text {
    pub rect: Rect,
    pub document: text::Document,
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

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Backdrop {
    pub rect: Rect,
    pub filter: BackdropFilter,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Clip {
    pub rect: Rect,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum BackdropFilter {
    Blur { amount: f32 },
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
    use crate::geometry::{area, point, rect};
    use crate::icon;

    use super::*;

    fn solid_quad(x: f32) -> Quad {
        Quad {
            rect: Rect::new(point::logical(x, 0.0), area::logical(10.0, 10.0)),
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
            document: text::Document::plain("Label"),
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
        let backdrop = Backdrop {
            rect: Rect::new(point::logical(1.72, 0.0), area::logical(10.0, 10.0)),
            filter: BackdropFilter::Blur { amount: 0.5 },
        };
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
                rect::Radius::none(),
            ),
            ..solid_quad(2.0)
        };

        scene.push_quad(first);
        scene.push_tint(tint);
        scene.push_icon(icon);
        scene.push_text(text.clone());
        scene.push_shadow(shadow);
        scene.push_backdrop(backdrop);
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
                Item::Backdrop(backdrop),
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
                point::logical(0.0, 0.0),
                area::logical(20.0, 10.0),
                rect::Radius::splat(1.0),
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
    fn backdrop_item_is_stored() {
        let mut scene = Scene::new();
        let backdrop = Backdrop {
            rect: Rect::new(point::logical(0.0, 0.0), area::logical(10.0, 10.0)),
            filter: BackdropFilter::Blur { amount: 0.5 },
        };

        scene.push_backdrop(backdrop);

        assert_eq!(scene.items(), &[Item::Backdrop(backdrop)]);
    }

    #[test]
    fn backdrop_preserves_rounded_rect_shape() {
        let mut scene = Scene::new();
        let backdrop = Backdrop {
            rect: Rect::rounded(
                point::logical(0.0, 0.0),
                area::logical(20.0, 10.0),
                rect::Radius::splat(1.0),
            ),
            filter: BackdropFilter::Blur { amount: 0.5 },
        };

        scene.push_backdrop(backdrop);

        assert_eq!(scene.items(), &[Item::Backdrop(backdrop)]);
    }

    #[test]
    fn clip_commands_preserve_order_and_shape() {
        let mut scene = Scene::new();
        let clip = Clip {
            rect: Rect::rounded(
                point::logical(0.0, 0.0),
                area::logical(20.0, 10.0),
                rect::Radius::splat(0.5),
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
            rect: Rect::new(point::logical(0.0, 0.0), area::logical(10.0, 10.0)),
            document: text::Document::plain(""),
        });

        assert!(scene.items().is_empty());
        assert!(scene.is_empty());
    }
}
