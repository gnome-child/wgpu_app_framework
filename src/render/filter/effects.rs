use crate::paint;

pub(super) fn refraction_effect(refraction: paint::Refraction) -> [f32; 4] {
    [
        refraction.displacement,
        refraction.splay,
        refraction.feather,
        refraction.curve,
    ]
}
