use crate::{geometry, text, theme::Theme};

use super::super::primitive::TextColor;
use super::super::{Rule, TextSurface};

fn surface_rect(
    viewport: geometry::Rect,
    surface: &text::layout::TextAreaSurface,
) -> geometry::Rect {
    geometry::Rect::new(
        viewport.x().saturating_add(surface.x().round() as i32),
        viewport.y().saturating_add(surface.y().round() as i32),
        surface.width().ceil().max(0.0) as i32,
        surface.height().ceil().max(0.0) as i32,
    )
}

pub(super) fn surface(
    viewport: geometry::Rect,
    surface: &text::layout::TextAreaSurface,
) -> TextSurface {
    TextSurface::new(
        surface_rect(viewport, surface),
        surface.shaped_buffer(),
        text_color(surface.default_color()),
    )
}

pub(super) fn clipped_span_rect(
    viewport: geometry::Rect,
    x: f32,
    y: f32,
    width: f32,
    height: f32,
) -> Option<geometry::Rect> {
    clip_rect(span_rect(viewport, x, y, width, height), viewport)
}

pub(super) fn caret_rule(rect: geometry::Rect, theme: &Theme) -> Rule {
    Rule::vertical(rect, theme.text_input().caret, 2)
}

fn span_rect(viewport: geometry::Rect, x: f32, y: f32, width: f32, height: f32) -> geometry::Rect {
    geometry::Rect::new(
        viewport.x().saturating_add(x.floor() as i32),
        viewport.y().saturating_add(y.floor() as i32),
        width.ceil().max(0.0) as i32,
        height.ceil().max(0.0) as i32,
    )
}

fn clip_rect(rect: geometry::Rect, bounds: geometry::Rect) -> Option<geometry::Rect> {
    let left = rect.x().max(bounds.x());
    let top = rect.y().max(bounds.y());
    let right = rect
        .x()
        .saturating_add(rect.width())
        .min(bounds.x().saturating_add(bounds.width()));
    let bottom = rect
        .y()
        .saturating_add(rect.height())
        .min(bounds.y().saturating_add(bounds.height()));

    (right > left && bottom > top)
        .then(|| geometry::Rect::new(left, top, right - left, bottom - top))
}

fn text_color(color: text::Color) -> TextColor {
    let (r, g, b, a) = color.channels();
    TextColor::rgba(r, g, b, a)
}
