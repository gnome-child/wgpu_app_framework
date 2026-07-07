use crate::scratch::{geometry, layout, theme::Theme};
use crate::text;

use super::super::primitive::TextColor;
use super::super::{
    EdgeMode, Quad, Rasterization, Scene, Snapping, TextSurface, TextViewport, Visuals,
};

pub(super) fn paint_text(frame: &layout::frame::Frame, scene: &mut Scene) -> bool {
    let Some(text_box) = frame.text_box() else {
        return false;
    };
    if text_box.text().is_empty() {
        return false;
    }

    let Some(field) = frame.text_box_layout() else {
        return false;
    };
    let Some(surface) = field.render_surface() else {
        return false;
    };
    let rect = frame.text_box_text_rect();
    let surface_rect = geometry::Rect::new(
        rect.x().saturating_add(surface.x().round() as i32),
        rect.y().saturating_add(surface.y().round() as i32),
        surface.width().ceil().max(0.0) as i32,
        surface.height().ceil().max(0.0) as i32,
    );

    scene.push_text_viewport(TextViewport::new(
        rect,
        vec![TextSurface::new(
            surface_rect,
            surface.buffer(),
            into_scene_text_color(surface.default_color()),
        )],
    ));

    true
}

pub(super) fn paint_selection(frame: &layout::frame::Frame, scene: &mut Scene, theme: &Theme) {
    let Some(field) = frame.text_box_layout() else {
        return;
    };
    let rect = frame.text_box_text_rect();

    for span in field.layout().selection_spans() {
        if let Some(span) = clip_rect(
            span_rect(rect, span.x(), span.y(), span.width(), span.height()),
            rect,
        ) {
            scene.push_quad(Quad::new(span, theme.text().selection));
        }
    }
}

pub(super) fn paint_caret(
    frame: &layout::frame::Frame,
    scene: &mut Scene,
    theme: &Theme,
    visuals: &Visuals,
) {
    if !frame.is_focused() {
        return;
    }
    if frame
        .target()
        .is_some_and(|target| !visuals.caret_visible(target))
    {
        return;
    }

    let Some(field) = frame.text_box_layout() else {
        return;
    };
    let rect = frame.text_box_text_rect();

    if let Some(caret) = field.layout().caret()
        && let Some(caret) = clip_rect(
            span_rect(rect, caret.x(), caret.y(), 1.0, caret.height()),
            rect,
        )
    {
        scene.push_quad(caret_quad(caret, theme));
    }
}

fn caret_quad(rect: geometry::Rect, theme: &Theme) -> Quad {
    Quad::new(rect, theme.text_input().caret).with_rasterization(Rasterization::new(
        Snapping::FixedWidth { width_px: 2 },
        EdgeMode::Hard,
    ))
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

fn into_scene_text_color(color: text::Color) -> TextColor {
    let (r, g, b, a) = color.channels();
    TextColor::rgba(r, g, b, a)
}
