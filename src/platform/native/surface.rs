use crate::geometry::area;
use std::time::Instant;

use crate::{diagnostics, render};

use super::super::{NativeError, Window};
use super::window::{InitialSize, Options, Window as NativeWindow};
use super::{Native, NativeContext, PopupPrewarmState};
use crate::{shell, window as app_window};

impl Native {
    pub(in crate::platform::native) fn ensure_context(&mut self) -> Result<(), NativeError> {
        if self.context.is_none() {
            log::debug!("initializing native render context");
            let explicit = wgpu::Backends::from_env();
            let attempts = native_backend_attempts(explicit);
            let mut last_error = None;
            for (index, backends) in attempts.iter().copied().enumerate() {
                match pollster::block_on(render::Context::new(render_context_options(backends))) {
                    Ok(context) => {
                        if index > 0 {
                            log::warn!(
                                target: "wgpu_l3::native_popup",
                                "DX12-first context initialization failed; continuing with backend set {backends:?} and non-tenancy material realization"
                            );
                        }
                        self.context = Some(context);
                        break;
                    }
                    Err(error) => {
                        log::warn!(
                            target: "wgpu_l3::native_popup",
                            "native graphics attempt {index} with {backends:?} failed: {error}"
                        );
                        last_error = Some(error);
                    }
                }
            }
            if self.context.is_none() {
                return Err(NativeError::Render(
                    last_error.expect("native backend attempts are never empty"),
                ));
            }
        }

        Ok(())
    }

    pub(in crate::platform::native) fn ensure_renderer(&mut self, format: wgpu::TextureFormat) {
        if self.renderers.contains_key(&format) {
            return;
        }

        log::debug!("initializing native renderer for surface format {format:?}");
        let context = self
            .context
            .as_ref()
            .expect("render context should exist before creating renderer");
        self.renderers
            .insert(format, render::Renderer::new(context, format));
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
            inner_size: InitialSize::Logical(area::Logical::from_size(window.size())),
            kind: window.kind(),
            owner: None,
            popup_presentation_mode: None,
        };
        let handle = NativeWindow::open(native_options, context.event_loop())?;
        let render_context = self
            .context
            .as_ref()
            .expect("render context should exist before creating window canvas");
        let inner_size = handle.inner_size();
        let canvas = render::Canvas::new(
            render::CanvasOptions {
                area: area::physical(inner_size.width, inner_size.height).clamp_min(1),
                scale_factor: handle.scale_factor() as f32,
                color: render::color_to_wgpu(super::color::paint_color(window.canvas_color())),
                composite_alpha: render::CompositeAlphaPreference::Default,
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
        let render_format = render_format_for_canvas(native_window.canvas());
        self.ensure_renderer(render_format);

        let context = self
            .context
            .as_ref()
            .expect("render context should exist before clearing");
        let renderer = self
            .renderers
            .get_mut(&render_format)
            .expect("renderer should exist before clearing");
        renderer.clear(context, native_window.canvas_mut())?;

        Ok(())
    }

    pub(in crate::platform::native) fn present_native(
        &mut self,
        presentation: &shell::Presentation,
    ) -> Result<diagnostics::RenderReport, NativeError> {
        let window = presentation.window();
        self.sync_window_surface(window)?;
        let render_format = {
            let native_window = self.windows.get(&window).ok_or_else(|| {
                log::error!("cannot present missing native window: {window:?}");
                NativeError::MissingWindow { window }
            })?;
            render_format_for_canvas(native_window.canvas())
        };
        self.ensure_renderer(render_format);

        let context = self
            .context
            .as_ref()
            .expect("render context should exist before presenting");
        let renderer = self
            .renderers
            .get_mut(&render_format)
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
        let encode_submit_present = report
            .present_timing
            .map(render::PresentTiming::encode_submit_present)
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

        let presented = report.present_timing.is_some();
        if presented
            && context.windows_popup_composition_supported()
            && !self.popup_prewarm.contains_key(&window)
        {
            self.popup_prewarm.insert(window, PopupPrewarmState::Armed);
        }

        Ok(
            diagnostics::RenderReport::new(acquire_wait, draw, Instant::now())
                .with_presented(presented)
                .with_pipeline_timings(report.batch_prepare, encode_submit_present)
                .with_draw_stats(report.stats)
                .with_group_composites(group_composites)
                .with_filter_pool_entries(filter_layer_pool_entries, filter_scratch_pool_entries),
        )
    }

    pub(in crate::platform::native) fn sync_window_surface(
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

pub(in crate::platform::native) fn render_format_for_canvas(
    canvas: &render::Canvas,
) -> wgpu::TextureFormat {
    let format = canvas.surface().config().format;
    if render::supports_windows_premultiplied_popup_pack(format, canvas.composite_alpha_mode()) {
        render::scene_format_for_surface_format(format)
    } else {
        format
    }
}

fn render_context_options(backends: wgpu::Backends) -> render::ContextOptions {
    render::ContextOptions {
        device_label: "wgpu_l3 device",
        backends,
        power_preference: wgpu::PowerPreference::HighPerformance,
        force_fallback_adapter: false,
        required_features: wgpu::Features::empty(),
        required_limits: wgpu::Limits::default(),
    }
}

fn native_backend_attempts(explicit: Option<wgpu::Backends>) -> Vec<wgpu::Backends> {
    if let Some(explicit) = explicit {
        return vec![explicit];
    }

    #[cfg(target_os = "windows")]
    {
        vec![wgpu::Backends::DX12, wgpu::Backends::all()]
    }
    #[cfg(not(target_os = "windows"))]
    {
        vec![wgpu::Backends::all()]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn explicit_backend_choice_is_the_only_attempt() {
        assert_eq!(
            native_backend_attempts(Some(wgpu::Backends::VULKAN)),
            vec![wgpu::Backends::VULKAN]
        );
    }

    #[test]
    #[cfg(target_os = "windows")]
    fn implicit_windows_policy_earns_tenancy_through_dx12_first() {
        let attempts = native_backend_attempts(None);
        assert_eq!(attempts[0], wgpu::Backends::DX12);
        assert!(attempts[1].contains(wgpu::Backends::VULKAN));
    }
}
