use thiserror::Error;

pub use canvas::Canvas;
pub use context::Context;
pub use frame::Frame;
pub use renderer::Renderer;
pub use surface::Surface;

mod backdrop;
mod batch;
pub mod canvas;
pub mod context;
pub mod frame;
pub mod primitive;
mod quad;
pub mod renderer;
pub mod surface;
mod text_renderer;

pub fn color_to_wgpu(color: crate::paint::Color) -> wgpu::Color {
    wgpu::Color {
        r: color.r as f64,
        g: color.g as f64,
        b: color.b as f64,
        a: color.a as f64,
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Scissor {
    x: u32,
    y: u32,
    width: u32,
    height: u32,
}

impl Scissor {
    pub fn new(x: u32, y: u32, width: u32, height: u32) -> Option<Self> {
        if width == 0 || height == 0 {
            None
        } else {
            Some(Self {
                x,
                y,
                width,
                height,
            })
        }
    }

    pub fn x(self) -> u32 {
        self.x
    }

    pub fn y(self) -> u32 {
        self.y
    }

    pub fn width(self) -> u32 {
        self.width
    }

    pub fn height(self) -> u32 {
        self.height
    }
}

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, Error)]
pub enum Error {
    #[error(transparent)]
    Context(#[from] context::Error),

    #[error(transparent)]
    Surface(#[from] surface::Error),

    #[error(transparent)]
    Text(#[from] text_renderer::Error),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn surface_error_maps_through_render_facade() {
        let error = Error::from(surface::Error::NoSurfaceConfiguration);

        assert!(matches!(
            error,
            Error::Surface(surface::Error::NoSurfaceConfiguration)
        ));
    }

    #[test]
    fn text_prepare_error_maps_through_render_facade() {
        let error = Error::from(text_renderer::Error::from(glyphon::PrepareError::AtlasFull));

        assert!(matches!(
            error,
            Error::Text(text_renderer::Error::Prepare(
                glyphon::PrepareError::AtlasFull
            ))
        ));
    }

    #[test]
    fn text_render_error_maps_through_render_facade() {
        let error = Error::from(text_renderer::Error::from(
            glyphon::RenderError::RemovedFromAtlas,
        ));

        assert!(matches!(
            error,
            Error::Text(text_renderer::Error::Render(
                glyphon::RenderError::RemovedFromAtlas
            ))
        ));
    }
}
