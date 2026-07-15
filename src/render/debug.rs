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
    GlassPane,
    TransparentPopup,
}

impl Case {
    pub const ALL: [Self; 16] = [
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
    pub fn new(scale_factor: f32) -> Result<Self, String> {
        let scale_factor = if scale_factor.is_finite() && scale_factor > 0.0 {
            scale_factor
        } else {
            return Err("scale factor must be finite and positive".to_owned());
        };
        #[cfg(target_os = "windows")]
        let backends = render::context::Backends::dx12();
        #[cfg(not(target_os = "windows"))]
        let backends = render::context::Backends::all();
        let context = pollster::block_on(render::Context::new(render::context::Options::native(
            backends,
        )))
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
        let paint =
            render::scene::to_paint_commit_at_scale(&commit, &properties, self.scale_factor)
                .map_err(|error| error.to_string())?;
        let (width, height) = self.physical_extent(commit.size());
        if case.uses_popup_pack() {
            render_popup_image(
                &self.context,
                &mut self.popup_candidate,
                &paint,
                width,
                height,
                self.scale_factor,
            )
        } else {
            render_image(
                &self.context,
                &mut self.candidate,
                &paint,
                width,
                height,
                self.scale_factor,
            )
        }
    }

    pub fn render_pair(&mut self, case: Case) -> Result<(Image, Image, PairSample), String> {
        let (commit, properties) =
            scene::renderer_fixture(case.fixture()).map_err(|error| error.to_string())?;
        let source = commit
            .compatibility_scene(&properties)
            .map_err(|error| error.to_string())?;
        let legacy_paint = render::scene::to_paint_scene_at_scale(&source, self.scale_factor);
        let candidate_paint =
            render::scene::to_paint_commit_at_scale(&commit, &properties, self.scale_factor)
                .map_err(|error| error.to_string())?;
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
                render_popup_image(
                    &self.context,
                    &mut self.popup_candidate,
                    &candidate_paint,
                    width,
                    height,
                    self.scale_factor,
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
                render_image(
                    &self.context,
                    &mut self.candidate,
                    &candidate_paint,
                    width,
                    height,
                    self.scale_factor,
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
