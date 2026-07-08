use std::{
    collections::VecDeque,
    time::{Duration, Instant},
};

use crate::state;

const SAMPLE_LIMIT: usize = 128;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Render {
    pub frames_presented: usize,
    intervals: Samples,
    acquire_wait: Samples,
    draw: Samples,
    key_to_present: Samples,
    pending_inputs: VecDeque<InputSample>,
    last_presented_at: Option<Instant>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Samples {
    values: VecDeque<u128>,
    limit: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct InputSample {
    revision: state::Revision,
    started_at: Instant,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Report {
    acquire_wait: Duration,
    draw: Duration,
    presented_at: Instant,
}

impl Render {
    pub(crate) fn record_input(&mut self, revision: state::Revision, started_at: Instant) {
        if self.pending_inputs.len() == SAMPLE_LIMIT {
            self.pending_inputs.pop_front();
        }
        self.pending_inputs.push_back(InputSample {
            revision,
            started_at,
        });
    }

    pub(crate) fn record_present(&mut self, revision: state::Revision, report: Report) {
        self.frames_presented += 1;
        if let Some(previous) = self.last_presented_at {
            self.intervals.record(duration_micros(
                report.presented_at.saturating_duration_since(previous),
            ));
        }
        self.last_presented_at = Some(report.presented_at);
        self.acquire_wait
            .record(duration_micros(report.acquire_wait));
        self.draw.record(duration_micros(report.draw));

        let mut remaining = VecDeque::with_capacity(self.pending_inputs.len());
        while let Some(sample) = self.pending_inputs.pop_front() {
            if sample.revision <= revision {
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
            frames_presented: 0,
            intervals: Samples::default(),
            acquire_wait: Samples::default(),
            draw: Samples::default(),
            key_to_present: Samples::default(),
            pending_inputs: VecDeque::new(),
            last_presented_at: None,
        }
    }
}

impl Samples {
    pub(crate) fn record(&mut self, value: u128) {
        if self.values.len() == self.limit {
            self.values.pop_front();
        }
        self.values.push_back(value);
    }

    #[cfg(test)]
    pub fn len(&self) -> usize {
        self.values.len()
    }

    pub fn p95(&self) -> u128 {
        percentile(&self.values, 95)
    }
}

impl Default for Samples {
    fn default() -> Self {
        Self {
            values: VecDeque::with_capacity(SAMPLE_LIMIT),
            limit: SAMPLE_LIMIT,
        }
    }
}

impl Report {
    pub fn new(acquire_wait: Duration, draw: Duration, presented_at: Instant) -> Self {
        Self {
            acquire_wait,
            draw,
            presented_at,
        }
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
}

fn duration_micros(duration: Duration) -> u128 {
    duration.as_micros()
}

fn percentile(values: &VecDeque<u128>, percentile: usize) -> u128 {
    if values.is_empty() {
        return 0;
    }

    let mut sorted = values.iter().copied().collect::<Vec<_>>();
    sorted.sort_unstable();
    let rank = ((sorted.len() * percentile).div_ceil(100)).saturating_sub(1);
    sorted[rank.min(sorted.len() - 1)]
}

#[cfg(test)]
mod tests {
    use super::*;

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
    fn input_samples_wait_for_presented_revision() {
        #[derive(Clone)]
        struct Model;

        impl state::State for Model {}

        let mut store = state::Store::new(Model);
        let initial = store.revision();
        let changed = store.commit(state::Reason::programmatic("test")).revision();
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
}
