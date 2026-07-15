use crate::geometry::area;
use thiserror::Error;

pub(crate) use crate::paint::Scene;
pub(crate) use canvas::Canvas;
pub(crate) use context::Context;
pub(crate) use frame::Frame;
pub(in crate::render) use primitive::Vertex;
pub(crate) use renderer::Renderer;
pub(crate) use report::DrawStats;
pub use report::{RenderReport, RendererEnvironment};
pub(crate) use surface::Surface;

mod alpha;
mod batch;
pub(crate) mod canvas;
mod color;
pub(crate) mod context;
#[cfg(feature = "renderer-debug")]
pub mod debug;
mod filter;
mod frame;
mod material;
mod popup_pack;
mod primitive;
mod quad;
mod renderer;
mod report;
mod retained;
pub(crate) mod scene;
mod silhouette;
pub(crate) mod surface;
mod text_renderer;

pub(crate) fn color_to_wgpu(color: crate::paint::Color) -> wgpu::Color {
    wgpu::Color {
        r: color.r as f64,
        g: color.g as f64,
        b: color.b as f64,
        a: color.a as f64,
    }
}

fn surface_color(color: crate::scene::Color) -> wgpu::Color {
    color_to_wgpu(self::color::paint_color(color))
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) struct Viewport {
    physical_area: area::Physical,
    logical_area: area::Logical,
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

    pub(crate) fn from_logical_area(logical_area: area::Logical, scale_factor: f32) -> Self {
        let physical_area = logical_area.to_physical(scale_factor).clamp_min(1);
        Self {
            physical_area,
            logical_area,
            scale_factor,
        }
    }

    pub(crate) fn physical_area(self) -> area::Physical {
        self.physical_area
    }

    pub(crate) fn logical_area(self) -> area::Logical {
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

    #[error("retained scene properties do not belong to the supplied commit")]
    RetainedSceneContract,

    #[error(
        "retained {kind} layer physical size {width}x{height} exceeds the device 2D texture limit {limit}"
    )]
    RetainedLayerLimit {
        kind: &'static str,
        width: u32,
        height: u32,
        limit: u32,
    },
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
        let viewport = Viewport::from_logical_area(area::logical(10.4, 20.6), 2.0);

        assert_eq!(viewport.physical_area(), area::physical(21, 41));
        assert_eq!(viewport.logical_area(), area::logical(10.4, 20.6));
    }
}
