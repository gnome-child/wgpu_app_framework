use std::sync::Arc;

use winit::{event::WindowEvent, event_loop::ActiveEventLoop, window::WindowAttributes};

pub use winit::window::WindowId as Id;

use crate::geometry::Rect;
use crate::geometry::area;
use crate::geometry::point;
use crate::geometry::rect;
use crate::native;
use crate::paint;
use crate::render;

pub type Handle = Arc<winit::window::Window>;

pub struct Window {
    handle: Handle,
    canvas: render::Canvas,
}

pub struct Options {
    pub title: &'static str,
    pub outer_area: area::Physical,
    pub canvas_color: wgpu::Color,
}

impl Window {
    pub fn new(
        options: Options,
        render_context: &render::Context,
        renderer: &mut render::Renderer,
        event_loop: &ActiveEventLoop,
    ) -> native::Result<Self> {
        let mut window_attributes = WindowAttributes::default()
            .with_title(options.title)
            .with_visible(false);

        window_attributes = window_attributes.with_inner_size(winit::dpi::PhysicalSize::new(
            options.outer_area.width(),
            options.outer_area.height(),
        ));

        let handle = Arc::new(event_loop.create_window(window_attributes)?);

        let mut canvas = render::Canvas::new(
            render::canvas::Options {
                area: area::physical(handle.inner_size().width, handle.inner_size().height),
                scale_factor: handle.scale_factor() as f32,
                color: options.canvas_color,
            },
            render_context,
            handle.clone(),
        )?;

        use render::frame::Status::*;

        match renderer.clear(render_context, &mut canvas)? {
            Presented => {
                handle.set_visible(true);
            }

            Skipped(reason) => {
                log::warn!("initial frame was skipped: {:#?}", reason);

                handle.set_visible(true);
            }
        }

        Ok(Self { handle, canvas })
    }

    pub fn id(&self) -> Id {
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

    pub fn handle_event(
        &mut self,
        render_context: &render::Context,
        renderer: &mut render::Renderer,
        event_loop: &ActiveEventLoop,
        event: WindowEvent,
    ) -> native::Result<()> {
        use winit::event::WindowEvent::*;

        match event {
            CloseRequested => event_loop.exit(),

            Resized(size) => {
                self.canvas.resize(
                    render_context,
                    area::physical(size.width, size.height),
                    self.handle.scale_factor() as f32,
                );

                self.handle.request_redraw();
            }

            ScaleFactorChanged { scale_factor, .. } => {
                let inner_size = self.handle.inner_size();

                self.canvas.resize(
                    render_context,
                    area::physical(inner_size.width, inner_size.height),
                    scale_factor as f32,
                );

                self.handle.request_redraw();
            }

            RedrawRequested => {
                let quad = paint::Quad {
                    rect: Rect {
                        origin: point::logical(0.0, 0.0),
                        area: self.canvas.logical_area(),
                        radius: rect::Radius::none(),
                    },
                    style: paint::Style {
                        fill: Some(paint::Fill::Brush(paint::Brush::Solid(paint::Color::RED))),
                        stroke: None,
                        tint: None,
                    },
                };

                use render::frame::Status::*;
                match renderer.draw(render_context, &mut self.canvas, &[quad])? {
                    Presented => {}

                    Skipped(reason) => {
                        log::warn!("clear pass was skipped: {:#?}", reason);
                        self.handle.request_redraw();
                    }
                }

                // TODO: actually render things
                // use render::frame::Status::*;
                // match renderer.clear(render_context, &mut self.canvas)? {
                //     Presented => {}

                //     Skipped(reason) => {
                //         log::warn!("clear pass was skipped: {:#?}", reason);

                //         self.handle.request_redraw();
                //     }
                // }
            }
            _ => {}
        }

        Ok(())
    }
}
