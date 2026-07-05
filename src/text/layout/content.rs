use super::super::document::Style;
use super::super::edit::{Area, AreaWrap};
use super::key::{BoundsKey, StyleKey};
use crate::geometry::area;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub(crate) struct AreaScrollKey {
    buffer_id: u64,
    revision: u64,
    style: StyleKey,
    viewport: BoundsKey,
    wrap: AreaWrap,
}

impl AreaScrollKey {
    pub(super) fn new(area_model: &Area, style: Style, viewport: area::Logical) -> Self {
        Self {
            buffer_id: area_model.buffer().id(),
            revision: area_model.buffer().revision(),
            style: StyleKey::new(style),
            viewport: BoundsKey::new(viewport),
            wrap: area_model.wrap(),
        }
    }
}

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

pub(crate) fn text_area_scroll_base_content_area(
    area_model: &Area,
    style: Style,
    viewport: area::Logical,
) -> (AreaScrollKey, area::Logical) {
    let key = AreaScrollKey::new(area_model, style, viewport);
    let line_height = text_area_estimated_line_height(style);
    let height = (area_model.buffer().logical_line_count() as f32 * line_height)
        .max(viewport.height().max(0.0));

    (key, area::logical(viewport.width().max(0.0), height))
}

pub(crate) fn stable_text_area_content_area(
    wrap: AreaWrap,
    base: area::Logical,
    hint: Option<area::Logical>,
    observed: area::Logical,
    viewport: area::Logical,
) -> area::Logical {
    let hint = hint.unwrap_or(base);
    let width = match wrap {
        AreaWrap::None => viewport
            .width()
            .max(base.width())
            .max(hint.width())
            .max(observed.width()),
        AreaWrap::WordOrGlyph => viewport.width().max(0.0),
    };

    area::logical(
        width,
        viewport
            .height()
            .max(base.height())
            .max(hint.height())
            .max(observed.height()),
    )
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
