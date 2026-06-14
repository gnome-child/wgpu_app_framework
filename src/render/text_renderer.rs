use crate::paint;
use crate::render;
use crate::text;

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
            font_system: glyphon::FontSystem::new(),
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
        texts: &[&paint::Text],
    ) -> Result<bool> {
        if texts.is_empty() {
            return Ok(false);
        }

        let scale_factor = canvas.scale_factor();
        let mut prepared = Vec::with_capacity(texts.len());

        for text in texts {
            if let Some(text) = prepare_text(&mut self.font_system, text, scale_factor) {
                prepared.push(text);
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
        .map(|run| (run.text(), attrs_for_style(run.style())))
        .collect::<Vec<_>>();

    let default_attrs = attrs_for_style(first_style);
    buffer.set_rich_text(
        font_system,
        spans,
        &default_attrs,
        glyphon::Shaping::Advanced,
        Some(align(block.align())),
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
        default_color: color(first_style.color),
    })
}

fn attrs_for_style(style: text::Style) -> glyphon::Attrs<'static> {
    glyphon::Attrs::new()
        .family(glyphon::Family::SansSerif)
        .weight(weight(style.weight))
        .color(color(style.color))
        .metrics(glyphon::Metrics::relative(style.size.max(1.0), 1.25))
}

fn align(align: text::Align) -> glyphon::cosmic_text::Align {
    match align {
        text::Align::Start => glyphon::cosmic_text::Align::Left,
        text::Align::Center => glyphon::cosmic_text::Align::Center,
        text::Align::End => glyphon::cosmic_text::Align::Right,
    }
}

fn weight(weight: text::Weight) -> glyphon::Weight {
    match weight {
        text::Weight::Normal => glyphon::Weight::NORMAL,
        text::Weight::Medium => glyphon::Weight::MEDIUM,
        text::Weight::Bold => glyphon::Weight::BOLD,
    }
}

fn color(color: paint::Color) -> glyphon::Color {
    let channel = |value: f32| (value.clamp(0.0, 1.0) * 255.0).round() as u8;

    glyphon::Color::rgba(
        channel(color.r),
        channel(color.g),
        channel(color.b),
        channel(color.a),
    )
}
