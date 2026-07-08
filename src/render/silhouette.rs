use crate::paint::{self, Grid, Rect};

pub(crate) const WGSL: &str = r#"
fn silhouette_corner_radius(point: vec2<f32>, rect: vec4<f32>, rounding: vec4<f32>) -> f32 {
    let center = rect.xy + rect.zw * 0.5;

    if point.x < center.x {
        if point.y < center.y {
            return rounding.x;
        }

        return rounding.w;
    }

    if point.y < center.y {
        return rounding.y;
    }

    return rounding.z;
}

fn rounded_rect_sdf(point: vec2<f32>, rect: vec4<f32>, rounding: vec4<f32>) -> f32 {
    let size = max(rect.zw, vec2<f32>(0.0));
    let center = rect.xy + size * 0.5;
    let r = silhouette_corner_radius(point, rect, rounding);
    let q = abs(point - center) - size * 0.5 + vec2<f32>(r);

    return length(max(q, vec2<f32>(0.0))) + min(max(q.x, q.y), 0.0) - r;
}

fn rounded_rect_coverage(sdf: f32) -> f32 {
    let width = max(fwidth(sdf), 0.0001);

    return clamp(0.5 - sdf / width, 0.0, 1.0);
}

fn rounded_rect_hard_coverage(sdf: f32) -> f32 {
    if sdf <= 0.0 {
        return 1.0;
    }

    return 0.0;
}

fn silhouette_signed_nonzero(value: f32) -> f32 {
    return select(-1.0, 1.0, value >= 0.0);
}

fn rounded_rect_normal(point: vec2<f32>, rect: vec4<f32>, rounding: vec4<f32>) -> vec2<f32> {
    let size = max(rect.zw, vec2<f32>(0.0));
    let center = rect.xy + size * 0.5;
    let radius = silhouette_corner_radius(point, rect, rounding);
    let local = point - center;
    let sign = vec2<f32>(silhouette_signed_nonzero(local.x), silhouette_signed_nonzero(local.y));
    let half_extent = max(size * 0.5 - vec2<f32>(radius), vec2<f32>(0.0));
    let q = abs(local) - half_extent;
    let outside = max(q, vec2<f32>(0.0));
    let outside_length = length(outside);

    if outside_length > 0.0001 {
        return outside * sign / outside_length;
    }

    if q.x > q.y {
        return vec2<f32>(sign.x, 0.0);
    }

    return vec2<f32>(0.0, sign.y);
}
"#;

#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) struct PreparedSilhouette {
    pub(crate) raster_rect: Rect,
    pub(crate) shape_rect: Rect,
    pub(crate) rounding: [f32; 4],
}

impl PreparedSilhouette {
    pub(crate) fn for_rect(rect: Rect, grid: Grid, snap: bool, antialias: bool) -> Option<Self> {
        if rect.area.width() <= 0.0 || rect.area.height() <= 0.0 {
            return None;
        }

        let shape_rect = if snap { grid.snap_rect(rect) } else { rect };
        let raster_rect = if antialias {
            expand_rect(shape_rect, grid.logical_pixel())
        } else {
            shape_rect
        };

        Some(Self::from_parts(shape_rect, raster_rect))
    }

    pub(crate) fn for_filter_rect(rect: Rect, scale_factor: f32) -> Option<Self> {
        Self::for_rect(rect, Grid::new(scale_factor), true, true)
    }

    pub(crate) fn from_parts(shape_rect: Rect, raster_rect: Rect) -> Self {
        Self {
            raster_rect,
            shape_rect,
            rounding: shape_rect.rounding.resolve(shape_rect.area),
        }
    }

    #[cfg(test)]
    pub(crate) fn with_rounding(mut self, rounding: [f32; 4]) -> Self {
        self.rounding = clamp_resolved_rounding(rounding, self.shape_rect.area);
        self
    }
}

pub(crate) fn wgsl_module_source(body: &str) -> String {
    format!("{WGSL}\n{body}")
}

pub(crate) fn inset_rect(rect: Rect, inset: f32) -> Option<Rect> {
    let width = rect.area.width() - inset * 2.0;
    let height = rect.area.height() - inset * 2.0;

    if width <= 0.0 || height <= 0.0 {
        return None;
    }

    Some(Rect::new(
        paint::point::logical(rect.origin.x() + inset, rect.origin.y() + inset),
        paint::area::logical(width, height),
    ))
}

pub(crate) fn expand_rect(rect: Rect, amount: f32) -> Rect {
    Rect::new(
        paint::point::logical(rect.origin.x() - amount, rect.origin.y() - amount),
        paint::area::logical(
            rect.area.width() + amount * 2.0,
            rect.area.height() + amount * 2.0,
        ),
    )
}

pub(crate) fn offset_rect(rect: Rect, offset: paint::point::Logical) -> Rect {
    Rect::rounded(
        paint::point::logical(rect.origin.x() + offset.x(), rect.origin.y() + offset.y()),
        rect.area,
        rect.rounding,
    )
}

pub(crate) fn union_rects(a: Rect, b: Rect) -> Rect {
    let (a_left, a_top, a_right, a_bottom) = edges(a);
    let (b_left, b_top, b_right, b_bottom) = edges(b);
    let left = a_left.min(b_left);
    let top = a_top.min(b_top);
    let right = a_right.max(b_right);
    let bottom = a_bottom.max(b_bottom);

    Rect::new(
        paint::point::logical(left, top),
        paint::area::logical(right - left, bottom - top),
    )
}

pub(crate) fn expand_rounding(rounding: [f32; 4], amount: f32) -> [f32; 4] {
    [
        expand_corner_radius(rounding[0], amount),
        expand_corner_radius(rounding[1], amount),
        expand_corner_radius(rounding[2], amount),
        expand_corner_radius(rounding[3], amount),
    ]
}

pub(crate) fn shrink_rounding(rounding: [f32; 4], amount: f32) -> [f32; 4] {
    [
        (rounding[0] - amount).max(0.0),
        (rounding[1] - amount).max(0.0),
        (rounding[2] - amount).max(0.0),
        (rounding[3] - amount).max(0.0),
    ]
}

pub(crate) fn clamp_resolved_rounding(rounding: [f32; 4], area: paint::area::Logical) -> [f32; 4] {
    let width = area.width().max(0.0);
    let height = area.height().max(0.0);
    let scale = 1.0_f32
        .min(edge_scale(width, rounding[0] + rounding[1]))
        .min(edge_scale(width, rounding[3] + rounding[2]))
        .min(edge_scale(height, rounding[0] + rounding[3]))
        .min(edge_scale(height, rounding[1] + rounding[2]));

    if scale >= 1.0 {
        return rounding;
    }

    [
        rounding[0] * scale,
        rounding[1] * scale,
        rounding[2] * scale,
        rounding[3] * scale,
    ]
}

pub(crate) fn clamped_width(rect: Rect, width: f32) -> f32 {
    width
        .max(0.0)
        .min(rect.area.width() / 2.0)
        .min(rect.area.height() / 2.0)
}

pub(crate) fn edges(rect: Rect) -> (f32, f32, f32, f32) {
    let x0 = rect.origin.x();
    let y0 = rect.origin.y();

    (x0, y0, x0 + rect.area.width(), y0 + rect.area.height())
}

pub(crate) fn rect_data(rect: Rect) -> [f32; 4] {
    [
        rect.origin.x(),
        rect.origin.y(),
        rect.area.width(),
        rect.area.height(),
    ]
}

pub(crate) fn rounding_data(rounding: [f32; 4]) -> [f32; 4] {
    rounding
}

fn expand_corner_radius(rounding: f32, amount: f32) -> f32 {
    if rounding <= 0.0 {
        0.0
    } else {
        rounding + amount
    }
}

fn edge_scale(edge: f32, radius_sum: f32) -> f32 {
    if radius_sum <= edge || radius_sum <= 0.0 {
        1.0
    } else {
        edge / radius_sum
    }
}

#[cfg(test)]
mod tests {
    use crate::paint;
    use crate::render;
    use crate::render::{filter, quad};

    use super::*;

    #[test]
    fn fixed_rounding_is_clamped_to_snapped_shape_area() {
        let rect = Rect::rounded(
            paint::point::logical(10.2, 20.3),
            paint::area::logical(9.6, 4.2),
            paint::Rounding::fixed(10.0),
        );
        let prepared =
            PreparedSilhouette::for_filter_rect(rect, 2.0).expect("silhouette should prepare");

        assert_eq!(edges(prepared.shape_rect), (10.0, 20.5, 20.0, 24.5));
        assert_eq!(prepared.rounding, [2.0, 2.0, 2.0, 2.0]);
    }

    #[test]
    fn relative_rounding_resolves_after_snap() {
        let rect = Rect::rounded(
            paint::point::logical(10.2, 20.3),
            paint::area::logical(40.4, 30.8),
            paint::Rounding::relative(1.0),
        );
        let prepared =
            PreparedSilhouette::for_filter_rect(rect, 2.0).expect("silhouette should prepare");

        assert_eq!(edges(prepared.shape_rect), (10.0, 20.5, 50.5, 51.0));
        assert_eq!(prepared.rounding, [15.25, 15.25, 15.25, 15.25]);
    }

    #[test]
    fn shader_source_wraps_body_with_shared_silhouette_functions() {
        let source = wgsl_module_source("fn body() {}\n");

        assert!(source.contains("fn rounded_rect_sdf("));
        assert!(source.contains("fn rounded_rect_coverage("));
        assert!(source.contains("fn rounded_rect_hard_coverage("));
        assert!(source.contains("fn rounded_rect_normal("));
        assert!(source.ends_with("fn body() {}\n"));
    }

    #[test]
    fn quad_filter_and_clip_silhouettes_share_snapped_panel_edge() {
        for rounding in [paint::Rounding::fixed(10.0), paint::Rounding::relative(1.0)] {
            let rect = Rect::rounded(
                paint::point::logical(10.2, 20.3),
                paint::area::logical(40.4, 30.8),
                rounding,
            );

            for scale_factor in [1.0, 2.0] {
                let quad = quad::prepared_fill_silhouette_for_test(rect, scale_factor);
                let filter = filter::prepared_filter_silhouette_for_test(rect, scale_factor)
                    .expect("filter silhouette should prepare");
                let clip = filter::prepared_clip_silhouette_for_test(rect, scale_factor)
                    .expect("clip silhouette should prepare");

                assert_eq!(quad, filter);
                assert_eq!(quad, clip);
            }
        }
    }

    #[test]
    fn small_fixed_radius_silhouettes_clamp_identically() {
        let rect = Rect::rounded(
            paint::point::logical(10.2, 20.3),
            paint::area::logical(9.6, 4.2),
            paint::Rounding::fixed(10.0),
        );

        for scale_factor in [1.0, 2.0] {
            let quad = quad::prepared_fill_silhouette_for_test(rect, scale_factor);
            let filter = filter::prepared_filter_silhouette_for_test(rect, scale_factor)
                .expect("filter silhouette should prepare");

            assert_eq!(quad, filter);
            assert_eq!(
                quad.rounding,
                paint::Rounding::fixed(10.0).resolve(quad.shape_rect.area)
            );
        }
    }

    #[test]
    fn shadow_cutout_uses_same_silhouette_as_owner_fill() {
        let rect = Rect::rounded(
            paint::point::logical(10.2, 20.3),
            paint::area::logical(40.4, 30.8),
            paint::Rounding::relative(1.0),
        );
        let shadow = paint::Shadow {
            rect,
            brush: paint::Brush::solid(paint::Color::rgba(0.0, 0.0, 0.0, 0.35)),
            blur: 18.0,
            spread: 2.0,
            offset: paint::point::logical(0.0, 6.0),
        };

        for scale_factor in [1.0, 2.0] {
            let fill = quad::prepared_fill_silhouette_for_test(rect, scale_factor);
            let cutout = quad::prepared_shadow_cutout_silhouette_for_test(shadow, scale_factor);

            assert_eq!(cutout.shape_rect, fill.shape_rect);
            assert_eq!(cutout.raster_rect, fill.raster_rect);
            assert_eq!(cutout.rounding, fill.rounding);
        }
    }

    #[test]
    fn quad_and_filter_shader_sources_define_silhouette_once() {
        for source in [quad::shader_source(), filter::shader_source()] {
            assert_eq!(source.matches("fn rounded_rect_sdf(").count(), 1);
            assert_eq!(source.matches("fn rounded_rect_coverage(").count(), 1);
            assert_eq!(source.matches("fn rounded_rect_hard_coverage(").count(), 1);
            assert_eq!(source.matches("fn rounded_rect_normal(").count(), 1);
            assert!(!source.contains("fn coverage("));
            assert!(!source.contains("fn hard_coverage("));
            assert!(!source.contains("fn signed_nonzero("));
        }
    }

    #[test]
    #[ignore = "requires a GPU adapter; run with the renderer smoke tier"]
    fn shared_silhouette_shaders_compile_in_wgpu_pipelines() {
        let context = pollster::block_on(render::Context::new(render::ContextOptions {
            device_label: "wgpu_l3 silhouette shader smoke device",
            backends: wgpu::Backends::all(),
            power_preference: wgpu::PowerPreference::HighPerformance,
            force_fallback_adapter: false,
            required_features: wgpu::Features::empty(),
            required_limits: wgpu::Limits::default(),
        }))
        .expect("render context should initialize");

        let _renderer = render::Renderer::new(&context, wgpu::TextureFormat::Bgra8UnormSrgb);
    }
}
