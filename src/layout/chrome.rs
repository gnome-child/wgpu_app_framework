use super::super::{composition, geometry::Rect, interaction, theme};
use super::{Frame, Viewport, frame::Clip};

use interaction::ScrollbarAxis as Axis;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) struct Axes {
    horizontal: bool,
    vertical: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct ContainerLayout {
    axes: Axes,
    presentation: crate::view::ScrollChromePresentation,
    direction: crate::view::ScrollDirection,
    introduction_passes: u8,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) struct ViewportGeometry {
    viewport: Rect,
    visible_frame: Rect,
    visible_content: Rect,
}

impl Axes {
    pub(super) const NONE: Self = Self {
        horizontal: false,
        vertical: false,
    };
    pub(super) const HORIZONTAL: Self = Self {
        horizontal: true,
        vertical: false,
    };
    pub(super) const VERTICAL: Self = Self {
        horizontal: false,
        vertical: true,
    };
    pub(super) const BOTH: Self = Self {
        horizontal: true,
        vertical: true,
    };

    pub(super) const fn new(horizontal: bool, vertical: bool) -> Self {
        Self {
            horizontal,
            vertical,
        }
    }

    pub(super) const fn horizontal(self) -> bool {
        self.horizontal
    }

    pub(super) const fn vertical(self) -> bool {
        self.vertical
    }
}

impl ContainerLayout {
    pub(super) const fn new(
        axes: Axes,
        presentation: crate::view::ScrollChromePresentation,
        direction: crate::view::ScrollDirection,
        introduction_passes: u8,
    ) -> Self {
        Self {
            axes,
            presentation,
            direction,
            introduction_passes,
        }
    }

    pub(super) const fn axes(self) -> Axes {
        self.axes
    }

    pub(crate) const fn presentation(self) -> crate::view::ScrollChromePresentation {
        self.presentation
    }

    pub(super) const fn direction(self) -> crate::view::ScrollDirection {
        self.direction
    }

    #[cfg(test)]
    pub(crate) const fn introduction_passes(self) -> u8 {
        self.introduction_passes
    }
}

impl ViewportGeometry {
    pub(super) fn viewport(self) -> Rect {
        self.viewport
    }

    pub(super) fn visible_frame(self) -> Rect {
        self.visible_frame
    }

    pub(super) fn visible_content(self) -> Rect {
        self.visible_content
    }
}

pub(super) fn viewport_geometry(
    rect: Rect,
    inherited: Option<Clip>,
    theme: &theme::Theme,
    axes: Axes,
) -> ViewportGeometry {
    let visible_frame = intersect_rect(inherited.map(Clip::rect), rect);
    ViewportGeometry {
        viewport: reserve_gutters(rect, theme, axes),
        visible_frame,
        visible_content: reserve_gutters(visible_frame, theme, axes),
    }
}

pub(super) fn container_geometry(
    rect: Rect,
    inherited: Option<Clip>,
    theme: &theme::Theme,
    container: ContainerLayout,
) -> ViewportGeometry {
    let visible_frame = intersect_rect(inherited.map(Clip::rect), rect);
    ViewportGeometry {
        viewport: reserve_container_gutters(rect, theme, container),
        visible_frame,
        visible_content: reserve_container_gutters(visible_frame, theme, container),
    }
}

#[derive(Debug, Clone, PartialEq)]
pub(crate) struct Chrome {
    owner: composition::tree::NodeId,
    target: interaction::Target,
    scroll_target: interaction::Target,
    scope: ViewportScope,
    scrollbar: Scrollbar,
    presentation: crate::view::ScrollChromePresentation,
}

#[derive(Debug, Clone, PartialEq)]
struct ViewportScope {
    clips: Vec<Clip>,
}

#[derive(Debug, Clone, Copy, PartialEq)]
struct Scrollbar {
    axis: Axis,
    track: Rect,
    interaction_track: Rect,
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
    pub(crate) fn owner(&self) -> composition::tree::NodeId {
        self.owner
    }

    pub(crate) fn target(&self) -> &interaction::Target {
        &self.target
    }

    pub(crate) fn scroll_target(&self) -> &interaction::Target {
        &self.scroll_target
    }

    pub(crate) fn clips(&self) -> &[Clip] {
        &self.scope.clips
    }

    pub(crate) fn accepts_hit(&self, point: super::super::geometry::Point) -> bool {
        self.scope.contains(point) && self.scrollbar.interaction_track.contains(point)
    }

    pub(crate) fn scroll_offset_at(
        &self,
        point: super::super::geometry::Point,
    ) -> interaction::ScrollOffset {
        self.scrollbar.scroll_offset_at(point)
    }

    pub(crate) fn resolved_scroll(&self) -> interaction::ScrollOffset {
        self.scrollbar.viewport.resolved_scroll()
    }

    pub(crate) fn axis(&self) -> Axis {
        self.scrollbar.axis
    }

    pub(crate) fn presentation(&self) -> crate::view::ScrollChromePresentation {
        self.presentation
    }

    pub(crate) fn maximum_offset(&self) -> i32 {
        match self.scrollbar.axis {
            Axis::Vertical => self.scrollbar.viewport.max_scroll().y(),
            Axis::Horizontal => self.scrollbar.viewport.max_scroll().x(),
        }
    }

    pub(crate) fn track_with_thickness(&self, thickness: i32) -> Rect {
        self.scrollbar.track_with_thickness(thickness)
    }

    pub(crate) fn thumb_with_thickness(&self, thickness: i32) -> Rect {
        self.scrollbar.thumb_with_thickness(thickness)
    }

    #[cfg(test)]
    pub(crate) fn viewport(&self) -> Viewport {
        self.scrollbar.viewport
    }

    #[cfg(test)]
    pub(crate) fn track(&self) -> Rect {
        self.scrollbar.track
    }

    #[cfg(test)]
    pub(crate) fn interaction_track(&self) -> Rect {
        self.scrollbar.interaction_track
    }
}

impl Scrollbar {
    fn track_with_thickness(self, thickness: i32) -> Rect {
        resize_cross_axis(self.track, self.axis, thickness.max(1))
    }

    fn thumb_with_thickness(self, thickness: i32) -> Rect {
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
    let container = frame.scroll_container_layout();
    if !viewport.is_scrollable()
        && container.is_none_or(|container| {
            let axes = container.axes();
            !axes.horizontal() && !axes.vertical()
        })
    {
        return Vec::new();
    }
    let scope = ViewportScope::new(frame, viewport);
    let presentation = container.map_or_else(
        || match theme.scrollbar().metrics.policy {
            theme::ScrollbarPolicy::OverlayAuto => crate::view::ScrollChromePresentation::Overlay,
            theme::ScrollbarPolicy::GutterAlways => {
                crate::view::ScrollChromePresentation::Consuming
            }
        },
        ContainerLayout::presentation,
    );
    let direction = container.map_or(
        crate::view::ScrollDirection::LeftToRight,
        ContainerLayout::direction,
    );
    let horizontal = container.map_or_else(
        || viewport.max_scroll().x() > 0,
        |container| container.axes().horizontal(),
    );
    let vertical = container.map_or_else(
        || viewport.max_scroll().y() > 0,
        |container| container.axes().vertical(),
    );

    let mut scrollbars = Vec::new();
    if vertical
        && let Some(scrollbar) =
            scrollbar_for_axis(viewport, theme, Axis::Vertical, presentation, direction)
    {
        scrollbars.push(Chrome {
            owner: frame.node_id(),
            target: scrollbar_target(frame, Axis::Vertical),
            scroll_target: scroll_target.clone(),
            scope: scope.clone(),
            scrollbar,
            presentation,
        });
    }
    if horizontal
        && let Some(scrollbar) =
            scrollbar_for_axis(viewport, theme, Axis::Horizontal, presentation, direction)
    {
        scrollbars.push(Chrome {
            owner: frame.node_id(),
            target: scrollbar_target(frame, Axis::Horizontal),
            scroll_target,
            scope,
            scrollbar,
            presentation,
        });
    }

    scrollbars
}

impl ViewportScope {
    fn new(frame: &Frame, viewport: Viewport) -> Self {
        let mut clips = frame.clip().into_iter().collect::<Vec<_>>();
        let viewport = Clip::new(viewport.visible_frame());
        if !clips.contains(&viewport) {
            clips.push(viewport);
        }
        Self { clips }
    }

    fn contains(&self, point: super::super::geometry::Point) -> bool {
        self.clips.iter().all(|clip| clip.contains(point))
    }
}

fn scrollbar_target(frame: &Frame, axis: Axis) -> interaction::Target {
    let label = match axis {
        Axis::Horizontal => "Horizontal Scrollbar",
        Axis::Vertical => "Vertical Scrollbar",
    };
    interaction::Target::scrollbar_node(
        frame.node_id(),
        frame.target().and_then(interaction::Target::element_id),
        axis,
        label,
    )
}

fn scrollbar_for_axis(
    viewport: Viewport,
    theme: &theme::Theme,
    axis: Axis,
    presentation: crate::view::ScrollChromePresentation,
    direction: crate::view::ScrollDirection,
) -> Option<Scrollbar> {
    let scrollbar = theme.scrollbar();
    let thickness = match presentation {
        crate::view::ScrollChromePresentation::Consuming => scrollbar.metrics.thickness,
        crate::view::ScrollChromePresentation::Overlay => scrollbar.appearance.overlay_thickness,
    }
    .max(1);
    let margin = scrollbar.appearance.margin.max(0);
    let bounds = scrollbar_bounds(viewport);
    let track = match axis {
        Axis::Vertical => Rect::new(
            match direction {
                crate::view::ScrollDirection::LeftToRight => bounds
                    .right()
                    .saturating_sub(margin)
                    .saturating_sub(thickness),
                crate::view::ScrollDirection::RightToLeft => bounds.x().saturating_add(margin),
            },
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
        interaction_track: resize_cross_axis(
            track,
            axis,
            scrollbar.appearance.hover_thickness.max(thickness).max(1),
        ),
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
    if track.width() <= 0 || track.height() <= 0 {
        return None;
    }
    if content_extent <= viewport_extent || max_offset <= 0 {
        return Some(track);
    }

    let track_extent = match axis {
        Axis::Vertical => track.height(),
        Axis::Horizontal => track.width(),
    };
    let thumb_extent =
        rounded_product_ratio(track_extent, viewport_extent.max(1), content_extent.max(1))
            .max(min_thumb_length.max(1))
            .min(track_extent);
    let travel = track_extent.saturating_sub(thumb_extent);
    let thumb_offset = if max_offset <= 0 {
        0
    } else {
        rounded_product_ratio(travel, offset.clamp(0, max_offset), max_offset)
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
    rounded_product_ratio(local, max, travel)
}

fn rounded_product_ratio(left: i32, right: i32, denominator: i32) -> i32 {
    if left <= 0 || right <= 0 || denominator <= 0 {
        return 0;
    }

    let denominator = i128::from(denominator);
    let rounded = (i128::from(left) * i128::from(right) + denominator / 2) / denominator;
    i32::try_from(rounded).unwrap_or(i32::MAX)
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

fn reserve_gutters(rect: Rect, theme: &theme::Theme, axes: Axes) -> Rect {
    let metrics = theme.scrollbar().metrics;
    if metrics.policy != theme::ScrollbarPolicy::GutterAlways {
        return rect;
    }

    let gutter = metrics.thickness.max(1);
    Rect::new(
        rect.x(),
        rect.y(),
        rect.width()
            .saturating_sub(if axes.vertical { gutter } else { 0 }),
        rect.height()
            .saturating_sub(if axes.horizontal { gutter } else { 0 }),
    )
}

fn reserve_container_gutters(rect: Rect, theme: &theme::Theme, container: ContainerLayout) -> Rect {
    if container.presentation() != crate::view::ScrollChromePresentation::Consuming {
        return rect;
    }

    let axes = container.axes();
    let gutter = theme.scrollbar().metrics.thickness.max(1);
    let vertical = if axes.vertical() { gutter } else { 0 };
    let horizontal = if axes.horizontal() { gutter } else { 0 };
    let x = match container.direction() {
        crate::view::ScrollDirection::LeftToRight => rect.x(),
        crate::view::ScrollDirection::RightToLeft => rect.x().saturating_add(vertical),
    };
    Rect::new(
        x,
        rect.y(),
        rect.width().saturating_sub(vertical),
        rect.height().saturating_sub(horizontal),
    )
}

fn intersect_rect(inherited: Option<Rect>, rect: Rect) -> Rect {
    let Some(inherited) = inherited else {
        return rect;
    };
    let x = inherited.x().max(rect.x());
    let y = inherited.y().max(rect.y());
    let right = inherited.right().min(rect.right());
    let bottom = inherited.bottom().min(rect.bottom());
    Rect::new(x, y, right.saturating_sub(x), bottom.saturating_sub(y))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn scrollbar_drag_preserves_integral_truth_past_f32_precision() {
        for (maximum, travel, local, expected) in [
            (16_777_215, 97, 49, 8_475_088),
            (16_777_216, 127, 22, 2_906_289),
            (16_777_217, 97, 49, 8_475_089),
            (23_999_897, 97, 9, 2_226_795),
            (23_999_898, 97, 4, 989_687),
            (23_999_899, 97, 21, 5_195_854),
        ] {
            let thumb_extent = 20;
            let track_extent = travel + thumb_extent;
            let point = 11 + thumb_extent / 2 + local;
            assert_eq!(
                axis_offset(point, 11, track_extent, thumb_extent, maximum),
                expected,
                "maximum={maximum} travel={travel} local={local}"
            );
        }
    }
}
