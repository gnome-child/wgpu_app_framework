use wgpu::util::DeviceExt;

use crate::paint;
use crate::paint_geometry::{self, Rect};
use crate::render;
use crate::render::batch;
use crate::render::silhouette::{
    self, PixelGeometry, PreparedSilhouette, clamped_width, edges, expand_rect, expand_rounding,
    inset_rect, offset_rect, rect_data, rounding_data, shrink_rounding, union_rects,
};

const QUAD_WGSL: &str = include_str!("quad.wgsl");

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
    let shader_source = shader_source();
    let shader = render_context
        .device()
        .create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("quad.wgsl"),
            source: wgpu::ShaderSource::Wgsl(shader_source.into()),
        });

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
                buffers: &[render::Vertex::layout()],
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

pub(crate) fn shader_source() -> String {
    silhouette::wgsl_module_source(QUAD_WGSL)
}

pub fn prepare_batch(
    render_context: &render::Context,
    viewport: render::Viewport,
    shapes: &[batch::Shape<'_>],
) -> Option<Batch> {
    let mut vertex_buf = Vec::new();
    for shape in shapes {
        push_shape_vertices(&mut vertex_buf, viewport, shape);
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
    antialias: bool,
    snap_outer: bool,
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
    antialias: bool,
}

#[derive(Debug, Clone, Copy, PartialEq)]
struct PreparedBrush {
    from: paint::Color,
    to: paint::Color,
    points: [f32; 4],
    kind: f32,
}

fn push_shape_vertices(
    buffer: &mut Vec<render::Vertex>,
    viewport: render::Viewport,
    shape: &batch::Shape<'_>,
) {
    let canvas_area = viewport.logical_area();
    let pixel_geometry = PixelGeometry::new(viewport.scale_factor());

    if canvas_area.width() <= 0.0 || canvas_area.height() <= 0.0 {
        log::debug!("skipping shape draw for zero-size canvas");
        return;
    }

    for shape in analytic_shapes_for_shape(shape, viewport.scale_factor()) {
        push_analytic_shape_vertices(buffer, canvas_area, pixel_geometry, shape);
    }
}

fn analytic_shapes_for_shape(shape: &batch::Shape<'_>, scale_factor: f32) -> Vec<AnalyticShape> {
    match shape {
        batch::Shape::Quad(quad) => analytic_shapes_for_quad_at_scale(quad, scale_factor),
        batch::Shape::Shadow(shadow) => analytic_shapes_for_shadow(shadow),
        batch::Shape::Outline(outline) => analytic_shapes_for_outline(outline),
    }
}

#[cfg(test)]
fn analytic_shapes_for_quad(quad: &paint::Quad) -> Vec<AnalyticShape> {
    let rect = quad.transform.transformed_rect(quad.rect);
    analytic_shapes_for_quad_rect(quad, rect)
}

fn analytic_shapes_for_quad_at_scale(quad: &paint::Quad, scale_factor: f32) -> Vec<AnalyticShape> {
    let rect = rasterized_quad_rect(quad, scale_factor);

    analytic_shapes_for_quad_rect(quad, rect)
}

fn rasterized_quad_rect(quad: &paint::Quad, scale_factor: f32) -> Rect {
    let pixel_geometry = PixelGeometry::new(scale_factor);
    let rect = quad.transform.transformed_rect(quad.rect);

    match quad.rasterization.snapping {
        paint::Snapping::Disabled => rect,
        paint::Snapping::Rect => {
            debug_assert!(
                pixel_geometry.rect_is_aligned(rect),
                "Snapping::Rect expects geometry snapped at the layout-to-paint boundary"
            );
            rect
        }
        paint::Snapping::FixedWidth { width_px } => {
            pixel_geometry.snap_fixed_width_rect(rect, width_px)
        }
    }
}

fn analytic_shapes_for_quad_rect(quad: &paint::Quad, rect: Rect) -> Vec<AnalyticShape> {
    let mut shapes = Vec::new();
    let antialias = quad.rasterization.edge_mode == paint::EdgeMode::Antialiased;

    if let Some(fill) = quad.style.fill {
        match fill {
            paint::Fill::Brush(brush) => {
                let mut shape = fill_shape(rect, brush);
                shape.antialias = antialias;
                shape.snap_outer = false;
                shapes.push(shape);
            }
        }
    }

    if let Some(stroke) = quad.style.stroke
        && let Some(mut shape) = internal_stroke_shape(rect, stroke.width, stroke.brush)
    {
        shape.snap_outer = false;
        shapes.push(shape);
    }

    if let Some(brush) = quad.style.tint {
        let mut shape = fill_shape(rect, brush);
        shape.snap_outer = false;
        shapes.push(shape);
    }

    shapes
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
    buffer: &mut Vec<render::Vertex>,
    canvas_area: paint_geometry::LogicalArea,
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
        _ => [
            mode,
            0.0,
            brush.kind,
            if shape.antialias { 1.0 } else { 0.0 },
        ],
    };

    let mut push = |x: f32, y: f32| {
        buffer.push(render::Vertex {
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
    let silhouette = PreparedSilhouette::for_rect(
        shape.outer_rect,
        pixel_geometry,
        shape.snap_outer,
        shape.antialias,
    )?;

    Some(PreparedShape {
        kind: shape.kind,
        raster_rect: silhouette.raster_rect,
        outer_rect: silhouette.shape_rect,
        outer_rounding: silhouette.rounding,
        inner: None,
        brush: shape.brush,
        blur: shape.blur,
        antialias: shape.antialias,
    })
}

fn prepare_internal_ring_shape(
    shape: AnalyticShape,
    pixel_geometry: PixelGeometry,
) -> Option<PreparedShape> {
    let outer =
        PreparedSilhouette::for_rect(shape.outer_rect, pixel_geometry, shape.snap_outer, true)?;
    let outer_rect = outer.shape_rect;
    let width = pixel_geometry.snap_distance(ring_width(shape)?);
    let Some(inner_rect) = inset_rect(outer_rect, width) else {
        return prepare_fill_shape(fill_shape(outer_rect, shape.brush), pixel_geometry);
    };
    let outer_rounding = outer.rounding;
    let inner = AnalyticInner {
        rect: inner_rect,
        rounding: shrink_rounding(outer_rounding, width),
    };

    Some(PreparedShape {
        kind: shape.kind,
        raster_rect: outer.raster_rect,
        outer_rect,
        outer_rounding,
        inner: Some(inner),
        brush: shape.brush,
        blur: shape.blur,
        antialias: shape.antialias,
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
    let outer_rounding = silhouette::clamp_resolved_rounding(
        expand_rounding(inner_rounding, width),
        outer_rect.area,
    );
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
        antialias: shape.antialias,
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
        outer_rounding: silhouette::clamp_resolved_rounding(shape.outer_rounding, outer_rect.area),
        inner: Some(AnalyticInner {
            rect: inner_rect,
            rounding: inner_rect.rounding.resolve(inner_rect.area),
        }),
        brush: shape.brush,
        blur,
        antialias: shape.antialias,
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
        antialias: true,
        snap_outer: true,
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
        antialias: true,
        snap_outer: true,
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
        antialias: true,
        snap_outer: true,
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
        antialias: true,
        snap_outer: true,
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

#[cfg(test)]
pub(crate) fn prepared_fill_silhouette_for_test(
    rect: Rect,
    scale_factor: f32,
) -> PreparedSilhouette {
    let shape = fill_shape(rect, paint::Brush::solid(paint::Color::BLACK));
    let prepared = prepare_shape(shape, PixelGeometry::new(scale_factor))
        .expect("fill silhouette should prepare");

    PreparedSilhouette::from_parts(prepared.outer_rect, prepared.raster_rect)
        .with_rounding(prepared.outer_rounding)
}

#[cfg(test)]
pub(crate) fn prepared_shadow_cutout_silhouette_for_test(
    shadow: paint::Shadow,
    scale_factor: f32,
) -> PreparedSilhouette {
    let shape = analytic_shapes_for_shadow(&shadow)
        .into_iter()
        .next()
        .expect("shadow should lower to shape");
    let prepared = prepare_shape(shape, PixelGeometry::new(scale_factor))
        .expect("shadow silhouette should prepare");
    let inner = prepared.inner.expect("shadow should keep owner cutout");
    let raster_rect = expand_rect(inner.rect, PixelGeometry::new(scale_factor).logical_pixel());

    PreparedSilhouette::from_parts(inner.rect, raster_rect).with_rounding(inner.rounding)
}

#[cfg(test)]
mod tests {
    use crate::paint_geometry;

    use super::*;

    fn rect() -> Rect {
        Rect::new(
            paint_geometry::logical_point(10.0, 20.0),
            paint_geometry::logical_area(40.0, 30.0),
        )
    }

    fn quad(style: paint::Style) -> paint::Quad {
        paint::Quad {
            rect: rect(),
            style,
            rasterization: paint::Rasterization::default(),
            transform: paint::Transform::identity(),
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

    fn vertices_for_shape(shape: AnalyticShape) -> Vec<render::Vertex> {
        let mut vertices = Vec::new();

        push_analytic_shape_vertices(
            &mut vertices,
            paint_geometry::logical_area(100.0, 100.0),
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

    fn assert_approx_eq(actual: f32, expected: f32) {
        assert!(
            (actual - expected).abs() < 0.0001,
            "expected {actual} to approximately equal {expected}"
        );
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
        assert_eq!(vertices[0].params, [0.0, 0.0, 1.0, 1.0]);
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
            offset: paint_geometry::logical_point(0.0, 4.0),
        })[0];

        assert_eq!(quad_shapes[0].brush, brush);
        assert_eq!(quad_shapes[1].brush, brush);
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
    fn transformed_quad_lowers_to_transformed_analytic_bounds() {
        let mut quad = quad(style(Some(solid(paint::Color::RED)), None, None));
        quad.transform =
            paint::Transform::scale_y_about(paint_geometry::logical_point(30.0, 35.0), 1.5);
        let shapes = analytic_shapes_for_quad(&quad);

        assert_eq!(shapes.len(), 1);
        assert_eq!(rect_bounds(shapes[0].outer_rect), (10.0, 12.5, 50.0, 57.5));
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
    fn unsnapped_transformed_quad_preserves_subpixel_bounds() {
        let mut quad = quad(style(Some(solid(paint::Color::RED)), None, None));
        quad.rect = Rect::new(
            paint_geometry::logical_point(10.0, 20.0),
            paint_geometry::logical_area(40.0, 4.0),
        );
        quad.transform =
            paint::Transform::scale_y_about(paint_geometry::logical_point(30.0, 22.0), 1.03125);

        let shapes = analytic_shapes_for_quad_at_scale(&quad, 1.0);
        let prepared = prepared_shape(shapes[0], 1.0);

        assert_eq!(
            rect_bounds(shapes[0].outer_rect),
            (10.0, 19.9375, 50.0, 24.0625)
        );
        assert_eq!(
            rect_bounds(prepared.outer_rect),
            (10.0, 19.9375, 50.0, 24.0625)
        );
    }

    #[test]
    fn rect_snapping_mode_checks_boundary_aligned_geometry() {
        let mut quad = quad(style(Some(solid(paint::Color::RED)), None, None));
        quad.rect = Rect::new(
            paint_geometry::logical_point(10.4, 20.0),
            paint_geometry::logical_area(32.8, 11.2),
        );
        quad.rasterization.snapping = paint::Snapping::Rect;

        let shapes = analytic_shapes_for_quad_at_scale(&quad, 1.25);
        let (left, top, right, bottom) = rect_bounds(shapes[0].outer_rect);

        assert_approx_eq(left, 10.4);
        assert_approx_eq(top, 20.0);
        assert_approx_eq(right, 43.2);
        assert_approx_eq(bottom, 31.2);
    }

    #[test]
    #[should_panic(expected = "Snapping::Rect expects geometry snapped")]
    fn rect_snapping_mode_rejects_unsnapped_geometry() {
        let mut quad = quad(style(Some(solid(paint::Color::RED)), None, None));
        quad.rect = Rect::new(
            paint_geometry::logical_point(10.0, 20.0),
            paint_geometry::logical_area(33.0, 11.0),
        );
        quad.rasterization.snapping = paint::Snapping::Rect;

        let _ = analytic_shapes_for_quad_at_scale(&quad, 1.25);
    }

    #[test]
    fn snapped_rect_edges_round_through_physical_coordinates() {
        let rect = Rect::new(
            paint_geometry::logical_point(10.2, 20.3),
            paint_geometry::logical_area(40.4, 30.8),
        );
        let snapped = PixelGeometry::new(2.0).snap_rect(rect);

        assert_eq!(rect_bounds(snapped), (10.0, 20.5, 50.5, 51.0));
    }

    #[test]
    fn fixed_width_snapping_forces_stable_quad_width_only_when_requested() {
        let scale_factor = 1.5;
        let rect = Rect::new(
            paint_geometry::logical_point(10.2, 20.3),
            paint_geometry::logical_area(7.7, 9.2),
        );
        let mut snapped_quad = quad(style(Some(solid(paint::Color::RED)), None, None));
        snapped_quad.rect = rect;
        snapped_quad.rasterization.snapping = paint::Snapping::FixedWidth { width_px: 2 };
        snapped_quad.rasterization.edge_mode = paint::EdgeMode::Hard;

        let snapped = analytic_shapes_for_quad_at_scale(&snapped_quad, scale_factor);
        let (left, top, right, bottom) = rect_bounds(snapped[0].outer_rect);

        assert_approx_eq(
            left * scale_factor,
            (rect.origin.x() * scale_factor).round(),
        );
        assert_approx_eq(top * scale_factor, (rect.origin.y() * scale_factor).round());
        assert_approx_eq(
            bottom * scale_factor,
            ((rect.origin.y() + rect.area.height()) * scale_factor).round(),
        );
        assert_approx_eq((right - left) * scale_factor, 2.0);

        let mut ordinary_quad = snapped_quad;
        ordinary_quad.rasterization = paint::Rasterization::default();
        let ordinary = analytic_shapes_for_quad_at_scale(&ordinary_quad, scale_factor);

        assert_eq!(ordinary[0].outer_rect, rect);
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
            Rect::new(
                paint_geometry::logical_point(12.0, 22.0),
                paint_geometry::logical_area(36.0, 26.0)
            )
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
            Rect::new(
                paint_geometry::logical_point(12.0, 22.0),
                paint_geometry::logical_area(36.0, 26.0)
            )
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
                crate::paint_geometry::Rounding::relative(1.0),
            ),
            brush: paint::Brush::solid(paint::Color::rgba(0.0, 0.0, 0.0, 0.35)),
            blur: 18.0,
            spread: 2.0,
            offset: paint_geometry::logical_point(0.0, 6.0),
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
            offset: paint_geometry::logical_point(0.0, 6.0),
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
            offset: paint_geometry::logical_point(0.0, 6.0),
        };
        let vertices = vertices_for_shape(analytic_shapes_for_shadow(&shadow)[0]);

        assert_eq!(vertices.len(), 6);
        assert_eq!(vertices[0].params, [2.0, 10.0, 0.0, 0.0]);
    }

    #[test]
    fn tint_raster_bounds_expand_while_shape_bounds_stay_exact() {
        let quad = quad(style(
            None,
            None,
            Some(paint::Brush::solid(paint::Color::rgba(1.0, 1.0, 1.0, 0.25))),
        ));
        let shape = analytic_shapes_for_quad(&quad)[0];
        let prepared = prepared_shape(shape, 1.0);

        assert_eq!(shape.outer_rect, quad.rect);
        assert_eq!(prepared.outer_rect, quad.rect);
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
            Rect::new(
                paint_geometry::logical_point(7.0, 17.0),
                paint_geometry::logical_area(46.0, 36.0)
            )
        );
    }

    #[test]
    fn rounded_outline_lowers_to_expanded_rounded_ring() {
        let rect = Rect::rounded(
            paint_geometry::logical_point(10.0, 20.0),
            paint_geometry::logical_area(80.0, 30.0),
            crate::paint_geometry::Rounding::relative(1.0),
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
            paint_geometry::logical_point(10.0, 20.0),
            paint_geometry::logical_area(80.0, 30.0),
            crate::paint_geometry::Rounding::relative(1.0),
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
            Rect::new(
                paint_geometry::logical_point(7.0, 17.0),
                paint_geometry::logical_area(46.0, 36.0)
            )
        );
    }

    #[test]
    fn rounded_fill_lowers_to_one_analytic_shape() {
        let quad = paint::Quad {
            rect: Rect::rounded(
                paint_geometry::logical_point(10.0, 20.0),
                paint_geometry::logical_area(40.0, 40.0),
                crate::paint_geometry::Rounding::relative(1.0),
            ),
            style: style(Some(solid(paint::Color::RED)), None, None),
            rasterization: paint::Rasterization::default(),
            transform: paint::Transform::identity(),
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
            paint_geometry::logical_point(10.0, 20.0),
            paint_geometry::logical_area(80.0, 30.0),
            crate::paint_geometry::Rounding::relative(1.0),
        );
        let fill = fill_shape(rect, paint::Brush::solid(paint::Color::RED));
        let quad = paint::Quad {
            rect,
            style: style(
                None,
                None,
                Some(paint::Brush::solid(paint::Color::rgba(1.0, 1.0, 1.0, 0.25))),
            ),
            rasterization: paint::Rasterization::default(),
            transform: paint::Transform::identity(),
        };
        let shapes = analytic_shapes_for_quad(&quad);

        assert_eq!(shapes.len(), 1);
        assert_eq!(shapes[0].outer_rect, fill.outer_rect);
        assert_eq!(shapes[0].outer_rounding, fill.outer_rounding);
        assert_eq!(bounds(&shapes), edges(rect));
    }

    #[test]
    fn rounded_internal_stroke_stays_inside_rect_bounds() {
        let quad = paint::Quad {
            rect: Rect::rounded(
                paint_geometry::logical_point(10.0, 20.0),
                paint_geometry::logical_area(80.0, 30.0),
                crate::paint_geometry::Rounding::relative(1.0),
            ),
            style: style(None, Some(stroke(4.0)), None),
            rasterization: paint::Rasterization::default(),
            transform: paint::Transform::identity(),
        };
        let shapes = analytic_shapes_for_quad(&quad);
        let inner = shapes[0].inner.expect("stroke should be a ring");

        assert_eq!(shapes.len(), 1);
        assert_eq!(bounds(&shapes), edges(quad.rect));
        assert_eq!(
            inner.rect,
            Rect::new(
                paint_geometry::logical_point(14.0, 24.0),
                paint_geometry::logical_area(72.0, 22.0)
            )
        );
        assert_eq!(shapes[0].outer_rounding[0], 15.0);
        assert_eq!(inner.rounding[0], 11.0);
    }
}
