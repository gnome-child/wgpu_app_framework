use wgpu::{SurfaceTarget, SurfaceTargetUnsafe};

use crate::paint;
use crate::render;

use std::time::{Duration, Instant};
use thiserror::Error;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, Error)]
pub enum Error {
    #[error(transparent)]
    Create(#[from] wgpu::CreateSurfaceError),

    #[error("surface could not be configured")]
    NoSurfaceConfiguration,
}

pub struct Surface {
    inner: wgpu::Surface<'static>,
    config: wgpu::SurfaceConfiguration,
    reconfigure_after_present: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum CompositeAlphaPreference {
    Default,
    PreMultiplied,
}

const LOW_LATENCY_FRAME_LIMIT: u32 = 1;
const PRESENT_MODE_PRIORITY: [wgpu::PresentMode; 4] = [
    wgpu::PresentMode::Mailbox,
    wgpu::PresentMode::Immediate,
    wgpu::PresentMode::FifoRelaxed,
    wgpu::PresentMode::Fifo,
];

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct PresentTiming {
    acquire_wait: Duration,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum AcquireOutcome {
    Success,
    Suboptimal,
    Outdated,
    Timeout,
    Occluded,
    Validation,
    Lost,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct SurfaceReport {
    acquire: AcquireOutcome,
    present_timing: Option<PresentTiming>,
}

impl Surface {
    pub fn new(
        area: paint::area::Physical,
        render_context: &render::Context,
        target: impl Into<SurfaceTarget<'static>>,
        alpha_preference: CompositeAlphaPreference,
    ) -> Result<Self> {
        log::debug!(
            "creating render surface: {}x{} physical",
            area.width(),
            area.height()
        );
        let inner = render_context.instance().create_surface(target)?;
        Self::from_wgpu_surface(area, render_context, inner, alpha_preference)
    }

    /// Creates a surface whose raw target lifetime is retained by the native
    /// presentation owner rather than by wgpu's safe handle wrapper.
    pub(crate) unsafe fn new_unsafe(
        area: paint::area::Physical,
        render_context: &render::Context,
        target: SurfaceTargetUnsafe,
        alpha_preference: CompositeAlphaPreference,
    ) -> Result<Self> {
        let inner = unsafe { render_context.instance().create_surface_unsafe(target) }?;
        Self::from_wgpu_surface(area, render_context, inner, alpha_preference)
    }

    fn from_wgpu_surface(
        area: paint::area::Physical,
        render_context: &render::Context,
        inner: wgpu::Surface<'static>,
        alpha_preference: CompositeAlphaPreference,
    ) -> Result<Self> {
        let Some(mut config) = inner.get_default_config(
            render_context.adapter(),
            area.width().max(1),
            area.height().max(1),
        ) else {
            log::error!(
                "wgpu surface has no default configuration for {}x{}",
                area.width().max(1),
                area.height().max(1)
            );
            return Err(Error::NoSurfaceConfiguration);
        };

        let capabilities = inner.get_capabilities(render_context.adapter());

        let default_alpha_mode = config.alpha_mode;
        let default_format = config.format;
        config.present_mode =
            preferred_present_mode(&capabilities.present_modes, config.present_mode);
        config.format = preferred_format(&capabilities.formats, config.format, alpha_preference);
        config.alpha_mode = preferred_alpha_mode(
            &capabilities.alpha_modes,
            config.alpha_mode,
            alpha_preference,
        );
        if matches!(alpha_preference, CompositeAlphaPreference::PreMultiplied) {
            log::debug!(
                target: "wgpu_l3::native_popup",
                "popup surface capabilities: formats={:?}, default_format={:?}, selected_format={:?}, alpha={:?}, default_alpha={:?}, selected_alpha={:?}",
                capabilities.formats,
                default_format,
                config.format,
                capabilities.alpha_modes,
                default_alpha_mode,
                config.alpha_mode
            );
        }
        config.desired_maximum_frame_latency = LOW_LATENCY_FRAME_LIMIT;

        inner.configure(render_context.device(), &config);
        log::debug!(
            "configured render surface: {}x{}, format={:?}, present_mode={:?}, alpha_mode={:?}, desired_frame_latency={}",
            config.width,
            config.height,
            config.format,
            config.present_mode,
            config.alpha_mode,
            config.desired_maximum_frame_latency
        );

        Ok(Self {
            inner,
            config,
            reconfigure_after_present: false,
        })
    }

    pub(crate) fn wgpu_surface(&self) -> &wgpu::Surface<'static> {
        &self.inner
    }

    pub fn config(&self) -> &wgpu::SurfaceConfiguration {
        &self.config
    }

    pub fn resize(&mut self, render_context: &render::Context, area: paint::area::Physical) {
        let area = area.clamp_min(1);

        log::debug!(
            "resizing render surface: {}x{} -> {}x{}",
            self.config.width,
            self.config.height,
            area.width(),
            area.height()
        );
        self.config.width = area.width();
        self.config.height = area.height();

        self.inner.configure(render_context.device(), &self.config);
        self.reconfigure_after_present = false;
    }

    pub fn acquire_frame(
        &mut self,
        render_context: &render::Context,
    ) -> Result<(render::FrameOutcome, AcquireOutcome)> {
        use wgpu::CurrentSurfaceTexture::*;

        match self.inner.get_current_texture() {
            Success(surface_texture) => Ok((
                render::FrameOutcome::Acquired(render::Frame::new(surface_texture)),
                AcquireOutcome::Success,
            )),
            Suboptimal(surface_texture) => {
                log::debug!(
                    "acquired suboptimal surface texture; deferring reconfiguration until present"
                );
                self.reconfigure_after_present = true;
                Ok((
                    render::FrameOutcome::Acquired(render::Frame::new(surface_texture)),
                    AcquireOutcome::Suboptimal,
                ))
            }
            Outdated => {
                log::debug!(
                    "surface texture is outdated; reconfiguring surface and skipping frame"
                );
                self.reconfigure(render_context);
                Ok((render::FrameOutcome::Skipped, AcquireOutcome::Outdated))
            }
            Timeout => {
                log::debug!("surface texture acquisition timed out; skipping frame");
                Ok((render::FrameOutcome::Skipped, AcquireOutcome::Timeout))
            }
            Occluded => {
                log::debug!("surface is occluded; skipping frame");
                Ok((render::FrameOutcome::Skipped, AcquireOutcome::Occluded))
            }
            Validation => {
                log::warn!(
                    "surface texture acquisition returned validation status; skipping frame"
                );
                Ok((render::FrameOutcome::Skipped, AcquireOutcome::Validation))
            }
            Lost => {
                log::warn!("surface was lost; reconfiguring surface and skipping frame");
                self.reconfigure(render_context);
                Ok((render::FrameOutcome::Skipped, AcquireOutcome::Lost))
            }
        }
    }

    pub fn render(
        &mut self,
        render_context: &render::Context,
        encode: impl FnOnce(&mut wgpu::CommandEncoder, &render::Frame),
    ) -> Result<SurfaceReport> {
        let acquire_started = Instant::now();
        let (outcome, acquire) = self.acquire_frame(render_context)?;
        let acquire_wait = acquire_started.elapsed();

        use render::FrameOutcome::*;
        let frame = match outcome {
            Acquired(frame) => frame,
            Skipped => return Ok(SurfaceReport::new(acquire, None)),
        };

        let mut encoder =
            render_context
                .device()
                .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                    label: Some("Frame Encoder"),
                });

        encode(&mut encoder, &frame);
        render_context.queue().submit([encoder.finish()]);
        frame.present();
        if self.reconfigure_after_present {
            self.reconfigure(render_context);
        }

        Ok(SurfaceReport::new(
            acquire,
            Some(PresentTiming { acquire_wait }),
        ))
    }

    pub fn reconfigure(&mut self, render_context: &render::Context) {
        log::debug!(
            "reconfiguring render surface: {}x{}, format={:?}, present_mode={:?}",
            self.config.width,
            self.config.height,
            self.config.format,
            self.config.present_mode
        );
        self.inner.configure(render_context.device(), &self.config);
        self.reconfigure_after_present = false;
    }
}

impl PresentTiming {
    pub(crate) fn acquire_wait(self) -> Duration {
        self.acquire_wait
    }
}

impl SurfaceReport {
    fn new(acquire: AcquireOutcome, present_timing: Option<PresentTiming>) -> Self {
        Self {
            acquire,
            present_timing,
        }
    }

    pub(crate) fn acquire(self) -> AcquireOutcome {
        self.acquire
    }

    pub(crate) fn present_timing(self) -> Option<PresentTiming> {
        self.present_timing
    }
}

fn preferred_present_mode(
    supported: &[wgpu::PresentMode],
    fallback: wgpu::PresentMode,
) -> wgpu::PresentMode {
    PRESENT_MODE_PRIORITY
        .into_iter()
        .find(|mode| supported.contains(mode))
        .unwrap_or(fallback)
}

fn preferred_alpha_mode(
    supported: &[wgpu::CompositeAlphaMode],
    fallback: wgpu::CompositeAlphaMode,
    preference: CompositeAlphaPreference,
) -> wgpu::CompositeAlphaMode {
    match preference {
        CompositeAlphaPreference::Default => fallback,
        CompositeAlphaPreference::PreMultiplied => {
            if supported.contains(&wgpu::CompositeAlphaMode::PreMultiplied) {
                wgpu::CompositeAlphaMode::PreMultiplied
            } else {
                log::warn!(
                    target: "wgpu_l3::native_popup",
                    "premultiplied popup surface alpha unsupported; supported={supported:?}; using {fallback:?}"
                );
                fallback
            }
        }
    }
}

fn preferred_format(
    supported: &[wgpu::TextureFormat],
    fallback: wgpu::TextureFormat,
    preference: CompositeAlphaPreference,
) -> wgpu::TextureFormat {
    match preference {
        CompositeAlphaPreference::Default => fallback,
        CompositeAlphaPreference::PreMultiplied => {
            const PREFERRED: [wgpu::TextureFormat; 2] = [
                wgpu::TextureFormat::Bgra8Unorm,
                wgpu::TextureFormat::Rgba8Unorm,
            ];

            PREFERRED
                .into_iter()
                .find(|format| supported.contains(format))
                .or_else(|| {
                    non_srgb_variant(fallback)
                        .filter(|format| supported.contains(format))
                })
                .unwrap_or_else(|| {
                    log::warn!(
                        target: "wgpu_l3::native_popup",
                        "non-sRGB premultiplied popup surface format unsupported; supported={supported:?}; using {fallback:?}"
                    );
                    fallback
                })
        }
    }
}

pub(crate) fn scene_format_for_surface_format(format: wgpu::TextureFormat) -> wgpu::TextureFormat {
    match format {
        wgpu::TextureFormat::Bgra8Unorm => wgpu::TextureFormat::Bgra8UnormSrgb,
        wgpu::TextureFormat::Rgba8Unorm => wgpu::TextureFormat::Rgba8UnormSrgb,
        _ => format,
    }
}

pub(crate) fn supports_windows_premultiplied_popup_pack(
    format: wgpu::TextureFormat,
    alpha_mode: wgpu::CompositeAlphaMode,
) -> bool {
    alpha_mode == wgpu::CompositeAlphaMode::PreMultiplied
        && matches!(
            format,
            wgpu::TextureFormat::Bgra8Unorm | wgpu::TextureFormat::Rgba8Unorm
        )
}

fn non_srgb_variant(format: wgpu::TextureFormat) -> Option<wgpu::TextureFormat> {
    match format {
        wgpu::TextureFormat::Bgra8UnormSrgb => Some(wgpu::TextureFormat::Bgra8Unorm),
        wgpu::TextureFormat::Rgba8UnormSrgb => Some(wgpu::TextureFormat::Rgba8Unorm),
        wgpu::TextureFormat::Bgra8Unorm | wgpu::TextureFormat::Rgba8Unorm => Some(format),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn surface_present_mode_priority_preserves_existing_order() {
        let supported = [wgpu::PresentMode::Fifo, wgpu::PresentMode::Immediate];

        assert_eq!(
            preferred_present_mode(&supported, wgpu::PresentMode::Fifo),
            wgpu::PresentMode::Immediate
        );
    }

    #[test]
    fn surface_frame_latency_defaults_to_low_latency_gui_value() {
        assert_eq!(LOW_LATENCY_FRAME_LIMIT, 1);
    }

    #[test]
    fn surface_alpha_prefers_premultiplied_when_supported() {
        let supported = [
            wgpu::CompositeAlphaMode::Opaque,
            wgpu::CompositeAlphaMode::PreMultiplied,
        ];

        assert_eq!(
            preferred_alpha_mode(
                &supported,
                wgpu::CompositeAlphaMode::Opaque,
                CompositeAlphaPreference::PreMultiplied,
            ),
            wgpu::CompositeAlphaMode::PreMultiplied
        );
    }

    #[test]
    fn surface_alpha_falls_back_when_premultiplied_is_unsupported() {
        let supported = [wgpu::CompositeAlphaMode::Opaque];

        assert_eq!(
            preferred_alpha_mode(
                &supported,
                wgpu::CompositeAlphaMode::Opaque,
                CompositeAlphaPreference::PreMultiplied,
            ),
            wgpu::CompositeAlphaMode::Opaque
        );
    }

    #[test]
    fn premultiplied_popup_format_prefers_non_srgb_bgra() {
        let supported = [
            wgpu::TextureFormat::Bgra8UnormSrgb,
            wgpu::TextureFormat::Bgra8Unorm,
            wgpu::TextureFormat::Rgba8Unorm,
        ];

        assert_eq!(
            preferred_format(
                &supported,
                wgpu::TextureFormat::Bgra8UnormSrgb,
                CompositeAlphaPreference::PreMultiplied,
            ),
            wgpu::TextureFormat::Bgra8Unorm
        );
    }

    #[test]
    fn premultiplied_popup_format_falls_back_when_non_srgb_is_unavailable() {
        let supported = [wgpu::TextureFormat::Bgra8UnormSrgb];

        assert_eq!(
            preferred_format(
                &supported,
                wgpu::TextureFormat::Bgra8UnormSrgb,
                CompositeAlphaPreference::PreMultiplied,
            ),
            wgpu::TextureFormat::Bgra8UnormSrgb
        );
    }

    #[test]
    fn premultiplied_popup_pack_requires_non_srgb_format_and_premultiplied_alpha() {
        assert!(supports_windows_premultiplied_popup_pack(
            wgpu::TextureFormat::Bgra8Unorm,
            wgpu::CompositeAlphaMode::PreMultiplied
        ));
        assert!(!supports_windows_premultiplied_popup_pack(
            wgpu::TextureFormat::Bgra8UnormSrgb,
            wgpu::CompositeAlphaMode::PreMultiplied
        ));
        assert!(!supports_windows_premultiplied_popup_pack(
            wgpu::TextureFormat::Bgra8Unorm,
            wgpu::CompositeAlphaMode::Opaque
        ));
    }

    #[test]
    fn non_srgb_popup_surface_maps_to_srgb_scene_format() {
        assert_eq!(
            scene_format_for_surface_format(wgpu::TextureFormat::Bgra8Unorm),
            wgpu::TextureFormat::Bgra8UnormSrgb
        );
        assert_eq!(
            scene_format_for_surface_format(wgpu::TextureFormat::Rgba8Unorm),
            wgpu::TextureFormat::Rgba8UnormSrgb
        );
    }
}
