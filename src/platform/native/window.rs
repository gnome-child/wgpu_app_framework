use super::{Native, NativeError};
use crate::window as app_window;
use crate::{paint, render};
use std::sync::Arc;
use winit::dpi::LogicalSize;
use winit::{event_loop::ActiveEventLoop, window::WindowAttributes};

pub(in crate::platform::native) type Handle = Arc<winit::window::Window>;

pub(in crate::platform::native) struct Window {
    handle: Handle,
    canvas: render::Canvas,
}

pub(in crate::platform::native) struct Options {
    pub title: String,
    pub inner_size: InitialSize,
}

pub(in crate::platform::native) enum InitialSize {
    Logical(paint::area::Logical),
}

impl Window {
    pub fn open(
        options: Options,
        event_loop: &ActiveEventLoop,
    ) -> Result<Handle, winit::error::OsError> {
        let mut window_attributes = WindowAttributes::default()
            .with_title(options.title)
            .with_visible(false);

        match options.inner_size {
            InitialSize::Logical(inner_area) => {
                window_attributes = window_attributes.with_inner_size(LogicalSize::new(
                    inner_area.width() as f64,
                    inner_area.height() as f64,
                ));
            }
        }

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

    pub fn inner_area(&self) -> paint::area::Physical {
        paint::area::physical(
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

    pub fn canvas(&self) -> &render::Canvas {
        &self.canvas
    }

    pub fn canvas_mut(&mut self) -> &mut render::Canvas {
        &mut self.canvas
    }

    pub fn resize(
        &mut self,
        render_context: &render::Context,
        area: paint::area::Physical,
        scale_factor: f32,
    ) {
        self.canvas.resize(render_context, area, scale_factor);
    }
}

impl Native {
    pub fn contains(&self, window: app_window::Id) -> bool {
        self.windows.contains_key(&window)
    }

    pub fn window_for_raw(&self, raw: winit::window::WindowId) -> Option<app_window::Id> {
        self.raw_windows.get(&raw).copied()
    }

    pub fn scale_factor(&self, window: app_window::Id) -> Option<f64> {
        self.windows.get(&window).map(Window::scale_factor)
    }

    pub fn is_empty(&self) -> bool {
        self.windows.is_empty()
    }

    pub fn request_redraw(&self, window: app_window::Id) -> Result<(), NativeError> {
        let Some(window) = self.windows.get(&window) else {
            return Err(NativeError::MissingWindow { window });
        };

        window.request_redraw();
        Ok(())
    }

    #[cfg(test)]
    pub fn track_window_for_test(&mut self, raw: winit::window::WindowId, window: app_window::Id) {
        self.raw_windows.insert(raw, window);
    }
}
