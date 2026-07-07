use wgpu::SurfaceTarget;

use crate::paint_geometry;
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
        area: paint_geometry::PhysicalArea,
        render_context: &render::Context,
        target: impl Into<SurfaceTarget<'static>>,
    ) -> Result<Self> {
        let inner = render_context.instance().create_surface(target)?;

        let Some(mut config) = inner.get_default_config(
            render_context.adapter(),
            area.width().max(1),
            area.height().max(1),
        ) else {
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

        Ok(Self {
            inner,
            config,
            ready: false,
        })
    }

    pub fn config(&self) -> &wgpu::SurfaceConfiguration {
        &self.config
    }

    pub fn resize(&mut self, render_context: &render::Context, area: paint_geometry::PhysicalArea) {
        let area = area.clamp_min(1);

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
                self.reconfigure(render_context);
                Ok(render::FrameOutcome::Acquired(render::Frame::new(
                    surface_texture,
                )))
            }
            Outdated => {
                self.reconfigure(render_context);
                Ok(render::FrameOutcome::Skipped)
            }
            Timeout | Occluded | Validation => Ok(render::FrameOutcome::Skipped),
            Lost => Err(Error::Lost),
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
        self.inner.configure(render_context.device(), &self.config);
    }
}
