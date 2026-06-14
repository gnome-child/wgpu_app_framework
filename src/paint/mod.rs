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

    pub fn push_backdrop_blur(&mut self, blur: Blur) {
        self.items.push(Item::BackdropBlur(blur));
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
    BackdropBlur(Blur),
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
    pub color: Color,
    pub blur: f32,
    pub spread: f32,
    pub offset: point::Logical,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Tint {
    pub rect: Rect,
    pub color: Color,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Outline {
    pub rect: Rect,
    pub brush: Brush,
    pub width: f32,
    pub offset: f32,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Blur {
    pub rect: Rect,
    pub radius: f32,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Style {
    pub fill: Option<Fill>,
    pub stroke: Option<Stroke>,
    pub tint: Option<Color>,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Fill {
    Brush(Brush),
    Blur,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Stroke {
    pub brush: Brush,
    pub width: f32,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Brush {
    Solid(Color),
    Gradient { from: Color, to: Color },
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
                fill: Some(Fill::Brush(Brush::Solid(Color::RED))),
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
            color: Color::rgba(1.0, 1.0, 1.0, 0.2),
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
            color: Color::rgba(0.0, 0.0, 0.0, 0.35),
            blur: 16.0,
            spread: 1.0,
            offset: point::logical(0.0, 4.0),
        };
        let outline = Outline {
            rect: Rect::new(point::logical(1.75, 0.0), area::logical(10.0, 10.0)),
            brush: Brush::Solid(Color::BLACK),
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
        scene.push_outline(outline);
        scene.push_quad(second);

        assert_eq!(
            scene.items(),
            &[
                Item::Quad(first),
                Item::Tint(tint),
                Item::Icon(icon),
                Item::Text(text),
                Item::Shadow(shadow),
                Item::Outline(outline),
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
            color: Color::rgba(0.0, 0.0, 0.0, 0.3),
            blur: 18.0,
            spread: 1.0,
            offset: point::logical(0.0, 6.0),
        };

        scene.push_shadow(shadow);

        assert_eq!(scene.items(), &[Item::Shadow(shadow)]);
    }

    #[test]
    fn backdrop_blur_item_is_stored() {
        let mut scene = Scene::new();
        let blur = Blur {
            rect: Rect::new(point::logical(0.0, 0.0), area::logical(10.0, 10.0)),
            radius: 8.0,
        };

        scene.push_backdrop_blur(blur);

        assert_eq!(scene.items(), &[Item::BackdropBlur(blur)]);
    }

    #[test]
    fn backdrop_blur_preserves_rounded_rect_shape() {
        let mut scene = Scene::new();
        let blur = Blur {
            rect: Rect::rounded(
                point::logical(0.0, 0.0),
                area::logical(20.0, 10.0),
                rect::Radius::splat(1.0),
            ),
            radius: 8.0,
        };

        scene.push_backdrop_blur(blur);

        assert_eq!(scene.items(), &[Item::BackdropBlur(blur)]);
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
