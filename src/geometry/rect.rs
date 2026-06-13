use crate::geometry::{area, point};

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Rect {
    pub origin: point::Logical,
    pub area: area::Logical,
    pub radius: Radius,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Radius {
    pub top_left: f32,
    pub top_right: f32,
    pub bottom_left: f32,
    pub bottom_right: f32,
}

impl Rect {
    pub fn new(origin: point::Logical, area: area::Logical) -> Self {
        Self {
            origin,
            area,
            radius: Radius::none(),
        }
    }

    pub fn rounded(origin: point::Logical, area: area::Logical, radius: Radius) -> Self {
        Self {
            origin,
            area,
            radius,
        }
    }
}

impl Radius {
    pub fn splat(radius: f32) -> Self {
        Self {
            top_left: radius,
            top_right: radius,
            bottom_left: radius,
            bottom_right: radius,
        }
    }

    pub fn none() -> Self {
        Self::splat(0.0)
    }

    pub fn left(radius: f32) -> Self {
        Self {
            bottom_right: 0.0,
            top_right: 0.0,
            ..Self::splat(radius)
        }
    }

    pub fn right(radius: f32) -> Self {
        Self {
            bottom_left: 0.0,
            top_left: 0.0,
            ..Self::splat(radius)
        }
    }

    pub fn top(radius: f32) -> Self {
        Self {
            bottom_left: 0.0,
            bottom_right: 0.0,
            ..Self::splat(radius)
        }
    }

    pub fn bottom(radius: f32) -> Self {
        Self {
            top_left: 0.0,
            top_right: 0.0,
            ..Self::splat(radius)
        }
    }
}
