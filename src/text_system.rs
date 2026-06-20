use std::sync::Arc;

use glyphon::{Attrs, Buffer, Family, FontSystem, Metrics, Shaping};

use crate::{icon, paint, text};

pub struct PreparedBuffer {
    pub buffer: Buffer,
    pub default_color: glyphon::Color,
    pub line_height: f32,
}

pub fn font_system() -> FontSystem {
    let mut font_system = FontSystem::new();

    for font in iconflow::fonts() {
        font_system
            .db_mut()
            .load_font_source(glyphon::fontdb::Source::Binary(Arc::new(
                font.bytes.to_vec(),
            )));
    }

    font_system
}

pub fn measure_document(
    font_system: &mut FontSystem,
    document: &text::Document,
    measure: text::Measure,
) -> text::Metrics {
    let mut width = 0.0_f32;
    let mut height = 0.0_f32;
    let mut line_count = 0_usize;

    for block in document.blocks().iter().filter(|block| !block.is_empty()) {
        let Some(first_style) = block
            .runs()
            .iter()
            .find(|run| !run.is_empty())
            .map(text::Run::style)
        else {
            continue;
        };
        let font_size = first_style.size().max(1.0);
        let mut buffer = Buffer::new(font_system, Metrics::relative(font_size, 1.25));
        let max_width = measure
            .max()
            .map(|max| max.width().max(0.0))
            .filter(|width| width.is_finite());

        set_document_buffer(
            font_system,
            &mut buffer,
            block,
            first_style,
            max_width,
            None,
            glyphon::Wrap::WordOrGlyph,
        );

        let mut block_height = 0.0_f32;
        let mut block_lines = 0_usize;
        for run in buffer.layout_runs() {
            width = width.max(run.line_w);
            block_height = block_height.max(run.line_top + run.line_height);
            block_lines += 1;
        }

        if block_lines == 0 {
            block_height = block_height.max(font_size * 1.25);
            block_lines = 1;
        }

        height += block_height;
        line_count += block_lines;
    }

    text::Metrics::new(crate::geometry::area::logical(width, height), line_count)
}

pub fn prepare_document_buffer(
    font_system: &mut FontSystem,
    document: &text::Document,
    width: f32,
    height: f32,
    wrap: glyphon::Wrap,
) -> Option<PreparedBuffer> {
    let block = document.blocks().iter().find(|block| !block.is_empty())?;
    let first_style = block.runs().iter().find(|run| !run.is_empty())?.style();
    let font_size = first_style.size().max(1.0);
    let line_height = font_size * 1.25;
    let mut buffer = Buffer::new(font_system, Metrics::relative(font_size, 1.25));

    set_document_buffer(
        font_system,
        &mut buffer,
        block,
        first_style,
        Some(width.max(0.0)),
        Some(height.max(0.0)),
        wrap,
    );

    Some(PreparedBuffer {
        buffer,
        default_color: color(first_style.color()),
        line_height,
    })
}

pub fn prepare_icon_buffer(
    font_system: &mut FontSystem,
    icon: icon::Glyph,
    size: f32,
    color: paint::Color,
    width: f32,
    height: f32,
) -> Option<PreparedBuffer> {
    let Some(character) = icon.character() else {
        log::debug!("skipping invalid icon codepoint: {:?}", icon);
        return None;
    };

    let font_size = size.max(1.0);
    let line_height = font_size;
    let buffer_height = height.min(line_height);
    let mut buffer = Buffer::new(font_system, Metrics::relative(font_size, 1.0));
    let attrs = attrs_for_icon(icon, font_size, color);
    let text = character.to_string();

    buffer.set_size(
        font_system,
        Some(width.max(0.0)),
        Some(buffer_height.max(0.0)),
    );
    buffer.set_rich_text(
        font_system,
        vec![(text.as_str(), attrs.clone())],
        &attrs,
        Shaping::Basic,
        Some(glyphon::cosmic_text::Align::Center),
    );
    buffer.shape_until_scroll(font_system, false);

    Some(PreparedBuffer {
        buffer,
        default_color: self::color(color),
        line_height,
    })
}

fn set_document_buffer(
    font_system: &mut FontSystem,
    buffer: &mut Buffer,
    block: &text::Block,
    first_style: text::Style,
    width: Option<f32>,
    height: Option<f32>,
    wrap: glyphon::Wrap,
) {
    let spans = block
        .runs()
        .iter()
        .filter(|run| !run.is_empty())
        .map(|run| (run.text(), attrs_for_style(run.style())))
        .collect::<Vec<_>>();
    let default_attrs = attrs_for_style(first_style);

    buffer.set_size(font_system, width, height);
    buffer.set_wrap(font_system, wrap);
    buffer.set_rich_text(
        font_system,
        spans,
        &default_attrs,
        Shaping::Advanced,
        Some(align(block.align(), text::block_direction(block))),
    );
    buffer.shape_until_scroll(font_system, false);
}

pub fn attrs_for_style(style: text::Style) -> Attrs<'static> {
    Attrs::new()
        .family(Family::SansSerif)
        .weight(weight(style.weight()))
        .color(color(style.color()))
        .metrics(Metrics::relative(style.size().max(1.0), 1.25))
}

pub fn attrs_for_icon(glyph: icon::Glyph, size: f32, color: paint::Color) -> Attrs<'static> {
    Attrs::new()
        .family(Family::Name(glyph.family()))
        .color(self::color(color))
        .metrics(Metrics::relative(size.max(1.0), 1.0))
}

pub fn align(
    align: text::Align,
    direction: text::ResolvedTextDirection,
) -> glyphon::cosmic_text::Align {
    match align {
        text::Align::Start if direction.is_rtl() => glyphon::cosmic_text::Align::Right,
        text::Align::Start => glyphon::cosmic_text::Align::Left,
        text::Align::Center => glyphon::cosmic_text::Align::Center,
        text::Align::End if direction.is_rtl() => glyphon::cosmic_text::Align::Left,
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
