use std::{
    collections::VecDeque,
    fmt::Write as _,
    time::{Duration, Instant},
};

use super::samples::Samples;

const TRACE_LIMIT: usize = 32;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum TransitionOutcome {
    Unchanged,
    PropertyTick,
    NeedsResidency,
}

impl TransitionOutcome {
    const fn name(self) -> &'static str {
        match self {
            Self::Unchanged => "unchanged",
            Self::PropertyTick => "property-tick",
            Self::NeedsResidency => "needs-residency",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct FrameTrace {
    epoch: crate::window::PresentationEpoch,
    target_key: u64,
    first_request_serial: u64,
    last_request_serial: u64,
    coalesced_inputs: usize,
    requested: crate::interaction::ScrollOffset,
    clamped: crate::interaction::ScrollOffset,
    resident_offset: crate::interaction::ScrollOffset,
    resident_accepted: bool,
    outcome: TransitionOutcome,
    input_started_at: Option<Instant>,
    candidate_epoch: Option<crate::window::PresentationEpoch>,
    candidate_attempts: usize,
    candidate_property_serial: Option<u64>,
    gpu_submitted_property_serial: Option<u64>,
    present_submitted_property_serial: Option<u64>,
    superseded_by_epoch: Option<crate::window::PresentationEpoch>,
    input_to_present_submitted_us: Option<u128>,
}

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
    request_serial: u64,
    traces: VecDeque<FrameTrace>,
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

    pub(crate) fn record_transition(
        &mut self,
        epoch: crate::window::PresentationEpoch,
        target_key: u64,
        requested: crate::interaction::ScrollOffset,
        clamped: crate::interaction::ScrollOffset,
        resident_offset: crate::interaction::ScrollOffset,
        resident_accepted: bool,
        outcome: &'static str,
    ) {
        let outcome = match outcome {
            "unchanged" => TransitionOutcome::Unchanged,
            "property-tick" => TransitionOutcome::PropertyTick,
            "needs-residency" => TransitionOutcome::NeedsResidency,
            _ => return,
        };
        self.request_serial = self.request_serial.saturating_add(1);
        let request_serial = self.request_serial;
        if let Some(trace) = self
            .traces
            .iter_mut()
            .rev()
            .find(|trace| trace.epoch == epoch && trace.target_key == target_key)
        {
            trace.last_request_serial = request_serial;
            trace.coalesced_inputs = trace.coalesced_inputs.saturating_add(1);
            trace.requested = requested;
            trace.clamped = clamped;
            trace.resident_offset = resident_offset;
            trace.resident_accepted = resident_accepted;
            trace.outcome = outcome;
            return;
        }
        if self.traces.len() == TRACE_LIMIT {
            self.traces.pop_front();
        }
        self.traces.push_back(FrameTrace {
            epoch,
            target_key,
            first_request_serial: request_serial,
            last_request_serial: request_serial,
            coalesced_inputs: 1,
            requested,
            clamped,
            resident_offset,
            resident_accepted,
            outcome,
            input_started_at: None,
            candidate_epoch: None,
            candidate_attempts: 0,
            candidate_property_serial: None,
            gpu_submitted_property_serial: None,
            present_submitted_property_serial: None,
            superseded_by_epoch: None,
            input_to_present_submitted_us: None,
        });
    }

    pub(crate) fn record_input(
        &mut self,
        epoch: crate::window::PresentationEpoch,
        started_at: Instant,
    ) {
        if let Some(trace) = self.traces.iter_mut().rev().find(|trace| {
            trace.epoch <= epoch
                && trace.outcome != TransitionOutcome::Unchanged
                && trace.input_started_at.is_none()
        }) {
            trace.input_started_at.get_or_insert(started_at);
        }
    }

    pub(crate) fn record_candidate(
        &mut self,
        epoch: crate::window::PresentationEpoch,
        property_serial: u64,
    ) {
        if self
            .traces
            .iter()
            .any(|trace| trace.candidate_epoch == Some(epoch))
        {
            for trace in self
                .traces
                .iter_mut()
                .filter(|trace| trace.candidate_epoch == Some(epoch))
            {
                trace.candidate_attempts = trace.candidate_attempts.saturating_add(1);
                trace.candidate_property_serial = Some(property_serial);
            }
            return;
        }

        let mut selected = Vec::new();
        for (index, trace) in self.traces.iter().enumerate().rev() {
            if trace.epoch <= epoch
                && trace.outcome != TransitionOutcome::Unchanged
                && trace.candidate_property_serial.is_none()
                && trace.superseded_by_epoch.is_none()
                && !selected
                    .iter()
                    .any(|(target_key, _, _)| *target_key == trace.target_key)
            {
                selected.push((trace.target_key, index, trace.epoch));
            }
        }
        for (target_key, selected_index, selected_request_epoch) in selected.iter().copied() {
            for (index, trace) in self.traces.iter_mut().enumerate() {
                if index != selected_index
                    && trace.target_key == target_key
                    && trace.epoch <= selected_request_epoch
                    && trace.outcome != TransitionOutcome::Unchanged
                    && trace.candidate_property_serial.is_none()
                    && trace.superseded_by_epoch.is_none()
                {
                    trace.superseded_by_epoch = Some(selected_request_epoch);
                }
            }
        }
        for (_, selected_index, _) in selected {
            if let Some(trace) = self.traces.get_mut(selected_index) {
                trace.candidate_epoch = Some(epoch);
                trace.candidate_attempts = 1;
                trace.candidate_property_serial = Some(property_serial);
            }
        }
    }

    pub(crate) fn record_present_submitted(
        &mut self,
        epoch: crate::window::PresentationEpoch,
        property_serial: u64,
        presented_at: Instant,
    ) {
        for trace in self
            .traces
            .iter_mut()
            .rev()
            .filter(|trace| trace.candidate_epoch == Some(epoch))
        {
            trace.gpu_submitted_property_serial = Some(property_serial);
            trace.present_submitted_property_serial = Some(property_serial);
            trace.input_to_present_submitted_us = trace.input_started_at.map(|started_at| {
                presented_at
                    .saturating_duration_since(started_at)
                    .as_micros()
            });
        }
    }

    pub(crate) fn trace_receipt_text(&self) -> String {
        let mut receipt = String::new();
        let _ = writeln!(receipt, "scroll_trace_schema=wgpu_l3.scroll_trace.v1");
        let _ = writeln!(receipt, "scroll_trace_limit={TRACE_LIMIT}");
        let _ = writeln!(receipt, "scroll_trace_count={}", self.traces.len());
        for (index, trace) in self.traces.iter().enumerate() {
            let optional = |value: Option<u64>| {
                value.map_or_else(|| "none".to_owned(), |value| value.to_string())
            };
            let latency = trace
                .input_to_present_submitted_us
                .map_or_else(|| "none".to_owned(), |value| value.to_string());
            let _ = writeln!(
                receipt,
                "scroll_trace_{index:02}=epoch={},target_key={},first_request_serial={},last_request_serial={},coalesced_inputs={},requested_x={},requested_y={},clamped_x={},clamped_y={},resident_offset_x={},resident_offset_y={},resident_accepted={},outcome={},candidate_epoch={},candidate_attempts={},candidate_property_serial={},gpu_submitted_property_serial={},present_submitted_property_serial={},superseded_by_epoch={},input_to_present_submitted_us={}",
                trace.epoch.value(),
                trace.target_key,
                trace.first_request_serial,
                trace.last_request_serial,
                trace.coalesced_inputs,
                trace.requested.x(),
                trace.requested.y(),
                trace.clamped.x(),
                trace.clamped.y(),
                trace.resident_offset.x(),
                trace.resident_offset.y(),
                trace.resident_accepted,
                trace.outcome.name(),
                optional(
                    trace
                        .candidate_epoch
                        .map(crate::window::PresentationEpoch::value)
                ),
                trace.candidate_attempts,
                optional(trace.candidate_property_serial),
                optional(trace.gpu_submitted_property_serial),
                optional(trace.present_submitted_property_serial),
                optional(
                    trace
                        .superseded_by_epoch
                        .map(crate::window::PresentationEpoch::value)
                ),
                latency,
            );
        }
        receipt
    }
}

#[cfg(test)]
mod tests {
    use std::time::{Duration, Instant};

    use super::{Scroll, TRACE_LIMIT};
    use crate::{interaction::ScrollOffset, window::PresentationEpoch};

    #[test]
    fn trace_correlates_transition_candidate_and_present_submission() {
        let mut scroll = Scroll::default();
        let epoch = PresentationEpoch::initial().next();
        let started_at = Instant::now();
        scroll.record_transition(
            epoch,
            41,
            ScrollOffset::new(20, 0),
            ScrollOffset::new(18, 0),
            ScrollOffset::new(18, 0),
            true,
            "property-tick",
        );
        scroll.record_input(epoch, started_at);
        scroll.record_candidate(epoch, 7);
        scroll.record_present_submitted(epoch, 7, started_at + Duration::from_micros(350));

        let receipt = scroll.trace_receipt_text();
        assert!(receipt.contains("scroll_trace_count=1"));
        assert!(receipt.contains("first_request_serial=1,last_request_serial=1"));
        assert!(receipt.contains("requested_x=20"));
        assert!(receipt.contains("clamped_x=18"));
        assert!(receipt.contains("candidate_property_serial=7"));
        assert!(receipt.contains("present_submitted_property_serial=7"));
        assert!(receipt.contains("input_to_present_submitted_us=350"));
    }

    #[test]
    fn later_candidate_marks_unselected_transition_as_superseded() {
        let mut scroll = Scroll::default();
        let first = PresentationEpoch::initial().next();
        let second = first.next();
        for (epoch, offset) in [(first, 10), (second, 20)] {
            scroll.record_transition(
                epoch,
                41,
                ScrollOffset::new(offset, 0),
                ScrollOffset::new(offset, 0),
                ScrollOffset::new(offset, 0),
                true,
                "property-tick",
            );
        }
        scroll.record_candidate(second, 9);

        let receipt = scroll.trace_receipt_text();
        assert!(receipt.contains("superseded_by_epoch=2"));
        assert!(receipt.contains("candidate_property_serial=9"));
    }

    #[test]
    fn one_candidate_selects_the_latest_pending_request_for_each_target() {
        let mut scroll = Scroll::default();
        let first = PresentationEpoch::initial().next();
        let second = first.next();
        for (epoch, target_key, offset) in [(first, 41, 10), (second, 41, 20), (second, 42, 30)] {
            scroll.record_transition(
                epoch,
                target_key,
                ScrollOffset::new(offset, 0),
                ScrollOffset::new(offset, 0),
                ScrollOffset::new(offset, 0),
                true,
                "property-tick",
            );
        }
        scroll.record_candidate(second, 12);
        scroll.record_present_submitted(second, 12, Instant::now());

        let receipt = scroll.trace_receipt_text();
        assert_eq!(receipt.matches("candidate_property_serial=12").count(), 2);
        assert_eq!(
            receipt
                .matches("present_submitted_property_serial=12")
                .count(),
            2
        );
        assert_eq!(receipt.matches("superseded_by_epoch=2").count(), 1);
    }

    #[test]
    fn coalesced_requests_keep_their_serial_range() {
        let mut scroll = Scroll::default();
        let epoch = PresentationEpoch::initial().next();
        for offset in [10, 20] {
            scroll.record_transition(
                epoch,
                41,
                ScrollOffset::new(offset, 0),
                ScrollOffset::new(offset, 0),
                ScrollOffset::new(offset, 0),
                true,
                "property-tick",
            );
        }

        let receipt = scroll.trace_receipt_text();
        assert!(receipt.contains("first_request_serial=1,last_request_serial=2"));
        assert!(receipt.contains("coalesced_inputs=2"));
        assert!(receipt.contains("requested_x=20"));
    }

    #[test]
    fn residency_request_links_to_later_semantic_candidate_epoch() {
        let mut scroll = Scroll::default();
        let request_epoch = PresentationEpoch::initial().next();
        let candidate_epoch = request_epoch.next().next();
        let started_at = Instant::now();
        scroll.record_transition(
            request_epoch,
            41,
            ScrollOffset::new(80, 0),
            ScrollOffset::new(80, 0),
            ScrollOffset::new(20, 0),
            false,
            "needs-residency",
        );
        scroll.record_input(request_epoch, started_at);
        scroll.record_candidate(candidate_epoch, 11);
        scroll.record_present_submitted(
            candidate_epoch,
            11,
            started_at + Duration::from_micros(900),
        );

        let receipt = scroll.trace_receipt_text();
        assert!(receipt.contains("epoch=1"));
        assert!(receipt.contains("candidate_epoch=3"));
        assert!(receipt.contains("candidate_property_serial=11"));
        assert!(receipt.contains("present_submitted_property_serial=11"));
        assert!(receipt.contains("input_to_present_submitted_us=900"));
    }

    #[test]
    fn repeated_attempt_at_one_frame_epoch_reports_latest_submitted_serial() {
        let mut scroll = Scroll::default();
        let epoch = PresentationEpoch::initial().next();
        scroll.record_transition(
            epoch,
            41,
            ScrollOffset::new(20, 0),
            ScrollOffset::new(20, 0),
            ScrollOffset::new(20, 0),
            true,
            "property-tick",
        );
        scroll.record_candidate(epoch, 7);
        scroll.record_candidate(epoch, 8);
        scroll.record_present_submitted(epoch, 8, Instant::now());

        let receipt = scroll.trace_receipt_text();
        assert!(receipt.contains("candidate_attempts=2"));
        assert!(receipt.contains("candidate_property_serial=8"));
        assert!(receipt.contains("present_submitted_property_serial=8"));
    }

    #[test]
    fn trace_storage_is_bounded() {
        let mut scroll = Scroll::default();
        let mut epoch = PresentationEpoch::initial();
        for offset in 0..(TRACE_LIMIT + 5) {
            epoch = epoch.next();
            scroll.record_transition(
                epoch,
                offset as u64,
                ScrollOffset::new(offset as i32, 0),
                ScrollOffset::new(offset as i32, 0),
                ScrollOffset::new(offset as i32, 0),
                true,
                "property-tick",
            );
        }

        assert!(
            scroll
                .trace_receipt_text()
                .contains(&format!("scroll_trace_count={TRACE_LIMIT}"))
        );
    }

    #[test]
    fn unrelated_frame_does_not_attach_to_unchanged_scroll_input() {
        let mut scroll = Scroll::default();
        let epoch = PresentationEpoch::initial();
        scroll.record_transition(
            epoch,
            41,
            ScrollOffset::default(),
            ScrollOffset::default(),
            ScrollOffset::default(),
            false,
            "unchanged",
        );
        scroll.record_candidate(epoch, 3);
        scroll.record_present_submitted(epoch, 3, Instant::now());

        let receipt = scroll.trace_receipt_text();
        assert!(receipt.contains("outcome=unchanged"));
        assert!(receipt.contains("candidate_property_serial=none"));
        assert!(receipt.contains("present_submitted_property_serial=none"));
    }
}
