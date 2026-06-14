use crate::geometry::Rect;
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
        let text = Text {
            rect: Rect::new(point::logical(1.5, 0.0), area::logical(10.0, 10.0)),
            document: text::Document::plain("Label"),
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
        scene.push_text(text.clone());
        scene.push_quad(second);

        assert_eq!(
            scene.items(),
            &[Item::Quad(first), Item::Text(text), Item::Quad(second)]
        );
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
