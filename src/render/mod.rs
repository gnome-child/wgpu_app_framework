use thiserror::Error;

pub(crate) use canvas::{Canvas, Options as CanvasOptions};
pub(crate) use context::{Context, Options as ContextOptions};
pub(crate) use frame::{Frame, Outcome as FrameOutcome};
pub(in crate::render) use primitive::Vertex;
pub(crate) use renderer::Renderer;
pub(crate) use surface::{PresentTiming, Surface};

mod batch;
mod canvas;
mod context;
mod filter;
mod frame;
mod material;
mod primitive;
mod quad;
mod renderer;
mod silhouette;
mod surface;
mod text_renderer;

pub(crate) fn color_to_wgpu(color: crate::paint::Color) -> wgpu::Color {
    wgpu::Color {
        r: color.r as f64,
        g: color.g as f64,
        b: color.b as f64,
        a: color.a as f64,
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) struct Viewport {
    physical_area: crate::paint::area::Physical,
    logical_area: crate::paint::area::Logical,
    scale_factor: f32,
}

impl Viewport {
    pub(crate) fn from_canvas(canvas: &Canvas) -> Self {
        Self {
            physical_area: canvas.physical_area(),
            logical_area: canvas.logical_area(),
            scale_factor: canvas.scale_factor(),
        }
    }

    pub(crate) fn from_logical_area(
        logical_area: crate::paint::area::Logical,
        scale_factor: f32,
    ) -> Self {
        let physical_area = logical_area.to_physical(scale_factor).clamp_min(1);
        Self {
            physical_area,
            logical_area,
            scale_factor,
        }
    }

    pub(crate) fn physical_area(self) -> crate::paint::area::Physical {
        self.physical_area
    }

    pub(crate) fn logical_area(self) -> crate::paint::area::Logical {
        self.logical_area
    }

    pub(crate) fn scale_factor(self) -> f32 {
        self.scale_factor
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct Scissor {
    x: u32,
    y: u32,
    width: u32,
    height: u32,
}

impl Scissor {
    pub(crate) fn new(x: u32, y: u32, width: u32, height: u32) -> Option<Self> {
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

    pub(crate) fn x(self) -> u32 {
        self.x
    }

    pub(crate) fn y(self) -> u32 {
        self.y
    }

    pub(crate) fn width(self) -> u32 {
        self.width
    }

    pub(crate) fn height(self) -> u32 {
        self.height
    }
}

pub(crate) type Result<T> = std::result::Result<T, Error>;

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

    #[test]
    fn logical_viewport_preserves_requested_scene_area() {
        let viewport = Viewport::from_logical_area(crate::paint::area::logical(10.4, 20.6), 2.0);

        assert_eq!(
            viewport.physical_area(),
            crate::paint::area::physical(21, 41)
        );
        assert_eq!(
            viewport.logical_area(),
            crate::paint::area::logical(10.4, 20.6)
        );
    }
}
