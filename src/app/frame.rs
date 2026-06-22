use std::time::{Duration, Instant};

const SAMPLE_COUNT: usize = 60;

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct Diagnostics {
    pub full_redraws: usize,
    pub scroll_only_redraws: usize,
    pub invalidations: usize,
    pub coalesced_invalidations: usize,
    pub native_redraw_requests: usize,
    pub clean_redraw_skips: usize,
    pub redraw_events_without_dirty: usize,
    pub scroll_only_fallbacks_to_full: usize,
    pub render_skips: usize,
    pub compose: TimingDiagnostics,
    pub scroll_commit: TimingDiagnostics,
    pub scroll_projection_sync: TimingDiagnostics,
    pub paint: TimingDiagnostics,
    pub render: TimingDiagnostics,
    pub render_acquire: TimingDiagnostics,
    pub render_batching: TimingDiagnostics,
    pub render_quad_prepare: TimingDiagnostics,
    pub render_text_prepare: TimingDiagnostics,
    pub render_scene_text_prepare: TimingDiagnostics,
    pub render_layer_update_text_prepare: TimingDiagnostics,
    pub render_backdrop_prepare: TimingDiagnostics,
    pub render_encode_submit: TimingDiagnostics,
    pub total: TimingDiagnostics,
    pub scroll_input_to_present: TimingDiagnostics,
    pub dirty_to_present: TimingDiagnostics,
    pub scene_items: CountDiagnostics,
    pub render_batches: CountDiagnostics,
    pub glyph_batches: CountDiagnostics,
    pub text_surfaces: CountDiagnostics,
    pub quad_vertices: CountDiagnostics,
    pub clip_batches: CountDiagnostics,
    pub backdrops: CountDiagnostics,
    pub layer_items: CountDiagnostics,
    pub layer_updates: CountDiagnostics,
    pub last_scroll_frame: LastScrollFrameDiagnostics,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct LastScrollFrameDiagnostics {
    pub render_text_prepare_us: u64,
    pub render_scene_text_prepare_us: u64,
    pub render_layer_update_text_prepare_us: u64,
    pub render_total_us: u64,
    pub total_us: u64,
    pub text_surfaces: u64,
    pub glyph_batches: u64,
    pub layer_items: u64,
    pub layer_updates: u64,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct TimingDiagnostics {
    pub latest_us: u64,
    pub max_us: u64,
    pub average_us: u64,
    pub samples: usize,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct CountDiagnostics {
    pub latest: u64,
    pub max: u64,
    pub average: u64,
    pub samples: usize,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub(crate) enum RedrawKind {
    ScrollOnly,
    #[default]
    Full,
}

impl RedrawKind {
    pub(crate) fn merge(self, other: Self) -> Self {
        match (self, other) {
            (Self::Full, _) | (_, Self::Full) => Self::Full,
            (Self::ScrollOnly, Self::ScrollOnly) => Self::ScrollOnly,
        }
    }

    pub(crate) fn is_scroll_only(self) -> bool {
        self == Self::ScrollOnly
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct RedrawWork {
    kind: RedrawKind,
    dirty_to_frame: Option<Duration>,
}

impl RedrawWork {
    pub(crate) fn kind(self) -> RedrawKind {
        self.kind
    }

    pub(crate) fn dirty_to_frame(self) -> Option<Duration> {
        self.dirty_to_frame
    }
}

#[derive(Debug, Clone, Copy, Default)]
pub(crate) struct StageTimings {
    pub compose: Duration,
    pub scroll_commit: Duration,
    pub scroll_projection_sync: Duration,
    pub paint: Duration,
}

#[derive(Debug, Clone, Copy, Default)]
pub(crate) struct FrameTimings {
    pub stages: StageTimings,
    pub render: Duration,
    pub render_stages: RenderTimings,
    pub render_stats: RenderStats,
    pub total: Duration,
    pub scroll_input_to_present: Option<Duration>,
    pub dirty_to_present: Option<Duration>,
}

#[derive(Debug, Clone, Copy, Default)]
pub(crate) struct RenderTimings {
    pub acquire: Duration,
    pub batching: Duration,
    pub quad_prepare: Duration,
    pub text_prepare: Duration,
    pub scene_text_prepare: Duration,
    pub layer_update_text_prepare: Duration,
    pub backdrop_prepare: Duration,
    pub encode_submit: Duration,
}

#[derive(Debug, Clone, Copy, Default)]
pub(crate) struct RenderStats {
    pub scene_items: usize,
    pub render_batches: usize,
    pub glyph_batches: usize,
    pub text_surfaces: usize,
    pub quad_vertices: usize,
    pub clip_batches: usize,
    pub backdrops: usize,
    pub layer_items: usize,
    pub layer_updates: usize,
}

#[derive(Debug, Clone, Default)]
pub(crate) struct State {
    dirty: Option<RedrawKind>,
    dirty_since: Option<Instant>,
    native_redraw_pending: bool,
    presented_once: bool,
    recorder: Recorder,
}

impl State {
    pub(crate) fn invalidate(&mut self, kind: RedrawKind, now: Instant) {
        self.recorder.record_invalidation(self.dirty.is_some());
        if self.dirty.is_none() {
            self.dirty_since = Some(now);
        }

        self.dirty = Some(match self.dirty {
            Some(current) => current.merge(kind),
            None => kind,
        });
    }

    pub(crate) fn needs_native_redraw(&self) -> bool {
        !self.native_redraw_pending && (self.dirty.is_some() || !self.presented_once)
    }

    pub(crate) fn pending_redraw_kind(&self) -> Option<RedrawKind> {
        self.dirty
    }

    pub(crate) fn native_redraw_requested(&mut self) {
        self.native_redraw_pending = true;
        self.recorder.record_native_redraw_request();
    }

    pub(crate) fn begin_redraw(&mut self, now: Instant) -> Option<RedrawWork> {
        self.native_redraw_pending = false;

        if let Some(kind) = self.dirty.take() {
            let kind = if self.presented_once {
                kind
            } else {
                kind.merge(RedrawKind::Full)
            };
            let dirty_to_frame = self
                .dirty_since
                .take()
                .map(|dirty| now.saturating_duration_since(dirty));
            return Some(RedrawWork {
                kind,
                dirty_to_frame,
            });
        }

        if !self.presented_once {
            return Some(RedrawWork {
                kind: RedrawKind::Full,
                dirty_to_frame: None,
            });
        }

        self.recorder.record_clean_redraw_skip();
        None
    }

    pub(crate) fn record_presented(&mut self, kind: RedrawKind, timings: FrameTimings) {
        self.presented_once = true;
        self.recorder.record_frame(kind, timings);
    }

    pub(crate) fn record_scroll_only_fallback_to_full(&mut self) {
        self.recorder.record_scroll_only_fallback_to_full();
    }

    pub(crate) fn record_render_skip(&mut self) {
        self.recorder.record_render_skip();
    }

    pub(crate) fn diagnostics(&self) -> Diagnostics {
        self.recorder.diagnostics()
    }
}

#[derive(Debug, Clone)]
pub(crate) struct Recorder {
    diagnostics: Diagnostics,
    compose: TimingSeries,
    scroll_commit: TimingSeries,
    scroll_projection_sync: TimingSeries,
    paint: TimingSeries,
    render: TimingSeries,
    render_acquire: TimingSeries,
    render_batching: TimingSeries,
    render_quad_prepare: TimingSeries,
    render_text_prepare: TimingSeries,
    render_scene_text_prepare: TimingSeries,
    render_layer_update_text_prepare: TimingSeries,
    render_backdrop_prepare: TimingSeries,
    render_encode_submit: TimingSeries,
    total: TimingSeries,
    scroll_input_to_present: TimingSeries,
    dirty_to_present: TimingSeries,
    scene_items: CountSeries,
    render_batches: CountSeries,
    glyph_batches: CountSeries,
    text_surfaces: CountSeries,
    quad_vertices: CountSeries,
    clip_batches: CountSeries,
    backdrops: CountSeries,
    layer_items: CountSeries,
    layer_updates: CountSeries,
}

impl Recorder {
    fn record_frame(&mut self, kind: RedrawKind, timings: FrameTimings) {
        match kind {
            RedrawKind::ScrollOnly => self.diagnostics.scroll_only_redraws += 1,
            RedrawKind::Full => self.diagnostics.full_redraws += 1,
        }

        self.diagnostics.compose = self.compose.record(timings.stages.compose);
        self.diagnostics.scroll_commit = self.scroll_commit.record(timings.stages.scroll_commit);
        self.diagnostics.scroll_projection_sync = self
            .scroll_projection_sync
            .record(timings.stages.scroll_projection_sync);
        self.diagnostics.paint = self.paint.record(timings.stages.paint);
        self.diagnostics.render = self.render.record(timings.render);
        self.diagnostics.render_acquire = self.render_acquire.record(timings.render_stages.acquire);
        self.diagnostics.render_batching =
            self.render_batching.record(timings.render_stages.batching);
        self.diagnostics.render_quad_prepare = self
            .render_quad_prepare
            .record(timings.render_stages.quad_prepare);
        self.diagnostics.render_text_prepare = self
            .render_text_prepare
            .record(timings.render_stages.text_prepare);
        self.diagnostics.render_scene_text_prepare = self
            .render_scene_text_prepare
            .record(timings.render_stages.scene_text_prepare);
        self.diagnostics.render_layer_update_text_prepare = self
            .render_layer_update_text_prepare
            .record(timings.render_stages.layer_update_text_prepare);
        self.diagnostics.render_backdrop_prepare = self
            .render_backdrop_prepare
            .record(timings.render_stages.backdrop_prepare);
        self.diagnostics.render_encode_submit = self
            .render_encode_submit
            .record(timings.render_stages.encode_submit);
        self.diagnostics.total = self.total.record(timings.total);
        self.diagnostics.scene_items = self.scene_items.record(timings.render_stats.scene_items);
        self.diagnostics.render_batches = self
            .render_batches
            .record(timings.render_stats.render_batches);
        self.diagnostics.glyph_batches = self
            .glyph_batches
            .record(timings.render_stats.glyph_batches);
        self.diagnostics.text_surfaces = self
            .text_surfaces
            .record(timings.render_stats.text_surfaces);
        self.diagnostics.quad_vertices = self
            .quad_vertices
            .record(timings.render_stats.quad_vertices);
        self.diagnostics.clip_batches = self.clip_batches.record(timings.render_stats.clip_batches);
        self.diagnostics.backdrops = self.backdrops.record(timings.render_stats.backdrops);
        self.diagnostics.layer_items = self.layer_items.record(timings.render_stats.layer_items);
        self.diagnostics.layer_updates = self
            .layer_updates
            .record(timings.render_stats.layer_updates);
        if kind == RedrawKind::ScrollOnly {
            self.diagnostics.last_scroll_frame = LastScrollFrameDiagnostics {
                render_text_prepare_us: duration_to_micros(timings.render_stages.text_prepare),
                render_scene_text_prepare_us: duration_to_micros(
                    timings.render_stages.scene_text_prepare,
                ),
                render_layer_update_text_prepare_us: duration_to_micros(
                    timings.render_stages.layer_update_text_prepare,
                ),
                render_total_us: duration_to_micros(timings.render),
                total_us: duration_to_micros(timings.total),
                text_surfaces: timings.render_stats.text_surfaces as u64,
                glyph_batches: timings.render_stats.glyph_batches as u64,
                layer_items: timings.render_stats.layer_items as u64,
                layer_updates: timings.render_stats.layer_updates as u64,
            };
        }
        if let Some(latency) = timings.scroll_input_to_present {
            self.diagnostics.scroll_input_to_present = self.scroll_input_to_present.record(latency);
        }
        if let Some(latency) = timings.dirty_to_present {
            self.diagnostics.dirty_to_present = self.dirty_to_present.record(latency);
        }
    }

    fn record_invalidation(&mut self, coalesced: bool) {
        self.diagnostics.invalidations += 1;
        if coalesced {
            self.diagnostics.coalesced_invalidations += 1;
        }
    }

    fn record_native_redraw_request(&mut self) {
        self.diagnostics.native_redraw_requests += 1;
    }

    fn record_clean_redraw_skip(&mut self) {
        self.diagnostics.redraw_events_without_dirty += 1;
        self.diagnostics.clean_redraw_skips += 1;
    }

    fn record_scroll_only_fallback_to_full(&mut self) {
        self.diagnostics.scroll_only_fallbacks_to_full += 1;
    }

    fn record_render_skip(&mut self) {
        self.diagnostics.render_skips += 1;
    }

    pub(crate) fn diagnostics(&self) -> Diagnostics {
        self.diagnostics
    }
}

impl Default for Recorder {
    fn default() -> Self {
        Self {
            diagnostics: Diagnostics::default(),
            compose: TimingSeries::default(),
            scroll_commit: TimingSeries::default(),
            scroll_projection_sync: TimingSeries::default(),
            paint: TimingSeries::default(),
            render: TimingSeries::default(),
            render_acquire: TimingSeries::default(),
            render_batching: TimingSeries::default(),
            render_quad_prepare: TimingSeries::default(),
            render_text_prepare: TimingSeries::default(),
            render_scene_text_prepare: TimingSeries::default(),
            render_layer_update_text_prepare: TimingSeries::default(),
            render_backdrop_prepare: TimingSeries::default(),
            render_encode_submit: TimingSeries::default(),
            total: TimingSeries::default(),
            scroll_input_to_present: TimingSeries::default(),
            dirty_to_present: TimingSeries::default(),
            scene_items: CountSeries::default(),
            render_batches: CountSeries::default(),
            glyph_batches: CountSeries::default(),
            text_surfaces: CountSeries::default(),
            quad_vertices: CountSeries::default(),
            clip_batches: CountSeries::default(),
            backdrops: CountSeries::default(),
            layer_items: CountSeries::default(),
            layer_updates: CountSeries::default(),
        }
    }
}

#[derive(Debug, Clone)]
struct TimingSeries {
    samples: [u64; SAMPLE_COUNT],
    next: usize,
    len: usize,
    max: u64,
}

impl TimingSeries {
    fn record(&mut self, duration: Duration) -> TimingDiagnostics {
        let micros = duration_to_micros(duration);

        self.samples[self.next] = micros;
        self.next = (self.next + 1) % SAMPLE_COUNT;
        self.len = (self.len + 1).min(SAMPLE_COUNT);
        self.max = self.max.max(micros);

        TimingDiagnostics {
            latest_us: micros,
            max_us: self.max,
            average_us: self.average(),
            samples: self.len,
        }
    }

    fn average(&self) -> u64 {
        if self.len == 0 {
            return 0;
        }

        let total = self.samples.iter().take(self.len).copied().sum::<u64>();
        total / self.len as u64
    }
}

impl Default for TimingSeries {
    fn default() -> Self {
        Self {
            samples: [0; SAMPLE_COUNT],
            next: 0,
            len: 0,
            max: 0,
        }
    }
}

#[derive(Debug, Clone)]
struct CountSeries {
    samples: [u64; SAMPLE_COUNT],
    next: usize,
    len: usize,
    max: u64,
}

impl CountSeries {
    fn record(&mut self, value: usize) -> CountDiagnostics {
        let value = value as u64;

        self.samples[self.next] = value;
        self.next = (self.next + 1) % SAMPLE_COUNT;
        self.len = (self.len + 1).min(SAMPLE_COUNT);
        self.max = self.max.max(value);

        CountDiagnostics {
            latest: value,
            max: self.max,
            average: self.average(),
            samples: self.len,
        }
    }

    fn average(&self) -> u64 {
        if self.len == 0 {
            return 0;
        }

        let total = self.samples.iter().take(self.len).copied().sum::<u64>();
        total / self.len as u64
    }
}

impl Default for CountSeries {
    fn default() -> Self {
        Self {
            samples: [0; SAMPLE_COUNT],
            next: 0,
            len: 0,
            max: 0,
        }
    }
}

fn duration_to_micros(duration: Duration) -> u64 {
    duration.as_micros().min(u128::from(u64::MAX)) as u64
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn timing_series_tracks_latest_max_and_average() {
        let mut series = TimingSeries::default();

        let first = series.record(Duration::from_micros(10));
        assert_eq!(first.latest_us, 10);
        assert_eq!(first.max_us, 10);
        assert_eq!(first.average_us, 10);
        assert_eq!(first.samples, 1);

        let second = series.record(Duration::from_micros(30));
        assert_eq!(second.latest_us, 30);
        assert_eq!(second.max_us, 30);
        assert_eq!(second.average_us, 20);
        assert_eq!(second.samples, 2);
    }

    #[test]
    fn count_series_tracks_latest_max_and_average() {
        let mut series = CountSeries::default();

        let first = series.record(5);
        assert_eq!(first.latest, 5);
        assert_eq!(first.max, 5);
        assert_eq!(first.average, 5);
        assert_eq!(first.samples, 1);

        let second = series.record(17);
        assert_eq!(second.latest, 17);
        assert_eq!(second.max, 17);
        assert_eq!(second.average, 11);
        assert_eq!(second.samples, 2);
    }

    #[test]
    fn recorder_separates_full_and_scroll_only_frames() {
        let mut recorder = Recorder::default();

        recorder.record_frame(
            RedrawKind::Full,
            FrameTimings {
                total: Duration::from_micros(100),
                render_stages: RenderTimings {
                    text_prepare: Duration::from_micros(12),
                    scene_text_prepare: Duration::from_micros(8),
                    layer_update_text_prepare: Duration::from_micros(3),
                    ..RenderTimings::default()
                },
                render_stats: RenderStats {
                    scene_items: 40,
                    glyph_batches: 3,
                    text_surfaces: 2,
                    ..RenderStats::default()
                },
                ..FrameTimings::default()
            },
        );
        recorder.record_frame(
            RedrawKind::ScrollOnly,
            FrameTimings {
                total: Duration::from_micros(40),
                render_stages: RenderTimings {
                    text_prepare: Duration::from_micros(4),
                    scene_text_prepare: Duration::from_micros(1),
                    layer_update_text_prepare: Duration::from_micros(2),
                    ..RenderTimings::default()
                },
                render_stats: RenderStats {
                    scene_items: 10,
                    glyph_batches: 1,
                    text_surfaces: 1,
                    ..RenderStats::default()
                },
                ..FrameTimings::default()
            },
        );

        let diagnostics = recorder.diagnostics();
        assert_eq!(diagnostics.full_redraws, 1);
        assert_eq!(diagnostics.scroll_only_redraws, 1);
        assert_eq!(diagnostics.total.latest_us, 40);
        assert_eq!(diagnostics.total.max_us, 100);
        assert_eq!(diagnostics.total.average_us, 70);
        assert_eq!(diagnostics.render_text_prepare.latest_us, 4);
        assert_eq!(diagnostics.render_text_prepare.max_us, 12);
        assert_eq!(diagnostics.render_scene_text_prepare.latest_us, 1);
        assert_eq!(diagnostics.render_scene_text_prepare.max_us, 8);
        assert_eq!(diagnostics.render_layer_update_text_prepare.latest_us, 2);
        assert_eq!(diagnostics.render_layer_update_text_prepare.max_us, 3);
        assert_eq!(diagnostics.last_scroll_frame.render_text_prepare_us, 4);
        assert_eq!(
            diagnostics.last_scroll_frame.render_scene_text_prepare_us,
            1
        );
        assert_eq!(
            diagnostics
                .last_scroll_frame
                .render_layer_update_text_prepare_us,
            2
        );
        assert_eq!(diagnostics.scene_items.latest, 10);
        assert_eq!(diagnostics.scene_items.max, 40);
        assert_eq!(diagnostics.scene_items.average, 25);
        assert_eq!(diagnostics.glyph_batches.latest, 1);
        assert_eq!(diagnostics.text_surfaces.latest, 1);
    }

    #[test]
    fn full_redraw_kind_wins() {
        assert_eq!(
            RedrawKind::ScrollOnly.merge(RedrawKind::Full),
            RedrawKind::Full
        );
        assert_eq!(
            RedrawKind::Full.merge(RedrawKind::ScrollOnly),
            RedrawKind::Full
        );
    }

    #[test]
    fn state_coalesces_invalidations_before_native_redraw() {
        let mut state = State::default();
        let now = Instant::now();

        assert!(state.needs_native_redraw());
        state.native_redraw_requested();
        assert!(!state.needs_native_redraw());

        state.invalidate(RedrawKind::ScrollOnly, now);
        state.invalidate(RedrawKind::ScrollOnly, now);
        assert!(!state.needs_native_redraw());

        let diagnostics = state.diagnostics();
        assert_eq!(diagnostics.native_redraw_requests, 1);
        assert_eq!(diagnostics.invalidations, 2);
        assert_eq!(diagnostics.coalesced_invalidations, 1);
    }

    #[test]
    fn full_invalidation_overrides_scroll_only() {
        let mut state = State::default();
        let now = Instant::now();

        state.invalidate(RedrawKind::ScrollOnly, now);
        state.invalidate(RedrawKind::Full, now);

        let work = state.begin_redraw(now).expect("dirty frame");
        assert_eq!(work.kind(), RedrawKind::Full);
        assert!(work.dirty_to_frame().is_some());
    }

    #[test]
    fn first_frame_is_full_even_with_scroll_only_invalidation() {
        let mut state = State::default();
        let now = Instant::now();

        state.invalidate(RedrawKind::ScrollOnly, now);

        let work = state.begin_redraw(now).expect("first frame");
        assert_eq!(work.kind(), RedrawKind::Full);
    }

    #[test]
    fn dirty_work_is_consumed_once() {
        let mut state = State::default();
        let now = Instant::now();

        assert_eq!(
            state.begin_redraw(now).expect("first frame").kind(),
            RedrawKind::Full
        );
        state.record_presented(
            RedrawKind::Full,
            FrameTimings {
                total: Duration::from_micros(1),
                ..FrameTimings::default()
            },
        );

        state.invalidate(RedrawKind::ScrollOnly, now);
        assert_eq!(
            state.begin_redraw(now).expect("dirty frame").kind(),
            RedrawKind::ScrollOnly
        );
        state.record_presented(
            RedrawKind::ScrollOnly,
            FrameTimings {
                total: Duration::from_micros(1),
                ..FrameTimings::default()
            },
        );

        assert!(state.begin_redraw(now).is_none());
        assert_eq!(state.diagnostics().clean_redraw_skips, 1);
    }

    #[test]
    fn render_skip_records_and_can_be_reinvalidated() {
        let mut state = State::default();
        let now = Instant::now();

        state.record_render_skip();
        state.invalidate(RedrawKind::Full, now);

        assert!(state.needs_native_redraw());
        assert_eq!(state.diagnostics().render_skips, 1);
        assert_eq!(
            state.begin_redraw(now).expect("dirty frame").kind(),
            RedrawKind::Full
        );
    }
}
