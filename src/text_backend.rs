use std::sync::Arc;

use crate::{icon, paint, text};

pub fn font_system() -> glyphon::FontSystem {
    let mut font_system = glyphon::FontSystem::new();

    for font in iconflow::fonts() {
        font_system
            .db_mut()
            .load_font_source(glyphon::fontdb::Source::Binary(Arc::new(
                font.bytes.to_vec(),
            )));
    }

    font_system
}

pub fn attrs_for_style(style: text::Style) -> glyphon::Attrs<'static> {
    glyphon::Attrs::new()
        .family(glyphon::Family::SansSerif)
        .weight(weight(style.weight))
        .color(color(style.color))
        .metrics(glyphon::Metrics::relative(style.size.max(1.0), 1.25))
}

pub fn attrs_for_icon(
    glyph: icon::Glyph,
    size: f32,
    color: paint::Color,
) -> glyphon::Attrs<'static> {
    glyphon::Attrs::new()
        .family(glyphon::Family::Name(glyph.family()))
        .color(self::color(color))
        .metrics(glyphon::Metrics::relative(size.max(1.0), 1.0))
}

pub fn align(align: text::Align) -> glyphon::cosmic_text::Align {
    match align {
        text::Align::Start => glyphon::cosmic_text::Align::Left,
        text::Align::Center => glyphon::cosmic_text::Align::Center,
        text::Align::End => glyphon::cosmic_text::Align::Right,
    }
}

pub fn weight(weight: text::Weight) -> glyphon::Weight {
    match weight {
        text::Weight::Normal => glyphon::Weight::NORMAL,
        text::Weight::Medium => glyphon::Weight::MEDIUM,
        text::Weight::Bold => glyphon::Weight::BOLD,
    }
}

pub fn color(color: paint::Color) -> glyphon::Color {
    let channel = |value: f32| (value.clamp(0.0, 1.0) * 255.0).round() as u8;

    glyphon::Color::rgba(
        channel(color.r),
        channel(color.g),
        channel(color.b),
        channel(color.a),
    )
}
