use std::sync::Arc;

use winit::{event_loop::ActiveEventLoop, window::WindowAttributes};

use crate::geometry::area;
use crate::native;
use crate::render;
use crate::window;

pub type Handle = Arc<winit::window::Window>;
pub type Id = window::Id;

pub struct Window {
    id: Id,
    handle: Handle,
    canvas: render::Canvas,
}

pub struct Options {
    pub title: String,
    pub inner_area: area::Physical,
    pub canvas_color: wgpu::Color,
}

impl Window {
    pub fn new(
        id: Id,
        options: Options,
        render_context: &render::Context,
        event_loop: &ActiveEventLoop,
    ) -> native::Result<Self> {
        let mut window_attributes = WindowAttributes::default()
            .with_title(options.title)
            .with_visible(false);

        window_attributes = window_attributes.with_inner_size(winit::dpi::PhysicalSize::new(
            options.inner_area.width(),
            options.inner_area.height(),
        ));

        let handle = Arc::new(event_loop.create_window(window_attributes)?);

        let canvas = render::Canvas::new(
            render::canvas::Options {
                area: area::physical(handle.inner_size().width, handle.inner_size().height),
                scale_factor: handle.scale_factor() as f32,
                color: options.canvas_color,
            },
            render_context,
            handle.clone(),
        )?;

        Ok(Self { id, handle, canvas })
    }

    pub fn id(&self) -> Id {
        self.id
    }

    pub fn raw_id(&self) -> winit::window::WindowId {
        self.handle.id()
    }

    pub fn request_redraw(&self) {
        self.handle.request_redraw();
    }

    pub fn inner_area(&self) -> area::Physical {
        area::physical(
            self.handle.inner_size().width,
            self.handle.inner_size().height,
        )
    }

    pub fn scale_factor(&self) -> f64 {
        self.handle.scale_factor()
    }

    pub fn set_title(&self, title: &str) {
        self.handle.set_title(title);
    }

    pub fn set_visibility(&self, visible: bool) {
        self.handle.set_visible(visible);
    }

    pub fn hide_cursor(&self, hide_cursor: bool) {
        self.handle.set_cursor_visible(!hide_cursor);
    }

    pub fn has_focus(&self) -> bool {
        self.handle.has_focus()
    }

    pub fn canvas(&self) -> &render::Canvas {
        &self.canvas
    }

    pub fn canvas_mut(&mut self) -> &mut render::Canvas {
        &mut self.canvas
    }

    pub fn resize(
        &mut self,
        render_context: &render::Context,
        area: area::Physical,
        scale_factor: f32,
    ) {
        self.canvas.resize(render_context, area, scale_factor);
    }
}
