use bytemuck::{Pod, Zeroable};

use crate::paint::{self, Rect};
use crate::render::silhouette::{rect_data, rounding_data};

use super::geometry::PreparedFilter;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum AlphaMode {
    Source,
    Shape,
}

impl AlphaMode {
    pub(super) fn shader_value(self) -> f32 {
        match self {
            Self::Source => 0.0,
            Self::Shape => 1.0,
        }
    }
}

pub(super) struct ParamInput {
    pub(super) target_scale_factor: f32,
    pub(super) texture_area: paint::area::Physical,
    pub(super) texture_logical_area: paint::area::Logical,
    pub(super) prepared: PreparedFilter,
    pub(super) source_rect: Rect,
    pub(super) direction: [f32; 2],
    pub(super) effect: [f32; 4],
    pub(super) alpha_mode: AlphaMode,
    pub(super) sampling: paint::LayerSampling,
}

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
pub(super) struct Params {
    pub(super) texture_size: [f32; 2],
    pub(super) source_scale: [f32; 2],
    pub(super) direction_radius: [f32; 4],
    pub(super) effect: [f32; 4],
    pub(super) rect: [f32; 4],
    pub(super) source_rect: [f32; 4],
    pub(super) target_rect: [f32; 4],
    pub(super) rounding: [f32; 4],
    pub(super) alpha_mode: [f32; 4],
}

pub(super) fn with_texture_area(input: ParamInput) -> Params {
    let physical_area = input.texture_area.clamp_min(1);

    let source_rect_data = physical_source_rect_data(
        input.source_rect,
        input.texture_logical_area,
        physical_area,
        input.target_scale_factor,
        input.sampling,
    );
    let source_scale = source_step_data(
        source_rect_data,
        input.prepared.shape_rect.area,
        input.target_scale_factor,
        input.sampling,
    );

    Params {
        texture_size: [physical_area.width() as f32, physical_area.height() as f32],
        source_scale,
        direction_radius: [
            input.direction[0],
            input.direction[1],
            input.prepared.blur_radius_px,
            0.0,
        ],
        effect: input.effect,
        rect: rect_data(input.prepared.shape_rect),
        source_rect: source_rect_data,
        target_rect: physical_rect_data(input.prepared.shape_rect, input.target_scale_factor),
        rounding: rounding_data(input.prepared.rounding),
        alpha_mode: [input.alpha_mode.shader_value(), 0.0, 0.0, 0.0],
    }
}

pub(super) fn physical_source_rect_data(
    source_rect: Rect,
    texture_logical_area: paint::area::Logical,
    texture_physical_area: paint::area::Physical,
    target_scale_factor: f32,
    sampling: paint::LayerSampling,
) -> [f32; 4] {
    let [x_scale, y_scale] = match sampling {
        paint::LayerSampling::PixelAligned => [target_scale_factor, target_scale_factor],
        paint::LayerSampling::Filtered => {
            source_scale_data(texture_logical_area, texture_physical_area)
        }
    };

    [
        (source_rect.origin.x() * x_scale).round(),
        (source_rect.origin.y() * y_scale).round(),
        (source_rect.area.width() * x_scale).round().max(1.0),
        (source_rect.area.height() * y_scale).round().max(1.0),
    ]
}

pub(super) fn physical_rect_data(rect: Rect, scale_factor: f32) -> [f32; 4] {
    let scale_factor = scale_factor.max(0.0001);

    [
        (rect.origin.x() * scale_factor).round(),
        (rect.origin.y() * scale_factor).round(),
        (rect.area.width() * scale_factor).round().max(1.0),
        (rect.area.height() * scale_factor).round().max(1.0),
    ]
}

pub(super) fn source_scale_data(
    texture_logical_area: paint::area::Logical,
    texture_physical_area: paint::area::Physical,
) -> [f32; 2] {
    [
        texture_physical_area.width() as f32 / texture_logical_area.width().max(1.0),
        texture_physical_area.height() as f32 / texture_logical_area.height().max(1.0),
    ]
}

pub(super) fn source_step_data(
    source_rect_data: [f32; 4],
    destination_area: paint::area::Logical,
    target_scale_factor: f32,
    sampling: paint::LayerSampling,
) -> [f32; 2] {
    match sampling {
        paint::LayerSampling::PixelAligned => [target_scale_factor, target_scale_factor],
        paint::LayerSampling::Filtered => [
            source_rect_data[2] / destination_area.width().max(1.0),
            source_rect_data[3] / destination_area.height().max(1.0),
        ],
    }
}

#[cfg(test)]
pub(super) fn noise_material_position_data(local_position: [f32; 2], params: Params) -> [f32; 2] {
    let logical_size = [params.rect[2].max(0.0001), params.rect[3].max(0.0001)];
    let physical_size = [
        params.target_rect[2].max(1.0),
        params.target_rect[3].max(1.0),
    ];

    [
        (local_position[0] - params.rect[0]) * (physical_size[0] / logical_size[0]),
        (local_position[1] - params.rect[1]) * (physical_size[1] / logical_size[1]),
    ]
}
