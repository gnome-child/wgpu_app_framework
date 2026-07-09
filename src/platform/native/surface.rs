use std::time::Instant;

use crate::{diagnostics, paint, render};

use super::super::{NativeError, Window};
use super::window::{InitialSize, Options, Window as NativeWindow};
use super::{Native, NativeContext};
use crate::{geometry, shell, window as app_window};

impl Native {
    pub(in crate::platform::native) fn ensure_context(&mut self) -> Result<(), NativeError> {
        if self.context.is_none() {
            log::debug!("initializing native render context");
            self.context = Some(pollster::block_on(render::Context::new(
                render_context_options(),
            ))?);
        }

        Ok(())
    }

    pub(in crate::platform::native) fn ensure_renderer(&mut self, format: wgpu::TextureFormat) {
        if self.renderer.is_some() {
            return;
        }

        log::debug!("initializing native renderer for surface format {format:?}");
        let context = self
            .context
            .as_ref()
            .expect("render context should exist before creating renderer");
        self.renderer = Some(render::Renderer::new(context, format));
    }

    pub(in crate::platform::native) fn create_native_window(
        &mut self,
        context: &NativeContext<'_>,
        window: &Window,
    ) -> Result<NativeWindow, NativeError> {
        self.ensure_context()?;
        log::debug!(
            "opening native window {:?}: title={:?}, size={:?}",
            window.id(),
            window.title(),
            window.size()
        );

        let native_options = Options {
            title: window.title().to_owned(),
            inner_size: InitialSize::Logical(native_logical_area(
                geometry::LogicalArea::from_size(window.size()),
            )),
        };
        let handle = NativeWindow::open(native_options, context.event_loop())?;
        let render_context = self
            .context
            .as_ref()
            .expect("render context should exist before creating window canvas");
        let inner_size = handle.inner_size();
        let canvas = render::Canvas::new(
            render::CanvasOptions {
                area: paint::area::physical(inner_size.width, inner_size.height).clamp_min(1),
                scale_factor: handle.scale_factor() as f32,
                color: render::color_to_wgpu(super::color::paint_color(window.canvas_color())),
            },
            render_context,
            handle.clone(),
        )?;
        log::debug!(
            "created native window {:?}: raw={:?}, inner={}x{}, scale={}",
            window.id(),
            handle.id(),
            inner_size.width,
            inner_size.height,
            handle.scale_factor()
        );

        Ok(NativeWindow::new(handle, canvas))
    }

    pub(in crate::platform::native) fn clear_window(
        &mut self,
        native_window: &mut NativeWindow,
    ) -> Result<(), NativeError> {
        let format = native_window.canvas().surface().config().format;
        self.ensure_renderer(format);

        let context = self
            .context
            .as_ref()
            .expect("render context should exist before clearing");
        let renderer = self
            .renderer
            .as_mut()
            .expect("renderer should exist before clearing");
        renderer.clear(context, native_window.canvas_mut())?;

        Ok(())
    }

    pub(in crate::platform::native) fn present_native(
        &mut self,
        presentation: &shell::Presentation,
    ) -> Result<diagnostics::RenderReport, NativeError> {
        let window = presentation.window();
        let format = self.sync_window_surface(window)?;
        self.ensure_renderer(format);

        let context = self
            .context
            .as_ref()
            .expect("render context should exist before presenting");
        let renderer = self
            .renderer
            .as_mut()
            .expect("renderer should exist before presenting");
        let native_window = self.windows.get_mut(&window).ok_or_else(|| {
            log::error!("cannot present missing native window: {window:?}");
            NativeError::MissingWindow { window }
        })?;
        let scene = super::paint::to_paint_scene_at_scale(
            presentation.scene(),
            native_window.canvas().scale_factor(),
        );

        let draw_started = Instant::now();
        let report = renderer.draw(context, native_window.canvas_mut(), &scene)?;
        let draw = draw_started.elapsed();
        let acquire_wait = report
            .present_timing
            .map(render::PresentTiming::acquire_wait)
            .unwrap_or_default();
        let group_composites = report.stats.group_composites;
        let filter_layer_pool_entries = report.stats.filter_layer_pool_entries;
        let filter_scratch_pool_entries = report.stats.filter_scratch_pool_entries;
        if acquire_wait.as_millis() >= 8 {
            log::debug!(
                "surface acquire wait for window {:?}: {}us",
                window,
                acquire_wait.as_micros()
            );
        }

        Ok(
            diagnostics::RenderReport::new(acquire_wait, draw, Instant::now())
                .with_group_composites(group_composites)
                .with_filter_pool_entries(filter_layer_pool_entries, filter_scratch_pool_entries),
        )
    }

    fn sync_window_surface(
        &mut self,
        window: app_window::Id,
    ) -> Result<wgpu::TextureFormat, NativeError> {
        self.ensure_context()?;
        let native_window = self.windows.get_mut(&window).ok_or_else(|| {
            log::error!("cannot sync surface for missing native window: {window:?}");
            NativeError::MissingWindow { window }
        })?;
        let area = native_window.inner_area().clamp_min(1);
        let scale_factor = native_window.scale_factor() as f32;
        let needs_resize = native_window.canvas().physical_area() != area
            || (native_window.canvas().scale_factor() - scale_factor).abs() > f32::EPSILON;

        if needs_resize {
            log::debug!(
                "syncing native surface {:?}: area={}x{}, scale={}",
                window,
                area.width(),
                area.height(),
                scale_factor
            );
            let context = self
                .context
                .as_ref()
                .expect("render context should exist before resizing");
            native_window.resize(context, area, scale_factor);
        }

        Ok(native_window.canvas().surface().config().format)
    }
}

fn render_context_options() -> render::ContextOptions {
    render::ContextOptions {
        device_label: "wgpu_l3 device",
        backends: wgpu::Backends::all(),
        power_preference: wgpu::PowerPreference::HighPerformance,
        force_fallback_adapter: false,
        required_features: wgpu::Features::empty(),
        required_limits: wgpu::Limits::default(),
    }
}

fn native_logical_area(area: geometry::LogicalArea) -> paint::area::Logical {
    paint::area::logical(area.width(), area.height())
}
