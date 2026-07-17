use std::{
    collections::VecDeque,
    fmt::Write as _,
    time::{Duration, Instant},
};

use super::samples::Samples;

const TRACE_LIMIT: usize = 32;
const VIRTUAL_RESIDENCY_ISSUE_LIMIT: usize = 2_048;

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
    redraw_requested_at: Option<Instant>,
    redraw_delivered_at: Option<Instant>,
    candidate_constructed_at: Option<Instant>,
    candidate_epoch: Option<crate::window::PresentationEpoch>,
    candidate_attempts: usize,
    candidate_property_serial: Option<u64>,
    gpu_submitted_property_serial: Option<u64>,
    present_submitted_property_serial: Option<u64>,
    superseded_by_epoch: Option<crate::window::PresentationEpoch>,
    acquire_started_at: Option<Instant>,
    acquire_finished_at: Option<Instant>,
    queue_submitted_at: Option<Instant>,
    surface_present_called_at: Option<Instant>,
    input_to_present_submitted_us: Option<u128>,
    candidate_work: Option<CandidateWork>,
    render_work: Option<RenderWork>,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub(crate) struct CandidateWork {
    layout_recomposes: usize,
    semantic_commits: usize,
    scene_node_paints: usize,
    text_line_shape_calls: usize,
    text_horizontal_index_source_bytes: usize,
    text_horizontal_window_source_bytes: usize,
    text_render_source_bytes: usize,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
struct RenderWork {
    primitive_prepare_calls: usize,
    text_prepare_calls: usize,
    text_shape_calls: usize,
    content_upload_bytes: usize,
    property_upload_bytes: usize,
    gpu_resource_creations: usize,
    gpu_resource_replacements: usize,
    gpu_resource_removals: usize,
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
    pub scroll_resident_acceptances: usize,
    pub scroll_unchanged: usize,
    pub scroll_property_ticks: usize,
    pub scroll_proactive_replenishments: usize,
    pub scroll_residency_candidates_scheduled: usize,
    pub scroll_residency_candidates_coalesced: usize,
    pub scroll_residency_candidates_selected: usize,
    pub scroll_residency_candidates_superseded: usize,
    pub scroll_residency_proactive_preemptions: usize,
    pub scroll_residency_pipelines_cancelled: usize,
    pub scroll_residency_follow_ups: usize,
    pub scroll_needs_residency: usize,
    pub desired_resident_lag_x_max: usize,
    pub desired_resident_lag_y_max: usize,
    pub virtual_residency_rejections: usize,
    pub virtual_residency_last_issue: Option<String>,
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
        self.scroll_resident_acceptances += 1;
        self.scroll_property_ticks += 1;
        self.request.record(duration.as_micros());
        self.property_tick.record(duration.as_micros());
    }

    pub(crate) fn record_needs_residency(
        &mut self,
        desired_x: i32,
        desired_y: i32,
        resident_x: i32,
        resident_y: i32,
        duration: Duration,
    ) {
        self.scroll_input_events += 1;
        self.scroll_desired_changes += 1;
        self.scroll_needs_residency += 1;
        self.desired_resident_lag_x_max = self
            .desired_resident_lag_x_max
            .max(desired_x.abs_diff(resident_x) as usize);
        self.desired_resident_lag_y_max = self
            .desired_resident_lag_y_max
            .max(desired_y.abs_diff(resident_y) as usize);
        self.request.record(duration.as_micros());
        self.needs_residency.record(duration.as_micros());
    }

    pub(crate) fn record_proactive_replenishment(&mut self) {
        self.scroll_proactive_replenishments =
            self.scroll_proactive_replenishments.saturating_add(1);
    }

    pub(crate) fn record_residency_candidate_scheduled(&mut self) {
        self.scroll_residency_candidates_scheduled =
            self.scroll_residency_candidates_scheduled.saturating_add(1);
    }

    pub(crate) fn record_residency_candidate_coalesced(&mut self) {
        self.scroll_residency_candidates_coalesced =
            self.scroll_residency_candidates_coalesced.saturating_add(1);
    }

    pub(crate) fn record_residency_candidate_selected(&mut self) {
        self.scroll_residency_candidates_selected =
            self.scroll_residency_candidates_selected.saturating_add(1);
    }

    pub(crate) fn record_residency_candidate_superseded(&mut self) {
        self.scroll_residency_candidates_superseded = self
            .scroll_residency_candidates_superseded
            .saturating_add(1);
    }

    pub(crate) fn record_residency_proactive_preemption(&mut self) {
        self.scroll_residency_proactive_preemptions = self
            .scroll_residency_proactive_preemptions
            .saturating_add(1);
    }

    pub(crate) fn record_residency_pipeline_cancelled(&mut self) {
        self.scroll_residency_pipelines_cancelled =
            self.scroll_residency_pipelines_cancelled.saturating_add(1);
    }

    pub(crate) fn record_residency_follow_up(&mut self) {
        self.scroll_residency_follow_ups = self.scroll_residency_follow_ups.saturating_add(1);
    }

    pub(crate) fn record_virtual_residency_rejection(&mut self, issue: &str) {
        self.virtual_residency_rejections = self.virtual_residency_rejections.saturating_add(1);
        self.virtual_residency_last_issue = Some(
            issue
                .chars()
                .take(VIRTUAL_RESIDENCY_ISSUE_LIMIT)
                .collect::<String>(),
        );
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
            if outcome != TransitionOutcome::Unchanged
                || trace.outcome == TransitionOutcome::Unchanged
            {
                trace.outcome = outcome;
            }
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
            redraw_requested_at: None,
            redraw_delivered_at: None,
            candidate_constructed_at: None,
            candidate_epoch: None,
            candidate_attempts: 0,
            candidate_property_serial: None,
            gpu_submitted_property_serial: None,
            present_submitted_property_serial: None,
            superseded_by_epoch: None,
            acquire_started_at: None,
            acquire_finished_at: None,
            queue_submitted_at: None,
            surface_present_called_at: None,
            input_to_present_submitted_us: None,
            candidate_work: None,
            render_work: None,
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

    pub(crate) fn record_redraw_requested(
        &mut self,
        epoch: crate::window::PresentationEpoch,
        requested_at: Instant,
    ) {
        for trace in self.traces.iter_mut().rev().filter(|trace| {
            trace.epoch <= epoch
                && trace.outcome != TransitionOutcome::Unchanged
                && trace.candidate_property_serial.is_none()
                && trace.superseded_by_epoch.is_none()
        }) {
            trace.redraw_requested_at.get_or_insert(requested_at);
        }
    }

    pub(crate) fn record_redraw_delivered(
        &mut self,
        epoch: crate::window::PresentationEpoch,
        delivered_at: Instant,
    ) {
        for trace in self.traces.iter_mut().rev().filter(|trace| {
            trace.epoch <= epoch
                && trace.redraw_requested_at.is_some()
                && trace.redraw_delivered_at.is_none()
                && trace.superseded_by_epoch.is_none()
        }) {
            trace.redraw_delivered_at = Some(delivered_at);
        }
    }

    pub(crate) fn record_candidate_constructed(
        &mut self,
        epoch: crate::window::PresentationEpoch,
        property_serial: u64,
        constructed_at: Instant,
        work: CandidateWork,
    ) {
        self.record_candidate(epoch, property_serial);
        for trace in self
            .traces
            .iter_mut()
            .filter(|trace| trace.candidate_epoch == Some(epoch))
        {
            trace.candidate_constructed_at = Some(constructed_at);
            if trace.outcome == TransitionOutcome::NeedsResidency {
                trace.candidate_work = Some(
                    trace
                        .candidate_work
                        .map_or(work, |current| current.saturating_add(work)),
                );
            }
        }
    }

    pub(crate) fn record_render_work(
        &mut self,
        epoch: crate::window::PresentationEpoch,
        report: &crate::render::RenderReport,
    ) {
        let work = RenderWork::from_report(report);
        for trace in self.traces.iter_mut().filter(|trace| {
            trace.candidate_epoch == Some(epoch)
                && trace.outcome == TransitionOutcome::NeedsResidency
        }) {
            trace.render_work = Some(
                trace
                    .render_work
                    .map_or(work, |current| current.saturating_add(work)),
            );
        }
    }

    pub(crate) fn record_frame_timeline(
        &mut self,
        epoch: crate::window::PresentationEpoch,
        timeline: crate::render::FrameTimeline,
    ) {
        for trace in self
            .traces
            .iter_mut()
            .filter(|trace| trace.candidate_epoch == Some(epoch))
        {
            trace.acquire_started_at = timeline.acquire_started_at();
            trace.acquire_finished_at = timeline.acquire_finished_at();
            trace.queue_submitted_at = timeline.queue_submitted_at();
            trace.surface_present_called_at = timeline.surface_present_called_at();
        }
    }

    pub(crate) fn record_present_submitted(
        &mut self,
        epoch: crate::window::PresentationEpoch,
        property_serial: u64,
        present_submitted_at: Instant,
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
                present_submitted_at
                    .saturating_duration_since(started_at)
                    .as_micros()
            });
        }
    }

    pub(crate) fn trace_receipt_text(&self) -> String {
        let mut receipt = String::new();
        let _ = writeln!(receipt, "scroll_trace_schema=wgpu_l3.scroll_trace.v3");
        let _ = writeln!(receipt, "scroll_trace_limit={TRACE_LIMIT}");
        let _ = writeln!(receipt, "scroll_trace_count={}", self.traces.len());
        for (index, trace) in self.traces.iter().enumerate() {
            let optional = |value: Option<u64>| {
                value.map_or_else(|| "none".to_owned(), |value| value.to_string())
            };
            let optional_count = |value: Option<usize>| {
                value.map_or_else(|| "none".to_owned(), |value| value.to_string())
            };
            let latency = trace
                .input_to_present_submitted_us
                .map_or_else(|| "none".to_owned(), |value| value.to_string());
            let stage_latency = |stage: Option<Instant>| {
                trace.input_started_at.zip(stage).map_or_else(
                    || "none".to_owned(),
                    |(input, stage)| {
                        stage
                            .saturating_duration_since(input)
                            .as_micros()
                            .to_string()
                    },
                )
            };
            let acquire_wait = trace
                .acquire_started_at
                .zip(trace.acquire_finished_at)
                .map_or_else(
                    || "none".to_owned(),
                    |(started, finished)| {
                        finished
                            .saturating_duration_since(started)
                            .as_micros()
                            .to_string()
                    },
                );
            let _ = writeln!(
                receipt,
                "scroll_trace_{index:02}=epoch={},target_key={},first_request_serial={},last_request_serial={},coalesced_inputs={},requested_x={},requested_y={},clamped_x={},clamped_y={},resident_offset_x={},resident_offset_y={},resident_accepted={},outcome={},candidate_epoch={},candidate_attempts={},candidate_property_serial={},gpu_submitted_property_serial={},present_submitted_property_serial={},superseded_by_epoch={},input_to_redraw_request_us={},input_to_redraw_delivery_us={},input_to_candidate_us={},input_to_acquire_start_us={},acquire_wait_us={},input_to_queue_submit_us={},input_to_surface_present_call_us={},input_to_present_submitted_us={},residency_layout_recomposes={},residency_semantic_commits={},residency_scene_node_paints={},residency_text_line_shape_calls={},residency_text_horizontal_index_source_bytes={},residency_text_horizontal_window_source_bytes={},residency_text_render_source_bytes={},residency_primitive_prepare_calls={},residency_text_prepare_calls={},residency_text_shape_calls={},residency_content_upload_bytes={},residency_property_upload_bytes={},residency_gpu_resource_creations={},residency_gpu_resource_replacements={},residency_gpu_resource_removals={}",
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
                stage_latency(trace.redraw_requested_at),
                stage_latency(trace.redraw_delivered_at),
                stage_latency(trace.candidate_constructed_at),
                stage_latency(trace.acquire_started_at),
                acquire_wait,
                stage_latency(trace.queue_submitted_at),
                stage_latency(trace.surface_present_called_at),
                latency,
                optional_count(trace.candidate_work.map(|work| work.layout_recomposes)),
                optional_count(trace.candidate_work.map(|work| work.semantic_commits)),
                optional_count(trace.candidate_work.map(|work| work.scene_node_paints)),
                optional_count(trace.candidate_work.map(|work| work.text_line_shape_calls)),
                optional_count(
                    trace
                        .candidate_work
                        .map(|work| work.text_horizontal_index_source_bytes)
                ),
                optional_count(
                    trace
                        .candidate_work
                        .map(|work| work.text_horizontal_window_source_bytes)
                ),
                optional_count(
                    trace
                        .candidate_work
                        .map(|work| work.text_render_source_bytes)
                ),
                optional_count(trace.render_work.map(|work| work.primitive_prepare_calls)),
                optional_count(trace.render_work.map(|work| work.text_prepare_calls)),
                optional_count(trace.render_work.map(|work| work.text_shape_calls)),
                optional_count(trace.render_work.map(|work| work.content_upload_bytes)),
                optional_count(trace.render_work.map(|work| work.property_upload_bytes)),
                optional_count(trace.render_work.map(|work| work.gpu_resource_creations)),
                optional_count(trace.render_work.map(|work| work.gpu_resource_replacements)),
                optional_count(trace.render_work.map(|work| work.gpu_resource_removals)),
            );
        }
        receipt
    }
}

impl CandidateWork {
    fn saturating_add(self, other: Self) -> Self {
        Self {
            layout_recomposes: self
                .layout_recomposes
                .saturating_add(other.layout_recomposes),
            semantic_commits: self.semantic_commits.saturating_add(other.semantic_commits),
            scene_node_paints: self
                .scene_node_paints
                .saturating_add(other.scene_node_paints),
            text_line_shape_calls: self
                .text_line_shape_calls
                .saturating_add(other.text_line_shape_calls),
            text_horizontal_index_source_bytes: self
                .text_horizontal_index_source_bytes
                .saturating_add(other.text_horizontal_index_source_bytes),
            text_horizontal_window_source_bytes: self
                .text_horizontal_window_source_bytes
                .saturating_add(other.text_horizontal_window_source_bytes),
            text_render_source_bytes: self
                .text_render_source_bytes
                .saturating_add(other.text_render_source_bytes),
        }
    }

    pub(crate) fn snapshot(diagnostics: Option<&super::Diagnostics>) -> Self {
        diagnostics.map_or_else(Self::default, |diagnostics| Self {
            layout_recomposes: diagnostics.frame.layout_recomposes,
            text_line_shape_calls: diagnostics.text.text_area_line_shape_calls,
            text_horizontal_index_source_bytes: diagnostics
                .text
                .text_area_horizontal_index_source_bytes,
            text_horizontal_window_source_bytes: diagnostics
                .text
                .text_area_horizontal_window_source_bytes,
            text_render_source_bytes: diagnostics.text.text_area_render_surface_source_bytes,
            ..Self::default()
        })
    }

    pub(crate) fn since(
        before: Self,
        diagnostics: Option<&super::Diagnostics>,
        scene: crate::scene::PaintStats,
    ) -> Self {
        let after = Self::snapshot(diagnostics);
        Self {
            layout_recomposes: after
                .layout_recomposes
                .saturating_sub(before.layout_recomposes),
            semantic_commits: scene.commits_created(),
            scene_node_paints: scene.node_paints(),
            text_line_shape_calls: after
                .text_line_shape_calls
                .saturating_sub(before.text_line_shape_calls),
            text_horizontal_index_source_bytes: after
                .text_horizontal_index_source_bytes
                .saturating_sub(before.text_horizontal_index_source_bytes),
            text_horizontal_window_source_bytes: after
                .text_horizontal_window_source_bytes
                .saturating_sub(before.text_horizontal_window_source_bytes),
            text_render_source_bytes: after
                .text_render_source_bytes
                .saturating_sub(before.text_render_source_bytes),
        }
    }
}

impl RenderWork {
    fn saturating_add(self, other: Self) -> Self {
        Self {
            primitive_prepare_calls: self
                .primitive_prepare_calls
                .saturating_add(other.primitive_prepare_calls),
            text_prepare_calls: self
                .text_prepare_calls
                .saturating_add(other.text_prepare_calls),
            text_shape_calls: self.text_shape_calls.saturating_add(other.text_shape_calls),
            content_upload_bytes: self
                .content_upload_bytes
                .saturating_add(other.content_upload_bytes),
            property_upload_bytes: self
                .property_upload_bytes
                .saturating_add(other.property_upload_bytes),
            gpu_resource_creations: self
                .gpu_resource_creations
                .saturating_add(other.gpu_resource_creations),
            gpu_resource_replacements: self
                .gpu_resource_replacements
                .saturating_add(other.gpu_resource_replacements),
            gpu_resource_removals: self
                .gpu_resource_removals
                .saturating_add(other.gpu_resource_removals),
        }
    }

    fn from_report(report: &crate::render::RenderReport) -> Self {
        let stats = report.draw_stats;
        Self {
            primitive_prepare_calls: stats.quad_prepare_calls,
            text_prepare_calls: stats.text_prepare_calls,
            text_shape_calls: stats.inline_text_shape_calls,
            content_upload_bytes: stats.content_upload_bytes,
            property_upload_bytes: stats.property_upload_bytes,
            gpu_resource_creations: stats.retained_gpu_resource_creations,
            gpu_resource_replacements: stats.retained_gpu_resource_replacements,
            gpu_resource_removals: stats.retained_gpu_resource_removals,
        }
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
    fn trace_correlates_every_pacing_stage_without_log_order() {
        let mut scroll = Scroll::default();
        let epoch = PresentationEpoch::initial().next();
        let input = Instant::now();
        scroll.record_transition(
            epoch,
            44,
            ScrollOffset::new(0, 20),
            ScrollOffset::new(0, 20),
            ScrollOffset::new(0, 20),
            true,
            "property-tick",
        );
        scroll.record_input(epoch, input);
        scroll.record_redraw_requested(epoch, input + Duration::from_micros(10));
        scroll.record_redraw_delivered(epoch, input + Duration::from_micros(20));
        scroll.record_candidate_constructed(
            epoch,
            13,
            input + Duration::from_micros(30),
            super::CandidateWork::default(),
        );
        scroll.record_frame_timeline(
            epoch,
            crate::render::FrameTimeline::new(
                input + Duration::from_micros(40),
                input + Duration::from_micros(45),
                Some(input + Duration::from_micros(50)),
                Some(input + Duration::from_micros(55)),
            ),
        );
        scroll.record_present_submitted(epoch, 13, input + Duration::from_micros(55));

        let receipt = scroll.trace_receipt_text();
        for field in [
            "input_to_redraw_request_us=10",
            "input_to_redraw_delivery_us=20",
            "input_to_candidate_us=30",
            "input_to_acquire_start_us=40",
            "acquire_wait_us=5",
            "input_to_queue_submit_us=50",
            "input_to_surface_present_call_us=55",
            "input_to_present_submitted_us=55",
        ] {
            assert!(receipt.contains(field), "missing causal field {field}");
        }
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
    fn repeated_no_op_does_not_erase_a_coalesced_residency_transition() {
        let mut scroll = Scroll::default();
        let epoch = PresentationEpoch::initial().next();
        scroll.record_transition(
            epoch,
            41,
            ScrollOffset::new(80, 0),
            ScrollOffset::new(80, 0),
            ScrollOffset::new(20, 0),
            false,
            "needs-residency",
        );
        scroll.record_transition(
            epoch,
            41,
            ScrollOffset::new(80, 0),
            ScrollOffset::new(80, 0),
            ScrollOffset::new(20, 0),
            false,
            "unchanged",
        );

        let receipt = scroll.trace_receipt_text();
        assert!(receipt.contains("coalesced_inputs=2"));
        assert!(receipt.contains("outcome=needs-residency"));
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
    fn residency_work_is_attributed_to_its_selected_property_generation() {
        let mut scroll = Scroll::default();
        let request_epoch = PresentationEpoch::initial().next();
        let candidate_epoch = request_epoch.next();
        let now = Instant::now();
        scroll.record_transition(
            request_epoch,
            91,
            ScrollOffset::new(0, 80),
            ScrollOffset::new(0, 80),
            ScrollOffset::new(0, 20),
            false,
            "needs-residency",
        );
        scroll.record_candidate_constructed(
            candidate_epoch,
            17,
            now,
            super::CandidateWork {
                layout_recomposes: 1,
                semantic_commits: 0,
                scene_node_paints: 7,
                text_line_shape_calls: 3,
                text_horizontal_index_source_bytes: 0,
                text_horizontal_window_source_bytes: 240,
                text_render_source_bytes: 240,
            },
        );
        scroll.record_candidate_constructed(
            candidate_epoch,
            18,
            now,
            super::CandidateWork {
                layout_recomposes: 2,
                semantic_commits: 0,
                scene_node_paints: 1,
                text_line_shape_calls: 1,
                text_horizontal_index_source_bytes: 0,
                text_horizontal_window_source_bytes: 10,
                text_render_source_bytes: 20,
            },
        );
        let mut draw = crate::render::DrawStats::default();
        draw.quad_prepare_calls = 2;
        draw.text_prepare_calls = 1;
        draw.inline_text_shape_calls = 0;
        draw.content_upload_bytes = 384;
        draw.property_upload_bytes = 64;
        draw.retained_gpu_resource_creations = 1;
        let report = crate::render::RenderReport::new(Duration::ZERO, Duration::ZERO, now)
            .with_draw_stats(draw);
        scroll.record_render_work(candidate_epoch, &report);
        scroll.record_render_work(candidate_epoch, &report);
        scroll.record_present_submitted(candidate_epoch, 18, now);

        let receipt = scroll.trace_receipt_text();
        for field in [
            "candidate_attempts=2",
            "candidate_property_serial=18",
            "present_submitted_property_serial=18",
            "residency_layout_recomposes=3",
            "residency_semantic_commits=0",
            "residency_scene_node_paints=8",
            "residency_text_line_shape_calls=4",
            "residency_text_horizontal_window_source_bytes=250",
            "residency_text_render_source_bytes=260",
            "residency_primitive_prepare_calls=4",
            "residency_text_prepare_calls=2",
            "residency_content_upload_bytes=768",
            "residency_property_upload_bytes=128",
            "residency_gpu_resource_creations=2",
        ] {
            assert!(receipt.contains(field), "missing residency field {field}");
        }
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
