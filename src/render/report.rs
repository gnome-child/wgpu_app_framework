use std::time::{Duration, Instant};

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub(crate) struct DrawStats {
    pub(crate) scene_items: usize,
    pub(crate) render_batches: usize,
    pub(crate) glyph_batches: usize,
    pub(crate) text_surfaces: usize,
    pub(crate) inline_text_cache_hits: usize,
    pub(crate) inline_text_cache_misses: usize,
    pub(crate) inline_text_shape_calls: usize,
    pub(crate) inline_icon_cache_hits: usize,
    pub(crate) inline_icon_cache_misses: usize,
    pub(crate) inline_icon_shape_calls: usize,
    pub(crate) quad_vertices: usize,
    pub(crate) geometry_upload_bytes: usize,
    pub(crate) geometry_buffer_creations: usize,
    pub(crate) draw_passes: usize,
    pub(crate) clip_batches: usize,
    pub(crate) group_composites: usize,
    pub(crate) filter_layer_pool_entries: usize,
    pub(crate) filter_scratch_pool_entries: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RenderReport {
    pub(crate) acquire_wait: Duration,
    pub(crate) draw: Duration,
    pub(crate) batch_prepare: Duration,
    pub(crate) encode_submit_present: Duration,
    pub(crate) draw_stats: DrawStats,
    pub(crate) presented: bool,
    pub(crate) presented_at: Instant,
    pub(crate) group_composites: usize,
    pub(crate) filter_layer_pool_entries: usize,
    pub(crate) filter_scratch_pool_entries: usize,
}

impl RenderReport {
    pub fn new(acquire_wait: Duration, draw: Duration, presented_at: Instant) -> Self {
        Self {
            acquire_wait,
            draw,
            batch_prepare: Duration::ZERO,
            encode_submit_present: Duration::ZERO,
            draw_stats: DrawStats::default(),
            presented: true,
            presented_at,
            group_composites: 0,
            filter_layer_pool_entries: 0,
            filter_scratch_pool_entries: 0,
        }
    }

    pub fn with_group_composites(mut self, group_composites: usize) -> Self {
        self.group_composites = group_composites;
        self
    }

    pub fn with_filter_pool_entries(
        mut self,
        layer_entries: usize,
        scratch_entries: usize,
    ) -> Self {
        self.filter_layer_pool_entries = layer_entries;
        self.filter_scratch_pool_entries = scratch_entries;
        self
    }

    pub(crate) fn with_pipeline_timings(
        mut self,
        batch_prepare: Duration,
        encode_submit_present: Duration,
    ) -> Self {
        self.batch_prepare = batch_prepare;
        self.encode_submit_present = encode_submit_present;
        self
    }

    pub(crate) fn with_draw_stats(mut self, stats: DrawStats) -> Self {
        self.draw_stats = stats;
        self
    }

    pub(crate) fn with_presented(mut self, presented: bool) -> Self {
        self.presented = presented;
        self
    }

    pub fn acquire_wait(self) -> Duration {
        self.acquire_wait
    }

    pub fn draw(self) -> Duration {
        self.draw
    }

    pub fn presented_at(self) -> Instant {
        self.presented_at
    }

    pub(crate) fn presented(self) -> bool {
        self.presented
    }
}
