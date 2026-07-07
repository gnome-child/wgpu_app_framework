use std::sync::{Arc, OnceLock};

use glyphon::{Attrs, Buffer, Family, FontSystem, Metrics, Shaping};

use crate::{paint_geometry::area, text};

type FontDatabase = glyphon::fontdb::Database;

static FONT_DATABASE: OnceLock<(String, FontDatabase)> = OnceLock::new();

pub(crate) struct PreparedBuffer {
    pub(crate) buffer: Buffer,
    pub(crate) default_color: glyphon::Color,
    pub(crate) content_height: f32,
}

pub(crate) fn font_system() -> FontSystem {
    let (locale, database) = FONT_DATABASE.get_or_init(loaded_font_database);

    FontSystem::new_with_locale_and_db(locale.clone(), database.clone())
}

fn loaded_font_database() -> (String, FontDatabase) {
    let mut font_system = FontSystem::new();

    for font in iconflow::fonts() {
        font_system
            .db_mut()
            .load_font_source(glyphon::fontdb::Source::Binary(Arc::new(
                font.bytes.to_vec(),
            )));
    }

    font_system.into_locale_and_db()
}

pub(crate) fn measure_document(
    font_system: &mut FontSystem,
    document: &text::document::Document,
    measure: super::Measure,
) -> super::Metrics {
    let mut width = 0.0_f32;
    let mut height = 0.0_f32;
    let mut line_count = 0_usize;

    for block in document.blocks().iter().filter(|block| !block.is_empty()) {
        let Some(first_style) = block
            .runs()
            .iter()
            .find(|run| !run.is_empty())
            .map(text::document::Run::style)
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

    super::Metrics::new(area::logical(width, height), line_count)
}

pub(crate) fn prepare_document_buffer(
    font_system: &mut FontSystem,
    document: &text::document::Document,
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

    let content_height = prepared_content_height(&buffer, line_height);

    Some(PreparedBuffer {
        buffer,
        default_color: color(first_style.color()),
        content_height,
    })
}

fn prepared_content_height(buffer: &Buffer, fallback: f32) -> f32 {
    let mut height = 0.0_f32;
    let mut line_count = 0_usize;

    for run in buffer.layout_runs() {
        height = height.max(run.line_top + run.line_height);
        line_count += 1;
    }

    if line_count == 0 { fallback } else { height }
}

fn set_document_buffer(
    font_system: &mut FontSystem,
    buffer: &mut Buffer,
    block: &text::document::Block,
    first_style: text::document::Style,
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
        Some(align(block.align(), text::document::block_direction(block))),
    );
    buffer.shape_until_scroll(font_system, false);
}

pub(crate) fn attrs_for_style(style: text::document::Style) -> Attrs<'static> {
    Attrs::new()
        .family(Family::SansSerif)
        .weight(weight(style.weight()))
        .color(color(style.color()))
        .metrics(Metrics::relative(style.size().max(1.0), 1.25))
}

pub(crate) fn align(
    align: text::document::Align,
    direction: text::document::ResolvedTextDirection,
) -> glyphon::cosmic_text::Align {
    match align {
        text::document::Align::Start if direction.is_rtl() => glyphon::cosmic_text::Align::Right,
        text::document::Align::Start => glyphon::cosmic_text::Align::Left,
        text::document::Align::Center => glyphon::cosmic_text::Align::Center,
        text::document::Align::End if direction.is_rtl() => glyphon::cosmic_text::Align::Left,
        text::document::Align::End => glyphon::cosmic_text::Align::Right,
    }
}

pub(crate) fn weight(weight: text::document::Weight) -> glyphon::Weight {
    match weight {
        text::document::Weight::Normal => glyphon::Weight::NORMAL,
        text::document::Weight::Medium => glyphon::Weight::MEDIUM,
        text::document::Weight::Bold => glyphon::Weight::BOLD,
    }
}

pub(crate) fn color(color: text::Color) -> glyphon::Color {
    let (r, g, b, a) = color.channels();
    let channel = |value: f32| (value.clamp(0.0, 1.0) * 255.0).round() as u8;

    glyphon::Color::rgba(channel(r), channel(g), channel(b), channel(a))
}
