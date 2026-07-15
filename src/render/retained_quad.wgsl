struct Viewport {
    size: vec2<f32>,
    _padding: vec2<f32>,
};

struct NodeProperty {
    origin: vec2<f32>,
    translate: vec2<f32>,
    scroll: vec2<f32>,
    scale: vec2<f32>,
    grid: vec2<f32>,
    scene_origin: vec2<f32>,
    target_size: vec2<f32>,
    opacity: f32,
    _padding_scalar: f32,
    _padding_vector: vec2<f32>,
};

@group(0) @binding(0) var<uniform> viewport: Viewport;
@group(0) @binding(1) var<uniform> node_property: NodeProperty;

struct VertexIn {
    @location(0) corner: vec2<f32>,
    @location(1) raster_rect: vec4<f32>,
    @location(2) outer_rect: vec4<f32>,
    @location(3) outer_rounding: vec4<f32>,
    @location(4) inner_rect: vec4<f32>,
    @location(5) inner_rounding: vec4<f32>,
    @location(6) color: vec4<f32>,
    @location(7) color_to: vec4<f32>,
    @location(8) brush_points: vec4<f32>,
    @location(9) params: vec4<f32>,
    @location(10) source_rect: vec4<f32>,
};

struct VertexOut {
    @builtin(position) position: vec4<f32>,
    @location(0) local_position: vec2<f32>,
    @location(1) outer_rect: vec4<f32>,
    @location(2) outer_rounding: vec4<f32>,
    @location(3) inner_rect: vec4<f32>,
    @location(4) inner_rounding: vec4<f32>,
    @location(5) color: vec4<f32>,
    @location(6) color_to: vec4<f32>,
    @location(7) brush_points: vec4<f32>,
    @location(8) params: vec4<f32>,
};

fn transform_point(point: vec2<f32>) -> vec2<f32> {
    return (point - node_property.origin) * node_property.scale +
        node_property.origin + node_property.translate + node_property.scroll -
        node_property.scene_origin;
}

fn transform_rect(rect: vec4<f32>) -> vec4<f32> {
    let first = transform_point(rect.xy);
    let second = transform_point(rect.xy + rect.zw);
    let minimum = min(first, second);
    let maximum = max(first, second);
    return vec4<f32>(minimum, maximum - minimum);
}

fn transform_scrolled_point(point: vec2<f32>) -> vec2<f32> {
    let origin = node_property.origin + node_property.scroll;
    return (point - origin) * node_property.scale + origin + node_property.translate -
        node_property.scene_origin;
}

fn transform_scrolled_rect(rect: vec4<f32>) -> vec4<f32> {
    let first = transform_scrolled_point(rect.xy);
    let second = transform_scrolled_point(rect.xy + rect.zw);
    let minimum = min(first, second);
    let maximum = max(first, second);
    return vec4<f32>(minimum, maximum - minimum);
}

fn round_ties_toward_zero(value: f32) -> f32 {
    let truncated = trunc(value);
    let fraction = abs(value - truncated);
    return select(round(value), truncated, abs(fraction - 0.5) <= 0.00001);
}

fn snap_distance(distance: f32) -> f32 {
    let scale = max(node_property.grid.x, 0.0001);
    return max(round_ties_toward_zero(distance * scale), 1.0) / scale;
}

fn snap_position(value: f32) -> f32 {
    let scale = max(node_property.grid.x, 0.0001);
    return round_ties_toward_zero(value * scale) / scale;
}

fn snap_rect(rect: vec4<f32>) -> vec4<f32> {
    let pixel = 1.0 / max(node_property.grid.x, 0.0001);
    let left = snap_position(rect.x);
    let top = snap_position(rect.y);
    let right = max(snap_position(rect.x + rect.z), left + pixel);
    let bottom = max(snap_position(rect.y + rect.w), top + pixel);
    return vec4<f32>(left, top, right - left, bottom - top);
}

fn snap_span(start: f32, distance: f32) -> vec2<f32> {
    let scale = max(node_property.grid.x, 0.0001);
    let snapped_distance = snap_distance(distance);
    let physical_distance = snapped_distance * scale;
    let physical_center = (start + distance * 0.5) * scale;
    let physical_start = round_ties_toward_zero(physical_center - physical_distance * 0.5);
    return vec2<f32>(physical_start / scale, (physical_start + physical_distance) / scale);
}

fn snap_rect_with_stable_size(rect: vec4<f32>) -> vec4<f32> {
    let horizontal = snap_span(rect.x, rect.z);
    let vertical = snap_span(rect.y, rect.w);
    return vec4<f32>(
        horizontal.x,
        vertical.x,
        horizontal.y - horizontal.x,
        vertical.y - vertical.x,
    );
}

@vertex
fn vs_main(in: VertexIn) -> VertexOut {
    var outer_rect = transform_rect(in.outer_rect);
    var raster_rect = transform_rect(in.raster_rect);
    if node_property.grid.y > 0.5 && in.params.x < 0.5 {
        let source_rect = snap_rect(vec4<f32>(
            in.source_rect.xy + node_property.scroll,
            in.source_rect.zw,
        ));
        outer_rect = snap_rect_with_stable_size(transform_scrolled_rect(source_rect));
        let pixel = 1.0 / max(node_property.grid.x, 0.0001);
        raster_rect = vec4<f32>(
            outer_rect.xy - vec2<f32>(pixel),
            outer_rect.zw + vec2<f32>(pixel * 2.0),
        );
    }
    let transformed = raster_rect.xy + in.corner * raster_rect.zw;
    let clip = vec2<f32>(
        (transformed.x / node_property.target_size.x) * 2.0 - 1.0,
        1.0 - (transformed.y / node_property.target_size.y) * 2.0,
    );

    var out: VertexOut;
    out.position = vec4<f32>(clip, 0.0, 1.0);
    out.local_position = transformed;
    out.outer_rect = outer_rect;
    out.outer_rounding = in.outer_rounding;
    out.inner_rect = transform_rect(in.inner_rect);
    out.inner_rounding = in.inner_rounding;
    out.color = in.color;
    out.color_to = in.color_to;
    out.brush_points = in.brush_points;
    out.params = in.params;
    return out;
}

fn brush_color(in: VertexOut) -> vec4<f32> {
    if in.params.z <= 0.5 {
        return in.color;
    }

    let start = in.outer_rect.xy + in.brush_points.xy * in.outer_rect.zw;
    let end = in.outer_rect.xy + in.brush_points.zw * in.outer_rect.zw;
    let axis = end - start;
    let denominator = max(dot(axis, axis), 0.0001);
    let t = clamp(dot(in.local_position - start, axis) / denominator, 0.0, 1.0);

    return mix(in.color, in.color_to, t);
}

@fragment
fn fs_main(in: VertexOut) -> @location(0) vec4<f32> {
    if node_property.opacity <= 0.0 {
        discard;
    }
    let material = brush_color(in);

    if in.params.x > 1.5 {
        let caster_distance = rounded_rect_sdf(
            in.local_position,
            in.outer_rect,
            in.outer_rounding,
        );
        let cutout_alpha = rounded_rect_coverage(rounded_rect_sdf(
            in.local_position,
            in.inner_rect,
            in.inner_rounding,
        ));
        let penumbra = max(in.params.y, max(fwidth(caster_distance), 0.0001));
        let alpha = (1.0 - smoothstep(-penumbra * 0.5, penumbra * 0.5, caster_distance)) *
            (1.0 - cutout_alpha);

        if alpha <= 0.0 {
            discard;
        }

        return vec4<f32>(material.rgb, material.a * alpha * node_property.opacity);
    }

    let outer_sdf = rounded_rect_sdf(in.local_position, in.outer_rect, in.outer_rounding);
    let outer_alpha = select(
        rounded_rect_hard_coverage(outer_sdf),
        rounded_rect_coverage(outer_sdf),
        in.params.w > 0.5,
    );
    var alpha = outer_alpha;

    if in.params.x > 0.5 {
        let inner_alpha = rounded_rect_coverage(rounded_rect_sdf(
            in.local_position,
            in.inner_rect,
            in.inner_rounding,
        ));
        alpha = alpha * (1.0 - inner_alpha);
    }

    if alpha <= 0.0 {
        discard;
    }

    return vec4<f32>(material.rgb, material.a * alpha * node_property.opacity);
}
