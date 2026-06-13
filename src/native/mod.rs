use thiserror::Error;

pub use window::Window;

pub mod window;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, Error)]
pub enum Error {
    #[error(transparent)]
    NativeWindow(#[from] winit::error::OsError),

    #[error(transparent)]
    Render(#[from] crate::render::Error),
}
