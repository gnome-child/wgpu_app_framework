use crate::geometry::area;
use crate::{render, scene};

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
    pub color: scene::Color,
    pub composite_alpha: render::surface::CompositeAlphaPreference,
}

impl Canvas {
    pub fn new(
        options: Options,
        render_context: &render::Context,
        target: impl render::surface::WindowTarget,
    ) -> render::Result<Self> {
        let physical_area = options.area;
        let logical_area = physical_area.to_logical(options.scale_factor);
        let surface = render::Surface::new(
            physical_area,
            render_context,
            target,
            options.composite_alpha,
        )?;

        Ok(Self {
            surface,
            physical_area,
            logical_area,
            scale_factor: options.scale_factor,
            color: render::surface_color(options.color),
        })
    }

    pub(crate) unsafe fn new_unsafe(
        options: Options,
        render_context: &render::Context,
        target: render::surface::Target,
    ) -> render::Result<Self> {
        let physical_area = options.area;
        let logical_area = physical_area.to_logical(options.scale_factor);
        let surface = unsafe {
            render::Surface::new_unsafe(
                physical_area,
                render_context,
                target,
                options.composite_alpha,
            )?
        };

        Ok(Self {
            surface,
            physical_area,
            logical_area,
            scale_factor: options.scale_factor,
            color: render::surface_color(options.color),
        })
    }

    pub fn surface(&self) -> &render::Surface {
        &self.surface
    }

    pub(in crate::render) fn composite_alpha_mode(&self) -> wgpu::CompositeAlphaMode {
        self.surface.config().alpha_mode
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

    pub(in crate::render) fn color(&self) -> wgpu::Color {
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
    ) -> render::Result<render::surface::SurfaceReport> {
        Ok(self.surface.render(render_context, encode)?)
    }
}
