use std::sync::Arc;

use winit::dpi::{LogicalPosition, LogicalSize};
use winit::{
    event_loop::ActiveEventLoop,
    window::{CursorIcon, WindowAttributes},
};

use crate::geometry::Rect;
use crate::geometry::area;
use crate::native;
use crate::render;
use crate::ui;

pub type Handle = Arc<winit::window::Window>;

pub struct Window {
    handle: Handle,
    canvas: render::Canvas,
}

pub struct Options {
    pub title: String,
    pub inner_area: area::Physical,
}

impl Window {
    pub fn open(options: Options, event_loop: &ActiveEventLoop) -> native::Result<Handle> {
        let mut window_attributes = WindowAttributes::default()
            .with_title(options.title)
            .with_visible(false);

        window_attributes = window_attributes.with_inner_size(winit::dpi::PhysicalSize::new(
            options.inner_area.width(),
            options.inner_area.height(),
        ));

        Ok(Arc::new(event_loop.create_window(window_attributes)?))
    }

    pub fn new(handle: Handle, canvas: render::Canvas) -> Self {
        Self { handle, canvas }
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

    pub fn set_visibility(&self, visible: bool) {
        self.handle.set_visible(visible);
    }

    pub fn set_ime_allowed(&self, allowed: bool) {
        self.handle.set_ime_allowed(allowed);
    }

    pub fn set_ime_cursor_area(&self, rect: Rect) {
        self.handle.set_ime_cursor_area(
            LogicalPosition::new(rect.origin.x() as f64, rect.origin.y() as f64),
            LogicalSize::new(
                rect.area.width().max(1.0) as f64,
                rect.area.height().max(1.0) as f64,
            ),
        );
    }

    pub fn set_cursor(&self, cursor: ui::Cursor) {
        let icon = match cursor {
            ui::Cursor::Default => CursorIcon::Default,
            ui::Cursor::Text => CursorIcon::Text,
        };

        self.handle.set_cursor(icon);
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
