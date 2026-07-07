#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Scroll {
    pub wheel_events: usize,
    pub scroll_offset_changes: usize,
    pub scroll_redraw_requests: usize,
    pub frame_scroll_commits: usize,
    pub text_area_viewports: usize,
}
