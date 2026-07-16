use crate::{layout, theme::Theme};

use super::super::{Quad, Rule, Scene, TextViewport};
use super::text_surface;

pub(super) fn paint(frame: &layout::Frame, scene: &mut Scene, theme: &Theme) -> Option<Rule> {
    let Some(text_area) = frame.text_area_layout() else {
        return None;
    };
    let rect = frame.text_area_text_rect();
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

    frame
        .text_caret_rect()
        .map(|caret| text_surface::caret_rule(caret, theme))
}
