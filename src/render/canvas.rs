use wgpu::SurfaceTarget;

use crate::paint_geometry::area;
use crate::render;

pub struct Canvas {
    surface: render::Surface,
    physical_area: area::Physical,
    logical_area: area::Logical,
    scale_factor: f32,
    color: wgpu::Color,
}

pub struct Options {
    pub area: area::Physical,
    pub scale_factor: f32,
    pub color: wgpu::Color,
}

impl Canvas {
    pub fn new(
        options: Options,
        render_context: &render::Context,
        target: impl Into<SurfaceTarget<'static>>,
    ) -> render::Result<Self> {
        let physical_area = options.area;
        let logical_area = physical_area.to_logical(options.scale_factor);
        let surface = render::Surface::new(physical_area, render_context, target)?;

        Ok(Self {
            surface,
            physical_area,
            logical_area,
            scale_factor: options.scale_factor,
            color: options.color,
        })
    }

    pub fn surface(&self) -> &render::Surface {
        &self.surface
    }

    pub fn physical_area(&self) -> area::Physical {
        self.physical_area
    }

    pub fn logical_area(&self) -> area::Logical {
        self.logical_area
    }

    pub fn scale_factor(&self) -> f32 {
        self.scale_factor
    }

    pub fn color(&self) -> wgpu::Color {
        self.color
    }

    pub fn resize(
        &mut self,
        render_context: &render::Context,
        area: area::Physical,
        scale_factor: f32,
    ) {
        self.physical_area = area;
        self.scale_factor = scale_factor;
        self.logical_area = area.to_logical(scale_factor);
        self.surface.resize(render_context, area);
    }

    pub fn draw(
        &mut self,
        render_context: &render::Context,
        encode: impl FnOnce(&mut wgpu::CommandEncoder, &render::Frame),
    ) -> render::Result<render::frame::SurfaceReport> {
        Ok(self.surface.render(render_context, encode)?)
    }
}
