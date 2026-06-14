use thiserror::Error;

pub use canvas::Canvas;
pub use context::Context;
pub use frame::Frame;
pub use renderer::Renderer;
pub use surface::Surface;

pub mod canvas;
pub mod context;
pub mod frame;
pub mod primitive;
pub mod renderer;
pub mod surface;

pub(crate) fn color_to_wgpu(color: crate::paint::Color) -> wgpu::Color {
    wgpu::Color {
        r: color.r as f64,
        g: color.g as f64,
        b: color.b as f64,
        a: color.a as f64,
    }
}

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, Error)]
pub enum Error {
    #[error(transparent)]
    CreateSurface(#[from] wgpu::CreateSurfaceError),

    #[error(transparent)]
    RequestAdapter(#[from] wgpu::RequestAdapterError),

    #[error(transparent)]
    RequestDevice(#[from] wgpu::RequestDeviceError),

    #[error("surface could not be configured")]
    ConfigureFailed,

    #[error("surface was lost")]
    SurfaceLost,

    #[error(transparent)]
    TextPrepare(#[from] glyphon::PrepareError),

    #[error(transparent)]
    TextRender(#[from] glyphon::RenderError),
}
