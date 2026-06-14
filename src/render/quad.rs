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

#[derive(Debug, Clone, PartialEq)]
struct DynamicSolidMesh {
    points: Vec<point::Logical>,
    color: paint::Color,
}

const CORNER_SEGMENTS: usize = 12;

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

    for mesh in solid_meshes_for_shape(shape) {
        push_mesh_vertices(buffer, canvas_area, &mesh);
    }
}

fn solid_meshes_for_shape(shape: &batch::Shape<'_>) -> Vec<DynamicSolidMesh> {
    match shape {
        batch::Shape::Quad(quad) => solid_meshes_for_quad(quad),
        batch::Shape::Tint(tint) => solid_meshes_for_tint(tint),
        batch::Shape::Outline(outline) => solid_meshes_for_outline(outline),
        batch::Shape::BackdropBlur(blur) => {
            log::debug!("skipping unsupported backdrop blur: {blur:?}");
            Vec::new()
        }
    }
}

fn solid_meshes_for_quad(quad: &paint::Quad) -> Vec<DynamicSolidMesh> {
    let rect = quad.rect;
    let mut meshes = Vec::new();

    if let Some(fill) = quad.style.fill {
        match fill {
            paint::Fill::Brush(brush) => {
                if let Some(color) = solid_color(brush) {
                    meshes.push(solid_mesh_for_rect(rect, color));
                }
            }
            paint::Fill::Blur => {
                log::debug!("skipping unsupported blur quad fill");
            }
        }
    }

    if let Some(stroke) = quad.style.stroke {
        if let Some(color) = solid_color(stroke.brush) {
            if rect.radius.resolve(rect.area).is_none() {
                for rect in internal_stroke_rects(rect, stroke.width) {
                    meshes.push(solid_mesh_for_rect(rect, color));
                }
            } else if let Some(mesh) = rounded_stroke_mesh(rect, stroke.width, color) {
                meshes.push(mesh);
            }
        }
    }

    if let Some(color) = quad.style.tint {
        meshes.push(solid_mesh_for_rect(rect, color));
    }

    meshes
}

fn solid_meshes_for_tint(tint: &paint::Tint) -> Vec<DynamicSolidMesh> {
    vec![solid_mesh_for_rect(tint.rect, tint.color)]
}

fn solid_meshes_for_outline(outline: &paint::Outline) -> Vec<DynamicSolidMesh> {
    let Some(color) = solid_color(outline.brush) else {
        return Vec::new();
    };

    if outline.rect.radius.resolve(outline.rect.area).is_none() {
        return external_outline_rects(outline.rect, outline.width, outline.offset)
            .into_iter()
            .map(|rect| solid_mesh_for_rect(rect, color))
            .collect();
    }

    rounded_outline_mesh(outline.rect, outline.width, outline.offset, color)
        .into_iter()
        .collect()
}

fn push_mesh_vertices(
    buffer: &mut Vec<render::primitive::Vertex>,
    canvas_area: area::Logical,
    mesh: &DynamicSolidMesh,
) {
    let to_clip = |x: f32, y: f32| -> [f32; 2] {
        [
            (x / canvas_area.width()) * 2.0 - 1.0,
            1.0 - (y / canvas_area.height()) * 2.0,
        ]
    };

    let color = mesh.color.to_array();

    buffer.extend(mesh.points.iter().map(|point| render::primitive::Vertex {
        position: to_clip(point.x(), point.y()),
        color,
    }));
}

fn solid_mesh_for_rect(rect: Rect, color: paint::Color) -> DynamicSolidMesh {
    let radius = rect.radius.resolve(rect.area);
    let points = if radius.is_none() {
        rect_vertices(rect)
    } else {
        rounded_fill_vertices(rect, radius)
    };

    DynamicSolidMesh { points, color }
}

fn rect_vertices(rect: Rect) -> Vec<point::Logical> {
    let (x0, y0, x1, y1) = edges(rect);

    vec![
        point::logical(x0, y0),
        point::logical(x0, y1),
        point::logical(x1, y1),
        point::logical(x0, y0),
        point::logical(x1, y1),
        point::logical(x1, y0),
    ]
}

fn rounded_fill_vertices(
    rect: Rect,
    radius: crate::geometry::rect::ResolvedRadius,
) -> Vec<point::Logical> {
    let boundary = rounded_boundary(rect, radius);
    let (x0, y0, x1, y1) = edges(rect);
    let center = point::logical((x0 + x1) / 2.0, (y0 + y1) / 2.0);
    let mut vertices = Vec::with_capacity(boundary.len() * 3);

    for index in 0..boundary.len() {
        let next = (index + 1) % boundary.len();

        vertices.push(center);
        vertices.push(boundary[index]);
        vertices.push(boundary[next]);
    }

    vertices
}

fn rounded_stroke_mesh(rect: Rect, width: f32, color: paint::Color) -> Option<DynamicSolidMesh> {
    let width = clamped_width(rect, width);
    if width <= 0.0 {
        return None;
    }

    let outer_radius = rect.radius.resolve(rect.area);
    let Some(inner_rect) = inset_rect(rect, width) else {
        return Some(solid_mesh_for_rect(rect, color));
    };
    let inner_radius = crate::geometry::rect::ResolvedRadius {
        top_left: (outer_radius.top_left - width).max(0.0),
        top_right: (outer_radius.top_right - width).max(0.0),
        bottom_left: (outer_radius.bottom_left - width).max(0.0),
        bottom_right: (outer_radius.bottom_right - width).max(0.0),
    };

    Some(rounded_ring_mesh(
        rect,
        outer_radius,
        inner_rect,
        inner_radius,
        color,
    ))
}

fn rounded_outline_mesh(
    rect: Rect,
    width: f32,
    offset: f32,
    color: paint::Color,
) -> Option<DynamicSolidMesh> {
    if width <= 0.0 {
        return None;
    }

    let offset = offset.max(0.0);
    let base_radius = rect.radius.resolve(rect.area);
    let inner_rect = expand_rect(rect, offset);
    let outer_rect = expand_rect(rect, offset + width);
    let inner_radius = expand_radius(base_radius, offset);
    let outer_radius = expand_radius(base_radius, offset + width);

    Some(rounded_ring_mesh(
        outer_rect,
        outer_radius,
        inner_rect,
        inner_radius,
        color,
    ))
}

fn rounded_ring_mesh(
    outer_rect: Rect,
    outer_radius: crate::geometry::rect::ResolvedRadius,
    inner_rect: Rect,
    inner_radius: crate::geometry::rect::ResolvedRadius,
    color: paint::Color,
) -> DynamicSolidMesh {
    let outer = rounded_boundary(outer_rect, outer_radius);
    let inner = rounded_boundary(inner_rect, inner_radius);
    let mut points = Vec::with_capacity(outer.len() * 6);

    for index in 0..outer.len() {
        let next = (index + 1) % outer.len();

        points.push(outer[index]);
        points.push(inner[index]);
        points.push(inner[next]);
        points.push(outer[index]);
        points.push(inner[next]);
        points.push(outer[next]);
    }

    DynamicSolidMesh { points, color }
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

fn rounded_boundary(
    rect: Rect,
    radius: crate::geometry::rect::ResolvedRadius,
) -> Vec<point::Logical> {
    let (x0, y0, x1, y1) = edges(rect);
    let mut points = Vec::with_capacity((CORNER_SEGMENTS + 1) * 4);

    push_corner(
        &mut points,
        point::logical(x0 + radius.top_left, y0 + radius.top_left),
        radius.top_left,
        std::f32::consts::PI,
        std::f32::consts::PI * 1.5,
        point::logical(x0, y0),
    );
    push_corner(
        &mut points,
        point::logical(x1 - radius.top_right, y0 + radius.top_right),
        radius.top_right,
        std::f32::consts::PI * 1.5,
        std::f32::consts::PI * 2.0,
        point::logical(x1, y0),
    );
    push_corner(
        &mut points,
        point::logical(x1 - radius.bottom_right, y1 - radius.bottom_right),
        radius.bottom_right,
        0.0,
        std::f32::consts::PI * 0.5,
        point::logical(x1, y1),
    );
    push_corner(
        &mut points,
        point::logical(x0 + radius.bottom_left, y1 - radius.bottom_left),
        radius.bottom_left,
        std::f32::consts::PI * 0.5,
        std::f32::consts::PI,
        point::logical(x0, y1),
    );

    points
}

fn push_corner(
    points: &mut Vec<point::Logical>,
    center: point::Logical,
    radius: f32,
    start_angle: f32,
    end_angle: f32,
    fallback: point::Logical,
) {
    for segment in 0..=CORNER_SEGMENTS {
        if radius <= 0.0 {
            points.push(fallback);
            continue;
        }

        let t = segment as f32 / CORNER_SEGMENTS as f32;
        let angle = start_angle + (end_angle - start_angle) * t;
        points.push(point::logical(
            center.x() + angle.cos() * radius,
            center.y() + angle.sin() * radius,
        ));
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
        solid_meshes_for_quad(quad)
            .iter()
            .map(|mesh| mesh.points.len())
            .sum()
    }

    fn bounds(meshes: &[DynamicSolidMesh]) -> (f32, f32, f32, f32) {
        meshes.iter().flat_map(|mesh| mesh.points.iter()).fold(
            (
                f32::INFINITY,
                f32::INFINITY,
                f32::NEG_INFINITY,
                f32::NEG_INFINITY,
            ),
            |bounds, point| {
                (
                    bounds.0.min(point.x()),
                    bounds.1.min(point.y()),
                    bounds.2.max(point.x()),
                    bounds.3.max(point.y()),
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
        let meshes = solid_meshes_for_quad(&quad);

        assert_eq!(meshes.len(), 4);
        assert_eq!(vertex_count_for_quad(&quad), 24);
        assert_eq!(bounds(&meshes), edges(rect()));
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
        let meshes = solid_meshes_for_quad(&quad);

        assert_eq!(meshes.len(), 6);
        assert_eq!(vertex_count_for_quad(&quad), 36);
        assert_eq!(meshes[0].color, paint::Color::RED);
        assert_eq!(meshes[5].color, paint::Color::rgba(1.0, 1.0, 1.0, 0.25));
    }

    #[test]
    fn outline_item_expands_outside_rect_bounds() {
        let outline = paint::Outline {
            rect: rect(),
            brush: paint::Brush::Solid(paint::Color::RED),
            width: 2.0,
            offset: 3.0,
        };
        let meshes = solid_meshes_for_outline(&outline);

        assert_eq!(meshes.len(), 4);
        assert_eq!(bounds(&meshes), (5.0, 15.0, 55.0, 55.0));
    }

    #[test]
    fn rounded_outline_renders_as_expanded_rounded_ring() {
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
        let meshes = solid_meshes_for_outline(&outline);

        assert_eq!(meshes.len(), 1);
        assert!(meshes[0].points.len() > 24);
        assert_eq!(bounds(&meshes), (4.0, 14.0, 96.0, 56.0));
        assert!(
            meshes[0]
                .points
                .iter()
                .any(|point| point.x() == 4.0 && point.y() > 14.0 && point.y() < 56.0)
        );
    }

    #[test]
    fn rounded_fill_generates_mesh_inside_rect_bounds() {
        let quad = paint::Quad {
            rect: Rect::rounded(
                point::logical(10.0, 20.0),
                area::logical(40.0, 40.0),
                crate::geometry::rect::Radius::splat(1.0),
            ),
            style: style(Some(solid(paint::Color::RED)), None, None),
        };
        let meshes = solid_meshes_for_quad(&quad);

        assert_eq!(meshes.len(), 1);
        assert!(meshes[0].points.len() > 6);
        assert_eq!(bounds(&meshes), edges(quad.rect));
    }

    #[test]
    fn rounded_tint_uses_same_geometry_as_rounded_fill() {
        let rect = Rect::rounded(
            point::logical(10.0, 20.0),
            area::logical(80.0, 30.0),
            crate::geometry::rect::Radius::splat(1.0),
        );
        let fill = solid_mesh_for_rect(rect, paint::Color::RED);
        let tint = paint::Tint {
            rect,
            color: paint::Color::rgba(1.0, 1.0, 1.0, 0.25),
        };
        let meshes = solid_meshes_for_tint(&tint);

        assert_eq!(meshes.len(), 1);
        assert_eq!(meshes[0].points, fill.points);
        assert_eq!(bounds(&meshes), edges(rect));
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
        let meshes = solid_meshes_for_quad(&quad);

        assert_eq!(meshes.len(), 1);
        assert!(meshes[0].points.len() > 24);
        assert_eq!(bounds(&meshes), edges(quad.rect));
    }

    #[test]
    fn backdrop_blur_generates_no_solid_rects() {
        let blur = paint::Blur {
            rect: rect(),
            radius: 8.0,
        };

        assert!(solid_meshes_for_shape(&batch::Shape::BackdropBlur(&blur)).is_empty());
    }
}
