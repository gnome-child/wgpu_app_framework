use crate::render;
use crate::render::batch;
use crate::text::layout::system;
use crate::{icon, paint};

use std::cell::Ref;

use thiserror::Error;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, Error)]
pub enum Error {
    #[error(transparent)]
    Prepare(#[from] glyphon::PrepareError),

    #[error(transparent)]
    Render(#[from] glyphon::RenderError),
}

pub struct TextRenderer {
    viewport: glyphon::Viewport,
    atlas: glyphon::TextAtlas,
    renderers: Vec<glyphon::TextRenderer>,
    font_system: glyphon::FontSystem,
    swash_cache: glyphon::SwashCache,
}

struct PreparedText<'a> {
    buffer: PreparedTextBuffer<'a>,
    left: f32,
    top: f32,
    bounds: glyphon::TextBounds,
    default_color: glyphon::Color,
}

enum PreparedTextBuffer<'a> {
    Owned(glyphon::cosmic_text::Buffer),
    Shared(Ref<'a, glyphon::cosmic_text::Buffer>),
}

impl<'a> PreparedTextBuffer<'a> {
    fn as_ref(&self) -> &glyphon::cosmic_text::Buffer {
        match self {
            Self::Owned(buffer) => buffer,
            Self::Shared(buffer) => buffer,
        }
    }
}

impl TextRenderer {
    pub fn new(render_context: &render::Context, format: wgpu::TextureFormat) -> Self {
        let cache = glyphon::Cache::new(render_context.device());
        let viewport = glyphon::Viewport::new(render_context.device(), &cache);
        let atlas = glyphon::TextAtlas::new(
            render_context.device(),
            render_context.queue(),
            &cache,
            format,
        );

        Self {
            viewport,
            atlas,
            renderers: Vec::new(),
            font_system: system::font_system(),
            swash_cache: glyphon::SwashCache::new(),
        }
    }

    pub fn prepare_frame(&mut self, render_context: &render::Context, batch_count: usize) {
        self.ensure_renderers(batch_count, render_context);
    }

    pub fn prepare_batch(
        &mut self,
        render_context: &render::Context,
        viewport: render::Viewport,
        renderer_index: usize,
        glyphs: &[batch::Glyph<'_>],
    ) -> Result<bool> {
        if glyphs.is_empty() {
            return Ok(false);
        }

        self.update_viewport(render_context, viewport);
        let scale_factor = viewport.scale_factor();
        let mut prepared = Vec::with_capacity(glyphs.len());

        for glyph in glyphs {
            match glyph {
                batch::Glyph::Text(text) => {
                    if let Some(glyph) = prepare_text(&mut self.font_system, text, scale_factor) {
                        prepared.push(glyph);
                    }
                }
                batch::Glyph::TextSurface(text) => {
                    if let Some(glyph) = prepare_text_surface(text, scale_factor) {
                        prepared.push(glyph);
                    }
                }
                batch::Glyph::TextViewport(text) => {
                    prepared.extend(prepare_text_viewport(text, scale_factor));
                }
                batch::Glyph::Icon(icon) => {
                    if let Some(glyph) = prepare_icon(&mut self.font_system, icon, scale_factor) {
                        prepared.push(glyph);
                    }
                }
            }
        }

        if prepared.is_empty() {
            return Ok(false);
        }

        let text_areas = prepared.iter().map(|text| glyphon::TextArea {
            buffer: text.buffer.as_ref(),
            left: text.left,
            top: text.top,
            scale: scale_factor,
            bounds: text.bounds,
            default_color: text.default_color,
            custom_glyphs: &[],
        });

        self.renderers[renderer_index].prepare(
            render_context.device(),
            render_context.queue(),
            &mut self.font_system,
            &mut self.atlas,
            &self.viewport,
            text_areas,
            &mut self.swash_cache,
        )?;

        Ok(true)
    }

    pub fn render(
        &mut self,
        render_context: &render::Context,
        viewport: render::Viewport,
        renderer_index: usize,
        pass: &mut wgpu::RenderPass<'_>,
    ) -> Result<()> {
        self.update_viewport(render_context, viewport);
        self.renderers[renderer_index]
            .render(&self.atlas, &self.viewport, pass)
            .map_err(Error::from)
    }

    pub fn trim(&mut self) {
        self.atlas.trim();
    }

    fn update_viewport(&mut self, render_context: &render::Context, viewport: render::Viewport) {
        let physical_area = viewport.physical_area();
        self.viewport.update(
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
        }
    }
}

fn prepare_text(
    font_system: &mut glyphon::FontSystem,
    text: &paint::Text,
    scale_factor: f32,
) -> Option<PreparedText<'static>> {
    let width = text.rect.area.width().max(0.0);
    let height = text.rect.area.height().max(0.0);
    let prepared = system::prepare_document_buffer(
        font_system,
        &text.document,
        width,
        height,
        wrap(text.wrap),
    )?;

    let clip_left = text.rect.origin.x() * scale_factor;
    let clip_top = text.rect.origin.y() * scale_factor;
    let clip_right = clip_left + width * scale_factor;
    let clip_bottom = clip_top + height * scale_factor;
    let left = snap_text_origin(clip_left);
    let top = match text.vertical_align {
        paint::TextVerticalAlign::Start => text.rect.origin.y(),
        paint::TextVerticalAlign::Center => {
            text.rect.origin.y() + (height - height.min(prepared.content_height)).max(0.0) * 0.5
        }
    } * scale_factor;
    let top = snap_text_origin(top);

    Some(PreparedText {
        buffer: PreparedTextBuffer::Owned(prepared.buffer),
        left,
        top,
        bounds: glyphon::TextBounds {
            left: clip_left.floor() as i32,
            top: clip_top.floor() as i32,
            right: clip_right.ceil() as i32,
            bottom: clip_bottom.ceil() as i32,
        },
        default_color: prepared.default_color,
    })
}

#[cfg(test)]
mod tests {
    use crate::geometry::{Rect, area, point};
    use crate::text::layout::system;
    use crate::{paint, text};

    use super::*;

    fn centered_text(document: text::document::Document, height: f32) -> paint::Text {
        paint::Text {
            rect: Rect::new(point::logical(4.0, 7.0), area::logical(240.0, height)),
            document,
            wrap: paint::TextWrap::None,
            vertical_align: paint::TextVerticalAlign::Center,
        }
    }

    fn document_height(document: &text::document::Document) -> f32 {
        let mut font_system = system::font_system();
        system::measure_document(
            &mut font_system,
            document,
            text::layout::Measure::bounded(area::logical(240.0, 1_000.0)),
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
        let mut font_system = system::font_system();

        let prepared = prepare_text(&mut font_system, &text, 1.0).expect("text should prepare");

        assert_close(prepared.top, text.rect.origin.y());
    }

    #[test]
    fn centered_single_line_text_keeps_existing_centering() {
        let document = text::document::Document::plain("one");
        let content_height = document_height(&document);
        let rect_height = content_height + 40.0;
        let text = centered_text(document, rect_height);
        let mut font_system = system::font_system();

        let prepared = prepare_text(&mut font_system, &text, 1.0).expect("text should prepare");

        assert_close(prepared.top, text.rect.origin.y() + 20.0);
    }
}

fn prepare_text_surface<'a>(
    text: &'a paint::TextSurface,
    scale_factor: f32,
) -> Option<PreparedText<'a>> {
    prepare_text_surface_in_bounds(text, text.rect, scale_factor)
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
    bounds_rect: crate::geometry::Rect,
    scale_factor: f32,
) -> Option<PreparedText<'a>> {
    let width = text.rect.area.width().max(0.0);
    let height = text.rect.area.height().max(0.0);
    if width <= 0.0 || height <= 0.0 {
        return None;
    }

    let clip_left = bounds_rect.origin.x() * scale_factor;
    let clip_top = bounds_rect.origin.y() * scale_factor;
    let clip_right = clip_left + bounds_rect.area.width().max(0.0) * scale_factor;
    let clip_bottom = clip_top + bounds_rect.area.height().max(0.0) * scale_factor;
    let left = snap_text_origin(text.rect.origin.x() * scale_factor);
    let top = snap_text_origin(text.rect.origin.y() * scale_factor);

    Some(PreparedText {
        buffer: PreparedTextBuffer::Shared(text.buffer.borrow()),
        left,
        top,
        bounds: glyphon::TextBounds {
            left: clip_left.floor() as i32,
            top: clip_top.floor() as i32,
            right: clip_right.ceil() as i32,
            bottom: clip_bottom.ceil() as i32,
        },
        default_color: paint_color(text.default_color),
    })
}

fn snap_text_origin(position: f32) -> f32 {
    position.round()
}

fn wrap(wrap: paint::TextWrap) -> glyphon::Wrap {
    match wrap {
        paint::TextWrap::WordOrGlyph => glyphon::Wrap::WordOrGlyph,
        paint::TextWrap::None => glyphon::Wrap::None,
    }
}

fn prepare_icon(
    font_system: &mut glyphon::FontSystem,
    icon: &paint::Icon,
    scale_factor: f32,
) -> Option<PreparedText<'static>> {
    let Some(glyph) = icon.icon.glyph() else {
        log::debug!("skipping missing icon glyph: {:?}", icon.icon);
        return None;
    };

    let width = icon.rect.area.width().max(0.0);
    let height = icon.rect.area.height().max(0.0);
    let prepared = prepare_icon_buffer(font_system, glyph, icon.size, icon.color, width, height)?;
    let buffer_height = height.min(prepared.line_height);

    let clip_left = icon.rect.origin.x() * scale_factor;
    let clip_top = icon.rect.origin.y() * scale_factor;
    let clip_right = clip_left + width * scale_factor;
    let clip_bottom = clip_top + height * scale_factor;
    let left = clip_left;
    let top = (icon.rect.origin.y() + (height - buffer_height).max(0.0) * 0.5) * scale_factor;

    Some(PreparedText {
        buffer: PreparedTextBuffer::Owned(prepared.buffer),
        left,
        top,
        bounds: glyphon::TextBounds {
            left: clip_left.floor() as i32,
            top: clip_top.floor() as i32,
            right: clip_right.ceil() as i32,
            bottom: clip_bottom.ceil() as i32,
        },
        default_color: prepared.default_color,
    })
}

struct PreparedIconBuffer {
    buffer: glyphon::Buffer,
    default_color: glyphon::Color,
    line_height: f32,
}

fn prepare_icon_buffer(
    font_system: &mut glyphon::FontSystem,
    icon: icon::Glyph,
    size: f32,
    color: paint::Color,
    width: f32,
    height: f32,
) -> Option<PreparedIconBuffer> {
    let Some(character) = icon.character() else {
        log::debug!("skipping invalid icon codepoint: {:?}", icon);
        return None;
    };

    let font_size = size.max(1.0);
    let line_height = font_size;
    let buffer_height = height.min(line_height);
    let mut buffer = glyphon::Buffer::new(font_system, glyphon::Metrics::relative(font_size, 1.0));
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
        glyphon::Shaping::Basic,
        Some(glyphon::cosmic_text::Align::Center),
    );
    buffer.shape_until_scroll(font_system, false);

    Some(PreparedIconBuffer {
        buffer,
        default_color: paint_color(color),
        line_height,
    })
}

fn attrs_for_icon(glyph: icon::Glyph, size: f32, color: paint::Color) -> glyphon::Attrs<'static> {
    glyphon::Attrs::new()
        .family(glyphon::Family::Name(glyph.family()))
        .color(paint_color(color))
        .metrics(glyphon::Metrics::relative(size.max(1.0), 1.0))
}

fn paint_color(color: paint::Color) -> glyphon::Color {
    let channel = |value: f32| (value.clamp(0.0, 1.0) * 255.0).round() as u8;

    glyphon::Color::rgba(
        channel(color.r),
        channel(color.g),
        channel(color.b),
        channel(color.a),
    )
}
