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

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ResolvedRadius {
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

    pub fn resolve(self, area: area::Logical) -> ResolvedRadius {
        let max = area.width().min(area.height()).max(0.0) / 2.0;

        ResolvedRadius {
            top_left: normalized(self.top_left) * max,
            top_right: normalized(self.top_right) * max,
            bottom_left: normalized(self.bottom_left) * max,
            bottom_right: normalized(self.bottom_right) * max,
        }
    }
}

impl ResolvedRadius {
    pub fn none() -> Self {
        Self {
            top_left: 0.0,
            top_right: 0.0,
            bottom_left: 0.0,
            bottom_right: 0.0,
        }
    }

    pub fn is_none(self) -> bool {
        self == Self::none()
    }
}

fn normalized(value: f32) -> f32 {
    value.clamp(0.0, 1.0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn full_radius_resolves_to_half_size_on_square() {
        let radius = Radius::splat(1.0).resolve(area::logical(40.0, 40.0));

        assert_eq!(radius.top_left, 20.0);
        assert_eq!(radius.top_right, 20.0);
        assert_eq!(radius.bottom_left, 20.0);
        assert_eq!(radius.bottom_right, 20.0);
    }

    #[test]
    fn full_radius_resolves_to_pill_caps_on_rectangle() {
        let radius = Radius::splat(1.0).resolve(area::logical(100.0, 40.0));

        assert_eq!(radius.top_left, 20.0);
        assert_eq!(radius.top_right, 20.0);
        assert_eq!(radius.bottom_left, 20.0);
        assert_eq!(radius.bottom_right, 20.0);
    }

    #[test]
    fn radius_values_clamp_to_normalized_range() {
        let radius = Radius::splat(4.0).resolve(area::logical(30.0, 10.0));

        assert_eq!(radius.top_left, 5.0);
        assert_eq!(radius.top_right, 5.0);
        assert_eq!(radius.bottom_left, 5.0);
        assert_eq!(radius.bottom_right, 5.0);

        let radius = Radius::splat(-1.0).resolve(area::logical(30.0, 10.0));

        assert!(radius.is_none());
    }

    #[test]
    fn asymmetric_corner_radii_resolve_without_overlap() {
        let radius = Radius {
            top_left: 1.0,
            top_right: 0.5,
            bottom_left: 0.25,
            bottom_right: 2.0,
        }
        .resolve(area::logical(100.0, 40.0));

        assert_eq!(radius.top_left, 20.0);
        assert_eq!(radius.top_right, 10.0);
        assert_eq!(radius.bottom_left, 5.0);
        assert_eq!(radius.bottom_right, 20.0);
        assert!(radius.top_left + radius.top_right <= 100.0);
        assert!(radius.top_left + radius.bottom_left <= 40.0);
        assert!(radius.top_right + radius.bottom_right <= 40.0);
        assert!(radius.bottom_left + radius.bottom_right <= 100.0);
    }
}
