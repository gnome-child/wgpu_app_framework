struct Params {
    texture_size: vec2<f32>,
    source_scale: vec2<f32>,
    direction_radius: vec4<f32>,
    effect: vec4<f32>,
    rect: vec4<f32>,
    source_rect: vec4<f32>,
    target_rect: vec4<f32>,
    rounding: vec4<f32>,
    alpha_mode: vec4<f32>,
};

@group(0) @binding(0) var source_texture: texture_2d<f32>;
@group(0) @binding(1) var source_sampler: sampler;
@group(0) @binding(2) var<uniform> params: Params;
@group(0) @binding(3) var noise_texture: texture_2d<f32>;
@group(0) @binding(4) var noise_sampler: sampler;

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

fn smootherstep(value: f32) -> f32 {
    let t = clamp(value, 0.0, 1.0);

    return t * t * t * (t * (t * 6.0 - 15.0) + 10.0);
}

fn mirror_axis(position: f32, min_position: f32, span: f32) -> f32 {
    let safe_span = max(span, 1.0);
    let period = safe_span * 2.0;
    let local = position - min_position;
    let wrapped = local - floor(local / period) * period;
    let mirrored = select(wrapped, period - wrapped, wrapped > safe_span);

    return min_position + clamp(mirrored, 0.0, safe_span);
}

fn mirror_source_position(position: vec2<f32>) -> vec2<f32> {
    return vec2<f32>(
        mirror_axis(position.x, params.source_rect.x, params.source_rect.z),
        mirror_axis(position.y, params.source_rect.y, params.source_rect.w),
    );
}

fn source_position_for_target(position: vec2<f32>) -> vec2<f32> {
    return params.source_rect.xy + (position - params.target_rect.xy);
}

fn sample_mirrored_source(target_position: vec2<f32>) -> vec4<f32> {
    let source_position = source_position_for_target(target_position);
    let uv = mirror_source_position(source_position) / max(params.texture_size, vec2<f32>(1.0));

    return textureSample(source_texture, source_sampler, uv);
}

fn unpremultiply(color: vec4<f32>) -> vec3<f32> {
    return select(color.rgb / max(color.a, 0.0001), vec3<f32>(0.0), color.a <= 0.0001);
}

fn filter_source_rgb(color: vec4<f32>) -> vec3<f32> {
    if params.alpha_mode.x >= 0.5 {
        return color.rgb;
    }

    return unpremultiply(color);
}

fn filter_alpha(source_alpha: f32, shape_alpha: f32, opacity: f32) -> f32 {
    let coverage = shape_alpha * opacity;

    if params.alpha_mode.x >= 0.5 {
        return coverage;
    }

    return source_alpha * coverage;
}

fn source_position_for_local(local_position: vec2<f32>) -> vec2<f32> {
    return params.source_rect.xy
        + (local_position - params.rect.xy) * params.source_scale;
}

fn sample_source_for_local(local_position: vec2<f32>) -> vec4<f32> {
    let uv = source_position_for_local(local_position) / max(params.texture_size, vec2<f32>(1.0));

    return textureSample(source_texture, source_sampler, uv);
}

fn material_position_for_local(local_position: vec2<f32>) -> vec2<f32> {
    let logical_size = max(params.rect.zw, vec2<f32>(0.0001));
    let physical_size = max(params.target_rect.zw, vec2<f32>(1.0));

    return (local_position - params.rect.xy) * (physical_size / logical_size);
}

@fragment
fn fs_blur(in: FullscreenOut) -> @location(0) vec4<f32> {
    let direction = params.direction_radius.xy;
    let material_radius_px = clamp(params.direction_radius.z, 0.0, 256.0);
    let sigma = max(params.effect.x, 0.001);

    var color = sample_mirrored_source(in.position.xy);
    var total_weight = 1.0;

    for (var i: i32 = 1; i <= 256; i = i + 2) {
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

        color += sample_mirrored_source(in.position.xy + direction * pair_offset) * pair_weight;
        color += sample_mirrored_source(in.position.xy - direction * pair_offset) * pair_weight;
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
    let alpha = rounded_rect_coverage(rounded_rect_sdf(in.local_position, in.rect, in.rounding));

    if alpha <= 0.0 {
        discard;
    }

    let source_position = params.source_rect.xy
        + (in.local_position - params.rect.xy) * params.source_scale;
    let uv = source_position / max(params.texture_size, vec2<f32>(1.0));
    let color = textureSample(source_texture, source_sampler, uv);
    let rgb = filter_source_rgb(color);
    let opacity = clamp(params.effect.x, 0.0, 1.0);

    return vec4<f32>(rgb, filter_alpha(color.a, alpha, opacity));
}

@fragment
fn fs_refraction(in: CompositeOut) -> @location(0) vec4<f32> {
    let sdf = rounded_rect_sdf(in.local_position, in.rect, in.rounding);
    let alpha = rounded_rect_coverage(sdf);

    if alpha <= 0.0 {
        discard;
    }

    let depth = max(params.effect.x, 0.0);
    let splay = max(params.effect.y, 0.0);
    let feather = max(params.effect.z, 0.0001);
    let curve = max(params.effect.w, 0.1);
    let normal = rounded_rect_normal(in.local_position, in.rect, in.rounding);
    let inside_distance = max(-sdf, 0.0);
    let edge_span = feather + splay;
    let edge_amount = pow(smootherstep(1.0 - inside_distance / edge_span), curve);
    let splay_boost = 1.0 + splay / max(feather, 1.0);
    let displacement = -normal * depth * edge_amount * splay_boost;
    let source_position = params.source_rect.xy
        + (in.local_position - params.rect.xy + displacement) * params.source_scale;
    let uv = source_position / max(params.texture_size, vec2<f32>(1.0));
    let color = textureSample(source_texture, source_sampler, uv);
    let rgb = filter_source_rgb(color);

    return vec4<f32>(rgb, filter_alpha(color.a, alpha, 1.0));
}

@fragment
fn fs_luminosity(in: CompositeOut) -> @location(0) vec4<f32> {
    let alpha = rounded_rect_coverage(rounded_rect_sdf(in.local_position, in.rect, in.rounding));

    if alpha <= 0.0 {
        discard;
    }

    let color = sample_source_for_local(in.local_position);
    let rgb = filter_source_rgb(color);
    let target_color = clamp(params.effect.rgb, vec3<f32>(0.0), vec3<f32>(1.0));
    let opacity = clamp(params.effect.w, 0.0, 1.0);
    let weights = vec3<f32>(0.2126, 0.7152, 0.0722);
    let source_luma = dot(rgb, weights);
    let target_luma = dot(target_color, weights);
    let adjusted = clamp(rgb + (target_luma - source_luma) * opacity, vec3<f32>(0.0), vec3<f32>(1.0));

    return vec4<f32>(adjusted, filter_alpha(color.a, alpha, 1.0));
}

@fragment
fn fs_noise(in: CompositeOut) -> @location(0) vec4<f32> {
    let alpha = rounded_rect_coverage(rounded_rect_sdf(in.local_position, in.rect, in.rounding));

    if alpha <= 0.0 {
        discard;
    }

    let color = sample_source_for_local(in.local_position);
    let rgb = filter_source_rgb(color);
    let opacity = clamp(params.effect.x, 0.0, 1.0);
    let noise_uv = material_position_for_local(in.local_position) / vec2<f32>(128.0);
    let noise = textureSample(noise_texture, noise_sampler, noise_uv);
    let adjusted = mix(rgb, noise.rgb, opacity * noise.a);

    return vec4<f32>(adjusted, filter_alpha(color.a, alpha, 1.0));
}

@fragment
fn fs_composite_pixel(in: CompositeOut) -> @location(0) vec4<f32> {
    let alpha = rounded_rect_coverage(rounded_rect_sdf(in.local_position, in.rect, in.rounding));

    if alpha <= 0.0 {
        discard;
    }

    let source_position = params.source_rect.xy
        + (in.local_position - params.rect.xy) * params.source_scale;
    let max_position = max(params.texture_size - vec2<f32>(1.0), vec2<f32>(0.0));
    let texel = vec2<i32>(floor(clamp(source_position, vec2<f32>(0.0), max_position)));
    let color = textureLoad(source_texture, texel, 0);
    let rgb = filter_source_rgb(color);
    let opacity = clamp(params.effect.x, 0.0, 1.0);

    return vec4<f32>(rgb, filter_alpha(color.a, alpha, opacity));
}
