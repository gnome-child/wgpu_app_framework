struct Params {
    texture_size: vec2<f32>,
    canvas_size: vec2<f32>,
    direction_radius: vec4<f32>,
    rect: vec4<f32>,
    rounding: vec4<f32>,
};

@group(0) @binding(0) var source_texture: texture_2d<f32>;
@group(0) @binding(1) var source_sampler: sampler;
@group(0) @binding(2) var<uniform> params: Params;

struct FullscreenOut {
    @builtin(position) position: vec4<f32>,
    @location(0) uv: vec2<f32>,
};

struct CompositeIn {
    @location(0) position: vec2<f32>,
    @location(1) local_position: vec2<f32>,
    @location(2) rect: vec4<f32>,
    @location(3) rounding: vec4<f32>,
};

struct CompositeOut {
    @builtin(position) position: vec4<f32>,
    @location(0) local_position: vec2<f32>,
    @location(1) rect: vec4<f32>,
    @location(2) rounding: vec4<f32>,
};

@vertex
fn vs_fullscreen(@builtin(vertex_index) index: u32) -> FullscreenOut {
    let positions = array<vec2<f32>, 3>(
        vec2<f32>(-1.0, -3.0),
        vec2<f32>(-1.0, 1.0),
        vec2<f32>(3.0, 1.0),
    );
    let uvs = array<vec2<f32>, 3>(
        vec2<f32>(0.0, 2.0),
        vec2<f32>(0.0, 0.0),
        vec2<f32>(2.0, 0.0),
    );

    var out: FullscreenOut;
    out.position = vec4<f32>(positions[index], 0.0, 1.0);
    out.uv = uvs[index];
    return out;
}

@vertex
fn vs_composite(in: CompositeIn) -> CompositeOut {
    var out: CompositeOut;
    out.position = vec4<f32>(in.position, 0.0, 1.0);
    out.local_position = in.local_position;
    out.rect = in.rect;
    out.rounding = in.rounding;
    return out;
}

fn corner_radius(point: vec2<f32>, rect: vec4<f32>, rounding: vec4<f32>) -> f32 {
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
    let r = corner_radius(point, rect, rounding);
    let q = abs(point - center) - size * 0.5 + vec2<f32>(r);

    return length(max(q, vec2<f32>(0.0))) + min(max(q.x, q.y), 0.0) - r;
}

fn coverage(sdf: f32) -> f32 {
    let width = max(fwidth(sdf), 0.0001);

    return clamp(0.5 - sdf / width, 0.0, 1.0);
}

@fragment
fn fs_blur(in: FullscreenOut) -> @location(0) vec4<f32> {
    let direction = params.direction_radius.xy;
    let texel = direction / max(params.texture_size, vec2<f32>(1.0));
    let material_radius_px = clamp(params.direction_radius.z, 0.0, 96.0);
    let sigma = max(material_radius_px * 0.42, 0.001);

    var color = textureSample(source_texture, source_sampler, in.uv);
    var total_weight = 1.0;

    for (var i: i32 = 1; i <= 96; i = i + 2) {
        let offset_a = f32(i);
        if offset_a > material_radius_px {
            break;
        }

        let offset_b = min(offset_a + 1.0, material_radius_px);
        let weight_a = exp(-0.5 * (offset_a / sigma) * (offset_a / sigma));
        let weight_b = select(
            exp(-0.5 * (offset_b / sigma) * (offset_b / sigma)),
            0.0,
            offset_b <= offset_a,
        );
        let pair_weight = weight_a + weight_b;
        let pair_offset = (offset_a * weight_a + offset_b * weight_b) / pair_weight;

        color += textureSample(source_texture, source_sampler, in.uv + texel * pair_offset) * pair_weight;
        color += textureSample(source_texture, source_sampler, in.uv - texel * pair_offset) * pair_weight;
        total_weight += pair_weight * 2.0;
    }

    return color / total_weight;
}

@fragment
fn fs_blit(in: FullscreenOut) -> @location(0) vec4<f32> {
    return textureSample(source_texture, source_sampler, in.uv);
}

@fragment
fn fs_composite(in: CompositeOut) -> @location(0) vec4<f32> {
    let alpha = coverage(rounded_rect_sdf(in.local_position, in.rect, in.rounding));

    if alpha <= 0.0 {
        discard;
    }

    let scale_factor = params.direction_radius.w;
    let uv = in.local_position * scale_factor / max(params.texture_size, vec2<f32>(1.0));
    let color = textureSample(source_texture, source_sampler, uv);
    let rgb = select(color.rgb / max(color.a, 0.0001), vec3<f32>(0.0), color.a <= 0.0001);

    return vec4<f32>(rgb, color.a * alpha);
}
