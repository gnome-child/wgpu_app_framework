use crate::{paint, render};

#[derive(Debug, Clone, Copy)]
pub struct Target {
    pub(super) physical_area: paint::area::Physical,
    pub(super) logical_area: paint::area::Logical,
    pub(super) scale_factor: f32,
}

impl Target {
    pub fn new(canvas: &render::Canvas) -> Self {
        Self {
            physical_area: canvas.physical_area(),
            logical_area: canvas.logical_area(),
            scale_factor: canvas.scale_factor(),
        }
    }

    pub fn from_viewport(viewport: render::Viewport) -> Self {
        Self {
            physical_area: viewport.physical_area(),
            logical_area: viewport.logical_area(),
            scale_factor: viewport.scale_factor(),
        }
    }

    pub fn from_logical_area(logical_area: paint::area::Logical, scale_factor: f32) -> Self {
        Self {
            physical_area: logical_area.to_physical(scale_factor).clamp_min(1),
            logical_area,
            scale_factor,
        }
    }

    pub fn physical_area(self) -> paint::area::Physical {
        self.physical_area
    }

    pub fn logical_area(self) -> paint::area::Logical {
        self.logical_area
    }
}
