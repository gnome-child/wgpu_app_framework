use crate::{composition, geometry};

use super::{Clip, Material, Pane, Rounding};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum MaterialRenderer {
    InFrame,
    NativePopup { opaque: bool },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) enum MaterialFidelity {
    Full,
    Frost,
    Fallback,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub(crate) struct RealizedMaterialParts {
    backdrop_frost: bool,
    surface_tint: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct MaterialRealizationReport {
    id: composition::NodeId,
    parts: RealizedMaterialParts,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub(crate) struct MaterialCapabilities {
    backdrop_frost: bool,
}

impl MaterialCapabilities {
    pub(crate) const fn none() -> Self {
        Self {
            backdrop_frost: false,
        }
    }

    pub(crate) const fn backdrop_frost() -> Self {
        Self {
            backdrop_frost: true,
        }
    }

    pub(crate) const fn forecasts_backdrop_frost(self) -> bool {
        self.backdrop_frost
    }
}

impl RealizedMaterialParts {
    pub(crate) const fn none() -> Self {
        Self {
            backdrop_frost: false,
            surface_tint: false,
        }
    }

    pub(crate) const fn frost(surface_tint: bool) -> Self {
        Self {
            backdrop_frost: true,
            surface_tint,
        }
    }

    pub(crate) const fn backdrop_frost(self) -> bool {
        self.backdrop_frost
    }

    pub(crate) const fn surface_tint(self) -> bool {
        self.surface_tint
    }
}

impl MaterialRealizationReport {
    pub(crate) const fn new(id: composition::NodeId, parts: RealizedMaterialParts) -> Self {
        Self { id, parts }
    }

    pub(crate) const fn id(self) -> composition::NodeId {
        self.id
    }

    pub(crate) const fn parts(self) -> RealizedMaterialParts {
        self.parts
    }
}

/// Retained scene request for one shaped material surface.
///
/// Identity belongs to the declaring composition node. Position in the
/// surrounding `Vec` is paint order only and never participates in identity.
#[derive(Debug, Clone, PartialEq)]
pub(crate) struct MaterialRegion {
    id: composition::NodeId,
    rect: geometry::Rect,
    rounding: Rounding,
    clips: Vec<Clip>,
    opacity: f32,
    material: Material,
}

#[cfg_attr(not(test), allow(dead_code))]
impl MaterialRegion {
    pub(super) fn from_pane(id: composition::NodeId, pane: &Pane, clip: Option<Clip>) -> Self {
        Self {
            id,
            rect: pane.rect(),
            rounding: pane.rounding(),
            clips: clip.into_iter().collect(),
            opacity: 1.0,
            material: pane.material().clone(),
        }
    }

    pub(crate) fn id(&self) -> composition::NodeId {
        self.id
    }

    pub(crate) fn rect(&self) -> geometry::Rect {
        self.rect
    }

    pub(crate) fn rounding(&self) -> Rounding {
        self.rounding
    }

    pub(crate) fn clips(&self) -> &[Clip] {
        &self.clips
    }

    pub(crate) fn opacity(&self) -> f32 {
        self.opacity
    }

    pub(crate) fn material(&self) -> &Material {
        &self.material
    }

    pub(super) fn translated(&self, dx: i32, dy: i32) -> Self {
        Self {
            id: self.id,
            rect: translate_rect(self.rect, dx, dy),
            rounding: self.rounding,
            clips: self
                .clips
                .iter()
                .map(|clip| clip.translated(dx, dy))
                .collect(),
            opacity: self.opacity,
            material: self.material.clone(),
        }
    }

    pub(super) fn with_parent_opacity(&self, opacity: f32) -> Self {
        let mut projected = self.clone();
        projected.opacity = (projected.opacity * opacity).clamp(0.0, 1.0);
        projected
    }
}

fn translate_rect(rect: geometry::Rect, dx: i32, dy: i32) -> geometry::Rect {
    geometry::Rect::new(
        rect.x().saturating_add(dx),
        rect.y().saturating_add(dy),
        rect.width(),
        rect.height(),
    )
}
