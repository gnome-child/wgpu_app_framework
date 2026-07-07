use crate::{layout, theme::Theme};

use super::super::{Quad, Scene, TextViewport, Visuals};
use super::text_surface;

pub(super) fn paint(
    frame: &layout::Frame,
    text_area: &layout::text::Area,
    scene: &mut Scene,
    theme: &Theme,
    visuals: &Visuals,
) {
    let rect = frame.rect();
    for span in text_area.layout().selection_spans() {
        if let Some(span) =
            text_surface::clipped_span_rect(rect, span.x(), span.y(), span.width(), span.height())
        {
            scene.push_quad(Quad::new(span, theme.text().selection));
        }
    }

    scene.push_text_viewport(TextViewport::new(
        rect,
        text_area
            .render_surfaces()
            .iter()
            .map(|surface| text_surface::surface(rect, surface))
            .collect(),
    ));

    let caret_visible = frame
        .target()
        .is_none_or(|target| visuals.caret_visible(target));
    if caret_visible
        && let Some(caret) = text_area.layout().caret()
        && let Some(caret) =
            text_surface::clipped_span_rect(rect, caret.x(), caret.y(), 1.0, caret.height())
    {
        scene.push_quad(text_surface::caret_quad(caret, theme));
    }
}
