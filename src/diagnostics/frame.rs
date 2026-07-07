#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Frame {
    pub full_redraws: usize,
    pub view_rebuilds: usize,
    pub layout_recomposes: usize,
    pub layout_reuses: usize,
    pub text_area_render_surfaces: usize,
}
