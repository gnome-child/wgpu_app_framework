struct VertexIn {
    @location(0) position: vec2<f32>,
    @location(1) local_position: vec2<f32>,
    @location(2) outer_rect: vec4<f32>,
    @location(3) outer_radius: vec4<f32>,
    @location(4) inner_rect: vec4<f32>,
    @location(5) inner_radius: vec4<f32>,
    @location(6) color: vec4<f32>,
    @location(7) params: vec4<f32>,
};

struct VertexOut {
    @builtin(position) position: vec4<f32>,
    @location(0) local_position: vec2<f32>,
    @location(1) outer_rect: vec4<f32>,
    @location(2) outer_radius: vec4<f32>,
    @location(3) inner_rect: vec4<f32>,
    @location(4) inner_radius: vec4<f32>,
    @location(5) color: vec4<f32>,
    @location(6) params: vec4<f32>,
};

@vertex
fn vs_main(in: VertexIn) -> VertexOut {
    var out: VertexOut;
    out.position = vec4<f32>(in.position, 0.0, 1.0);
    out.local_position = in.local_position;
    out.outer_rect = in.outer_rect;
    out.outer_radius = in.outer_radius;
    out.inner_rect = in.inner_rect;
    out.inner_radius = in.inner_radius;
    out.color = in.color;
    out.params = in.params;
    return out;
}

fn corner_radius(point: vec2<f32>, rect: vec4<f32>, radius: vec4<f32>) -> f32 {
    let center = rect.xy + rect.zw * 0.5;

    if point.x < center.x {
        if point.y < center.y {
            return radius.x;
        }

        return radius.w;
    }

    if point.y < center.y {
        return radius.y;
    }

    return radius.z;
}

fn rounded_rect_sdf(point: vec2<f32>, rect: vec4<f32>, radius: vec4<f32>) -> f32 {
    let size = max(rect.zw, vec2<f32>(0.0));
    let center = rect.xy + size * 0.5;
    let r = corner_radius(point, rect, radius);
    let q = abs(point - center) - size * 0.5 + vec2<f32>(r);

    return length(max(q, vec2<f32>(0.0))) + min(max(q.x, q.y), 0.0) - r;
}

fn coverage(sdf: f32) -> f32 {
    let width = max(fwidth(sdf), 0.0001);

    return clamp(0.5 - sdf / width, 0.0, 1.0);
}

@fragment
fn fs_main(in: VertexOut) -> @location(0) vec4<f32> {
    let outer_alpha = coverage(rounded_rect_sdf(
        in.local_position,
        in.outer_rect,
        in.outer_radius,
    ));
    var alpha = outer_alpha;

    if in.params.x > 0.5 {
        let inner_alpha = coverage(rounded_rect_sdf(
            in.local_position,
            in.inner_rect,
            in.inner_radius,
        ));
        alpha = alpha * (1.0 - inner_alpha);
    }

    if alpha <= 0.0 {
        discard;
    }

    return vec4<f32>(in.color.rgb, in.color.a * alpha);
}
