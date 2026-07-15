#[cfg(test)]
use crate::geometry::area;
use bytemuck::{Pod, Zeroable};

use crate::paint::{self, Grid, Rect};
use crate::render;
use crate::render::content;
use crate::render::silhouette::{
    self, PreparedSilhouette, clamped_width, edges, expand_rect, expand_rounding, inset_rect,
    offset_rect, rect_data, rounding_data, shrink_rounding, union_rects,
};

#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Pod, Zeroable)]
pub(in crate::render) struct Instance {
    raster_rect: [f32; 4],
    outer_rect: [f32; 4],
    outer_rounding: [f32; 4],
    inner_rect: [f32; 4],
    inner_rounding: [f32; 4],
    color: [f32; 4],
    color_to: [f32; 4],
    brush_points: [f32; 4],
    params: [f32; 4],
    source_rect: [f32; 4],
}

impl Instance {
    pub(in crate::render) fn layout() -> wgpu::VertexBufferLayout<'static> {
        const ATTRIBUTES: [wgpu::VertexAttribute; 10] = wgpu::vertex_attr_array![
            1 => Float32x4,
            2 => Float32x4,
            3 => Float32x4,
            4 => Float32x4,
            5 => Float32x4,
            6 => Float32x4,
            7 => Float32x4,
            8 => Float32x4,
            9 => Float32x4,
            10 => Float32x4
        ];

        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<Self>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Instance,
            attributes: &ATTRIBUTES,
        }
    }

    pub(in crate::render) fn set_source_rect(&mut self, rect: [f32; 4]) {
        self.source_rect = rect;
    }
}

pub(in crate::render) fn prepare_instances(
    viewport: render::Viewport,
    shapes: &[content::Shape<'_>],
) -> Vec<Instance> {
    let grid = Grid::new(viewport.scale_factor());
    shapes
        .iter()
        .flat_map(|shape| analytic_shapes_for_shape(shape, viewport.scale_factor()))
        .filter_map(|shape| prepare_instance(shape, grid))
        .collect()
}

#[derive(Debug, Clone, Copy, PartialEq)]
struct AnalyticShape {
    kind: AnalyticShapeKind,
    outer_rect: Rect,
    outer_rounding: [f32; 4],
    inner: Option<AnalyticInner>,
    external_outline: Option<ExternalOutline>,
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
struct ExternalOutline {
    base_rect: Rect,
    offset: f32,
    width: f32,
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

fn analytic_shapes_for_shape(shape: &content::Shape<'_>, scale_factor: f32) -> Vec<AnalyticShape> {
    match shape {
        content::Shape::Quad(quad) => analytic_shapes_for_quad_at_scale(quad, scale_factor),
        content::Shape::Rule(rule) => analytic_shapes_for_rule_at_scale(rule, scale_factor),
        content::Shape::Shadow(shadow) => analytic_shapes_for_shadow(shadow),
        content::Shape::Outline(outline) => analytic_shapes_for_outline(outline),
    }
}

#[cfg(test)]
fn analytic_shapes_for_quad(quad: &paint::Quad) -> Vec<AnalyticShape> {
    let rect = quad.transform().transformed_rect(quad.rect());
    analytic_shapes_for_quad_rect(quad, rect)
}

fn analytic_shapes_for_quad_at_scale(quad: &paint::Quad, scale_factor: f32) -> Vec<AnalyticShape> {
    let rect = rasterized_quad_rect(quad, scale_factor);

    analytic_shapes_for_quad_rect(quad, rect)
}

fn rasterized_quad_rect(quad: &paint::Quad, scale_factor: f32) -> Rect {
    let grid = Grid::new(scale_factor);
    let rect = transformed_quad_rect(quad, grid);

    if quad.transform().motion == paint::Motion::Resting {
        debug_assert!(
            grid.rect_is_aligned(rect),
            "resting quad geometry must be snapped at the layout-to-paint boundary: rect=({}, {}, {}, {}) scale={} transform={:?}",
            rect.origin.x(),
            rect.origin.y(),
            rect.area.width(),
            rect.area.height(),
            scale_factor,
            quad.transform()
        );
    }

    rect
}

fn transformed_quad_rect(quad: &paint::Quad, grid: Grid) -> Rect {
    let (rect, transform) = quad.transform().resolve_rect(quad.rect(), grid);
    transform.transformed_rect(rect)
}

fn analytic_shapes_for_quad_rect(quad: &paint::Quad, rect: Rect) -> Vec<AnalyticShape> {
    let mut shapes = Vec::new();
    let antialias = quad.rasterization().edge_mode == paint::EdgeMode::Antialiased;

    if let Some(fill) = quad.style().fill {
        match fill {
            paint::Fill::Brush(brush) => {
                let mut shape = fill_shape(rect, brush);
                shape.antialias = antialias;
                shape.snap_outer = false;
                shapes.push(shape);
            }
        }
    }

    if let Some(stroke) = quad.style().stroke
        && let Some(mut shape) = internal_stroke_shape(rect, stroke.width, stroke.brush)
    {
        shape.snap_outer = false;
        shapes.push(shape);
    }

    if let Some(brush) = quad.style().tint {
        let mut shape = fill_shape(rect, brush);
        shape.snap_outer = false;
        shapes.push(shape);
    }

    shapes
}

fn analytic_shapes_for_rule_at_scale(rule: &paint::Rule, scale_factor: f32) -> Vec<AnalyticShape> {
    let rect = rasterized_rule_rect(rule, scale_factor);
    let mut shape = fill_shape(rect, rule.brush);
    shape.antialias = false;
    shape.snap_outer = false;

    vec![shape]
}

fn rasterized_rule_rect(rule: &paint::Rule, scale_factor: f32) -> Rect {
    let grid = Grid::new(scale_factor);
    match rule.axis {
        paint::Axis::Horizontal => grid.snap_horizontal_rule_rect(rule.rect, rule.thickness_px),
        paint::Axis::Vertical => grid.snap_vertical_rule_rect(rule.rect, rule.thickness_px),
    }
}

fn analytic_shapes_for_shadow(shadow: &paint::Shadow) -> Vec<AnalyticShape> {
    shadow_shape(shadow).into_iter().collect()
}

fn analytic_shapes_for_outline(outline: &paint::Outline) -> Vec<AnalyticShape> {
    external_outline_shape(outline.rect, outline.width, outline.offset, outline.brush)
        .into_iter()
        .collect()
}

fn prepare_instance(shape: AnalyticShape, grid: Grid) -> Option<Instance> {
    let shape = prepare_shape(shape, grid)?;
    let (x0, y0, x1, y1) = edges(shape.raster_rect);
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

    Some(Instance {
        raster_rect: [x0, y0, x1 - x0, y1 - y0],
        outer_rect: rect_data(shape.outer_rect),
        outer_rounding: rounding_data(shape.outer_rounding),
        inner_rect,
        inner_rounding,
        color: brush.from.to_array(),
        color_to: brush.to.to_array(),
        brush_points: brush.points,
        params,
        source_rect: rect_data(shape.outer_rect),
    })
}

fn prepare_shape(shape: AnalyticShape, grid: Grid) -> Option<PreparedShape> {
    match shape.kind {
        AnalyticShapeKind::Fill => prepare_fill_shape(shape, grid),
        AnalyticShapeKind::InternalRing => prepare_internal_ring_shape(shape, grid),
        AnalyticShapeKind::ExternalRing => prepare_external_ring_shape(shape, grid),
        AnalyticShapeKind::Shadow => prepare_shadow_shape(shape, grid),
    }
}

fn prepare_fill_shape(shape: AnalyticShape, grid: Grid) -> Option<PreparedShape> {
    let silhouette =
        PreparedSilhouette::for_rect(shape.outer_rect, grid, shape.snap_outer, shape.antialias)?;

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

fn prepare_internal_ring_shape(shape: AnalyticShape, grid: Grid) -> Option<PreparedShape> {
    let outer = PreparedSilhouette::for_rect(shape.outer_rect, grid, shape.snap_outer, true)?;
    let outer_rect = outer.shape_rect;
    let width = grid.snap_distance(ring_width(shape)?);
    let Some(inner_rect) = inset_rect(outer_rect, width) else {
        return prepare_fill_shape(fill_shape(outer_rect, shape.brush), grid);
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

fn prepare_external_ring_shape(shape: AnalyticShape, grid: Grid) -> Option<PreparedShape> {
    let outline = shape.external_outline?;
    let outset = grid.snap_outset(outline.base_rect, outline.offset, outline.width);
    let base_rounding = outline.base_rect.rounding.resolve(outset.base_rect.area);
    let inner_rect = outset.inner_rect;
    let outer_rect = outset.outer_rect;
    let inner_rounding = silhouette::clamp_resolved_rounding(
        expand_rounding(base_rounding, outset.offset),
        inner_rect.area,
    );
    let outer_rounding = silhouette::clamp_resolved_rounding(
        expand_rounding(base_rounding, outset.offset + outset.width),
        outer_rect.area,
    );
    let raster_rect = expand_rect(outer_rect, grid.logical_pixel());

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

fn prepare_shadow_shape(shape: AnalyticShape, grid: Grid) -> Option<PreparedShape> {
    let inner = shape.inner?;
    let outer_rect = grid.snap_rect(shape.outer_rect);
    let inner_rect = grid.snap_rect(inner.rect);
    let blur = grid.snap_distance(shape.blur);
    let raster_rect = union_rects(
        expand_rect(outer_rect, blur + grid.logical_pixel()),
        expand_rect(inner_rect, grid.logical_pixel()),
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
        external_outline: None,
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
        external_outline: None,
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
        external_outline: None,
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
        external_outline: Some(ExternalOutline {
            base_rect: rect,
            offset,
            width,
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
pub(in crate::render) fn prepared_fill_silhouette_for_test(
    rect: Rect,
    scale_factor: f32,
) -> PreparedSilhouette {
    let shape = fill_shape(rect, paint::Brush::solid(paint::Color::BLACK));
    let prepared =
        prepare_shape(shape, Grid::new(scale_factor)).expect("fill silhouette should prepare");

    PreparedSilhouette::from_parts(prepared.outer_rect, prepared.raster_rect)
        .with_rounding(prepared.outer_rounding)
}

#[cfg(test)]
pub(in crate::render) fn prepared_shadow_cutout_silhouette_for_test(
    shadow: paint::Shadow,
    scale_factor: f32,
) -> PreparedSilhouette {
    let shape = analytic_shapes_for_shadow(&shadow)
        .into_iter()
        .next()
        .expect("shadow should lower to shape");
    let prepared =
        prepare_shape(shape, Grid::new(scale_factor)).expect("shadow silhouette should prepare");
    let inner = prepared.inner.expect("shadow should keep owner cutout");
    let raster_rect = expand_rect(inner.rect, Grid::new(scale_factor).logical_pixel());

    PreparedSilhouette::from_parts(inner.rect, raster_rect).with_rounding(inner.rounding)
}

#[cfg(test)]
mod tests {
    use crate::geometry::point;
    use crate::paint;

    use super::*;

    fn rect() -> Rect {
        Rect::new(point::logical(10.0, 20.0), area::logical(40.0, 30.0))
    }

    fn quad(style: paint::Style) -> paint::Quad {
        paint::Quad::unchecked_for_test(
            rect(),
            style,
            paint::Rasterization::default(),
            paint::Transform::identity(),
        )
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

    fn instance_count_for_shape(shape: AnalyticShape) -> usize {
        usize::from(prepare_instance(shape, Grid::new(1.0)).is_some())
    }

    fn instance_for_shape(shape: AnalyticShape) -> Instance {
        prepare_instance(shape, Grid::new(1.0)).expect("shape should produce one instance")
    }

    fn prepared_shape(shape: AnalyticShape, scale_factor: f32) -> PreparedShape {
        prepare_shape(shape, Grid::new(scale_factor)).expect("shape should prepare")
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

    fn physical_bounds(rect: Rect, scale: f32) -> (f32, f32, f32, f32) {
        let (left, top, right, bottom) = edges(rect);

        (left * scale, top * scale, right * scale, bottom * scale)
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
    fn gradient_shape_instance_preserves_brush_payload() {
        let brush = gradient_brush();
        let instance = instance_for_shape(fill_shape(rect(), brush));

        assert_eq!(instance.color, [1.0, 0.0, 0.0, 0.25]);
        assert_eq!(instance.color_to, [0.0, 0.0, 1.0, 0.75]);
        assert_eq!(instance.brush_points, [0.0, 1.0, 1.0, 0.0]);
        assert_eq!(instance.params, [0.0, 0.0, 1.0, 1.0]);
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
            offset: point::logical(0.0, 4.0),
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
        assert_eq!(instance_count_for_shape(shapes[0]), 1);
    }

    #[test]
    fn transformed_quad_lowers_to_transformed_analytic_bounds() {
        let mut quad = quad(style(Some(solid(paint::Color::RED)), None, None));
        quad.set_transform_for_test(paint::Transform::scale_y_about(
            point::logical(30.0, 35.0),
            1.5,
        ));
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
    fn moving_transformed_quad_preserves_subpixel_bounds() {
        let mut quad = quad(style(Some(solid(paint::Color::RED)), None, None));
        quad.set_rect_for_test(Rect::new(
            point::logical(10.0, 20.0),
            area::logical(40.0, 4.0),
        ));
        quad.set_transform_for_test(
            paint::Transform::scale_y_about(point::logical(30.0, 22.0), 1.03125)
                .with_motion(paint::Motion::Moving),
        );

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
    fn moving_scale_endpoint_starts_at_snapped_rest_pose() {
        let mut quad = quad(style(Some(solid(paint::Color::RED)), None, None));
        let rect = Rect::new(point::logical(10.0, 20.0), area::logical(40.0, 6.0));
        quad.set_rect_for_test(rect);
        quad.set_transform_for_test(
            paint::Transform::scale_y_about(point::logical(30.0, 23.0), 1.0)
                .with_motion(paint::Motion::Moving)
                .with_scale_motion(1.0, 1.0, 1.0, 1.5, 0.0),
        );

        let moving = analytic_shapes_for_quad_at_scale(&quad, 1.25)[0].outer_rect;
        let snapped_start = Grid::new(1.25).snap_rect_with_stable_size(rect);

        assert_eq!(rect_bounds(moving), rect_bounds(snapped_start));
    }

    #[test]
    fn moving_scale_endpoint_matches_resting_target_at_fractional_scale() {
        let mut moving = quad(style(Some(solid(paint::Color::RED)), None, None));
        moving.set_rect_for_test(Rect::new(
            point::logical(10.0, 20.0),
            area::logical(40.0, 6.0),
        ));
        moving.set_transform_for_test(
            paint::Transform::scale_y_about(point::logical(30.0, 23.0), 1.5)
                .with_motion(paint::Motion::Moving)
                .with_scale_motion(1.0, 1.0, 1.0, 1.5, 1.0),
        );
        let mut resting = moving;
        resting.set_transform_for_test(paint::Transform::scale_y_about(
            point::logical(30.0, 23.0),
            1.5,
        ));

        let moving_rect = analytic_shapes_for_quad_at_scale(&moving, 1.25)[0].outer_rect;
        let resting_rect = analytic_shapes_for_quad_at_scale(&resting, 1.25)[0].outer_rect;

        assert_eq!(rect_bounds(moving_rect), rect_bounds(resting_rect));
        assert!(Grid::new(1.25).rect_is_aligned(moving_rect));
    }

    #[test]
    fn moving_scale_endpoint_matches_resting_target_at_one_x() {
        let mut moving = quad(style(Some(solid(paint::Color::RED)), None, None));
        moving.set_rect_for_test(Rect::new(
            point::logical(10.0, 20.0),
            area::logical(40.0, 6.0),
        ));
        moving.set_transform_for_test(
            paint::Transform::scale_y_about(point::logical(30.0, 23.0), 1.5)
                .with_motion(paint::Motion::Moving)
                .with_scale_motion(1.0, 1.0, 1.0, 1.5, 1.0),
        );
        let mut resting = moving;
        resting.set_transform_for_test(paint::Transform::scale_y_about(
            point::logical(30.0, 23.0),
            1.5,
        ));

        let moving_rect = analytic_shapes_for_quad_at_scale(&moving, 1.0)[0].outer_rect;
        let resting_rect = analytic_shapes_for_quad_at_scale(&resting, 1.0)[0].outer_rect;

        assert_eq!(rect_bounds(moving_rect), rect_bounds(resting_rect));
        assert!(Grid::new(1.0).rect_is_aligned(moving_rect));
    }

    #[test]
    fn resting_transformed_quad_snaps_to_aligned_bounds() {
        let mut quad = quad(style(Some(solid(paint::Color::RED)), None, None));
        quad.set_rect_for_test(Rect::new(
            point::logical(10.0, 20.0),
            area::logical(40.0, 6.0),
        ));
        quad.set_transform_for_test(paint::Transform::scale_y_about(
            point::logical(30.0, 23.0),
            1.5,
        ));

        let shapes = analytic_shapes_for_quad_at_scale(&quad, 1.0);
        let (left, top, right, bottom) = rect_bounds(shapes[0].outer_rect);

        assert_eq!((left, top, right, bottom), (10.0, 18.0, 50.0, 27.0));
        assert!(Grid::new(1.0).rect_is_aligned(shapes[0].outer_rect));
    }

    #[test]
    fn resting_scale_snaps_to_stable_size_across_origins() {
        let mut first = quad(style(Some(solid(paint::Color::RED)), None, None));
        first.set_rect_for_test(Rect::new(
            point::logical(10.0, 20.0),
            area::logical(40.0, 6.0),
        ));
        first.set_transform_for_test(paint::Transform::scale_y_about(
            point::logical(30.0, 23.0),
            1.5,
        ));

        let mut second = first;
        second.set_rect_for_test(Rect::new(
            point::logical(10.0, 21.0),
            area::logical(40.0, 6.0),
        ));
        second.set_transform_for_test(paint::Transform::scale_y_about(
            point::logical(30.0, 24.0),
            1.5,
        ));

        let first_rect = analytic_shapes_for_quad_at_scale(&first, 1.0)[0].outer_rect;
        let second_rect = analytic_shapes_for_quad_at_scale(&second, 1.0)[0].outer_rect;

        assert_eq!(first_rect.area.height(), 9.0);
        assert_eq!(second_rect.area.height(), 9.0);
        assert!(Grid::new(1.0).rect_is_aligned(first_rect));
        assert!(Grid::new(1.0).rect_is_aligned(second_rect));
    }

    #[test]
    fn resting_transform_bake_preserves_stroke_and_rounding_semantics() {
        let mut moving = quad(style(
            Some(solid(paint::Color::RED)),
            Some(stroke(2.0)),
            None,
        ));
        moving.set_rect_for_test(Rect::rounded(
            point::logical(10.0, 20.0),
            area::logical(40.0, 6.0),
            paint::Rounding::relative(1.0),
        ));
        moving.set_transform_for_test(
            paint::Transform::scale_y_about(point::logical(30.0, 23.0), 1.5)
                .with_motion(paint::Motion::Moving),
        );
        let mut resting = moving;
        resting.set_transform_for_test(resting.transform().with_motion(paint::Motion::Resting));

        let moving_shapes = analytic_shapes_for_quad_at_scale(&moving, 1.0);
        let resting_shapes = analytic_shapes_for_quad_at_scale(&resting, 1.0);

        assert_eq!(moving_shapes.len(), resting_shapes.len());
        for (moving, resting) in moving_shapes.iter().zip(resting_shapes.iter()) {
            assert_eq!(moving.kind, resting.kind);
            assert_eq!(moving.brush, resting.brush);
            assert_eq!(moving.antialias, resting.antialias);
            assert_eq!(moving.inner.is_some(), resting.inner.is_some());
        }
    }

    #[test]
    fn resting_quad_geometry_checks_boundary_aligned_geometry() {
        let mut quad = quad(style(Some(solid(paint::Color::RED)), None, None));
        quad.set_rect_for_test(Rect::new(
            point::logical(10.4, 20.0),
            area::logical(32.8, 11.2),
        ));

        let shapes = analytic_shapes_for_quad_at_scale(&quad, 1.25);
        let (left, top, right, bottom) = rect_bounds(shapes[0].outer_rect);

        assert_approx_eq(left, 10.4);
        assert_approx_eq(top, 20.0);
        assert_approx_eq(right, 43.2);
        assert_approx_eq(bottom, 31.2);
    }

    #[test]
    #[should_panic(expected = "resting quad geometry must be snapped")]
    fn resting_quad_geometry_rejects_unsnapped_geometry() {
        let mut quad = quad(style(Some(solid(paint::Color::RED)), None, None));
        quad.set_rect_for_test(Rect::new(
            point::logical(10.0, 20.0),
            area::logical(33.0, 11.0),
        ));

        let _ = analytic_shapes_for_quad_at_scale(&quad, 1.25);
    }

    #[test]
    fn moving_baked_scale_geometry_does_not_trip_resting_alignment_witness() {
        let mut quad = quad(style(Some(solid(paint::Color::RED)), None, None));
        quad.set_rect_for_test(Rect::new(
            point::logical(192.0, 123.08172),
            area::logical(369.59998, 4.236557),
        ));
        quad.set_transform_for_test(
            paint::Transform::identity().with_motion(paint::Motion::Moving),
        );

        let shapes = analytic_shapes_for_quad_at_scale(&quad, 1.25);

        assert_eq!(shapes.len(), 1);
        assert_eq!(
            rect_bounds(shapes[0].outer_rect),
            (192.0, 123.08172, 561.6, 127.318275)
        );
    }

    #[test]
    fn snapped_rect_edges_round_through_physical_coordinates() {
        let rect = Rect::new(point::logical(10.2, 20.3), area::logical(40.4, 30.8));
        let snapped = Grid::new(2.0).snap_rect(rect);

        assert_eq!(rect_bounds(snapped), (10.0, 20.5, 50.5, 51.0));
    }

    #[test]
    fn vertical_rule_snapping_forces_stable_physical_width() {
        let scale_factor = 1.5;
        let rect = Rect::new(point::logical(10.2, 20.3), area::logical(7.7, 9.2));
        let rule = paint::Rule {
            axis: paint::Axis::Vertical,
            rect,
            brush: paint::Brush::solid(paint::Color::RED),
            thickness_px: 2,
        };

        let snapped = analytic_shapes_for_rule_at_scale(&rule, scale_factor);
        let (left, top, right, bottom) = rect_bounds(snapped[0].outer_rect);

        let center = rect.origin.x() + (rect.area.width() / 2.0);
        assert_approx_eq(left * scale_factor, ((center * scale_factor) - 1.0).round());
        assert_approx_eq(top * scale_factor, (rect.origin.y() * scale_factor).round());
        assert_approx_eq(
            bottom * scale_factor,
            ((rect.origin.y() + rect.area.height()) * scale_factor).round(),
        );
        assert_approx_eq((right - left) * scale_factor, 2.0);
        assert!(!snapped[0].antialias);
    }

    #[test]
    fn vertical_rules_keep_one_physical_pixel_at_common_scales() {
        let rule = paint::Rule {
            axis: paint::Axis::Vertical,
            rect: Rect::new(point::logical(79.0, 20.0), area::logical(2.0, 100.0)),
            brush: paint::Brush::solid(paint::Color::BLACK),
            thickness_px: 1,
        };

        for scale_factor in [1.0, 1.25, 1.5, 2.0] {
            let rect = rasterized_rule_rect(&rule, scale_factor);
            let physical_center = 80.0 * scale_factor;
            let raster_center = (rect.origin.x() + rect.area.width() / 2.0) * scale_factor;

            assert_approx_eq(rect.area.width() * scale_factor, 1.0);
            assert!(
                (raster_center - physical_center).abs() <= 0.5,
                "one-pixel rule must remain centered on the nearest physical pixel"
            );
        }
    }

    #[test]
    fn horizontal_rules_keep_equal_physical_thickness_across_positions() {
        let first = paint::Rule {
            axis: paint::Axis::Horizontal,
            rect: Rect::new(point::logical(10.0, 20.0), area::logical(40.0, 1.0)),
            brush: paint::Brush::solid(paint::Color::BLACK),
            thickness_px: 1,
        };
        let mut second = first;
        second.rect = Rect::new(point::logical(10.0, 21.0), area::logical(40.0, 1.0));

        for scale_factor in [1.0, 1.25, 1.5, 2.0] {
            let first_rect = rasterized_rule_rect(&first, scale_factor);
            let second_rect = rasterized_rule_rect(&second, scale_factor);

            assert_approx_eq(first_rect.area.height() * scale_factor, 1.0);
            assert_approx_eq(second_rect.area.height() * scale_factor, 1.0);
        }
    }

    #[test]
    fn positive_distances_snap_to_at_least_one_physical_pixel() {
        assert_eq!(Grid::new(1.0).snap_distance(0.2), 1.0);
        assert_eq!(Grid::new(2.0).snap_distance(0.2), 0.5);
        assert_eq!(Grid::new(2.0).snap_distance(2.2), 2.0);
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
        assert_eq!(instance_count_for_shape(shapes[0]), 1);
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
                crate::paint::Rounding::relative(1.0),
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
    fn shadow_instance_uses_shadow_shader_mode() {
        let shadow = paint::Shadow {
            rect: rect(),
            brush: paint::Brush::solid(paint::Color::rgba(0.0, 0.0, 0.0, 0.35)),
            blur: 10.0,
            spread: 2.0,
            offset: point::logical(0.0, 6.0),
        };
        let instance = instance_for_shape(analytic_shapes_for_shadow(&shadow)[0]);

        assert_eq!(instance.params, [2.0, 10.0, 0.0, 0.0]);
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

        assert_eq!(shape.outer_rect, quad.rect());
        assert_eq!(prepared.outer_rect, quad.rect());
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
            crate::paint::Rounding::relative(1.0),
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
        assert_eq!(instance_count_for_shape(shapes[0]), 1);
        assert_eq!(bounds(&shapes), (4.0, 14.0, 96.0, 56.0));
        assert_eq!(shapes[0].outer_rounding[0], 21.0);
        assert_eq!(inner.rounding[0], 17.0);
    }

    #[test]
    fn rounded_outline_raster_bounds_include_outline_and_aa_fringe() {
        let rect = Rect::rounded(
            point::logical(10.0, 20.0),
            area::logical(80.0, 30.0),
            crate::paint::Rounding::relative(1.0),
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
    fn external_outline_preparation_keeps_relative_gaps_symmetric() {
        let outline = paint::Outline {
            rect: Rect::new(point::logical(10.0, 20.0), area::logical(80.0, 30.0)),
            brush: paint::Brush::solid(paint::Color::RED),
            width: 1.0,
            offset: 2.0,
        };

        for scale in [1.0, 1.25, 1.5, 1.75, 2.0] {
            let grid = Grid::new(scale);
            let shape = analytic_shapes_for_outline(&outline)[0];
            let prepared = prepared_shape(shape, scale);
            let inner = prepared.inner.expect("outline should stay a ring");
            let base = grid.snap_rect(outline.rect);
            let (base_left, base_top, base_right, base_bottom) = physical_bounds(base, scale);
            let (inner_left, inner_top, inner_right, inner_bottom) =
                physical_bounds(inner.rect, scale);
            let (outer_left, outer_top, outer_right, outer_bottom) =
                physical_bounds(prepared.outer_rect, scale);
            let gap = grid.snap_distance(outline.offset) * scale;
            let width = grid.snap_distance(outline.width) * scale;

            assert_approx_eq(base_left - inner_left, gap);
            assert_approx_eq(base_top - inner_top, gap);
            assert_approx_eq(inner_right - base_right, gap);
            assert_approx_eq(inner_bottom - base_bottom, gap);
            assert_approx_eq(inner_left - outer_left, width);
            assert_approx_eq(inner_top - outer_top, width);
            assert_approx_eq(outer_right - inner_right, width);
            assert_approx_eq(outer_bottom - inner_bottom, width);
            assert!(grid.rect_is_aligned(inner.rect));
            assert!(grid.rect_is_aligned(prepared.outer_rect));
        }
    }

    #[test]
    fn external_outline_default_focus_values_are_centered_at_one_point_five_scale() {
        let scale = 1.5;
        let grid = Grid::new(scale);
        let outline = paint::Outline {
            rect: Rect::new(point::logical(10.0, 20.0), area::logical(80.0, 30.0)),
            brush: paint::Brush::solid(paint::Color::RED),
            width: 1.0,
            offset: 2.0,
        };
        let prepared = prepared_shape(analytic_shapes_for_outline(&outline)[0], scale);
        let inner = prepared.inner.expect("outline should stay a ring");
        let base = grid.snap_rect(outline.rect);
        let (base_left, base_top, base_right, base_bottom) = physical_bounds(base, scale);
        let (inner_left, inner_top, inner_right, inner_bottom) = physical_bounds(inner.rect, scale);
        let (outer_left, outer_top, outer_right, outer_bottom) =
            physical_bounds(prepared.outer_rect, scale);

        // 150% is the regression scale for offset=2 and width=1: offset snaps
        // to a 3dp gap and width snaps to a 1dp ring. Snapping offset+width as
        // one distance would hit a 4.5dp tie and shift the ring.
        assert_approx_eq(base_left - inner_left, 3.0);
        assert_approx_eq(base_top - inner_top, 3.0);
        assert_approx_eq(inner_right - base_right, 3.0);
        assert_approx_eq(inner_bottom - base_bottom, 3.0);
        assert_approx_eq(inner_left - outer_left, 1.0);
        assert_approx_eq(inner_top - outer_top, 1.0);
        assert_approx_eq(outer_right - inner_right, 1.0);
        assert_approx_eq(outer_bottom - inner_bottom, 1.0);
    }

    #[test]
    fn rounded_fill_lowers_to_one_analytic_shape() {
        let quad = paint::Quad::unchecked_for_test(
            Rect::rounded(
                point::logical(10.0, 20.0),
                area::logical(40.0, 40.0),
                crate::paint::Rounding::relative(1.0),
            ),
            style(Some(solid(paint::Color::RED)), None, None),
            paint::Rasterization::default(),
            paint::Transform::identity(),
        );
        let shapes = analytic_shapes_for_quad(&quad);

        assert_eq!(shapes.len(), 1);
        assert_eq!(instance_count_for_shape(shapes[0]), 1);
        assert_eq!(bounds(&shapes), edges(quad.rect()));
        assert_eq!(shapes[0].outer_rounding[0], 20.0);
    }

    #[test]
    fn rounded_tint_uses_same_shape_as_rounded_fill() {
        let rect = Rect::rounded(
            point::logical(10.0, 20.0),
            area::logical(80.0, 30.0),
            crate::paint::Rounding::relative(1.0),
        );
        let fill = fill_shape(rect, paint::Brush::solid(paint::Color::RED));
        let quad = paint::Quad::unchecked_for_test(
            rect,
            style(
                None,
                None,
                Some(paint::Brush::solid(paint::Color::rgba(1.0, 1.0, 1.0, 0.25))),
            ),
            paint::Rasterization::default(),
            paint::Transform::identity(),
        );
        let shapes = analytic_shapes_for_quad(&quad);

        assert_eq!(shapes.len(), 1);
        assert_eq!(shapes[0].outer_rect, fill.outer_rect);
        assert_eq!(shapes[0].outer_rounding, fill.outer_rounding);
        assert_eq!(bounds(&shapes), edges(rect));
    }

    #[test]
    fn rounded_internal_stroke_stays_inside_rect_bounds() {
        let quad = paint::Quad::unchecked_for_test(
            Rect::rounded(
                point::logical(10.0, 20.0),
                area::logical(80.0, 30.0),
                crate::paint::Rounding::relative(1.0),
            ),
            style(None, Some(stroke(4.0)), None),
            paint::Rasterization::default(),
            paint::Transform::identity(),
        );
        let shapes = analytic_shapes_for_quad(&quad);
        let inner = shapes[0].inner.expect("stroke should be a ring");

        assert_eq!(shapes.len(), 1);
        assert_eq!(bounds(&shapes), edges(quad.rect()));
        assert_eq!(
            inner.rect,
            Rect::new(point::logical(14.0, 24.0), area::logical(72.0, 22.0))
        );
        assert_eq!(shapes[0].outer_rounding[0], 15.0);
        assert_eq!(inner.rounding[0], 11.0);
    }
}
