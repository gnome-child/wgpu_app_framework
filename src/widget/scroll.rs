use crate::geometry::{Rect, area, point, rect};
use crate::{paint, ui};

use super::Frame;

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct Bars {
    vertical: bool,
    horizontal: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Axis {
    Vertical,
    Horizontal,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Part {
    VerticalThumb,
    VerticalTrack,
    HorizontalThumb,
    HorizontalTrack,
    Corner,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Style {
    thickness: f32,
    min_thumb_length: f32,
    track: paint::Brush,
    thumb: paint::Brush,
    thumb_hover_tint: paint::Brush,
    thumb_pressed_tint: paint::Brush,
    corner: paint::Brush,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Metrics {
    frame: Frame,
    offset: point::Logical,
    max_offset: point::Logical,
    vertical_track: Option<Rect>,
    vertical_thumb: Option<Rect>,
    horizontal_track: Option<Rect>,
    horizontal_thumb: Option<Rect>,
    corner: Option<Rect>,
    style: Style,
}

impl Bars {
    pub const fn none() -> Self {
        Self {
            vertical: false,
            horizontal: false,
        }
    }

    pub const fn vertical() -> Self {
        Self {
            vertical: true,
            horizontal: false,
        }
    }

    pub const fn horizontal() -> Self {
        Self {
            vertical: false,
            horizontal: true,
        }
    }

    pub const fn both() -> Self {
        Self {
            vertical: true,
            horizontal: true,
        }
    }

    pub const fn vertical_enabled(self) -> bool {
        self.vertical
    }

    pub const fn horizontal_enabled(self) -> bool {
        self.horizontal
    }

    pub const fn is_enabled(self) -> bool {
        self.vertical || self.horizontal
    }
}

impl Style {
    pub fn new(
        thickness: f32,
        min_thumb_length: f32,
        track: impl Into<paint::Brush>,
        thumb: impl Into<paint::Brush>,
        thumb_hover_tint: impl Into<paint::Brush>,
        thumb_pressed_tint: impl Into<paint::Brush>,
        corner: impl Into<paint::Brush>,
    ) -> Self {
        Self {
            thickness: thickness.max(0.0),
            min_thumb_length: min_thumb_length.max(0.0),
            track: track.into(),
            thumb: thumb.into(),
            thumb_hover_tint: thumb_hover_tint.into(),
            thumb_pressed_tint: thumb_pressed_tint.into(),
            corner: corner.into(),
        }
    }

    pub fn thickness(self) -> f32 {
        self.thickness
    }

    pub fn min_thumb_length(self) -> f32 {
        self.min_thumb_length
    }

    pub fn track(self) -> paint::Brush {
        self.track
    }

    pub fn thumb(self) -> paint::Brush {
        self.thumb
    }

    pub fn thumb_hover_tint(self) -> paint::Brush {
        self.thumb_hover_tint
    }

    pub fn thumb_pressed_tint(self) -> paint::Brush {
        self.thumb_pressed_tint
    }

    pub fn corner(self) -> paint::Brush {
        self.corner
    }
}

impl Default for Style {
    fn default() -> Self {
        let track = paint::Color::rgba(0.0, 0.0, 0.0, 0.18);

        Self::new(
            10.0,
            18.0,
            track,
            paint::Color::rgba(1.0, 1.0, 1.0, 0.26),
            paint::Color::rgba(1.0, 1.0, 1.0, 0.10),
            paint::Color::rgba(1.0, 1.0, 1.0, 0.18),
            track,
        )
    }
}

impl Metrics {
    pub fn frame(self) -> Frame {
        self.frame
    }

    pub fn outer(self) -> Rect {
        self.frame.outer()
    }

    pub fn viewport(self) -> Rect {
        self.frame.viewport()
    }

    pub fn content_size(self) -> area::Logical {
        self.frame.content_size()
    }

    pub fn offset(self) -> point::Logical {
        self.offset
    }

    pub fn max_offset(self) -> point::Logical {
        self.max_offset
    }

    pub fn vertical_track(self) -> Option<Rect> {
        self.vertical_track
    }

    pub fn vertical_thumb(self) -> Option<Rect> {
        self.vertical_thumb
    }

    pub fn horizontal_track(self) -> Option<Rect> {
        self.horizontal_track
    }

    pub fn horizontal_thumb(self) -> Option<Rect> {
        self.horizontal_thumb
    }

    pub fn corner(self) -> Option<Rect> {
        self.corner
    }

    pub fn style(self) -> Style {
        self.style
    }

    pub fn hit_test(self, position: point::Logical) -> Option<Part> {
        if self
            .vertical_thumb
            .is_some_and(|rect| contains(rect, position))
        {
            return Some(Part::VerticalThumb);
        }

        if self
            .horizontal_thumb
            .is_some_and(|rect| contains(rect, position))
        {
            return Some(Part::HorizontalThumb);
        }

        if self.corner.is_some_and(|rect| contains(rect, position)) {
            return Some(Part::Corner);
        }

        if self
            .vertical_track
            .is_some_and(|rect| contains(rect, position))
        {
            return Some(Part::VerticalTrack);
        }

        if self
            .horizontal_track
            .is_some_and(|rect| contains(rect, position))
        {
            return Some(Part::HorizontalTrack);
        }

        None
    }

    pub fn wheel_offset(self, delta: point::Logical) -> point::Logical {
        self.clamp_offset(point::logical(
            self.offset.x() - delta.x(),
            self.offset.y() - delta.y(),
        ))
    }

    pub fn page_offset(self, part: Part, position: point::Logical) -> Option<point::Logical> {
        match part {
            Part::VerticalTrack => {
                let thumb = self.vertical_thumb?;
                let direction = if position.y() < thumb.origin.y() {
                    -1.0
                } else {
                    1.0
                };

                Some(self.clamp_offset(point::logical(
                    self.offset.x(),
                    self.offset.y() + self.viewport().area.height() * direction,
                )))
            }
            Part::HorizontalTrack => {
                let thumb = self.horizontal_thumb?;
                let direction = if position.x() < thumb.origin.x() {
                    -1.0
                } else {
                    1.0
                };

                Some(self.clamp_offset(point::logical(
                    self.offset.x() + self.viewport().area.width() * direction,
                    self.offset.y(),
                )))
            }
            _ => None,
        }
    }

    pub fn drag_offset(
        self,
        part: Part,
        position: point::Logical,
        grab_offset: point::Logical,
    ) -> Option<point::Logical> {
        match part {
            Part::VerticalThumb => {
                let track = self.vertical_track?;
                let thumb = self.vertical_thumb?;
                let travel = (track.area.height() - thumb.area.height()).max(0.0);
                if travel <= 0.0 || self.max_offset.y() <= 0.0 {
                    return None;
                }

                let thumb_y = position.y() - grab_offset.y();
                let ratio = ((thumb_y - track.origin.y()) / travel).clamp(0.0, 1.0);

                Some(
                    self.clamp_offset(point::logical(self.offset.x(), self.max_offset.y() * ratio)),
                )
            }
            Part::HorizontalThumb => {
                let track = self.horizontal_track?;
                let thumb = self.horizontal_thumb?;
                let travel = (track.area.width() - thumb.area.width()).max(0.0);
                if travel <= 0.0 || self.max_offset.x() <= 0.0 {
                    return None;
                }

                let thumb_x = position.x() - grab_offset.x();
                let ratio = ((thumb_x - track.origin.x()) / travel).clamp(0.0, 1.0);

                Some(
                    self.clamp_offset(point::logical(self.max_offset.x() * ratio, self.offset.y())),
                )
            }
            Part::VerticalTrack | Part::HorizontalTrack | Part::Corner => None,
        }
    }

    pub fn clamp_offset(self, offset: point::Logical) -> point::Logical {
        point::logical(
            offset.x().clamp(0.0, self.max_offset.x()),
            offset.y().clamp(0.0, self.max_offset.y()),
        )
    }
}

pub fn metrics(node: &ui::Node, layout: &ui::Frame) -> Option<Metrics> {
    let scroll = node.scroll()?;
    let scrollbars = scroll.bars();
    if !scrollbars.is_enabled() {
        return None;
    }

    let style = scroll.style();
    let viewport = viewport_rect(node, layout.rect());
    let content_size = content_size(node, layout, viewport);
    let max_offset = point::logical(
        (content_size.width() - viewport.area.width()).max(0.0),
        (content_size.height() - viewport.area.height()).max(0.0),
    );
    let offset = scroll.offset();
    let offset = point::logical(
        offset.x().clamp(0.0, max_offset.x()),
        offset.y().clamp(0.0, max_offset.y()),
    );
    let vertical_track = vertical_track(node, layout.rect());
    let horizontal_track = horizontal_track(node, layout.rect());
    let vertical_thumb = vertical_track.map(|track| {
        thumb_rect(
            track,
            viewport.area.height(),
            content_size.height(),
            offset.y(),
            max_offset.y(),
            Axis::Vertical,
            style.min_thumb_length(),
        )
    });
    let horizontal_thumb = horizontal_track.map(|track| {
        thumb_rect(
            track,
            viewport.area.width(),
            content_size.width(),
            offset.x(),
            max_offset.x(),
            Axis::Horizontal,
            style.min_thumb_length(),
        )
    });
    let corner = if scrollbars.vertical_enabled() && scrollbars.horizontal_enabled() {
        let padding = node.style().padding();
        let thickness = style.thickness();
        Some(Rect::new(
            point::logical(
                layout.rect().origin.x() + layout.rect().area.width() - padding.right - thickness,
                layout.rect().origin.y() + layout.rect().area.height() - padding.bottom - thickness,
            ),
            area::logical(thickness, thickness),
        ))
    } else {
        None
    };

    Some(Metrics {
        frame: Frame::new(layout.rect(), viewport, content_size),
        offset,
        max_offset,
        vertical_track,
        vertical_thumb,
        horizontal_track,
        horizontal_thumb,
        corner,
        style,
    })
}

pub fn paint_chrome(
    node: &ui::Node,
    layout: &ui::Frame,
    interaction: &ui::Interaction,
    scene: &mut paint::Scene,
) {
    let Some(metrics) = metrics(node, layout) else {
        return;
    };

    let style = metrics.style();

    if let Some(track) = metrics.vertical_track() {
        push_chrome(scene, track, style.track());
    }

    if let Some(track) = metrics.horizontal_track() {
        push_chrome(scene, track, style.track());
    }

    if let Some(corner) = metrics.corner() {
        push_chrome(scene, corner, style.corner());
    }

    if let Some(thumb) = metrics.vertical_thumb() {
        push_chrome(scene, thumb, style.thumb());
        push_thumb_tint(
            scene,
            metrics,
            layout.path(),
            thumb,
            Part::VerticalThumb,
            interaction,
        );
    }

    if let Some(thumb) = metrics.horizontal_thumb() {
        push_chrome(scene, thumb, style.thumb());
        push_thumb_tint(
            scene,
            metrics,
            layout.path(),
            thumb,
            Part::HorizontalThumb,
            interaction,
        );
    }
}

pub fn viewport_rect(node: &ui::Node, rect: Rect) -> Rect {
    let padding = node.style().padding();
    let scroll = node.scroll();
    let scrollbars = scroll.map_or_else(Bars::none, |scroll| scroll.bars());
    let style = scroll.map_or_else(Style::default, |scroll| scroll.style());
    let vertical_gutter = if scrollbars.vertical_enabled() {
        style.thickness()
    } else {
        0.0
    };
    let horizontal_gutter = if scrollbars.horizontal_enabled() {
        style.thickness()
    } else {
        0.0
    };

    Rect::rounded(
        point::logical(
            rect.origin.x() + padding.left,
            rect.origin.y() + padding.top,
        ),
        area::logical(
            (rect.area.width() - padding.horizontal() - vertical_gutter).max(0.0),
            (rect.area.height() - padding.vertical() - horizontal_gutter).max(0.0),
        ),
        node.style().rounding(),
    )
}

fn push_chrome(scene: &mut paint::Scene, rect: Rect, brush: paint::Brush) {
    scene.push_quad(paint::Quad {
        rect,
        rasterization: paint::Rasterization::default(),
        style: paint::Style {
            fill: Some(paint::Fill::Brush(brush)),
            stroke: None,
            tint: None,
        },
    });
}

fn push_thumb_tint(
    scene: &mut paint::Scene,
    metrics: Metrics,
    path: &ui::Path,
    thumb: Rect,
    part: Part,
    interaction: &ui::Interaction,
) {
    let captured = interaction.pointer_capture().is_some_and(|capture| {
        capture.target() == path && capture.part() == super::Part::Scroll(part)
    });
    let hovered = interaction
        .pointer_position()
        .is_some_and(|position| metrics.hit_test(position) == Some(part));

    let brush = if captured {
        Some(metrics.style().thumb_pressed_tint())
    } else if hovered {
        Some(metrics.style().thumb_hover_tint())
    } else {
        None
    };

    if let Some(brush) = brush {
        scene.push_tint(paint::Tint { rect: thumb, brush });
    }
}

fn content_size(node: &ui::Node, layout: &ui::Frame, viewport: Rect) -> area::Logical {
    let offset = node
        .scroll()
        .map_or_else(|| point::logical(0.0, 0.0), |scroll| scroll.offset());
    let mut width = viewport.area.width();
    let mut height = viewport.area.height();

    extend_content_size(layout.children(), viewport, offset, &mut width, &mut height);

    area::logical(width.max(0.0), height.max(0.0))
}

fn extend_content_size(
    children: &[ui::Frame],
    viewport: Rect,
    offset: point::Logical,
    width: &mut f32,
    height: &mut f32,
) {
    for child in children {
        *width = width.max(
            child.rect().origin.x() + child.rect().area.width() - viewport.origin.x() + offset.x(),
        );
        *height = height.max(
            child.rect().origin.y() + child.rect().area.height() - viewport.origin.y() + offset.y(),
        );

        extend_content_size(child.children(), viewport, offset, width, height);
    }
}

fn vertical_track(node: &ui::Node, rect: Rect) -> Option<Rect> {
    let scroll = node.scroll()?;
    if !scroll.bars().vertical_enabled() {
        return None;
    }

    let padding = node.style().padding();
    let style = scroll.style();
    let horizontal_gutter = if scroll.bars().horizontal_enabled() {
        style.thickness()
    } else {
        0.0
    };

    Some(Rect::new(
        point::logical(
            rect.origin.x() + rect.area.width() - padding.right - style.thickness(),
            rect.origin.y() + padding.top,
        ),
        area::logical(
            style.thickness(),
            (rect.area.height() - padding.vertical() - horizontal_gutter).max(0.0),
        ),
    ))
}

fn horizontal_track(node: &ui::Node, rect: Rect) -> Option<Rect> {
    let scroll = node.scroll()?;
    if !scroll.bars().horizontal_enabled() {
        return None;
    }

    let padding = node.style().padding();
    let style = scroll.style();
    let vertical_gutter = if scroll.bars().vertical_enabled() {
        style.thickness()
    } else {
        0.0
    };

    Some(Rect::new(
        point::logical(
            rect.origin.x() + padding.left,
            rect.origin.y() + rect.area.height() - padding.bottom - style.thickness(),
        ),
        area::logical(
            (rect.area.width() - padding.horizontal() - vertical_gutter).max(0.0),
            style.thickness(),
        ),
    ))
}

fn thumb_rect(
    track: Rect,
    viewport_length: f32,
    content_length: f32,
    offset: f32,
    max_offset: f32,
    axis: Axis,
    min_thumb_length: f32,
) -> Rect {
    let track_length = match axis {
        Axis::Vertical => track.area.height(),
        Axis::Horizontal => track.area.width(),
    };
    let thumb_length = if content_length <= viewport_length || content_length <= 0.0 {
        track_length
    } else {
        (track_length * (viewport_length / content_length))
            .max(min_thumb_length)
            .min(track_length)
    };
    let travel = (track_length - thumb_length).max(0.0);
    let position = if max_offset > 0.0 {
        travel * (offset / max_offset).clamp(0.0, 1.0)
    } else {
        0.0
    };

    match axis {
        Axis::Vertical => Rect::rounded(
            point::logical(track.origin.x(), track.origin.y() + position),
            area::logical(track.area.width(), thumb_length),
            rect::Rounding::relative(1.0),
        ),
        Axis::Horizontal => Rect::rounded(
            point::logical(track.origin.x() + position, track.origin.y()),
            area::logical(thumb_length, track.area.height()),
            rect::Rounding::relative(1.0),
        ),
    }
}

fn contains(rect: Rect, position: point::Logical) -> bool {
    let x = position.x();
    let y = position.y();
    let left = rect.origin.x();
    let top = rect.origin.y();
    let right = left + rect.area.width();
    let bottom = top + rect.area.height();

    x >= left && x < right && y >= top && y < bottom
}
