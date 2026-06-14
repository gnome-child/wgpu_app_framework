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
struct AnalyticShape {
    outer_rect: Rect,
    outer_radius: crate::geometry::rect::ResolvedRadius,
    inner: Option<AnalyticInner>,
    color: paint::Color,
}

#[derive(Debug, Clone, Copy, PartialEq)]
struct AnalyticInner {
    rect: Rect,
    radius: crate::geometry::rect::ResolvedRadius,
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

    for shape in analytic_shapes_for_shape(shape) {
        push_analytic_shape_vertices(buffer, canvas_area, shape);
    }
}

fn analytic_shapes_for_shape(shape: &batch::Shape<'_>) -> Vec<AnalyticShape> {
    match shape {
        batch::Shape::Quad(quad) => analytic_shapes_for_quad(quad),
        batch::Shape::Tint(tint) => analytic_shapes_for_tint(tint),
        batch::Shape::Outline(outline) => analytic_shapes_for_outline(outline),
        batch::Shape::BackdropBlur(blur) => {
            log::debug!("skipping unsupported backdrop blur: {blur:?}");
            Vec::new()
        }
    }
}

fn analytic_shapes_for_quad(quad: &paint::Quad) -> Vec<AnalyticShape> {
    let rect = quad.rect;
    let mut shapes = Vec::new();

    if let Some(fill) = quad.style.fill {
        match fill {
            paint::Fill::Brush(brush) => {
                if let Some(color) = solid_color(brush) {
                    shapes.push(fill_shape(rect, color));
                }
            }
            paint::Fill::Blur => {
                log::debug!("skipping unsupported blur quad fill");
            }
        }
    }

    if let Some(stroke) = quad.style.stroke {
        if let Some(color) = solid_color(stroke.brush) {
            if let Some(shape) = internal_stroke_shape(rect, stroke.width, color) {
                shapes.push(shape);
            }
        }
    }

    if let Some(color) = quad.style.tint {
        shapes.push(fill_shape(rect, color));
    }

    shapes
}

fn analytic_shapes_for_tint(tint: &paint::Tint) -> Vec<AnalyticShape> {
    vec![fill_shape(tint.rect, tint.color)]
}

fn analytic_shapes_for_outline(outline: &paint::Outline) -> Vec<AnalyticShape> {
    let Some(color) = solid_color(outline.brush) else {
        return Vec::new();
    };

    external_outline_shape(outline.rect, outline.width, outline.offset, color)
        .into_iter()
        .collect()
}

fn push_analytic_shape_vertices(
    buffer: &mut Vec<render::primitive::Vertex>,
    canvas_area: area::Logical,
    shape: AnalyticShape,
) {
    let to_clip = |x: f32, y: f32| -> [f32; 2] {
        [
            (x / canvas_area.width()) * 2.0 - 1.0,
            1.0 - (y / canvas_area.height()) * 2.0,
        ]
    };

    if shape.outer_rect.area.width() <= 0.0 || shape.outer_rect.area.height() <= 0.0 {
        return;
    }

    let (x0, y0, x1, y1) = edges(shape.outer_rect);
    let outer_rect = rect_data(shape.outer_rect);
    let outer_radius = radius_data(shape.outer_radius);
    let (inner_rect, inner_radius, mode) = match shape.inner {
        Some(inner) => (rect_data(inner.rect), radius_data(inner.radius), 1.0),
        None => ([0.0, 0.0, -1.0, -1.0], [0.0; 4], 0.0),
    };
    let color = shape.color.to_array();
    let params = [mode, 0.0, 0.0, 0.0];

    let mut push = |x: f32, y: f32| {
        buffer.push(render::primitive::Vertex {
            position: to_clip(x, y),
            local_position: [x, y],
            outer_rect,
            outer_radius,
            inner_rect,
            inner_radius,
            color,
            params,
        });
    };

    push(x0, y0);
    push(x0, y1);
    push(x1, y1);
    push(x0, y0);
    push(x1, y1);
    push(x1, y0);
}

fn fill_shape(rect: Rect, color: paint::Color) -> AnalyticShape {
    AnalyticShape {
        outer_rect: rect,
        outer_radius: rect.radius.resolve(rect.area),
        inner: None,
        color,
    }
}

fn internal_stroke_shape(rect: Rect, width: f32, color: paint::Color) -> Option<AnalyticShape> {
    let width = clamped_width(rect, width);
    if width <= 0.0 {
        return None;
    }

    let outer_radius = rect.radius.resolve(rect.area);
    let Some(inner_rect) = inset_rect(rect, width) else {
        return Some(fill_shape(rect, color));
    };

    Some(AnalyticShape {
        outer_rect: rect,
        outer_radius,
        inner: Some(AnalyticInner {
            rect: inner_rect,
            radius: shrink_radius(outer_radius, width),
        }),
        color,
    })
}

fn external_outline_shape(
    rect: Rect,
    width: f32,
    offset: f32,
    color: paint::Color,
) -> Option<AnalyticShape> {
    if width <= 0.0 {
        return None;
    }

    let offset = offset.max(0.0);
    let base_radius = rect.radius.resolve(rect.area);
    let inner_rect = expand_rect(rect, offset);
    let outer_rect = expand_rect(rect, offset + width);
    Some(AnalyticShape {
        outer_rect,
        outer_radius: expand_radius(base_radius, offset + width),
        inner: Some(AnalyticInner {
            rect: inner_rect,
            radius: expand_radius(base_radius, offset),
        }),
        color,
    })
}

fn inset_rect(rect: Rect, inset: f32) -> Option<Rect> {
    let width = rect.area.width() - inset * 2.0;
    let height = rect.area.height() - inset * 2.0;

    if width <= 0.0 || height <= 0.0 {
        return None;
    }

    Some(Rect::new(
        point::logical(rect.origin.x() + inset, rect.origin.y() + inset),
        area::logical(width, height),
    ))
}

fn expand_rect(rect: Rect, amount: f32) -> Rect {
    Rect::new(
        point::logical(rect.origin.x() - amount, rect.origin.y() - amount),
        area::logical(
            rect.area.width() + amount * 2.0,
            rect.area.height() + amount * 2.0,
        ),
    )
}

fn expand_radius(
    radius: crate::geometry::rect::ResolvedRadius,
    amount: f32,
) -> crate::geometry::rect::ResolvedRadius {
    crate::geometry::rect::ResolvedRadius {
        top_left: expand_corner_radius(radius.top_left, amount),
        top_right: expand_corner_radius(radius.top_right, amount),
        bottom_left: expand_corner_radius(radius.bottom_left, amount),
        bottom_right: expand_corner_radius(radius.bottom_right, amount),
    }
}

fn expand_corner_radius(radius: f32, amount: f32) -> f32 {
    if radius <= 0.0 { 0.0 } else { radius + amount }
}

fn shrink_radius(
    radius: crate::geometry::rect::ResolvedRadius,
    amount: f32,
) -> crate::geometry::rect::ResolvedRadius {
    crate::geometry::rect::ResolvedRadius {
        top_left: (radius.top_left - amount).max(0.0),
        top_right: (radius.top_right - amount).max(0.0),
        bottom_left: (radius.bottom_left - amount).max(0.0),
        bottom_right: (radius.bottom_right - amount).max(0.0),
    }
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

fn rect_data(rect: Rect) -> [f32; 4] {
    [
        rect.origin.x(),
        rect.origin.y(),
        rect.area.width(),
        rect.area.height(),
    ]
}

fn radius_data(radius: crate::geometry::rect::ResolvedRadius) -> [f32; 4] {
    [
        radius.top_left,
        radius.top_right,
        radius.bottom_right,
        radius.bottom_left,
    ]
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

    fn vertex_count_for_shape(shape: AnalyticShape) -> usize {
        let mut vertices = Vec::new();

        push_analytic_shape_vertices(&mut vertices, area::logical(100.0, 100.0), shape);

        vertices.len()
    }

    fn bounds(shapes: &[AnalyticShape]) -> (f32, f32, f32, f32) {
        shapes.iter().fold(
            (
                f32::INFINITY,
                f32::INFINITY,
                f32::NEG_INFINITY,
                f32::NEG_INFINITY,
            ),
            |bounds, shape| {
                let (x0, y0, x1, y1) = edges(shape.outer_rect);

                (
                    bounds.0.min(x0),
                    bounds.1.min(y0),
                    bounds.2.max(x1),
                    bounds.3.max(y1),
                )
            },
        )
    }

    #[test]
    fn fill_only_quad_lowers_to_one_analytic_shape() {
        let quad = quad(style(Some(solid(paint::Color::RED)), None, None));
        let shapes = analytic_shapes_for_quad(&quad);

        assert_eq!(shapes.len(), 1);
        assert_eq!(shapes[0].outer_rect, rect());
        assert_eq!(shapes[0].outer_radius, rect().radius.resolve(rect().area));
        assert_eq!(shapes[0].inner, None);
        assert_eq!(shapes[0].color, paint::Color::RED);
        assert_eq!(vertex_count_for_shape(shapes[0]), 6);
    }

    #[test]
    fn stroke_only_quad_lowers_to_internal_ring() {
        let quad = quad(style(None, Some(stroke(2.0)), None));
        let shapes = analytic_shapes_for_quad(&quad);
        let inner = shapes[0].inner.expect("stroke should be a ring");

        assert_eq!(shapes.len(), 1);
        assert_eq!(bounds(&shapes), edges(rect()));
        assert_eq!(
            inner.rect,
            Rect::new(point::logical(12.0, 22.0), area::logical(36.0, 26.0))
        );
        assert_eq!(vertex_count_for_shape(shapes[0]), 6);
    }

    #[test]
    fn tint_only_quad_lowers_to_one_overlay_shape() {
        let quad = quad(style(
            None,
            None,
            Some(paint::Color::rgba(1.0, 1.0, 1.0, 0.25)),
        ));
        let shapes = analytic_shapes_for_quad(&quad);

        assert_eq!(shapes.len(), 1);
        assert_eq!(shapes[0].outer_rect, rect());
        assert_eq!(shapes[0].color, paint::Color::rgba(1.0, 1.0, 1.0, 0.25));
    }

    #[test]
    fn fill_stroke_and_tint_quad_preserves_shape_order() {
        let quad = quad(style(
            Some(solid(paint::Color::RED)),
            Some(stroke(2.0)),
            Some(paint::Color::rgba(1.0, 1.0, 1.0, 0.25)),
        ));
        let shapes = analytic_shapes_for_quad(&quad);

        assert_eq!(shapes.len(), 3);
        assert_eq!(shapes[0].color, paint::Color::RED);
        assert!(shapes[1].inner.is_some());
        assert_eq!(shapes[2].color, paint::Color::rgba(1.0, 1.0, 1.0, 0.25));
    }

    #[test]
    fn outline_item_expands_outside_rect_bounds() {
        let outline = paint::Outline {
            rect: rect(),
            brush: paint::Brush::Solid(paint::Color::RED),
            width: 2.0,
            offset: 3.0,
        };
        let shapes = analytic_shapes_for_outline(&outline);
        let inner = shapes[0].inner.expect("outline should be a ring");

        assert_eq!(shapes.len(), 1);
        assert_eq!(bounds(&shapes), (5.0, 15.0, 55.0, 55.0));
        assert_eq!(
            inner.rect,
            Rect::new(point::logical(7.0, 17.0), area::logical(46.0, 36.0))
        );
    }

    #[test]
    fn rounded_outline_lowers_to_expanded_rounded_ring() {
        let rect = Rect::rounded(
            point::logical(10.0, 20.0),
            area::logical(80.0, 30.0),
            crate::geometry::rect::Radius::splat(1.0),
        );
        let outline = paint::Outline {
            rect,
            brush: paint::Brush::Solid(paint::Color::RED),
            width: 4.0,
            offset: 2.0,
        };
        let shapes = analytic_shapes_for_outline(&outline);
        let inner = shapes[0].inner.expect("outline should be a ring");

        assert_eq!(shapes.len(), 1);
        assert_eq!(vertex_count_for_shape(shapes[0]), 6);
        assert_eq!(bounds(&shapes), (4.0, 14.0, 96.0, 56.0));
        assert_eq!(shapes[0].outer_radius.top_left, 21.0);
        assert_eq!(inner.radius.top_left, 17.0);
    }

    #[test]
    fn rounded_fill_lowers_to_one_analytic_shape() {
        let quad = paint::Quad {
            rect: Rect::rounded(
                point::logical(10.0, 20.0),
                area::logical(40.0, 40.0),
                crate::geometry::rect::Radius::splat(1.0),
            ),
            style: style(Some(solid(paint::Color::RED)), None, None),
        };
        let shapes = analytic_shapes_for_quad(&quad);

        assert_eq!(shapes.len(), 1);
        assert_eq!(vertex_count_for_shape(shapes[0]), 6);
        assert_eq!(bounds(&shapes), edges(quad.rect));
        assert_eq!(shapes[0].outer_radius.top_left, 20.0);
    }

    #[test]
    fn rounded_tint_uses_same_shape_as_rounded_fill() {
        let rect = Rect::rounded(
            point::logical(10.0, 20.0),
            area::logical(80.0, 30.0),
            crate::geometry::rect::Radius::splat(1.0),
        );
        let fill = fill_shape(rect, paint::Color::RED);
        let tint = paint::Tint {
            rect,
            color: paint::Color::rgba(1.0, 1.0, 1.0, 0.25),
        };
        let shapes = analytic_shapes_for_tint(&tint);

        assert_eq!(shapes.len(), 1);
        assert_eq!(shapes[0].outer_rect, fill.outer_rect);
        assert_eq!(shapes[0].outer_radius, fill.outer_radius);
        assert_eq!(bounds(&shapes), edges(rect));
    }

    #[test]
    fn rounded_internal_stroke_stays_inside_rect_bounds() {
        let quad = paint::Quad {
            rect: Rect::rounded(
                point::logical(10.0, 20.0),
                area::logical(80.0, 30.0),
                crate::geometry::rect::Radius::splat(1.0),
            ),
            style: style(None, Some(stroke(4.0)), None),
        };
        let shapes = analytic_shapes_for_quad(&quad);
        let inner = shapes[0].inner.expect("stroke should be a ring");

        assert_eq!(shapes.len(), 1);
        assert_eq!(bounds(&shapes), edges(quad.rect));
        assert_eq!(
            inner.rect,
            Rect::new(point::logical(14.0, 24.0), area::logical(72.0, 22.0))
        );
        assert_eq!(shapes[0].outer_radius.top_left, 15.0);
        assert_eq!(inner.radius.top_left, 11.0);
    }

    #[test]
    fn backdrop_blur_generates_no_analytic_shapes() {
        let blur = paint::Blur {
            rect: rect(),
            radius: 8.0,
        };

        assert!(analytic_shapes_for_shape(&batch::Shape::BackdropBlur(&blur)).is_empty());
    }
}
