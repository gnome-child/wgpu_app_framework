use crate::paint;

const MAX_LIQUID_REFRACTION: f32 = 48.0;

pub(super) fn liquid_depth_displacement(depth: f32) -> f32 {
    depth.clamp(0.0, 1.0) * MAX_LIQUID_REFRACTION
}

pub(super) fn liquid_effect(depth: f32, splay: f32, feather: f32, curve: f32) -> [f32; 4] {
    [
        liquid_depth_displacement(depth),
        splay.max(0.0),
        feather.max(0.0),
        curve.max(0.1),
    ]
}

pub(super) fn refraction_effect(refraction: paint::Refraction) -> [f32; 4] {
    [
        refraction.displacement.clamp(0.0, 4.0),
        refraction.splay.max(0.0),
        refraction.feather.max(0.0),
        refraction.curve.max(0.1),
    ]
}

pub(super) fn liquid_is_identity(depth: f32) -> bool {
    depth <= 0.0
}
