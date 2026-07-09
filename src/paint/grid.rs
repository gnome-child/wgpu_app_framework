use super::{area, point, rect};

#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) struct Grid {
    scale_factor: f32,
    logical_pixel: f32,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) struct SnappedOutset {
    pub(crate) base_rect: rect::Rect,
    pub(crate) inner_rect: rect::Rect,
    pub(crate) outer_rect: rect::Rect,
    pub(crate) offset: f32,
    pub(crate) width: f32,
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

    pub(crate) fn scale_factor(self) -> f32 {
        self.scale_factor
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

    pub(crate) fn snap_text_origin(self, origin: f32) -> f32 {
        round_ties_toward_zero(origin * self.scale_factor)
    }

    pub(crate) fn snap_centered_text_origin(
        self,
        origin: f32,
        extent: f32,
        content_extent: f32,
    ) -> f32 {
        let inset = (extent - extent.min(content_extent)).max(0.0) * 0.5;

        self.snap_text_origin(origin + inset)
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

    pub(crate) fn snap_rect_with_stable_size(self, rect: rect::Rect) -> rect::Rect {
        let (left, right) = self.snap_span_with_stable_distance(rect.origin.x(), rect.area.width());
        let (top, bottom) =
            self.snap_span_with_stable_distance(rect.origin.y(), rect.area.height());

        rect::Rect::rounded(
            point::logical(left, top),
            area::logical(right - left, bottom - top),
            rect.rounding,
        )
    }

    pub(crate) fn snap_outset(
        self,
        base_rect: rect::Rect,
        offset: f32,
        width: f32,
    ) -> SnappedOutset {
        let base_rect = self.snap_rect(base_rect);
        let offset = self.snap_distance(offset.max(0.0));
        let width = self.snap_distance(width.max(0.0));
        let inner_rect = outset_rect(base_rect, offset);
        let outer_rect = outset_rect(base_rect, offset + width);

        SnappedOutset {
            base_rect,
            inner_rect,
            outer_rect,
            offset,
            width,
        }
    }

    pub(crate) fn snap_vertical_rule_rect(self, rect: rect::Rect, width_px: u32) -> rect::Rect {
        let width = (width_px.max(1) as f32) / self.scale_factor;
        let center = rect.origin.x() + (rect.area.width() / 2.0);
        let left = self.snap_rule_start(center, width);
        let mut top = self.snap_position(rect.origin.y());
        let mut bottom = self.snap_position(rect.origin.y() + rect.area.height());

        if bottom <= top {
            top = self.snap_position(rect.origin.y());
            bottom = top + self.logical_pixel;
        }

        rect::Rect::rounded(
            point::logical(left, top),
            area::logical(width, bottom - top),
            rect.rounding,
        )
    }

    pub(crate) fn snap_horizontal_rule_rect(self, rect: rect::Rect, height_px: u32) -> rect::Rect {
        let height = (height_px.max(1) as f32) / self.scale_factor;
        let center = rect.origin.y() + (rect.area.height() / 2.0);
        let top = self.snap_rule_start(center, height);
        let mut left = self.snap_position(rect.origin.x());
        let mut right = self.snap_position(rect.origin.x() + rect.area.width());

        if right <= left {
            left = self.snap_position(rect.origin.x());
            right = left + self.logical_pixel;
        }

        rect::Rect::rounded(
            point::logical(left, top),
            area::logical(right - left, height),
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

    fn snap_rule_start(self, center: f32, thickness: f32) -> f32 {
        let physical_start = round_ties_toward_zero(
            (center * self.scale_factor) - ((thickness * self.scale_factor) / 2.0),
        );

        physical_start / self.scale_factor
    }

    fn snap_span_with_stable_distance(self, start: f32, distance: f32) -> (f32, f32) {
        if distance <= 0.0 {
            let start = self.snap_position(start);
            return (start, start + self.logical_pixel);
        }

        let snapped_distance = self.snap_distance(distance);
        let snapped_physical_distance = snapped_distance * self.scale_factor;
        let physical_center = (start + (distance / 2.0)) * self.scale_factor;
        let physical_start =
            round_ties_toward_zero(physical_center - (snapped_physical_distance / 2.0));
        let physical_end = physical_start + snapped_physical_distance;

        (
            physical_start / self.scale_factor,
            physical_end / self.scale_factor,
        )
    }
}

fn outset_rect(rect: rect::Rect, distance: f32) -> rect::Rect {
    rect::Rect::rounded(
        point::logical(rect.origin.x() - distance, rect.origin.y() - distance),
        area::logical(
            rect.area.width() + (distance * 2.0),
            rect.area.height() + (distance * 2.0),
        ),
        rect.rounding,
    )
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

    fn physical_edges(grid: Grid, rect: rect::Rect) -> (f32, f32, f32, f32) {
        let scale = grid.scale_factor;

        (
            rect.origin.x() * scale,
            rect.origin.y() * scale,
            (rect.origin.x() + rect.area.width()) * scale,
            (rect.origin.y() + rect.area.height()) * scale,
        )
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

    #[test]
    fn text_origin_snaps_through_device_space() {
        let grid = Grid::new(1.25);

        assert_approx_eq(grid.snap_text_origin(10.0), 12.0);
    }

    #[test]
    fn centered_text_origin_snaps_after_centering() {
        let grid = Grid::new(1.25);

        assert_approx_eq(grid.snap_centered_text_origin(10.0, 22.0, 11.0), 19.0);
    }

    #[test]
    fn stable_size_rect_snaps_scaled_plateau_to_aligned_edges() {
        let grid = Grid::new(1.0);
        let rect = rect::Rect::new(point::logical(10.0, 18.5), area::logical(40.0, 9.0));
        let snapped = grid.snap_rect_with_stable_size(rect);
        let (left, top, right, bottom) = bounds(snapped);

        assert_approx_eq(left, 10.0);
        assert_approx_eq(top, 18.0);
        assert_approx_eq(right, 50.0);
        assert_approx_eq(bottom, 27.0);
        assert!(grid.rect_is_aligned(snapped));
    }

    #[test]
    fn stable_size_rect_keeps_distance_stable_across_positions() {
        let grid = Grid::new(1.0);
        let first = grid.snap_rect_with_stable_size(rect::Rect::new(
            point::logical(10.0, 18.5),
            area::logical(40.0, 9.0),
        ));
        let second = grid.snap_rect_with_stable_size(rect::Rect::new(
            point::logical(10.0, 19.5),
            area::logical(40.0, 9.0),
        ));

        assert_approx_eq(first.area.height(), 9.0);
        assert_approx_eq(second.area.height(), 9.0);
        assert!(grid.rect_is_aligned(first));
        assert!(grid.rect_is_aligned(second));
    }

    #[test]
    fn outset_snaps_offset_and_width_independently_at_common_scales() {
        let base = rect::Rect::new(point::logical(10.0, 20.0), area::logical(80.0, 30.0));

        for scale in [1.0, 1.25, 1.5, 1.75, 2.0] {
            let grid = Grid::new(scale);
            let outset = grid.snap_outset(base, 2.0, 1.0);
            let (base_left, base_top, base_right, base_bottom) =
                physical_edges(grid, outset.base_rect);
            let (inner_left, inner_top, inner_right, inner_bottom) =
                physical_edges(grid, outset.inner_rect);
            let (outer_left, outer_top, outer_right, outer_bottom) =
                physical_edges(grid, outset.outer_rect);
            let gap = outset.offset * scale;
            let width = outset.width * scale;

            assert_approx_eq(base_left - inner_left, gap);
            assert_approx_eq(base_top - inner_top, gap);
            assert_approx_eq(inner_right - base_right, gap);
            assert_approx_eq(inner_bottom - base_bottom, gap);
            assert_approx_eq(inner_left - outer_left, width);
            assert_approx_eq(inner_top - outer_top, width);
            assert_approx_eq(outer_right - inner_right, width);
            assert_approx_eq(outer_bottom - inner_bottom, width);
            assert!(grid.rect_is_aligned(outset.inner_rect));
            assert!(grid.rect_is_aligned(outset.outer_rect));
        }
    }

    #[test]
    fn outset_at_one_point_five_keeps_focus_gap_centered() {
        let grid = Grid::new(1.5);
        let base = rect::Rect::new(point::logical(10.0, 20.0), area::logical(80.0, 30.0));
        let outset = grid.snap_outset(base, 2.0, 1.0);
        let (base_left, base_top, base_right, base_bottom) = physical_edges(grid, outset.base_rect);
        let (inner_left, inner_top, inner_right, inner_bottom) =
            physical_edges(grid, outset.inner_rect);
        let (outer_left, outer_top, outer_right, outer_bottom) =
            physical_edges(grid, outset.outer_rect);

        // Regression scale: offset=2lp becomes a 3dp gap, while width=1lp
        // becomes a 1dp ring. Snapping offset+width as one 4.5dp distance
        // would reintroduce the asymmetric tie.
        assert_approx_eq(base_left - inner_left, 3.0);
        assert_approx_eq(base_top - inner_top, 3.0);
        assert_approx_eq(inner_right - base_right, 3.0);
        assert_approx_eq(inner_bottom - base_bottom, 3.0);
        assert_approx_eq(inner_left - outer_left, 1.0);
        assert_approx_eq(inner_top - outer_top, 1.0);
        assert_approx_eq(outer_right - inner_right, 1.0);
        assert_approx_eq(outer_bottom - inner_bottom, 1.0);
    }
}
