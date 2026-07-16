use std::time::{Duration, Instant};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RendererEnvironment {
    pub os: String,
    pub architecture: String,
    pub adapter_info: String,
    pub adapter_name: String,
    pub adapter_backend: String,
    pub adapter_device_type: String,
    pub adapter_driver: String,
    pub adapter_driver_info: String,
    pub adapter_vendor: u32,
    pub adapter_device: u32,
    pub fallback_adapter_requested: bool,
    pub fallback_adapter_cpu_class: bool,
    pub presentation_system: String,
    pub display_name: Option<String>,
    pub display_refresh_millihertz: Option<u32>,
    pub scale_factor_milli: u32,
    pub surface_width: u32,
    pub surface_height: u32,
    pub surface_format: String,
    pub alpha_mode: String,
    pub present_mode: String,
    pub desired_maximum_frame_latency: u32,
}

impl RendererEnvironment {
    pub(crate) fn new(
        context: &super::Context,
        canvas: &super::Canvas,
        display_name: Option<String>,
        display_refresh_millihertz: Option<u32>,
    ) -> Self {
        let adapter = context.adapter_info();
        let surface = canvas.surface().config();
        Self {
            os: std::env::consts::OS.to_owned(),
            architecture: std::env::consts::ARCH.to_owned(),
            adapter_info: format!("{adapter:#?}"),
            adapter_name: adapter.name,
            adapter_backend: format!("{:?}", adapter.backend),
            adapter_device_type: format!("{:?}", adapter.device_type),
            adapter_driver: adapter.driver,
            adapter_driver_info: adapter.driver_info,
            adapter_vendor: adapter.vendor,
            adapter_device: adapter.device,
            fallback_adapter_requested: context.force_fallback_adapter(),
            fallback_adapter_cpu_class: adapter.device_type == wgpu::DeviceType::Cpu,
            presentation_system: context.presentation_system().to_owned(),
            display_name,
            display_refresh_millihertz,
            scale_factor_milli: (canvas.scale_factor() * 1_000.0).round().max(0.0) as u32,
            surface_width: surface.width,
            surface_height: surface.height,
            surface_format: format!("{:?}", surface.format),
            alpha_mode: format!("{:?}", surface.alpha_mode),
            present_mode: format!("{:?}", surface.present_mode),
            desired_maximum_frame_latency: surface.desired_maximum_frame_latency,
        }
    }

    pub fn fallback_selection_verified(&self) -> bool {
        !self.fallback_adapter_requested || self.fallback_adapter_cpu_class
    }
}

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
    pub(crate) scene_node_realization_rebuilds: usize,
    pub(crate) quad_prepare_calls: usize,
    pub(crate) quad_instances: usize,
    pub(crate) text_prepare_calls: usize,
    pub(crate) quad_vertices: usize,
    pub(crate) geometry_upload_bytes: usize,
    pub(crate) content_upload_bytes: usize,
    pub(crate) property_upload_bytes: usize,
    pub(crate) viewport_property_upload_bytes: usize,
    pub(crate) node_property_upload_bytes: usize,
    pub(crate) scroll_property_upload_bytes: usize,
    pub(crate) text_property_upload_bytes: usize,
    pub(crate) property_value_visits: usize,
    pub(crate) property_index_lookups: usize,
    pub(crate) property_dirty_indices: usize,
    pub(crate) property_write_ranges: usize,
    pub(crate) property_full_initializations: usize,
    pub(crate) property_full_buffer_replacements: usize,
    pub(crate) property_full_topology_replacements: usize,
    pub(crate) property_full_dense_transfers: usize,
    pub(crate) property_full_generation_resyncs: usize,
    pub(crate) geometry_buffer_creations: usize,
    pub(crate) retained_gpu_resource_count: usize,
    pub(crate) retained_gpu_resource_bytes: usize,
    pub(crate) retained_gpu_resource_creations: usize,
    pub(crate) retained_gpu_resource_replacements: usize,
    pub(crate) retained_gpu_resource_removals: usize,
    pub(crate) commit_preparation_slices: usize,
    pub(crate) commit_preparation_total_nanos: u64,
    pub(crate) commit_preparation_max_nanos: u64,
    pub(crate) commit_preparation_deadline_misses: usize,
    pub(crate) render_plan_rebuilds: usize,
    pub(crate) render_plan_reuses: usize,
    pub(crate) direct_surface_plans: usize,
    pub(crate) surface_sampling_plans: usize,
    pub(crate) draw_calls: usize,
    pub(crate) draw_passes: usize,
    pub(crate) explicit_copy_commands: usize,
    pub(crate) resource_transition_boundaries: usize,
    pub(crate) pipeline_changes: usize,
    pub(crate) bind_group_changes: usize,
    pub(crate) scroll_layer_cache_hits: usize,
    pub(crate) scroll_layer_cache_misses: usize,
    pub(crate) clip_batches: usize,
    pub(crate) opaque_nodes: usize,
    pub(crate) blended_nodes: usize,
    pub(crate) opacity_unclassified_nodes: usize,
    pub(crate) clipped_nodes: usize,
    pub(crate) effect_island_nodes: usize,
    pub(crate) effect_intermediate_clears: usize,
    pub(crate) effect_intermediate_clear_bytes: u64,
    pub(crate) effect_intermediate_composites: usize,
    pub(crate) effect_intermediate_composite_bytes: u64,
    pub(crate) largest_effect_intermediate_bytes: u64,
    pub(crate) culled_nodes: usize,
    pub(crate) group_composites: usize,
    pub(crate) filter_layer_pool_entries: usize,
    pub(crate) filter_scratch_pool_entries: usize,
    pub(crate) ordinary_surface_clears: usize,
    pub(crate) ordinary_surface_clear_bytes: u64,
    pub(crate) extra_full_surface_intermediate_clears: usize,
    pub(crate) extra_full_surface_intermediate_clear_bytes: u64,
    pub(crate) full_surface_blits: usize,
    pub(crate) full_surface_blit_bytes: u64,
    pub(crate) popup_surface_packs: usize,
    pub(crate) popup_surface_pack_bytes: u64,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub(crate) struct FrameTimeline {
    acquire_started_at: Option<Instant>,
    acquire_finished_at: Option<Instant>,
    queue_submitted_at: Option<Instant>,
    surface_present_called_at: Option<Instant>,
}

impl FrameTimeline {
    pub(crate) fn new(
        acquire_started_at: Instant,
        acquire_finished_at: Instant,
        queue_submitted_at: Option<Instant>,
        surface_present_called_at: Option<Instant>,
    ) -> Self {
        Self {
            acquire_started_at: Some(acquire_started_at),
            acquire_finished_at: Some(acquire_finished_at),
            queue_submitted_at,
            surface_present_called_at,
        }
    }

    pub(crate) fn acquire_started_at(self) -> Option<Instant> {
        self.acquire_started_at
    }

    pub(crate) fn acquire_finished_at(self) -> Option<Instant> {
        self.acquire_finished_at
    }

    pub(crate) fn queue_submitted_at(self) -> Option<Instant> {
        self.queue_submitted_at
    }

    pub(crate) fn surface_present_called_at(self) -> Option<Instant> {
        self.surface_present_called_at
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RenderReport {
    pub(crate) acquire_wait: Duration,
    pub(crate) draw: Duration,
    pub(crate) batch_prepare: Duration,
    pub(crate) encode_submit_present: Duration,
    pub(crate) draw_stats: DrawStats,
    pub(crate) present_submitted: bool,
    pub(crate) present_submitted_at: Instant,
    pub(crate) frame_timeline: FrameTimeline,
    pub(crate) group_composites: usize,
    pub(crate) filter_layer_pool_entries: usize,
    pub(crate) filter_scratch_pool_entries: usize,
    pub(crate) environment: Option<RendererEnvironment>,
    pub(crate) acquire_outcome: super::surface::AcquireOutcome,
}

impl RenderReport {
    pub fn new(acquire_wait: Duration, draw: Duration, present_submitted_at: Instant) -> Self {
        Self {
            acquire_wait,
            draw,
            batch_prepare: Duration::ZERO,
            encode_submit_present: Duration::ZERO,
            draw_stats: DrawStats::default(),
            present_submitted: true,
            present_submitted_at,
            frame_timeline: FrameTimeline::default(),
            group_composites: 0,
            filter_layer_pool_entries: 0,
            filter_scratch_pool_entries: 0,
            environment: None,
            acquire_outcome: super::surface::AcquireOutcome::Success,
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

    pub(crate) fn with_environment(mut self, environment: RendererEnvironment) -> Self {
        self.environment = Some(environment);
        self
    }

    pub(crate) fn with_acquire_outcome(
        mut self,
        acquire_outcome: super::surface::AcquireOutcome,
    ) -> Self {
        self.acquire_outcome = acquire_outcome;
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

    pub(crate) fn with_present_submitted(mut self, present_submitted: bool) -> Self {
        self.present_submitted = present_submitted;
        self
    }

    pub(crate) fn with_frame_timeline(mut self, frame_timeline: FrameTimeline) -> Self {
        self.frame_timeline = frame_timeline;
        self
    }

    pub fn acquire_wait(&self) -> Duration {
        self.acquire_wait
    }

    pub fn draw(&self) -> Duration {
        self.draw
    }

    pub fn present_submitted_at(&self) -> Instant {
        self.present_submitted_at
    }

    pub(crate) fn present_submitted(&self) -> bool {
        self.present_submitted
    }

    pub(crate) fn frame_timeline(&self) -> FrameTimeline {
        self.frame_timeline
    }
}
