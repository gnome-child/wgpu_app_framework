struct Params {
    texture_size: vec2<f32>,
    canvas_size: vec2<f32>,
    direction_radius: vec4<f32>,
    rect: vec4<f32>,
    radius: vec4<f32>,
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
    @location(3) radius: vec4<f32>,
};

struct CompositeOut {
    @builtin(position) position: vec4<f32>,
    @location(0) local_position: vec2<f32>,
    @location(1) rect: vec4<f32>,
    @location(2) radius: vec4<f32>,
};

@vertex
fn vs_fullscreen(@builtin(vertex_index) index: u32) -> FullscreenOut {
    let positions = array<vec2<f32>, 6>(
        vec2<f32>(-1.0, 1.0),
        vec2<f32>(-1.0, -1.0),
        vec2<f32>(1.0, -1.0),
        vec2<f32>(-1.0, 1.0),
        vec2<f32>(1.0, -1.0),
        vec2<f32>(1.0, 1.0),
    );
    let uvs = array<vec2<f32>, 6>(
        vec2<f32>(0.0, 0.0),
        vec2<f32>(0.0, 1.0),
        vec2<f32>(1.0, 1.0),
        vec2<f32>(0.0, 0.0),
        vec2<f32>(1.0, 1.0),
        vec2<f32>(1.0, 0.0),
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
    out.radius = in.radius;
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
fn fs_blur(in: FullscreenOut) -> @location(0) vec4<f32> {
    let direction = params.direction_radius.xy;
    let radius = max(params.direction_radius.z, 0.0);
    let texel = direction * radius / max(params.texture_size, vec2<f32>(1.0));
    var color = textureSample(source_texture, source_sampler, in.uv) * 0.227027;

    color += textureSample(source_texture, source_sampler, in.uv + texel * 0.25) * 0.1945946;
    color += textureSample(source_texture, source_sampler, in.uv - texel * 0.25) * 0.1945946;
    color += textureSample(source_texture, source_sampler, in.uv + texel * 0.50) * 0.1216216;
    color += textureSample(source_texture, source_sampler, in.uv - texel * 0.50) * 0.1216216;
    color += textureSample(source_texture, source_sampler, in.uv + texel * 0.75) * 0.054054;
    color += textureSample(source_texture, source_sampler, in.uv - texel * 0.75) * 0.054054;
    color += textureSample(source_texture, source_sampler, in.uv + texel) * 0.016216;
    color += textureSample(source_texture, source_sampler, in.uv - texel) * 0.016216;

    return color;
}

@fragment
fn fs_composite(in: CompositeOut) -> @location(0) vec4<f32> {
    let alpha = coverage(rounded_rect_sdf(in.local_position, in.rect, in.radius));

    if alpha <= 0.0 {
        discard;
    }

    let scale_factor = params.direction_radius.w;
    let uv = in.local_position * scale_factor / max(params.texture_size, vec2<f32>(1.0));
    let color = textureSample(source_texture, source_sampler, uv);

    return vec4<f32>(color.rgb, color.a * alpha);
}
