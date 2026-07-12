use super::super::{geometry::Rect, interaction, theme};
use super::{Frame, Viewport};

#[derive(Debug, Clone, PartialEq)]
pub(crate) struct Chrome {
    target: interaction::Target,
    scroll_target: interaction::Target,
    kind: Kind,
}

#[derive(Debug, Clone, PartialEq)]
pub(crate) enum Kind {
    Scrollbar(Scrollbar),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Axis {
    Horizontal,
    Vertical,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) struct Scrollbar {
    axis: Axis,
    track: Rect,
    thumb: Rect,
    viewport: Viewport,
}

pub(crate) fn project(frames: &[Frame], theme: &theme::Theme) -> Vec<Chrome> {
    frames
        .iter()
        .flat_map(|frame| scrollbars_for_frame(frame, theme))
        .collect()
}

impl Chrome {
    pub(crate) fn target(&self) -> &interaction::Target {
        &self.target
    }

    pub(crate) fn scroll_target(&self) -> &interaction::Target {
        &self.scroll_target
    }

    pub(crate) fn kind(&self) -> &Kind {
        &self.kind
    }

    pub(crate) fn accepts_hit(&self, point: super::super::geometry::Point) -> bool {
        match &self.kind {
            Kind::Scrollbar(scrollbar) => scrollbar.track.contains(point),
        }
    }

    pub(crate) fn scroll_offset_at(
        &self,
        point: super::super::geometry::Point,
    ) -> interaction::ScrollOffset {
        match self.kind {
            Kind::Scrollbar(scrollbar) => scrollbar.scroll_offset_at(point),
        }
    }
}

impl Scrollbar {
    #[cfg(test)]
    pub(crate) fn track(self) -> Rect {
        self.track
    }

    pub(crate) fn viewport(self) -> Viewport {
        self.viewport
    }

    pub(crate) fn track_with_thickness(self, thickness: i32) -> Rect {
        resize_cross_axis(self.track, self.axis, thickness.max(1))
    }

    pub(crate) fn thumb_with_thickness(self, thickness: i32) -> Rect {
        resize_cross_axis(self.thumb, self.axis, thickness.max(1))
    }

    fn scroll_offset_at(self, point: super::super::geometry::Point) -> interaction::ScrollOffset {
        let max = self.viewport.max_scroll();
        match self.axis {
            Axis::Vertical => interaction::ScrollOffset::new(
                self.viewport.resolved_scroll().x(),
                axis_offset(
                    point.y(),
                    self.track.y(),
                    self.track.height(),
                    self.thumb.height(),
                    max.y(),
                ),
            ),
            Axis::Horizontal => interaction::ScrollOffset::new(
                axis_offset(
                    point.x(),
                    self.track.x(),
                    self.track.width(),
                    self.thumb.width(),
                    max.x(),
                ),
                self.viewport.resolved_scroll().y(),
            ),
        }
    }
}

fn scrollbars_for_frame(frame: &Frame, theme: &theme::Theme) -> Vec<Chrome> {
    let Some(viewport) = frame.viewport() else {
        return Vec::new();
    };
    let Some(scroll_target) = frame.target().cloned() else {
        return Vec::new();
    };
    if !viewport.is_scrollable() {
        return Vec::new();
    }

    let mut scrollbars = Vec::new();
    if viewport.max_scroll().y() > 0
        && let Some(scrollbar) = scrollbar_for_axis(viewport, theme, Axis::Vertical)
    {
        scrollbars.push(Chrome {
            target: scrollbar_target(frame, Axis::Vertical),
            scroll_target: scroll_target.clone(),
            kind: Kind::Scrollbar(scrollbar),
        });
    }
    if viewport.max_scroll().x() > 0
        && let Some(scrollbar) = scrollbar_for_axis(viewport, theme, Axis::Horizontal)
    {
        scrollbars.push(Chrome {
            target: scrollbar_target(frame, Axis::Horizontal),
            scroll_target,
            kind: Kind::Scrollbar(scrollbar),
        });
    }

    scrollbars
}

fn scrollbar_target(frame: &Frame, axis: Axis) -> interaction::Target {
    let label = match axis {
        Axis::Horizontal => "Horizontal Scrollbar",
        Axis::Vertical => "Vertical Scrollbar",
    };
    interaction::Target::scrollbar_node(
        frame.node_id(),
        frame.target().and_then(interaction::Target::element_id),
        label,
    )
}

fn scrollbar_for_axis(viewport: Viewport, theme: &theme::Theme, axis: Axis) -> Option<Scrollbar> {
    let scrollbar = theme.scrollbar();
    let thickness = match scrollbar.metrics.policy {
        theme::ScrollbarPolicy::GutterAlways => scrollbar.metrics.thickness,
        theme::ScrollbarPolicy::OverlayAuto => scrollbar.appearance.overlay_thickness,
    }
    .max(1);
    let margin = scrollbar.appearance.margin.max(0);
    let bounds = scrollbar_bounds(viewport);
    let track = match axis {
        Axis::Vertical => Rect::new(
            bounds
                .right()
                .saturating_sub(margin)
                .saturating_sub(thickness),
            bounds.y().saturating_add(margin),
            thickness,
            bounds.height().saturating_sub(margin.saturating_mul(2)),
        ),
        Axis::Horizontal => Rect::new(
            bounds.x().saturating_add(margin),
            bounds
                .bottom()
                .saturating_sub(margin)
                .saturating_sub(thickness),
            bounds.width().saturating_sub(margin.saturating_mul(2)),
            thickness,
        ),
    };
    let thumb = thumb_rect(
        axis,
        track,
        viewport_extent(viewport, axis),
        content_extent(viewport, axis),
        resolved_offset(viewport, axis),
        max_offset(viewport, axis),
        scrollbar.appearance.min_thumb_length,
    )?;

    Some(Scrollbar {
        axis,
        track,
        thumb,
        viewport,
    })
}

fn scrollbar_bounds(viewport: Viewport) -> Rect {
    viewport.visible_frame()
}

fn thumb_rect(
    axis: Axis,
    track: Rect,
    viewport_extent: i32,
    content_extent: i32,
    offset: i32,
    max_offset: i32,
    min_thumb_length: i32,
) -> Option<Rect> {
    if track.width() <= 0 || track.height() <= 0 || content_extent <= viewport_extent {
        return None;
    }

    let track_extent = match axis {
        Axis::Vertical => track.height(),
        Axis::Horizontal => track.width(),
    };
    let thumb_extent = ((track_extent as f32 * viewport_extent.max(1) as f32
        / content_extent.max(1) as f32)
        .round() as i32)
        .max(min_thumb_length.max(1))
        .min(track_extent);
    let travel = track_extent.saturating_sub(thumb_extent);
    let thumb_offset = if max_offset <= 0 {
        0
    } else {
        ((travel as f32 * offset.clamp(0, max_offset) as f32 / max_offset as f32).round()) as i32
    };

    Some(match axis {
        Axis::Vertical => Rect::new(
            track.x(),
            track.y().saturating_add(thumb_offset),
            track.width(),
            thumb_extent,
        ),
        Axis::Horizontal => Rect::new(
            track.x().saturating_add(thumb_offset),
            track.y(),
            thumb_extent,
            track.height(),
        ),
    })
}

fn resize_cross_axis(rect: Rect, axis: Axis, thickness: i32) -> Rect {
    match axis {
        Axis::Vertical => Rect::new(
            rect.right().saturating_sub(thickness),
            rect.y(),
            thickness,
            rect.height(),
        ),
        Axis::Horizontal => Rect::new(
            rect.x(),
            rect.bottom().saturating_sub(thickness),
            rect.width(),
            thickness,
        ),
    }
}

fn axis_offset(point: i32, origin: i32, track_extent: i32, thumb_extent: i32, max: i32) -> i32 {
    let travel = track_extent.saturating_sub(thumb_extent);
    if travel <= 0 || max <= 0 {
        return 0;
    }

    let local = point
        .saturating_sub(origin)
        .saturating_sub(thumb_extent / 2)
        .clamp(0, travel);
    ((local as f32 / travel as f32) * max as f32).round() as i32
}

fn viewport_extent(viewport: Viewport, axis: Axis) -> i32 {
    match axis {
        Axis::Vertical => viewport.visible_content().height(),
        Axis::Horizontal => viewport.visible_content().width(),
    }
}

fn content_extent(viewport: Viewport, axis: Axis) -> i32 {
    match axis {
        Axis::Vertical => viewport.content().height(),
        Axis::Horizontal => viewport.content().width(),
    }
}

fn resolved_offset(viewport: Viewport, axis: Axis) -> i32 {
    match axis {
        Axis::Vertical => viewport.resolved_scroll().y(),
        Axis::Horizontal => viewport.resolved_scroll().x(),
    }
}

fn max_offset(viewport: Viewport, axis: Axis) -> i32 {
    match axis {
        Axis::Vertical => viewport.max_scroll().y(),
        Axis::Horizontal => viewport.max_scroll().x(),
    }
}
