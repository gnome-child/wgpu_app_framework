#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Frame {
    pub full_redraws: usize,
    pub view_rebuilds: usize,
    pub layout_recomposes: usize,
    pub layout_reuses: usize,
    pub scroll_only_redraws: usize,
    pub scroll_only_fallbacks_to_full: usize,
    pub render_skips: usize,
    pub paint: Timing,
    pub render: Timing,
    pub render_text_prepare: Timing,
    pub total: Timing,
    pub last_scroll_frame: LastScrollFrame,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Timing {
    pub latest_us: u64,
    pub average_us: u64,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct LastScrollFrame {
    pub render_text_prepare_us: u64,
    pub render_total_us: u64,
    pub total_us: u64,
    pub text_surfaces: usize,
    pub glyph_batches: usize,
}
