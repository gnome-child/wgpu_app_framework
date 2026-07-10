@group(0) @binding(0) var source_texture: texture_2d<f32>;
@group(0) @binding(1) var source_sampler: sampler;

struct FullscreenOut {
    @builtin(position) position: vec4<f32>,
    @location(0) uv: vec2<f32>,
};

@vertex
fn vs_main(@builtin(vertex_index) index: u32) -> FullscreenOut {
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

fn srgb_encode_channel(value: f32) -> f32 {
    let v = clamp(value, 0.0, 1.0);

    if v <= 0.0031308 {
        return 12.92 * v;
    }

    return 1.055 * pow(v, 1.0 / 2.4) - 0.055;
}

fn srgb_encode(color: vec3<f32>) -> vec3<f32> {
    return vec3<f32>(
        srgb_encode_channel(color.r),
        srgb_encode_channel(color.g),
        srgb_encode_channel(color.b),
    );
}

@fragment
fn fs_main(in: FullscreenOut) -> @location(0) vec4<f32> {
    let color = textureSample(source_texture, source_sampler, in.uv);
    let alpha = clamp(color.a, 0.0, 1.0);

    if alpha <= 0.00001 {
        return vec4<f32>(0.0);
    }

    let straight = clamp(color.rgb / alpha, vec3<f32>(0.0), vec3<f32>(1.0));
    let packed_rgb = srgb_encode(straight) * alpha;
    return vec4<f32>(packed_rgb, alpha);
}
