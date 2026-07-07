use crate::{layout, theme::Theme};

use super::super::{Quad, Scene, TextViewport, Visuals};
use super::text_surface;

pub(super) fn paint_text(frame: &layout::Frame, scene: &mut Scene) -> bool {
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

    scene.push_text_viewport(TextViewport::new(
        rect,
        vec![text_surface::surface(rect, surface)],
    ));

    true
}

pub(super) fn paint_selection(frame: &layout::Frame, scene: &mut Scene, theme: &Theme) {
    let Some(field) = frame.text_box_layout() else {
        return;
    };
    let rect = frame.text_box_text_rect();

    for span in field.layout().selection_spans() {
        if let Some(span) =
            text_surface::clipped_span_rect(rect, span.x(), span.y(), span.width(), span.height())
        {
            scene.push_quad(Quad::new(span, theme.text().selection));
        }
    }
}

pub(super) fn paint_caret(
    frame: &layout::Frame,
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
        && let Some(caret) =
            text_surface::clipped_span_rect(rect, caret.x(), caret.y(), 1.0, caret.height())
    {
        scene.push_quad(text_surface::caret_quad(caret, theme));
    }
}
