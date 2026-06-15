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
    kind: AnalyticShapeKind,
    outer_rect: Rect,
    outer_rounding: [f32; 4],
    inner: Option<AnalyticInner>,
    brush: paint::Brush,
    blur: f32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum AnalyticShapeKind {
    Fill,
    InternalRing,
    ExternalRing,
    Shadow,
}

#[derive(Debug, Clone, Copy, PartialEq)]
struct AnalyticInner {
    rect: Rect,
    rounding: [f32; 4],
}

#[derive(Debug, Clone, Copy, PartialEq)]
struct PreparedShape {
    kind: AnalyticShapeKind,
    raster_rect: Rect,
    outer_rect: Rect,
    outer_rounding: [f32; 4],
    inner: Option<AnalyticInner>,
    brush: paint::Brush,
    blur: f32,
}

#[derive(Debug, Clone, Copy, PartialEq)]
struct PreparedBrush {
    from: paint::Color,
    to: paint::Color,
    points: [f32; 4],
    kind: f32,
}

#[derive(Debug, Clone, Copy, PartialEq)]
struct PixelGeometry {
    scale_factor: f32,
    logical_pixel: f32,
}

impl PixelGeometry {
    fn new(scale_factor: f32) -> Self {
        let scale_factor = scale_factor.max(0.0001);

        Self {
            scale_factor,
            logical_pixel: 1.0 / scale_factor,
        }
    }

    fn logical_pixel(self) -> f32 {
        self.logical_pixel
    }

    fn snap_distance(self, distance: f32) -> f32 {
        if distance <= 0.0 {
            return 0.0;
        }

        (distance * self.scale_factor).round().max(1.0) / self.scale_factor
    }

    fn snap_rect(self, rect: Rect) -> Rect {
        let (left, top, right, bottom) = edges(rect);
        let left = self.snap_position(left);
        let top = self.snap_position(top);
        let mut right = self.snap_position(right);
        let mut bottom = self.snap_position(bottom);

        if right <= left {
            right = left + self.logical_pixel;
        }

        if bottom <= top {
            bottom = top + self.logical_pixel;
        }

        Rect::rounded(
            point::logical(left, top),
            area::logical(right - left, bottom - top),
            rect.rounding,
        )
    }

    fn snap_position(self, position: f32) -> f32 {
        (position * self.scale_factor).round() / self.scale_factor
    }
}

fn push_shape_vertices(
    buffer: &mut Vec<render::primitive::Vertex>,
    canvas: &render::Canvas,
    shape: &batch::Shape<'_>,
) {
    let canvas_area = canvas.logical_area();
    let pixel_geometry = PixelGeometry::new(canvas.scale_factor());

    if canvas_area.width() <= 0.0 || canvas_area.height() <= 0.0 {
        log::debug!("skipping shape draw for zero-size canvas");
        return;
    }

    for shape in analytic_shapes_for_shape(shape) {
        push_analytic_shape_vertices(buffer, canvas_area, pixel_geometry, shape);
    }
}

fn analytic_shapes_for_shape(shape: &batch::Shape<'_>) -> Vec<AnalyticShape> {
    match shape {
        batch::Shape::Quad(quad) => analytic_shapes_for_quad(quad),
        batch::Shape::Shadow(shadow) => analytic_shapes_for_shadow(shadow),
        batch::Shape::Tint(tint) => analytic_shapes_for_tint(tint),
        batch::Shape::Outline(outline) => analytic_shapes_for_outline(outline),
    }
}

fn analytic_shapes_for_quad(quad: &paint::Quad) -> Vec<AnalyticShape> {
    let rect = quad.rect;
    let mut shapes = Vec::new();

    if let Some(fill) = quad.style.fill {
        match fill {
            paint::Fill::Brush(brush) => {
                shapes.push(fill_shape(rect, brush));
            }
        }
    }

    if let Some(stroke) = quad.style.stroke {
        if let Some(shape) = internal_stroke_shape(rect, stroke.width, stroke.brush) {
            shapes.push(shape);
        }
    }

    if let Some(brush) = quad.style.tint {
        shapes.push(fill_shape(rect, brush));
    }

    shapes
}

fn analytic_shapes_for_tint(tint: &paint::Tint) -> Vec<AnalyticShape> {
    vec![fill_shape(tint.rect, tint.brush)]
}

fn analytic_shapes_for_shadow(shadow: &paint::Shadow) -> Vec<AnalyticShape> {
    shadow_shape(shadow).into_iter().collect()
}

fn analytic_shapes_for_outline(outline: &paint::Outline) -> Vec<AnalyticShape> {
    external_outline_shape(outline.rect, outline.width, outline.offset, outline.brush)
        .into_iter()
        .collect()
}

fn push_analytic_shape_vertices(
    buffer: &mut Vec<render::primitive::Vertex>,
    canvas_area: area::Logical,
    pixel_geometry: PixelGeometry,
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

    let Some(shape) = prepare_shape(shape, pixel_geometry) else {
        return;
    };

    let (x0, y0, x1, y1) = edges(shape.raster_rect);
    let outer_rect = rect_data(shape.outer_rect);
    let outer_rounding = rounding_data(shape.outer_rounding);
    let (inner_rect, inner_rounding, mode) = match shape.inner {
        Some(inner) => (rect_data(inner.rect), rounding_data(inner.rounding), 1.0),
        None => ([0.0, 0.0, -1.0, -1.0], [0.0; 4], 0.0),
    };
    let brush = brush_data(shape.brush);
    let params = match shape.kind {
        AnalyticShapeKind::Shadow => [2.0, shape.blur, brush.kind, 0.0],
        _ => [mode, 0.0, brush.kind, 0.0],
    };

    let mut push = |x: f32, y: f32| {
        buffer.push(render::primitive::Vertex {
            position: to_clip(x, y),
            local_position: [x, y],
            outer_rect,
            outer_rounding,
            inner_rect,
            inner_rounding,
            color: brush.from.to_array(),
            color_to: brush.to.to_array(),
            brush_points: brush.points,
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

fn prepare_shape(shape: AnalyticShape, pixel_geometry: PixelGeometry) -> Option<PreparedShape> {
    match shape.kind {
        AnalyticShapeKind::Fill => prepare_fill_shape(shape, pixel_geometry),
        AnalyticShapeKind::InternalRing => prepare_internal_ring_shape(shape, pixel_geometry),
        AnalyticShapeKind::ExternalRing => prepare_external_ring_shape(shape, pixel_geometry),
        AnalyticShapeKind::Shadow => prepare_shadow_shape(shape, pixel_geometry),
    }
}

fn prepare_fill_shape(
    shape: AnalyticShape,
    pixel_geometry: PixelGeometry,
) -> Option<PreparedShape> {
    let outer_rect = pixel_geometry.snap_rect(shape.outer_rect);
    let raster_rect = expand_rect(outer_rect, pixel_geometry.logical_pixel());

    Some(PreparedShape {
        kind: shape.kind,
        raster_rect,
        outer_rect,
        outer_rounding: outer_rect.rounding.resolve(outer_rect.area),
        inner: None,
        brush: shape.brush,
        blur: shape.blur,
    })
}

fn prepare_internal_ring_shape(
    shape: AnalyticShape,
    pixel_geometry: PixelGeometry,
) -> Option<PreparedShape> {
    let outer_rect = pixel_geometry.snap_rect(shape.outer_rect);
    let width = pixel_geometry.snap_distance(ring_width(shape)?);
    let Some(inner_rect) = inset_rect(outer_rect, width) else {
        return prepare_fill_shape(fill_shape(outer_rect, shape.brush), pixel_geometry);
    };
    let outer_rounding = outer_rect.rounding.resolve(outer_rect.area);
    let inner = AnalyticInner {
        rect: inner_rect,
        rounding: shrink_rounding(outer_rounding, width),
    };
    let raster_rect = expand_rect(outer_rect, pixel_geometry.logical_pixel());

    Some(PreparedShape {
        kind: shape.kind,
        raster_rect,
        outer_rect,
        outer_rounding,
        inner: Some(inner),
        brush: shape.brush,
        blur: shape.blur,
    })
}

fn prepare_external_ring_shape(
    shape: AnalyticShape,
    pixel_geometry: PixelGeometry,
) -> Option<PreparedShape> {
    let inner = shape.inner?;
    let width = pixel_geometry.snap_distance(ring_width(shape)?);
    let inner_rect = pixel_geometry.snap_rect(inner.rect);
    let outer_rect = expand_rect(inner_rect, width);
    let inner_rounding = inner.rounding;
    let outer_rounding = expand_rounding(inner_rounding, width);
    let raster_rect = expand_rect(outer_rect, pixel_geometry.logical_pixel());

    Some(PreparedShape {
        kind: shape.kind,
        raster_rect,
        outer_rect,
        outer_rounding,
        inner: Some(AnalyticInner {
            rect: inner_rect,
            rounding: inner_rounding,
        }),
        brush: shape.brush,
        blur: shape.blur,
    })
}

fn prepare_shadow_shape(
    shape: AnalyticShape,
    pixel_geometry: PixelGeometry,
) -> Option<PreparedShape> {
    let inner = shape.inner?;
    let outer_rect = pixel_geometry.snap_rect(shape.outer_rect);
    let inner_rect = pixel_geometry.snap_rect(inner.rect);
    let blur = pixel_geometry.snap_distance(shape.blur);
    let raster_rect = union_rects(
        expand_rect(outer_rect, blur + pixel_geometry.logical_pixel()),
        expand_rect(inner_rect, pixel_geometry.logical_pixel()),
    );

    Some(PreparedShape {
        kind: shape.kind,
        raster_rect,
        outer_rect,
        outer_rounding: shape.outer_rounding,
        inner: Some(AnalyticInner {
            rect: inner_rect,
            rounding: inner.rounding,
        }),
        brush: shape.brush,
        blur,
    })
}

fn fill_shape(rect: Rect, brush: paint::Brush) -> AnalyticShape {
    AnalyticShape {
        kind: AnalyticShapeKind::Fill,
        outer_rect: rect,
        outer_rounding: rect.rounding.resolve(rect.area),
        inner: None,
        brush,
        blur: 0.0,
    }
}

fn internal_stroke_shape(rect: Rect, width: f32, brush: paint::Brush) -> Option<AnalyticShape> {
    let width = clamped_width(rect, width);
    if width <= 0.0 {
        return None;
    }

    let outer_rounding = rect.rounding.resolve(rect.area);
    let Some(inner_rect) = inset_rect(rect, width) else {
        return Some(fill_shape(rect, brush));
    };

    Some(AnalyticShape {
        kind: AnalyticShapeKind::InternalRing,
        outer_rect: rect,
        outer_rounding,
        inner: Some(AnalyticInner {
            rect: inner_rect,
            rounding: shrink_rounding(outer_rounding, width),
        }),
        brush,
        blur: 0.0,
    })
}

fn shadow_shape(shadow: &paint::Shadow) -> Option<AnalyticShape> {
    if !shadow.brush.is_visible() {
        return None;
    }

    let rect = shadow.rect;
    let spread = shadow.spread.max(0.0);
    let base_rounding = rect.rounding.resolve(rect.area);
    let caster_rect = offset_rect(expand_rect(rect, spread), shadow.offset);

    Some(AnalyticShape {
        kind: AnalyticShapeKind::Shadow,
        outer_rect: caster_rect,
        outer_rounding: expand_rounding(base_rounding, spread),
        inner: Some(AnalyticInner {
            rect,
            rounding: base_rounding,
        }),
        brush: shadow.brush,
        blur: shadow.blur.max(0.0),
    })
}

fn external_outline_shape(
    rect: Rect,
    width: f32,
    offset: f32,
    brush: paint::Brush,
) -> Option<AnalyticShape> {
    if width <= 0.0 {
        return None;
    }

    let offset = offset.max(0.0);
    let base_rounding = rect.rounding.resolve(rect.area);
    let inner_rect = expand_rect(rect, offset);
    let outer_rect = expand_rect(rect, offset + width);
    Some(AnalyticShape {
        kind: AnalyticShapeKind::ExternalRing,
        outer_rect,
        outer_rounding: expand_rounding(base_rounding, offset + width),
        inner: Some(AnalyticInner {
            rect: inner_rect,
            rounding: expand_rounding(base_rounding, offset),
        }),
        brush,
        blur: 0.0,
    })
}

fn ring_width(shape: AnalyticShape) -> Option<f32> {
    let inner = shape.inner?;
    let (outer_left, outer_top, outer_right, outer_bottom) = edges(shape.outer_rect);
    let (inner_left, inner_top, inner_right, inner_bottom) = edges(inner.rect);
    let widths = [
        inner_left - outer_left,
        inner_top - outer_top,
        outer_right - inner_right,
        outer_bottom - inner_bottom,
    ];

    widths
        .into_iter()
        .filter(|width| *width > 0.0)
        .min_by(|a, b| a.total_cmp(b))
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

fn offset_rect(rect: Rect, offset: point::Logical) -> Rect {
    Rect::rounded(
        point::logical(rect.origin.x() + offset.x(), rect.origin.y() + offset.y()),
        rect.area,
        rect.rounding,
    )
}

fn union_rects(a: Rect, b: Rect) -> Rect {
    let (a_left, a_top, a_right, a_bottom) = edges(a);
    let (b_left, b_top, b_right, b_bottom) = edges(b);
    let left = a_left.min(b_left);
    let top = a_top.min(b_top);
    let right = a_right.max(b_right);
    let bottom = a_bottom.max(b_bottom);

    Rect::new(
        point::logical(left, top),
        area::logical(right - left, bottom - top),
    )
}

fn expand_rounding(rounding: [f32; 4], amount: f32) -> [f32; 4] {
    [
        expand_corner_radius(rounding[0], amount),
        expand_corner_radius(rounding[1], amount),
        expand_corner_radius(rounding[2], amount),
        expand_corner_radius(rounding[3], amount),
    ]
}

fn expand_corner_radius(rounding: f32, amount: f32) -> f32 {
    if rounding <= 0.0 {
        0.0
    } else {
        rounding + amount
    }
}

fn shrink_rounding(rounding: [f32; 4], amount: f32) -> [f32; 4] {
    [
        (rounding[0] - amount).max(0.0),
        (rounding[1] - amount).max(0.0),
        (rounding[2] - amount).max(0.0),
        (rounding[3] - amount).max(0.0),
    ]
}

fn brush_data(brush: paint::Brush) -> PreparedBrush {
    match brush {
        paint::Brush::Solid(color) => PreparedBrush {
            from: color,
            to: color,
            points: [0.0, 0.0, 1.0, 1.0],
            kind: 0.0,
        },
        paint::Brush::Gradient(paint::Gradient::Linear(gradient)) => PreparedBrush {
            from: gradient.from(),
            to: gradient.to(),
            points: [
                gradient.start().x(),
                gradient.start().y(),
                gradient.end().x(),
                gradient.end().y(),
            ],
            kind: 1.0,
        },
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

fn rounding_data(rounding: [f32; 4]) -> [f32; 4] {
    rounding
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
        tint: Option<paint::Brush>,
    ) -> paint::Style {
        paint::Style { fill, stroke, tint }
    }

    fn solid(color: paint::Color) -> paint::Fill {
        paint::Fill::Brush(paint::Brush::solid(color))
    }

    fn gradient_brush() -> paint::Brush {
        paint::Brush::Gradient(paint::Gradient::Linear(paint::LinearGradient::new(
            paint::UnitPoint::new(0.0, 1.0),
            paint::UnitPoint::new(1.0, 0.0),
            paint::Color::rgba(1.0, 0.0, 0.0, 0.25),
            paint::Color::rgba(0.0, 0.0, 1.0, 0.75),
        )))
    }

    fn stroke(width: f32) -> paint::Stroke {
        paint::Stroke {
            brush: paint::Brush::solid(paint::Color::BLACK),
            width,
        }
    }

    fn vertex_count_for_shape(shape: AnalyticShape) -> usize {
        vertices_for_shape(shape).len()
    }

    fn vertices_for_shape(shape: AnalyticShape) -> Vec<render::primitive::Vertex> {
        let mut vertices = Vec::new();

        push_analytic_shape_vertices(
            &mut vertices,
            area::logical(100.0, 100.0),
            PixelGeometry::new(1.0),
            shape,
        );

        vertices
    }

    fn prepared_shape(shape: AnalyticShape, scale_factor: f32) -> PreparedShape {
        prepare_shape(shape, PixelGeometry::new(scale_factor)).expect("shape should prepare")
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

    fn rect_bounds(rect: Rect) -> (f32, f32, f32, f32) {
        edges(rect)
    }

    #[test]
    fn gradient_fill_lowers_to_one_analytic_shape() {
        let brush = gradient_brush();
        let quad = quad(style(Some(paint::Fill::Brush(brush)), None, None));
        let shapes = analytic_shapes_for_quad(&quad);

        assert_eq!(shapes.len(), 1);
        assert_eq!(shapes[0].outer_rect, rect());
        assert_eq!(shapes[0].brush, brush);
    }

    #[test]
    fn gradient_shape_vertices_preserve_brush_payload() {
        let brush = gradient_brush();
        let vertices = vertices_for_shape(fill_shape(rect(), brush));

        assert_eq!(vertices.len(), 6);
        assert_eq!(vertices[0].color, [1.0, 0.0, 0.0, 0.25]);
        assert_eq!(vertices[0].color_to, [0.0, 0.0, 1.0, 0.75]);
        assert_eq!(vertices[0].brush_points, [0.0, 1.0, 1.0, 0.0]);
        assert_eq!(vertices[0].params, [0.0, 0.0, 1.0, 0.0]);
    }

    #[test]
    fn gradient_stroke_tint_outline_and_shadow_preserve_brushes() {
        let brush = gradient_brush();
        let quad = quad(style(
            None,
            Some(paint::Stroke { brush, width: 2.0 }),
            Some(brush),
        ));
        let quad_shapes = analytic_shapes_for_quad(&quad);
        let tint_shape = analytic_shapes_for_tint(&paint::Tint {
            rect: rect(),
            brush,
        })[0];
        let outline_shape = analytic_shapes_for_outline(&paint::Outline {
            rect: rect(),
            brush,
            width: 2.0,
            offset: 1.0,
        })[0];
        let shadow_shape = analytic_shapes_for_shadow(&paint::Shadow {
            rect: rect(),
            brush,
            blur: 10.0,
            spread: 1.0,
            offset: point::logical(0.0, 4.0),
        })[0];

        assert_eq!(quad_shapes[0].brush, brush);
        assert_eq!(quad_shapes[1].brush, brush);
        assert_eq!(tint_shape.brush, brush);
        assert_eq!(outline_shape.brush, brush);
        assert_eq!(shadow_shape.brush, brush);
    }

    #[test]
    fn fill_only_quad_lowers_to_one_analytic_shape() {
        let quad = quad(style(Some(solid(paint::Color::RED)), None, None));
        let shapes = analytic_shapes_for_quad(&quad);

        assert_eq!(shapes.len(), 1);
        assert_eq!(shapes[0].outer_rect, rect());
        assert_eq!(
            shapes[0].outer_rounding,
            rect().rounding.resolve(rect().area)
        );
        assert_eq!(shapes[0].inner, None);
        assert_eq!(shapes[0].brush, paint::Brush::solid(paint::Color::RED));
        assert_eq!(vertex_count_for_shape(shapes[0]), 6);
    }

    #[test]
    fn fill_raster_bounds_expand_by_one_physical_pixel() {
        let shape = fill_shape(rect(), paint::Brush::solid(paint::Color::RED));
        let prepared = prepared_shape(shape, 1.0);

        assert_eq!(shape.outer_rect, rect());
        assert_eq!(prepared.outer_rect, rect());
        assert_eq!(rect_bounds(prepared.raster_rect), (9.0, 19.0, 51.0, 51.0));
    }

    #[test]
    fn scale_factor_controls_logical_aa_expansion() {
        let shape = fill_shape(rect(), paint::Brush::solid(paint::Color::RED));
        let prepared = prepared_shape(shape, 2.0);

        assert_eq!(prepared.outer_rect, rect());
        assert_eq!(rect_bounds(prepared.raster_rect), (9.5, 19.5, 50.5, 50.5));
    }

    #[test]
    fn snapped_rect_edges_round_through_physical_coordinates() {
        let rect = Rect::new(point::logical(10.2, 20.3), area::logical(40.4, 30.8));
        let snapped = PixelGeometry::new(2.0).snap_rect(rect);

        assert_eq!(rect_bounds(snapped), (10.0, 20.5, 50.5, 51.0));
    }

    #[test]
    fn positive_distances_snap_to_at_least_one_physical_pixel() {
        assert_eq!(PixelGeometry::new(1.0).snap_distance(0.2), 1.0);
        assert_eq!(PixelGeometry::new(2.0).snap_distance(0.2), 0.5);
        assert_eq!(PixelGeometry::new(2.0).snap_distance(2.2), 2.0);
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
    fn internal_stroke_shape_metadata_survives_raster_expansion() {
        let shape =
            internal_stroke_shape(rect(), 2.0, paint::Brush::solid(paint::Color::BLACK)).unwrap();
        let prepared = prepared_shape(shape, 1.0);
        let inner = prepared.inner.expect("stroke should stay a ring");

        assert_eq!(rect_bounds(prepared.raster_rect), (9.0, 19.0, 51.0, 51.0));
        assert_eq!(prepared.outer_rect, rect());
        assert_eq!(
            inner.rect,
            Rect::new(point::logical(12.0, 22.0), area::logical(36.0, 26.0))
        );
    }

    #[test]
    fn tint_only_quad_lowers_to_one_overlay_shape() {
        let quad = quad(style(
            None,
            None,
            Some(paint::Brush::solid(paint::Color::rgba(1.0, 1.0, 1.0, 0.25))),
        ));
        let shapes = analytic_shapes_for_quad(&quad);

        assert_eq!(shapes.len(), 1);
        assert_eq!(shapes[0].outer_rect, rect());
        assert_eq!(
            shapes[0].brush,
            paint::Brush::solid(paint::Color::rgba(1.0, 1.0, 1.0, 0.25))
        );
    }

    #[test]
    fn shadow_lowers_to_caster_with_owner_cutout() {
        let shadow = paint::Shadow {
            rect: Rect::rounded(
                rect().origin,
                rect().area,
                crate::geometry::rect::Rounding::relative(1.0),
            ),
            brush: paint::Brush::solid(paint::Color::rgba(0.0, 0.0, 0.0, 0.35)),
            blur: 18.0,
            spread: 2.0,
            offset: point::logical(0.0, 6.0),
        };
        let shapes = analytic_shapes_for_shadow(&shadow);
        let inner = shapes[0].inner.expect("shadow should cut out owner");

        assert_eq!(shapes.len(), 1);
        assert_eq!(shapes[0].kind, AnalyticShapeKind::Shadow);
        assert_eq!(rect_bounds(shapes[0].outer_rect), (8.0, 24.0, 52.0, 58.0));
        assert_eq!(inner.rect, shadow.rect);
        assert_eq!(
            inner.rounding,
            shadow.rect.rounding.resolve(shadow.rect.area)
        );
        assert_eq!(shapes[0].blur, 18.0);
    }

    #[test]
    fn shadow_raster_bounds_include_blur_and_cutout_metadata() {
        let shadow = paint::Shadow {
            rect: rect(),
            brush: paint::Brush::solid(paint::Color::rgba(0.0, 0.0, 0.0, 0.35)),
            blur: 10.0,
            spread: 2.0,
            offset: point::logical(0.0, 6.0),
        };
        let shape = analytic_shapes_for_shadow(&shadow)[0];
        let prepared = prepared_shape(shape, 1.0);
        let inner = prepared.inner.expect("shadow should keep cutout metadata");

        assert_eq!(rect_bounds(prepared.outer_rect), (8.0, 24.0, 52.0, 58.0));
        assert_eq!(rect_bounds(inner.rect), rect_bounds(rect()));
        assert_eq!(rect_bounds(prepared.raster_rect), (-3.0, 13.0, 63.0, 69.0));
    }

    #[test]
    fn shadow_vertices_use_shadow_shader_mode() {
        let shadow = paint::Shadow {
            rect: rect(),
            brush: paint::Brush::solid(paint::Color::rgba(0.0, 0.0, 0.0, 0.35)),
            blur: 10.0,
            spread: 2.0,
            offset: point::logical(0.0, 6.0),
        };
        let vertices = vertices_for_shape(analytic_shapes_for_shadow(&shadow)[0]);

        assert_eq!(vertices.len(), 6);
        assert_eq!(vertices[0].params, [2.0, 10.0, 0.0, 0.0]);
    }

    #[test]
    fn tint_raster_bounds_expand_while_shape_bounds_stay_exact() {
        let tint = paint::Tint {
            rect: rect(),
            brush: paint::Brush::solid(paint::Color::rgba(1.0, 1.0, 1.0, 0.25)),
        };
        let shape = analytic_shapes_for_tint(&tint)[0];
        let prepared = prepared_shape(shape, 1.0);

        assert_eq!(shape.outer_rect, tint.rect);
        assert_eq!(prepared.outer_rect, tint.rect);
        assert_eq!(rect_bounds(prepared.raster_rect), (9.0, 19.0, 51.0, 51.0));
    }

    #[test]
    fn fill_stroke_and_tint_quad_preserves_shape_order() {
        let quad = quad(style(
            Some(solid(paint::Color::RED)),
            Some(stroke(2.0)),
            Some(paint::Brush::solid(paint::Color::rgba(1.0, 1.0, 1.0, 0.25))),
        ));
        let shapes = analytic_shapes_for_quad(&quad);

        assert_eq!(shapes.len(), 3);
        assert_eq!(shapes[0].brush, paint::Brush::solid(paint::Color::RED));
        assert!(shapes[1].inner.is_some());
        assert_eq!(
            shapes[2].brush,
            paint::Brush::solid(paint::Color::rgba(1.0, 1.0, 1.0, 0.25))
        );
    }

    #[test]
    fn outline_item_expands_outside_rect_bounds() {
        let outline = paint::Outline {
            rect: rect(),
            brush: paint::Brush::solid(paint::Color::RED),
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
            crate::geometry::rect::Rounding::relative(1.0),
        );
        let outline = paint::Outline {
            rect,
            brush: paint::Brush::solid(paint::Color::RED),
            width: 4.0,
            offset: 2.0,
        };
        let shapes = analytic_shapes_for_outline(&outline);
        let inner = shapes[0].inner.expect("outline should be a ring");

        assert_eq!(shapes.len(), 1);
        assert_eq!(vertex_count_for_shape(shapes[0]), 6);
        assert_eq!(bounds(&shapes), (4.0, 14.0, 96.0, 56.0));
        assert_eq!(shapes[0].outer_rounding[0], 21.0);
        assert_eq!(inner.rounding[0], 17.0);
    }

    #[test]
    fn rounded_outline_raster_bounds_include_outline_and_aa_fringe() {
        let rect = Rect::rounded(
            point::logical(10.0, 20.0),
            area::logical(80.0, 30.0),
            crate::geometry::rect::Rounding::relative(1.0),
        );
        let outline = paint::Outline {
            rect,
            brush: paint::Brush::solid(paint::Color::RED),
            width: 4.0,
            offset: 2.0,
        };
        let shape = analytic_shapes_for_outline(&outline)[0];
        let prepared = prepared_shape(shape, 1.0);
        let inner = prepared.inner.expect("outline should stay a ring");

        assert_eq!(rect_bounds(prepared.outer_rect), (4.0, 14.0, 96.0, 56.0));
        assert_eq!(rect_bounds(prepared.raster_rect), (3.0, 13.0, 97.0, 57.0));
        assert_eq!(prepared.outer_rounding[0], 21.0);
        assert_eq!(inner.rounding[0], 17.0);
    }

    #[test]
    fn outline_shape_metadata_survives_raster_expansion() {
        let outline = paint::Outline {
            rect: rect(),
            brush: paint::Brush::solid(paint::Color::RED),
            width: 2.0,
            offset: 3.0,
        };
        let shape = analytic_shapes_for_outline(&outline)[0];
        let prepared = prepared_shape(shape, 1.0);
        let inner = prepared.inner.expect("outline should stay a ring");

        assert_eq!(rect_bounds(prepared.outer_rect), (5.0, 15.0, 55.0, 55.0));
        assert_eq!(rect_bounds(prepared.raster_rect), (4.0, 14.0, 56.0, 56.0));
        assert_eq!(
            inner.rect,
            Rect::new(point::logical(7.0, 17.0), area::logical(46.0, 36.0))
        );
    }

    #[test]
    fn rounded_fill_lowers_to_one_analytic_shape() {
        let quad = paint::Quad {
            rect: Rect::rounded(
                point::logical(10.0, 20.0),
                area::logical(40.0, 40.0),
                crate::geometry::rect::Rounding::relative(1.0),
            ),
            style: style(Some(solid(paint::Color::RED)), None, None),
        };
        let shapes = analytic_shapes_for_quad(&quad);

        assert_eq!(shapes.len(), 1);
        assert_eq!(vertex_count_for_shape(shapes[0]), 6);
        assert_eq!(bounds(&shapes), edges(quad.rect));
        assert_eq!(shapes[0].outer_rounding[0], 20.0);
    }

    #[test]
    fn rounded_tint_uses_same_shape_as_rounded_fill() {
        let rect = Rect::rounded(
            point::logical(10.0, 20.0),
            area::logical(80.0, 30.0),
            crate::geometry::rect::Rounding::relative(1.0),
        );
        let fill = fill_shape(rect, paint::Brush::solid(paint::Color::RED));
        let tint = paint::Tint {
            rect,
            brush: paint::Brush::solid(paint::Color::rgba(1.0, 1.0, 1.0, 0.25)),
        };
        let shapes = analytic_shapes_for_tint(&tint);

        assert_eq!(shapes.len(), 1);
        assert_eq!(shapes[0].outer_rect, fill.outer_rect);
        assert_eq!(shapes[0].outer_rounding, fill.outer_rounding);
        assert_eq!(bounds(&shapes), edges(rect));
    }

    #[test]
    fn rounded_internal_stroke_stays_inside_rect_bounds() {
        let quad = paint::Quad {
            rect: Rect::rounded(
                point::logical(10.0, 20.0),
                area::logical(80.0, 30.0),
                crate::geometry::rect::Rounding::relative(1.0),
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
        assert_eq!(shapes[0].outer_rounding[0], 15.0);
        assert_eq!(inner.rounding[0], 11.0);
    }
}
