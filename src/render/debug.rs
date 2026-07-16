//! Development-only capability boundary for renderer witnesses.

use std::time::{Duration, Instant};

use crate::{render, scene};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Case {
    Empty,
    SolidQuad,
    GradientQuad,
    TransformedQuad,
    Rule,
    Text,
    TextViewport,
    Icon,
    Shadow,
    Outline,
    SolidPane,
    RoundedClip,
    NestedClip,
    Scroll,
    GroupOpacity,
    OrderedGroup,
    GlassPane,
    TransparentPopup,
}

impl Case {
    pub const ALL: [Self; 18] = [
        Self::Empty,
        Self::SolidQuad,
        Self::GradientQuad,
        Self::TransformedQuad,
        Self::Rule,
        Self::Text,
        Self::TextViewport,
        Self::Icon,
        Self::Shadow,
        Self::Outline,
        Self::SolidPane,
        Self::RoundedClip,
        Self::NestedClip,
        Self::Scroll,
        Self::GroupOpacity,
        Self::OrderedGroup,
        Self::GlassPane,
        Self::TransparentPopup,
    ];

    pub const fn name(self) -> &'static str {
        match self {
            Self::Empty => "empty",
            Self::SolidQuad => "solid-quad",
            Self::GradientQuad => "gradient-quad",
            Self::TransformedQuad => "transformed-quad",
            Self::Rule => "rule",
            Self::Text => "text",
            Self::TextViewport => "text-viewport",
            Self::Icon => "icon",
            Self::Shadow => "shadow",
            Self::Outline => "outline",
            Self::SolidPane => "solid-pane",
            Self::RoundedClip => "rounded-clip",
            Self::NestedClip => "nested-clip",
            Self::Scroll => "scroll",
            Self::GroupOpacity => "group-opacity",
            Self::OrderedGroup => "ordered-group",
            Self::GlassPane => "glass-pane",
            Self::TransparentPopup => "transparent-popup",
        }
    }

    pub fn from_name(value: &str) -> Option<Self> {
        Self::ALL.into_iter().find(|case| case.name() == value)
    }

    pub const fn expects_visible_pixels(self) -> bool {
        !matches!(self, Self::Empty)
    }

    const fn fixture(self) -> scene::FixtureCase {
        match self {
            Self::Empty => scene::FixtureCase::Empty,
            Self::SolidQuad => scene::FixtureCase::SolidQuad,
            Self::GradientQuad => scene::FixtureCase::GradientQuad,
            Self::TransformedQuad => scene::FixtureCase::TransformedQuad,
            Self::Rule => scene::FixtureCase::Rule,
            Self::Text => scene::FixtureCase::Text,
            Self::TextViewport => scene::FixtureCase::TextViewport,
            Self::Icon => scene::FixtureCase::Icon,
            Self::Shadow => scene::FixtureCase::Shadow,
            Self::Outline => scene::FixtureCase::Outline,
            Self::SolidPane => scene::FixtureCase::SolidPane,
            Self::RoundedClip => scene::FixtureCase::RoundedClip,
            Self::NestedClip => scene::FixtureCase::NestedClip,
            Self::Scroll => scene::FixtureCase::Scroll,
            Self::GroupOpacity => scene::FixtureCase::GroupOpacity,
            Self::OrderedGroup => scene::FixtureCase::OrderedGroup,
            Self::GlassPane => scene::FixtureCase::GlassPane,
            Self::TransparentPopup => scene::FixtureCase::TransparentPopup,
        }
    }

    const fn uses_popup_pack(self) -> bool {
        matches!(self, Self::TransparentPopup)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Image {
    width: u32,
    height: u32,
    pixels: Vec<[f32; 4]>,
}

impl Image {
    pub fn new(width: u32, height: u32, pixels: Vec<[f32; 4]>) -> Result<Self, String> {
        if pixels.len() != (width as usize).saturating_mul(height as usize) {
            return Err("pixel count does not match image dimensions".to_owned());
        }
        Ok(Self {
            width,
            height,
            pixels,
        })
    }

    pub fn width(&self) -> u32 {
        self.width
    }

    pub fn height(&self) -> u32 {
        self.height
    }

    pub fn pixels(&self) -> &[[f32; 4]] {
        &self.pixels
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Environment {
    adapter_name: String,
    backend: String,
    device_type: String,
    os: &'static str,
    architecture: &'static str,
}

impl Environment {
    pub fn adapter_name(&self) -> &str {
        &self.adapter_name
    }

    pub fn backend(&self) -> &str {
        &self.backend
    }

    pub fn device_type(&self) -> &str {
        &self.device_type
    }

    pub fn os(&self) -> &str {
        self.os
    }

    pub fn architecture(&self) -> &str {
        self.architecture
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Sample {
    elapsed: Duration,
}

impl Sample {
    pub fn elapsed(self) -> Duration {
        self.elapsed
    }
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct Work {
    scene_node_realization_rebuilds: usize,
    primitive_prepare_calls: usize,
    text_prepare_calls: usize,
    text_shape_calls: usize,
    content_upload_bytes: usize,
    property_upload_bytes: usize,
    gpu_resource_count: usize,
    gpu_resource_bytes: usize,
    gpu_resource_creations: usize,
    gpu_resource_replacements: usize,
    gpu_resource_removals: usize,
    commit_preparation_slices: usize,
    commit_preparation_max_nanos: u64,
    commit_preparation_deadline_misses: usize,
    render_plan_rebuilds: usize,
    render_plan_reuses: usize,
    direct_surface_plans: usize,
    surface_sampling_plans: usize,
    draw_calls: usize,
    scroll_layer_cache_hits: usize,
    scroll_layer_cache_misses: usize,
    draw_passes: usize,
    explicit_copy_commands: usize,
    resource_transition_boundaries: usize,
    opaque_nodes: usize,
    blended_nodes: usize,
    opacity_unclassified_nodes: usize,
    effect_intermediate_clears: usize,
    effect_intermediate_clear_bytes: u64,
    effect_intermediate_composites: usize,
    effect_intermediate_composite_bytes: u64,
    largest_effect_intermediate_bytes: u64,
    target_bytes: u64,
}

impl Work {
    pub fn scene_node_realization_rebuilds(self) -> usize {
        self.scene_node_realization_rebuilds
    }

    pub fn primitive_prepare_calls(self) -> usize {
        self.primitive_prepare_calls
    }

    pub fn text_prepare_calls(self) -> usize {
        self.text_prepare_calls
    }

    pub fn text_shape_calls(self) -> usize {
        self.text_shape_calls
    }

    pub fn content_upload_bytes(self) -> usize {
        self.content_upload_bytes
    }

    pub fn property_upload_bytes(self) -> usize {
        self.property_upload_bytes
    }

    pub fn gpu_resource_count(self) -> usize {
        self.gpu_resource_count
    }

    pub fn gpu_resource_bytes(self) -> usize {
        self.gpu_resource_bytes
    }

    pub fn gpu_resource_creations(self) -> usize {
        self.gpu_resource_creations
    }

    pub fn gpu_resource_replacements(self) -> usize {
        self.gpu_resource_replacements
    }

    pub fn gpu_resource_removals(self) -> usize {
        self.gpu_resource_removals
    }

    pub fn commit_preparation_slices(self) -> usize {
        self.commit_preparation_slices
    }

    pub fn commit_preparation_max_nanos(self) -> u64 {
        self.commit_preparation_max_nanos
    }

    pub fn commit_preparation_deadline_misses(self) -> usize {
        self.commit_preparation_deadline_misses
    }

    pub fn render_plan_rebuilds(self) -> usize {
        self.render_plan_rebuilds
    }

    pub fn render_plan_reuses(self) -> usize {
        self.render_plan_reuses
    }

    pub fn direct_surface_plans(self) -> usize {
        self.direct_surface_plans
    }

    pub fn surface_sampling_plans(self) -> usize {
        self.surface_sampling_plans
    }

    pub fn draw_calls(self) -> usize {
        self.draw_calls
    }

    pub fn scroll_layer_cache_hits(self) -> usize {
        self.scroll_layer_cache_hits
    }

    pub fn scroll_layer_cache_misses(self) -> usize {
        self.scroll_layer_cache_misses
    }

    pub fn draw_passes(self) -> usize {
        self.draw_passes
    }

    pub fn explicit_copy_commands(self) -> usize {
        self.explicit_copy_commands
    }

    pub fn resource_transition_boundaries(self) -> usize {
        self.resource_transition_boundaries
    }

    pub fn opaque_nodes(self) -> usize {
        self.opaque_nodes
    }

    pub fn blended_nodes(self) -> usize {
        self.blended_nodes
    }

    pub fn opacity_unclassified_nodes(self) -> usize {
        self.opacity_unclassified_nodes
    }

    pub fn effect_intermediate_clears(self) -> usize {
        self.effect_intermediate_clears
    }

    pub fn effect_intermediate_clear_bytes(self) -> u64 {
        self.effect_intermediate_clear_bytes
    }

    pub fn effect_intermediate_composites(self) -> usize {
        self.effect_intermediate_composites
    }

    pub fn effect_intermediate_composite_bytes(self) -> u64 {
        self.effect_intermediate_composite_bytes
    }

    pub fn largest_effect_intermediate_bytes(self) -> u64 {
        self.largest_effect_intermediate_bytes
    }

    pub fn target_bytes(self) -> u64 {
        self.target_bytes
    }
}

impl From<render::DrawStats> for Work {
    fn from(stats: render::DrawStats) -> Self {
        Self {
            scene_node_realization_rebuilds: stats.scene_node_realization_rebuilds,
            primitive_prepare_calls: stats.quad_prepare_calls,
            text_prepare_calls: stats.text_prepare_calls,
            text_shape_calls: stats.inline_text_shape_calls + stats.inline_icon_shape_calls,
            content_upload_bytes: stats.content_upload_bytes,
            property_upload_bytes: stats.property_upload_bytes,
            gpu_resource_count: stats.retained_gpu_resource_count,
            gpu_resource_bytes: stats.retained_gpu_resource_bytes,
            gpu_resource_creations: stats.retained_gpu_resource_creations,
            gpu_resource_replacements: stats.retained_gpu_resource_replacements,
            gpu_resource_removals: stats.retained_gpu_resource_removals,
            commit_preparation_slices: stats.commit_preparation_slices,
            commit_preparation_max_nanos: stats.commit_preparation_max_nanos,
            commit_preparation_deadline_misses: stats.commit_preparation_deadline_misses,
            render_plan_rebuilds: stats.render_plan_rebuilds,
            render_plan_reuses: stats.render_plan_reuses,
            direct_surface_plans: stats.direct_surface_plans,
            surface_sampling_plans: stats.surface_sampling_plans,
            draw_calls: stats.draw_calls,
            scroll_layer_cache_hits: stats.scroll_layer_cache_hits,
            scroll_layer_cache_misses: stats.scroll_layer_cache_misses,
            draw_passes: stats.draw_passes,
            explicit_copy_commands: stats.explicit_copy_commands,
            resource_transition_boundaries: stats.resource_transition_boundaries,
            opaque_nodes: stats.opaque_nodes,
            blended_nodes: stats.blended_nodes,
            opacity_unclassified_nodes: stats.opacity_unclassified_nodes,
            effect_intermediate_clears: stats.effect_intermediate_clears,
            effect_intermediate_clear_bytes: stats.effect_intermediate_clear_bytes,
            effect_intermediate_composites: stats.effect_intermediate_composites,
            effect_intermediate_composite_bytes: stats.effect_intermediate_composite_bytes,
            largest_effect_intermediate_bytes: stats.largest_effect_intermediate_bytes,
            target_bytes: 0,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RetentionReceipt {
    first: Work,
    unchanged: Work,
    recreated: Work,
    retired: Work,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PartialUpdateReceipt {
    first: Work,
    changed: Work,
    surviving: Work,
    retired: Work,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ScrollTickReceipt {
    initial: Work,
    tick: Work,
    unchanged: Work,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct UnrelatedSemanticReceipt {
    initial: Work,
    changed: Work,
    unchanged: Work,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TextAtlasRetentionReceipt {
    pressure: Work,
    surviving: Work,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PendingActiveReceipt {
    preparation_slices: usize,
    active_draws: usize,
    peak_pending_states: usize,
    peak_resources: usize,
    peak_bytes: usize,
    activated: Work,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IncrementalActivationReceipt {
    environment: Environment,
    slices: usize,
    batch_prepare: Duration,
    activated: Work,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ChurnReceipt {
    iterations: usize,
    peak_resources: usize,
    peak_bytes: usize,
    post_warm_min_resources: usize,
    post_warm_max_resources: usize,
    post_warm_min_bytes: usize,
    post_warm_max_bytes: usize,
    settled: Work,
}

impl ChurnReceipt {
    pub fn iterations(self) -> usize {
        self.iterations
    }

    pub fn peak_resources(self) -> usize {
        self.peak_resources
    }

    pub fn peak_bytes(self) -> usize {
        self.peak_bytes
    }

    pub fn post_warm_resource_range(self) -> (usize, usize) {
        (self.post_warm_min_resources, self.post_warm_max_resources)
    }

    pub fn post_warm_byte_range(self) -> (usize, usize) {
        (self.post_warm_min_bytes, self.post_warm_max_bytes)
    }

    pub fn settled(self) -> Work {
        self.settled
    }
}

impl PartialUpdateReceipt {
    pub fn first(self) -> Work {
        self.first
    }

    pub fn changed(self) -> Work {
        self.changed
    }

    pub fn surviving(self) -> Work {
        self.surviving
    }

    pub fn retired(self) -> Work {
        self.retired
    }
}

impl TextAtlasRetentionReceipt {
    pub fn pressure(self) -> Work {
        self.pressure
    }

    pub fn surviving(self) -> Work {
        self.surviving
    }
}

impl ScrollTickReceipt {
    pub fn initial(self) -> Work {
        self.initial
    }

    pub fn tick(self) -> Work {
        self.tick
    }

    pub fn unchanged(self) -> Work {
        self.unchanged
    }
}

impl UnrelatedSemanticReceipt {
    pub fn initial(self) -> Work {
        self.initial
    }

    pub fn changed(self) -> Work {
        self.changed
    }

    pub fn unchanged(self) -> Work {
        self.unchanged
    }
}

impl PendingActiveReceipt {
    pub fn preparation_slices(self) -> usize {
        self.preparation_slices
    }

    pub fn active_draws(self) -> usize {
        self.active_draws
    }

    pub fn peak_pending_states(self) -> usize {
        self.peak_pending_states
    }

    pub fn peak_resources(self) -> usize {
        self.peak_resources
    }

    pub fn peak_bytes(self) -> usize {
        self.peak_bytes
    }

    pub fn activated(self) -> Work {
        self.activated
    }
}

impl IncrementalActivationReceipt {
    pub fn environment(&self) -> &Environment {
        &self.environment
    }

    pub fn slices(&self) -> usize {
        self.slices
    }

    pub fn batch_prepare(&self) -> Duration {
        self.batch_prepare
    }

    pub fn activated(&self) -> Work {
        self.activated
    }
}

impl RetentionReceipt {
    pub fn first(self) -> Work {
        self.first
    }

    pub fn unchanged(self) -> Work {
        self.unchanged
    }

    pub fn recreated(self) -> Work {
        self.recreated
    }

    pub fn retired(self) -> Work {
        self.retired
    }
}

pub struct Harness {
    context: render::Context,
    witness: render::Renderer,
    candidate: render::Renderer,
    popup_candidate: render::Renderer,
    scale_factor: f32,
}

impl Harness {
    pub async fn new(scale_factor: f32) -> Result<Self, String> {
        let scale_factor = if scale_factor.is_finite() && scale_factor > 0.0 {
            scale_factor
        } else {
            return Err("scale factor must be finite and positive".to_owned());
        };
        #[cfg(target_os = "windows")]
        let backends = render::context::Backends::dx12();
        #[cfg(not(target_os = "windows"))]
        let backends = render::context::Backends::all();
        let context = render::Context::new(render::context::Options::native(backends))
            .await
            .map_err(|error| error.to_string())?;
        let format = render::surface::Format::from_wgpu(wgpu::TextureFormat::Rgba8Unorm);
        let witness = render::Renderer::new(&context, format);
        let candidate = render::Renderer::new(&context, format);
        let popup_format = render::surface::Format::from_wgpu(wgpu::TextureFormat::Rgba8UnormSrgb);
        let popup_candidate = render::Renderer::new(&context, popup_format);
        Ok(Self {
            context,
            witness,
            candidate,
            popup_candidate,
            scale_factor,
        })
    }

    pub fn environment(&self) -> Environment {
        let adapter = self.context.adapter_info();
        Environment {
            adapter_name: adapter.name,
            backend: format!("{:?}", adapter.backend),
            device_type: format!("{:?}", adapter.device_type),
            os: std::env::consts::OS,
            architecture: std::env::consts::ARCH,
        }
    }

    pub fn scale_factor(&self) -> f32 {
        self.scale_factor
    }

    pub fn render(&mut self, case: Case) -> Result<(Image, Sample), String> {
        let (commit, properties) =
            scene::renderer_fixture(case.fixture()).map_err(|error| error.to_string())?;
        let commit = std::sync::Arc::new(commit);
        let (width, height) = self.physical_extent(commit.size());
        let renderer = if case.uses_popup_pack() {
            &mut self.popup_candidate
        } else {
            &mut self.candidate
        };
        render_commit_image(
            &self.context,
            renderer,
            &commit,
            &properties,
            width,
            height,
            self.scale_factor,
            case.uses_popup_pack(),
        )
    }

    pub fn work_receipt(&mut self, case: Case) -> Result<Work, String> {
        let (commit, properties) =
            scene::renderer_fixture(case.fixture()).map_err(|error| error.to_string())?;
        let commit = std::sync::Arc::new(commit);
        let (width, height) = self.physical_extent(commit.size());
        let renderer = if case.uses_popup_pack() {
            &mut self.popup_candidate
        } else {
            &mut self.candidate
        };
        let (_, stats) = renderer.draw_commit_offscreen_debug(
            &self.context,
            &commit,
            &properties,
            width,
            height,
            self.scale_factor,
            case.uses_popup_pack(),
        )?;
        let mut work = Work::from(stats);
        work.target_bytes = u64::from(width)
            .saturating_mul(u64::from(height))
            .saturating_mul(4);
        Ok(work)
    }

    pub fn retention_receipt(&mut self, case: Case) -> Result<RetentionReceipt, String> {
        let (commit, properties) =
            scene::renderer_fixture(case.fixture()).map_err(|error| error.to_string())?;
        let commit = std::sync::Arc::new(commit);
        let (width, height) = self.physical_extent(commit.size());
        let pack_popup = case.uses_popup_pack();
        let renderer = if pack_popup {
            &mut self.popup_candidate
        } else {
            &mut self.candidate
        };
        let (first_pixels, first_stats) = renderer.draw_commit_offscreen_debug(
            &self.context,
            &commit,
            &properties,
            width,
            height,
            self.scale_factor,
            pack_popup,
        )?;
        let (unchanged_pixels, unchanged_stats) = renderer.draw_commit_offscreen_debug(
            &self.context,
            &commit,
            &properties,
            width,
            height,
            self.scale_factor,
            pack_popup,
        )?;
        if first_pixels != unchanged_pixels {
            return Err("unchanged retained draw changed pixels".to_owned());
        }

        let format = if pack_popup {
            wgpu::TextureFormat::Rgba8UnormSrgb
        } else {
            wgpu::TextureFormat::Rgba8Unorm
        };
        let mut recreated =
            render::Renderer::new(&self.context, render::surface::Format::from_wgpu(format));
        let (recreated_pixels, recreated_stats) = recreated.draw_commit_offscreen_debug(
            &self.context,
            &commit,
            &properties,
            width,
            height,
            self.scale_factor,
            pack_popup,
        )?;
        if first_pixels != recreated_pixels {
            return Err("renderer recreation changed retained pixels".to_owned());
        }
        drop(recreated);
        drop(properties);
        drop(commit);

        let (empty, empty_properties) = scene::renderer_fixture(scene::FixtureCase::Empty)
            .map_err(|error| error.to_string())?;
        let empty = std::sync::Arc::new(empty);
        let (_, retired_stats) = renderer.draw_commit_offscreen_debug(
            &self.context,
            &empty,
            &empty_properties,
            width,
            height,
            self.scale_factor,
            pack_popup,
        )?;

        Ok(RetentionReceipt {
            first: first_stats.into(),
            unchanged: unchanged_stats.into(),
            recreated: recreated_stats.into(),
            retired: retired_stats.into(),
        })
    }

    pub fn scroll_tick_receipt(&mut self) -> Result<ScrollTickReceipt, String> {
        let (commit, initial_properties) = scene::renderer_fixture(scene::FixtureCase::Scroll)
            .map_err(|error| error.to_string())?;
        let tick_properties =
            scene::renderer_scroll_properties(&commit, 24, 2).map_err(|error| error.to_string())?;
        let commit = std::sync::Arc::new(commit);
        let (width, height) = self.physical_extent(commit.size());

        let (_, initial_stats) = self.candidate.draw_commit_offscreen_debug(
            &self.context,
            &commit,
            &initial_properties,
            width,
            height,
            self.scale_factor,
            false,
        )?;
        let (tick_pixels, tick_stats) = self.candidate.draw_commit_offscreen_debug(
            &self.context,
            &commit,
            &tick_properties,
            width,
            height,
            self.scale_factor,
            false,
        )?;
        let (unchanged_pixels, unchanged_stats) = self.candidate.draw_commit_offscreen_debug(
            &self.context,
            &commit,
            &tick_properties,
            width,
            height,
            self.scale_factor,
            false,
        )?;
        let (expected_pixels, _) = render_commit_image(
            &self.context,
            &mut self.witness,
            &commit,
            &tick_properties,
            width,
            height,
            self.scale_factor,
            false,
        )?;
        if tick_pixels.as_slice() != expected_pixels.pixels() {
            let differing = tick_pixels
                .iter()
                .zip(expected_pixels.pixels())
                .enumerate()
                .filter_map(|(index, (candidate, expected))| {
                    (candidate != expected).then_some((index, *candidate, *expected))
                })
                .collect::<Vec<_>>();
            let first = differing.first().copied();
            let last = differing.last().copied();
            return Err(format!(
                "retained scroll property tick differs from an independent retained realization: pixels={} first={first:?} last={last:?}",
                differing.len()
            ));
        }
        if tick_pixels != unchanged_pixels {
            return Err("unchanged retained scroll property state changed pixels".to_owned());
        }

        Ok(ScrollTickReceipt {
            initial: initial_stats.into(),
            tick: tick_stats.into(),
            unchanged: unchanged_stats.into(),
        })
    }

    pub fn require_scroll_text_runway(&mut self) -> Result<(), String> {
        let ((commit, initial, tick), (expected, expected_properties)) =
            scene::renderer_scroll_text_runway_pair().map_err(|error| error.to_string())?;
        let commit = std::sync::Arc::new(commit);
        let expected = std::sync::Arc::new(expected);
        let (width, height) = self.physical_extent(commit.size());

        self.candidate.draw_commit_offscreen_debug(
            &self.context,
            &commit,
            &initial,
            width,
            height,
            self.scale_factor,
            false,
        )?;
        let (actual, _) = self.candidate.draw_commit_offscreen_debug(
            &self.context,
            &commit,
            &tick,
            width,
            height,
            self.scale_factor,
            false,
        )?;
        let (expected, _) = self.witness.draw_commit_offscreen_debug(
            &self.context,
            &expected,
            &expected_properties,
            width,
            height,
            self.scale_factor,
            false,
        )?;
        if actual != expected {
            let mut changed = 0_usize;
            let mut bounds = (width, height, 0_u32, 0_u32);
            let mut first = None;
            for (index, (actual, expected)) in actual.iter().zip(&expected).enumerate() {
                if actual == expected {
                    continue;
                }
                changed += 1;
                let x = index as u32 % width;
                let y = index as u32 / width;
                bounds.0 = bounds.0.min(x);
                bounds.1 = bounds.1.min(y);
                bounds.2 = bounds.2.max(x);
                bounds.3 = bounds.3.max(y);
                first.get_or_insert((x, y, *actual, *expected));
            }
            return Err(format!(
                "a scroll property tick exposed {changed} incomplete runway pixels: bounds={bounds:?}, first={first:?}"
            ));
        }
        Ok(())
    }

    pub fn scroll_unrelated_semantic_receipt(
        &mut self,
    ) -> Result<UnrelatedSemanticReceipt, String> {
        let ((initial, initial_properties), (changed, changed_properties)) =
            scene::renderer_scroll_semantic_pair().map_err(|error| error.to_string())?;
        let initial = std::sync::Arc::new(initial);
        let changed = std::sync::Arc::new(changed);
        let (width, height) = self.physical_extent(initial.size());

        let (initial_pixels, initial_stats) = self.candidate.draw_commit_offscreen_debug(
            &self.context,
            &initial,
            &initial_properties,
            width,
            height,
            self.scale_factor,
            false,
        )?;
        let (changed_pixels, changed_stats) = self.candidate.draw_commit_offscreen_debug(
            &self.context,
            &changed,
            &changed_properties,
            width,
            height,
            self.scale_factor,
            false,
        )?;
        drop(initial);
        let (unchanged_pixels, unchanged_stats) = self.candidate.draw_commit_offscreen_debug(
            &self.context,
            &changed,
            &changed_properties,
            width,
            height,
            self.scale_factor,
            false,
        )?;
        if initial_pixels != changed_pixels || changed_pixels != unchanged_pixels {
            return Err("an unrelated semantic commit changed retained scroll pixels".to_owned());
        }

        Ok(UnrelatedSemanticReceipt {
            initial: initial_stats.into(),
            changed: changed_stats.into(),
            unchanged: unchanged_stats.into(),
        })
    }

    pub fn pending_active_receipt(&mut self) -> Result<PendingActiveReceipt, String> {
        let ((initial, initial_properties), (changed, changed_properties)) =
            scene::renderer_scroll_layer_semantic_pair().map_err(|error| error.to_string())?;
        let initial = std::sync::Arc::new(initial);
        let changed = std::sync::Arc::new(changed);
        let (width, height) = self.physical_extent(initial.size());
        let (active_pixels, _) = self.candidate.draw_commit_offscreen_debug(
            &self.context,
            &initial,
            &initial_properties,
            width,
            height,
            self.scale_factor,
            false,
        )?;

        let first = self.candidate.synchronize_commit_offscreen_debug(
            &self.context,
            &changed,
            &changed_properties,
            width,
            height,
            self.scale_factor,
            Duration::ZERO,
        )?;
        if first != render::CommitReadiness::Pending {
            return Err("a zero-budget semantic synchronization completed atomically".to_owned());
        }
        if self.candidate.retained_state_counts_debug() != (1, 1) {
            return Err(
                "initial pending work did not retain exactly active plus pending state".to_owned(),
            );
        }
        self.candidate.cancel_commit_synchronization_debug(&changed);
        if self.candidate.retained_state_counts_debug() != (1, 0) {
            return Err("cancelled stale pending work remained eligible for activation".to_owned());
        }

        let mut preparation_slices = 0;
        let mut active_draws = 0;
        let mut peak_pending_states = 0;
        let mut peak_resources = 0;
        let mut peak_bytes = 0;
        loop {
            let readiness = self.candidate.synchronize_commit_offscreen_debug(
                &self.context,
                &changed,
                &changed_properties,
                width,
                height,
                self.scale_factor,
                Duration::ZERO,
            )?;
            let (ready_plans, pending_states) = self.candidate.retained_state_counts_debug();
            if ready_plans > 2 || pending_states > 1 {
                return Err(format!(
                    "active/pending preparation exceeded its state bound: ready={ready_plans}, pending={pending_states}"
                ));
            }
            peak_pending_states = peak_pending_states.max(pending_states);
            let (resources, bytes) = self.candidate.retained_resource_state_debug();
            peak_resources = peak_resources.max(resources);
            peak_bytes = peak_bytes.max(bytes);
            if readiness == render::CommitReadiness::Ready {
                break;
            }
            preparation_slices += 1;
            let (visible, active_work) = self.candidate.draw_commit_offscreen_debug(
                &self.context,
                &initial,
                &initial_properties,
                width,
                height,
                self.scale_factor,
                false,
            )?;
            if visible != active_pixels {
                return Err(
                    "pending resources leaked into active output before activation".to_owned(),
                );
            }
            if active_work.property_upload_bytes != 0 {
                return Err(format!(
                    "pending preparation forced {} active property-upload bytes",
                    active_work.property_upload_bytes
                ));
            }
            self.candidate
                .advance_candidate_after_present_offscreen_debug(
                    &self.context,
                    &changed,
                    &changed_properties,
                    width,
                    height,
                    self.scale_factor,
                )?;
            active_draws += 1;
            if preparation_slices > 512 {
                return Err("bounded semantic preparation did not converge".to_owned());
            }
        }

        let (activated_pixels, activated) = self.candidate.draw_commit_offscreen_debug(
            &self.context,
            &changed,
            &changed_properties,
            width,
            height,
            self.scale_factor,
            false,
        )?;
        if activated_pixels != active_pixels {
            return Err("atomic activation changed an equivalent semantic fixture".to_owned());
        }
        if active_draws == 0 {
            return Err("pending preparation never yielded to the active state".to_owned());
        }
        if activated.scroll_layer_cache_misses != 0 || activated.scroll_layer_cache_hits != 0 {
            return Err("atomic activation reintroduced a scroll offscreen cache".to_owned());
        }
        if peak_pending_states != 1 {
            return Err("pending preparation never occupied exactly one candidate slot".to_owned());
        }
        if peak_resources > activated.retained_gpu_resource_count
            || peak_bytes > activated.retained_gpu_resource_bytes
        {
            return Err(format!(
                "pending preparation exceeded the activated active-plus-pending resource bound: peak={peak_resources}/{peak_bytes}, activated={}/{}",
                activated.retained_gpu_resource_count, activated.retained_gpu_resource_bytes,
            ));
        }

        let activated = Work::from(activated);
        Ok(PendingActiveReceipt {
            preparation_slices: activated.commit_preparation_slices,
            active_draws,
            peak_pending_states,
            peak_resources,
            peak_bytes,
            activated,
        })
    }

    pub fn text_atlas_retention_receipt(&mut self) -> Result<TextAtlasRetentionReceipt, String> {
        let ((active, active_properties), (pressure, pressure_properties)) =
            scene::renderer_text_atlas_pressure_pair().map_err(|error| error.to_string())?;
        let active = std::sync::Arc::new(active);
        let pressure = std::sync::Arc::new(pressure);
        let (width, height) = self.physical_extent(active.size());
        let (expected, _) = self.candidate.draw_commit_offscreen_debug(
            &self.context,
            &active,
            &active_properties,
            width,
            height,
            self.scale_factor,
            false,
        )?;
        if !expected.iter().any(|pixel| pixel[0] > 0.5) {
            return Err("retained text atlas fixture produced no visible active glyphs".to_owned());
        }
        let (_, pressure_work) = self.candidate.draw_commit_offscreen_debug(
            &self.context,
            &pressure,
            &pressure_properties,
            width,
            height,
            self.scale_factor,
            false,
        )?;
        let (surviving, surviving_work) = self.candidate.draw_commit_offscreen_debug(
            &self.context,
            &active,
            &active_properties,
            width,
            height,
            self.scale_factor,
            false,
        )?;
        if surviving != expected {
            let differing = expected
                .iter()
                .zip(&surviving)
                .filter(|(expected, actual)| expected != actual)
                .count();
            return Err(format!(
                "atlas pressure changed {differing} pixels of unchanged retained text"
            ));
        }

        Ok(TextAtlasRetentionReceipt {
            pressure: pressure_work.into(),
            surviving: surviving_work.into(),
        })
    }

    pub fn pending_resize_receipt(&mut self) -> Result<(), String> {
        let ((commit, properties), _) =
            scene::renderer_scroll_semantic_pair().map_err(|error| error.to_string())?;
        let commit = std::sync::Arc::new(commit);
        let (width, height) = self.physical_extent(commit.size());
        let resized_width = width.saturating_add(64);
        let resized_height = height.saturating_add(48);

        let first = self.candidate.synchronize_commit_offscreen_debug(
            &self.context,
            &commit,
            &properties,
            width,
            height,
            self.scale_factor,
            Duration::ZERO,
        )?;
        if first != render::CommitReadiness::Pending
            || self.candidate.retained_state_counts_debug() != (0, 1)
        {
            return Err("initial pending resize fixture was not singular and bounded".to_owned());
        }

        let resized_first = self.candidate.synchronize_commit_offscreen_debug(
            &self.context,
            &commit,
            &properties,
            resized_width,
            resized_height,
            self.scale_factor,
            Duration::ZERO,
        )?;
        if resized_first != render::CommitReadiness::Pending
            || self.candidate.retained_state_counts_debug() != (0, 1)
        {
            return Err(
                "resize did not replace the stale pending viewport exactly once".to_owned(),
            );
        }

        for slice in 0..16_384 {
            let readiness = self.candidate.synchronize_commit_offscreen_debug(
                &self.context,
                &commit,
                &properties,
                resized_width,
                resized_height,
                self.scale_factor,
                Duration::ZERO,
            )?;
            if readiness == render::CommitReadiness::Ready {
                break;
            }
            self.candidate
                .advance_candidate_after_present_offscreen_debug(
                    &self.context,
                    &commit,
                    &properties,
                    resized_width,
                    resized_height,
                    self.scale_factor,
                )?;
            if slice == 16_383 {
                return Err("resized pending realization did not converge".to_owned());
            }
        }
        if self.candidate.retained_state_counts_debug() != (1, 0) {
            return Err("ready resized realization did not retire pending state".to_owned());
        }

        let (expected, _) = self.witness.draw_commit_offscreen_debug(
            &self.context,
            &commit,
            &properties,
            resized_width,
            resized_height,
            self.scale_factor,
            false,
        )?;
        let (actual, _) = self.candidate.draw_commit_offscreen_debug(
            &self.context,
            &commit,
            &properties,
            resized_width,
            resized_height,
            self.scale_factor,
            false,
        )?;
        if actual != expected {
            return Err("resized pending realization changed final pixels".to_owned());
        }

        let old_viewport_again = self.candidate.synchronize_commit_offscreen_debug(
            &self.context,
            &commit,
            &properties,
            width,
            height,
            self.scale_factor,
            Duration::ZERO,
        )?;
        if old_viewport_again != render::CommitReadiness::Pending
            || self.candidate.retained_state_counts_debug() != (0, 1)
        {
            return Err("viewport replacement retained an obsolete ready plan".to_owned());
        }
        self.candidate.cancel_commit_synchronization_debug(&commit);
        if self.candidate.retained_state_counts_debug() != (0, 0) {
            return Err("cancelled resize realization left pending or ready residue".to_owned());
        }

        Ok(())
    }

    pub(crate) fn compare_incremental_activation(
        &mut self,
        commit: &std::sync::Arc<scene::Commit>,
        properties: &scene::Properties,
    ) -> Result<IncrementalActivationReceipt, String> {
        let (width, height) = self.physical_extent(commit.size());
        let (expected, _) = self.witness.draw_commit_offscreen_debug(
            &self.context,
            commit,
            properties,
            width,
            height,
            self.scale_factor,
            false,
        )?;

        let mut slices = 0_usize;
        loop {
            slices = slices.saturating_add(1);
            let readiness = self.candidate.synchronize_commit_offscreen_debug(
                &self.context,
                commit,
                properties,
                width,
                height,
                self.scale_factor,
                Duration::ZERO,
            )?;
            if readiness == render::CommitReadiness::Ready {
                break;
            }
            self.candidate
                .advance_candidate_after_present_offscreen_debug(
                    &self.context,
                    commit,
                    properties,
                    width,
                    height,
                    self.scale_factor,
                )?;
            if slices > 16_384 {
                return Err("incremental semantic preparation did not converge".to_owned());
            }
        }
        if slices <= 1 {
            return Err("production scene did not exercise incremental preparation".to_owned());
        }

        let (actual, activated, batch_prepare) = self.candidate.draw_commit_offscreen_debug_timed(
            &self.context,
            commit,
            properties,
            width,
            height,
            self.scale_factor,
            false,
        )?;
        if actual != expected {
            let changed = actual
                .iter()
                .zip(&expected)
                .filter(|(actual, expected)| actual != expected)
                .count();
            let visible_actual = actual
                .iter()
                .filter(|pixel| pixel[0] > 0.01 || pixel[1] > 0.01 || pixel[2] > 0.01)
                .count();
            let visible_expected = expected
                .iter()
                .filter(|pixel| pixel[0] > 0.01 || pixel[1] > 0.01 || pixel[2] > 0.01)
                .count();
            return Err(format!(
                "incremental activation changed {changed} pixels (visible incremental={visible_actual}, synchronous={visible_expected}, slices={slices})"
            ));
        }

        Ok(IncrementalActivationReceipt {
            environment: self.environment(),
            slices,
            batch_prepare,
            activated: activated.into(),
        })
    }

    pub(crate) fn compare_pending_transition_preserves_active(
        &mut self,
        active_commit: &std::sync::Arc<scene::Commit>,
        active_properties: &scene::Properties,
        candidate_commit: &std::sync::Arc<scene::Commit>,
        candidate_properties: &scene::Properties,
    ) -> Result<(), String> {
        let (width, height) = self.physical_extent(active_commit.size());
        let (active_pixels, _) = self.candidate.draw_commit_offscreen_debug(
            &self.context,
            active_commit,
            active_properties,
            width,
            height,
            self.scale_factor,
            false,
        )?;
        let (expected_candidate, _) = self.witness.draw_commit_offscreen_debug(
            &self.context,
            candidate_commit,
            candidate_properties,
            width,
            height,
            self.scale_factor,
            false,
        )?;

        let mut slices = 0_usize;
        let mut active_draws = 0_usize;
        loop {
            slices = slices.saturating_add(1);
            let readiness = self.candidate.synchronize_commit_offscreen_debug(
                &self.context,
                candidate_commit,
                candidate_properties,
                width,
                height,
                self.scale_factor,
                Duration::ZERO,
            )?;
            if readiness == render::CommitReadiness::Ready {
                break;
            }
            let (visible, active_work) = self.candidate.draw_commit_offscreen_debug(
                &self.context,
                active_commit,
                active_properties,
                width,
                height,
                self.scale_factor,
                false,
            )?;
            if visible != active_pixels {
                let changed = visible
                    .iter()
                    .zip(&active_pixels)
                    .filter(|(visible, expected)| visible != expected)
                    .count();
                return Err(format!(
                    "pending production resources changed {changed} active pixels after slice {slices}"
                ));
            }
            if active_work.property_upload_bytes != 0 {
                return Err(format!(
                    "pending production preparation forced {} active property-upload bytes after slice {slices}",
                    active_work.property_upload_bytes
                ));
            }
            self.candidate
                .advance_candidate_after_present_offscreen_debug(
                    &self.context,
                    candidate_commit,
                    candidate_properties,
                    width,
                    height,
                    self.scale_factor,
                )?;
            active_draws = active_draws.saturating_add(1);
            if slices > 16_384 {
                return Err("pending production transition did not converge".to_owned());
            }
        }
        if active_draws == 0 {
            return Err("production transition did not yield to an active draw".to_owned());
        }

        let (activated, _) = self.candidate.draw_commit_offscreen_debug(
            &self.context,
            candidate_commit,
            candidate_properties,
            width,
            height,
            self.scale_factor,
            false,
        )?;
        if activated != expected_candidate {
            let changed = activated
                .iter()
                .zip(&expected_candidate)
                .filter(|(actual, expected)| actual != expected)
                .count();
            return Err(format!(
                "atomic production activation changed {changed} candidate pixels after {slices} slices"
            ));
        }

        Ok(())
    }

    pub(crate) fn compare_pending_property_projection(
        &mut self,
        active_commit: &std::sync::Arc<scene::Commit>,
        active_properties: &scene::Properties,
        candidate_commit: &std::sync::Arc<scene::Commit>,
        candidate_properties: &scene::Properties,
    ) -> Result<(), String> {
        let (width, height) = self.physical_extent(active_commit.size());
        let (active_pixels, _) = self.candidate.draw_commit_offscreen_debug(
            &self.context,
            active_commit,
            active_properties,
            width,
            height,
            self.scale_factor,
            false,
        )?;
        let first = self.candidate.synchronize_commit_offscreen_debug(
            &self.context,
            candidate_commit,
            candidate_properties,
            width,
            height,
            self.scale_factor,
            Duration::ZERO,
        )?;
        if first != render::CommitReadiness::Pending {
            return Err("property projection fixture did not leave a pending candidate".to_owned());
        }
        let (projected, changed) = candidate_properties
            .project_onto(active_commit, active_properties)
            .map_err(|error| error.to_string())?;
        if !changed {
            return Err(
                "pending candidate carried no property change shared with active".to_owned(),
            );
        }
        let (expected, _) = self.witness.draw_commit_offscreen_debug(
            &self.context,
            active_commit,
            &projected,
            width,
            height,
            self.scale_factor,
            false,
        )?;
        let (actual, stats) = self.candidate.draw_commit_offscreen_debug(
            &self.context,
            active_commit,
            &projected,
            width,
            height,
            self.scale_factor,
            false,
        )?;
        self.candidate
            .cancel_commit_synchronization_debug(candidate_commit);
        if actual != expected {
            return Err("pending property projection changed active output pixels".to_owned());
        }
        if actual == active_pixels {
            return Err(
                "pending property projection did not advance visible active state".to_owned(),
            );
        }
        if stats.render_plan_reuses == 0
            || stats.content_upload_bytes > 0
            || stats.quad_prepare_calls > 0
            || stats.text_prepare_calls > 0
        {
            return Err("pending active property refresh rebuilt semantic content".to_owned());
        }
        Ok(())
    }

    pub(crate) fn require_visible_commit(
        &mut self,
        commit: &std::sync::Arc<scene::Commit>,
        properties: &scene::Properties,
        context: &str,
    ) -> Result<(), String> {
        let (width, height) = self.physical_extent(commit.size());
        let (pixels, _) = self.witness.draw_commit_offscreen_debug(
            &self.context,
            commit,
            properties,
            width,
            height,
            self.scale_factor,
            false,
        )?;
        let clear = render::surface_color(commit.clear());
        let clear = [
            clear.r as f32,
            clear.g as f32,
            clear.b as f32,
            clear.a as f32,
        ];
        let visible = pixels
            .iter()
            .filter(|pixel| {
                pixel
                    .iter()
                    .zip(clear)
                    .any(|(channel, clear)| (channel - clear).abs() > (1.0 / 255.0))
            })
            .count();
        if visible == 0 {
            return Err(format!("{context} rendered only the clear color"));
        }
        Ok(())
    }

    pub(crate) fn compare_retained_scroll_transition(
        &mut self,
        commit: &std::sync::Arc<scene::Commit>,
        initial: &scene::Properties,
        tick: &scene::Properties,
    ) -> Result<(), String> {
        let (width, height) = self.physical_extent(commit.size());

        let (initial_candidate, _) = self.candidate.draw_commit_offscreen_debug(
            &self.context,
            commit,
            initial,
            width,
            height,
            self.scale_factor,
            false,
        )?;
        let (initial_expected, _) = render_commit_image(
            &self.context,
            &mut self.witness,
            commit,
            initial,
            width,
            height,
            self.scale_factor,
            false,
        )?;
        let (candidate, _) = self.candidate.draw_commit_offscreen_debug(
            &self.context,
            commit,
            tick,
            width,
            height,
            self.scale_factor,
            false,
        )?;
        let (expected, _) = render_commit_image(
            &self.context,
            &mut self.witness,
            commit,
            tick,
            width,
            height,
            self.scale_factor,
            false,
        )?;

        if candidate.as_slice() == expected.pixels()
            && initial_candidate.as_slice() == initial_expected.pixels()
        {
            Ok(())
        } else {
            let mut differing = 0_usize;
            let mut min_x = width;
            let mut min_y = height;
            let mut max_x = 0_u32;
            let mut max_y = 0_u32;
            let mut maximum_channel_delta = 0.0_f32;
            let mut over_four_bytes = 0_usize;
            let mut first = None;
            let mut last = None;
            let mut maximum_sample = None;
            for (index, (candidate, expected)) in
                candidate.iter().zip(expected.pixels()).enumerate()
            {
                if candidate == expected {
                    continue;
                }
                let x = index as u32 % width;
                let y = index as u32 / width;
                differing += 1;
                min_x = min_x.min(x);
                min_y = min_y.min(y);
                max_x = max_x.max(x);
                max_y = max_y.max(y);
                let delta = candidate
                    .iter()
                    .zip(expected)
                    .map(|(candidate, expected)| (candidate - expected).abs())
                    .fold(0.0_f32, f32::max);
                if delta > maximum_channel_delta {
                    maximum_channel_delta = delta;
                    maximum_sample = Some((x, y, *candidate, *expected));
                }
                over_four_bytes += usize::from(delta > 4.0 / 255.0);
                let sample = (x, y, *candidate, *expected);
                first.get_or_insert(sample);
                last = Some(sample);
            }
            let initial_differing = initial_candidate
                .iter()
                .zip(initial_expected.pixels())
                .filter(|(candidate, expected)| candidate != expected)
                .count();
            let initial_maximum_channel_delta = initial_candidate
                .iter()
                .zip(initial_expected.pixels())
                .flat_map(|(candidate, expected)| candidate.iter().zip(expected))
                .map(|(candidate, expected)| (candidate - expected).abs())
                .fold(0.0_f32, f32::max);
            let maximum_context = maximum_sample.map(|(x, y, _, _)| {
                let sample = |pixels: &[[f32; 4]], sample_y: u32| {
                    pixels[(sample_y.min(height.saturating_sub(1)) * width + x) as usize]
                };
                (
                    sample(&initial_candidate, y),
                    sample(initial_expected.pixels(), y),
                    sample(&candidate, y.saturating_sub(10)),
                    sample(expected.pixels(), y.saturating_sub(10)),
                    sample(&candidate, y.saturating_add(10)),
                    sample(expected.pixels(), y.saturating_add(10)),
                )
            });
            let maximum_nearest = maximum_sample.map(|(x, y, candidate_pixel, _)| {
                let mut nearest = (0_i32, 0_i32, f32::MAX, [0.0_f32; 4]);
                for dy in -4_i32..=4 {
                    for dx in -4_i32..=4 {
                        let sample_x = (x as i32 + dx).clamp(0, width as i32 - 1) as u32;
                        let sample_y = (y as i32 + dy).clamp(0, height as i32 - 1) as u32;
                        let expected_pixel =
                            expected.pixels()[(sample_y * width + sample_x) as usize];
                        let delta = candidate_pixel
                            .iter()
                            .zip(expected_pixel)
                            .map(|(candidate, expected)| (candidate - expected).abs())
                            .fold(0.0_f32, f32::max);
                        if delta < nearest.2 {
                            nearest = (dx, dy, delta, expected_pixel);
                        }
                    }
                }
                nearest
            });
            const FLOATING_BLEND_TOLERANCE: f32 = 4.0 / 255.0;
            if maximum_channel_delta <= FLOATING_BLEND_TOLERANCE
                && initial_maximum_channel_delta <= FLOATING_BLEND_TOLERANCE
            {
                Ok(())
            } else {
                Err(format!(
                    "runtime retained scroll transition exceeds the floating-blend oracle tolerance: tolerance={FLOATING_BLEND_TOLERANCE} tick_pixels={differing} tick_max_delta={maximum_channel_delta} tick_over_tolerance={over_four_bytes} initial_pixels={initial_differing} initial_max_delta={initial_maximum_channel_delta} bounds=({min_x},{min_y})-({max_x},{max_y}) maximum={maximum_sample:?} maximum_nearest_expected=(dx,dy,delta,pixel)={maximum_nearest:?} maximum_context=(initial_candidate, initial_expected, candidate_y_minus_10, expected_y_minus_10, candidate_y_plus_10, expected_y_plus_10)={maximum_context:?} first={first:?} last={last:?}"
                ))
            }
        }
    }

    pub(crate) fn require_retained_scroll_region_change(
        &mut self,
        commit: &std::sync::Arc<scene::Commit>,
        initial: &scene::Properties,
        tick: &scene::Properties,
        region: crate::geometry::Rect,
        context: &str,
    ) -> Result<(), String> {
        self.compare_retained_scroll_transition(commit, initial, tick)?;

        let (width, height) = self.physical_extent(commit.size());
        let (initial, _) = render_commit_image(
            &self.context,
            &mut self.witness,
            commit,
            initial,
            width,
            height,
            self.scale_factor,
            false,
        )?;
        let (tick, _) = render_commit_image(
            &self.context,
            &mut self.witness,
            commit,
            tick,
            width,
            height,
            self.scale_factor,
            false,
        )?;
        let physical_edge = |value: i32, round: fn(f32) -> f32, limit: u32| {
            round(value as f32 * self.scale_factor).clamp(0.0, limit as f32) as u32
        };
        let left = physical_edge(region.x(), f32::floor, width);
        let top = physical_edge(region.y(), f32::floor, height);
        let right = physical_edge(region.right(), f32::ceil, width);
        let bottom = physical_edge(region.bottom(), f32::ceil, height);
        if left >= right || top >= bottom {
            return Err(format!(
                "{context} selected an empty scroll witness region: logical={region:?}, physical=({left}, {top})..({right}, {bottom})"
            ));
        }

        let mut changed = 0_usize;
        for y in top..bottom {
            for x in left..right {
                let index = (y * width + x) as usize;
                changed = changed
                    .saturating_add(usize::from(initial.pixels()[index] != tick.pixels()[index]));
            }
        }
        if changed == 0 {
            return Err(format!(
                "{context} did not update any entering-side pixels in {region:?} at {}x",
                self.scale_factor
            ));
        }
        Ok(())
    }

    pub fn partial_update_receipt(&mut self) -> Result<PartialUpdateReceipt, String> {
        let (first_commit, first_properties) =
            scene::renderer_partial_update_fixture(1).map_err(|error| error.to_string())?;
        let first_commit = std::sync::Arc::new(first_commit);
        let (changed_commit, changed_properties) =
            scene::renderer_partial_update_fixture(2).map_err(|error| error.to_string())?;
        let changed_commit = std::sync::Arc::new(changed_commit);
        let (width, height) = self.physical_extent(first_commit.size());

        let (_, first_stats) = self.candidate.draw_commit_offscreen_debug(
            &self.context,
            &first_commit,
            &first_properties,
            width,
            height,
            self.scale_factor,
            false,
        )?;
        let (_, changed_stats) = self.candidate.draw_commit_offscreen_debug(
            &self.context,
            &changed_commit,
            &changed_properties,
            width,
            height,
            self.scale_factor,
            false,
        )?;
        drop(first_properties);
        drop(first_commit);
        let (_, surviving_stats) = self.candidate.draw_commit_offscreen_debug(
            &self.context,
            &changed_commit,
            &changed_properties,
            width,
            height,
            self.scale_factor,
            false,
        )?;
        drop(changed_properties);
        drop(changed_commit);

        let (empty, empty_properties) = scene::renderer_fixture(scene::FixtureCase::Empty)
            .map_err(|error| error.to_string())?;
        let empty = std::sync::Arc::new(empty);
        let (_, retired_stats) = self.candidate.draw_commit_offscreen_debug(
            &self.context,
            &empty,
            &empty_properties,
            width,
            height,
            self.scale_factor,
            false,
        )?;

        Ok(PartialUpdateReceipt {
            first: first_stats.into(),
            changed: changed_stats.into(),
            surviving: surviving_stats.into(),
            retired: retired_stats.into(),
        })
    }

    pub fn churn_receipt(&mut self, iterations: usize) -> Result<ChurnReceipt, String> {
        if iterations < 3 {
            return Err("churn receipt requires at least three iterations".to_owned());
        }
        let mut active = None;
        let mut peak_resources = 0;
        let mut peak_bytes = 0;
        let mut post_warm_min_resources = usize::MAX;
        let mut post_warm_max_resources = 0;
        let mut post_warm_min_bytes = usize::MAX;
        let mut post_warm_max_bytes = 0;

        for version in 1..=iterations as u64 {
            let (commit, properties) = scene::renderer_partial_update_fixture(version)
                .map_err(|error| error.to_string())?;
            let commit = std::sync::Arc::new(commit);
            let (width, height) = self.physical_extent(commit.size());
            let (_, stats) = self.candidate.draw_commit_offscreen_debug(
                &self.context,
                &commit,
                &properties,
                width,
                height,
                self.scale_factor,
                false,
            )?;
            let work = Work::from(stats);
            peak_resources = peak_resources.max(work.gpu_resource_count());
            peak_bytes = peak_bytes.max(work.gpu_resource_bytes());
            if version >= 3 {
                post_warm_min_resources = post_warm_min_resources.min(work.gpu_resource_count());
                post_warm_max_resources = post_warm_max_resources.max(work.gpu_resource_count());
                post_warm_min_bytes = post_warm_min_bytes.min(work.gpu_resource_bytes());
                post_warm_max_bytes = post_warm_max_bytes.max(work.gpu_resource_bytes());
            }
            active = Some((commit, properties));
        }
        drop(active);

        let (empty, empty_properties) = scene::renderer_fixture(scene::FixtureCase::Empty)
            .map_err(|error| error.to_string())?;
        let empty = std::sync::Arc::new(empty);
        let (width, height) = self.physical_extent(empty.size());
        let (_, settled) = self.candidate.draw_commit_offscreen_debug(
            &self.context,
            &empty,
            &empty_properties,
            width,
            height,
            self.scale_factor,
            false,
        )?;

        Ok(ChurnReceipt {
            iterations,
            peak_resources,
            peak_bytes,
            post_warm_min_resources,
            post_warm_max_resources,
            post_warm_min_bytes,
            post_warm_max_bytes,
            settled: settled.into(),
        })
    }

    fn physical_extent(&self, size: crate::geometry::Size) -> (u32, u32) {
        (
            (size.width() as f32 * self.scale_factor).round() as u32,
            (size.height() as f32 * self.scale_factor).round() as u32,
        )
    }

    pub(crate) fn compare_retained_property_transition(
        &mut self,
        commit: &std::sync::Arc<scene::Commit>,
        initial: &scene::Properties,
        tick: &scene::Properties,
    ) -> Result<Work, String> {
        let (width, height) = self.physical_extent(commit.size());
        let (initial_pixels, _) = self.candidate.draw_commit_offscreen_debug(
            &self.context,
            commit,
            initial,
            width,
            height,
            self.scale_factor,
            false,
        )?;
        let (candidate, work) = self.candidate.draw_commit_offscreen_debug(
            &self.context,
            commit,
            tick,
            width,
            height,
            self.scale_factor,
            false,
        )?;
        let (expected, _) = render_commit_image(
            &self.context,
            &mut self.witness,
            commit,
            tick,
            width,
            height,
            self.scale_factor,
            false,
        )?;
        if candidate.as_slice() != expected.pixels() {
            let differing = candidate
                .iter()
                .zip(expected.pixels())
                .filter(|(candidate, expected)| candidate != expected)
                .count();
            return Err(format!(
                "retained property transition differs from independent realization at {differing} pixels"
            ));
        }
        if tick.changed().iter().any(|property| {
            matches!(
                tick.value(*property),
                Some(scene::PropertyValue::Caret { .. })
            )
        }) && candidate == initial_pixels
        {
            return Err(
                "retained caret property transition changed state without changing pixels"
                    .to_owned(),
            );
        }
        Ok(work.into())
    }
}

fn render_commit_image(
    context: &render::Context,
    renderer: &mut render::Renderer,
    commit: &std::sync::Arc<scene::Commit>,
    properties: &scene::Properties,
    width: u32,
    height: u32,
    scale_factor: f32,
    pack_popup: bool,
) -> Result<(Image, Sample), String> {
    let started = Instant::now();
    let (pixels, _) = renderer.draw_commit_offscreen_debug(
        context,
        commit,
        properties,
        width,
        height,
        scale_factor,
        pack_popup,
    )?;
    Ok((
        Image::new(width, height, pixels)?,
        Sample {
            elapsed: started.elapsed(),
        },
    ))
}
