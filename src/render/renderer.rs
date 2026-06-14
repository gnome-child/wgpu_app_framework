use crate::paint;
use crate::render;
use crate::text;
use wgpu::util::DeviceExt;

pub struct Renderer {
    quad_pipeline: wgpu::RenderPipeline,
    text_viewport: glyphon::Viewport,
    text_atlas: glyphon::TextAtlas,
    text_renderers: Vec<glyphon::TextRenderer>,
    font_system: glyphon::FontSystem,
    swash_cache: glyphon::SwashCache,
}

enum ItemBatch<'a> {
    Quads(Vec<&'a paint::Quad>),
    Texts(Vec<&'a paint::Text>),
}

enum RenderBatch {
    Quads {
        vertex_buffer: wgpu::Buffer,
        vertex_count: u32,
    },
    Text {
        renderer_index: usize,
    },
}

struct PreparedText {
    buffer: glyphon::Buffer,
    left: f32,
    top: f32,
    bounds: glyphon::TextBounds,
    default_color: glyphon::Color,
}

impl Renderer {
    pub fn new(render_context: &render::Context, format: wgpu::TextureFormat) -> Self {
        let shader = render_context
            .device()
            .create_shader_module(wgpu::include_wgsl!("quad.wgsl"));

        let pipeline_layout =
            render_context
                .device()
                .create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                    label: Some("Quad Pipeline Layout"),
                    bind_group_layouts: &[],
                    immediate_size: 0,
                });

        let quad_pipeline =
            render_context
                .device()
                .create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                    label: Some("Quad Pipeline"),
                    layout: Some(&pipeline_layout),
                    vertex: wgpu::VertexState {
                        module: &shader,
                        entry_point: Some("vs_main"),
                        buffers: &[render::primitive::Vertex::layout()],
                        compilation_options: Default::default(),
                    },
                    primitive: wgpu::PrimitiveState::default(),
                    depth_stencil: None,
                    multisample: wgpu::MultisampleState::default(),
                    fragment: Some(wgpu::FragmentState {
                        module: &shader,
                        entry_point: Some("fs_main"),
                        targets: &[Some(wgpu::ColorTargetState {
                            format,
                            blend: Some(wgpu::BlendState::ALPHA_BLENDING),
                            write_mask: wgpu::ColorWrites::ALL,
                        })],
                        compilation_options: Default::default(),
                    }),
                    multiview_mask: None,
                    cache: None,
                });

        let text_cache = glyphon::Cache::new(render_context.device());
        let text_viewport = glyphon::Viewport::new(render_context.device(), &text_cache);
        let text_atlas = glyphon::TextAtlas::new(
            render_context.device(),
            render_context.queue(),
            &text_cache,
            format,
        );

        Self {
            quad_pipeline,
            text_viewport,
            text_atlas,
            text_renderers: Vec::new(),
            font_system: glyphon::FontSystem::new(),
            swash_cache: glyphon::SwashCache::new(),
        }
    }

    pub fn clear(
        &mut self,
        render_context: &render::Context,
        canvas: &mut render::Canvas,
    ) -> render::Result<render::frame::Status> {
        let clear_color = canvas.color();

        canvas.draw(render_context, |encoder, frame| {
            let view = frame.create_view();

            let _pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Main Render Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    depth_slice: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(clear_color),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                occlusion_query_set: None,
                timestamp_writes: None,
                multiview_mask: None,
            });
        })
    }

    pub fn draw(
        &mut self,
        render_context: &render::Context,
        canvas: &mut render::Canvas,
        scene: &paint::Scene,
    ) -> render::Result<render::frame::Status> {
        let clear_color = scene
            .clear_color()
            .map(render::color_to_wgpu)
            .unwrap_or_else(|| canvas.color());
        let item_batches = item_batches(scene.items());
        let text_batch_count = item_batches
            .iter()
            .filter(|batch| matches!(batch, ItemBatch::Texts(_)))
            .count();

        if text_batch_count > 0 {
            self.update_text_viewport(render_context, canvas);
            self.ensure_text_renderers(text_batch_count, render_context);
        }

        let mut render_batches = Vec::with_capacity(item_batches.len());
        let mut text_renderer_index = 0;

        for batch in item_batches {
            match batch {
                ItemBatch::Quads(quads) => {
                    if let Some(batch) = self.prepare_quad_batch(render_context, canvas, &quads) {
                        render_batches.push(batch);
                    }
                }
                ItemBatch::Texts(texts) => {
                    if self.prepare_text_batch(
                        render_context,
                        canvas,
                        text_renderer_index,
                        &texts,
                    )? {
                        render_batches.push(RenderBatch::Text {
                            renderer_index: text_renderer_index,
                        });
                    }

                    text_renderer_index += 1;
                }
            }
        }

        let quad_pipeline = &self.quad_pipeline;
        let text_renderers = &self.text_renderers;
        let text_atlas = &self.text_atlas;
        let text_viewport = &self.text_viewport;
        let mut text_render_error = None;

        let status = canvas.draw(render_context, |encoder, frame| {
            let view = frame.create_view();

            let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Main Render Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    depth_slice: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(clear_color),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                occlusion_query_set: None,
                timestamp_writes: None,
                multiview_mask: None,
            });

            for batch in &render_batches {
                match batch {
                    RenderBatch::Quads {
                        vertex_buffer,
                        vertex_count,
                    } => {
                        pass.set_pipeline(quad_pipeline);
                        pass.set_vertex_buffer(0, vertex_buffer.slice(..));
                        pass.draw(0..*vertex_count, 0..1);
                    }
                    RenderBatch::Text { renderer_index } => {
                        if let Err(error) = text_renderers[*renderer_index].render(
                            text_atlas,
                            text_viewport,
                            &mut pass,
                        ) {
                            text_render_error = Some(error);
                            break;
                        }
                    }
                }
            }
        });

        if let Some(error) = text_render_error {
            return Err(error.into());
        }

        self.text_atlas.trim();

        status
    }

    fn prepare_quad_batch(
        &self,
        render_context: &render::Context,
        canvas: &render::Canvas,
        quads: &[&paint::Quad],
    ) -> Option<RenderBatch> {
        let mut vertex_buf = Vec::new();
        for quad in quads {
            push_quad_vertices(&mut vertex_buf, canvas, quad);
        }

        let vertex_count = vertex_buf.len() as u32;
        if vertex_count == 0 {
            return None;
        }

        let vertex_buffer =
            render_context
                .device()
                .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                    label: Some("Quad Vertex Buffer"),
                    contents: bytemuck::cast_slice(&vertex_buf),
                    usage: wgpu::BufferUsages::VERTEX,
                });

        Some(RenderBatch::Quads {
            vertex_buffer,
            vertex_count,
        })
    }

    fn update_text_viewport(&mut self, render_context: &render::Context, canvas: &render::Canvas) {
        let physical_area = canvas.physical_area();
        self.text_viewport.update(
            render_context.queue(),
            glyphon::Resolution {
                width: physical_area.width(),
                height: physical_area.height(),
            },
        );
    }

    fn ensure_text_renderers(&mut self, count: usize, render_context: &render::Context) {
        while self.text_renderers.len() < count {
            self.text_renderers.push(glyphon::TextRenderer::new(
                &mut self.text_atlas,
                render_context.device(),
                wgpu::MultisampleState::default(),
                None,
            ));
        }
    }

    fn prepare_text_batch(
        &mut self,
        render_context: &render::Context,
        canvas: &render::Canvas,
        renderer_index: usize,
        texts: &[&paint::Text],
    ) -> render::Result<bool> {
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

        self.text_renderers[renderer_index].prepare(
            render_context.device(),
            render_context.queue(),
            &mut self.font_system,
            &mut self.text_atlas,
            &self.text_viewport,
            text_areas,
            &mut self.swash_cache,
        )?;

        Ok(true)
    }
}

fn item_batches(items: &[paint::Item]) -> Vec<ItemBatch<'_>> {
    let mut batches = Vec::new();

    for item in items {
        match item {
            paint::Item::Quad(quad) => match batches.last_mut() {
                Some(ItemBatch::Quads(quads)) => quads.push(quad),
                _ => batches.push(ItemBatch::Quads(vec![quad])),
            },
            paint::Item::Text(text) => match batches.last_mut() {
                Some(ItemBatch::Texts(texts)) => texts.push(text),
                _ => batches.push(ItemBatch::Texts(vec![text])),
            },
        }
    }

    batches
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

fn push_quad_vertices(
    buffer: &mut Vec<render::primitive::Vertex>,
    canvas: &render::Canvas,
    quad: &paint::Quad,
) {
    let Some(fill) = quad.style.fill else {
        return;
    };

    let paint::Fill::Brush(brush) = fill else {
        log::debug!("skipping unsupported non-brush quad fill");
        return;
    };

    let paint::Brush::Solid(color) = brush else {
        log::debug!("skipping unsupported non-solid quad brush");
        return;
    };

    let rect = quad.rect;

    if quad.style.stroke.is_some() {
        log::debug!("skipping unsupported quad stroke");
    }

    if quad.style.tint.is_some() {
        log::debug!("skipping unsupported quad tint");
    }

    if rect.radius != crate::geometry::rect::Radius::none() {
        log::debug!("skipping unsupported quad radius");
    }

    let origin = rect.origin;
    let area = rect.area;
    let canvas_area = canvas.logical_area();

    if canvas_area.width() <= 0.0 || canvas_area.height() <= 0.0 {
        log::debug!("skipping quad draw for zero-size canvas");
        return;
    }

    let x0 = origin.x();
    let y0 = origin.y();
    let x1 = x0 + area.width();
    let y1 = y0 + area.height();

    let to_clip = |x: f32, y: f32| -> [f32; 2] {
        [
            (x / canvas_area.width()) * 2.0 - 1.0,
            1.0 - (y / canvas_area.height()) * 2.0,
        ]
    };

    let color = color.to_array();

    buffer.extend_from_slice(&[
        render::primitive::Vertex {
            position: to_clip(x0, y0),
            color,
        },
        render::primitive::Vertex {
            position: to_clip(x0, y1),
            color,
        },
        render::primitive::Vertex {
            position: to_clip(x1, y1),
            color,
        },
        render::primitive::Vertex {
            position: to_clip(x0, y0),
            color,
        },
        render::primitive::Vertex {
            position: to_clip(x1, y1),
            color,
        },
        render::primitive::Vertex {
            position: to_clip(x1, y0),
            color,
        },
    ]);
}

#[cfg(test)]
mod tests {
    use crate::geometry::{Rect, area, point};
    use crate::{paint, text};

    use super::*;

    #[derive(Debug, PartialEq, Eq)]
    enum Kind {
        Quads(usize),
        Texts(usize),
    }

    fn solid_quad(x: f32) -> paint::Quad {
        paint::Quad {
            rect: Rect::new(point::logical(x, 0.0), area::logical(10.0, 10.0)),
            style: paint::Style {
                fill: Some(paint::Fill::Brush(paint::Brush::Solid(paint::Color::RED))),
                stroke: None,
                tint: None,
            },
        }
    }

    fn label(x: f32) -> paint::Text {
        paint::Text {
            rect: Rect::new(point::logical(x, 0.0), area::logical(10.0, 10.0)),
            document: text::Document::plain("Label"),
        }
    }

    fn kinds(batches: &[ItemBatch<'_>]) -> Vec<Kind> {
        batches
            .iter()
            .map(|batch| match batch {
                ItemBatch::Quads(quads) => Kind::Quads(quads.len()),
                ItemBatch::Texts(texts) => Kind::Texts(texts.len()),
            })
            .collect()
    }

    #[test]
    fn item_batches_preserve_mixed_render_order() {
        let items = vec![
            paint::Item::Quad(solid_quad(0.0)),
            paint::Item::Quad(solid_quad(1.0)),
            paint::Item::Text(label(2.0)),
            paint::Item::Quad(solid_quad(3.0)),
        ];

        assert_eq!(
            kinds(&item_batches(&items)),
            vec![Kind::Quads(2), Kind::Texts(1), Kind::Quads(1)]
        );
    }
}
