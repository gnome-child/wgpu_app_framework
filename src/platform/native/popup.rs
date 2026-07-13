use std::collections::{HashMap, HashSet};
use std::time::Instant;

use crate::{geometry, overlay, paint, pointer, render, window as app_window};

use super::window::{InitialSize, Options, Window as NativeWindow};
use super::{
    ApplyDue, Native, NativeContext, NativeError, PopupFirstPresentAction, PopupFirstPresentState,
    PopupFirstPresentTrace, PopupGeometry, PopupHost, PopupKey, PopupMaterialReadiness,
    PopupPresentationMode, PopupPrewarmState, PopupWindow, popup_accent_due, popup_border_due,
    popup_geometry_due,
};

impl Native {
    fn allocate_popup_generation(&mut self) -> crate::popup::Generation {
        self.next_popup_generation = self.next_popup_generation.saturating_add(1);
        crate::popup::Generation::new(self.next_popup_generation)
    }

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
            .map(PopupKey::for_presentation)
            .collect::<HashSet<_>>();
        let mut depths = HashMap::<app_window::Id, usize>::new();
        for presentation in presentations {
            *depths.entry(presentation.parent()).or_default() += 1;
        }
        for (parent, depth) in depths {
            self.popup_pool_capacity
                .entry(parent)
                .and_modify(|capacity| *capacity = (*capacity).max(depth))
                .or_insert(depth);
        }
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
        let key = PopupKey::for_presentation(presentation);
        self.popups
            .get_mut(&key)
            .expect("popup should exist before assigning lifecycle participation")
            .accepts_input =
            presentation.kind() == overlay::LayerKind::Live && presentation.accepts_input();
        self.prepare_popup_generation(presentation, now)?;
        self.configure_popup_window(presentation, now)?;

        let surface_started = Instant::now();
        self.sync_popup_surface(key)?;
        let (render_format, reused_host) = {
            let popup = self
                .popups
                .get(&key)
                .expect("popup should exist before selecting render format");
            (
                super::surface::render_format_for_canvas(popup.host.window.canvas()),
                popup.host.reused,
            )
        };
        log::debug!(
            target: "wgpu_l3::native_popup",
            "first-present stage=surface-configured popup={:?} parent={:?} elapsed_us={} stage_us={} reused_host={reused_host}",
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
        let popup_realization = popup
            .pending_realization
            .or(popup.realization)
            .expect("configured popup should carry one realization");
        let material = presentation.material();
        let prior_material = popup.material;
        let material_changed = prior_material != Some(material);
        let dark_changed =
            prior_material.map(overlay::PopupMaterial::dark) != Some(material.dark());
        if dark_changed {
            popup.host.window.set_popup_material_theme(material.dark());
        }
        if material_changed {
            popup.material = Some(material);
        }
        #[cfg(target_os = "windows")]
        let uses_composition = popup.host.composition.is_some();
        #[cfg(not(target_os = "windows"))]
        let uses_composition = false;
        let (surface_format, alpha_mode, surface_width, surface_height) = {
            let config = popup.host.window.canvas().surface().config();
            (
                config.format,
                config.alpha_mode,
                config.width,
                config.height,
            )
        };
        let realization = popup.host.presentation_mode.realization_for(
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
                    popup.host.presentation_mode,
                    surface_format,
                    alpha_mode,
                    material.preference()
                );
            } else if realization.uses_os_material() {
                log::info!(
                    target: "wgpu_l3::native_popup",
                    "native popup {:?} uses Windows accent acrylic: mode={:?}, format={:?}, alpha={:?}, preference={:?}, tint={:?}",
                    presentation.id(),
                    popup.host.presentation_mode,
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
                    popup.host.presentation_mode,
                    surface_format,
                    alpha_mode,
                    material.preference()
                );
            } else if material.preference() == overlay::PopupMaterialPreference::OpaqueFallback {
                log::info!(
                    target: "wgpu_l3::native_popup",
                    "native popup {:?} uses requested opaque fallback: mode={:?}, format={:?}, alpha={:?}, preference={:?}",
                    presentation.id(),
                    popup.host.presentation_mode,
                    surface_format,
                    alpha_mode,
                    material.preference()
                );
            } else {
                log::warn!(
                    target: "wgpu_l3::native_popup",
                    "native popup {:?} downgraded to opaque fallback: mode={:?}, format={:?}, alpha={:?}, preference={:?}, reason={}",
                    presentation.id(),
                    popup.host.presentation_mode,
                    surface_format,
                    alpha_mode,
                    material.preference(),
                    realization
                        .fallback_reason(popup.host.presentation_mode, surface_format, alpha_mode)
                        .unwrap_or("unknown")
                );
            }
            popup.material_realization = Some(realization);
        }
        let material_region = presentation.scene().legacy_full_window_material_region();
        #[cfg(target_os = "windows")]
        let (mut reports, material_readiness) = popup
            .host
            .composition
            .as_mut()
            .map(|composition| {
                composition
                    .sync_material_regions(
                        presentation.scene().material_regions(),
                        popup.host.window.canvas().scale_factor(),
                        popup_realization.panel_offset_physical(),
                        presentation.scene().shadows().into_iter().next().copied(),
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
        if let Some(composition) = popup.host.composition.as_mut() {
            let fade_result = if !popup.exposed && !popup.reconfiguring {
                match presentation.fade() {
                    overlay::PopupFade::Entering { duration, .. } => {
                        composition.prepare_entrance(popup.generation.serial(), duration)
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
        let scene = super::paint::translate_popup_scene(
            super::paint::to_paint_scene_at_scale(
                source_scene,
                popup.host.window.canvas().scale_factor(),
            ),
            popup_realization,
        );
        let canvas = popup.host.window.canvas();
        let observed_area = popup.host.window.inner_area();
        log::debug!(
            target: "wgpu_l3::native_popup",
            "native popup scale chain {:?}: source_logical={}x{} bounds={}x{} observed_inner={}x{} canvas={}x{} surface={}x{} scale={} realization={:?} render_format={:?}",
            presentation.id(),
            source_scene.size().width(),
            source_scene.size().height(),
            popup_realization.local_bounds().width(),
            popup_realization.local_bounds().height(),
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
            popup
                .host
                .window
                .prepare_popup_first_present()
                .map_err(|code| NativeError::PopupPresentation {
                    operation: "prepare-first-present",
                    code,
                })?;
            popup.presentation_prepared = true;
            popup.first_present.record_prepared(key);
        }

        if uses_composition
            && !popup_scene_needs_submission(
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
        let report = renderer.draw(render_context, popup.host.window.canvas_mut(), &scene)?;
        let draw = draw_started.elapsed();
        let generation = popup.generation;
        popup
            .first_present
            .record_acquire(key, generation, report.acquire_outcome);
        let action = if let Some(timing) = report.present_timing {
            popup.last_presented_scene = Some(scene.clone());
            if popup.exposed && popup.reconfiguring {
                commit_pending_popup_geometry(popup, key);
            }
            let action = popup
                .first_present
                .record_presented(key, generation, timing);
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

        if popup.exposed
            && popup.first_present.is_complete_for(popup.generation)
            && popup
                .realization
                .is_some_and(|realization| realization.generation() != popup.generation)
        {
            popup.realization = popup.pending_realization.take().or_else(|| {
                popup
                    .realization
                    .map(|realization| realization.with_generation(popup.generation))
            });
            popup.reconfiguring = false;
            log::debug!(
                target: "wgpu_l3::native_popup",
                "popup atomic generation committed popup={:?} parent={:?} generation={}",
                key.id,
                key.parent,
                popup.generation.serial(),
            );
        }

        let mut request_redraw = action == PopupFirstPresentAction::RequestRedraw;
        if !popup.exposed && popup.first_present.is_complete_for(popup.generation) {
            request_redraw |= expose_popup_when_ready(popup, key)?;
        }

        Ok(request_redraw)
    }

    fn prepare_popup_generation(
        &mut self,
        presentation: &overlay::PopupPresentation,
        now: Instant,
    ) -> Result<(), NativeError> {
        let key = PopupKey::for_presentation(presentation);
        let (exposed, content_changed, needs_concealment) = self
            .popups
            .get(&key)
            .map(|popup| {
                (
                    popup.exposed,
                    !presentation.paint_only()
                        && (popup.source_scene.as_ref() != Some(presentation.scene())
                            || popup.context_fingerprint != presentation.context_fingerprint()),
                    popup_needs_concealment(
                        popup.exposed,
                        popup.material,
                        presentation.material(),
                        popup.realization.map(crate::popup::Realization::scale),
                        popup.host.window.scale_factor(),
                    ),
                )
            })
            .expect("popup should exist before preparing its generation");

        let transition = popup_generation_transition(exposed, content_changed, needs_concealment);
        let next_generation = (transition != PopupGenerationTransition::None)
            .then(|| self.allocate_popup_generation());
        let popup = self
            .popups
            .get_mut(&key)
            .expect("popup should exist before preparing its generation");

        popup.source_scene = Some(presentation.scene().clone());
        popup.context_fingerprint = presentation.context_fingerprint();

        let Some(generation) = next_generation else {
            return Ok(());
        };

        popup.generation = generation;
        popup.first_present = PopupFirstPresentTrace::new(now, generation);

        if transition == PopupGenerationTransition::Atomic {
            log::debug!(
                target: "wgpu_l3::native_popup",
                "popup content generation staged without concealment popup={:?} parent={:?} generation={} content_changed={content_changed}",
                key.id,
                key.parent,
                generation.serial(),
            );
            return Ok(());
        }

        popup
            .host
            .window
            .prepare_popup_first_present()
            .map_err(|code| NativeError::PopupPresentation {
                operation: "prepare-current-generation",
                code,
            })?;
        popup.realization = None;
        popup.pending_realization = None;
        popup.presentation_prepared = true;
        popup.exposed = false;
        popup.reconfiguring = true;
        popup.first_present.record_prepared(key);
        popup.material_readiness = popup.host.readiness_for_session();
        log::debug!(
            target: "wgpu_l3::native_popup",
            "popup generation staged popup={:?} parent={:?} generation={} local_bounds={:?}",
            key.id,
            key.parent,
            generation.serial(),
            presentation.local_bounds(),
        );
        Ok(())
    }

    fn ensure_popup_window(
        &mut self,
        context: &NativeContext<'_>,
        presentation: &overlay::PopupPresentation,
    ) -> Result<(), NativeError> {
        let key = PopupKey::for_presentation(presentation);
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
        let parent_scale = parent.scale_factor();
        let parent_handle = parent.handle();
        let initial_projection = super::paint::PopupProjection::resolve(
            presentation.scene(),
            parent_scale as f32,
            presentation_mode == PopupPresentationMode::CompositionBacked,
        );
        if let Some(mut host) =
            self.take_reusable_popup(presentation.parent(), presentation_mode, parent_scale)
        {
            host.reused = true;
            let generation = self.allocate_popup_generation();
            let popup = PopupWindow::new(host, lifecycle_epoch, generation);
            let raw = popup.host.window.raw_id();
            self.raw_popups.insert(raw, key);
            self.popups.insert(key, popup);
            log::info!(
                target: "wgpu_l3::native_popup",
                "reused warm popup host {:?} for parent {:?} raw={raw:?} mode={presentation_mode:?}",
                presentation.id(),
                presentation.parent(),
            );
            return Ok(());
        }
        let host_started = Instant::now();
        let host = self.create_popup_host(
            context,
            parent_handle,
            initial_projection.logical_area(),
            presentation_mode,
            presentation.scene().clear(),
        )?;
        let generation = self.allocate_popup_generation();
        let popup = PopupWindow::new(host, lifecycle_epoch, generation);
        log::debug!(
            target: "wgpu_l3::native_popup",
            "first-present stage=created popup={:?} parent={:?} elapsed_us={} raw={:?} mode={:?} no_redirection_bitmap={} backend={:?} logical_size={:?} scale={}",
            presentation.id(),
            presentation.parent(),
            popup.first_present.elapsed_micros(),
            popup.host.window.raw_id(),
            presentation_mode,
            presentation_mode.no_redirection_bitmap(),
            self.context
                .as_ref()
                .expect("render context should exist after creating popup")
                .adapter()
                .get_info()
                .backend,
            presentation.scene().size(),
            popup.host.window.scale_factor()
        );
        #[cfg(target_os = "windows")]
        log::info!(
            target: "wgpu_l3::native_popup",
            "native popup {:?} composition tenancy={} setup_us={}",
            presentation.id(),
            popup.host.composition.is_some(),
            host_started.elapsed().as_micros()
        );

        self.raw_popups.insert(popup.host.window.raw_id(), key);
        self.popups.insert(key, popup);

        Ok(())
    }

    fn create_popup_host(
        &mut self,
        context: &NativeContext<'_>,
        parent: super::window::Handle,
        initial_area: paint::area::Logical,
        presentation_mode: PopupPresentationMode,
        clear: crate::scene::Color,
    ) -> Result<PopupHost, NativeError> {
        let native_options = Options {
            title: "wgpu_l3 popup host".to_owned(),
            inner_size: InitialSize::Logical(initial_area),
            kind: app_window::Kind::Popup,
            owner: Some(parent),
            popup_presentation_mode: Some(presentation_mode),
        };
        let handle = NativeWindow::open(native_options, context.event_loop())?;
        let inner_size = handle.inner_size();
        let canvas_options = || render::CanvasOptions {
            area: paint::area::physical(inner_size.width, inner_size.height).clamp_min(1),
            scale_factor: handle.scale_factor() as f32,
            color: render::color_to_wgpu(super::color::paint_color(clear)),
            composite_alpha: presentation_mode.alpha_preference(),
        };
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
        let window = NativeWindow::new(handle, canvas);
        window.set_ime_allowed(false);
        Ok(PopupHost::new(
            window,
            presentation_mode,
            #[cfg(target_os = "windows")]
            composition,
        ))
    }

    pub(in crate::platform::native) fn advance_popup_prewarm(
        &mut self,
        context: &NativeContext<'_>,
    ) {
        let scheduled = self.popup_prewarm.iter().find_map(|(parent, state)| {
            (*state == PopupPrewarmState::Scheduled).then_some(*parent)
        });
        if let Some(parent) = scheduled {
            if self.popups.keys().any(|key| key.parent == parent) {
                self.popup_prewarm
                    .insert(parent, PopupPrewarmState::Complete);
                return;
            }
            if let Some(host) = self
                .popup_pool
                .get_mut(&parent)
                .and_then(|pool| pool.last_mut())
            {
                #[cfg(target_os = "windows")]
                let readiness = host
                    .composition
                    .as_mut()
                    .map(super::composition::Host::material_readiness)
                    .unwrap_or(super::composition::MaterialReadiness::NotRequired);
                #[cfg(not(target_os = "windows"))]
                let readiness = ();

                #[cfg(target_os = "windows")]
                if matches!(readiness, super::composition::MaterialReadiness::Pending(_)) {
                    self.schedule_poll_request();
                    return;
                }
                host.window.hide_popup_before_teardown();
                self.popup_prewarm
                    .insert(parent, PopupPrewarmState::Complete);
                log::debug!(
                    target: "wgpu_l3::native_popup",
                    "prewarmed popup host is receipted and dormant parent={parent:?} readiness={readiness:?}"
                );
                return;
            }

            let started = Instant::now();
            match self.prewarm_popup_host(context, parent) {
                Ok(Some(host)) => {
                    let raw = host.window.raw_id();
                    self.popup_pool_capacity
                        .entry(parent)
                        .and_modify(|capacity| *capacity = (*capacity).max(1))
                        .or_insert(1);
                    self.popup_pool.entry(parent).or_default().push(host);
                    log::info!(
                        target: "wgpu_l3::native_popup",
                        "prewarmed inert root popup host parent={parent:?} raw={raw:?} elapsed_us={} pool_capacity=1",
                        started.elapsed().as_micros(),
                    );
                    self.schedule_poll_request();
                    return;
                }
                Ok(None) => log::debug!(
                    target: "wgpu_l3::native_popup",
                    "popup host prewarm skipped parent={parent:?}: composition tenancy unavailable"
                ),
                Err(error) => log::warn!(
                    target: "wgpu_l3::native_popup",
                    "popup host prewarm failed parent={parent:?}; cold creation remains available: {error}"
                ),
            }
            self.popup_prewarm
                .insert(parent, PopupPrewarmState::Complete);
            return;
        }

        let armed = self
            .popup_prewarm
            .iter()
            .find_map(|(parent, state)| (*state == PopupPrewarmState::Armed).then_some(*parent));
        if let Some(parent) = armed {
            self.popup_prewarm
                .insert(parent, PopupPrewarmState::Scheduled);
            self.schedule_poll_request();
        }
    }

    fn prewarm_popup_host(
        &mut self,
        context: &NativeContext<'_>,
        parent: app_window::Id,
    ) -> Result<Option<PopupHost>, NativeError> {
        let Some(parent_handle) = self.windows.get(&parent).map(super::window::Window::handle)
        else {
            return Ok(None);
        };
        let presentation_mode = PopupPresentationMode::from_render_context(
            self.context
                .as_ref()
                .expect("render context should exist after a stable parent presentation"),
        );
        if presentation_mode != PopupPresentationMode::CompositionBacked {
            return Ok(None);
        }
        let mut host = self.create_popup_host(
            context,
            parent_handle,
            paint::area::logical(1.0, 1.0),
            presentation_mode,
            crate::scene::Color::rgba(0, 0, 0, 0),
        )?;
        #[cfg(target_os = "windows")]
        if host.composition.is_none() {
            return Ok(None);
        }

        let render_format = super::surface::render_format_for_canvas(host.window.canvas());
        self.ensure_renderer(render_format);
        let render_context = self
            .context
            .as_ref()
            .expect("render context should exist while prewarming popup host");
        self.renderers
            .get_mut(&render_format)
            .expect("renderer should exist while prewarming popup host")
            .clear(render_context, host.window.canvas_mut())?;
        #[cfg(target_os = "windows")]
        if let Some(composition) = host.composition.as_mut() {
            match composition.prewarm_material() {
                Ok(super::composition::MaterialReadiness::NotRequired) => return Ok(None),
                Ok(readiness) => log::debug!(
                    target: "wgpu_l3::native_popup",
                    "composition material prewarm began readiness={readiness:?}"
                ),
                Err(error) => {
                    log::warn!(
                        target: "wgpu_l3::native_popup",
                        "cannot prewarm composition material visual: {error}"
                    );
                    return Ok(None);
                }
            }
        }
        host.window.prepare_popup_first_present().map_err(|code| {
            NativeError::PopupPresentation {
                operation: "prepare-popup-prewarm",
                code,
            }
        })?;
        Ok(Some(host))
    }

    fn configure_popup_window(
        &mut self,
        presentation: &overlay::PopupPresentation,
        now: Instant,
    ) -> Result<(), NativeError> {
        let parent_id = presentation.parent();
        let key = PopupKey::for_presentation(presentation);
        let Some(parent) = self.windows.get(&parent_id) else {
            return Err(NativeError::MissingWindow { window: parent_id });
        };
        let parent_handle = parent.handle();
        let parent_origin = parent_handle.inner_position().unwrap_or_else(|error| {
            log::warn!(
                target: "wgpu_l3::native_popup",
                "cannot read parent client origin for popup {:?}: {error}; falling back to outer origin",
                presentation.id()
            );
            parent_handle
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
        let available = presentation.placement().and_then(|placement| {
            super::sys::popup_available_bounds(&parent_handle, placement.anchor())
        });
        let bounds = presentation
            .placement()
            .zip(available)
            .map(|(placement, available)| placement.resolve(available))
            .unwrap_or_else(|| presentation.local_bounds());
        let (
            projection,
            popup_scale,
            exposed,
            already_reconfiguring,
            generation,
            realized_generation,
            applied_geometry,
        ) = {
            let popup = self
                .popups
                .get(&key)
                .expect("popup should exist before configuring");
            (
                super::paint::PopupProjection::resolve(
                    presentation.scene(),
                    popup.host.window.scale_factor() as f32,
                    popup.host.composition.is_some(),
                ),
                popup.host.window.scale_factor(),
                popup.exposed,
                popup.reconfiguring,
                popup.generation,
                popup.realization.map(crate::popup::Realization::generation),
                popup.geometry.applied(),
            )
        };
        let (visual_dx, visual_dy) = projection.visual_offset_physical();
        let x = parent_origin
            .x
            .saturating_add(((bounds.x() as f64) * parent_scale).round() as i32)
            .saturating_add(visual_dx);
        let y = parent_origin
            .y
            .saturating_add(((bounds.y() as f64) * parent_scale).round() as i32)
            .saturating_add(visual_dy);

        let area = projection.logical_area();
        let desired = PopupGeometry {
            x,
            y,
            width: area.width(),
            height: area.height(),
            scale_factor_bits: popup_scale.to_bits(),
        };
        let geometry_changed = popup_geometry_changed(exposed, applied_geometry, desired);
        let geometry_generation = popup_geometry_needs_generation(
            geometry_changed,
            already_reconfiguring,
            realized_generation,
            generation,
        )
        .then(|| self.allocate_popup_generation());
        let popup = self
            .popups
            .get_mut(&key)
            .expect("popup should exist before configuring");
        if let Some(generation) = geometry_generation {
            popup.generation = generation;
            popup.first_present = PopupFirstPresentTrace::new(now, generation);
            log::debug!(
                target: "wgpu_l3::native_popup",
                "popup geometry minted a current generation popup={:?} parent={:?} generation={}",
                key.id,
                key.parent,
                generation.serial(),
            );
        }
        if exposed {
            popup.reconfiguring = geometry_changed;
            if !geometry_changed {
                popup.pending_geometry = None;
            }
        }
        let realization = crate::popup::Realization::native(
            presentation.id(),
            parent_id,
            popup.generation,
            presentation.local_bounds(),
            bounds,
            available
                .map(|available| visible_intersection(bounds, available))
                .unwrap_or(bounds),
            projection.visual_bounds_at(bounds),
            projection.panel_offset_logical(),
            popup_scale,
        );
        let staging_geometry = popup.exposed && popup.reconfiguring;
        if popup.exposed
            && popup
                .realization
                .is_some_and(|current| current.generation() != popup.generation)
        {
            popup.pending_realization = Some(realization);
        } else {
            popup.realization = Some(realization);
            popup.pending_realization = None;
        }
        log::trace!(
            target: "wgpu_l3::native_popup",
            "resolved popup realization popup={:?} generation={} local={:?} host={:?} clip={:?} visual={:?} scale={}",
            realization.popup(),
            realization.generation().serial(),
            realization.local_bounds(),
            realization.host_bounds(),
            realization.visible_clip(),
            realization.visual_bounds(),
            realization.scale(),
        );
        let observed_position = popup.host.window.handle().outer_position().ok();
        let observed_area = popup.host.window.inner_area();

        popup.geometry.set_desired(desired, now);
        if staging_geometry {
            popup.pending_geometry = Some(desired);
            log::debug!(
                target: "wgpu_l3::native_popup",
                "staged popup geometry behind current presentation popup={:?} generation={} desired={desired:?}",
                key.id,
                popup.generation.serial(),
            );
            return Ok(());
        }

        set_popup_hit_rect_for_realization(&popup.host.window, realization, popup.accepts_input);
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
            .host
            .window
            .configure_popup_bounds(desired.x, desired.y, desired.logical_area());
        popup.geometry.mark_applied(desired);
        popup.pending_geometry = None;
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
        let scale_factor = popup.host.window.scale_factor() as f32;
        let area = popup
            .pending_geometry
            .map(|geometry| geometry.logical_area().to_physical(scale_factor))
            .unwrap_or_else(|| popup.host.window.inner_area())
            .clamp_min(1);
        let needs_resize = popup.host.window.canvas().physical_area() != area
            || (popup.host.window.canvas().scale_factor() - scale_factor).abs() > f32::EPSILON;

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
            popup.host.window.resize(context, area, scale_factor);
        }

        Ok(popup.host.window.canvas().surface().config().format)
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
            self.release_ime_from_popup(key);
            if let Some(popup) = self.popups.remove(&key) {
                self.raw_popups.remove(&popup.host.window.raw_id());
                if self.popup_is_reusable(key.parent, &popup) {
                    popup.host.window.set_ime_allowed(false);
                    popup.host.window.set_cursor(pointer::Cursor::Default);
                    popup.host.window.hide_popup_before_teardown();
                    let raw = popup.host.window.raw_id();
                    let area = popup.host.window.canvas().physical_area();
                    let host = popup.into_host();
                    let pool = self.popup_pool.entry(key.parent).or_default();
                    pool.push(host);
                    log::info!(
                        target: "wgpu_l3::native_popup",
                        "returned popup host to warm pool parent={:?} raw={raw:?} surface={}x{} dormant={} capacity={}",
                        key.parent,
                        area.width(),
                        area.height(),
                        pool.len(),
                        self.popup_pool_capacity.get(&key.parent).copied().unwrap_or(0),
                    );
                } else {
                    log::debug!(
                        target: "wgpu_l3::native_popup",
                        "closed stale native popup {:?} for parent {:?}",
                        key.id,
                        key.parent
                    );
                }
            }
        }
    }

    fn take_reusable_popup(
        &mut self,
        parent: app_window::Id,
        mode: PopupPresentationMode,
        scale_factor: f64,
    ) -> Option<PopupHost> {
        let (popup, empty, evicted) = {
            let pool = self.popup_pool.get_mut(&parent)?;
            let before = pool.len();
            pool.retain(|popup| {
                popup.presentation_mode == mode
                    && (popup.window.scale_factor() - scale_factor).abs() <= f64::EPSILON
            });
            let evicted = before.saturating_sub(pool.len());
            let popup = pool.pop();
            (popup, pool.is_empty(), evicted)
        };
        if evicted != 0 {
            log::info!(
                target: "wgpu_l3::native_popup",
                "evicted incompatible popup hosts parent={parent:?} count={evicted} requested_mode={mode:?} requested_scale={scale_factor}"
            );
        }
        if empty {
            self.popup_pool.remove(&parent);
        }
        if popup.is_some() {
            self.popup_prewarm
                .insert(parent, PopupPrewarmState::Complete);
        }
        popup
    }

    fn popup_is_reusable(&self, parent: app_window::Id, popup: &PopupWindow) -> bool {
        let capacity = self
            .popup_pool_capacity
            .get(&parent)
            .copied()
            .unwrap_or_default();
        let dormant = self.popup_pool.get(&parent).map_or(0, Vec::len);
        self.windows.contains_key(&parent)
            && dormant < capacity
            && matches!(popup.material_readiness, PopupMaterialReadiness::Ready(_))
            && popup_has_reusable_composition(popup)
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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum PopupGenerationTransition {
    None,
    Atomic,
    Concealed,
}

fn popup_generation_transition(
    exposed: bool,
    content_changed: bool,
    needs_concealment: bool,
) -> PopupGenerationTransition {
    if !exposed || (!content_changed && !needs_concealment) {
        PopupGenerationTransition::None
    } else if needs_concealment {
        PopupGenerationTransition::Concealed
    } else {
        PopupGenerationTransition::Atomic
    }
}

#[allow(clippy::too_many_arguments)]
fn popup_needs_concealment(
    exposed: bool,
    current_material: Option<overlay::PopupMaterial>,
    next_material: overlay::PopupMaterial,
    current_scale: Option<f64>,
    next_scale: f64,
) -> bool {
    exposed
        && (current_material != Some(next_material)
            || current_scale.is_some_and(|scale| (scale - next_scale).abs() > f64::EPSILON))
}

fn popup_geometry_changed(
    exposed: bool,
    applied: Option<PopupGeometry>,
    desired: PopupGeometry,
) -> bool {
    exposed && applied != Some(desired)
}

fn popup_geometry_needs_generation(
    geometry_changed: bool,
    already_reconfiguring: bool,
    realized_generation: Option<crate::popup::Generation>,
    current_generation: crate::popup::Generation,
) -> bool {
    geometry_changed && !already_reconfiguring && realized_generation == Some(current_generation)
}

fn visible_intersection(rect: geometry::Rect, clip: geometry::Rect) -> geometry::Rect {
    let x = rect.x().max(clip.x());
    let y = rect.y().max(clip.y());
    let right = rect.right().min(clip.right());
    let bottom = rect.bottom().min(clip.bottom());
    geometry::Rect::new(
        x,
        y,
        right.saturating_sub(x).max(0),
        bottom.saturating_sub(y).max(0),
    )
}

fn set_popup_hit_rect_for_realization(
    window: &super::window::Window,
    realization: crate::popup::Realization,
    accepts_input: bool,
) {
    window.set_popup_hit_rect(popup_hit_rect_for_realization(realization, accepts_input));
}

fn popup_hit_rect_for_realization(
    realization: crate::popup::Realization,
    accepts_input: bool,
) -> geometry::Rect {
    if !accepts_input {
        return geometry::Rect::new(0, 0, 0, 0);
    }
    let (panel_x, panel_y) = realization.panel_offset_physical();
    let scale = realization.scale();
    geometry::Rect::new(
        panel_x,
        panel_y,
        (f64::from(realization.host_bounds().width()) * scale).round() as i32,
        (f64::from(realization.host_bounds().height()) * scale).round() as i32,
    )
}

fn commit_pending_popup_geometry(popup: &mut PopupWindow, key: PopupKey) {
    let Some(desired) = popup.pending_geometry.take() else {
        return;
    };
    let observed_position = popup.host.window.handle().outer_position().ok();
    let observed_area = popup.host.window.inner_area();
    popup
        .host
        .window
        .configure_popup_bounds(desired.x, desired.y, desired.logical_area());
    popup.geometry.mark_applied(desired);
    if let Some(realization) = popup.pending_realization {
        set_popup_hit_rect_for_realization(&popup.host.window, realization, popup.accepts_input);
    }
    popup
        .first_present
        .record_configured(key, desired, observed_position, observed_area);
    log::debug!(
        target: "wgpu_l3::native_popup",
        "committed popup geometry with freshly presented content popup={:?} parent={:?} generation={} desired={desired:?}",
        key.id,
        key.parent,
        popup.generation.serial(),
    );
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
    if popup.host.window.set_popup_accent_material(accent) {
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
    if popup.host.composition.is_some() {
        super::sys::suppress_popup_border(&popup.host.window.handle());
    } else {
        popup.host.window.set_popup_border_color(border);
    }
    popup.border.mark_applied(border);
}

fn expose_popup_when_ready(popup: &mut PopupWindow, key: PopupKey) -> Result<bool, NativeError> {
    #[cfg(target_os = "windows")]
    if let Some(composition) = popup.host.composition.as_mut() {
        match composition.entrance_readiness(popup.generation.serial()) {
            Ok(readiness) if readiness.waits_for_receipt() => {
                let super::composition::EntranceReadiness::Pending(generation) = readiness else {
                    unreachable!("only pending entrance readiness waits for a receipt");
                };
                log::debug!(
                    target: "wgpu_l3::native_popup",
                    "popup entrance generation={generation} awaits prepared-root commit before exposure"
                );
                return Ok(true);
            }
            Ok(_) => {}
            Err(error) => {
                log::warn!(
                    target: "wgpu_l3::native_popup",
                    "popup prepared-root commit failed before exposure: {error}"
                );
                return abandon_material_prewarm(popup, key);
            }
        }
    }
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
        .host
        .window
        .expose_popup_after_present()
        .map_err(|code| NativeError::PopupPresentation {
            operation: "expose-after-present",
            code,
        })?;
    #[cfg(target_os = "windows")]
    if !popup.reconfiguring
        && let Some(composition) = popup.host.composition.as_mut()
        && let Err(error) =
            composition.start_prepared_entrance(popup.generation.serial(), Instant::now())
    {
        log::warn!(
            target: "wgpu_l3::native_popup",
            "failed to start prepared popup entrance after exposure: {error}"
        );
    }
    popup.exposed = true;
    popup.reconfiguring = false;
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
    if popup.reconfiguring {
        for barrier in 1..=2 {
            if let Err(code) = super::sys::synchronize_popup_presentation() {
                log::warn!(
                    target: "wgpu_l3::native_popup",
                    "popup material generation={generation} concealed-reconfigure barrier={barrier} failed code={code}; using framework fallback"
                );
                return abandon_material_prewarm(popup, key);
            }
        }
        if !popup.material_readiness.mark_ready(generation) {
            log::warn!(
                target: "wgpu_l3::native_popup",
                "popup material generation={generation} became stale during concealed reconfiguration"
            );
            return abandon_material_prewarm(popup, key);
        }
        popup
            .host
            .window
            .expose_popup_after_present()
            .map_err(|code| NativeError::PopupPresentation {
                operation: "expose-current-generation",
                code,
            })?;
        popup.exposed = true;
        popup.reconfiguring = false;
        popup.first_present.record_exposed(key);
        log::debug!(
            target: "wgpu_l3::native_popup",
            "popup material generation={generation} reconfigured atomically elapsed_us={}",
            started.elapsed().as_micros()
        );
        return Ok(false);
    }

    popup
        .host
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

    if !popup.reconfiguring {
        let entrance = popup
            .host
            .composition
            .as_mut()
            .expect("committed material requires a composition host")
            .start_prepared_entrance(popup.generation.serial(), Instant::now());
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
    }
    if !popup.material_readiness.mark_ready(generation) {
        log::warn!(
            target: "wgpu_l3::native_popup",
            "popup material generation={generation} became stale during prewarm; keeping popup concealed"
        );
        return abandon_material_prewarm(popup, key);
    }
    debug_assert!(popup_reveal_gate_open(
        popup.first_present.is_complete_for(popup.generation),
        popup.material_readiness
    ));

    popup.exposed = true;
    popup.reconfiguring = false;
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
        .host
        .window
        .prepare_popup_first_present()
        .map_err(|code| NativeError::PopupPresentation {
            operation: "reconceal-after-material-failure",
            code,
        })?;
    if let Some(composition) = popup.host.composition.as_mut() {
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

    fn record_acquire(
        &mut self,
        key: PopupKey,
        generation: crate::popup::Generation,
        outcome: render::AcquireOutcome,
    ) {
        if !self.accepts(generation) {
            log::debug!(
                target: "wgpu_l3::native_popup",
                "ignored stale popup acquire receipt popup={:?} parent={:?} receipt_generation={} current_generation={}",
                key.id,
                key.parent,
                generation.serial(),
                self.generation.serial(),
            );
            return;
        }
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
        generation: crate::popup::Generation,
        timing: render::PresentTiming,
    ) -> PopupFirstPresentAction {
        if !self.accepts(generation) {
            log::debug!(
                target: "wgpu_l3::native_popup",
                "ignored stale popup present receipt popup={:?} parent={:?} receipt_generation={} current_generation={}",
                key.id,
                key.parent,
                generation.serial(),
                self.generation.serial(),
            );
            return PopupFirstPresentAction::None;
        }
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

    fn is_complete_for(&self, generation: crate::popup::Generation) -> bool {
        generation == self.generation && self.is_complete()
    }

    fn accepts(&self, generation: crate::popup::Generation) -> bool {
        generation == self.generation
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
    exposed: bool,
    freshness_pending: bool,
    last_presented: Option<&paint::Scene>,
    requested: &paint::Scene,
) -> bool {
    if !exposed || freshness_pending {
        return true;
    }
    last_presented != Some(requested)
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

#[cfg(target_os = "windows")]
fn popup_has_reusable_composition(popup: &PopupWindow) -> bool {
    popup.host.composition.is_some()
}

#[cfg(not(target_os = "windows"))]
fn popup_has_reusable_composition(_popup: &PopupWindow) -> bool {
    false
}

#[cfg(test)]
mod tests {
    use std::collections::HashSet;
    use std::time::Instant;

    use super::{
        PopupFirstPresentAction, PopupFirstPresentState, PopupFirstPresentTrace,
        PopupGenerationTransition, PopupKey, PopupMaterialReadiness, first_present_follow_up,
        popup_generation_transition, popup_geometry_changed, popup_geometry_needs_generation,
        popup_hit_rect_for_realization, popup_is_stale, popup_needs_concealment,
        popup_reveal_gate_open, popup_scene_needs_submission, queue_popup_parent_redraw,
    };
    use crate::platform::native::PopupGeometry;
    use crate::{interaction, overlay, paint, window};

    #[test]
    fn unchanged_composition_scene_does_not_submit_after_fresh_exposure() {
        let scene = paint::Scene::new();
        assert!(popup_scene_needs_submission(false, false, None, &scene));
        assert!(popup_scene_needs_submission(
            true,
            true,
            Some(&scene),
            &scene
        ));
        assert!(!popup_scene_needs_submission(
            true,
            false,
            Some(&scene),
            &scene
        ));

        let mut changed = paint::Scene::new();
        changed.clear(paint::Color::BLACK);
        assert!(popup_scene_needs_submission(
            true,
            false,
            Some(&scene),
            &changed
        ));
        assert!(!popup_scene_needs_submission(
            true,
            false,
            Some(&changed),
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

        let mut trace =
            PopupFirstPresentTrace::new(Instant::now(), crate::popup::Generation::initial());
        assert!(
            trace.needs_redraw(),
            "a skipped first acquire must retry because no present occurred"
        );
        trace.state = PopupFirstPresentState::Complete;
        assert!(!trace.needs_redraw());
    }

    #[test]
    fn popup_receipts_are_bound_to_their_exact_generation() {
        let generation_a = crate::popup::Generation::new(41);
        let generation_b = crate::popup::Generation::new(42);
        let generation_a_again = crate::popup::Generation::new(43);
        let trace = PopupFirstPresentTrace::new(Instant::now(), generation_a_again);

        assert!(!trace.accepts(generation_a));
        assert!(!trace.accepts(generation_b));
        assert!(trace.accepts(generation_a_again));
        assert_ne!(generation_a, generation_a_again);
    }

    #[test]
    fn material_and_scale_changes_require_concealment() {
        let material = overlay::PopupMaterial::NativeWindow {
            dark: true,
            tint: crate::scene::Color::rgba(10, 20, 30, 40),
            preference: overlay::PopupMaterialPreference::System,
        };
        let next_material = overlay::PopupMaterial::NativeWindow {
            dark: false,
            tint: crate::scene::Color::rgba(10, 20, 30, 40),
            preference: overlay::PopupMaterialPreference::System,
        };

        assert!(!popup_needs_concealment(
            false,
            Some(material),
            next_material,
            Some(1.0),
            2.0,
        ));
        assert!(
            !popup_needs_concealment(true, Some(material), material, Some(1.0), 1.0,),
            "ordinary same-geometry pixel updates present atomically without recloaking"
        );
        assert!(popup_needs_concealment(
            true,
            Some(material),
            next_material,
            Some(1.0),
            1.0,
        ));
        assert!(popup_needs_concealment(
            true,
            Some(material),
            material,
            Some(1.0),
            1.25,
        ));
    }

    #[test]
    fn content_serials_and_concealment_are_orthogonal() {
        assert_eq!(
            popup_generation_transition(true, false, false),
            PopupGenerationTransition::None,
            "parent-window activity and unchanged popup content borrow no popup clock"
        );
        assert_eq!(
            popup_generation_transition(true, true, false),
            PopupGenerationTransition::Atomic,
            "same-realization content gets a serial without hiding the current surface"
        );
        assert_eq!(
            popup_generation_transition(true, true, true),
            PopupGenerationTransition::Concealed,
            "material and scale changes retain the concealed realization gate"
        );
        assert_eq!(
            popup_generation_transition(false, true, true),
            PopupGenerationTransition::None,
            "birth already owns its initial concealed generation"
        );
    }

    #[test]
    fn geometry_transaction_includes_movement_and_does_not_remint_while_pending() {
        let generation = crate::popup::Generation::new(7);
        let replacement = crate::popup::Generation::new(8);
        let applied = PopupGeometry {
            x: 40,
            y: 20,
            width: 180.0,
            height: 120.0,
            scale_factor_bits: 1.0_f64.to_bits(),
        };
        let moved = PopupGeometry { x: 220, ..applied };
        let resized = PopupGeometry {
            height: 180.0,
            ..applied
        };

        assert!(popup_geometry_changed(true, Some(applied), moved));
        assert!(popup_geometry_changed(true, Some(applied), resized));
        assert!(!popup_geometry_changed(false, Some(applied), moved));
        assert!(popup_geometry_needs_generation(
            true,
            false,
            Some(generation),
            generation,
        ));
        assert!(!popup_geometry_needs_generation(
            true,
            true,
            Some(generation),
            replacement,
        ));
        assert!(!popup_geometry_needs_generation(
            true,
            false,
            Some(generation),
            replacement,
        ));
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

    #[test]
    fn retiring_popup_keeps_paint_geometry_but_carries_no_hit_region() {
        let realization = crate::popup::Realization::native(
            interaction::Id::new("context_menu"),
            window::Id::new(1),
            crate::popup::Generation::new(7),
            crate::geometry::Rect::new(300, 40, 120, 80),
            crate::geometry::Rect::new(900, 60, 120, 80),
            crate::geometry::Rect::new(900, 60, 120, 80),
            crate::geometry::Rect::new(892, 52, 136, 96),
            crate::geometry::Point::new(8, 8),
            1.25,
        );

        assert_eq!(
            popup_hit_rect_for_realization(realization, true),
            crate::geometry::Rect::new(10, 10, 150, 100),
            "the live panel consumes the same projected panel geometry as paint"
        );
        assert_eq!(
            popup_hit_rect_for_realization(realization, false),
            crate::geometry::Rect::new(0, 0, 0, 0),
            "the retiring authored-menu layer remains visual-only"
        );
    }
}
