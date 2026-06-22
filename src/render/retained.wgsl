struct Params {
    texture_size: vec4<f32>,
    source_origin_scale: vec4<f32>,
    rect: vec4<f32>,
};

@group(0) @binding(0) var source_texture: texture_2d<f32>;
@group(0) @binding(1) var<uniform> params: Params;

struct VertexIn {
    @location(0) position: vec2<f32>,
    @location(1) local_position: vec2<f32>,
    @location(2) rect: vec4<f32>,
};

struct VertexOut {
    @builtin(position) position: vec4<f32>,
    @location(0) local_position: vec2<f32>,
};

@vertex
fn vs_main(in: VertexIn) -> VertexOut {
    var out: VertexOut;
    out.position = vec4<f32>(in.position, 0.0, 1.0);
    out.local_position = in.local_position;
    return out;
}

@fragment
fn fs_main(in: VertexOut) -> @location(0) vec4<f32> {
    let source_position = params.source_origin_scale.xy
        + (in.local_position - params.rect.xy) * params.source_origin_scale.zw;
    let max_position = max(params.texture_size.xy - vec2<f32>(1.0), vec2<f32>(0.0));
    let texel = vec2<i32>(floor(clamp(source_position, vec2<f32>(0.0), max_position)));
    let color = textureLoad(source_texture, texel, 0);
    let rgb = select(color.rgb / max(color.a, 0.0001), vec3<f32>(0.0), color.a <= 0.0001);

    return vec4<f32>(rgb, color.a);
}
