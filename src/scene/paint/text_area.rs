use crate::{layout, theme::Theme};

use super::super::{Quad, Scene, TextViewport, Visuals};
use super::text_surface;

pub(super) fn paint(
    frame: &layout::Frame,
    text_area: &layout::TextArea,
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
    if caret_visible && let Some(caret) = frame.text_caret_rect() {
        scene.push_rule(text_surface::caret_rule(caret, theme));
    }
}
