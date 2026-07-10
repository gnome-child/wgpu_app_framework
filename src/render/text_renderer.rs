use crate::paint;
use crate::render;
use crate::render::batch;
#[cfg(test)]
use crate::text::layout as text_layout;
use crate::text::layout::{InlineCache, InlineStats};

use std::cell::{Ref, RefCell};
use std::rc::Rc;

use thiserror::Error;

pub(in crate::render) type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, Error)]
pub enum Error {
    #[error(transparent)]
    Prepare(#[from] glyphon::PrepareError),

    #[error(transparent)]
    Render(#[from] glyphon::RenderError),
}

pub(in crate::render) struct TextRenderer {
    cache: glyphon::Cache,
    atlas: glyphon::TextAtlas,
    renderers: Vec<glyphon::TextRenderer>,
    viewports: Vec<glyphon::Viewport>,
    swash_cache: glyphon::SwashCache,
    inline_cache: InlineCache,
}

struct PreparedText<'a> {
    buffer: PreparedTextBuffer<'a>,
    left: f32,
    top: f32,
    bounds: glyphon::TextBounds,
    default_color: glyphon::Color,
    stats: InlineStats,
}

enum PreparedTextBuffer<'a> {
    Shared(Rc<RefCell<glyphon::cosmic_text::Buffer>>),
    Borrowed(Ref<'a, glyphon::cosmic_text::Buffer>),
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub(in crate::render) struct TextBatchReport {
    pub(in crate::render) has_text: bool,
    pub(in crate::render) stats: InlineStats,
}

impl TextRenderer {
    pub(in crate::render) fn new(
        render_context: &render::Context,
        format: wgpu::TextureFormat,
    ) -> Self {
        let cache = glyphon::Cache::new(render_context.device());
        let atlas = glyphon::TextAtlas::new(
            render_context.device(),
            render_context.queue(),
            &cache,
            format,
        );

        Self {
            cache,
            atlas,
            renderers: Vec::new(),
            viewports: Vec::new(),
            swash_cache: glyphon::SwashCache::new(),
            inline_cache: InlineCache::new(),
        }
    }

    pub(in crate::render) fn prepare_frame(
        &mut self,
        render_context: &render::Context,
        batch_count: usize,
    ) {
        self.ensure_renderers(batch_count, render_context);
    }

    pub(in crate::render) fn prepare_batch(
        &mut self,
        render_context: &render::Context,
        viewport: render::Viewport,
        renderer_index: usize,
        glyphs: &[batch::Glyph<'_>],
    ) -> Result<TextBatchReport> {
        if glyphs.is_empty() {
            return Ok(TextBatchReport::default());
        }

        self.update_viewport(render_context, renderer_index, viewport);
        let scale_factor = viewport.scale_factor();
        let mut prepared = Vec::with_capacity(glyphs.len());
        let mut stats = InlineStats::default();

        for glyph in glyphs {
            match glyph {
                batch::Glyph::Text(text) => {
                    if let Some(glyph) = prepare_text(&mut self.inline_cache, text, scale_factor) {
                        stats.add(glyph.stats);
                        prepared.push(glyph);
                    }
                }
                batch::Glyph::TextViewport(text) => {
                    prepared.extend(prepare_text_viewport(text, scale_factor));
                }
                batch::Glyph::Icon(icon) => {
                    if let Some(glyph) = prepare_icon(&mut self.inline_cache, icon, scale_factor) {
                        stats.add(glyph.stats);
                        prepared.push(glyph);
                    }
                }
            }
        }

        if prepared.is_empty() {
            return Ok(TextBatchReport {
                has_text: false,
                stats,
            });
        }

        let borrowed = prepared
            .iter()
            .filter_map(|text| match &text.buffer {
                PreparedTextBuffer::Shared(buffer) => Some(buffer.borrow()),
                _ => None,
            })
            .collect::<Vec<_>>();
        let mut borrowed_index = 0_usize;
        let text_areas = prepared
            .iter()
            .map(|text| {
                let buffer = match &text.buffer {
                    PreparedTextBuffer::Borrowed(buffer) => buffer,
                    PreparedTextBuffer::Shared(_) => {
                        let buffer = &*borrowed[borrowed_index];
                        borrowed_index += 1;
                        buffer
                    }
                };

                glyphon::TextArea {
                    buffer,
                    left: text.left,
                    top: text.top,
                    scale: scale_factor,
                    bounds: text.bounds,
                    default_color: text.default_color,
                    custom_glyphs: &[],
                }
            })
            .collect::<Vec<_>>();

        self.renderers[renderer_index].prepare(
            render_context.device(),
            render_context.queue(),
            self.inline_cache.font_system_mut(),
            &mut self.atlas,
            &self.viewports[renderer_index],
            text_areas,
            &mut self.swash_cache,
        )?;

        Ok(TextBatchReport {
            has_text: true,
            stats,
        })
    }

    pub(in crate::render) fn render(
        &mut self,
        renderer_index: usize,
        pass: &mut wgpu::RenderPass<'_>,
    ) -> Result<()> {
        self.renderers[renderer_index]
            .render(&self.atlas, &self.viewports[renderer_index], pass)
            .map_err(Error::from)
    }

    pub(in crate::render) fn trim(&mut self) {
        self.atlas.trim();
    }

    fn update_viewport(
        &mut self,
        render_context: &render::Context,
        renderer_index: usize,
        viewport: render::Viewport,
    ) {
        let physical_area = viewport.physical_area();
        self.viewports[renderer_index].update(
            render_context.queue(),
            glyphon::Resolution {
                width: physical_area.width(),
                height: physical_area.height(),
            },
        );
    }

    fn ensure_renderers(&mut self, count: usize, render_context: &render::Context) {
        while self.renderers.len() < count {
            self.renderers.push(glyphon::TextRenderer::new(
                &mut self.atlas,
                render_context.device(),
                wgpu::MultisampleState::default(),
                None,
            ));
            self.viewports
                .push(glyphon::Viewport::new(render_context.device(), &self.cache));
        }
    }
}

fn prepare_text(
    inline_cache: &mut InlineCache,
    text: &paint::Text,
    scale_factor: f32,
) -> Option<PreparedText<'static>> {
    let grid = paint::Grid::new(scale_factor);
    let width = text.rect.area.width().max(0.0);
    let height = text.rect.area.height().max(0.0);
    let prepared = inline_cache.prepare_text(&text.document, width, height, wrap(text.wrap))?;

    let clip_left = text.rect.origin.x() * scale_factor;
    let clip_top = text.rect.origin.y() * scale_factor;
    let clip_right = clip_left + width * scale_factor;
    let clip_bottom = clip_top + height * scale_factor;
    let left = grid.snap_text_origin(text.rect.origin.x());
    let top = grid.snap_centered_text_origin(text.rect.origin.y(), height, prepared.content_height);

    Some(PreparedText {
        buffer: PreparedTextBuffer::Shared(prepared.buffer),
        left,
        top,
        bounds: glyphon::TextBounds {
            left: clip_left.floor() as i32,
            top: clip_top.floor() as i32,
            right: clip_right.ceil() as i32,
            bottom: clip_bottom.ceil() as i32,
        },
        default_color: prepared.default_color,
        stats: prepared.stats,
    })
}

fn prepare_text_viewport<'a>(
    viewport: &'a paint::TextViewport,
    scale_factor: f32,
) -> impl Iterator<Item = PreparedText<'a>> + 'a {
    viewport.surfaces.iter().filter_map(move |surface| {
        prepare_text_surface_in_bounds(surface, viewport.rect, scale_factor)
    })
}

fn prepare_text_surface_in_bounds<'a>(
    text: &'a paint::TextSurface,
    bounds_rect: crate::paint::Rect,
    scale_factor: f32,
) -> Option<PreparedText<'a>> {
    let grid = paint::Grid::new(scale_factor);
    let width = text.rect.area.width().max(0.0);
    let height = text.rect.area.height().max(0.0);
    if width <= 0.0 || height <= 0.0 {
        return None;
    }

    let clip_left = bounds_rect.origin.x() * scale_factor;
    let clip_top = bounds_rect.origin.y() * scale_factor;
    let clip_right = clip_left + bounds_rect.area.width().max(0.0) * scale_factor;
    let clip_bottom = clip_top + bounds_rect.area.height().max(0.0) * scale_factor;
    let left = grid.snap_text_origin(text.rect.origin.x());
    let top = grid.snap_text_origin(text.rect.origin.y());

    Some(PreparedText {
        buffer: PreparedTextBuffer::Borrowed(text.buffer.borrow()),
        left,
        top,
        bounds: glyphon::TextBounds {
            left: clip_left.floor() as i32,
            top: clip_top.floor() as i32,
            right: clip_right.ceil() as i32,
            bottom: clip_bottom.ceil() as i32,
        },
        default_color: glyphon_color_from_linear_paint(text.default_color),
        stats: InlineStats::default(),
    })
}

fn wrap(wrap: paint::TextWrap) -> glyphon::Wrap {
    match wrap {
        paint::TextWrap::WordOrGlyph => glyphon::Wrap::WordOrGlyph,
        paint::TextWrap::None => glyphon::Wrap::None,
    }
}

fn prepare_icon(
    inline_cache: &mut InlineCache,
    icon: &paint::Icon,
    scale_factor: f32,
) -> Option<PreparedText<'static>> {
    let Some(glyph) = icon.icon.glyph() else {
        log::debug!("skipping missing icon glyph: {:?}", icon.icon);
        return None;
    };

    let width = icon.rect.area.width().max(0.0);
    let height = icon.rect.area.height().max(0.0);
    let prepared = inline_cache.prepare_icon(glyph, icon.size, width, height)?;
    let buffer_height = height.min(prepared.line_height);

    let clip_left = icon.rect.origin.x() * scale_factor;
    let clip_top = icon.rect.origin.y() * scale_factor;
    let clip_right = clip_left + width * scale_factor;
    let clip_bottom = clip_top + height * scale_factor;
    let left = clip_left;
    let top = (icon.rect.origin.y() + (height - buffer_height).max(0.0) * 0.5) * scale_factor;

    Some(PreparedText {
        buffer: PreparedTextBuffer::Shared(prepared.buffer),
        left,
        top,
        bounds: glyphon::TextBounds {
            left: clip_left.floor() as i32,
            top: clip_top.floor() as i32,
            right: clip_right.ceil() as i32,
            bottom: clip_bottom.ceil() as i32,
        },
        default_color: glyphon_color_from_linear_paint(icon.color),
        stats: prepared.stats,
    })
}

fn glyphon_color_from_linear_paint(color: paint::Color) -> glyphon::Color {
    glyphon::Color::rgba(
        crate::color::linear_unit_to_srgb_byte(color.r),
        crate::color::linear_unit_to_srgb_byte(color.g),
        crate::color::linear_unit_to_srgb_byte(color.b),
        crate::color::unit_to_byte(color.a),
    )
}

#[cfg(test)]
mod tests {
    use crate::paint::{self, Rect};
    use crate::text::document::{Align, Block, Run, Style, Weight};
    use crate::{icon, text};

    use super::*;

    fn centered_text(document: text::document::Document, height: f32) -> paint::Text {
        paint::Text {
            rect: Rect::new(
                paint::point::logical(4.0, 7.0),
                paint::area::logical(240.0, height),
            ),
            document,
            wrap: paint::TextWrap::None,
            vertical_align: paint::TextVerticalAlign::Center,
        }
    }

    fn label_text(
        value: &str,
        color: text::Color,
        size: f32,
        weight: Weight,
        origin_x: f32,
    ) -> paint::Text {
        let mut block = Block::new(Align::Start);
        block.push_run(Run::new(
            value,
            Style::default()
                .with_color(color)
                .with_size(size)
                .with_weight(weight),
        ));

        paint::Text {
            rect: Rect::new(
                paint::point::logical(origin_x, 0.0),
                paint::area::logical(160.0, 22.0),
            ),
            document: text::document::Document::from_block(block),
            wrap: paint::TextWrap::None,
            vertical_align: paint::TextVerticalAlign::Center,
        }
    }

    fn document_height(document: &text::document::Document) -> f32 {
        let mut font_system = text_layout::glyphon_font_system();
        text_layout::measure_document_with_glyphon(
            &mut font_system,
            document,
            text::layout::Measure::bounded(paint::area::logical(240.0, 1_000.0)),
        )
        .height()
    }

    fn assert_close(actual: f32, expected: f32) {
        assert!(
            (actual - expected).abs() <= 0.01,
            "expected {actual} to be within 0.01 of {expected}"
        );
    }

    #[test]
    fn centered_multiline_text_uses_prepared_content_height() {
        let document = text::document::Document::plain("one\ntwo\nthree");
        let content_height = document_height(&document);
        let text = centered_text(document, content_height);
        let mut cache = InlineCache::new();

        let prepared = prepare_text(&mut cache, &text, 1.0).expect("text should prepare");

        assert_close(prepared.top, text.rect.origin.y());
    }

    #[test]
    fn centered_single_line_text_keeps_existing_centering() {
        let document = text::document::Document::plain("one");
        let content_height = document_height(&document);
        let rect_height = content_height + 40.0;
        let text = centered_text(document, rect_height);
        let mut cache = InlineCache::new();

        let prepared = prepare_text(&mut cache, &text, 1.0).expect("text should prepare");

        assert_close(prepared.top, text.rect.origin.y() + 20.0);
    }

    #[test]
    fn prepared_text_cache_ignores_rect_origin() {
        let mut cache = InlineCache::new();
        let first = label_text("Command", text::Color::BLACK, 12.0, Weight::Normal, 0.0);
        let second = label_text("Command", text::Color::BLACK, 12.0, Weight::Normal, 40.0);

        let first = prepare_text(&mut cache, &first, 1.0).expect("first label should prepare");
        let second = prepare_text(&mut cache, &second, 1.0).expect("second label should prepare");

        assert_eq!(first.stats.text_cache_misses, 1);
        assert_eq!(first.stats.text_shape_calls, 1);
        assert_eq!(second.stats.text_cache_hits, 1);
        assert_eq!(second.stats.text_shape_calls, 0);
    }

    #[test]
    fn prepared_text_cache_uses_current_color_without_reshaping() {
        let mut cache = InlineCache::new();
        let red = label_text("Command", text::Color::RED, 12.0, Weight::Normal, 0.0);
        let black = label_text("Command", text::Color::BLACK, 12.0, Weight::Normal, 0.0);

        let _ = prepare_text(&mut cache, &red, 1.0).expect("red label should prepare");
        let black = prepare_text(&mut cache, &black, 1.0).expect("black label should prepare");

        assert_eq!(black.stats.text_cache_hits, 1);
        assert_eq!(black.stats.text_shape_calls, 0);
        assert_eq!(
            black.default_color,
            text_layout::glyphon_color(text::Color::BLACK)
        );
    }

    #[test]
    fn prepared_text_cache_misses_when_bounds_change() {
        let mut cache = InlineCache::new();
        let first = label_text("Command", text::Color::BLACK, 12.0, Weight::Normal, 0.0);
        let mut second = first.clone();
        second.rect.area = paint::area::logical(180.0, 22.0);

        let _ = prepare_text(&mut cache, &first, 1.0).expect("first label should prepare");
        let second = prepare_text(&mut cache, &second, 1.0).expect("second label should prepare");

        assert_eq!(second.stats.text_cache_misses, 1);
        assert_eq!(second.stats.text_shape_calls, 1);
    }

    #[test]
    fn multi_color_text_stays_on_uncached_path() {
        let mut cache = InlineCache::new();
        let mut block = Block::new(Align::Start);
        block.push_run(Run::new(
            "Red",
            Style::default()
                .with_color(text::Color::RED)
                .with_size(12.0),
        ));
        block.push_run(Run::new(
            "Black",
            Style::default()
                .with_color(text::Color::BLACK)
                .with_size(12.0),
        ));
        let rich = paint::Text {
            rect: Rect::new(
                paint::point::logical(0.0, 0.0),
                paint::area::logical(160.0, 22.0),
            ),
            document: text::document::Document::from_block(block),
            wrap: paint::TextWrap::None,
            vertical_align: paint::TextVerticalAlign::Center,
        };

        let first = prepare_text(&mut cache, &rich, 1.0).expect("rich text should prepare");
        let second = prepare_text(&mut cache, &rich, 1.0).expect("rich text should prepare again");

        assert_eq!(first.stats.text_cache_hits, 0);
        assert_eq!(second.stats.text_cache_hits, 0);
        assert_eq!(second.stats.text_shape_calls, 1);
    }

    #[test]
    fn prepared_icon_cache_uses_current_color_without_reshaping() {
        let mut cache = InlineCache::new();
        let icon = icon::Icon::phosphor(icon::Id::new("command"));
        let red = paint::Icon {
            rect: Rect::new(
                paint::point::logical(0.0, 0.0),
                paint::area::logical(18.0, 18.0),
            ),
            icon,
            color: paint::Color::RED,
            size: 12.0,
        };
        let black = paint::Icon {
            color: paint::Color::BLACK,
            ..red
        };

        let _ = prepare_icon(&mut cache, &red, 1.0).expect("red icon should prepare");
        let black = prepare_icon(&mut cache, &black, 1.0).expect("black icon should prepare");

        assert_eq!(black.stats.icon_cache_hits, 1);
        assert_eq!(black.stats.icon_shape_calls, 0);
        assert_eq!(
            black.default_color,
            glyphon_color_from_linear_paint(paint::Color::BLACK)
        );
    }
}
