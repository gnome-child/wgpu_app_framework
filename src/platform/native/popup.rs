use std::collections::HashSet;
use std::time::Instant;

use crate::{overlay, paint, render, window as app_window};

use super::window::{InitialSize, Options, Window as NativeWindow};
use super::{
    ApplyDue, Native, NativeContext, NativeError, PopupFirstPresentAction, PopupFirstPresentState,
    PopupFirstPresentTrace, PopupGeometry, PopupKey, PopupMaterialReadiness, PopupPresentationMode,
    PopupWindow, popup_accent_due, popup_border_due, popup_geometry_due,
};

impl Native {
    pub(in crate::platform::native) fn overlay_capabilities(&self) -> overlay::Capabilities {
        if !native_popups_supported() {
            return overlay::Capabilities::in_frame_only();
        }
        if self
            .context
            .as_ref()
            .is_some_and(|context| context.adapter().get_info().backend == wgpu::Backend::Dx12)
        {
            overlay::Capabilities::with_native_popups()
        } else {
            overlay::Capabilities::with_immediate_native_popups()
        }
    }

    pub(in crate::platform::native) fn present_popup_overlays(
        &mut self,
        context: &NativeContext<'_>,
        synchronized_parents: &[app_window::Id],
        presentations: &[overlay::PopupPresentation],
    ) -> Result<(), NativeError> {
        let now = Instant::now();
        let synchronized_parents = synchronized_parents.iter().copied().collect::<HashSet<_>>();
        let active = presentations
            .iter()
            .map(|presentation| PopupKey::new(presentation.parent(), presentation.id()))
            .collect::<HashSet<_>>();
        self.close_stale_popups(&synchronized_parents, &active);

        let mut redraw_parents = HashSet::new();
        for presentation in presentations {
            if self.present_popup_overlay(context, presentation, now)? {
                queue_popup_parent_redraw(&mut redraw_parents, presentation.parent());
            }
        }
        redraw_parents.extend(self.apply_due_popup_accents(now));
        self.apply_due_popup_borders(now);
        self.request_popup_parent_redraws(&redraw_parents);

        Ok(())
    }

    fn present_popup_overlay(
        &mut self,
        context: &NativeContext<'_>,
        presentation: &overlay::PopupPresentation,
        now: Instant,
    ) -> Result<bool, NativeError> {
        self.ensure_popup_window(context, presentation)?;
        self.configure_popup_window(presentation, now)?;

        let key = PopupKey::new(presentation.parent(), presentation.id());
        let surface_started = Instant::now();
        self.sync_popup_surface(key)?;
        let render_format = {
            let popup = self
                .popups
                .get(&key)
                .expect("popup should exist before selecting render format");
            super::surface::render_format_for_canvas(popup.window.canvas())
        };
        log::debug!(
            target: "wgpu_l3::native_popup",
            "first-present stage=surface-configured popup={:?} parent={:?} elapsed_us={} stage_us={}",
            presentation.id(),
            presentation.parent(),
            presentation.lifecycle_epoch().elapsed().as_micros(),
            surface_started.elapsed().as_micros()
        );
        let renderer_was_warm = self.renderers.contains_key(&render_format);
        let renderer_started = Instant::now();
        self.ensure_renderer(render_format);
        log::debug!(
            target: "wgpu_l3::native_popup",
            "first-present stage=renderer-ready popup={:?} parent={:?} elapsed_us={} stage_us={} warm={}",
            presentation.id(),
            presentation.parent(),
            presentation.lifecycle_epoch().elapsed().as_micros(),
            renderer_started.elapsed().as_micros(),
            renderer_was_warm
        );

        let render_context = self
            .context
            .as_ref()
            .expect("render context should exist before presenting popup");
        let renderer = self
            .renderers
            .get_mut(&render_format)
            .expect("renderer should exist before presenting popup");
        let popup = self
            .popups
            .get_mut(&key)
            .expect("popup should exist before presenting");
        let material = presentation.material();
        let prior_material = popup.material;
        let material_changed = prior_material != Some(material);
        let dark_changed =
            prior_material.map(overlay::PopupMaterial::dark) != Some(material.dark());
        if dark_changed {
            popup.window.set_popup_material_theme(material.dark());
        }
        if material_changed {
            popup.material = Some(material);
        }
        #[cfg(target_os = "windows")]
        let uses_composition = popup.composition.is_some();
        #[cfg(not(target_os = "windows"))]
        let uses_composition = false;
        let (surface_format, alpha_mode, surface_width, surface_height) = {
            let config = popup.window.canvas().surface().config();
            (
                config.format,
                config.alpha_mode,
                config.width,
                config.height,
            )
        };
        let realization = popup.presentation_mode.realization_for(
            surface_format,
            alpha_mode,
            material.preference(),
        );
        let realization_changed = popup.material_realization != Some(realization);
        if realization_changed {
            if uses_composition {
                log::info!(
                    target: "wgpu_l3::native_popup",
                    "native popup {:?} will resolve material from composition region reports: mode={:?}, format={:?}, alpha={:?}, preference={:?}",
                    presentation.id(),
                    popup.presentation_mode,
                    surface_format,
                    alpha_mode,
                    material.preference()
                );
            } else if realization.uses_os_material() {
                log::info!(
                    target: "wgpu_l3::native_popup",
                    "native popup {:?} uses Windows accent acrylic: mode={:?}, format={:?}, alpha={:?}, preference={:?}, tint={:?}",
                    presentation.id(),
                    popup.presentation_mode,
                    surface_format,
                    alpha_mode,
                    material.preference(),
                    material.tint()
                );
            } else if realization.uses_native_material_scene() {
                log::info!(
                    target: "wgpu_l3::native_popup",
                    "native popup {:?} uses transparent native scene without accent: mode={:?}, format={:?}, alpha={:?}, preference={:?}",
                    presentation.id(),
                    popup.presentation_mode,
                    surface_format,
                    alpha_mode,
                    material.preference()
                );
            } else if material.preference() == overlay::PopupMaterialPreference::OpaqueFallback {
                log::info!(
                    target: "wgpu_l3::native_popup",
                    "native popup {:?} uses requested opaque fallback: mode={:?}, format={:?}, alpha={:?}, preference={:?}",
                    presentation.id(),
                    popup.presentation_mode,
                    surface_format,
                    alpha_mode,
                    material.preference()
                );
            } else {
                log::warn!(
                    target: "wgpu_l3::native_popup",
                    "native popup {:?} downgraded to opaque fallback: mode={:?}, format={:?}, alpha={:?}, preference={:?}, reason={}",
                    presentation.id(),
                    popup.presentation_mode,
                    surface_format,
                    alpha_mode,
                    material.preference(),
                    realization
                        .fallback_reason(popup.presentation_mode, surface_format, alpha_mode)
                        .unwrap_or("unknown")
                );
            }
            popup.material_realization = Some(realization);
        }
        let material_region = presentation.scene().legacy_full_window_material_region();
        #[cfg(target_os = "windows")]
        let (mut reports, material_readiness) = popup
            .composition
            .as_mut()
            .map(|composition| {
                let projection = super::paint::PopupProjection::resolve(
                    presentation.scene(),
                    popup.window.canvas().scale_factor(),
                    true,
                );
                composition
                    .sync_material_regions(
                        presentation.scene().material_regions(),
                        popup.window.canvas().scale_factor(),
                        projection.panel_offset(),
                        projection.shadow(),
                    )
                    .into_parts()
            })
            .unwrap_or((
                Vec::new(),
                super::composition::MaterialReadiness::NotRequired,
            ));
        #[cfg(target_os = "windows")]
        popup.material_readiness.observe(match material_readiness {
            super::composition::MaterialReadiness::NotRequired => {
                PopupMaterialReadiness::NotRequired
            }
            super::composition::MaterialReadiness::Pending(generation) => {
                PopupMaterialReadiness::Pending(generation)
            }
            super::composition::MaterialReadiness::Committed(generation) => {
                PopupMaterialReadiness::Committed(generation)
            }
        });
        #[cfg(not(target_os = "windows"))]
        let mut reports = Vec::new();
        let tenancy_realized = !reports.is_empty();
        let capabilities = if realization.uses_os_material() {
            crate::scene::MaterialCapabilities::backdrop_frost()
        } else {
            crate::scene::MaterialCapabilities::none()
        };
        let accent = if !uses_composition
            && !tenancy_realized
            && capabilities.forecasts_backdrop_frost()
            && material_region.is_some()
        {
            super::sys::PopupAccentMaterial::Acrylic {
                tint: material.tint(),
            }
        } else {
            super::sys::PopupAccentMaterial::Disabled
        };
        if !uses_composition {
            if popup.accent.set_desired(accent, now) {
                log::debug!(
                    target: "wgpu_l3::native_popup",
                    "recorded legacy native popup accent desire {:?}: realization={:?}, accent={:?}",
                    presentation.id(),
                    realization,
                    accent
                );
            }
            if let Some(reason) = popup_accent_due(&popup.accent, now) {
                apply_popup_accent(key, popup, reason);
            }
        }
        if popup.border.set_desired(presentation.border(), now) {
            log::debug!(
                target: "wgpu_l3::native_popup",
                "recorded native popup border desire {:?}: border={:?}",
                presentation.id(),
                presentation.border()
            );
        }
        if let Some(reason) = popup_border_due(&popup.border, now) {
            apply_popup_border(key, popup, reason);
        }
        reports.extend(
            material_region
                .filter(|_| reports.is_empty())
                .filter(|_| {
                    matches!(
                        popup.accent.applied(),
                        Some(super::sys::PopupAccentMaterial::Acrylic { .. })
                    )
                })
                .map(|region| {
                    crate::scene::MaterialRealizationReport::new(
                        region.id(),
                        crate::scene::RealizedMaterialParts::frost(
                            popup.accent.applied() == Some(accent),
                        ),
                    )
                })
                .into_iter(),
        );
        let material_started = Instant::now();
        let material_resolution = presentation.scene().resolve_material(
            crate::scene::MaterialRenderer::NativePopup {
                opaque: reports.is_empty(),
            },
            &reports,
        );
        #[cfg(target_os = "windows")]
        if let Some(composition) = popup.composition.as_mut() {
            let fade_result = if !popup.exposed {
                match presentation.fade() {
                    overlay::PopupFade::Entering { duration, .. } => {
                        composition.prepare_entrance(duration)
                    }
                    fade => composition.apply_fade(fade, Instant::now()),
                }
            } else {
                composition.apply_fade(presentation.fade(), Instant::now())
            };
            if let Err(error) = fade_result {
                log::warn!(
                    target: "wgpu_l3::native_popup",
                    "failed to project popup opacity into composition tree: {error}"
                );
            }
        }
        let faded_scene = (!uses_composition).then(|| {
            let mut scene = crate::scene::Scene::new_with_clear(
                material_resolution.scene().size(),
                material_resolution.scene().clear(),
            );
            scene.append_scene_with_opacity(material_resolution.scene(), presentation.opacity());
            scene
        });
        let source_scene = faded_scene
            .as_ref()
            .unwrap_or_else(|| material_resolution.scene());
        log::debug!(
            target: "wgpu_l3::native_popup",
            "first-present stage=material-resolved popup={:?} parent={:?} elapsed_us={} stage_us={}",
            presentation.id(),
            presentation.parent(),
            presentation.lifecycle_epoch().elapsed().as_micros(),
            material_started.elapsed().as_micros()
        );
        log::debug!(
            target: "wgpu_l3::native_popup",
            "native popup material resolution {:?}: fidelity={:?}, regions={:?}",
            presentation.id(),
            material_resolution.fidelity(),
            material_resolution.region_fidelity()
        );
        let projection = super::paint::PopupProjection::resolve(
            source_scene,
            popup.window.canvas().scale_factor(),
            uses_composition,
        );
        let scene = projection.translate_scene(super::paint::to_paint_scene_at_scale(
            source_scene,
            popup.window.canvas().scale_factor(),
        ));
        let canvas = popup.window.canvas();
        let observed_area = popup.window.inner_area();
        log::debug!(
            target: "wgpu_l3::native_popup",
            "native popup scale chain {:?}: source_logical={}x{} bounds={}x{} observed_inner={}x{} canvas={}x{} surface={}x{} scale={} realization={:?} render_format={:?}",
            presentation.id(),
            source_scene.size().width(),
            source_scene.size().height(),
            presentation.bounds().width(),
            presentation.bounds().height(),
            observed_area.width(),
            observed_area.height(),
            canvas.physical_area().width(),
            canvas.physical_area().height(),
            surface_width,
            surface_height,
            canvas.scale_factor(),
            realization,
            render_format
        );

        if !popup.presentation_prepared {
            popup.window.prepare_popup_first_present().map_err(|code| {
                NativeError::PopupPresentation {
                    operation: "prepare-first-present",
                    code,
                }
            })?;
            popup.presentation_prepared = true;
            popup.first_present.record_prepared(key);
        }

        if uses_composition
            && !popup_scene_needs_submission(
                presentation.fade(),
                presentation.paint_only(),
                popup.exposed,
                popup.first_present.needs_redraw(),
                popup.last_presented_scene.as_ref(),
                &scene,
            )
        {
            log::debug!(
                target: "wgpu_l3::native_popup",
                "skipped unchanged composition popup submission {:?} for parent {:?}",
                presentation.id(),
                presentation.parent()
            );
            return Ok(false);
        }

        let draw_started = Instant::now();
        let report = renderer.draw(render_context, popup.window.canvas_mut(), &scene)?;
        let draw = draw_started.elapsed();
        popup
            .first_present
            .record_acquire(key, report.acquire_outcome);
        let action = if let Some(timing) = report.present_timing {
            popup.last_presented_scene = Some(scene.clone());
            let action = popup.first_present.record_presented(key, timing);
            log::debug!(
                target: "wgpu_l3::native_popup",
                "presented native popup {:?} for parent {:?}: draw={}us acquire={}us groups={}",
                presentation.id(),
                presentation.parent(),
                draw.as_micros(),
                timing.acquire_wait().as_micros(),
                report.stats.group_composites
            );
            action
        } else {
            log::debug!(
                target: "wgpu_l3::native_popup",
                "skipped native popup frame {:?} for parent {:?}: draw={}us visible={}",
                presentation.id(),
                presentation.parent(),
                draw.as_micros(),
                popup.exposed
            );
            if popup.first_present.needs_redraw() {
                PopupFirstPresentAction::RequestRedraw
            } else {
                PopupFirstPresentAction::None
            }
        };

        let mut request_redraw = action == PopupFirstPresentAction::RequestRedraw;
        if !popup.exposed && popup.first_present.is_complete() {
            request_redraw |= expose_popup_when_ready(popup, key)?;
        }

        Ok(request_redraw)
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

        let lifecycle_epoch = presentation.lifecycle_epoch();
        log::debug!(
            target: "wgpu_l3::native_popup",
            "first-present stage=native-request popup={:?} parent={:?} elapsed_us={}",
            presentation.id(),
            presentation.parent(),
            lifecycle_epoch.elapsed().as_micros()
        );
        let context_was_warm = self.context.is_some();
        let context_started = Instant::now();
        self.ensure_context()?;
        log::debug!(
            target: "wgpu_l3::native_popup",
            "first-present stage=renderer-context popup={:?} parent={:?} elapsed_us={} stage_us={} warm={}",
            presentation.id(),
            presentation.parent(),
            lifecycle_epoch.elapsed().as_micros(),
            context_started.elapsed().as_micros(),
            context_was_warm
        );
        let presentation_mode = PopupPresentationMode::from_render_context(
            self.context
                .as_ref()
                .expect("render context should exist before creating popup"),
        );
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
        let initial_projection = super::paint::PopupProjection::resolve(
            presentation.scene(),
            parent.scale_factor() as f32,
            presentation_mode == PopupPresentationMode::CompositionBacked,
        );
        let native_options = Options {
            title: format!("wgpu_l3 popup {}", presentation.id().as_str()),
            inner_size: InitialSize::Logical(initial_projection.logical_area()),
            kind: app_window::Kind::Popup,
            owner: Some(parent.handle()),
            popup_presentation_mode: Some(presentation_mode),
        };
        let hwnd_started = Instant::now();
        let handle = NativeWindow::open(native_options, context.event_loop())?;
        log::debug!(
            target: "wgpu_l3::native_popup",
            "first-present stage=hwnd-created popup={:?} parent={:?} elapsed_us={} stage_us={}",
            presentation.id(),
            presentation.parent(),
            lifecycle_epoch.elapsed().as_micros(),
            hwnd_started.elapsed().as_micros()
        );
        let inner_size = handle.inner_size();
        let canvas_options = || render::CanvasOptions {
            area: paint::area::physical(inner_size.width, inner_size.height).clamp_min(1),
            scale_factor: handle.scale_factor() as f32,
            color: render::color_to_wgpu(super::color::paint_color(presentation.scene().clear())),
            composite_alpha: presentation_mode.alpha_preference(),
        };
        #[cfg(target_os = "windows")]
        let tenancy_started = Instant::now();
        #[cfg(target_os = "windows")]
        let tenancy = if presentation_mode == PopupPresentationMode::CompositionBacked {
            if self.composition.is_none() {
                match super::composition::Runtime::new() {
                    Ok(runtime) => self.composition = Some(runtime),
                    Err(error) => log::warn!(
                        target: "wgpu_l3::native_popup",
                        "Windows composition runtime unavailable; retaining legacy popup realization: {error}"
                    ),
                }
            }
            self.composition.as_ref().and_then(|runtime| {
                let render_context = self
                    .context
                    .as_ref()
                    .expect("render context should exist before creating popup");
                let attempt = (|| {
                    let seed = runtime.create_surface_seed()?;
                    let canvas = unsafe {
                        render::Canvas::new_unsafe(canvas_options(), render_context, seed.target())
                    }
                    .map_err(|error| windows::core::Error::new(
                        windows::Win32::Foundation::E_FAIL,
                        format!("create tenancy surface: {error}"),
                    ))?;
                    let host = runtime.attach(seed, &handle, &canvas)?;
                    Ok::<_, windows::core::Error>((canvas, host))
                })();
                match attempt {
                    Ok(tenancy) => Some(tenancy),
                    Err(error) => {
                        log::warn!(
                            target: "wgpu_l3::native_popup",
                            "single-HWND composition tenancy unavailable; retaining legacy popup realization: {error}"
                        );
                        None
                    }
                }
            })
        } else {
            None
        };
        let render_context = self
            .context
            .as_ref()
            .expect("render context should exist before creating popup");
        #[cfg(target_os = "windows")]
        let (canvas, composition) = match tenancy {
            Some((canvas, host)) => (canvas, Some(host)),
            None => (
                render::Canvas::new(canvas_options(), render_context, handle.clone())?,
                None,
            ),
        };
        #[cfg(not(target_os = "windows"))]
        let canvas = render::Canvas::new(canvas_options(), render_context, handle.clone())?;
        let popup = NativeWindow::new(handle, canvas);
        popup.set_ime_allowed(false);
        let mut popup = PopupWindow::new(popup, presentation_mode, lifecycle_epoch);
        #[cfg(target_os = "windows")]
        {
            popup.composition = composition;
        }
        log::debug!(
            target: "wgpu_l3::native_popup",
            "first-present stage=created popup={:?} parent={:?} elapsed_us={} raw={:?} mode={:?} no_redirection_bitmap={} backend={:?} logical_size={:?} scale={}",
            presentation.id(),
            presentation.parent(),
            popup.first_present.elapsed_micros(),
            popup.window.raw_id(),
            presentation_mode,
            presentation_mode.no_redirection_bitmap(),
            render_context.adapter().get_info().backend,
            presentation.scene().size(),
            popup.window.scale_factor()
        );
        #[cfg(target_os = "windows")]
        log::info!(
            target: "wgpu_l3::native_popup",
            "native popup {:?} composition tenancy={} setup_us={}",
            presentation.id(),
            popup.composition.is_some(),
            tenancy_started.elapsed().as_micros()
        );

        self.raw_popups.insert(popup.window.raw_id(), key);
        self.popups.insert(key, popup);

        Ok(())
    }

    fn configure_popup_window(
        &mut self,
        presentation: &overlay::PopupPresentation,
        now: Instant,
    ) -> Result<(), NativeError> {
        let parent_id = presentation.parent();
        let key = PopupKey::new(parent_id, presentation.id());
        let Some(parent) = self.windows.get(&parent_id) else {
            return Err(NativeError::MissingWindow { window: parent_id });
        };
        let parent_origin = parent.handle().inner_position().unwrap_or_else(|error| {
            log::warn!(
                target: "wgpu_l3::native_popup",
                "cannot read parent client origin for popup {:?}: {error}; falling back to outer origin",
                presentation.id()
            );
            parent
                .handle()
                .outer_position()
                .unwrap_or_else(|fallback_error| {
                    log::warn!(
                        target: "wgpu_l3::native_popup",
                        "cannot read parent outer origin for popup {:?}: {fallback_error}; using screen origin",
                        presentation.id()
                    );
                    winit::dpi::PhysicalPosition::new(0, 0)
                })
        });
        let parent_scale = parent.scale_factor();
        let bounds = presentation.bounds();
        let popup = self
            .popups
            .get_mut(&key)
            .expect("popup should exist before configuring");
        let projection = super::paint::PopupProjection::resolve(
            presentation.scene(),
            popup.window.scale_factor() as f32,
            popup.composition.is_some(),
        );
        let (visual_dx, visual_dy) = projection.visual_offset_physical();
        let x = parent_origin
            .x
            .saturating_add(((bounds.x() as f64) * parent_scale).round() as i32)
            .saturating_add(visual_dx);
        let y = parent_origin
            .y
            .saturating_add(((bounds.y() as f64) * parent_scale).round() as i32)
            .saturating_add(visual_dy);

        popup.bounds = bounds;
        popup.panel_offset_physical = projection.panel_offset_physical();
        let area = projection.logical_area();
        let desired = PopupGeometry {
            x,
            y,
            width: area.width(),
            height: area.height(),
            scale_factor_bits: popup.window.scale_factor().to_bits(),
        };
        let observed_position = popup.window.handle().outer_position().ok();
        let observed_area = popup.window.inner_area();

        popup.geometry.set_desired(desired, now);
        let Some(reason) = popup_geometry_due(&popup.geometry, now) else {
            log::trace!(
                target: "wgpu_l3::native_popup",
                "skipped native popup geometry {:?}: desired={desired:?}, observed_position={observed_position:?}, observed_area={}x{}",
                key.id,
                observed_area.width(),
                observed_area.height()
            );
            return Ok(());
        };

        log::debug!(
            target: "wgpu_l3::native_popup",
            "applying native popup geometry {:?}: reason={reason:?}, desired={desired:?}, prior={:?}, observed_position={observed_position:?}, observed_area={}x{}",
            key.id,
            popup.geometry.applied(),
            observed_area.width(),
            observed_area.height()
        );
        popup
            .window
            .configure_popup_bounds(desired.x, desired.y, desired.logical_area());
        popup.geometry.mark_applied(desired);
        popup
            .first_present
            .record_configured(key, desired, observed_position, observed_area);

        Ok(())
    }

    fn sync_popup_surface(&mut self, key: PopupKey) -> Result<wgpu::TextureFormat, NativeError> {
        self.ensure_context()?;
        let popup = self
            .popups
            .get_mut(&key)
            .expect("popup should exist before syncing surface");
        let area = popup.window.inner_area().clamp_min(1);
        let scale_factor = popup.window.scale_factor() as f32;
        let needs_resize = popup.window.canvas().physical_area() != area
            || (popup.window.canvas().scale_factor() - scale_factor).abs() > f32::EPSILON;

        if needs_resize {
            log::debug!(
                target: "wgpu_l3::native_popup",
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
            popup.window.resize(context, area, scale_factor);
        }

        Ok(popup.window.canvas().surface().config().format)
    }

    fn close_stale_popups(
        &mut self,
        synchronized_parents: &HashSet<app_window::Id>,
        active: &HashSet<PopupKey>,
    ) {
        let stale = self
            .popups
            .keys()
            .filter(|key| popup_is_stale(key, synchronized_parents, active))
            .copied()
            .collect::<Vec<_>>();
        for key in stale {
            self.rehome_cursor_from_popup(key);
            if let Some(popup) = self.popups.remove(&key) {
                self.raw_popups.remove(&popup.window.raw_id());
                log::debug!(
                    target: "wgpu_l3::native_popup",
                    "closed stale native popup {:?} for parent {:?}",
                    key.id,
                    key.parent
                );
            }
        }
    }

    pub(in crate::platform::native) fn apply_due_popup_accents(
        &mut self,
        now: Instant,
    ) -> HashSet<app_window::Id> {
        let mut pending = false;
        let mut redraw_parents = HashSet::new();
        for (key, popup) in &mut self.popups {
            let Some(reason) = popup_accent_due(&popup.accent, now) else {
                if popup.accent.pending() && popup.accent.changed_instant() != Some(now) {
                    pending = true;
                    log::trace!(
                        target: "wgpu_l3::native_popup",
                        "native popup accent pending {:?}: desired={:?}",
                        key.id,
                        popup.accent.desired()
                    );
                }
                continue;
            };
            apply_popup_accent(*key, popup, reason);
            queue_popup_parent_redraw(&mut redraw_parents, key.parent);
        }

        if pending {
            self.schedule_poll_request();
        }

        redraw_parents
    }

    pub(in crate::platform::native) fn apply_due_popup_borders(&mut self, now: Instant) {
        let mut pending = false;
        for (key, popup) in &mut self.popups {
            let Some(reason) = popup_border_due(&popup.border, now) else {
                if popup.border.pending() && popup.border.changed_instant() != Some(now) {
                    pending = true;
                    log::trace!(
                        target: "wgpu_l3::native_popup",
                        "native popup border pending {:?}: desired={:?}",
                        key.id,
                        popup.border.desired()
                    );
                }
                continue;
            };
            apply_popup_border(*key, popup, reason);
        }

        if pending {
            self.schedule_poll_request();
        }
    }

    pub(in crate::platform::native) fn request_popup_parent_redraws(
        &self,
        parents: &HashSet<app_window::Id>,
    ) {
        for parent in parents {
            let Some(window) = self.windows.get(parent) else {
                log::trace!(
                    target: "wgpu_l3::native_popup",
                    "skipped popup maintenance redraw for closed parent {parent:?}"
                );
                continue;
            };
            log::debug!(
                target: "wgpu_l3::native_popup",
                "requested coalesced popup maintenance redraw for parent {parent:?}"
            );
            window.request_redraw();
        }
    }
}

fn apply_popup_accent(key: PopupKey, popup: &mut PopupWindow, reason: ApplyDue) -> bool {
    let accent = popup
        .accent
        .desired()
        .expect("due accent should have a desired material");
    log::debug!(
        target: "wgpu_l3::native_popup",
        "applying native popup accent {:?}: reason={:?}, accent={:?}",
        key.id,
        reason,
        accent
    );
    if popup.window.set_popup_accent_material(accent) {
        popup.accent.mark_applied(accent);
        true
    } else {
        log::warn!(
            target: "wgpu_l3::native_popup",
            "native popup accent {:?} was requested but not realized",
            key.id
        );
        false
    }
}

fn apply_popup_border(key: PopupKey, popup: &mut PopupWindow, reason: ApplyDue) {
    let border = popup
        .border
        .desired()
        .expect("due border should have a desired color");
    log::debug!(
        target: "wgpu_l3::native_popup",
        "applying native popup border {:?}: reason={:?}, border={:?}",
        key.id,
        reason,
        border
    );
    if popup.composition.is_some() {
        super::sys::suppress_popup_border(&popup.window.handle());
    } else {
        popup.window.set_popup_border_color(border);
    }
    popup.border.mark_applied(border);
}

fn expose_popup_when_ready(popup: &mut PopupWindow, key: PopupKey) -> Result<bool, NativeError> {
    match popup.material_readiness {
        PopupMaterialReadiness::Pending(generation) => {
            log::debug!(
                target: "wgpu_l3::native_popup",
                "popup material generation={generation} awaits effect commit before exposure"
            );
            Ok(true)
        }
        PopupMaterialReadiness::Committed(generation) => {
            #[cfg(target_os = "windows")]
            return expose_committed_material(popup, key, generation);

            #[cfg(not(target_os = "windows"))]
            {
                let _ = generation;
                expose_popup_without_material_gate(popup, key)
            }
        }
        readiness @ (PopupMaterialReadiness::NotRequired | PopupMaterialReadiness::Ready(_)) => {
            debug_assert!(popup_reveal_gate_open(true, readiness));
            expose_popup_without_material_gate(popup, key)
        }
    }
}

fn expose_popup_without_material_gate(
    popup: &mut PopupWindow,
    key: PopupKey,
) -> Result<bool, NativeError> {
    popup
        .window
        .expose_popup_after_present()
        .map_err(|code| NativeError::PopupPresentation {
            operation: "expose-after-present",
            code,
        })?;
    #[cfg(target_os = "windows")]
    if let Some(composition) = popup.composition.as_mut()
        && let Err(error) = composition.start_prepared_entrance(Instant::now())
    {
        log::warn!(
            target: "wgpu_l3::native_popup",
            "failed to start prepared popup entrance after exposure: {error}"
        );
    }
    popup.exposed = true;
    popup.first_present.record_exposed(key);
    Ok(false)
}

#[cfg(target_os = "windows")]
fn expose_committed_material(
    popup: &mut PopupWindow,
    key: PopupKey,
    generation: u64,
) -> Result<bool, NativeError> {
    let started = Instant::now();
    popup
        .window
        .expose_popup_after_present()
        .map_err(|code| NativeError::PopupPresentation {
            operation: "begin-material-prewarm",
            code,
        })?;

    for barrier in 1..=2 {
        if let Err(code) = super::sys::synchronize_popup_presentation() {
            log::warn!(
                target: "wgpu_l3::native_popup",
                "popup material generation={generation} host-frame barrier={barrier} failed code={code}; using framework fallback"
            );
            return abandon_material_prewarm(popup, key);
        }
    }

    let entrance = popup
        .composition
        .as_mut()
        .expect("committed material requires a composition host")
        .start_prepared_entrance(Instant::now());
    if let Err(error) = entrance {
        log::warn!(
            target: "wgpu_l3::native_popup",
            "popup material generation={generation} entrance failed after prewarm: {error}; using framework fallback"
        );
        return abandon_material_prewarm(popup, key);
    }
    if let Err(code) = super::sys::synchronize_popup_presentation() {
        log::warn!(
            target: "wgpu_l3::native_popup",
            "popup material generation={generation} fade-start barrier failed code={code}; using framework fallback"
        );
        return abandon_material_prewarm(popup, key);
    }
    if !popup.material_readiness.mark_ready(generation) {
        log::warn!(
            target: "wgpu_l3::native_popup",
            "popup material generation={generation} became stale during prewarm; keeping popup concealed"
        );
        return abandon_material_prewarm(popup, key);
    }
    debug_assert!(popup_reveal_gate_open(
        popup.first_present.is_complete(),
        popup.material_readiness
    ));

    popup.exposed = true;
    popup.first_present.record_exposed(key);
    log::debug!(
        target: "wgpu_l3::native_popup",
        "popup material generation={generation} ready and exposed elapsed_us={} application_redraws=0 host_frame_barriers=3",
        started.elapsed().as_micros()
    );
    Ok(false)
}

fn popup_reveal_gate_open(
    content_presented: bool,
    material_readiness: PopupMaterialReadiness,
) -> bool {
    content_presented
        && matches!(
            material_readiness,
            PopupMaterialReadiness::NotRequired | PopupMaterialReadiness::Ready(_)
        )
}

#[cfg(target_os = "windows")]
fn abandon_material_prewarm(popup: &mut PopupWindow, key: PopupKey) -> Result<bool, NativeError> {
    popup
        .window
        .prepare_popup_first_present()
        .map_err(|code| NativeError::PopupPresentation {
            operation: "reconceal-after-material-failure",
            code,
        })?;
    if let Some(composition) = popup.composition.as_mut() {
        composition.abandon_material();
    }
    popup.material_readiness = PopupMaterialReadiness::NotRequired;
    popup.last_presented_scene = None;
    log::debug!(
        target: "wgpu_l3::native_popup",
        "popup material realization abandoned before exposure popup={:?} parent={:?}",
        key.id,
        key.parent
    );
    Ok(true)
}

impl PopupFirstPresentTrace {
    fn record_configured(
        &mut self,
        key: PopupKey,
        desired: PopupGeometry,
        observed_position: Option<winit::dpi::PhysicalPosition<i32>>,
        observed_area: paint::area::Physical,
    ) {
        if self.configured {
            return;
        }
        self.configured = true;
        log::debug!(
            target: "wgpu_l3::native_popup",
            "first-present stage=configured popup={:?} parent={:?} elapsed_us={} desired={desired:?} observed_position={observed_position:?} observed_physical={}x{}",
            key.id,
            key.parent,
            self.elapsed_micros(),
            observed_area.width(),
            observed_area.height()
        );
    }

    fn record_prepared(&self, key: PopupKey) {
        if self.state == PopupFirstPresentState::Complete {
            return;
        }
        log::debug!(
            target: "wgpu_l3::native_popup",
            "first-present stage=prepared-concealed popup={:?} parent={:?} elapsed_us={}",
            key.id,
            key.parent,
            self.elapsed_micros()
        );
    }

    fn record_exposed(&self, key: PopupKey) {
        log::debug!(
            target: "wgpu_l3::native_popup",
            "first-present stage=exposed popup={:?} parent={:?} elapsed_us={}",
            key.id,
            key.parent,
            self.elapsed_micros()
        );
    }

    fn record_acquire(&mut self, key: PopupKey, outcome: render::AcquireOutcome) {
        if self.state == PopupFirstPresentState::Complete {
            return;
        }
        self.acquire_attempts = self.acquire_attempts.saturating_add(1);
        let stage = match self.state {
            PopupFirstPresentState::AwaitingFirst => "acquire",
            PopupFirstPresentState::AwaitingConfirmation => "confirmation-acquire",
            PopupFirstPresentState::Complete => return,
        };
        log::debug!(
            target: "wgpu_l3::native_popup",
            "first-present stage={stage} popup={:?} parent={:?} elapsed_us={} attempt={} outcome={outcome:?}",
            key.id,
            key.parent,
            self.elapsed_micros(),
            self.acquire_attempts
        );
    }

    fn record_presented(
        &mut self,
        key: PopupKey,
        timing: render::PresentTiming,
    ) -> PopupFirstPresentAction {
        let (stage, action, synchronization) = match self.state {
            PopupFirstPresentState::AwaitingFirst => {
                let started = Instant::now();
                let result = super::sys::synchronize_popup_presentation();
                let elapsed = started.elapsed();
                let (state, stage, action) = first_present_follow_up(result, false);
                self.state = state;
                (stage, action, Some((result, elapsed)))
            }
            PopupFirstPresentState::AwaitingConfirmation => {
                let started = Instant::now();
                let result = super::sys::synchronize_popup_presentation();
                let elapsed = started.elapsed();
                let (state, stage, action) = first_present_follow_up(result, true);
                self.state = state;
                (stage, action, Some((result, elapsed)))
            }
            PopupFirstPresentState::Complete => return PopupFirstPresentAction::None,
        };
        log::debug!(
            target: "wgpu_l3::native_popup",
            "first-present stage={stage} popup={:?} parent={:?} elapsed_us={} attempt={} acquire_us={} synchronization={synchronization:?}",
            key.id,
            key.parent,
            self.elapsed_micros(),
            self.acquire_attempts,
            timing.acquire_wait().as_micros()
        );
        action
    }

    fn needs_redraw(&self) -> bool {
        self.state != PopupFirstPresentState::Complete
    }

    fn is_complete(&self) -> bool {
        self.state == PopupFirstPresentState::Complete
    }
}

fn first_present_follow_up(
    synchronization: Result<(), i32>,
    confirmation: bool,
) -> (
    PopupFirstPresentState,
    &'static str,
    PopupFirstPresentAction,
) {
    if synchronization.is_ok() {
        (
            PopupFirstPresentState::Complete,
            if confirmation {
                "confirmation-synchronized"
            } else {
                "synchronized"
            },
            PopupFirstPresentAction::ContentReady,
        )
    } else if confirmation {
        (
            PopupFirstPresentState::Complete,
            "confirmation-sync-failed",
            PopupFirstPresentAction::ContentReady,
        )
    } else {
        (
            PopupFirstPresentState::AwaitingConfirmation,
            "visibility-sync-failed",
            PopupFirstPresentAction::RequestRedraw,
        )
    }
}

fn queue_popup_parent_redraw(redraw_parents: &mut HashSet<app_window::Id>, parent: app_window::Id) {
    redraw_parents.insert(parent);
}

fn popup_scene_needs_submission(
    fade: overlay::PopupFade,
    paint_only: bool,
    exposed: bool,
    freshness_pending: bool,
    last_presented: Option<&paint::Scene>,
    requested: &paint::Scene,
) -> bool {
    if !exposed || freshness_pending {
        return true;
    }
    match fade {
        overlay::PopupFade::Exiting { .. } => false,
        overlay::PopupFade::Stable if paint_only => last_presented != Some(requested),
        overlay::PopupFade::Entering { .. } | overlay::PopupFade::Stable => true,
    }
}

fn popup_is_stale(
    key: &PopupKey,
    synchronized_parents: &HashSet<app_window::Id>,
    active: &HashSet<PopupKey>,
) -> bool {
    synchronized_parents.contains(&key.parent) && !active.contains(key)
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

#[cfg(test)]
mod tests {
    use std::collections::HashSet;
    use std::time::Instant;

    use super::{
        PopupFirstPresentAction, PopupFirstPresentState, PopupFirstPresentTrace, PopupKey,
        PopupMaterialReadiness, first_present_follow_up, popup_is_stale, popup_reveal_gate_open,
        popup_scene_needs_submission, queue_popup_parent_redraw,
    };
    use crate::{interaction, overlay, paint, window};

    #[test]
    fn unchanged_composition_scene_does_not_submit_after_fresh_exposure() {
        let scene = paint::Scene::new();
        let now = Instant::now();
        let entering = overlay::PopupFade::Entering {
            duration: std::time::Duration::from_millis(80),
            started_at: now,
        };
        let exiting = overlay::PopupFade::Exiting {
            duration: std::time::Duration::from_millis(90),
            started_at: now,
            from_opacity: 1.0,
        };
        assert!(popup_scene_needs_submission(
            entering, false, false, false, None, &scene
        ));
        assert!(popup_scene_needs_submission(
            overlay::PopupFade::Stable,
            true,
            true,
            true,
            Some(&scene),
            &scene
        ));
        assert!(!popup_scene_needs_submission(
            overlay::PopupFade::Stable,
            true,
            true,
            false,
            Some(&scene),
            &scene
        ));

        let mut changed = paint::Scene::new();
        changed.clear(paint::Color::BLACK);
        assert!(popup_scene_needs_submission(
            overlay::PopupFade::Stable,
            true,
            true,
            false,
            Some(&scene),
            &changed
        ));
        assert!(popup_scene_needs_submission(
            overlay::PopupFade::Stable,
            false,
            true,
            false,
            Some(&scene),
            &scene
        ));
        assert!(!popup_scene_needs_submission(
            exiting,
            false,
            true,
            false,
            Some(&scene),
            &changed
        ));
    }

    #[test]
    fn popup_maintenance_redraws_coalesce_per_parent() {
        let first = window::Id::new(1);
        let second = window::Id::new(2);
        let mut parents = HashSet::new();

        queue_popup_parent_redraw(&mut parents, first);
        queue_popup_parent_redraw(&mut parents, first);
        queue_popup_parent_redraw(&mut parents, second);

        assert_eq!(parents, HashSet::from([first, second]));
    }

    #[test]
    fn popup_first_present_redraws_only_for_evidenced_failures() {
        assert_eq!(
            first_present_follow_up(Ok(()), false),
            (
                PopupFirstPresentState::Complete,
                "synchronized",
                PopupFirstPresentAction::ContentReady,
            ),
            "a compositor-synchronized present needs no policy confirmation frame"
        );
        assert_eq!(
            first_present_follow_up(Err(-1), false),
            (
                PopupFirstPresentState::AwaitingConfirmation,
                "visibility-sync-failed",
                PopupFirstPresentAction::RequestRedraw,
            ),
            "an explicit compositor synchronization failure earns one fallback redraw"
        );
        assert_eq!(
            first_present_follow_up(Ok(()), true),
            (
                PopupFirstPresentState::Complete,
                "confirmation-synchronized",
                PopupFirstPresentAction::ContentReady,
            )
        );
        assert_eq!(
            first_present_follow_up(Err(-1), true),
            (
                PopupFirstPresentState::Complete,
                "confirmation-sync-failed",
                PopupFirstPresentAction::ContentReady,
            ),
            "a second freshly presented frame ends the bounded fallback without exposing stale content"
        );

        let mut trace = PopupFirstPresentTrace::new(Instant::now());
        assert!(
            trace.needs_redraw(),
            "a skipped first acquire must retry because no present occurred"
        );
        trace.state = PopupFirstPresentState::Complete;
        assert!(!trace.needs_redraw());
    }

    #[test]
    fn popup_reveal_gate_consumes_content_and_current_material_receipts_once() {
        let mut material = PopupMaterialReadiness::Pending(7);
        assert!(!popup_reveal_gate_open(false, material));
        assert!(!popup_reveal_gate_open(true, material));

        material.observe(PopupMaterialReadiness::Committed(7));
        assert!(!popup_reveal_gate_open(true, material));
        assert!(material.mark_ready(7));
        assert!(!popup_reveal_gate_open(false, material));
        assert!(popup_reveal_gate_open(true, material));
        assert!(!material.mark_ready(7), "a duplicate receipt is inert");
    }

    #[test]
    fn material_replacement_invalidates_stale_receipts() {
        let mut material = PopupMaterialReadiness::Pending(11);
        material.observe(PopupMaterialReadiness::Committed(11));
        material.observe(PopupMaterialReadiness::Pending(12));
        material.observe(PopupMaterialReadiness::Committed(11));

        assert_eq!(material, PopupMaterialReadiness::Pending(12));
        assert!(!material.mark_ready(11));
        material.observe(PopupMaterialReadiness::Committed(12));
        assert!(material.mark_ready(12));
    }

    #[test]
    fn popup_without_platform_material_bypasses_effect_receipt() {
        assert!(popup_reveal_gate_open(
            true,
            PopupMaterialReadiness::NotRequired
        ));
        assert!(!popup_reveal_gate_open(
            false,
            PopupMaterialReadiness::NotRequired
        ));
    }

    #[test]
    fn stale_popup_cleanup_is_scoped_to_synchronized_parents() {
        let first = window::Id::new(1);
        let second = window::Id::new(2);
        let first_popup = PopupKey::new(first, interaction::Id::new("first.popup"));
        let second_popup = PopupKey::new(second, interaction::Id::new("second.popup"));
        let synchronized = HashSet::from([first]);

        assert!(popup_is_stale(&first_popup, &synchronized, &HashSet::new()));
        assert!(
            !popup_is_stale(&second_popup, &synchronized, &HashSet::new()),
            "redrawing one parent must not close another parent's popup"
        );
        assert!(!popup_is_stale(
            &first_popup,
            &synchronized,
            &HashSet::from([first_popup])
        ));
    }
}
