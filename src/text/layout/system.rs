use crate::geometry::area;
use std::sync::{Arc, OnceLock};

use glyphon::{Attrs, Buffer, Family, FontSystem, Metrics, Shaping};

use crate::text;

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

    for bytes in crate::icon::font_bytes() {
        font_system
            .db_mut()
            .load_font_source(glyphon::fontdb::Source::Binary(Arc::new(bytes.to_vec())));
    }

    font_system.into_locale_and_db()
}

pub(crate) fn measure_document(
    font_system: &mut FontSystem,
    document: &text::document::Document,
    measure: super::Measure,
) -> super::Metrics {
    measure_document_with_line_widths(font_system, document, measure).0
}

pub(crate) fn measure_document_with_line_widths(
    font_system: &mut FontSystem,
    document: &text::document::Document,
    measure: super::Measure,
) -> (super::Metrics, Vec<f32>) {
    let mut width = 0.0_f32;
    let mut height = 0.0_f32;
    let mut line_count = 0_usize;
    let mut line_widths = Vec::new();

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
        let mut block_line_widths = Vec::<f32>::new();
        for run in buffer.layout_runs() {
            width = width.max(run.line_w);
            block_height = block_height.max(run.line_top + run.line_height);
            block_lines += 1;
            if block_line_widths.len() <= run.line_i {
                block_line_widths.resize(run.line_i + 1, 0.0);
            }
            block_line_widths[run.line_i] = block_line_widths[run.line_i].max(run.line_w);
        }

        if block_lines == 0 {
            block_height = block_height.max(font_size * 1.25);
            block_lines = 1;
            block_line_widths.push(0.0);
        }

        height += block_height;
        line_count += block_lines;
        line_widths.extend(block_line_widths);
    }

    (
        super::Metrics::new(area::logical(width, height), line_count),
        line_widths,
    )
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

pub(in crate::text::layout) fn prepared_content_height(buffer: &Buffer, fallback: f32) -> f32 {
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
    glyphon::Color::rgba(
        crate::color::unit_to_byte(r),
        crate::color::unit_to_byte(g),
        crate::color::unit_to_byte(b),
        crate::color::unit_to_byte(a),
    )
}
