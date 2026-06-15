use crate::geometry::{area, point};

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Rect {
    pub origin: point::Logical,
    pub area: area::Logical,
    pub rounding: Rounding,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Radius {
    Relative(f32),
    Fixed(f32),
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Rounding {
    top_left: Radius,
    top_right: Radius,
    bottom_right: Radius,
    bottom_left: Radius,
}

impl Rect {
    pub fn new(origin: point::Logical, area: area::Logical) -> Self {
        Self {
            origin,
            area,
            rounding: Rounding::none(),
        }
    }

    pub fn rounded(origin: point::Logical, area: area::Logical, rounding: Rounding) -> Self {
        Self {
            origin,
            area,
            rounding,
        }
    }
}

impl Radius {
    pub fn relative(value: f32) -> Self {
        Self::Relative(value)
    }

    pub fn fixed(value: f32) -> Self {
        Self::Fixed(value)
    }

    fn resolve(self, max_relative: f32) -> f32 {
        match self {
            Self::Relative(value) => value.clamp(0.0, 1.0) * max_relative,
            Self::Fixed(value) => value.max(0.0),
        }
    }
}

impl Rounding {
    pub fn new(
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

    pub fn none() -> Self {
        Self::fixed(0.0)
    }

    pub fn relative(value: f32) -> Self {
        Self::all(Radius::relative(value))
    }

    pub fn fixed(value: f32) -> Self {
        Self::all(Radius::fixed(value))
    }

    pub fn relative_left(value: f32) -> Self {
        Self::left(Radius::relative(value))
    }

    pub fn relative_right(value: f32) -> Self {
        Self::right(Radius::relative(value))
    }

    pub fn relative_top(value: f32) -> Self {
        Self::top(Radius::relative(value))
    }

    pub fn relative_bottom(value: f32) -> Self {
        Self::bottom(Radius::relative(value))
    }

    pub fn fixed_left(value: f32) -> Self {
        Self::left(Radius::fixed(value))
    }

    pub fn fixed_right(value: f32) -> Self {
        Self::right(Radius::fixed(value))
    }

    pub fn fixed_top(value: f32) -> Self {
        Self::top(Radius::fixed(value))
    }

    pub fn fixed_bottom(value: f32) -> Self {
        Self::bottom(Radius::fixed(value))
    }

    pub fn top_left(self) -> Radius {
        self.top_left
    }

    pub fn top_right(self) -> Radius {
        self.top_right
    }

    pub fn bottom_right(self) -> Radius {
        self.bottom_right
    }

    pub fn bottom_left(self) -> Radius {
        self.bottom_left
    }

    pub fn resolve(self, area: area::Logical) -> [f32; 4] {
        let width = area.width().max(0.0);
        let height = area.height().max(0.0);
        let max_relative = width.min(height) / 2.0;

        let mut rounding = [
            self.top_left.resolve(max_relative),
            self.top_right.resolve(max_relative),
            self.bottom_right.resolve(max_relative),
            self.bottom_left.resolve(max_relative),
        ];

        let scale = 1.0_f32
            .min(edge_scale(width, rounding[0] + rounding[1]))
            .min(edge_scale(width, rounding[3] + rounding[2]))
            .min(edge_scale(height, rounding[0] + rounding[3]))
            .min(edge_scale(height, rounding[1] + rounding[2]));

        if scale < 1.0 {
            for value in &mut rounding {
                *value *= scale;
            }
        }

        rounding
    }

    fn all(radius: Radius) -> Self {
        Self::new(radius, radius, radius, radius)
    }

    fn left(radius: Radius) -> Self {
        Self {
            top_right: Radius::fixed(0.0),
            bottom_right: Radius::fixed(0.0),
            ..Self::all(radius)
        }
    }

    fn right(radius: Radius) -> Self {
        Self {
            top_left: Radius::fixed(0.0),
            bottom_left: Radius::fixed(0.0),
            ..Self::all(radius)
        }
    }

    fn top(radius: Radius) -> Self {
        Self {
            bottom_right: Radius::fixed(0.0),
            bottom_left: Radius::fixed(0.0),
            ..Self::all(radius)
        }
    }

    fn bottom(radius: Radius) -> Self {
        Self {
            top_left: Radius::fixed(0.0),
            top_right: Radius::fixed(0.0),
            ..Self::all(radius)
        }
    }
}

fn edge_scale(limit: f32, sum: f32) -> f32 {
    if sum > limit && sum > 0.0 {
        limit / sum
    } else {
        1.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn relative_full_rounding_resolves_to_half_size_on_square() {
        let rounding = Rounding::relative(1.0).resolve(area::logical(40.0, 40.0));

        assert_eq!(rounding, [20.0, 20.0, 20.0, 20.0]);
    }

    #[test]
    fn relative_full_rounding_resolves_to_pill_caps_on_rectangle() {
        let rounding = Rounding::relative(1.0).resolve(area::logical(100.0, 40.0));

        assert_eq!(rounding, [20.0, 20.0, 20.0, 20.0]);
    }

    #[test]
    fn relative_values_clamp_to_normalized_range() {
        let rounding = Rounding::relative(4.0).resolve(area::logical(30.0, 10.0));

        assert_eq!(rounding, [5.0, 5.0, 5.0, 5.0]);

        let rounding = Rounding::relative(-1.0).resolve(area::logical(30.0, 10.0));

        assert_eq!(rounding, [0.0, 0.0, 0.0, 0.0]);
    }

    #[test]
    fn fixed_rounding_resolves_consistently_across_rect_sizes() {
        let small = Rounding::fixed(8.0).resolve(area::logical(40.0, 20.0));
        let large = Rounding::fixed(8.0).resolve(area::logical(400.0, 200.0));

        assert_eq!(small, [8.0, 8.0, 8.0, 8.0]);
        assert_eq!(large, [8.0, 8.0, 8.0, 8.0]);
    }

    #[test]
    fn fixed_negative_values_clamp_to_zero() {
        let rounding = Rounding::fixed(-8.0).resolve(area::logical(40.0, 20.0));

        assert_eq!(rounding, [0.0, 0.0, 0.0, 0.0]);
    }

    #[test]
    fn fixed_and_relative_mixed_corners_scale_without_overlap() {
        let rounding = Rounding::new(
            Radius::fixed(30.0),
            Radius::relative(1.0),
            Radius::fixed(20.0),
            Radius::fixed(10.0),
        )
        .resolve(area::logical(40.0, 30.0));

        assert_eq!(rounding, [22.5, 11.25, 15.0, 7.5]);
        assert!(rounding[0] + rounding[1] <= 40.0);
        assert!(rounding[3] + rounding[2] <= 40.0);
        assert!(rounding[0] + rounding[3] <= 30.0);
        assert!(rounding[1] + rounding[2] <= 30.0);
    }
}
