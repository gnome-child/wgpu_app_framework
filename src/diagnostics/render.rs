use std::{
    collections::VecDeque,
    time::{Duration, Instant},
};

use super::{
    DrawStats,
    samples::{SAMPLE_LIMIT, Samples},
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Render {
    pub frames_attempted: usize,
    pub frames_presented: usize,
    pub scene_items: usize,
    pub render_batches: usize,
    pub glyph_batches: usize,
    pub text_surfaces: usize,
    pub inline_text_cache_hits: usize,
    pub inline_text_cache_misses: usize,
    pub inline_text_shape_calls: usize,
    pub inline_text_cache_hits_total: usize,
    pub inline_text_cache_misses_total: usize,
    pub inline_text_shape_calls_total: usize,
    pub inline_icon_cache_hits: usize,
    pub inline_icon_cache_misses: usize,
    pub inline_icon_shape_calls: usize,
    pub quad_vertices: usize,
    pub geometry_upload_bytes: usize,
    pub geometry_buffer_creations: usize,
    pub draw_passes: usize,
    pub clip_batches: usize,
    pub group_composites: usize,
    pub filter_layer_pool_entries: usize,
    pub filter_scratch_pool_entries: usize,
    intervals: Samples,
    acquire_wait: Samples,
    batch_prepare: Samples,
    encode_submit_present: Samples,
    draw: Samples,
    key_to_present: Samples,
    pending_inputs: VecDeque<InputSample>,
    last_presented_at: Option<Instant>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct InputSample {
    epoch: crate::window::PresentationEpoch,
    started_at: Instant,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Report {
    acquire_wait: Duration,
    draw: Duration,
    batch_prepare: Duration,
    encode_submit_present: Duration,
    draw_stats: DrawStats,
    presented: bool,
    presented_at: Instant,
    group_composites: usize,
    filter_layer_pool_entries: usize,
    filter_scratch_pool_entries: usize,
}

impl Render {
    pub(crate) fn record_input(
        &mut self,
        epoch: crate::window::PresentationEpoch,
        started_at: Instant,
    ) {
        if self.pending_inputs.len() == SAMPLE_LIMIT {
            self.pending_inputs.pop_front();
        }
        self.pending_inputs
            .push_back(InputSample { epoch, started_at });
    }

    pub(crate) fn record_present(
        &mut self,
        epoch: crate::window::PresentationEpoch,
        report: Report,
    ) {
        self.frames_attempted += 1;
        self.scene_items = report.draw_stats.scene_items;
        self.render_batches = report.draw_stats.render_batches;
        self.glyph_batches = report.draw_stats.glyph_batches;
        self.text_surfaces = report.draw_stats.text_surfaces;
        self.inline_text_cache_hits = report.draw_stats.inline_text_cache_hits;
        self.inline_text_cache_misses = report.draw_stats.inline_text_cache_misses;
        self.inline_text_shape_calls = report.draw_stats.inline_text_shape_calls;
        self.inline_text_cache_hits_total += report.draw_stats.inline_text_cache_hits;
        self.inline_text_cache_misses_total += report.draw_stats.inline_text_cache_misses;
        self.inline_text_shape_calls_total += report.draw_stats.inline_text_shape_calls;
        self.inline_icon_cache_hits = report.draw_stats.inline_icon_cache_hits;
        self.inline_icon_cache_misses = report.draw_stats.inline_icon_cache_misses;
        self.inline_icon_shape_calls = report.draw_stats.inline_icon_shape_calls;
        self.quad_vertices = report.draw_stats.quad_vertices;
        self.geometry_upload_bytes = report.draw_stats.geometry_upload_bytes;
        self.geometry_buffer_creations = report.draw_stats.geometry_buffer_creations;
        self.draw_passes = report.draw_stats.draw_passes;
        self.clip_batches = report.draw_stats.clip_batches;
        self.group_composites = report
            .group_composites
            .max(report.draw_stats.group_composites);
        self.filter_layer_pool_entries = report.filter_layer_pool_entries;
        self.filter_scratch_pool_entries = report.filter_scratch_pool_entries;
        if let Some(previous) = self.last_presented_at {
            self.intervals.record(duration_micros(
                report.presented_at.saturating_duration_since(previous),
            ));
        }
        self.last_presented_at = Some(report.presented_at);
        self.acquire_wait
            .record(duration_micros(report.acquire_wait));
        self.batch_prepare
            .record(duration_micros(report.batch_prepare));
        self.encode_submit_present
            .record(duration_micros(report.encode_submit_present));
        self.draw.record(duration_micros(report.draw));

        if !report.presented {
            return;
        }

        self.frames_presented += 1;

        let mut remaining = VecDeque::with_capacity(self.pending_inputs.len());
        while let Some(sample) = self.pending_inputs.pop_front() {
            if sample.epoch <= epoch {
                self.key_to_present.record(duration_micros(
                    report
                        .presented_at
                        .saturating_duration_since(sample.started_at),
                ));
            } else {
                remaining.push_back(sample);
            }
        }
        self.pending_inputs = remaining;
    }

    pub fn interval_p95_us(&self) -> u128 {
        self.intervals.p95()
    }

    pub fn acquire_wait_p95_us(&self) -> u128 {
        self.acquire_wait.p95()
    }

    pub fn draw_p95_us(&self) -> u128 {
        self.draw.p95()
    }

    pub fn batch_prepare_p95_us(&self) -> u128 {
        self.batch_prepare.p95()
    }

    pub fn encode_submit_present_p95_us(&self) -> u128 {
        self.encode_submit_present.p95()
    }

    pub fn key_to_present_p95_us(&self) -> u128 {
        self.key_to_present.p95()
    }

    pub fn pending_key_to_present_samples(&self) -> usize {
        self.pending_inputs.len()
    }
}

impl Default for Render {
    fn default() -> Self {
        Self {
            frames_attempted: 0,
            frames_presented: 0,
            scene_items: 0,
            render_batches: 0,
            glyph_batches: 0,
            text_surfaces: 0,
            inline_text_cache_hits: 0,
            inline_text_cache_misses: 0,
            inline_text_shape_calls: 0,
            inline_text_cache_hits_total: 0,
            inline_text_cache_misses_total: 0,
            inline_text_shape_calls_total: 0,
            inline_icon_cache_hits: 0,
            inline_icon_cache_misses: 0,
            inline_icon_shape_calls: 0,
            quad_vertices: 0,
            geometry_upload_bytes: 0,
            geometry_buffer_creations: 0,
            draw_passes: 0,
            clip_batches: 0,
            group_composites: 0,
            filter_layer_pool_entries: 0,
            filter_scratch_pool_entries: 0,
            intervals: Samples::default(),
            acquire_wait: Samples::default(),
            batch_prepare: Samples::default(),
            encode_submit_present: Samples::default(),
            draw: Samples::default(),
            key_to_present: Samples::default(),
            pending_inputs: VecDeque::new(),
            last_presented_at: None,
        }
    }
}

impl Report {
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

fn duration_micros(duration: Duration) -> u128 {
    duration.as_micros()
}

#[cfg(test)]
mod tests {
    use std::time::{Duration, Instant};

    use crate::window::PresentationEpoch;

    use super::{Render, Report, SAMPLE_LIMIT, Samples};

    #[test]
    fn samples_are_capped_and_report_p95() {
        let mut samples = Samples::default();
        for value in 0..(SAMPLE_LIMIT + 10) {
            samples.record(value as u128);
        }

        assert_eq!(samples.len(), SAMPLE_LIMIT);
        assert_eq!(samples.p95(), 131);
    }

    #[test]
    fn input_samples_wait_for_presented_epoch() {
        let initial = PresentationEpoch::initial();
        let changed = initial.next();
        let mut render = Render::default();
        let now = Instant::now();
        render.record_input(changed, now);

        render.record_present(
            initial,
            Report::new(
                Duration::from_micros(5),
                Duration::from_micros(10),
                now + Duration::from_millis(1),
            ),
        );
        assert_eq!(render.pending_key_to_present_samples(), 1);
        assert_eq!(render.key_to_present_p95_us(), 0);

        render.record_present(
            changed,
            Report::new(
                Duration::from_micros(5),
                Duration::from_micros(10),
                now + Duration::from_millis(2),
            ),
        );
        assert_eq!(render.pending_key_to_present_samples(), 0);
        assert_eq!(render.key_to_present_p95_us(), 2_000);
    }

    #[test]
    fn render_report_records_filter_pool_entries() {
        let mut render = Render::default();

        render.record_present(
            PresentationEpoch::initial(),
            Report::new(
                Duration::from_micros(5),
                Duration::from_micros(10),
                Instant::now(),
            )
            .with_group_composites(2)
            .with_filter_pool_entries(3, 4),
        );

        assert_eq!(render.group_composites, 2);
        assert_eq!(render.filter_layer_pool_entries, 3);
        assert_eq!(render.filter_scratch_pool_entries, 4);
    }

    #[test]
    fn skipped_attempt_does_not_acknowledge_epoch_or_latency() {
        let epoch = PresentationEpoch::initial().next();
        let now = Instant::now();
        let mut render = Render::default();
        render.record_input(epoch, now);

        render.record_present(
            epoch,
            Report::new(
                Duration::from_micros(5),
                Duration::from_micros(10),
                now + Duration::from_millis(1),
            )
            .with_presented(false),
        );

        assert_eq!(render.frames_attempted, 1);
        assert_eq!(render.frames_presented, 0);
        assert_eq!(render.pending_key_to_present_samples(), 1);
        assert_eq!(render.key_to_present_p95_us(), 0);
    }
}
