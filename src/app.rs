use std::collections::HashMap;

use thiserror::Error;
use winit::{
    application::ApplicationHandler,
    event::WindowEvent,
    event_loop::{ActiveEventLoop, EventLoop},
};

use crate::geometry::{area, point};
use crate::{native, paint, render, ui, window};

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, Error)]
pub enum Error {
    #[error(transparent)]
    EventLoop(#[from] winit::error::EventLoopError),

    #[error(transparent)]
    Native(#[from] native::Error),

    #[error(transparent)]
    Render(#[from] render::Error),
}

pub trait Application {
    fn started(&mut self, _cx: &mut Context<'_>) {}

    fn window_event(&mut self, _cx: &mut Context<'_>, _window: window::Id, _event: ui::Event) {}

    fn redraw(&mut self, _cx: &mut Context<'_>, _window: window::Id, _scene: &mut paint::Scene) {}
}

pub fn run<A: Application>(app: A) -> Result<()> {
    let event_loop = EventLoop::new()?;
    let mut runtime = Runtime::new(app);

    event_loop.run_app(&mut runtime)?;

    if let Some(error) = runtime.error {
        return Err(error);
    }

    Ok(())
}

pub struct Context<'a> {
    render_context: &'a render::Context,
    renderer: &'a mut Option<render::Renderer>,
    windows: &'a mut HashMap<window::Id, native::Window>,
    event_loop: &'a ActiveEventLoop,
}

impl Context<'_> {
    pub fn open_window(&mut self, options: window::Options) -> window::Id {
        self.try_open_window(options)
            .expect("failed to open framework window")
    }

    pub fn try_open_window(&mut self, options: window::Options) -> Result<window::Id> {
        let native_options = native::window::Options {
            title: options.title,
            inner_area: options.inner_area,
            canvas_color: render::color_to_wgpu(options.canvas_color),
        };

        let mut native_window =
            native::Window::new(native_options, self.render_context, self.event_loop)?;

        if self.renderer.is_none() {
            let format = native_window.canvas().surface().config().format;
            *self.renderer = Some(render::Renderer::new(self.render_context, format));
        }

        let renderer = self
            .renderer
            .as_mut()
            .expect("renderer should be initialized after opening a window");

        use render::frame::Status::*;
        match renderer.clear(self.render_context, native_window.canvas_mut())? {
            Presented => {}
            Skipped(reason) => {
                log::warn!("initial frame was skipped: {:#?}", reason);
            }
        }

        native_window.set_visibility(true);
        native_window.request_redraw();

        let id = native_window.id();
        self.windows.insert(id, native_window);

        Ok(id)
    }

    pub fn request_redraw(&self, window: window::Id) {
        if let Some(window) = self.windows.get(&window) {
            window.request_redraw();
        }
    }

    pub fn close_window(&mut self, window: window::Id) {
        self.windows.remove(&window);

        if self.windows.is_empty() {
            self.event_loop.exit();
        }
    }

    pub fn window_logical_area(&self, window: window::Id) -> Option<area::Logical> {
        self.windows
            .get(&window)
            .map(|window| window.canvas().logical_area())
    }

    pub fn window_physical_area(&self, window: window::Id) -> Option<area::Physical> {
        self.windows.get(&window).map(native::Window::inner_area)
    }
}

struct Runtime<A> {
    app: A,
    render_context: Option<render::Context>,
    renderer: Option<render::Renderer>,
    windows: HashMap<window::Id, native::Window>,
    started: bool,
    error: Option<Error>,
}

impl<A> Runtime<A> {
    fn new(app: A) -> Self {
        Self {
            app,
            render_context: None,
            renderer: None,
            windows: HashMap::new(),
            started: false,
            error: None,
        }
    }

    fn fail(&mut self, event_loop: &ActiveEventLoop, error: Error) {
        self.error = Some(error);
        event_loop.exit();
    }

    fn render_options() -> render::context::Options {
        render::context::Options {
            device_label: "wgpu_l3 device",
            backends: wgpu::Backends::all(),
            power_preference: wgpu::PowerPreference::HighPerformance,
            force_fallback_adapter: false,
            required_features: wgpu::Features::empty(),
            required_limits: wgpu::Limits::default(),
        }
    }
}

impl<A: Application> Runtime<A> {
    fn dispatch_window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        window: window::Id,
        event: ui::Event,
    ) {
        let Some(render_context) = self.render_context.as_ref() else {
            return;
        };

        let mut cx = Context {
            render_context,
            renderer: &mut self.renderer,
            windows: &mut self.windows,
            event_loop,
        };

        self.app.window_event(&mut cx, window, event);
    }

    fn redraw_window(&mut self, event_loop: &ActiveEventLoop, window: window::Id) {
        let Some(render_context) = self.render_context.as_ref() else {
            return;
        };

        let mut scene = paint::Scene::new();

        {
            let mut cx = Context {
                render_context,
                renderer: &mut self.renderer,
                windows: &mut self.windows,
                event_loop,
            };

            self.app.redraw(&mut cx, window, &mut scene);
        }

        let Some(native_window) = self.windows.get_mut(&window) else {
            return;
        };

        if self.renderer.is_none() {
            let format = native_window.canvas().surface().config().format;
            self.renderer = Some(render::Renderer::new(render_context, format));
        }

        let renderer = self
            .renderer
            .as_mut()
            .expect("renderer should be initialized before redraw");

        use render::frame::Status::*;
        match renderer.draw(render_context, native_window.canvas_mut(), &scene) {
            Ok(Presented) => {}
            Ok(Skipped(reason)) => {
                log::warn!("render pass was skipped: {:#?}", reason);
                native_window.request_redraw();
            }
            Err(error) => {
                self.fail(event_loop, error.into());
            }
        }
    }

    fn close_window(&mut self, event_loop: &ActiveEventLoop, window: window::Id) {
        self.windows.remove(&window);

        if self.windows.is_empty() {
            event_loop.exit();
        }
    }
}

impl<A: Application> ApplicationHandler for Runtime<A> {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if self.render_context.is_none() {
            match pollster::block_on(render::Context::new(Self::render_options())) {
                Ok(render_context) => {
                    self.render_context = Some(render_context);
                }
                Err(error) => {
                    self.fail(event_loop, error.into());
                    return;
                }
            }
        }

        if self.started {
            return;
        }

        self.started = true;

        let Some(render_context) = self.render_context.as_ref() else {
            return;
        };

        let mut cx = Context {
            render_context,
            renderer: &mut self.renderer,
            windows: &mut self.windows,
            event_loop,
        };

        self.app.started(&mut cx);
    }

    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        window: window::Id,
        event: WindowEvent,
    ) {
        if !self.windows.contains_key(&window) {
            return;
        }

        match event {
            WindowEvent::CloseRequested => {
                self.dispatch_window_event(event_loop, window, ui::Event::CloseRequested);

                if self.windows.contains_key(&window) {
                    self.close_window(event_loop, window);
                }
            }
            WindowEvent::Resized(size) => {
                let area = area::physical(size.width, size.height);
                let Some(render_context) = self.render_context.as_ref() else {
                    return;
                };

                let Some(native_window) = self.windows.get_mut(&window) else {
                    return;
                };

                let scale_factor = native_window.scale_factor() as f32;
                native_window.resize(render_context, area, scale_factor);
                native_window.request_redraw();

                self.dispatch_window_event(
                    event_loop,
                    window,
                    ui::Event::Resized { area, scale_factor },
                );
            }
            WindowEvent::ScaleFactorChanged { scale_factor, .. } => {
                let Some(render_context) = self.render_context.as_ref() else {
                    return;
                };

                let Some(native_window) = self.windows.get_mut(&window) else {
                    return;
                };

                let area = native_window.inner_area();
                let scale_factor = scale_factor as f32;
                native_window.resize(render_context, area, scale_factor);
                native_window.request_redraw();

                self.dispatch_window_event(
                    event_loop,
                    window,
                    ui::Event::ScaleFactorChanged { scale_factor },
                );
            }
            WindowEvent::Focused(focused) => {
                self.dispatch_window_event(event_loop, window, ui::Event::Focused(focused));
            }
            WindowEvent::CursorMoved { position, .. } => {
                let Some(native_window) = self.windows.get(&window) else {
                    return;
                };

                let position = point::physical(position.x as f32, position.y as f32)
                    .to_logical(native_window.scale_factor() as f32);

                self.dispatch_window_event(event_loop, window, ui::Event::CursorMoved { position });
            }
            WindowEvent::RedrawRequested => {
                self.redraw_window(event_loop, window);
            }
            _ => {}
        }
    }
}
