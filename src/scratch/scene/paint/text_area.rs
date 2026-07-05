use crate::scratch::{geometry, layout, theme::Theme};
use crate::text;

use super::super::primitive::TextColor;
use super::super::{Quad, Scene, TextSurface, TextViewport};

pub(super) fn paint(
    rect: geometry::Rect,
    text_area: &layout::text::Area,
    scene: &mut Scene,
    theme: &Theme,
) {
    for span in text_area.layout().selection_spans() {
        if let Some(span) = clip_rect(
            span_rect(rect, span.x(), span.y(), span.width(), span.height()),
            rect,
        ) {
            scene.push_quad(Quad::new(span, theme.palette().selection));
        }
    }

    scene.push_text_viewport(TextViewport::new(
        rect,
        text_area
            .render_surfaces()
            .iter()
            .map(|surface| {
                TextSurface::new(
                    geometry::Rect::new(
                        rect.x().saturating_add(surface.x().round() as i32),
                        rect.y().saturating_add(surface.y().round() as i32),
                        surface.width().ceil().max(0.0) as i32,
                        surface.height().ceil().max(0.0) as i32,
                    ),
                    surface.buffer(),
                    into_scene_text_color(surface.default_color()),
                )
            })
            .collect(),
    ));

    if let Some(caret) = text_area.layout().caret()
        && let Some(caret) = clip_rect(
            span_rect(rect, caret.x(), caret.y(), 1.0, caret.height()),
            rect,
        )
    {
        scene.push_quad(Quad::new(caret, theme.palette().text_inverse));
    }
}

fn into_scene_text_color(color: text::Color) -> TextColor {
    let (r, g, b, a) = color.channels();
    TextColor::rgba(r, g, b, a)
}

fn span_rect(rect: geometry::Rect, x: f32, y: f32, width: f32, height: f32) -> geometry::Rect {
    geometry::Rect::new(
        rect.x().saturating_add(x.floor() as i32),
        rect.y().saturating_add(y.floor() as i32),
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
