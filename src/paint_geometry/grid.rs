use super::{area, point, rect};

#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) struct Grid {
    scale_factor: f32,
    logical_pixel: f32,
}

impl Grid {
    pub(crate) fn new(scale_factor: f32) -> Self {
        let scale_factor = scale_factor.max(0.0001);

        Self {
            scale_factor,
            logical_pixel: 1.0 / scale_factor,
        }
    }

    pub(crate) fn logical_pixel(self) -> f32 {
        self.logical_pixel
    }

    pub(crate) fn snap_position(self, position: f32) -> f32 {
        round_ties_toward_zero(position * self.scale_factor) / self.scale_factor
    }

    pub(crate) fn snap_distance(self, distance: f32) -> f32 {
        if distance <= 0.0 {
            return 0.0;
        }

        round_ties_toward_zero(distance * self.scale_factor).max(1.0) / self.scale_factor
    }

    pub(crate) fn snap_rect(self, rect: rect::Rect) -> rect::Rect {
        let left = self.snap_position(rect.origin.x());
        let top = self.snap_position(rect.origin.y());
        let mut right = self.snap_position(rect.origin.x() + rect.area.width());
        let mut bottom = self.snap_position(rect.origin.y() + rect.area.height());

        if right <= left {
            right = left + self.logical_pixel;
        }

        if bottom <= top {
            bottom = top + self.logical_pixel;
        }

        rect::Rect::rounded(
            point::logical(left, top),
            area::logical(right - left, bottom - top),
            rect.rounding,
        )
    }

    pub(crate) fn snap_fixed_width_rect(self, rect: rect::Rect, width_px: u32) -> rect::Rect {
        let left = self.snap_position(rect.origin.x());
        let top = self.snap_position(rect.origin.y());
        let mut bottom = self.snap_position(rect.origin.y() + rect.area.height());
        let width = (width_px.max(1) as f32) / self.scale_factor;

        if bottom <= top {
            bottom = top + self.logical_pixel;
        }

        rect::Rect::rounded(
            point::logical(left, top),
            area::logical(width, bottom - top),
            rect.rounding,
        )
    }

    pub(crate) fn rect_is_aligned(self, rect: rect::Rect) -> bool {
        self.position_is_aligned(rect.origin.x())
            && self.position_is_aligned(rect.origin.y())
            && self.position_is_aligned(rect.origin.x() + rect.area.width())
            && self.position_is_aligned(rect.origin.y() + rect.area.height())
    }

    fn position_is_aligned(self, position: f32) -> bool {
        let physical = position * self.scale_factor;

        (physical - round_ties_toward_zero(physical)).abs() <= 0.001
    }
}

fn round_ties_toward_zero(value: f32) -> f32 {
    let truncated = value.trunc();
    let fraction = (value - truncated).abs();

    if fraction == 0.5 {
        truncated
    } else {
        value.round()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn bounds(rect: rect::Rect) -> (f32, f32, f32, f32) {
        (
            rect.origin.x(),
            rect.origin.y(),
            rect.origin.x() + rect.area.width(),
            rect.origin.y() + rect.area.height(),
        )
    }

    fn assert_approx_eq(left: f32, right: f32) {
        assert!(
            (left - right).abs() <= 0.0001,
            "expected {left} to be approximately {right}"
        );
    }

    #[test]
    fn grid_snaps_to_fractional_logical_values_at_fractional_scale() {
        let grid = Grid::new(1.25);
        let rect = rect::Rect::new(point::logical(10.0, 20.0), area::logical(33.0, 11.0));
        let snapped = grid.snap_rect(rect);
        let (left, top, right, bottom) = bounds(snapped);

        // 10.0 * 1.25 = 12.5 device px, so the exact tie rounds toward zero.
        assert_approx_eq(left, 9.6);
        assert_approx_eq(top, 20.0);
        assert_approx_eq(right, 43.2);
        assert_approx_eq(bottom, 31.2);
        assert!(grid.rect_is_aligned(snapped));
    }

    #[test]
    fn positive_midpoint_positions_round_toward_zero() {
        let grid = Grid::new(1.5);

        assert_approx_eq(grid.snap_position(1.0), 2.0 / 3.0);
    }

    #[test]
    fn negative_midpoint_positions_round_toward_zero() {
        let grid = Grid::new(1.5);

        assert_approx_eq(grid.snap_position(-1.0), -2.0 / 3.0);
    }

    #[test]
    fn non_midpoint_positions_still_round_to_nearest() {
        let grid = Grid::new(1.5);

        assert_approx_eq(grid.snap_position(1.1), 4.0 / 3.0);
        assert_approx_eq(grid.snap_position(-1.1), -4.0 / 3.0);
    }

    #[test]
    fn positive_midpoint_distance_at_fractional_scale_stays_thin() {
        let grid = Grid::new(1.5);

        assert_approx_eq(grid.snap_distance(1.0), 2.0 / 3.0);
    }

    #[test]
    fn tiny_positive_distances_snap_to_at_least_one_physical_pixel() {
        let grid = Grid::new(1.5);

        assert_approx_eq(grid.snap_distance(0.1), 2.0 / 3.0);
    }
}
