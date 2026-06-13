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
}
