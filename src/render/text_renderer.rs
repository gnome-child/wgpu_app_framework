use crate::paint;
use crate::render;
use crate::render::batch;
use crate::text_system;

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

struct PreparedText {
    buffer: glyphon::cosmic_text::Buffer,
    left: f32,
    top: f32,
    bounds: glyphon::TextBounds,
    default_color: glyphon::Color,
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
            font_system: text_system::font_system(),
            swash_cache: glyphon::SwashCache::new(),
        }
    }

    pub fn prepare_frame(
        &mut self,
        render_context: &render::Context,
        canvas: &render::Canvas,
        batch_count: usize,
    ) {
        self.update_viewport(render_context, canvas);
        self.ensure_renderers(batch_count, render_context);
    }

    pub fn prepare_batch(
        &mut self,
        render_context: &render::Context,
        canvas: &render::Canvas,
        renderer_index: usize,
        glyphs: &[batch::Glyph<'_>],
    ) -> Result<bool> {
        if glyphs.is_empty() {
            return Ok(false);
        }

        let scale_factor = canvas.scale_factor();
        let mut prepared = Vec::with_capacity(glyphs.len());

        for glyph in glyphs {
            let glyph = match glyph {
                batch::Glyph::Text(text) => prepare_text(&mut self.font_system, text, scale_factor),
                batch::Glyph::Icon(icon) => prepare_icon(&mut self.font_system, icon, scale_factor),
            };

            if let Some(glyph) = glyph {
                prepared.push(glyph);
            }
        }

        if prepared.is_empty() {
            return Ok(false);
        }

        let text_areas = prepared.iter().map(|text| glyphon::TextArea {
            buffer: &text.buffer,
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

    pub fn render(&self, renderer_index: usize, pass: &mut wgpu::RenderPass<'_>) -> Result<()> {
        self.renderers[renderer_index]
            .render(&self.atlas, &self.viewport, pass)
            .map_err(Error::from)
    }

    pub fn trim(&mut self) {
        self.atlas.trim();
    }

    fn update_viewport(&mut self, render_context: &render::Context, canvas: &render::Canvas) {
        let physical_area = canvas.physical_area();
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
) -> Option<PreparedText> {
    let width = text.rect.area.width().max(0.0);
    let height = text.rect.area.height().max(0.0);
    let prepared =
        text_system::prepare_document_buffer(font_system, &text.document, width, height)?;
    let buffer_height = height.min(prepared.line_height);

    let clip_left = text.rect.origin.x() * scale_factor;
    let clip_top = text.rect.origin.y() * scale_factor;
    let clip_right = clip_left + width * scale_factor;
    let clip_bottom = clip_top + height * scale_factor;
    let left = clip_left;
    let top = (text.rect.origin.y() + (height - buffer_height).max(0.0) * 0.5) * scale_factor;

    Some(PreparedText {
        buffer: prepared.buffer,
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

fn prepare_icon(
    font_system: &mut glyphon::FontSystem,
    icon: &paint::Icon,
    scale_factor: f32,
) -> Option<PreparedText> {
    let Some(glyph) = icon.icon.glyph() else {
        log::debug!("skipping missing icon glyph: {:?}", icon.icon);
        return None;
    };

    let width = icon.rect.area.width().max(0.0);
    let height = icon.rect.area.height().max(0.0);
    let prepared =
        text_system::prepare_icon_buffer(font_system, glyph, icon.size, icon.color, width, height)?;
    let buffer_height = height.min(prepared.line_height);

    let clip_left = icon.rect.origin.x() * scale_factor;
    let clip_top = icon.rect.origin.y() * scale_factor;
    let clip_right = clip_left + width * scale_factor;
    let clip_bottom = clip_top + height * scale_factor;
    let left = clip_left;
    let top = (icon.rect.origin.y() + (height - buffer_height).max(0.0) * 0.5) * scale_factor;

    Some(PreparedText {
        buffer: prepared.buffer,
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
