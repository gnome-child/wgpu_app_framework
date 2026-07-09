use crate::paint;

pub(super) fn refraction_effect(refraction: paint::Refraction) -> [f32; 4] {
    [
        refraction.displacement.clamp(0.0, 4.0),
        refraction.splay.max(0.0),
        refraction.feather.max(0.0),
        refraction.curve.max(0.1),
    ]
}
