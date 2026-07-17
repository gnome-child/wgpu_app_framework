use super::super::{
    geometry::{Rect, Size},
    interaction::{ScrollDelta, ScrollOffset, ScrollbarAxis},
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct Viewport {
    rect: Rect,
    visible_frame: Rect,
    visible_content: Rect,
    content: Size,
    offset: ScrollOffset,
    max: ScrollOffset,
    resolved: ScrollOffset,
}

impl Viewport {
    pub(crate) fn new(rect: Rect, content: Size, offset: ScrollOffset) -> Self {
        let max = ScrollOffset::new(
            content.width().saturating_sub(rect.width()).max(0),
            content.height().saturating_sub(rect.height()).max(0),
        );
        let resolved = offset.clamped(ScrollOffset::default(), max);

        Self {
            rect,
            visible_frame: rect,
            visible_content: rect,
            content,
            offset,
            max,
            resolved,
        }
    }

    pub(crate) fn rect(self) -> Rect {
        self.rect
    }

    pub(crate) fn with_visible(mut self, frame: Rect, content: Rect) -> Self {
        self.visible_frame = frame;
        self.visible_content = content;
        self
    }

    pub(crate) fn visible_frame(self) -> Rect {
        self.visible_frame
    }

    pub(crate) fn visible_content(self) -> Rect {
        self.visible_content
    }

    pub(crate) fn viewport_content_coverage(self) -> Option<Rect> {
        self.content_coverage_within(self.rect, self.resolved)
    }

    fn content_coverage_within(self, clip: Rect, offset: ScrollOffset) -> Option<Rect> {
        let content = Rect::new(
            self.rect.x().saturating_sub(offset.x()),
            self.rect.y().saturating_sub(offset.y()),
            self.content.width(),
            self.content.height(),
        );
        intersect_rect(clip, content)
    }

    pub(crate) fn content(self) -> Size {
        self.content
    }

    pub(crate) fn max_scroll(self) -> ScrollOffset {
        self.max
    }

    pub(crate) fn resolved_scroll(self) -> ScrollOffset {
        self.resolved
    }

    pub(crate) fn resolve(self, offset: ScrollOffset) -> ScrollOffset {
        offset.clamped(ScrollOffset::default(), self.max)
    }

    pub(crate) fn can_consume_from(self, offset: ScrollOffset, delta: ScrollDelta) -> bool {
        let resolved = self.resolve(offset);
        (delta.x() < 0.0
            && resolved
                .axis_cmp(ScrollOffset::default(), ScrollbarAxis::Horizontal)
                .is_gt())
            || (delta.x() > 0.0
                && resolved
                    .axis_cmp(self.max, ScrollbarAxis::Horizontal)
                    .is_lt())
            || (delta.y() < 0.0
                && resolved
                    .axis_cmp(ScrollOffset::default(), ScrollbarAxis::Vertical)
                    .is_gt())
            || (delta.y() > 0.0 && resolved.axis_cmp(self.max, ScrollbarAxis::Vertical).is_lt())
    }

    pub(crate) fn is_scrollable(self) -> bool {
        self.max.x() > 0 || self.max.y() > 0
    }

    pub(crate) fn reveal_rect(self, rect: Rect, margin: i32) -> ScrollOffset {
        ScrollOffset::new(
            reveal_axis(
                self.resolved.x(),
                self.max.x(),
                self.visible_content.x(),
                self.visible_content.width(),
                rect.x(),
                rect.width(),
                margin,
            ),
            reveal_axis(
                self.resolved.y(),
                self.max.y(),
                self.visible_content.y(),
                self.visible_content.height(),
                rect.y(),
                rect.height(),
                margin,
            ),
        )
    }
}

fn intersect_rect(left: Rect, right: Rect) -> Option<Rect> {
    let x = left.x().max(right.x());
    let y = left.y().max(right.y());
    let right_edge = left.right().min(right.right());
    let bottom = left.bottom().min(right.bottom());
    (right_edge > x && bottom > y)
        .then(|| Rect::new(x, y, right_edge.saturating_sub(x), bottom.saturating_sub(y)))
}

fn reveal_axis(
    current: i32,
    max: i32,
    viewport_start: i32,
    viewport_extent: i32,
    target_start: i32,
    target_extent: i32,
    margin: i32,
) -> i32 {
    let margin = margin.max(0).min(viewport_extent.saturating_div(2));
    let visible_start = viewport_start.saturating_add(margin);
    let visible_end = viewport_start.saturating_add(viewport_extent.saturating_sub(margin));
    let visible_extent = visible_end.saturating_sub(visible_start);
    let target_end = target_start.saturating_add(target_extent);
    let delta = if target_extent > visible_extent || target_start < visible_start {
        target_start as i64 - visible_start as i64
    } else if target_end > visible_end {
        target_end as i64 - visible_end as i64
    } else {
        0
    };

    ((current as i64 + delta).clamp(0, max as i64)) as i32
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reveal_is_noop_when_target_fully_visible() {
        let viewport = Viewport::new(
            Rect::new(0, 0, 100, 100),
            Size::new(100, 300),
            ScrollOffset::default(),
        );

        assert_eq!(
            viewport.reveal_rect(Rect::new(0, 20, 100, 20), 0),
            ScrollOffset::default()
        );
    }

    #[test]
    fn reveal_scrolls_bottom_edge_flush_when_target_is_below() {
        let viewport = Viewport::new(
            Rect::new(0, 0, 100, 100),
            Size::new(100, 300),
            ScrollOffset::default(),
        );

        assert_eq!(
            viewport.reveal_rect(Rect::new(0, 120, 100, 20), 0),
            ScrollOffset::new(0, 40)
        );
    }

    #[test]
    fn reveal_scrolls_top_edge_flush_when_target_is_above() {
        let viewport = Viewport::new(
            Rect::new(0, 0, 100, 100),
            Size::new(100, 300),
            ScrollOffset::new(0, 80),
        );

        assert_eq!(
            viewport.reveal_rect(Rect::new(0, -20, 100, 20), 0),
            ScrollOffset::new(0, 60)
        );
    }

    #[test]
    fn reveal_aligns_top_when_target_exceeds_viewport() {
        let viewport = Viewport::new(
            Rect::new(0, 0, 100, 100),
            Size::new(100, 300),
            ScrollOffset::default(),
        );

        assert_eq!(
            viewport.reveal_rect(Rect::new(0, 30, 100, 140), 0),
            ScrollOffset::new(0, 30)
        );
    }

    #[test]
    fn reveal_margin_expands_visibility_requirement() {
        let viewport = Viewport::new(
            Rect::new(0, 0, 100, 100),
            Size::new(100, 300),
            ScrollOffset::default(),
        );

        assert_eq!(
            viewport.reveal_rect(Rect::new(0, 90, 100, 10), 8),
            ScrollOffset::new(0, 8)
        );
    }
}
