use std::time::Duration;

use super::samples::Samples;

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Scroll {
    pub wheel_events: usize,
    pub scroll_offset_changes: usize,
    pub scroll_redraw_requests: usize,
    pub frame_scroll_commits: usize,
    pub text_area_viewports: usize,
    pub scroll_input_events: usize,
    pub scroll_desired_changes: usize,
    pub scroll_admitted_changes: usize,
    pub scroll_unchanged: usize,
    pub scroll_property_ticks: usize,
    pub scroll_needs_residency: usize,
    pub desired_admitted_lag_x_max: usize,
    pub desired_admitted_lag_y_max: usize,
    request: Samples,
    property_tick: Samples,
    needs_residency: Samples,
}

impl Scroll {
    pub(crate) fn record_unchanged(&mut self, duration: Duration) {
        self.scroll_input_events += 1;
        self.scroll_unchanged += 1;
        self.request.record(duration.as_micros());
    }

    pub(crate) fn record_property_tick(&mut self, duration: Duration) {
        self.scroll_input_events += 1;
        self.scroll_desired_changes += 1;
        self.scroll_admitted_changes += 1;
        self.scroll_property_ticks += 1;
        self.request.record(duration.as_micros());
        self.property_tick.record(duration.as_micros());
    }

    pub(crate) fn record_needs_residency(
        &mut self,
        desired_x: i32,
        desired_y: i32,
        admitted_x: i32,
        admitted_y: i32,
        duration: Duration,
    ) {
        self.scroll_input_events += 1;
        self.scroll_desired_changes += 1;
        self.scroll_needs_residency += 1;
        self.desired_admitted_lag_x_max = self
            .desired_admitted_lag_x_max
            .max(desired_x.abs_diff(admitted_x) as usize);
        self.desired_admitted_lag_y_max = self
            .desired_admitted_lag_y_max
            .max(desired_y.abs_diff(admitted_y) as usize);
        self.request.record(duration.as_micros());
        self.needs_residency.record(duration.as_micros());
    }

    pub fn request_p95_us(&self) -> u128 {
        self.request.p95()
    }

    pub fn property_tick_p95_us(&self) -> u128 {
        self.property_tick.p95()
    }

    pub fn needs_residency_p95_us(&self) -> u128 {
        self.needs_residency.p95()
    }
}
