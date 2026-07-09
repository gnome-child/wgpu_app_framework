use crate::{paint, render};

use super::Texture;

const TEXTURE_SIZE: u32 = 128;
const SEED: u32 = 0x8f73_d4a9;

pub(super) fn create_texture(render_context: &render::Context) -> Texture {
    let extent = wgpu::Extent3d {
        width: TEXTURE_SIZE,
        height: TEXTURE_SIZE,
        depth_or_array_layers: 1,
    };
    let texture = render_context
        .device()
        .create_texture(&wgpu::TextureDescriptor {
            label: Some("Filter Noise Texture"),
            size: extent,
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8Unorm,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
            view_formats: &[],
        });
    let bytes = bytes();
    render_context.queue().write_texture(
        wgpu::TexelCopyTextureInfo {
            texture: &texture,
            mip_level: 0,
            origin: wgpu::Origin3d::ZERO,
            aspect: wgpu::TextureAspect::All,
        },
        &bytes,
        wgpu::TexelCopyBufferLayout {
            offset: 0,
            bytes_per_row: Some(TEXTURE_SIZE * 4),
            rows_per_image: Some(TEXTURE_SIZE),
        },
        extent,
    );
    let view = texture.create_view(&wgpu::TextureViewDescriptor::default());

    Texture {
        _inner: texture,
        view,
        area: paint::area::physical(TEXTURE_SIZE, TEXTURE_SIZE),
    }
}

#[cfg(test)]
pub(super) fn texture_size() -> u32 {
    TEXTURE_SIZE
}

#[cfg(test)]
pub(super) fn bytes() -> Vec<u8> {
    let size = TEXTURE_SIZE as usize;
    let mut bytes = Vec::with_capacity(size * size * 4);

    for y in 0..TEXTURE_SIZE {
        for x in 0..TEXTURE_SIZE {
            let value = texel(x, y);
            bytes.extend_from_slice(&[value, value, value, u8::MAX]);
        }
    }

    bytes
}

#[cfg(not(test))]
fn bytes() -> Vec<u8> {
    let size = TEXTURE_SIZE as usize;
    let mut bytes = Vec::with_capacity(size * size * 4);

    for y in 0..TEXTURE_SIZE {
        for x in 0..TEXTURE_SIZE {
            let value = texel(x, y);
            bytes.extend_from_slice(&[value, value, value, u8::MAX]);
        }
    }

    bytes
}

#[cfg(test)]
pub(super) fn texel(x: u32, y: u32) -> u8 {
    texel_value(x, y)
}

#[cfg(not(test))]
fn texel(x: u32, y: u32) -> u8 {
    texel_value(x, y)
}

fn texel_value(x: u32, y: u32) -> u8 {
    let hash = hash_texel(x, y);
    let centered = hash as i16 - 128;
    let value = 128 + centered * 35 / 100;

    value.clamp(0, u8::MAX as i16) as u8
}

fn hash_texel(x: u32, y: u32) -> u8 {
    let mut value = x.wrapping_mul(0x9e37_79b9).rotate_left(7)
        ^ y.wrapping_mul(0x85eb_ca6b).rotate_left(13)
        ^ SEED;

    value ^= value >> 16;
    value = value.wrapping_mul(0x7feb_352d);
    value ^= value >> 15;
    value = value.wrapping_mul(0x846c_a68b);
    value ^= value >> 16;

    (value >> 24) as u8
}
