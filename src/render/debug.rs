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
    GroupOpacity,
    OrderedGroup,
    GlassPane,
    TransparentPopup,
}

impl Case {
    pub const ALL: [Self; 17] = [
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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PairSample {
    legacy: Sample,
    candidate: Sample,
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
    render_plan_rebuilds: usize,
    render_plan_reuses: usize,
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

    pub fn render_plan_rebuilds(self) -> usize {
        self.render_plan_rebuilds
    }

    pub fn render_plan_reuses(self) -> usize {
        self.render_plan_reuses
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
            render_plan_rebuilds: stats.render_plan_rebuilds,
            render_plan_reuses: stats.render_plan_reuses,
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

impl PairSample {
    pub fn legacy(self) -> Sample {
        self.legacy
    }

    pub fn candidate(self) -> Sample {
        self.candidate
    }
}

pub struct Harness {
    context: render::Context,
    legacy: render::Renderer,
    candidate: render::Renderer,
    popup_legacy: render::Renderer,
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
        let legacy = render::Renderer::new(&context, format);
        let candidate = render::Renderer::new(&context, format);
        let popup_format = render::surface::Format::from_wgpu(wgpu::TextureFormat::Rgba8UnormSrgb);
        let popup_legacy = render::Renderer::new(&context, popup_format);
        let popup_candidate = render::Renderer::new(&context, popup_format);
        Ok(Self {
            context,
            legacy,
            candidate,
            popup_legacy,
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

    pub fn render_legacy(&mut self, case: Case) -> Result<(Image, Sample), String> {
        let (commit, properties) =
            scene::renderer_fixture(case.fixture()).map_err(|error| error.to_string())?;
        let source = commit
            .compatibility_scene(&properties)
            .map_err(|error| error.to_string())?;
        let paint = render::scene::to_paint_scene_at_scale(&source, self.scale_factor);
        let (width, height) = self.physical_extent(commit.size());
        if case.uses_popup_pack() {
            render_popup_image(
                &self.context,
                &mut self.popup_legacy,
                &paint,
                width,
                height,
                self.scale_factor,
            )
        } else {
            render_image(
                &self.context,
                &mut self.legacy,
                &paint,
                width,
                height,
                self.scale_factor,
            )
        }
    }

    pub fn render_candidate(&mut self, case: Case) -> Result<(Image, Sample), String> {
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

    pub fn render_pair(&mut self, case: Case) -> Result<(Image, Image, PairSample), String> {
        let (commit, properties) =
            scene::renderer_fixture(case.fixture()).map_err(|error| error.to_string())?;
        let source = commit
            .compatibility_scene(&properties)
            .map_err(|error| error.to_string())?;
        let legacy_paint = render::scene::to_paint_scene_at_scale(&source, self.scale_factor);
        let commit = std::sync::Arc::new(commit);
        let (width, height) = self.physical_extent(commit.size());
        let ((legacy, legacy_sample), (candidate, candidate_sample)) = if case.uses_popup_pack() {
            (
                render_popup_image(
                    &self.context,
                    &mut self.popup_legacy,
                    &legacy_paint,
                    width,
                    height,
                    self.scale_factor,
                )?,
                render_commit_image(
                    &self.context,
                    &mut self.popup_candidate,
                    &commit,
                    &properties,
                    width,
                    height,
                    self.scale_factor,
                    true,
                )?,
            )
        } else {
            (
                render_image(
                    &self.context,
                    &mut self.legacy,
                    &legacy_paint,
                    width,
                    height,
                    self.scale_factor,
                )?,
                render_commit_image(
                    &self.context,
                    &mut self.candidate,
                    &commit,
                    &properties,
                    width,
                    height,
                    self.scale_factor,
                    false,
                )?,
            )
        };
        Ok((
            legacy,
            candidate,
            PairSample {
                legacy: legacy_sample,
                candidate: candidate_sample,
            },
        ))
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
}

fn render_image(
    context: &render::Context,
    renderer: &mut render::Renderer,
    scene: &crate::paint::Scene,
    width: u32,
    height: u32,
    scale_factor: f32,
) -> Result<(Image, Sample), String> {
    let started = Instant::now();
    let pixels = renderer.draw_offscreen_debug(context, scene, width, height, scale_factor)?;
    Ok((
        Image::new(width, height, pixels)?,
        Sample {
            elapsed: started.elapsed(),
        },
    ))
}

fn render_popup_image(
    context: &render::Context,
    renderer: &mut render::Renderer,
    scene: &crate::paint::Scene,
    width: u32,
    height: u32,
    scale_factor: f32,
) -> Result<(Image, Sample), String> {
    let started = Instant::now();
    let pixels =
        renderer.draw_offscreen_popup_debug(context, scene, width, height, scale_factor)?;
    Ok((
        Image::new(width, height, pixels)?,
        Sample {
            elapsed: started.elapsed(),
        },
    ))
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
