struct VertexIn {
    @location(0) position: vec2<f32>,
    @location(1) local_position: vec2<f32>,
    @location(2) outer_rect: vec4<f32>,
    @location(3) outer_rounding: vec4<f32>,
    @location(4) inner_rect: vec4<f32>,
    @location(5) inner_rounding: vec4<f32>,
    @location(6) color: vec4<f32>,
    @location(7) color_to: vec4<f32>,
    @location(8) brush_points: vec4<f32>,
    @location(9) params: vec4<f32>,
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

@vertex
fn vs_main(in: VertexIn) -> VertexOut {
    var out: VertexOut;
    out.position = vec4<f32>(in.position, 0.0, 1.0);
    out.local_position = in.local_position;
    out.outer_rect = in.outer_rect;
    out.outer_rounding = in.outer_rounding;
    out.inner_rect = in.inner_rect;
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
    let material = brush_color(in);

    if in.params.x > 1.5 {
        let caster_distance = rounded_rect_sdf(
            in.local_position,
            in.outer_rect,
            in.outer_rounding,
        );
        let cutout_alpha = rounded_rect_stable_coverage(
            in.local_position,
            in.inner_rect,
            in.inner_rounding,
        );
        let penumbra = max(
            in.params.y,
            rounded_rect_analytic_width(
                in.local_position,
                in.outer_rect,
                in.outer_rounding,
            ),
        );
        let alpha = (1.0 - smoothstep(-penumbra * 0.5, penumbra * 0.5, caster_distance)) *
            (1.0 - cutout_alpha);

        if alpha <= 0.0 {
            discard;
        }

        return vec4<f32>(material.rgb, material.a * alpha);
    }

    let outer_sdf = rounded_rect_sdf(in.local_position, in.outer_rect, in.outer_rounding);
    let outer_alpha = select(
        rounded_rect_hard_coverage(outer_sdf),
        rounded_rect_stable_coverage(
            in.local_position,
            in.outer_rect,
            in.outer_rounding,
        ),
        in.params.w > 0.5,
    );
    var alpha = outer_alpha;

    if in.params.x > 0.5 {
        let inner_alpha = rounded_rect_stable_coverage(
            in.local_position,
            in.inner_rect,
            in.inner_rounding,
        );
        alpha = alpha * (1.0 - inner_alpha);
    }

    if alpha <= 0.0 {
        discard;
    }

    return vec4<f32>(material.rgb, material.a * alpha);
}
