use crate::geometry::Rect;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Quad {
    pub rect: Rect,
    pub style: Style,
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
