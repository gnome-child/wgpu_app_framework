use crate::geometry::area;
use crate::paint;

use super::Target;

#[derive(Clone, Copy)]
pub(in crate::render) struct TextureSource<'a> {
    pub(in crate::render::filter) view: &'a wgpu::TextureView,
    pub(in crate::render::filter) area: area::Physical,
    pub(in crate::render::filter) logical_area: area::Logical,
    pub(in crate::render::filter) sampling: paint::LayerSampling,
}

impl<'a> TextureSource<'a> {
    pub(in crate::render) fn new(
        view: &'a wgpu::TextureView,
        area: area::Physical,
        logical_area: area::Logical,
        sampling: paint::LayerSampling,
    ) -> Self {
        debug_assert!(area.width() > 0 && area.height() > 0);
        debug_assert!(logical_area.width() > 0.0 && logical_area.height() > 0.0);
        Self {
            view,
            area,
            logical_area,
            sampling,
        }
    }

    pub(in crate::render::filter) fn for_target_view(
        view: &'a wgpu::TextureView,
        target: Target,
        sampling: paint::LayerSampling,
    ) -> Self {
        let area = target.physical_area.clamp_min(1);

        Self::new(view, area, area.to_logical(target.scale_factor), sampling)
    }
}
