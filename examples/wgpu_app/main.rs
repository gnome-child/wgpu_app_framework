use wgpu_l3::{native, render};
use winit::{
    application::ApplicationHandler,
    event::WindowEvent,
    event_loop::{ActiveEventLoop, EventLoop},
};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("debug")).init();

    let event_loop = EventLoop::new()?;
    let mut app = App::default();

    event_loop.run_app(&mut app)?;

    Ok(())
}

#[derive(Default)]
struct App {
    render_context: Option<render::Context>,
    renderer: Option<render::Renderer>,
    window: Option<native::Window>,
}

impl ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if self.window.is_some() {
            return;
        }

        let render_context = pollster::block_on(render::Context::new(render::context::Options {
            device_label: "wgpu_l3 device",
            backends: wgpu::Backends::all(),
            power_preference: wgpu::PowerPreference::HighPerformance,
            force_fallback_adapter: false,
            required_features: wgpu::Features::empty(),
            required_limits: wgpu::Limits::default(),
        }))
        .expect("failed to create render context");

        let window = native::Window::new(
            native::window::Options {
                title: "window",
                outer_area: wgpu_l3::geometry::area::physical(512, 512),
                canvas_color: wgpu::Color::RED,
            },
            &render_context,
            &mut self.renderer,
            event_loop,
        )
        .expect("failed to open window");

        let mut renderer =
            render::Renderer::new(&render_context, window.canvas().surface().config().format);

        self.render_context = Some(render_context);
        self.renderer = Some(renderer);
        self.window = Some(window);
    }

    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        window_id: winit::window::WindowId,
        event: WindowEvent,
    ) {
        let (Some(render_context), Some(window), Some(renderer)) = (
            self.render_context.as_ref(),
            self.window.as_mut(),
            self.renderer.as_mut(),
        ) else {
            return;
        };

        if window.id() != window_id {
            return;
        }

        window
            .handle_event(render_context, renderer, event_loop, event)
            .expect("failed to handle window event");
    }
}
