use crate::paint;
use crate::render;
use crate::render::batch;
use crate::text_backend;

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
    buffer: glyphon::Buffer,
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
            font_system: text_backend::font_system(),
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
    let block = text
        .document
        .blocks()
        .iter()
        .find(|block| !block.is_empty())?;
    let first_style = block.runs().iter().find(|run| !run.is_empty())?.style();
    let font_size = first_style.size.max(1.0);
    let line_height = font_size * 1.25;
    let mut buffer = glyphon::Buffer::new(font_system, glyphon::Metrics::relative(font_size, 1.25));
    let width = text.rect.area.width().max(0.0);
    let height = text.rect.area.height().max(0.0);
    let buffer_height = height.min(line_height);

    buffer.set_size(font_system, Some(width), Some(buffer_height));

    let spans = block
        .runs()
        .iter()
        .filter(|run| !run.is_empty())
        .map(|run| (run.text(), text_backend::attrs_for_style(run.style())))
        .collect::<Vec<_>>();

    let default_attrs = text_backend::attrs_for_style(first_style);
    buffer.set_rich_text(
        font_system,
        spans,
        &default_attrs,
        glyphon::Shaping::Advanced,
        Some(text_backend::align(block.align())),
    );
    buffer.shape_until_scroll(font_system, false);

    let clip_left = text.rect.origin.x() * scale_factor;
    let clip_top = text.rect.origin.y() * scale_factor;
    let clip_right = clip_left + width * scale_factor;
    let clip_bottom = clip_top + height * scale_factor;
    let left = clip_left;
    let top = (text.rect.origin.y() + (height - buffer_height).max(0.0) * 0.5) * scale_factor;

    Some(PreparedText {
        buffer,
        left,
        top,
        bounds: glyphon::TextBounds {
            left: clip_left.floor() as i32,
            top: clip_top.floor() as i32,
            right: clip_right.ceil() as i32,
            bottom: clip_bottom.ceil() as i32,
        },
        default_color: text_backend::color(first_style.color),
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
    let Some(character) = glyph.character() else {
        log::debug!("skipping invalid icon codepoint: {:?}", glyph);
        return None;
    };

    let font_size = icon.size.max(1.0);
    let line_height = font_size;
    let mut buffer = glyphon::Buffer::new(font_system, glyphon::Metrics::relative(font_size, 1.0));
    let width = icon.rect.area.width().max(0.0);
    let height = icon.rect.area.height().max(0.0);
    let buffer_height = height.min(line_height);
    let attrs = text_backend::attrs_for_icon(glyph, font_size, icon.color);
    let text = character.to_string();

    buffer.set_size(font_system, Some(width), Some(buffer_height));
    buffer.set_rich_text(
        font_system,
        vec![(text.as_str(), attrs.clone())],
        &attrs,
        glyphon::Shaping::Basic,
        Some(glyphon::cosmic_text::Align::Center),
    );
    buffer.shape_until_scroll(font_system, false);

    let clip_left = icon.rect.origin.x() * scale_factor;
    let clip_top = icon.rect.origin.y() * scale_factor;
    let clip_right = clip_left + width * scale_factor;
    let clip_bottom = clip_top + height * scale_factor;
    let left = clip_left;
    let top = (icon.rect.origin.y() + (height - buffer_height).max(0.0) * 0.5) * scale_factor;

    Some(PreparedText {
        buffer,
        left,
        top,
        bounds: glyphon::TextBounds {
            left: clip_left.floor() as i32,
            top: clip_top.floor() as i32,
            right: clip_right.ceil() as i32,
            bottom: clip_bottom.ceil() as i32,
        },
        default_color: text_backend::color(icon.color),
    })
}
