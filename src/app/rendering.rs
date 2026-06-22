use crate::geometry::area;
use crate::{native, paint, render};

pub struct Driver {
    context: Option<render::Context>,
    renderer: Option<render::Renderer>,
}

impl Driver {
    pub fn new() -> Self {
        Self {
            context: None,
            renderer: None,
        }
    }

    pub fn initialize(&mut self) -> render::Result<()> {
        if self.context.is_some() {
            return Ok(());
        }

        self.context = Some(pollster::block_on(render::Context::new(Self::options()))?);
        Ok(())
    }

    pub fn ready(&self) -> bool {
        self.context.is_some()
    }

    pub fn create_window(
        &mut self,
        handle: native::window::Handle,
        canvas_color: paint::Color,
    ) -> render::Result<native::Window> {
        let context = self
            .context
            .as_ref()
            .expect("render context should exist before creating windows");
        let canvas = render::Canvas::new(
            render::canvas::Options {
                area: area::physical(handle.inner_size().width, handle.inner_size().height),
                scale_factor: handle.scale_factor() as f32,
                color: render::color_to_wgpu(canvas_color),
            },
            context,
            handle.clone(),
        )?;

        Ok(native::Window::new(handle, canvas))
    }

    pub fn clear_window(
        &mut self,
        native_window: &mut native::Window,
    ) -> render::Result<render::frame::Status> {
        let format = native_window.canvas().surface().config().format;
        self.ensure_renderer(format);

        let context = self
            .context
            .as_ref()
            .expect("render context should exist before clearing");
        let renderer = self
            .renderer
            .as_mut()
            .expect("renderer should be initialized before clearing");

        renderer.clear(context, native_window.canvas_mut())
    }

    pub fn draw(
        &mut self,
        native_window: &mut native::Window,
        scene: &paint::Scene,
        layer_updates: &[paint::LayerUpdate],
    ) -> render::Result<render::renderer::DrawReport> {
        let format = native_window.canvas().surface().config().format;
        self.ensure_renderer(format);

        let context = self
            .context
            .as_ref()
            .expect("render context should exist before drawing");
        let renderer = self
            .renderer
            .as_mut()
            .expect("renderer should be initialized before drawing");

        renderer.draw(context, native_window.canvas_mut(), scene, layer_updates)
    }

    pub fn resize(
        &self,
        native_window: &mut native::Window,
        area: area::Physical,
        scale_factor: f32,
    ) {
        let context = self
            .context
            .as_ref()
            .expect("render context should exist before resizing");

        native_window.resize(context, area, scale_factor);
    }

    fn ensure_renderer(&mut self, format: wgpu::TextureFormat) {
        if self.renderer.is_some() {
            return;
        }

        let context = self
            .context
            .as_ref()
            .expect("render context should exist before creating renderer");
        self.renderer = Some(render::Renderer::new(context, format));
    }

    fn options() -> render::context::Options {
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

impl Default for Driver {
    fn default() -> Self {
        Self::new()
    }
}
