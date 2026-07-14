use super::{Point, Rect, Size};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Anchor {
    Point(Point),
    Rect(Rect),
}

impl Anchor {
    pub(crate) fn reference_point(self) -> Point {
        match self {
            Self::Point(point) => point,
            Self::Rect(rect) => Point::new(rect.x(), rect.y()),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct Request {
    anchor: Anchor,
    desired: Size,
    clearance: i32,
}

impl Request {
    pub(crate) fn new(anchor: Anchor, desired: Size) -> Self {
        Self {
            anchor,
            desired,
            clearance: 0,
        }
    }

    /// Keeps a point-attached panel away from its pointer hotspot while
    /// preserving the same four-candidate edge solver used by context menus.
    pub(crate) fn with_clearance(mut self, clearance: i32) -> Self {
        self.clearance = clearance.max(0);
        self
    }

    pub(crate) fn anchor(self) -> Anchor {
        self.anchor
    }

    pub(crate) fn resolve(self, available: Rect) -> Rect {
        let candidates = candidates(self.anchor, self.desired, self.clearance);
        if let Some(candidate) = candidates
            .iter()
            .copied()
            .find(|candidate| contains_rect(available, *candidate))
        {
            return candidate;
        }

        let [first, second, third, fourth] = candidates;
        let best = [second, third, fourth]
            .into_iter()
            .fold(first, |best, candidate| {
                if intersection_area(candidate, available) >= intersection_area(best, available) {
                    candidate
                } else {
                    best
                }
            });
        clamp_origin(best, available)
    }
}

fn candidates(anchor: Anchor, desired: Size, clearance: i32) -> [Rect; 4] {
    let (right_x, left_x, down_y, up_y) = match anchor {
        Anchor::Point(point) => (
            point.x().saturating_add(clearance),
            point
                .x()
                .saturating_sub(desired.width())
                .saturating_sub(clearance),
            point.y().saturating_add(clearance),
            point
                .y()
                .saturating_sub(desired.height())
                .saturating_sub(clearance),
        ),
        Anchor::Rect(rect) => (
            rect.x(),
            rect.right().saturating_sub(desired.width()),
            rect.bottom().saturating_add(clearance),
            rect.y()
                .saturating_sub(desired.height())
                .saturating_sub(clearance),
        ),
    };

    [
        Rect::new(right_x, down_y, desired.width(), desired.height()),
        Rect::new(left_x, down_y, desired.width(), desired.height()),
        Rect::new(right_x, up_y, desired.width(), desired.height()),
        Rect::new(left_x, up_y, desired.width(), desired.height()),
    ]
}

fn contains_rect(outer: Rect, inner: Rect) -> bool {
    inner.x() >= outer.x()
        && inner.y() >= outer.y()
        && inner.right() <= outer.right()
        && inner.bottom() <= outer.bottom()
}

fn intersection_area(left: Rect, right: Rect) -> i64 {
    let width = left
        .right()
        .min(right.right())
        .saturating_sub(left.x().max(right.x()))
        .max(0);
    let height = left
        .bottom()
        .min(right.bottom())
        .saturating_sub(left.y().max(right.y()))
        .max(0);
    i64::from(width) * i64::from(height)
}

fn clamp_origin(rect: Rect, available: Rect) -> Rect {
    let max_x = available.right().saturating_sub(rect.width());
    let max_y = available.bottom().saturating_sub(rect.height());
    Rect::new(
        rect.x().clamp(available.x(), max_x.max(available.x())),
        rect.y().clamp(available.y(), max_y.max(available.y())),
        rect.width(),
        rect.height(),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn point_anchor_flips_at_every_available_corner() {
        let available = Rect::new(0, 0, 100, 80);
        let desired = Size::new(30, 20);
        assert_eq!(
            Request::new(Anchor::Point(Point::new(0, 0)), desired).resolve(available),
            Rect::new(0, 0, 30, 20)
        );
        assert_eq!(
            Request::new(Anchor::Point(Point::new(100, 0)), desired).resolve(available),
            Rect::new(70, 0, 30, 20)
        );
        assert_eq!(
            Request::new(Anchor::Point(Point::new(0, 80)), desired).resolve(available),
            Rect::new(0, 60, 30, 20)
        );
        assert_eq!(
            Request::new(Anchor::Point(Point::new(100, 80)), desired).resolve(available),
            Rect::new(70, 60, 30, 20)
        );
    }

    #[test]
    fn rectangle_anchor_prefers_below_then_flips_above() {
        let available = Rect::new(0, 0, 120, 100);
        let desired = Size::new(50, 30);
        assert_eq!(
            Request::new(Anchor::Rect(Rect::new(10, 10, 20, 10)), desired).resolve(available),
            Rect::new(10, 20, 50, 30)
        );
        assert_eq!(
            Request::new(Anchor::Rect(Rect::new(10, 90, 20, 10)), desired).resolve(available),
            Rect::new(10, 60, 50, 30)
        );
    }

    #[test]
    fn final_fallback_clamps_without_resizing() {
        let available = Rect::new(-40, -20, 60, 40);
        let desired = Size::new(90, 70);
        let resolved = Request::new(Anchor::Point(Point::new(0, 0)), desired).resolve(available);
        assert_eq!(resolved, Rect::new(-40, -20, 90, 70));
        assert_eq!(resolved.width(), desired.width());
        assert_eq!(resolved.height(), desired.height());
    }

    #[test]
    fn equal_fallback_areas_keep_the_last_candidate_preference() {
        let available = Rect::new(0, 0, 100, 80);
        let desired = Size::new(120, 20);

        assert_eq!(
            Request::new(Anchor::Point(Point::new(50, 40)), desired).resolve(available),
            Rect::new(0, 20, 120, 20)
        );
    }

    #[test]
    fn point_clearance_survives_every_edge_flip() {
        let available = Rect::new(0, 0, 100, 80);
        let desired = Size::new(30, 20);
        let resolve = |point| {
            Request::new(Anchor::Point(point), desired)
                .with_clearance(8)
                .resolve(available)
        };

        assert_eq!(resolve(Point::new(20, 20)), Rect::new(28, 28, 30, 20));
        assert_eq!(resolve(Point::new(80, 20)), Rect::new(42, 28, 30, 20));
        assert_eq!(resolve(Point::new(20, 60)), Rect::new(28, 32, 30, 20));
        assert_eq!(resolve(Point::new(80, 60)), Rect::new(42, 32, 30, 20));
    }
}
