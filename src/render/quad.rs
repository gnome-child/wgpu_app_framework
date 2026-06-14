use wgpu::util::DeviceExt;

use crate::geometry::{Rect, area, point};
use crate::paint;
use crate::render;
use crate::render::batch;

pub struct Batch {
    vertex_buffer: wgpu::Buffer,
    vertex_count: u32,
}

impl Batch {
    pub fn vertex_buffer(&self) -> &wgpu::Buffer {
        &self.vertex_buffer
    }

    pub fn vertex_count(&self) -> u32 {
        self.vertex_count
    }
}

pub fn pipeline(
    render_context: &render::Context,
    format: wgpu::TextureFormat,
) -> wgpu::RenderPipeline {
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
        })
}

pub fn prepare_batch(
    render_context: &render::Context,
    canvas: &render::Canvas,
    shapes: &[batch::Shape<'_>],
) -> Option<Batch> {
    let mut vertex_buf = Vec::new();
    for shape in shapes {
        push_shape_vertices(&mut vertex_buf, canvas, shape);
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

    Some(Batch {
        vertex_buffer,
        vertex_count,
    })
}

#[derive(Debug, Clone, Copy, PartialEq)]
struct SolidRect {
    rect: Rect,
    color: paint::Color,
}

fn push_shape_vertices(
    buffer: &mut Vec<render::primitive::Vertex>,
    canvas: &render::Canvas,
    shape: &batch::Shape<'_>,
) {
    let canvas_area = canvas.logical_area();

    if canvas_area.width() <= 0.0 || canvas_area.height() <= 0.0 {
        log::debug!("skipping shape draw for zero-size canvas");
        return;
    }

    for rect in solid_rects_for_shape(shape) {
        push_rect_vertices(buffer, canvas_area, rect);
    }
}

fn solid_rects_for_shape(shape: &batch::Shape<'_>) -> Vec<SolidRect> {
    match shape {
        batch::Shape::Quad(quad) => solid_rects_for_quad(quad),
        batch::Shape::Tint(tint) => solid_rects_for_tint(tint),
        batch::Shape::Outline(outline) => solid_rects_for_outline(outline),
        batch::Shape::BackdropBlur(blur) => {
            log::debug!("skipping unsupported backdrop blur: {blur:?}");
            Vec::new()
        }
    }
}

fn solid_rects_for_quad(quad: &paint::Quad) -> Vec<SolidRect> {
    let rect = quad.rect;
    let mut rects = Vec::new();

    if rect.radius != crate::geometry::rect::Radius::none() {
        log::debug!("drawing rounded quad as square because radius is not yet supported");
    }

    if let Some(fill) = quad.style.fill {
        match fill {
            paint::Fill::Brush(brush) => {
                if let Some(color) = solid_color(brush) {
                    rects.push(SolidRect { rect, color });
                }
            }
            paint::Fill::Blur => {
                log::debug!("skipping unsupported blur quad fill");
            }
        }
    }

    if let Some(stroke) = quad.style.stroke {
        if let Some(color) = solid_color(stroke.brush) {
            for rect in internal_stroke_rects(rect, stroke.width) {
                rects.push(SolidRect { rect, color });
            }
        }
    }

    if let Some(color) = quad.style.tint {
        rects.push(SolidRect { rect, color });
    }

    rects
}

fn solid_rects_for_tint(tint: &paint::Tint) -> Vec<SolidRect> {
    vec![SolidRect {
        rect: tint.rect,
        color: tint.color,
    }]
}

fn solid_rects_for_outline(outline: &paint::Outline) -> Vec<SolidRect> {
    let Some(color) = solid_color(outline.brush) else {
        return Vec::new();
    };

    external_outline_rects(outline.rect, outline.width, outline.offset)
        .into_iter()
        .map(|rect| SolidRect { rect, color })
        .collect()
}

fn push_rect_vertices(
    buffer: &mut Vec<render::primitive::Vertex>,
    canvas_area: area::Logical,
    rect: SolidRect,
) {
    let origin = rect.rect.origin;
    let area = rect.rect.area;

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

    let color = rect.color.to_array();

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

fn solid_color(brush: paint::Brush) -> Option<paint::Color> {
    match brush {
        paint::Brush::Solid(color) => Some(color),
        paint::Brush::Gradient { .. } => {
            log::debug!("skipping unsupported gradient brush");
            None
        }
    }
}

fn internal_stroke_rects(rect: Rect, width: f32) -> Vec<Rect> {
    let width = clamped_width(rect, width);
    if width <= 0.0 {
        return Vec::new();
    }

    let (x0, y0, x1, y1) = edges(rect);
    let mut rects = Vec::new();

    push_rect(&mut rects, x0, y0, x1, y0 + width);
    push_rect(&mut rects, x0, y1 - width, x1, y1);
    push_rect(&mut rects, x0, y0 + width, x0 + width, y1 - width);
    push_rect(&mut rects, x1 - width, y0 + width, x1, y1 - width);

    rects
}

fn external_outline_rects(rect: Rect, width: f32, offset: f32) -> Vec<Rect> {
    if width <= 0.0 {
        return Vec::new();
    }

    let offset = offset.max(0.0);
    let (x0, y0, x1, y1) = edges(rect);
    let inner_left = x0 - offset;
    let inner_top = y0 - offset;
    let inner_right = x1 + offset;
    let inner_bottom = y1 + offset;
    let outer_left = inner_left - width;
    let outer_top = inner_top - width;
    let outer_right = inner_right + width;
    let outer_bottom = inner_bottom + width;
    let mut rects = Vec::new();

    push_rect(&mut rects, outer_left, outer_top, outer_right, inner_top);
    push_rect(
        &mut rects,
        outer_left,
        inner_bottom,
        outer_right,
        outer_bottom,
    );
    push_rect(&mut rects, outer_left, inner_top, inner_left, inner_bottom);
    push_rect(
        &mut rects,
        inner_right,
        inner_top,
        outer_right,
        inner_bottom,
    );

    rects
}

fn clamped_width(rect: Rect, width: f32) -> f32 {
    width
        .max(0.0)
        .min(rect.area.width() / 2.0)
        .min(rect.area.height() / 2.0)
}

fn edges(rect: Rect) -> (f32, f32, f32, f32) {
    let x0 = rect.origin.x();
    let y0 = rect.origin.y();

    (x0, y0, x0 + rect.area.width(), y0 + rect.area.height())
}

fn push_rect(rects: &mut Vec<Rect>, left: f32, top: f32, right: f32, bottom: f32) {
    if right <= left || bottom <= top {
        return;
    }

    rects.push(Rect::new(
        point::logical(left, top),
        area::logical(right - left, bottom - top),
    ));
}

#[cfg(test)]
mod tests {
    use super::*;

    fn rect() -> Rect {
        Rect::new(point::logical(10.0, 20.0), area::logical(40.0, 30.0))
    }

    fn quad(style: paint::Style) -> paint::Quad {
        paint::Quad {
            rect: rect(),
            style,
        }
    }

    fn style(
        fill: Option<paint::Fill>,
        stroke: Option<paint::Stroke>,
        tint: Option<paint::Color>,
    ) -> paint::Style {
        paint::Style { fill, stroke, tint }
    }

    fn solid(color: paint::Color) -> paint::Fill {
        paint::Fill::Brush(paint::Brush::Solid(color))
    }

    fn stroke(width: f32) -> paint::Stroke {
        paint::Stroke {
            brush: paint::Brush::Solid(paint::Color::BLACK),
            width,
        }
    }

    fn vertex_count_for_quad(quad: &paint::Quad) -> usize {
        solid_rects_for_quad(quad).len() * 6
    }

    fn bounds(rects: &[Rect]) -> (f32, f32, f32, f32) {
        rects.iter().fold(
            (
                f32::INFINITY,
                f32::INFINITY,
                f32::NEG_INFINITY,
                f32::NEG_INFINITY,
            ),
            |bounds, rect| {
                let (left, top, right, bottom) = edges(*rect);

                (
                    bounds.0.min(left),
                    bounds.1.min(top),
                    bounds.2.max(right),
                    bounds.3.max(bottom),
                )
            },
        )
    }

    #[test]
    fn fill_only_quad_generates_one_rect() {
        let quad = quad(style(Some(solid(paint::Color::RED)), None, None));

        assert_eq!(vertex_count_for_quad(&quad), 6);
    }

    #[test]
    fn stroke_only_quad_generates_internal_border_rects() {
        let quad = quad(style(None, Some(stroke(2.0)), None));
        let rects = solid_rects_for_quad(&quad);

        assert_eq!(rects.len(), 4);
        assert_eq!(vertex_count_for_quad(&quad), 24);
        assert_eq!(
            bounds(&rects.iter().map(|rect| rect.rect).collect::<Vec<_>>()),
            edges(rect())
        );
    }

    #[test]
    fn tint_only_quad_generates_one_overlay_rect() {
        let quad = quad(style(
            None,
            None,
            Some(paint::Color::rgba(1.0, 1.0, 1.0, 0.25)),
        ));

        assert_eq!(vertex_count_for_quad(&quad), 6);
    }

    #[test]
    fn fill_stroke_and_tint_quad_preserves_rect_order() {
        let quad = quad(style(
            Some(solid(paint::Color::RED)),
            Some(stroke(2.0)),
            Some(paint::Color::rgba(1.0, 1.0, 1.0, 0.25)),
        ));
        let rects = solid_rects_for_quad(&quad);

        assert_eq!(rects.len(), 6);
        assert_eq!(vertex_count_for_quad(&quad), 36);
        assert_eq!(rects[0].color, paint::Color::RED);
        assert_eq!(rects[5].color, paint::Color::rgba(1.0, 1.0, 1.0, 0.25));
    }

    #[test]
    fn outline_item_expands_outside_rect_bounds() {
        let outline = paint::Outline {
            rect: rect(),
            brush: paint::Brush::Solid(paint::Color::RED),
            width: 2.0,
            offset: 3.0,
        };
        let rects = solid_rects_for_outline(&outline);

        assert_eq!(rects.len(), 4);
        assert_eq!(
            bounds(&rects.iter().map(|rect| rect.rect).collect::<Vec<_>>()),
            (5.0, 15.0, 55.0, 55.0)
        );
    }

    #[test]
    fn backdrop_blur_generates_no_solid_rects() {
        let blur = paint::Blur {
            rect: rect(),
            radius: 8.0,
        };

        assert!(solid_rects_for_shape(&batch::Shape::BackdropBlur(&blur)).is_empty());
    }
}
