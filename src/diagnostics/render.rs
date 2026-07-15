use std::{
    collections::VecDeque,
    fmt::Write as _,
    time::SystemTime,
    time::{Duration, Instant},
};

use crate::render::{RenderReport, RendererEnvironment};

use super::samples::{SAMPLE_LIMIT, Samples};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Render {
    pub environment: Option<RendererEnvironment>,
    pub semantic_commits_created: usize,
    pub semantic_commits_activated: usize,
    pub scene_nodes_added: usize,
    pub scene_nodes_removed: usize,
    pub scene_nodes_reused: usize,
    pub scene_nodes_rebuilt: usize,
    pub scene_paint_calls: usize,
    pub scene_painted_nodes: usize,
    pub text_nodes_reused: usize,
    pub property_ticks: usize,
    pub changed_property_values: usize,
    pub property_upload_bytes: usize,
    pub candidate_property_serial: u64,
    pub sampled_property_serial: u64,
    pub visible_property_serial: u64,
    pub skipped_property_attempts: usize,
    pub virtual_guard_crossings: usize,
    pub replenishment_commits: usize,
    pub frames_attempted: usize,
    pub frames_presented: usize,
    pub property_frames_attempted: usize,
    pub property_frames_presented: usize,
    pub semantic_frames_attempted: usize,
    pub semantic_frames_presented: usize,
    pub missed_refresh_opportunities: usize,
    pub renderer_deadline_misses: usize,
    pub property_renderer_deadline_misses: usize,
    pub semantic_renderer_deadline_misses: usize,
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
    pub scene_node_realization_rebuilds: usize,
    pub scene_node_realization_rebuilds_total: usize,
    pub quad_prepare_calls: usize,
    pub quad_prepare_calls_total: usize,
    pub text_prepare_calls: usize,
    pub text_prepare_calls_total: usize,
    pub quad_instances: usize,
    pub quad_vertices: usize,
    pub geometry_upload_bytes: usize,
    pub geometry_upload_bytes_total: usize,
    pub content_upload_bytes: usize,
    pub content_upload_bytes_total: usize,
    pub geometry_buffer_creations: usize,
    pub geometry_buffer_creations_total: usize,
    pub retained_gpu_resource_count: usize,
    pub retained_gpu_resource_count_high_water: usize,
    pub retained_gpu_resource_bytes: usize,
    pub retained_gpu_resource_bytes_high_water: usize,
    pub retained_gpu_resource_creations: usize,
    pub retained_gpu_resource_replacements: usize,
    pub retained_gpu_resource_removals: usize,
    pub commit_preparation_slices: usize,
    pub commit_preparation_total_nanos: u64,
    pub commit_preparation_max_nanos: u64,
    pub commit_preparation_deadline_misses: usize,
    pub render_plan_rebuilds: usize,
    pub render_plan_rebuilds_total: usize,
    pub render_plan_reuses: usize,
    pub render_plan_reuses_total: usize,
    pub direct_surface_plans: usize,
    pub direct_surface_plans_total: usize,
    pub surface_sampling_plans: usize,
    pub surface_sampling_plans_total: usize,
    pub draw_calls: usize,
    pub draw_calls_total: usize,
    pub draw_passes: usize,
    pub draw_passes_total: usize,
    pub explicit_copy_commands: usize,
    pub explicit_copy_commands_total: usize,
    pub resource_transition_boundaries: usize,
    pub resource_transition_boundaries_total: usize,
    pub pipeline_changes: usize,
    pub pipeline_changes_total: usize,
    pub bind_group_changes: usize,
    pub bind_group_changes_total: usize,
    pub scroll_layer_cache_hits: usize,
    pub scroll_layer_cache_hits_total: usize,
    pub scroll_layer_cache_misses: usize,
    pub scroll_layer_cache_misses_total: usize,
    pub clip_batches: usize,
    pub opaque_nodes: usize,
    pub blended_nodes: usize,
    pub opacity_unclassified_nodes: usize,
    pub clipped_nodes: usize,
    pub effect_island_nodes: usize,
    pub effect_intermediate_clears: usize,
    pub effect_intermediate_clears_total: usize,
    pub effect_intermediate_clear_bytes_total: u64,
    pub effect_intermediate_composites: usize,
    pub effect_intermediate_composites_total: usize,
    pub effect_intermediate_composite_bytes_total: u64,
    pub largest_effect_intermediate_bytes_high_water: u64,
    pub culled_nodes: usize,
    pub group_composites: usize,
    pub filter_layer_pool_entries: usize,
    pub filter_layer_pool_entries_high_water: usize,
    pub filter_scratch_pool_entries: usize,
    pub filter_scratch_pool_entries_high_water: usize,
    pub ordinary_surface_clears: usize,
    pub ordinary_surface_clears_total: usize,
    pub ordinary_surface_clear_bytes_total: u64,
    pub extra_full_surface_intermediate_clears: usize,
    pub extra_full_surface_intermediate_clears_total: usize,
    pub extra_full_surface_intermediate_clear_bytes_total: u64,
    pub full_surface_blits: usize,
    pub full_surface_blits_total: usize,
    pub full_surface_blit_bytes_total: u64,
    pub popup_surface_packs: usize,
    pub popup_surface_packs_total: usize,
    pub popup_surface_pack_bytes_total: u64,
    pub acquire_successes: usize,
    pub acquire_suboptimal: usize,
    pub acquire_outdated: usize,
    pub acquire_timeouts: usize,
    pub acquire_occluded: usize,
    pub acquire_validation: usize,
    pub acquire_lost: usize,
    intervals: Samples,
    acquire_wait: Samples,
    batch_prepare: Samples,
    encode_submit_present: Samples,
    draw: Samples,
    property_batch_prepare: Samples,
    property_encode_submit_present: Samples,
    property_draw: Samples,
    semantic_batch_prepare: Samples,
    semantic_encode_submit_present: Samples,
    semantic_draw: Samples,
    key_to_present: Samples,
    replenishment_commit: Samples,
    pending_inputs: VecDeque<InputSample>,
    last_presented_at: Option<Instant>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct InputSample {
    epoch: crate::window::PresentationEpoch,
    started_at: Instant,
}

impl Render {
    pub(crate) fn record_property_attempt(
        &mut self,
        properties: &crate::scene::Properties,
        property_only: bool,
        presented: bool,
    ) {
        let serial = properties.serial().value();
        self.candidate_property_serial = serial;
        self.sampled_property_serial = serial;
        if property_only {
            self.property_ticks += 1;
            self.changed_property_values += properties.changed().len();
            if !presented {
                self.skipped_property_attempts += 1;
            }
        }
        if presented {
            self.visible_property_serial = serial;
        }
    }

    pub(crate) fn record_scene_projection(&mut self, stats: crate::scene::PaintStats) {
        self.semantic_commits_created += stats.commits_created();
        self.scene_nodes_added += stats.nodes_added();
        self.scene_nodes_removed += stats.nodes_removed();
        self.scene_nodes_reused += stats.node_reuses();
        self.scene_nodes_rebuilt += stats.node_paints();
        self.scene_paint_calls += stats.node_paints() + stats.auxiliary_paints();
        self.scene_painted_nodes += stats.node_paints();
    }

    pub(crate) fn record_semantic_activation(&mut self) {
        self.semantic_commits_activated += 1;
    }

    pub(crate) fn record_replenishment_commit(&mut self, elapsed: Duration) {
        self.virtual_guard_crossings += 1;
        self.replenishment_commits += 1;
        self.replenishment_commit.record(duration_micros(elapsed));
    }

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
        property_only: bool,
        report: RenderReport,
    ) {
        self.frames_attempted += 1;
        if property_only {
            self.property_frames_attempted += 1;
        } else {
            self.semantic_frames_attempted += 1;
        }
        if let Some(environment) = report.environment.clone() {
            self.environment = Some(environment);
        }
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
        self.scene_node_realization_rebuilds = report.draw_stats.scene_node_realization_rebuilds;
        self.scene_node_realization_rebuilds_total +=
            report.draw_stats.scene_node_realization_rebuilds;
        self.quad_prepare_calls = report.draw_stats.quad_prepare_calls;
        self.quad_prepare_calls_total += report.draw_stats.quad_prepare_calls;
        self.text_prepare_calls = report.draw_stats.text_prepare_calls;
        self.text_prepare_calls_total += report.draw_stats.text_prepare_calls;
        self.quad_instances = report.draw_stats.quad_instances;
        self.quad_vertices = report.draw_stats.quad_vertices;
        self.geometry_upload_bytes = report.draw_stats.geometry_upload_bytes;
        self.geometry_upload_bytes_total += report.draw_stats.geometry_upload_bytes;
        self.content_upload_bytes = report.draw_stats.content_upload_bytes;
        self.content_upload_bytes_total += report.draw_stats.content_upload_bytes;
        self.property_upload_bytes += report.draw_stats.property_upload_bytes;
        self.geometry_buffer_creations = report.draw_stats.geometry_buffer_creations;
        self.geometry_buffer_creations_total += report.draw_stats.geometry_buffer_creations;
        self.retained_gpu_resource_count = report.draw_stats.retained_gpu_resource_count;
        self.retained_gpu_resource_count_high_water = self
            .retained_gpu_resource_count_high_water
            .max(report.draw_stats.retained_gpu_resource_count);
        self.retained_gpu_resource_bytes = report.draw_stats.retained_gpu_resource_bytes;
        self.retained_gpu_resource_bytes_high_water = self
            .retained_gpu_resource_bytes_high_water
            .max(report.draw_stats.retained_gpu_resource_bytes);
        self.retained_gpu_resource_creations += report.draw_stats.retained_gpu_resource_creations;
        self.retained_gpu_resource_replacements +=
            report.draw_stats.retained_gpu_resource_replacements;
        self.retained_gpu_resource_removals += report.draw_stats.retained_gpu_resource_removals;
        self.commit_preparation_slices += report.draw_stats.commit_preparation_slices;
        self.commit_preparation_total_nanos = self
            .commit_preparation_total_nanos
            .saturating_add(report.draw_stats.commit_preparation_total_nanos);
        self.commit_preparation_max_nanos = self
            .commit_preparation_max_nanos
            .max(report.draw_stats.commit_preparation_max_nanos);
        self.commit_preparation_deadline_misses +=
            report.draw_stats.commit_preparation_deadline_misses;
        self.render_plan_rebuilds = report.draw_stats.render_plan_rebuilds;
        self.render_plan_rebuilds_total += report.draw_stats.render_plan_rebuilds;
        self.render_plan_reuses = report.draw_stats.render_plan_reuses;
        self.render_plan_reuses_total += report.draw_stats.render_plan_reuses;
        self.direct_surface_plans = report.draw_stats.direct_surface_plans;
        self.direct_surface_plans_total += report.draw_stats.direct_surface_plans;
        self.surface_sampling_plans = report.draw_stats.surface_sampling_plans;
        self.surface_sampling_plans_total += report.draw_stats.surface_sampling_plans;
        self.draw_calls = report.draw_stats.draw_calls;
        self.draw_calls_total += report.draw_stats.draw_calls;
        self.draw_passes = report.draw_stats.draw_passes;
        self.draw_passes_total += report.draw_stats.draw_passes;
        self.explicit_copy_commands = report.draw_stats.explicit_copy_commands;
        self.explicit_copy_commands_total += report.draw_stats.explicit_copy_commands;
        self.resource_transition_boundaries = report.draw_stats.resource_transition_boundaries;
        self.resource_transition_boundaries_total +=
            report.draw_stats.resource_transition_boundaries;
        self.pipeline_changes = report.draw_stats.pipeline_changes;
        self.pipeline_changes_total += report.draw_stats.pipeline_changes;
        self.bind_group_changes = report.draw_stats.bind_group_changes;
        self.bind_group_changes_total += report.draw_stats.bind_group_changes;
        self.scroll_layer_cache_hits = report.draw_stats.scroll_layer_cache_hits;
        self.scroll_layer_cache_hits_total += report.draw_stats.scroll_layer_cache_hits;
        self.scroll_layer_cache_misses = report.draw_stats.scroll_layer_cache_misses;
        self.scroll_layer_cache_misses_total += report.draw_stats.scroll_layer_cache_misses;
        self.clip_batches = report.draw_stats.clip_batches;
        self.opaque_nodes = report.draw_stats.opaque_nodes;
        self.blended_nodes = report.draw_stats.blended_nodes;
        self.opacity_unclassified_nodes = report.draw_stats.opacity_unclassified_nodes;
        self.clipped_nodes = report.draw_stats.clipped_nodes;
        self.effect_island_nodes = report.draw_stats.effect_island_nodes;
        self.effect_intermediate_clears = report.draw_stats.effect_intermediate_clears;
        self.effect_intermediate_clears_total += report.draw_stats.effect_intermediate_clears;
        self.effect_intermediate_clear_bytes_total +=
            report.draw_stats.effect_intermediate_clear_bytes;
        self.effect_intermediate_composites = report.draw_stats.effect_intermediate_composites;
        self.effect_intermediate_composites_total +=
            report.draw_stats.effect_intermediate_composites;
        self.effect_intermediate_composite_bytes_total +=
            report.draw_stats.effect_intermediate_composite_bytes;
        self.largest_effect_intermediate_bytes_high_water = self
            .largest_effect_intermediate_bytes_high_water
            .max(report.draw_stats.largest_effect_intermediate_bytes);
        self.culled_nodes = report.draw_stats.culled_nodes;
        self.group_composites = report
            .group_composites
            .max(report.draw_stats.group_composites);
        self.filter_layer_pool_entries = report.filter_layer_pool_entries;
        self.filter_layer_pool_entries_high_water = self
            .filter_layer_pool_entries_high_water
            .max(report.filter_layer_pool_entries);
        self.filter_scratch_pool_entries = report.filter_scratch_pool_entries;
        self.filter_scratch_pool_entries_high_water = self
            .filter_scratch_pool_entries_high_water
            .max(report.filter_scratch_pool_entries);
        self.ordinary_surface_clears = report.draw_stats.ordinary_surface_clears;
        self.ordinary_surface_clears_total += report.draw_stats.ordinary_surface_clears;
        self.ordinary_surface_clear_bytes_total += report.draw_stats.ordinary_surface_clear_bytes;
        self.extra_full_surface_intermediate_clears =
            report.draw_stats.extra_full_surface_intermediate_clears;
        self.extra_full_surface_intermediate_clears_total +=
            report.draw_stats.extra_full_surface_intermediate_clears;
        self.extra_full_surface_intermediate_clear_bytes_total += report
            .draw_stats
            .extra_full_surface_intermediate_clear_bytes;
        self.full_surface_blits = report.draw_stats.full_surface_blits;
        self.full_surface_blits_total += report.draw_stats.full_surface_blits;
        self.full_surface_blit_bytes_total += report.draw_stats.full_surface_blit_bytes;
        self.popup_surface_packs = report.draw_stats.popup_surface_packs;
        self.popup_surface_packs_total += report.draw_stats.popup_surface_packs;
        self.popup_surface_pack_bytes_total += report.draw_stats.popup_surface_pack_bytes;
        match report.acquire_outcome {
            crate::render::surface::AcquireOutcome::Success => self.acquire_successes += 1,
            crate::render::surface::AcquireOutcome::Suboptimal => self.acquire_suboptimal += 1,
            crate::render::surface::AcquireOutcome::Outdated => self.acquire_outdated += 1,
            crate::render::surface::AcquireOutcome::Timeout => self.acquire_timeouts += 1,
            crate::render::surface::AcquireOutcome::Occluded => self.acquire_occluded += 1,
            crate::render::surface::AcquireOutcome::Validation => self.acquire_validation += 1,
            crate::render::surface::AcquireOutcome::Lost => self.acquire_lost += 1,
        }
        self.acquire_wait
            .record(duration_micros(report.acquire_wait));
        self.batch_prepare
            .record(duration_micros(report.batch_prepare));
        self.encode_submit_present
            .record(duration_micros(report.encode_submit_present));
        self.draw.record(duration_micros(report.draw));
        let (batch_prepare, encode_submit_present, draw) = if property_only {
            (
                &mut self.property_batch_prepare,
                &mut self.property_encode_submit_present,
                &mut self.property_draw,
            )
        } else {
            (
                &mut self.semantic_batch_prepare,
                &mut self.semantic_encode_submit_present,
                &mut self.semantic_draw,
            )
        };
        batch_prepare.record(duration_micros(report.batch_prepare));
        encode_submit_present.record(duration_micros(report.encode_submit_present));
        draw.record(duration_micros(report.draw));
        if let Some(refresh) = self
            .environment
            .as_ref()
            .and_then(|environment| environment.display_refresh_millihertz)
        {
            let misses = missed_refresh_opportunities(report.draw, refresh);
            self.renderer_deadline_misses += misses;
            if property_only {
                self.property_renderer_deadline_misses += misses;
            } else {
                self.semantic_renderer_deadline_misses += misses;
            }
        }

        if !report.presented {
            return;
        }

        self.frames_presented += 1;
        if property_only {
            self.property_frames_presented += 1;
        } else {
            self.semantic_frames_presented += 1;
        }
        if let Some(previous) = self.last_presented_at {
            let interval = report.presented_at.saturating_duration_since(previous);
            self.intervals.record(duration_micros(interval));
            if let Some(refresh) = self
                .environment
                .as_ref()
                .and_then(|environment| environment.display_refresh_millihertz)
            {
                self.missed_refresh_opportunities +=
                    missed_refresh_opportunities(interval, refresh);
            }
        }
        self.last_presented_at = Some(report.presented_at);

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

    pub fn interval_p50_us(&self) -> u128 {
        self.intervals.p50()
    }

    pub fn interval_p99_us(&self) -> u128 {
        self.intervals.p99()
    }

    pub fn interval_max_us(&self) -> u128 {
        self.intervals.max()
    }

    pub fn acquire_wait_p95_us(&self) -> u128 {
        self.acquire_wait.p95()
    }

    pub fn acquire_wait_p50_us(&self) -> u128 {
        self.acquire_wait.p50()
    }

    pub fn acquire_wait_p99_us(&self) -> u128 {
        self.acquire_wait.p99()
    }

    pub fn acquire_wait_max_us(&self) -> u128 {
        self.acquire_wait.max()
    }

    pub fn draw_p95_us(&self) -> u128 {
        self.draw.p95()
    }

    pub fn draw_p50_us(&self) -> u128 {
        self.draw.p50()
    }

    pub fn draw_p99_us(&self) -> u128 {
        self.draw.p99()
    }

    pub fn draw_max_us(&self) -> u128 {
        self.draw.max()
    }

    pub fn batch_prepare_p95_us(&self) -> u128 {
        self.batch_prepare.p95()
    }

    pub fn batch_prepare_p50_us(&self) -> u128 {
        self.batch_prepare.p50()
    }

    pub fn batch_prepare_p99_us(&self) -> u128 {
        self.batch_prepare.p99()
    }

    pub fn batch_prepare_max_us(&self) -> u128 {
        self.batch_prepare.max()
    }

    pub fn encode_submit_present_p95_us(&self) -> u128 {
        self.encode_submit_present.p95()
    }

    pub fn encode_submit_present_p50_us(&self) -> u128 {
        self.encode_submit_present.p50()
    }

    pub fn encode_submit_present_p99_us(&self) -> u128 {
        self.encode_submit_present.p99()
    }

    pub fn encode_submit_present_max_us(&self) -> u128 {
        self.encode_submit_present.max()
    }

    pub fn key_to_present_p95_us(&self) -> u128 {
        self.key_to_present.p95()
    }

    pub fn key_to_present_p50_us(&self) -> u128 {
        self.key_to_present.p50()
    }

    pub fn key_to_present_p99_us(&self) -> u128 {
        self.key_to_present.p99()
    }

    pub fn key_to_present_max_us(&self) -> u128 {
        self.key_to_present.max()
    }

    pub fn frames_skipped(&self) -> usize {
        self.frames_attempted.saturating_sub(self.frames_presented)
    }

    pub fn pending_key_to_present_samples(&self) -> usize {
        self.pending_inputs.len()
    }

    pub fn receipt_text(&self, workload: &str) -> String {
        let captured_unix_ms = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis();
        let mut receipt = String::new();
        let _ = writeln!(receipt, "schema=wgpu_l3.renderer_receipt.v1");
        let _ = writeln!(receipt, "captured_unix_ms={captured_unix_ms}");
        let _ = writeln!(receipt, "workload={workload}");
        if let Some(environment) = &self.environment {
            let _ = writeln!(receipt, "os={}", environment.os);
            let _ = writeln!(receipt, "architecture={}", environment.architecture);
            let _ = writeln!(receipt, "adapter_name={}", environment.adapter_name);
            let _ = writeln!(receipt, "adapter_backend={}", environment.adapter_backend);
            let _ = writeln!(
                receipt,
                "adapter_device_type={}",
                environment.adapter_device_type
            );
            let _ = writeln!(receipt, "adapter_driver={}", environment.adapter_driver);
            let _ = writeln!(
                receipt,
                "adapter_driver_info={}",
                environment.adapter_driver_info
            );
            let _ = writeln!(receipt, "adapter_vendor={}", environment.adapter_vendor);
            let _ = writeln!(receipt, "adapter_device={}", environment.adapter_device);
            let _ = writeln!(
                receipt,
                "fallback_adapter_requested={}",
                environment.fallback_adapter_requested
            );
            let _ = writeln!(
                receipt,
                "fallback_adapter_cpu_class={}",
                environment.fallback_adapter_cpu_class
            );
            let _ = writeln!(
                receipt,
                "fallback_selection_verified={}",
                environment.fallback_selection_verified()
            );
            let _ = writeln!(
                receipt,
                "presentation_system={}",
                environment.presentation_system
            );
            let _ = writeln!(
                receipt,
                "display_name={}",
                environment.display_name.as_deref().unwrap_or("unavailable")
            );
            let _ = writeln!(
                receipt,
                "display_refresh_millihertz={}",
                environment
                    .display_refresh_millihertz
                    .map_or_else(|| "unavailable".to_owned(), |value| value.to_string())
            );
            let _ = writeln!(
                receipt,
                "scale_factor_milli={}",
                environment.scale_factor_milli
            );
            let _ = writeln!(receipt, "surface_width={}", environment.surface_width);
            let _ = writeln!(receipt, "surface_height={}", environment.surface_height);
            let _ = writeln!(receipt, "surface_format={}", environment.surface_format);
            let _ = writeln!(receipt, "alpha_mode={}", environment.alpha_mode);
            let _ = writeln!(receipt, "present_mode={}", environment.present_mode);
            let _ = writeln!(
                receipt,
                "desired_maximum_frame_latency={}",
                environment.desired_maximum_frame_latency
            );
            let _ = writeln!(
                receipt,
                "adapter_info_begin\n{}adapter_info_end",
                environment.adapter_info
            );
        } else {
            let _ = writeln!(receipt, "environment=unavailable");
        }
        let _ = writeln!(receipt, "frames_attempted={}", self.frames_attempted);
        let _ = writeln!(receipt, "frames_presented={}", self.frames_presented);
        let _ = writeln!(receipt, "frames_skipped={}", self.frames_skipped());
        let _ = writeln!(
            receipt,
            "property_frames_attempted={}",
            self.property_frames_attempted
        );
        let _ = writeln!(
            receipt,
            "property_frames_presented={}",
            self.property_frames_presented
        );
        let _ = writeln!(
            receipt,
            "semantic_frames_attempted={}",
            self.semantic_frames_attempted
        );
        let _ = writeln!(
            receipt,
            "semantic_frames_presented={}",
            self.semantic_frames_presented
        );
        let _ = writeln!(
            receipt,
            "missed_refresh_opportunities={}",
            self.missed_refresh_opportunities
        );
        let _ = writeln!(
            receipt,
            "renderer_deadline_misses={}",
            self.renderer_deadline_misses
        );
        let _ = writeln!(
            receipt,
            "property_renderer_deadline_misses={}",
            self.property_renderer_deadline_misses
        );
        let _ = writeln!(
            receipt,
            "semantic_renderer_deadline_misses={}",
            self.semantic_renderer_deadline_misses
        );
        let _ = writeln!(
            receipt,
            "commit_preparation_slices={}",
            self.commit_preparation_slices
        );
        let _ = writeln!(
            receipt,
            "commit_preparation_total_us={}",
            self.commit_preparation_total_nanos / 1_000
        );
        let _ = writeln!(
            receipt,
            "commit_preparation_max_us={}",
            self.commit_preparation_max_nanos / 1_000
        );
        let _ = writeln!(
            receipt,
            "commit_preparation_deadline_misses={}",
            self.commit_preparation_deadline_misses
        );
        let _ = writeln!(
            receipt,
            "semantic_commits_created={}",
            self.semantic_commits_created
        );
        let _ = writeln!(
            receipt,
            "semantic_commits_activated={}",
            self.semantic_commits_activated
        );
        let _ = writeln!(receipt, "scene_nodes_added={}", self.scene_nodes_added);
        let _ = writeln!(receipt, "scene_nodes_removed={}", self.scene_nodes_removed);
        let _ = writeln!(receipt, "scene_nodes_reused={}", self.scene_nodes_reused);
        let _ = writeln!(receipt, "scene_nodes_rebuilt={}", self.scene_nodes_rebuilt);
        let _ = writeln!(receipt, "scene_paint_calls={}", self.scene_paint_calls);
        let _ = writeln!(receipt, "scene_painted_nodes={}", self.scene_painted_nodes);
        let _ = writeln!(receipt, "text_nodes_reused={}", self.text_nodes_reused);
        let _ = writeln!(receipt, "property_ticks={}", self.property_ticks);
        let _ = writeln!(
            receipt,
            "changed_property_values={}",
            self.changed_property_values
        );
        let _ = writeln!(
            receipt,
            "property_upload_bytes={}",
            self.property_upload_bytes
        );
        let _ = writeln!(
            receipt,
            "candidate_property_serial={}",
            self.candidate_property_serial
        );
        let _ = writeln!(
            receipt,
            "sampled_property_serial={}",
            self.sampled_property_serial
        );
        let _ = writeln!(
            receipt,
            "visible_property_serial={}",
            self.visible_property_serial
        );
        let _ = writeln!(
            receipt,
            "skipped_property_attempts={}",
            self.skipped_property_attempts
        );
        let _ = writeln!(
            receipt,
            "virtual_guard_crossings={}",
            self.virtual_guard_crossings
        );
        let _ = writeln!(
            receipt,
            "replenishment_commits={}",
            self.replenishment_commits
        );
        write_distribution(&mut receipt, "frame_interval_us", &self.intervals);
        write_distribution(&mut receipt, "acquire_wait_us", &self.acquire_wait);
        write_distribution(&mut receipt, "batch_prepare_us", &self.batch_prepare);
        write_distribution(&mut receipt, "command_preparation_us", &self.batch_prepare);
        write_distribution(
            &mut receipt,
            "encode_submit_present_us",
            &self.encode_submit_present,
        );
        write_distribution(&mut receipt, "draw_us", &self.draw);
        write_distribution(
            &mut receipt,
            "property_batch_prepare_us",
            &self.property_batch_prepare,
        );
        write_distribution(
            &mut receipt,
            "property_encode_submit_present_us",
            &self.property_encode_submit_present,
        );
        write_distribution(&mut receipt, "property_draw_us", &self.property_draw);
        write_distribution(
            &mut receipt,
            "semantic_batch_prepare_us",
            &self.semantic_batch_prepare,
        );
        write_distribution(
            &mut receipt,
            "semantic_encode_submit_present_us",
            &self.semantic_encode_submit_present,
        );
        write_distribution(&mut receipt, "semantic_draw_us", &self.semantic_draw);
        write_distribution(&mut receipt, "key_to_present_us", &self.key_to_present);
        write_distribution(
            &mut receipt,
            "replenishment_commit_us",
            &self.replenishment_commit,
        );
        let _ = writeln!(receipt, "scene_items_latest={}", self.scene_items);
        let _ = writeln!(receipt, "render_batches_latest={}", self.render_batches);
        let _ = writeln!(receipt, "glyph_batches_latest={}", self.glyph_batches);
        let _ = writeln!(receipt, "text_surfaces_latest={}", self.text_surfaces);
        let _ = writeln!(receipt, "quad_vertices_latest={}", self.quad_vertices);
        let _ = writeln!(
            receipt,
            "scene_node_realization_rebuilds_latest={}",
            self.scene_node_realization_rebuilds
        );
        let _ = writeln!(
            receipt,
            "scene_node_realization_rebuilds_total={}",
            self.scene_node_realization_rebuilds_total
        );
        let _ = writeln!(receipt, "quad_instances_latest={}", self.quad_instances);
        let _ = writeln!(
            receipt,
            "quad_prepare_calls_latest={}",
            self.quad_prepare_calls
        );
        let _ = writeln!(
            receipt,
            "quad_prepare_calls_total={}",
            self.quad_prepare_calls_total
        );
        let _ = writeln!(
            receipt,
            "text_prepare_calls_latest={}",
            self.text_prepare_calls
        );
        let _ = writeln!(
            receipt,
            "text_prepare_calls_total={}",
            self.text_prepare_calls_total
        );
        let _ = writeln!(
            receipt,
            "geometry_upload_bytes_latest={}",
            self.geometry_upload_bytes
        );
        let _ = writeln!(
            receipt,
            "geometry_upload_bytes_total={}",
            self.geometry_upload_bytes_total
        );
        let _ = writeln!(
            receipt,
            "content_upload_bytes_latest={}",
            self.content_upload_bytes
        );
        let _ = writeln!(
            receipt,
            "content_upload_bytes_total={}",
            self.content_upload_bytes_total
        );
        let _ = writeln!(
            receipt,
            "geometry_buffer_creations_latest={}",
            self.geometry_buffer_creations
        );
        let _ = writeln!(
            receipt,
            "geometry_buffer_creations_total={}",
            self.geometry_buffer_creations_total
        );
        let _ = writeln!(
            receipt,
            "retained_gpu_resource_count={}",
            self.retained_gpu_resource_count
        );
        let _ = writeln!(
            receipt,
            "retained_gpu_resource_count_high_water={}",
            self.retained_gpu_resource_count_high_water
        );
        let _ = writeln!(
            receipt,
            "retained_gpu_resource_bytes={}",
            self.retained_gpu_resource_bytes
        );
        let _ = writeln!(
            receipt,
            "retained_gpu_resource_bytes_high_water={}",
            self.retained_gpu_resource_bytes_high_water
        );
        let _ = writeln!(
            receipt,
            "retained_gpu_resource_creations={}",
            self.retained_gpu_resource_creations
        );
        let _ = writeln!(
            receipt,
            "retained_gpu_resource_replacements={}",
            self.retained_gpu_resource_replacements
        );
        let _ = writeln!(
            receipt,
            "retained_gpu_resource_removals={}",
            self.retained_gpu_resource_removals
        );
        let _ = writeln!(receipt, "draw_passes_latest={}", self.draw_passes);
        let _ = writeln!(receipt, "draw_passes_total={}", self.draw_passes_total);
        let _ = writeln!(receipt, "draw_calls_latest={}", self.draw_calls);
        let _ = writeln!(receipt, "draw_calls_total={}", self.draw_calls_total);
        let _ = writeln!(
            receipt,
            "explicit_copy_commands_total={}",
            self.explicit_copy_commands_total
        );
        let _ = writeln!(
            receipt,
            "resource_transition_boundaries_total={}",
            self.resource_transition_boundaries_total
        );
        let _ = writeln!(receipt, "pipeline_changes_latest={}", self.pipeline_changes);
        let _ = writeln!(
            receipt,
            "pipeline_changes_total={}",
            self.pipeline_changes_total
        );
        let _ = writeln!(
            receipt,
            "bind_group_changes_latest={}",
            self.bind_group_changes
        );
        let _ = writeln!(
            receipt,
            "bind_group_changes_total={}",
            self.bind_group_changes_total
        );
        let _ = writeln!(
            receipt,
            "scroll_layer_cache_hits_latest={}",
            self.scroll_layer_cache_hits
        );
        let _ = writeln!(
            receipt,
            "scroll_layer_cache_hits_total={}",
            self.scroll_layer_cache_hits_total
        );
        let _ = writeln!(
            receipt,
            "scroll_layer_cache_misses_latest={}",
            self.scroll_layer_cache_misses
        );
        let _ = writeln!(
            receipt,
            "scroll_layer_cache_misses_total={}",
            self.scroll_layer_cache_misses_total
        );
        let _ = writeln!(
            receipt,
            "render_plan_rebuilds_total={}",
            self.render_plan_rebuilds_total
        );
        let _ = writeln!(
            receipt,
            "render_plan_reuses_total={}",
            self.render_plan_reuses_total
        );
        let _ = writeln!(
            receipt,
            "direct_surface_plans_total={}",
            self.direct_surface_plans_total
        );
        let _ = writeln!(
            receipt,
            "surface_sampling_plans_total={}",
            self.surface_sampling_plans_total
        );
        let _ = writeln!(receipt, "clip_batches_latest={}", self.clip_batches);
        let _ = writeln!(receipt, "opaque_nodes_latest={}", self.opaque_nodes);
        let _ = writeln!(receipt, "blended_nodes_latest={}", self.blended_nodes);
        let _ = writeln!(
            receipt,
            "opacity_unclassified_nodes_latest={}",
            self.opacity_unclassified_nodes
        );
        let _ = writeln!(receipt, "clipped_nodes_latest={}", self.clipped_nodes);
        let _ = writeln!(
            receipt,
            "effect_island_nodes_latest={}",
            self.effect_island_nodes
        );
        let _ = writeln!(
            receipt,
            "effect_intermediate_clears_total={}",
            self.effect_intermediate_clears_total
        );
        let _ = writeln!(
            receipt,
            "effect_intermediate_clear_bytes_total={}",
            self.effect_intermediate_clear_bytes_total
        );
        let _ = writeln!(
            receipt,
            "effect_intermediate_composites_total={}",
            self.effect_intermediate_composites_total
        );
        let _ = writeln!(
            receipt,
            "effect_intermediate_composite_bytes_total={}",
            self.effect_intermediate_composite_bytes_total
        );
        let _ = writeln!(
            receipt,
            "largest_effect_intermediate_bytes_high_water={}",
            self.largest_effect_intermediate_bytes_high_water
        );
        let _ = writeln!(receipt, "culled_nodes_latest={}", self.culled_nodes);
        let _ = writeln!(receipt, "group_composites_latest={}", self.group_composites);
        let _ = writeln!(
            receipt,
            "filter_layer_pool_entries={}",
            self.filter_layer_pool_entries
        );
        let _ = writeln!(
            receipt,
            "filter_scratch_pool_entries={}",
            self.filter_scratch_pool_entries
        );
        let _ = writeln!(
            receipt,
            "filter_layer_pool_entries_high_water={}",
            self.filter_layer_pool_entries_high_water
        );
        let _ = writeln!(
            receipt,
            "filter_scratch_pool_entries_high_water={}",
            self.filter_scratch_pool_entries_high_water
        );
        let _ = writeln!(
            receipt,
            "inline_text_cache_hits_total={}",
            self.inline_text_cache_hits_total
        );
        let _ = writeln!(
            receipt,
            "inline_text_cache_misses_total={}",
            self.inline_text_cache_misses_total
        );
        let _ = writeln!(
            receipt,
            "inline_text_shape_calls_total={}",
            self.inline_text_shape_calls_total
        );
        let _ = writeln!(
            receipt,
            "pending_key_to_present_samples={}",
            self.pending_key_to_present_samples()
        );
        let _ = writeln!(
            receipt,
            "ordinary_surface_clears_total={}",
            self.ordinary_surface_clears_total
        );
        let _ = writeln!(
            receipt,
            "ordinary_surface_clear_bytes_total={}",
            self.ordinary_surface_clear_bytes_total
        );
        let _ = writeln!(
            receipt,
            "extra_full_surface_intermediate_clears_total={}",
            self.extra_full_surface_intermediate_clears_total
        );
        let _ = writeln!(
            receipt,
            "extra_full_surface_intermediate_clear_bytes_total={}",
            self.extra_full_surface_intermediate_clear_bytes_total
        );
        let _ = writeln!(
            receipt,
            "full_surface_blits_total={}",
            self.full_surface_blits_total
        );
        let _ = writeln!(
            receipt,
            "full_surface_blit_bytes_total={}",
            self.full_surface_blit_bytes_total
        );
        let _ = writeln!(
            receipt,
            "popup_surface_packs_total={}",
            self.popup_surface_packs_total
        );
        let _ = writeln!(
            receipt,
            "popup_surface_pack_bytes_total={}",
            self.popup_surface_pack_bytes_total
        );
        let _ = writeln!(receipt, "acquire_successes={}", self.acquire_successes);
        let _ = writeln!(receipt, "acquire_suboptimal={}", self.acquire_suboptimal);
        let _ = writeln!(receipt, "acquire_outdated={}", self.acquire_outdated);
        let _ = writeln!(receipt, "acquire_timeouts={}", self.acquire_timeouts);
        let _ = writeln!(receipt, "acquire_occluded={}", self.acquire_occluded);
        let _ = writeln!(receipt, "acquire_validation={}", self.acquire_validation);
        let _ = writeln!(receipt, "acquire_lost={}", self.acquire_lost);
        receipt
    }
}

impl Default for Render {
    fn default() -> Self {
        Self {
            environment: None,
            semantic_commits_created: 0,
            semantic_commits_activated: 0,
            scene_nodes_added: 0,
            scene_nodes_removed: 0,
            scene_nodes_reused: 0,
            scene_nodes_rebuilt: 0,
            scene_paint_calls: 0,
            scene_painted_nodes: 0,
            text_nodes_reused: 0,
            property_ticks: 0,
            changed_property_values: 0,
            property_upload_bytes: 0,
            candidate_property_serial: 0,
            sampled_property_serial: 0,
            visible_property_serial: 0,
            skipped_property_attempts: 0,
            virtual_guard_crossings: 0,
            replenishment_commits: 0,
            frames_attempted: 0,
            frames_presented: 0,
            property_frames_attempted: 0,
            property_frames_presented: 0,
            semantic_frames_attempted: 0,
            semantic_frames_presented: 0,
            missed_refresh_opportunities: 0,
            renderer_deadline_misses: 0,
            property_renderer_deadline_misses: 0,
            semantic_renderer_deadline_misses: 0,
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
            scene_node_realization_rebuilds: 0,
            scene_node_realization_rebuilds_total: 0,
            quad_prepare_calls: 0,
            quad_prepare_calls_total: 0,
            text_prepare_calls: 0,
            text_prepare_calls_total: 0,
            quad_instances: 0,
            quad_vertices: 0,
            geometry_upload_bytes: 0,
            geometry_upload_bytes_total: 0,
            content_upload_bytes: 0,
            content_upload_bytes_total: 0,
            geometry_buffer_creations: 0,
            geometry_buffer_creations_total: 0,
            retained_gpu_resource_count: 0,
            retained_gpu_resource_count_high_water: 0,
            retained_gpu_resource_bytes: 0,
            retained_gpu_resource_bytes_high_water: 0,
            retained_gpu_resource_creations: 0,
            retained_gpu_resource_replacements: 0,
            retained_gpu_resource_removals: 0,
            commit_preparation_slices: 0,
            commit_preparation_total_nanos: 0,
            commit_preparation_max_nanos: 0,
            commit_preparation_deadline_misses: 0,
            render_plan_rebuilds: 0,
            render_plan_rebuilds_total: 0,
            render_plan_reuses: 0,
            render_plan_reuses_total: 0,
            direct_surface_plans: 0,
            direct_surface_plans_total: 0,
            surface_sampling_plans: 0,
            surface_sampling_plans_total: 0,
            draw_calls: 0,
            draw_calls_total: 0,
            draw_passes: 0,
            draw_passes_total: 0,
            explicit_copy_commands: 0,
            explicit_copy_commands_total: 0,
            resource_transition_boundaries: 0,
            resource_transition_boundaries_total: 0,
            pipeline_changes: 0,
            pipeline_changes_total: 0,
            bind_group_changes: 0,
            bind_group_changes_total: 0,
            scroll_layer_cache_hits: 0,
            scroll_layer_cache_hits_total: 0,
            scroll_layer_cache_misses: 0,
            scroll_layer_cache_misses_total: 0,
            clip_batches: 0,
            opaque_nodes: 0,
            blended_nodes: 0,
            opacity_unclassified_nodes: 0,
            clipped_nodes: 0,
            effect_island_nodes: 0,
            effect_intermediate_clears: 0,
            effect_intermediate_clears_total: 0,
            effect_intermediate_clear_bytes_total: 0,
            effect_intermediate_composites: 0,
            effect_intermediate_composites_total: 0,
            effect_intermediate_composite_bytes_total: 0,
            largest_effect_intermediate_bytes_high_water: 0,
            culled_nodes: 0,
            group_composites: 0,
            filter_layer_pool_entries: 0,
            filter_layer_pool_entries_high_water: 0,
            filter_scratch_pool_entries: 0,
            filter_scratch_pool_entries_high_water: 0,
            ordinary_surface_clears: 0,
            ordinary_surface_clears_total: 0,
            ordinary_surface_clear_bytes_total: 0,
            extra_full_surface_intermediate_clears: 0,
            extra_full_surface_intermediate_clears_total: 0,
            extra_full_surface_intermediate_clear_bytes_total: 0,
            full_surface_blits: 0,
            full_surface_blits_total: 0,
            full_surface_blit_bytes_total: 0,
            popup_surface_packs: 0,
            popup_surface_packs_total: 0,
            popup_surface_pack_bytes_total: 0,
            acquire_successes: 0,
            acquire_suboptimal: 0,
            acquire_outdated: 0,
            acquire_timeouts: 0,
            acquire_occluded: 0,
            acquire_validation: 0,
            acquire_lost: 0,
            intervals: Samples::default(),
            acquire_wait: Samples::default(),
            batch_prepare: Samples::default(),
            encode_submit_present: Samples::default(),
            draw: Samples::default(),
            property_batch_prepare: Samples::default(),
            property_encode_submit_present: Samples::default(),
            property_draw: Samples::default(),
            semantic_batch_prepare: Samples::default(),
            semantic_encode_submit_present: Samples::default(),
            semantic_draw: Samples::default(),
            key_to_present: Samples::default(),
            replenishment_commit: Samples::default(),
            pending_inputs: VecDeque::new(),
            last_presented_at: None,
        }
    }
}

fn duration_micros(duration: Duration) -> u128 {
    duration.as_micros()
}

fn missed_refresh_opportunities(interval: Duration, refresh_millihertz: u32) -> usize {
    if refresh_millihertz == 0 {
        return 0;
    }
    let refresh_period_nanos = 1_000_000_000_000_u128 / u128::from(refresh_millihertz);
    let opportunities = interval.as_nanos().div_ceil(refresh_period_nanos.max(1));
    opportunities.saturating_sub(1) as usize
}

fn write_distribution(receipt: &mut String, name: &str, samples: &Samples) {
    let _ = writeln!(receipt, "{name}_sample_count={}", samples.len());
    let _ = writeln!(receipt, "{name}_p50={}", samples.p50());
    let _ = writeln!(receipt, "{name}_p95={}", samples.p95());
    let _ = writeln!(receipt, "{name}_p99={}", samples.p99());
    let _ = writeln!(receipt, "{name}_max={}", samples.max());
}

#[cfg(feature = "renderer-debug")]
pub async fn compare_control_gallery_property_tick(scale_factor: f32) -> Result<(), String> {
    let mut app = crate::control_gallery::app(crate::control_gallery::State::default());
    app.start();
    let window = app.session().windows()[0].id();
    let size = crate::control_gallery::window_size();
    let initial = app
        .render_scene(window, size)
        .ok_or_else(|| "control gallery did not produce an initial scene".to_owned())?;
    app.finish_render_report(
        window,
        initial.epoch(),
        initial.invalidation(),
        initial.layout(),
        initial.properties(),
        initial.property_only(),
        RenderReport::new(Duration::ZERO, Duration::ZERO, Instant::now()),
    );
    let body = initial
        .layout()
        .frames()
        .iter()
        .find(|frame| frame.table_cell().is_some())
        .ok_or_else(|| "control gallery did not materialize a visible table cell".to_owned())?;
    let rect = body.rect();
    let point = crate::geometry::Point::new(rect.x() + 1, rect.y() + 1);
    let commit = std::sync::Arc::clone(initial.commit());
    let initial_properties = initial.properties().clone();
    let initial_scroll = initial.layout().scroll_projections().to_vec();
    drop(initial);

    let outcome = app
        .scroll_at(
            window,
            size,
            point,
            crate::interaction::ScrollDelta::vertical(8),
        )
        .map_err(|error| error.to_string())?;
    let tick = app
        .render_scene(window, size)
        .ok_or_else(|| "control gallery did not produce a property tick".to_owned())?;
    if !tick.property_only() || !std::sync::Arc::ptr_eq(&commit, tick.commit()) {
        return Err(format!(
            "control gallery scroll escaped the retained property clock: point={point:?}, effect={:?}, property_only={}, invalidation={:?}, initial_scroll={initial_scroll:?}",
            outcome.effect(),
            tick.property_only(),
            tick.invalidation(),
        ));
    }
    let order = commit
        .order()
        .ok_or_else(|| "control gallery commit has no retained draw order".to_owned())?;
    let vertical_owner = order
        .iter()
        .find_map(|draw| match draw {
            crate::scene::Draw::Content {
                node,
                projection:
                    crate::scene::ContentProjection::ScrollbarTrack {
                        axis: crate::interaction::ScrollbarAxis::Vertical,
                        ..
                    }
                    | crate::scene::ContentProjection::ScrollbarThumb {
                        axis: crate::interaction::ScrollbarAxis::Vertical,
                        ..
                    },
                ..
            } => Some(*node),
            _ => None,
        })
        .ok_or_else(|| "control gallery commit has no vertical scrollbar projection".to_owned())?;
    let mut scrollbar_owners = order
        .iter()
        .filter_map(|draw| match draw {
            crate::scene::Draw::Content {
                node,
                projection:
                    crate::scene::ContentProjection::ScrollbarTrack { .. }
                    | crate::scene::ContentProjection::ScrollbarThumb { .. },
                ..
            } => Some(*node),
            _ => None,
        })
        .collect::<Vec<_>>();
    scrollbar_owners.sort_unstable();
    scrollbar_owners.dedup();
    let initial_scroll = initial_properties
        .scroll_offset(vertical_owner)
        .ok_or_else(|| "vertical scrollbar owner has no initial scroll property".to_owned())?;
    let tick_scroll = tick
        .properties()
        .scroll_offset(vertical_owner)
        .ok_or_else(|| "vertical scrollbar owner has no ticked scroll property".to_owned())?;
    if initial_scroll == tick_scroll {
        return Err("vertical scrollbar property did not advance its scroll truth".to_owned());
    }
    for owner in scrollbar_owners {
        if initial_properties.scroll_offset(owner).is_none()
            || tick.properties().scroll_offset(owner).is_none()
            || initial_properties.scrollbar(owner).is_none()
            || tick.properties().scrollbar(owner).is_none()
        {
            return Err(format!(
                "scrollbar owner {owner:?} must consume scroll and chrome values from the same property snapshot"
            ));
        }
    }

    let mut harness = crate::render::debug::Harness::new(scale_factor).await?;
    harness.compare_retained_scroll_transition(&commit, &initial_properties, tick.properties())
}

#[cfg(feature = "renderer-debug")]
pub async fn compare_control_gallery_incremental_activation(
    scale_factor: f32,
) -> Result<(), String> {
    let mut app = crate::control_gallery::app(crate::control_gallery::State::default());
    app.start();
    let window = app.session().windows()[0].id();
    let initial = app
        .render_scene(window, crate::control_gallery::window_size())
        .ok_or_else(|| "control gallery did not produce an initial scene".to_owned())?;
    let commit = std::sync::Arc::clone(initial.commit());
    let properties = initial.properties().clone();
    let mut harness = crate::render::debug::Harness::new(scale_factor).await?;
    harness.compare_incremental_activation(&commit, &properties)
}

#[cfg(feature = "renderer-debug")]
pub async fn compare_control_gallery_pending_transition(scale_factor: f32) -> Result<(), String> {
    let size = crate::control_gallery::window_size();
    let mut app = crate::control_gallery::app(crate::control_gallery::State::default());
    app.start();
    let window = app.session().windows()[0].id();
    let active = app
        .render_scene(window, size)
        .ok_or_else(|| "control gallery did not produce an active scene".to_owned())?;
    let active_commit = std::sync::Arc::clone(active.commit());
    let active_properties = active.properties().clone();
    drop(active);

    let trigger = app
        .trigger::<crate::control_gallery::command::SubmitQuery>("abcdefghijklmnopqrst".to_owned());
    let _response = app.invoke(trigger);
    let candidate = app
        .render_scene(window, size)
        .ok_or_else(|| "control gallery did not produce a changed scene".to_owned())?;

    let mut harness = crate::render::debug::Harness::new(scale_factor).await?;
    harness.compare_pending_transition_preserves_active(
        &active_commit,
        &active_properties,
        candidate.commit(),
        candidate.properties(),
    )
}

#[cfg(feature = "renderer-debug")]
pub async fn compare_control_gallery_pending_property_refresh(
    scale_factor: f32,
) -> Result<(), String> {
    let size = crate::geometry::Size::new(760, 700);
    let mut app = crate::control_gallery::app(crate::control_gallery::State::default());
    app.start();
    let window = app.session().windows()[0].id();
    let active = app
        .render_scene(window, size)
        .ok_or_else(|| "control gallery did not produce an active scene".to_owned())?;
    app.finish_render_report(
        window,
        active.epoch(),
        active.invalidation(),
        active.layout(),
        active.properties(),
        active.property_only(),
        RenderReport::new(Duration::ZERO, Duration::ZERO, Instant::now()),
    );
    let body = active
        .layout()
        .frames()
        .iter()
        .find(|frame| {
            frame.table_cell().is_some_and(|cell| {
                cell.row() == crate::virtual_list::Key::new(1)
                    && cell.column() == crate::interaction::Id::new("detail")
            })
        })
        .ok_or_else(|| "control gallery did not materialize a table cell".to_owned())?;
    let point = crate::geometry::Point::new(body.rect().x() + 1, body.rect().y() + 1);
    let active_commit = std::sync::Arc::clone(active.commit());
    let active_properties = active.properties().clone();
    drop(active);

    let trigger = app.trigger::<crate::control_gallery::command::SetLevel>(73.0);
    let _response = app.invoke(trigger);
    let candidate = app
        .render_scene(window, size)
        .ok_or_else(|| "control gallery did not produce a semantic candidate".to_owned())?;
    let candidate_commit = std::sync::Arc::clone(candidate.commit());
    if std::sync::Arc::ptr_eq(&active_commit, &candidate_commit) {
        return Err("property refresh fixture did not create a semantic candidate".to_owned());
    }
    drop(candidate);

    app.scroll_at(
        window,
        size,
        point,
        crate::interaction::ScrollDelta::vertical(24),
    )
    .map_err(|error| error.to_string())?;
    let ticked_candidate = app
        .render_scene(window, size)
        .ok_or_else(|| "control gallery did not produce a pending scroll sample".to_owned())?;
    if !std::sync::Arc::ptr_eq(&candidate_commit, ticked_candidate.commit()) {
        let equal_nodes = candidate_commit
            .nodes()
            .iter()
            .filter(|node| {
                ticked_candidate.commit().nodes().iter().any(|candidate| {
                    candidate.id() == node.id() && candidate.as_ref() == node.as_ref()
                })
            })
            .count();
        return Err(format!(
            "pending scroll unexpectedly changed candidate structure: nodes {} -> {}, equal={equal_nodes}, order_equal={}, size_equal={}, property_only={}, invalidation={:?}",
            candidate_commit.nodes().len(),
            ticked_candidate.commit().nodes().len(),
            candidate_commit.order() == ticked_candidate.commit().order(),
            candidate_commit.size() == ticked_candidate.commit().size(),
            ticked_candidate.property_only(),
            ticked_candidate.invalidation(),
        ));
    }

    let mut harness = crate::render::debug::Harness::new(scale_factor).await?;
    harness.compare_pending_property_projection(
        &active_commit,
        &active_properties,
        &candidate_commit,
        ticked_candidate.properties(),
    )
}

#[cfg(feature = "renderer-debug")]
pub async fn compare_control_gallery_caret_blink(scale_factor: f32) -> Result<(), String> {
    let size = crate::control_gallery::window_size();
    let mut app = crate::control_gallery::app(crate::control_gallery::State::default());
    app.start();
    let window = app.session().windows()[0].id();
    let initial = app
        .render_scene(window, size)
        .ok_or_else(|| "control gallery did not produce its initial caret scene".to_owned())?;
    let initial_commit = std::sync::Arc::clone(initial.commit());
    let initial_properties = initial.properties().clone();
    app.finish_render_report(
        window,
        initial.epoch(),
        initial.invalidation(),
        initial.layout(),
        initial.properties(),
        initial.property_only(),
        RenderReport::new(Duration::ZERO, Duration::ZERO, Instant::now()),
    );
    drop(initial);

    let focus = crate::session::Focus::text(crate::control_gallery::RENDERER_DEBUG_QUERY_FOCUS);
    app.handle_input(window, crate::input::Input::focus(focus))
        .map_err(|error| error.to_string())?;
    let target = focus
        .text_target()
        .ok_or_else(|| "gallery query focus has no text target".to_owned())?;
    let epoch = app
        .session()
        .interaction(window)
        .and_then(|interaction| interaction.text_input().caret_epoch_for(&target))
        .ok_or_else(|| "gallery query focus did not mint a caret epoch".to_owned())?;

    let visible = app
        .render_scene_at(window, size, epoch)
        .ok_or_else(|| "control gallery did not produce its visible caret scene".to_owned())?;
    let visible_commit = std::sync::Arc::clone(visible.commit());
    let visible_properties = visible.properties().clone();
    let visible_chrome = caret_blink_content_counts(&visible_commit);
    app.finish_render_report(
        window,
        visible.epoch(),
        visible.invalidation(),
        visible.layout(),
        visible.properties(),
        visible.property_only(),
        RenderReport::new(Duration::ZERO, Duration::ZERO, Instant::now()),
    );
    drop(visible);

    let hidden = app
        .render_scene_at(window, size, epoch + Duration::from_millis(500))
        .ok_or_else(|| "control gallery did not produce its hidden caret scene".to_owned())?;
    let hidden_commit = std::sync::Arc::clone(hidden.commit());
    let hidden_properties = hidden.properties().clone();
    let hidden_chrome = caret_blink_content_counts(&hidden_commit);
    app.finish_render_report(
        window,
        hidden.epoch(),
        hidden.invalidation(),
        hidden.layout(),
        hidden.properties(),
        hidden.property_only(),
        RenderReport::new(Duration::ZERO, Duration::ZERO, Instant::now()),
    );
    drop(hidden);

    let visible_again = app
        .render_scene_at(window, size, epoch + Duration::from_millis(1_000))
        .ok_or_else(|| {
            "control gallery did not produce its second visible caret scene".to_owned()
        })?;
    let visible_again_chrome = caret_blink_content_counts(visible_again.commit());

    if visible_chrome.0 != hidden_chrome.0.saturating_add(1)
        || visible_again_chrome.0 != visible_chrome.0
    {
        return Err(format!(
            "caret blink must remove and restore exactly one rule: visible={}, hidden={}, visible_again={}",
            visible_chrome.0, hidden_chrome.0, visible_again_chrome.0,
        ));
    }
    if visible_chrome.1 != hidden_chrome.1 || visible_again_chrome.1 != visible_chrome.1 {
        return Err(format!(
            "caret blink must preserve every focus outline: visible={}, hidden={}, visible_again={}",
            visible_chrome.1, hidden_chrome.1, visible_again_chrome.1,
        ));
    }
    let (changed_nodes, stale_revisions) =
        caret_blink_changed_node_revisions(&visible_commit, &hidden_commit);
    if changed_nodes != 1 || stale_revisions != 0 {
        return Err(format!(
            "caret blink must advance the one changed node revision: changed_nodes={changed_nodes}, stale_revisions={stale_revisions}",
        ));
    }

    let mut harness = crate::render::debug::Harness::new(scale_factor).await?;
    harness.compare_pending_transition_preserves_active(
        &initial_commit,
        &initial_properties,
        &visible_commit,
        &visible_properties,
    )?;
    harness.compare_pending_transition_preserves_active(
        &visible_commit,
        &visible_properties,
        &hidden_commit,
        &hidden_properties,
    )?;
    harness.compare_pending_transition_preserves_active(
        &hidden_commit,
        &hidden_properties,
        visible_again.commit(),
        visible_again.properties(),
    )
}

#[cfg(feature = "renderer-debug")]
fn caret_blink_content_counts(commit: &crate::scene::Commit) -> (usize, usize) {
    commit.nodes().iter().flat_map(|node| node.content()).fold(
        (0_usize, 0_usize),
        |(rules, outlines), content| match content {
            crate::scene::Content::Rule(_) => (rules.saturating_add(1), outlines),
            crate::scene::Content::Outline(_) => (rules, outlines.saturating_add(1)),
            _ => (rules, outlines),
        },
    )
}

#[cfg(feature = "renderer-debug")]
fn caret_blink_changed_node_revisions(
    before: &crate::scene::Commit,
    after: &crate::scene::Commit,
) -> (usize, usize) {
    let mut changed = 0_usize;
    let mut stale = 0_usize;
    for before in before.nodes() {
        let Some(after) = after.nodes().iter().find(|after| after.id() == before.id()) else {
            continue;
        };
        if before.content() != after.content() {
            changed = changed.saturating_add(1);
            stale = stale.saturating_add(usize::from(
                before.content_revision() == after.content_revision(),
            ));
        }
    }
    (changed, stale)
}

#[cfg(feature = "renderer-debug")]
pub async fn compare_control_gallery_pending_scroll(scale_factor: f32) -> Result<(), String> {
    let mut app = crate::control_gallery::app(crate::control_gallery::State::default());
    app.start();
    let window = app.session().windows()[0].id();
    let size = crate::control_gallery::window_size();
    let initial = app
        .render_scene(window, size)
        .ok_or_else(|| "control gallery did not produce an initial scene".to_owned())?;
    app.finish_render_report(
        window,
        initial.epoch(),
        initial.invalidation(),
        initial.layout(),
        initial.properties(),
        initial.property_only(),
        RenderReport::new(Duration::ZERO, Duration::ZERO, Instant::now()),
    );
    let body = initial
        .layout()
        .frames()
        .iter()
        .find(|frame| frame.table_cell().is_some())
        .ok_or_else(|| "control gallery did not materialize a table cell".to_owned())?;
    let point = crate::geometry::Point::new(body.rect().x() + 1, body.rect().y() + 1);
    let active_commit = std::sync::Arc::clone(initial.commit());
    let active_properties = initial.properties().clone();
    drop(initial);

    let mut harness = crate::render::debug::Harness::new(scale_factor).await?;
    harness.require_visible_commit(&active_commit, &active_properties, "initial active commit")?;
    let mut semantic_candidates = 0_usize;
    let mut pending_transition_witnessed = false;

    for index in 1..=6 {
        app.scroll_at(
            window,
            size,
            point,
            crate::interaction::ScrollDelta::vertical(33_600),
        )
        .map_err(|error| error.to_string())?;
        let candidate = app
            .render_scene(window, size)
            .ok_or_else(|| format!("control gallery produced no scroll candidate {index}"))?;
        for projection in candidate.layout().scroll_projections() {
            let visible = projection.viewport().visible_content();
            let resident = projection.resident_bounds();
            let layer = projection.layer_bounds();
            if resident.x() < layer.x()
                || resident.y() < layer.y()
                || resident.right() > layer.right()
                || resident.bottom() > layer.bottom()
                || resident.x() > visible.x()
                || resident.y() > visible.y()
                || resident.right() < visible.right()
                || resident.bottom() < visible.bottom()
            {
                return Err(format!(
                    "scroll candidate {index} admits nonresident pixels: visible={visible:?}, resident={resident:?}, layer={layer:?}"
                ));
            }
        }

        if std::sync::Arc::ptr_eq(&active_commit, candidate.commit()) {
            harness.compare_incremental_activation(candidate.commit(), candidate.properties())?;
            harness.require_visible_commit(
                &active_commit,
                candidate.properties(),
                &format!("property-only scroll candidate {index}"),
            )?;
        } else {
            semantic_candidates = semantic_candidates.saturating_add(1);
            let (projected, changed) = candidate
                .properties()
                .project_onto(&active_commit, &active_properties)
                .map_err(|error| error.to_string())?;
            if changed
                && projected
                    .changed()
                    .iter()
                    .any(|property| property.kind() != crate::scene::PropertyKind::Scrollbar)
            {
                let details = projected
                    .changed()
                    .iter()
                    .map(|property| {
                        let declaration = active_commit
                            .nodes()
                            .iter()
                            .find(|node| node.id() == property.node())
                            .and_then(|node| node.scroll());
                        format!(
                            "{property:?}: active={:?}, candidate={:?}, projected={:?}, declaration={declaration:?}",
                            active_properties.value(*property),
                            candidate.properties().value(*property),
                            projected.value(*property),
                        )
                    })
                    .collect::<Vec<_>>();
                return Err(format!(
                    "semantic guard-crossing candidate {index} projected content motion beyond the complete active scroll envelope: {}",
                    details.join("; "),
                ));
            }
            if !pending_transition_witnessed {
                harness.compare_scroll_candidate_preserves_stable_prefix(
                    &active_commit,
                    &active_properties,
                    candidate.commit(),
                    candidate.properties(),
                )?;
                harness.compare_pending_transition_preserves_active(
                    &active_commit,
                    &projected,
                    candidate.commit(),
                    candidate.properties(),
                )?;
                pending_transition_witnessed = true;
            } else {
                harness
                    .compare_incremental_activation(candidate.commit(), candidate.properties())?;
            }
            harness.require_visible_commit(
                &active_commit,
                &projected,
                &format!("exact active state for semantic scroll candidate {index}"),
            )?;
        }
    }

    if semantic_candidates == 0 {
        return Err("large-scroll witness never crossed the retained semantic guard".to_owned());
    }
    if !pending_transition_witnessed {
        return Err("large-scroll witness never exercised pending resource isolation".to_owned());
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use std::time::{Duration, Instant};

    use crate::render::RenderReport;
    use crate::window::PresentationEpoch;

    use super::{Render, SAMPLE_LIMIT, Samples, missed_refresh_opportunities};

    #[test]
    fn samples_are_capped_and_report_percentiles_and_max() {
        let mut samples = Samples::default();
        for value in 0..(SAMPLE_LIMIT + 10) {
            samples.record(value as u128);
        }

        assert_eq!(samples.len(), SAMPLE_LIMIT);
        assert_eq!(samples.p50(), 73);
        assert_eq!(samples.p95(), 131);
        assert_eq!(samples.p99(), 136);
        assert_eq!(samples.max(), 137);
    }

    #[test]
    fn missed_refreshes_are_counted_relative_to_reported_display_cadence() {
        assert_eq!(
            missed_refresh_opportunities(Duration::from_micros(16_000), 60_000),
            0
        );
        assert_eq!(
            missed_refresh_opportunities(Duration::from_micros(17_000), 60_000),
            1
        );
        assert_eq!(
            missed_refresh_opportunities(Duration::from_micros(34_000), 60_000),
            2
        );
        assert_eq!(missed_refresh_opportunities(Duration::from_secs(1), 0), 0);
    }

    #[test]
    fn renderer_receipt_is_local_text_with_workload_and_distribution_fields() {
        let receipt = Render::default().receipt_text("control-gallery-500px-idle");

        for field in [
            "schema=wgpu_l3.renderer_receipt.v1",
            "workload=control-gallery-500px-idle",
            "environment=unavailable",
            "frame_interval_us_p50=0",
            "frame_interval_us_p95=0",
            "frame_interval_us_p99=0",
            "frame_interval_us_max=0",
            "command_preparation_us_p95=0",
            "property_draw_us_p95=0",
            "semantic_draw_us_p95=0",
            "replenishment_commit_us_p95=0",
            "geometry_upload_bytes_total=0",
            "content_upload_bytes_total=0",
            "property_upload_bytes=0",
            "retained_gpu_resource_count_high_water=0",
            "opacity_unclassified_nodes_latest=0",
        ] {
            assert!(receipt.contains(field), "missing receipt field {field}");
        }
    }

    #[test]
    fn replenishment_counter_and_timing_share_one_recording_boundary() {
        let mut render = Render::default();

        render.record_replenishment_commit(Duration::from_micros(250));

        assert_eq!(render.virtual_guard_crossings, 1);
        assert_eq!(render.replenishment_commits, 1);
        assert!(
            render
                .receipt_text("guard-crossing")
                .contains("replenishment_commit_us_p95=250")
        );
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
            false,
            RenderReport::new(
                Duration::from_micros(5),
                Duration::from_micros(10),
                now + Duration::from_millis(1),
            ),
        );
        assert_eq!(render.pending_key_to_present_samples(), 1);
        assert_eq!(render.key_to_present_p95_us(), 0);

        render.record_present(
            changed,
            false,
            RenderReport::new(
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
            false,
            RenderReport::new(
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
            true,
            RenderReport::new(
                Duration::from_micros(5),
                Duration::from_micros(10),
                now + Duration::from_millis(1),
            )
            .with_presented(false),
        );

        assert_eq!(render.frames_attempted, 1);
        assert_eq!(render.frames_presented, 0);
        assert_eq!(render.property_frames_attempted, 1);
        assert_eq!(render.property_frames_presented, 0);
        assert_eq!(render.pending_key_to_present_samples(), 1);
        assert_eq!(render.key_to_present_p95_us(), 0);
    }

    #[test]
    fn property_and_semantic_pipeline_timings_stay_disjoint() {
        let mut render = Render::default();
        let epoch = PresentationEpoch::initial();

        render.record_present(
            epoch,
            true,
            RenderReport::new(
                Duration::from_micros(1),
                Duration::from_micros(3),
                Instant::now(),
            )
            .with_pipeline_timings(Duration::from_micros(1), Duration::from_micros(2)),
        );
        render.record_present(
            epoch.next(),
            false,
            RenderReport::new(
                Duration::from_micros(1),
                Duration::from_micros(7),
                Instant::now(),
            )
            .with_pipeline_timings(Duration::from_micros(4), Duration::from_micros(3)),
        );

        let receipt = render.receipt_text("split-timing");
        for field in [
            "property_frames_presented=1",
            "semantic_frames_presented=1",
            "property_batch_prepare_us_p95=1",
            "property_encode_submit_present_us_p95=2",
            "property_draw_us_p95=3",
            "semantic_batch_prepare_us_p95=4",
            "semantic_encode_submit_present_us_p95=3",
            "semantic_draw_us_p95=7",
        ] {
            assert!(receipt.contains(field), "missing split field {field}");
        }
    }
}
