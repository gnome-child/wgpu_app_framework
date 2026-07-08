use wgpu::SurfaceTarget;

use crate::paint;
use crate::render;

use thiserror::Error;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, Error)]
pub enum Error {
    #[error(transparent)]
    Create(#[from] wgpu::CreateSurfaceError),

    #[error("surface could not be configured")]
    NoSurfaceConfiguration,

    #[error("surface was lost")]
    Lost,
}

pub struct Surface {
    inner: wgpu::Surface<'static>,
    config: wgpu::SurfaceConfiguration,
    ready: bool,
}

impl Surface {
    pub fn new(
        area: paint::area::Physical,
        render_context: &render::Context,
        target: impl Into<SurfaceTarget<'static>>,
    ) -> Result<Self> {
        log::debug!(
            "creating render surface: {}x{} physical",
            area.width(),
            area.height()
        );
        let inner = render_context.instance().create_surface(target)?;

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

        config.present_mode = [
            wgpu::PresentMode::Mailbox,
            wgpu::PresentMode::Immediate,
            wgpu::PresentMode::FifoRelaxed,
            wgpu::PresentMode::Fifo,
        ]
        .into_iter()
        .find(|mode| capabilities.present_modes.contains(mode))
        .unwrap_or(config.present_mode);

        inner.configure(render_context.device(), &config);
        log::debug!(
            "configured render surface: {}x{}, format={:?}, present_mode={:?}",
            config.width,
            config.height,
            config.format,
            config.present_mode
        );

        Ok(Self {
            inner,
            config,
            ready: false,
        })
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
    }

    pub fn acquire_frame(&self, render_context: &render::Context) -> Result<render::FrameOutcome> {
        use wgpu::CurrentSurfaceTexture::*;

        match self.inner.get_current_texture() {
            Success(surface_texture) => Ok(render::FrameOutcome::Acquired(render::Frame::new(
                surface_texture,
            ))),
            Suboptimal(surface_texture) => {
                log::debug!("acquired suboptimal surface texture; reconfiguring surface");
                self.reconfigure(render_context);
                Ok(render::FrameOutcome::Acquired(render::Frame::new(
                    surface_texture,
                )))
            }
            Outdated => {
                log::debug!(
                    "surface texture is outdated; reconfiguring surface and skipping frame"
                );
                self.reconfigure(render_context);
                Ok(render::FrameOutcome::Skipped)
            }
            Timeout => {
                log::debug!("surface texture acquisition timed out; skipping frame");
                Ok(render::FrameOutcome::Skipped)
            }
            Occluded => {
                log::debug!("surface is occluded; skipping frame");
                Ok(render::FrameOutcome::Skipped)
            }
            Validation => {
                log::warn!(
                    "surface texture acquisition returned validation status; skipping frame"
                );
                Ok(render::FrameOutcome::Skipped)
            }
            Lost => {
                log::error!("surface was lost");
                Err(Error::Lost)
            }
        }
    }

    pub fn render(
        &mut self,
        render_context: &render::Context,
        encode: impl FnOnce(&mut wgpu::CommandEncoder, &render::Frame),
    ) -> Result<()> {
        let outcome = self.acquire_frame(render_context)?;

        use render::FrameOutcome::*;
        let frame = match outcome {
            Acquired(frame) => frame,
            Skipped => return Ok(()),
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
        self.ready = true;

        Ok(())
    }

    pub fn reconfigure(&self, render_context: &render::Context) {
        log::debug!(
            "reconfiguring render surface: {}x{}, format={:?}, present_mode={:?}",
            self.config.width,
            self.config.height,
            self.config.format,
            self.config.present_mode
        );
        self.inner.configure(render_context.device(), &self.config);
    }
}
