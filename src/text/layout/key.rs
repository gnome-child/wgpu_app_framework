use super::super::document::{Style, TextDirection, Weight};
use crate::geometry::area;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub(super) struct StyleKey {
    size: u32,
    weight: Weight,
    direction: TextDirection,
}

impl StyleKey {
    pub(super) fn new(style: Style) -> Self {
        Self {
            size: finite_bits(style.size().max(1.0)),
            weight: style.weight(),
            direction: style.direction(),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub(super) struct BoundsKey {
    width: u32,
    height: u32,
}

impl BoundsKey {
    pub(super) fn new(bounds: area::Logical) -> Self {
        Self {
            width: finite_bits(bounds.width().max(0.0)),
            height: finite_bits(bounds.height().max(0.0)),
        }
    }
}

pub(super) fn finite_bits(value: f32) -> u32 {
    if value.is_finite() {
        value.to_bits()
    } else if value.is_sign_negative() {
        0.0_f32.to_bits()
    } else {
        f32::INFINITY.to_bits()
    }
}
