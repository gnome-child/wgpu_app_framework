use crate::{render, scratch::window};
use thiserror::Error as ThisError;

#[derive(Debug, ThisError)]
pub enum NativeError {
    #[error("native window error")]
    Window(#[from] winit::error::OsError),

    #[error("render error")]
    Render(#[from] render::Error),

    #[error("native window is not open: {window:?}")]
    MissingWindow { window: window::Id },
}
