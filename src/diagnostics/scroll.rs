#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Scroll {
    pub wheel_events: usize,
    pub scroll_offset_changes: usize,
    pub queued_scroll_updates: usize,
    pub scroll_redraw_requests: usize,
    pub frame_scroll_commits: usize,
    pub pending_scroll_applications: usize,
    pub pending_scroll_updates: usize,
    pub projection_count: usize,
    pub text_area_resolves: usize,
    pub text_area_projection_reuses: usize,
    pub text_area_projection_shifts: usize,
    pub text_area_projection_shift_misses: usize,
    pub text_area_projection_cold_jumps: usize,
    pub async_scroll_reconciles: usize,
    pub async_scroll_projection_sync_skips: usize,
}
