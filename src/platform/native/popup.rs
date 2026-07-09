use std::collections::HashSet;
use std::time::Instant;

use crate::{geometry, overlay, paint, render, window as app_window};

use super::surface::native_logical_area;
use super::window::{InitialSize, Options, Window as NativeWindow};
use super::{Native, NativeContext, NativeError, PopupKey};

impl Native {
    pub(in crate::platform::native) fn overlay_capabilities() -> overlay::Capabilities {
        if native_popups_supported() {
            overlay::Capabilities::with_native_popups()
        } else {
            overlay::Capabilities::in_frame_only()
        }
    }

    pub(in crate::platform::native) fn present_popup_overlays(
        &mut self,
        context: &NativeContext<'_>,
        presentations: &[overlay::PopupPresentation],
    ) -> Result<(), NativeError> {
        let active = presentations
            .iter()
            .map(|presentation| PopupKey::new(presentation.parent(), presentation.id()))
            .collect::<HashSet<_>>();
        self.close_stale_popups(&active);

        for presentation in presentations {
            self.present_popup_overlay(context, presentation)?;
        }

        Ok(())
    }

    fn present_popup_overlay(
        &mut self,
        context: &NativeContext<'_>,
        presentation: &overlay::PopupPresentation,
    ) -> Result<(), NativeError> {
        self.ensure_popup_window(context, presentation)?;
        self.configure_popup_window(presentation)?;

        let key = PopupKey::new(presentation.parent(), presentation.id());
        let format = self.sync_popup_surface(key)?;
        self.ensure_renderer(format);

        let render_context = self
            .context
            .as_ref()
            .expect("render context should exist before presenting popup");
        let renderer = self
            .renderer
            .as_mut()
            .expect("renderer should exist before presenting popup");
        let popup = self
            .popups
            .get_mut(&key)
            .expect("popup should exist before presenting");
        let scene = super::paint::to_paint_scene_at_scale(
            presentation.scene(),
            popup.canvas().scale_factor(),
        );

        let draw_started = Instant::now();
        let report = renderer.draw(render_context, popup.canvas_mut(), &scene)?;
        let draw = draw_started.elapsed();
        let acquire_wait = report
            .present_timing
            .map(render::PresentTiming::acquire_wait)
            .unwrap_or_default();
        log::debug!(
            "presented native popup {:?} for parent {:?}: draw={}us acquire={}us groups={}",
            presentation.id(),
            presentation.parent(),
            draw.as_micros(),
            acquire_wait.as_micros(),
            report.stats.group_composites
        );

        popup.set_visibility(true);

        Ok(())
    }

    fn ensure_popup_window(
        &mut self,
        context: &NativeContext<'_>,
        presentation: &overlay::PopupPresentation,
    ) -> Result<(), NativeError> {
        let key = PopupKey::new(presentation.parent(), presentation.id());
        if self.popups.contains_key(&key) {
            return Ok(());
        }

        self.ensure_context()?;
        let parent = self.windows.get(&presentation.parent()).ok_or_else(|| {
            log::error!(
                "cannot create popup {:?} for missing parent {:?}",
                presentation.id(),
                presentation.parent()
            );
            NativeError::MissingWindow {
                window: presentation.parent(),
            }
        })?;
        let native_options = Options {
            title: format!("wgpu_l3 popup {}", presentation.id().as_str()),
            inner_size: InitialSize::Logical(native_logical_area(
                geometry::LogicalArea::from_size(presentation.scene().size()),
            )),
            kind: app_window::Kind::Popup,
            owner: Some(parent.handle()),
        };
        let handle = NativeWindow::open(native_options, context.event_loop())?;
        let render_context = self
            .context
            .as_ref()
            .expect("render context should exist before creating popup canvas");
        let inner_size = handle.inner_size();
        let canvas = render::Canvas::new(
            render::CanvasOptions {
                area: paint::area::physical(inner_size.width, inner_size.height).clamp_min(1),
                scale_factor: handle.scale_factor() as f32,
                color: render::color_to_wgpu(super::color::paint_color(
                    presentation.scene().clear(),
                )),
            },
            render_context,
            handle.clone(),
        )?;
        let popup = NativeWindow::new(handle, canvas);
        log::debug!(
            "created native popup {:?} for parent {:?}: raw={:?}, size={:?}, scale={}",
            presentation.id(),
            presentation.parent(),
            popup.raw_id(),
            presentation.scene().size(),
            popup.scale_factor()
        );

        self.raw_popups.insert(popup.raw_id(), key);
        self.popups.insert(key, popup);

        Ok(())
    }

    fn configure_popup_window(
        &mut self,
        presentation: &overlay::PopupPresentation,
    ) -> Result<(), NativeError> {
        let parent_id = presentation.parent();
        let key = PopupKey::new(parent_id, presentation.id());
        let Some(parent) = self.windows.get(&parent_id) else {
            return Err(NativeError::MissingWindow { window: parent_id });
        };
        let parent_origin = parent
            .handle()
            .outer_position()
            .unwrap_or_else(|_| winit::dpi::PhysicalPosition::new(0, 0));
        let parent_scale = parent.scale_factor();
        let bounds = presentation.bounds();
        let x = parent_origin
            .x
            .saturating_add(((bounds.x() as f64) * parent_scale).round() as i32);
        let y = parent_origin
            .y
            .saturating_add(((bounds.y() as f64) * parent_scale).round() as i32);

        let popup = self
            .popups
            .get(&key)
            .expect("popup should exist before configuring");
        popup.set_outer_position(x, y);
        popup.request_inner_area(native_logical_area(geometry::LogicalArea::from_size(
            presentation.scene().size(),
        )));

        Ok(())
    }

    fn sync_popup_surface(&mut self, key: PopupKey) -> Result<wgpu::TextureFormat, NativeError> {
        self.ensure_context()?;
        let popup = self
            .popups
            .get_mut(&key)
            .expect("popup should exist before syncing surface");
        let area = popup.inner_area().clamp_min(1);
        let scale_factor = popup.scale_factor() as f32;
        let needs_resize = popup.canvas().physical_area() != area
            || (popup.canvas().scale_factor() - scale_factor).abs() > f32::EPSILON;

        if needs_resize {
            log::debug!(
                "syncing native popup {:?}: area={}x{}, scale={}",
                key.id,
                area.width(),
                area.height(),
                scale_factor
            );
            let context = self
                .context
                .as_ref()
                .expect("render context should exist before resizing popup");
            popup.resize(context, area, scale_factor);
        }

        Ok(popup.canvas().surface().config().format)
    }

    fn close_stale_popups(&mut self, active: &HashSet<PopupKey>) {
        let stale = self
            .popups
            .keys()
            .filter(|key| !active.contains(key))
            .copied()
            .collect::<Vec<_>>();
        for key in stale {
            if let Some(popup) = self.popups.remove(&key) {
                self.raw_popups.remove(&popup.raw_id());
                log::debug!(
                    "closed stale native popup {:?} for parent {:?}",
                    key.id,
                    key.parent
                );
            }
        }
    }
}

fn native_popups_supported() -> bool {
    if cfg!(target_os = "windows") || cfg!(target_os = "macos") {
        return true;
    }

    if cfg!(all(unix, not(target_os = "macos"))) {
        return std::env::var_os("WAYLAND_DISPLAY").is_none();
    }

    false
}
