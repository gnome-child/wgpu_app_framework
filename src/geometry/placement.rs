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
}

impl Request {
    pub(crate) fn new(anchor: Anchor, desired: Size) -> Self {
        Self { anchor, desired }
    }

    pub(crate) fn anchor(self) -> Anchor {
        self.anchor
    }

    pub(crate) fn resolve(self, available: Rect) -> Rect {
        let candidates = candidates(self.anchor, self.desired);
        if let Some(candidate) = candidates
            .iter()
            .copied()
            .find(|candidate| contains_rect(available, *candidate))
        {
            return candidate;
        }

        let best = candidates
            .into_iter()
            .max_by_key(|candidate| intersection_area(*candidate, available))
            .expect("menu placement always has four candidates");
        clamp_origin(best, available)
    }
}

fn candidates(anchor: Anchor, desired: Size) -> [Rect; 4] {
    let (right_x, left_x, down_y, up_y) = match anchor {
        Anchor::Point(point) => (
            point.x(),
            point.x().saturating_sub(desired.width()),
            point.y(),
            point.y().saturating_sub(desired.height()),
        ),
        Anchor::Rect(rect) => (
            rect.x(),
            rect.right().saturating_sub(desired.width()),
            rect.bottom(),
            rect.y().saturating_sub(desired.height()),
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
}
