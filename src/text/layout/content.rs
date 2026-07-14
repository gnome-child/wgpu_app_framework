use super::super::document::Style;
use super::super::edit::AreaWrap;
use crate::geometry::area;

pub(in crate::text) fn text_area_estimated_line_height(style: Style) -> f32 {
    glyphon::Metrics::relative(style.size().max(1.0), 1.25)
        .line_height
        .max(1.0)
}

pub(super) fn buffer_content_area(buffer: &glyphon::Buffer) -> area::Logical {
    let mut width: f32 = 0.0;
    let mut height: f32 = 0.0;

    for run in buffer.layout_runs() {
        let run_width = run
            .glyphs
            .iter()
            .map(|glyph| glyph.x + glyph.w)
            .fold(0.0_f32, f32::max);
        width = width.max(run_width);
        height = height.max(run.line_top + run.line_height);
    }

    if height == 0.0 {
        height = buffer.metrics().line_height;
    }

    area::logical(width, height)
}

pub(super) fn text_area_content_width(
    wrap: AreaWrap,
    viewport: area::Logical,
    observed_width: f32,
) -> f32 {
    match wrap {
        AreaWrap::None => observed_width.max(viewport.width().max(0.0)),
        AreaWrap::WordOrGlyph => viewport.width().max(0.0),
    }
}
