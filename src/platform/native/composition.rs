use std::collections::{HashMap, HashSet};

use crate::{composition, paint, render, scene};

use windows::Foundation::TimeSpan;
use windows::System::DispatcherQueueController;
use windows::UI::Composition::Desktop::DesktopWindowTarget;
use windows::UI::Composition::{
    CompositionBatchTypes, CompositionCommitBatch, CompositionDropShadowSourcePolicy,
    CompositionGeometricClip, CompositionRoundedRectangleGeometry, CompositionStretch,
    CompositionSurfaceBrush, CompositionVisualSurface, Compositor, ContainerVisual, DropShadow,
    ICompositionSurface, SpriteVisual,
};
use windows::Win32::Foundation::{E_HANDLE, E_NOINTERFACE, HWND};
use windows::Win32::Graphics::DirectComposition::{
    DCompositionCreateDevice2, IDCompositionDevice, IDCompositionVisual,
};
use windows::Win32::Graphics::Dwm::{DWMWA_USE_HOSTBACKDROPBRUSH, DwmSetWindowAttribute};
use windows::Win32::System::WinRT::Composition::{ICompositorDesktopInterop, ICompositorInterop};
use windows::Win32::System::WinRT::{
    CreateDispatcherQueueController, DQTAT_COM_NONE, DQTYPE_THREAD_CURRENT, DispatcherQueueOptions,
};
use windows::core::{HSTRING, Interface, Result};
use windows_future::{AsyncStatus, IAsyncAction};
use windows_numerics::{Vector2, Vector3};
use winit::raw_window_handle::{HasWindowHandle, RawWindowHandle};

/// UI-thread composition services shared by the native runtime.
///
/// DispatcherQueue and Compositor have thread affinity and outlive every popup;
/// popup targets and visuals remain independently owned by `Host`.
pub(super) struct Runtime {
    _dispatcher: DispatcherQueueController,
    compositor: Compositor,
    classic: IDCompositionDevice,
}

/// Unattached classic visual retained while wgpu creates and configures its
/// `DxgiFromVisual` swapchain.
pub(super) struct SurfaceSeed {
    visual: IDCompositionVisual,
}

/// One popup's single-HWND composition tenancy tree.
pub(super) struct Host {
    _classic_visual: IDCompositionVisual,
    _target: DesktopWindowTarget,
    root: ContainerVisual,
    regions: ContainerVisual,
    _content: SpriteVisual,
    _content_brush: CompositionSurfaceBrush,
    _wrapped_surface: ICompositionSurface,
    shadow: DropShadow,
    shadow_mask_visual: SpriteVisual,
    shadow_mask_geometry: CompositionRoundedRectangleGeometry,
    _shadow_mask_clip: CompositionGeometricClip,
    shadow_mask_surface: CompositionVisualSurface,
    shadow_mask_brush: CompositionSurfaceBrush,
    compositor: Compositor,
    host_backdrop_enabled: bool,
    material_regions: HashMap<composition::NodeId, RegionVisual>,
    material_spares: Vec<RegionVisual>,
    material_projection: Vec<(composition::NodeId, ProjectedRegion)>,
    shadow_projection: Option<ProjectedShadow>,
    material_generation: u64,
    material_commit: Option<MaterialCommit>,
    committed_generation: Option<u64>,
    fade: Option<FadeState>,
    entrance: EntranceState,
    entrance_action: Option<IAsyncAction>,
}

struct RegionVisual {
    visual: SpriteVisual,
    geometry: CompositionRoundedRectangleGeometry,
    _clip: CompositionGeometricClip,
}

struct MaterialCommit {
    generation: u64,
    batch: CompositionCommitBatch,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct PreparedEntrance {
    generation: u64,
    duration: std::time::Duration,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum EntranceReadiness {
    NotRequired,
    Pending(u64),
    Committed(u64),
}

#[derive(Debug, Default)]
struct EntranceState {
    prepared: Option<PreparedEntrance>,
    readiness: Option<EntranceReadiness>,
}

impl EntranceState {
    fn begin(&mut self, prepared: PreparedEntrance) {
        self.prepared = Some(prepared);
        self.readiness = Some(EntranceReadiness::Pending(prepared.generation));
    }

    fn readiness_for(&self, generation: u64) -> EntranceReadiness {
        let Some(prepared) = self.prepared else {
            return EntranceReadiness::NotRequired;
        };
        let readiness = self
            .readiness
            .unwrap_or(EntranceReadiness::Pending(prepared.generation));
        if prepared.generation == generation
            && readiness == EntranceReadiness::Committed(generation)
        {
            readiness
        } else {
            EntranceReadiness::Pending(prepared.generation)
        }
    }

    fn mark_committed(&mut self, generation: u64) -> bool {
        if self
            .prepared
            .is_some_and(|prepared| prepared.generation == generation)
            && self.readiness == Some(EntranceReadiness::Pending(generation))
        {
            self.readiness = Some(EntranceReadiness::Committed(generation));
            true
        } else {
            false
        }
    }

    fn take_committed(&mut self, generation: u64) -> Option<PreparedEntrance> {
        if self.readiness_for(generation) != EntranceReadiness::Committed(generation) {
            return None;
        }
        self.readiness = None;
        self.prepared.take()
    }
}

impl EntranceReadiness {
    pub(super) fn waits_for_receipt(self) -> bool {
        matches!(self, Self::Pending(_))
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum MaterialReadiness {
    NotRequired,
    Pending(u64),
    Committed(u64),
}

pub(super) struct MaterialSync {
    reports: Vec<scene::MaterialRealizationReport>,
    readiness: MaterialReadiness,
}

impl MaterialSync {
    pub(super) fn into_parts(self) -> (Vec<scene::MaterialRealizationReport>, MaterialReadiness) {
        (self.reports, self.readiness)
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
struct ProjectedRegion {
    offset: Vector3,
    size: Vector2,
    radius: f32,
    opacity: f32,
}

#[derive(Debug, Clone, Copy, PartialEq)]
struct ProjectedShadow {
    mask_offset: Vector2,
    mask_size: Vector2,
    mask_radius: f32,
    blur_radius: f32,
    offset: Vector3,
    color: scene::Color,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct FadeKey {
    phase: u8,
    duration: std::time::Duration,
    from_opacity_bits: u32,
}

#[derive(Debug, Clone, Copy)]
struct FadeState {
    key: FadeKey,
    started_at: std::time::Instant,
    duration: std::time::Duration,
    from: f32,
    target: f32,
}

const PREWARM_OPACITY: f32 = 0.001;

impl FadeState {
    fn opacity_at(self, now: std::time::Instant) -> f32 {
        if self.duration.is_zero() {
            return self.target;
        }
        let progress = now.saturating_duration_since(self.started_at).as_secs_f32()
            / self.duration.as_secs_f32();
        let eased = crate::animation::Easing::EaseOutCubic.sample(progress.clamp(0.0, 1.0));
        self.from + (self.target - self.from) * eased
    }
}

impl Runtime {
    pub(super) fn new() -> Result<Self> {
        let options = DispatcherQueueOptions {
            dwSize: std::mem::size_of::<DispatcherQueueOptions>() as u32,
            threadType: DQTYPE_THREAD_CURRENT,
            apartmentType: DQTAT_COM_NONE,
        };
        let dispatcher = unsafe { CreateDispatcherQueueController(options)? };
        let compositor = Compositor::new()?;
        let classic = unsafe { DCompositionCreateDevice2(None)? };

        Ok(Self {
            _dispatcher: dispatcher,
            compositor,
            classic,
        })
    }

    pub(super) fn create_surface_seed(&self) -> Result<SurfaceSeed> {
        Ok(SurfaceSeed {
            visual: unsafe { self.classic.CreateVisual()? },
        })
    }

    pub(super) fn attach(
        &self,
        seed: SurfaceSeed,
        window: &winit::window::Window,
        canvas: &render::Canvas,
    ) -> Result<Host> {
        let swap_chain = unsafe {
            canvas
                .surface()
                .wgpu_surface()
                .as_hal::<wgpu_hal::api::Dx12>()
                .and_then(|surface| surface.swap_chain())
        }
        .ok_or_else(|| windows::core::Error::from(E_NOINTERFACE))?;

        let interop: ICompositorInterop = self.compositor.cast()?;
        let wrapped_surface = unsafe { interop.CreateCompositionSurfaceForSwapChain(&swap_chain)? };
        let content_brush = self
            .compositor
            .CreateSurfaceBrushWithSurface(&wrapped_surface)?;
        let content = self.compositor.CreateSpriteVisual()?;
        content.SetBrush(&content_brush)?;
        content.SetRelativeSizeAdjustment(Vector2 { X: 1.0, Y: 1.0 })?;

        let shadow_mask_visual = self.compositor.CreateSpriteVisual()?;
        shadow_mask_visual.SetOffset(Vector3 {
            X: -16_384.0,
            Y: -16_384.0,
            Z: 0.0,
        })?;
        shadow_mask_visual.SetBrush(&self.compositor.CreateColorBrushWithColor(
            windows::UI::Color {
                A: 255,
                R: 255,
                G: 255,
                B: 255,
            },
        )?)?;
        let shadow_mask_geometry = self.compositor.CreateRoundedRectangleGeometry()?;
        let shadow_mask_clip = self
            .compositor
            .CreateGeometricClipWithGeometry(&shadow_mask_geometry)?;
        shadow_mask_visual.SetClip(&shadow_mask_clip)?;
        let shadow_mask_surface = self.compositor.CreateVisualSurface()?;
        shadow_mask_surface.SetSourceVisual(&shadow_mask_visual)?;
        let shadow_mask_brush = self
            .compositor
            .CreateSurfaceBrushWithSurface(&shadow_mask_surface)?;
        shadow_mask_brush.SetStretch(CompositionStretch::None)?;
        shadow_mask_brush.SetHorizontalAlignmentRatio(0.0)?;
        shadow_mask_brush.SetVerticalAlignmentRatio(0.0)?;
        let shadow = self.compositor.CreateDropShadow()?;
        shadow.SetMask(&shadow_mask_brush)?;
        shadow.SetSourcePolicy(CompositionDropShadowSourcePolicy::Default)?;
        shadow.SetOpacity(0.0)?;
        content.SetShadow(&shadow)?;

        let root = self.compositor.CreateContainerVisual()?;
        root.SetRelativeSizeAdjustment(Vector2 { X: 1.0, Y: 1.0 })?;
        let regions = self.compositor.CreateContainerVisual()?;
        regions.SetRelativeSizeAdjustment(Vector2 { X: 1.0, Y: 1.0 })?;
        root.Children()?.InsertAtBottom(&shadow_mask_visual)?;
        root.Children()?.InsertAtBottom(&regions)?;
        root.Children()?.InsertAtTop(&content)?;

        let desktop: ICompositorDesktopInterop = self.compositor.cast()?;
        let hwnd = hwnd(window)?;
        let target = unsafe { desktop.CreateDesktopWindowTarget(hwnd, false)? };
        target.SetRoot(&root)?;
        let enabled = 1_i32;
        let host_backdrop_enabled = unsafe {
            DwmSetWindowAttribute(
                hwnd,
                DWMWA_USE_HOSTBACKDROPBRUSH,
                (&raw const enabled).cast(),
                std::mem::size_of::<i32>() as u32,
            )
        }
        .is_ok();

        Ok(Host {
            _classic_visual: seed.visual,
            _target: target,
            root,
            regions,
            _content: content,
            _content_brush: content_brush,
            _wrapped_surface: wrapped_surface,
            shadow,
            shadow_mask_visual,
            shadow_mask_geometry,
            _shadow_mask_clip: shadow_mask_clip,
            shadow_mask_surface,
            shadow_mask_brush,
            compositor: self.compositor.clone(),
            host_backdrop_enabled,
            material_regions: HashMap::new(),
            material_spares: Vec::new(),
            material_projection: Vec::new(),
            shadow_projection: None,
            material_generation: 0,
            material_commit: None,
            committed_generation: None,
            fade: None,
            entrance: EntranceState::default(),
            entrance_action: None,
        })
    }
}

impl SurfaceSeed {
    pub(super) fn target(&self) -> wgpu::SurfaceTargetUnsafe {
        wgpu::SurfaceTargetUnsafe::CompositionVisual(self.visual.as_raw())
    }
}

impl Host {
    pub(super) fn material_readiness(&mut self) -> MaterialReadiness {
        self.poll_material_readiness()
    }

    pub(super) fn prewarm_material(&mut self) -> Result<MaterialReadiness> {
        if !self.host_backdrop_enabled {
            return Ok(MaterialReadiness::NotRequired);
        }
        if !self.material_spares.is_empty() || !self.material_regions.is_empty() {
            return Ok(self.poll_material_readiness());
        }

        let region = RegionVisual::new(&self.compositor)?;
        region.apply(ProjectedRegion {
            offset: Vector3 {
                X: 0.0,
                Y: 0.0,
                Z: 0.0,
            },
            size: Vector2 { X: 1.0, Y: 1.0 },
            radius: 0.0,
            opacity: 1.0,
        })?;
        self.regions.Children()?.InsertAtTop(&region.visual)?;
        self.material_spares.push(region);
        let readiness = self.begin_material_effect_commit();
        log::debug!(
            target: "wgpu_l3::native_popup",
            "composition material prewarm staged spares=1 readiness={readiness:?}"
        );
        Ok(readiness)
    }

    pub(super) fn prepare_entrance(
        &mut self,
        generation: u64,
        duration: std::time::Duration,
    ) -> Result<()> {
        let prepared = PreparedEntrance {
            generation,
            duration,
        };
        if self.entrance.prepared == Some(prepared) {
            return Ok(());
        }
        self.root.StopAnimation(&HSTRING::from("Opacity"))?;
        self.root.SetOpacity(PREWARM_OPACITY)?;
        self.fade = None;
        self.entrance.begin(prepared);
        self.entrance_action = Some(self.compositor.RequestCommitAsync()?);
        log::debug!(
            target: "wgpu_l3::native_popup",
            "composition entrance prepared generation={generation} opacity={PREWARM_OPACITY:.3} duration_us={} commit_receipt=pending application_redraws=0 dwm_flushes=0",
            duration.as_micros()
        );
        Ok(())
    }

    pub(super) fn entrance_readiness(&mut self, generation: u64) -> Result<EntranceReadiness> {
        let readiness = self.entrance.readiness_for(generation);
        if readiness != EntranceReadiness::Pending(generation) {
            return Ok(readiness);
        }
        let Some(action) = self.entrance_action.as_ref() else {
            return Ok(readiness);
        };
        match action.Status()? {
            AsyncStatus::Started => Ok(EntranceReadiness::Pending(generation)),
            AsyncStatus::Completed => {
                action.GetResults()?;
                self.entrance_action = None;
                if !self.entrance.mark_committed(generation) {
                    return Ok(self.entrance.readiness_for(generation));
                }
                log::debug!(
                    target: "wgpu_l3::native_popup",
                    "composition entrance generation={generation} prepared-root commit completed"
                );
                Ok(EntranceReadiness::Committed(generation))
            }
            AsyncStatus::Canceled | AsyncStatus::Error => {
                let code = action.ErrorCode()?;
                Err(windows::core::Error::from_hresult(code))
            }
            _ => Ok(EntranceReadiness::Pending(generation)),
        }
    }

    pub(super) fn start_prepared_entrance(
        &mut self,
        generation: u64,
        now: std::time::Instant,
    ) -> Result<()> {
        if self.entrance_readiness(generation)? != EntranceReadiness::Committed(generation) {
            return Err(windows::core::Error::new(
                windows::core::HRESULT(0x8000_FFFF_u32 as i32),
                "prepared popup entrance was exposed before its commit receipt",
            ));
        }
        let Some(prepared) = self.entrance.take_committed(generation) else {
            return Ok(());
        };
        let duration = prepared.duration;
        self.entrance_action = None;
        let key = FadeKey {
            phase: 0,
            duration,
            from_opacity_bits: PREWARM_OPACITY.to_bits(),
        };
        self.start_fade(key, now, duration, PREWARM_OPACITY, 1.0)?;
        log::debug!(
            target: "wgpu_l3::native_popup",
            "composition entrance started after exposure from={PREWARM_OPACITY:.3} target=1 duration_us={} application_redraws=0 dwm_flushes=0",
            duration.as_micros()
        );
        Ok(())
    }

    pub(super) fn apply_fade(
        &mut self,
        fade: crate::overlay::PopupFade,
        now: std::time::Instant,
    ) -> Result<()> {
        if stable_update_waits_for_prepared_entrance(fade, self.entrance.prepared.is_some()) {
            log::trace!(
                target: "wgpu_l3::native_popup",
                "deferred stable popup opacity until the concealed prepared entrance is exposed"
            );
            return Ok(());
        }
        let sampled_opacity = fade.opacity_at(now);
        let (key, elapsed, target) = match fade {
            crate::overlay::PopupFade::Entering {
                duration,
                started_at,
            } => (
                FadeKey {
                    phase: 0,
                    duration,
                    from_opacity_bits: PREWARM_OPACITY.to_bits(),
                },
                now.saturating_duration_since(started_at),
                1.0,
            ),
            crate::overlay::PopupFade::Stable => {
                let key = FadeKey {
                    phase: 1,
                    duration: std::time::Duration::ZERO,
                    from_opacity_bits: 1.0_f32.to_bits(),
                };
                if self
                    .fade
                    .is_some_and(|state| state.key.phase == 0 && state.opacity_at(now) < 1.0)
                {
                    return Ok(());
                }
                if self.fade.map(|state| state.key) != Some(key) {
                    self.root.StopAnimation(&HSTRING::from("Opacity"))?;
                    self.root.SetOpacity(1.0)?;
                    self.fade = Some(FadeState {
                        key,
                        started_at: now,
                        duration: std::time::Duration::ZERO,
                        from: 1.0,
                        target: 1.0,
                    });
                }
                return Ok(());
            }
            crate::overlay::PopupFade::Exiting {
                duration,
                started_at,
                from_opacity,
            } => (
                FadeKey {
                    phase: 2,
                    duration,
                    from_opacity_bits: from_opacity.to_bits(),
                },
                now.saturating_duration_since(started_at),
                0.0,
            ),
        };
        if self.fade.map(|state| state.key) == Some(key) {
            return Ok(());
        }

        let remaining = key.duration.saturating_sub(elapsed);
        let start = self
            .fade
            .map(|state| state.opacity_at(now))
            .unwrap_or(sampled_opacity)
            .clamp(0.0, 1.0);
        self.start_fade(key, now, remaining, start, target)?;
        log::debug!(
            target: "wgpu_l3::native_popup",
            "composition fade phase={} from={:.6} target={:.6} remaining_us={} application_redraws=0 dwm_flushes=0",
            key.phase,
            start,
            target,
            remaining.as_micros()
        );
        Ok(())
    }

    fn start_fade(
        &mut self,
        key: FadeKey,
        now: std::time::Instant,
        duration: std::time::Duration,
        from: f32,
        target: f32,
    ) -> Result<()> {
        self.root.StopAnimation(&HSTRING::from("Opacity"))?;
        self.root.SetOpacity(from)?;
        if duration.is_zero() {
            self.root.SetOpacity(target)?;
        } else {
            let animation = self.compositor.CreateScalarKeyFrameAnimation()?;
            animation.InsertKeyFrame(0.0, from)?;
            let easing = self.compositor.CreateCubicBezierEasingFunction(
                Vector2 { X: 0.33, Y: 1.0 },
                Vector2 { X: 0.68, Y: 1.0 },
            )?;
            animation.InsertKeyFrameWithEasingFunction(1.0, target, &easing)?;
            animation.SetDuration(TimeSpan {
                Duration: (duration.as_nanos() / 100).min(i64::MAX as u128) as i64,
            })?;
            self.root
                .StartAnimation(&HSTRING::from("Opacity"), &animation)?;
        }
        self.fade = Some(FadeState {
            key,
            started_at: now,
            duration,
            from,
            target,
        });
        Ok(())
    }

    pub(super) fn sync_material_regions(
        &mut self,
        requests: &[scene::MaterialRegion],
        scale_factor: f32,
        panel_offset_physical: (i32, i32),
        shadow: Option<scene::Shadow>,
    ) -> MaterialSync {
        let started = std::time::Instant::now();
        if !self.host_backdrop_enabled {
            self.clear_material_regions();
            return MaterialSync {
                reports: Vec::new(),
                readiness: MaterialReadiness::NotRequired,
            };
        }

        let desired = requests
            .iter()
            .filter_map(|request| {
                project_region(request, scale_factor, panel_offset_physical)
                    .map(|projected| (request.id(), projected))
            })
            .collect::<Vec<_>>();
        let desired_shadow = shadow
            .zip(desired.first().map(|(_, projected)| *projected))
            .and_then(|(recipe, silhouette)| project_shadow(recipe, silhouette, scale_factor));
        if desired == self.material_projection && desired_shadow == self.shadow_projection {
            let readiness = self.poll_material_readiness();
            return MaterialSync {
                reports: self.material_reports(),
                readiness,
            };
        }

        let prior_count = self.material_regions.len();
        let mut created = 0_usize;
        let mut recycled = 0_usize;
        let mut updated = 0_usize;
        let mut retained = HashSet::new();
        let mut applied = Vec::new();
        let desired_ids = desired.iter().map(|(id, _)| *id).collect::<HashSet<_>>();
        let children = match self.regions.Children() {
            Ok(children) => children,
            Err(error) => {
                log::warn!(target: "wgpu_l3::native_popup", "cannot access composition material-region collection: {error}");
                self.disable_material();
                return MaterialSync {
                    reports: Vec::new(),
                    readiness: MaterialReadiness::NotRequired,
                };
            }
        };
        if let Err(error) = children.RemoveAll() {
            log::warn!(target: "wgpu_l3::native_popup", "cannot reset composition material-region order: {error}");
            self.disable_material();
            return MaterialSync {
                reports: Vec::new(),
                readiness: MaterialReadiness::NotRequired,
            };
        }

        for (id, projected) in desired {
            if !self.material_regions.contains_key(&id) {
                let recycled_region = self.material_spares.pop().or_else(|| {
                    let obsolete = self
                        .material_regions
                        .keys()
                        .find(|candidate| !desired_ids.contains(candidate))
                        .copied()?;
                    self.material_regions.remove(&obsolete)
                });
                let region = if let Some(region) = recycled_region {
                    recycled += 1;
                    region
                } else {
                    match RegionVisual::new(&self.compositor) {
                        Ok(region) => {
                            created += 1;
                            region
                        }
                        Err(error) => {
                            log::warn!(target: "wgpu_l3::native_popup", "material region {id:?} creation failed: {error}");
                            continue;
                        }
                    }
                };
                self.material_regions.insert(id, region);
            } else {
                updated += 1;
            }
            let region = self
                .material_regions
                .get_mut(&id)
                .expect("material region should exist after creation or recycling");
            if let Err(error) = region.apply(projected) {
                log::warn!(target: "wgpu_l3::native_popup", "material region {id:?} update failed: {error}");
                continue;
            }
            if let Err(error) = children.InsertAtTop(&region.visual) {
                log::warn!(target: "wgpu_l3::native_popup", "material region {id:?} ordering failed: {error}");
                continue;
            }
            retained.insert(id);
            applied.push((id, projected));
        }

        let obsolete = self
            .material_regions
            .keys()
            .filter(|id| !retained.contains(id))
            .copied()
            .collect::<Vec<_>>();
        for id in obsolete {
            if let Some(region) = self.material_regions.remove(&id) {
                self.material_spares.push(region);
            }
        }
        let projected_shadow = shadow
            .zip(applied.first().map(|(_, projected)| *projected))
            .and_then(|(recipe, silhouette)| project_shadow(recipe, silhouette, scale_factor));
        if let Err(error) = self.sync_shadow(projected_shadow) {
            log::warn!(target: "wgpu_l3::native_popup", "cannot synchronize composition popup shadow: {error}");
        }
        self.material_projection = applied;
        self.shadow_projection = projected_shadow;
        let readiness = if self.material_projection.is_empty() {
            MaterialReadiness::NotRequired
        } else if created == 0
            && (self.material_commit.is_some() || self.committed_generation.is_some())
        {
            self.poll_material_readiness()
        } else {
            self.begin_material_effect_commit()
        };
        let retired = prior_count
            .saturating_add(created)
            .saturating_sub(self.material_regions.len());
        log::debug!(
            target: "wgpu_l3::native_popup",
            "composition material-region sync requested={} realized={} created={} recycled={} updated={} retired={} spares={} elapsed_us={}",
            requests.len(),
            self.material_projection.len(),
            created,
            recycled,
            updated,
            retired,
            self.material_spares.len(),
            started.elapsed().as_micros()
        );
        MaterialSync {
            reports: self.material_reports(),
            readiness,
        }
    }

    fn clear_material_regions(&mut self) {
        if let Ok(children) = self.regions.Children() {
            let _ = children.RemoveAll();
        }
        self.material_regions.clear();
        self.material_spares.clear();
        self.material_projection.clear();
        self.shadow_projection = None;
        self.material_commit = None;
        self.committed_generation = None;
        let _ = self.shadow.SetOpacity(0.0);
    }

    fn disable_material(&mut self) {
        self.host_backdrop_enabled = false;
        self.clear_material_regions();
    }

    pub(super) fn abandon_material(&mut self) {
        self.disable_material();
    }

    fn material_reports(&self) -> Vec<scene::MaterialRealizationReport> {
        self.material_projection
            .iter()
            .map(|(id, _)| {
                scene::MaterialRealizationReport::new(
                    *id,
                    scene::RealizedMaterialParts::frost(false),
                )
            })
            .collect()
    }

    fn begin_material_effect_commit(&mut self) -> MaterialReadiness {
        let Some(generation) = self.material_generation.checked_add(1) else {
            log::warn!(target: "wgpu_l3::native_popup", "composition material generation exhausted; using framework fallback");
            self.disable_material();
            return MaterialReadiness::NotRequired;
        };
        self.material_generation = generation;
        self.material_commit = None;
        self.committed_generation = None;
        match self
            .compositor
            .GetCommitBatch(CompositionBatchTypes::Effect)
        {
            Ok(batch) => {
                self.material_commit = Some(MaterialCommit { generation, batch });
                MaterialReadiness::Pending(generation)
            }
            Err(error) => {
                log::warn!(target: "wgpu_l3::native_popup", "cannot acquire composition effect receipt; falling back to framework material: {error}");
                self.disable_material();
                MaterialReadiness::NotRequired
            }
        }
    }

    fn poll_material_readiness(&mut self) -> MaterialReadiness {
        let Some(commit) = self.material_commit.as_ref() else {
            return self
                .committed_generation
                .map(MaterialReadiness::Committed)
                .unwrap_or(MaterialReadiness::NotRequired);
        };
        match commit.batch.IsEnded() {
            Ok(true) => {
                let generation = commit.generation;
                self.material_commit = None;
                self.committed_generation = Some(generation);
                log::debug!(target: "wgpu_l3::native_popup", "composition material generation={generation} committed");
                MaterialReadiness::Committed(generation)
            }
            Ok(false) => MaterialReadiness::Pending(commit.generation),
            Err(error) => {
                log::warn!(target: "wgpu_l3::native_popup", "composition effect receipt failed; falling back to framework material: {error}");
                self.disable_material();
                MaterialReadiness::NotRequired
            }
        }
    }

    fn sync_shadow(&self, projected: Option<ProjectedShadow>) -> Result<()> {
        let Some(projected) = projected else {
            self.shadow.SetOpacity(0.0)?;
            return Ok(());
        };
        let (r, g, b, a) = projected.color.channels();
        if a == 0 {
            self.shadow.SetOpacity(0.0)?;
            return Ok(());
        }
        self.shadow_mask_visual.SetSize(projected.mask_size)?;
        self.shadow_mask_geometry.SetSize(projected.mask_size)?;
        self.shadow_mask_geometry.SetCornerRadius(Vector2 {
            X: projected.mask_radius,
            Y: projected.mask_radius,
        })?;
        self.shadow_mask_surface
            .SetSourceSize(projected.mask_size)?;
        self.shadow_mask_brush.SetOffset(projected.mask_offset)?;
        self.shadow.SetBlurRadius(projected.blur_radius)?;
        self.shadow.SetOffset(projected.offset)?;
        self.shadow.SetColor(windows::UI::Color {
            A: 255,
            R: r,
            G: g,
            B: b,
        })?;
        self.shadow.SetOpacity(f32::from(a) / 255.0)
    }
}

fn stable_update_waits_for_prepared_entrance(
    fade: crate::overlay::PopupFade,
    entrance_prepared: bool,
) -> bool {
    entrance_prepared && fade == crate::overlay::PopupFade::Stable
}

fn project_shadow(
    recipe: scene::Shadow,
    silhouette: ProjectedRegion,
    scale_factor: f32,
) -> Option<ProjectedShadow> {
    let (_, _, _, alpha) = recipe.color().channels();
    if alpha == 0 {
        return None;
    }
    let scale_factor = paint::Grid::new(scale_factor).scale_factor();
    let spread = recipe.spread().max(0.0) * scale_factor;
    Some(ProjectedShadow {
        mask_offset: Vector2 {
            X: silhouette.offset.X - spread,
            Y: silhouette.offset.Y - spread,
        },
        mask_size: Vector2 {
            X: silhouette.size.X + spread * 2.0,
            Y: silhouette.size.Y + spread * 2.0,
        },
        mask_radius: silhouette.radius + spread,
        blur_radius: recipe.blur().max(0.0) * scale_factor,
        offset: Vector3 {
            X: recipe.offset().x() * scale_factor,
            Y: recipe.offset().y() * scale_factor,
            Z: 0.0,
        },
        color: recipe.color(),
    })
}

impl Drop for Host {
    fn drop(&mut self) {
        let started = std::time::Instant::now();
        let retained = self.material_regions.len();
        self.clear_material_regions();
        log::debug!(
            target: "wgpu_l3::native_popup",
            "composition host teardown retained_regions={} elapsed_us={}",
            retained,
            started.elapsed().as_micros()
        );
    }
}

impl RegionVisual {
    fn new(compositor: &Compositor) -> Result<Self> {
        let visual = compositor.CreateSpriteVisual()?;
        visual.SetBrush(&compositor.CreateHostBackdropBrush()?)?;
        let geometry = compositor.CreateRoundedRectangleGeometry()?;
        let clip = compositor.CreateGeometricClipWithGeometry(&geometry)?;
        visual.SetClip(&clip)?;
        Ok(Self {
            visual,
            geometry,
            _clip: clip,
        })
    }

    fn apply(&self, projected: ProjectedRegion) -> Result<()> {
        self.visual.SetOffset(projected.offset)?;
        self.visual.SetSize(projected.size)?;
        self.visual.SetOpacity(projected.opacity)?;
        self.geometry.SetSize(projected.size)?;
        self.geometry.SetCornerRadius(Vector2 {
            X: projected.radius,
            Y: projected.radius,
        })
    }
}

fn project_region(
    request: &scene::MaterialRegion,
    scale_factor: f32,
    panel_offset_physical: (i32, i32),
) -> Option<ProjectedRegion> {
    if !matches!(request.material(), scene::Material::Glass(_))
        || !clips_preserve_geometry(request.rect(), request.rounding(), request.clips())
    {
        return None;
    }
    let mut projected = project_geometry(
        request.rect(),
        request.rounding(),
        request.opacity(),
        scale_factor,
    )?;
    projected.offset.X += panel_offset_physical.0 as f32;
    projected.offset.Y += panel_offset_physical.1 as f32;
    Some(projected)
}

fn project_geometry(
    source: crate::geometry::Rect,
    source_rounding: scene::Rounding,
    opacity: f32,
    scale_factor: f32,
) -> Option<ProjectedRegion> {
    let grid = paint::Grid::new(scale_factor);
    let rect = render::scene::into_paint_rounded_rect_at_scale(source, source_rounding, grid);
    let rounding = rect.rounding.resolve(rect.area);
    if !rounding
        .iter()
        .all(|radius| (radius - rounding[0]).abs() <= f32::EPSILON)
    {
        return None;
    }
    let scale = grid.scale_factor();
    Some(ProjectedRegion {
        offset: Vector3 {
            X: rect.origin.x() * scale,
            Y: rect.origin.y() * scale,
            Z: 0.0,
        },
        size: Vector2 {
            X: rect.area.width() * scale,
            Y: rect.area.height() * scale,
        },
        radius: rounding[0] * scale,
        opacity: opacity.clamp(0.0, 1.0),
    })
}

fn clips_preserve_geometry(
    region: crate::geometry::Rect,
    rounding: scene::Rounding,
    clips: &[scene::Clip],
) -> bool {
    clips.iter().all(|clip| {
        let clip_rect = clip.rect();
        let contains = clip_rect.x() <= region.x()
            && clip_rect.y() <= region.y()
            && clip_rect.right() >= region.right()
            && clip_rect.bottom() >= region.bottom();
        contains
            && (clip.rounding() == scene::Rounding::none()
                || (clip_rect == region && clip.rounding() == rounding))
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{geometry, layout, theme::Theme, view};

    fn production_shadow(theme: &Theme) -> scene::Shadow {
        let tree = view::View::new(
            view::Node::root()
                .child(view::Node::floating_panel("panel").child(view::Node::label("row"))),
        );
        let mut engine = layout::Engine::new();
        let layout = layout::Layout::compose_with_theme(
            &tree,
            geometry::Size::new(240, 160),
            &mut engine,
            theme,
        );
        scene::Scene::paint_with_theme(&layout, theme).shadows()[0].to_owned()
    }

    #[test]
    fn region_projection_emits_snapped_physical_geometry_at_all_campaign_scales() {
        let source = geometry::Rect::new(3, 5, 40, 24);
        for scale in [1.0, 1.25, 1.5, 2.0] {
            let projected = project_geometry(source, scene::Rounding::fixed(7.0), 0.65, scale)
                .expect("uniform rounding is representable");
            for edge in [
                projected.offset.X,
                projected.offset.Y,
                projected.offset.X + projected.size.X,
                projected.offset.Y + projected.size.Y,
            ] {
                assert_eq!(edge.fract(), 0.0, "physical edge at scale {scale}");
            }
            assert_eq!(projected.opacity, 0.65);
            assert!((projected.radius - 7.0 * scale).abs() < f32::EPSILON);
        }
    }

    #[test]
    fn desktop_composition_projection_consumes_the_physical_projection() {
        let scale = 1.25;
        let projected = project_geometry(
            geometry::Rect::new(48, 16, 80, 40),
            scene::Rounding::fixed(8.0),
            1.0,
            scale,
        )
        .expect("uniform popup silhouette should project");

        assert_eq!(projected.offset.X, 60.0);
        assert_eq!(projected.offset.Y, 20.0);
        assert_eq!(projected.size.X, 100.0);
        assert_eq!(projected.size.Y, 50.0);
    }

    #[test]
    fn panel_surface_offset_translates_material_without_changing_its_shape() {
        let source = geometry::Rect::new(0, 0, 80, 40);
        let base = project_geometry(source, scene::Rounding::fixed(8.0), 1.0, 1.5)
            .expect("uniform material region should project");
        let mut shifted = base;
        shifted.offset.X += 27.0;
        shifted.offset.Y += 21.0;

        assert_eq!(shifted.size, base.size);
        assert_eq!(shifted.radius, base.radius);
        assert_eq!(shifted.offset.X - base.offset.X, 27.0);
        assert_eq!(shifted.offset.Y - base.offset.Y, 21.0);
    }

    #[test]
    fn production_shadow_projects_from_the_same_rounded_silhouette_at_all_scales() {
        for theme in [Theme::light(), Theme::dark()] {
            let recipe = production_shadow(&theme);
            for scale in [1.0, 1.25, 1.5, 2.0] {
                let silhouette = project_geometry(
                    geometry::Rect::new(0, 0, 240, 160),
                    scene::Rounding::fixed(10.0),
                    1.0,
                    scale,
                )
                .expect("uniform popup silhouette should project");
                let shadow = project_shadow(recipe, silhouette, scale)
                    .expect("production shadow is visible");
                let spread = recipe.spread() * scale;
                assert_eq!(shadow.mask_offset.X, silhouette.offset.X - spread);
                assert_eq!(shadow.mask_offset.Y, silhouette.offset.Y - spread);
                assert_eq!(shadow.mask_size.X, silhouette.size.X + spread * 2.0);
                assert_eq!(shadow.mask_size.Y, silhouette.size.Y + spread * 2.0);
                assert_eq!(shadow.mask_radius, silhouette.radius + spread);
                assert_eq!(shadow.blur_radius, recipe.blur() * scale);
                assert_eq!(shadow.offset.Y, recipe.offset().y() * scale);
            }
        }
    }

    #[test]
    fn compositor_retarget_starts_from_the_prior_timeline_without_a_jump() {
        let now = std::time::Instant::now();
        let retiring = FadeState {
            key: FadeKey {
                phase: 2,
                duration: std::time::Duration::from_millis(100),
                from_opacity_bits: 1.0_f32.to_bits(),
            },
            started_at: now,
            duration: std::time::Duration::from_millis(100),
            from: 1.0,
            target: 0.0,
        };
        let retargeted_at = now + std::time::Duration::from_millis(50);
        let inherited = retiring.opacity_at(retargeted_at);
        let entering = FadeState {
            key: FadeKey {
                phase: 0,
                duration: std::time::Duration::from_millis(80),
                from_opacity_bits: 0.0_f32.to_bits(),
            },
            started_at: retargeted_at,
            duration: std::time::Duration::from_millis(80),
            from: inherited,
            target: 1.0,
        };

        assert_eq!(entering.opacity_at(retargeted_at), inherited);
        assert!(inherited > 0.0 && inherited < 1.0);
        assert_eq!(
            entering.opacity_at(retargeted_at + std::time::Duration::from_millis(80)),
            1.0
        );
    }

    #[test]
    fn logical_stability_cannot_overwrite_a_concealed_prepared_entrance() {
        let stable = crate::overlay::PopupFade::Stable;
        let entering = crate::overlay::PopupFade::Entering {
            duration: std::time::Duration::from_millis(100),
            started_at: std::time::Instant::now(),
        };

        assert!(stable_update_waits_for_prepared_entrance(stable, true));
        assert!(!stable_update_waits_for_prepared_entrance(stable, false));
        assert!(!stable_update_waits_for_prepared_entrance(entering, true));
    }

    #[test]
    fn prepared_entrance_requires_its_exact_generation_receipt() {
        let mut entrance = EntranceState::default();
        let first = PreparedEntrance {
            generation: 41,
            duration: std::time::Duration::from_millis(100),
        };
        entrance.begin(first);

        assert_eq!(entrance.readiness_for(41), EntranceReadiness::Pending(41));
        assert!(entrance.readiness_for(41).waits_for_receipt());
        assert!(!entrance.mark_committed(40), "a stale receipt is inert");
        assert_eq!(entrance.readiness_for(41), EntranceReadiness::Pending(41));
        assert!(entrance.mark_committed(41));
        assert_eq!(entrance.readiness_for(41), EntranceReadiness::Committed(41));
        assert!(!entrance.readiness_for(41).waits_for_receipt());
        assert_eq!(entrance.take_committed(41), Some(first));
        assert_eq!(entrance.readiness_for(41), EntranceReadiness::NotRequired);
    }

    #[test]
    fn reused_host_cannot_accept_an_earlier_entrance_receipt() {
        let mut entrance = EntranceState::default();
        entrance.begin(PreparedEntrance {
            generation: 7,
            duration: std::time::Duration::from_millis(100),
        });
        assert!(entrance.mark_committed(7));
        assert_eq!(
            entrance.readiness_for(8),
            EntranceReadiness::Pending(7),
            "a committed prior tenant must not unlock the next popup generation"
        );
        assert_eq!(entrance.take_committed(8), None);

        let replacement = PreparedEntrance {
            generation: 8,
            duration: std::time::Duration::from_millis(120),
        };
        entrance.begin(replacement);
        assert!(!entrance.mark_committed(7), "late prior receipt is inert");
        assert_eq!(entrance.readiness_for(8), EntranceReadiness::Pending(8));
        assert!(entrance.mark_committed(8));
        assert_eq!(entrance.take_committed(8), Some(replacement));
    }

    #[test]
    fn per_corner_rounding_and_cutting_clips_decline_only_that_region() {
        let region = geometry::Rect::new(10, 10, 80, 40);
        let per_corner = scene::Rounding::new(
            scene::Radius::Fixed(4.0),
            scene::Radius::Fixed(8.0),
            scene::Radius::Fixed(4.0),
            scene::Radius::Fixed(8.0),
        );
        assert!(project_geometry(region, per_corner, 1.0, 1.0).is_none());

        let containing = scene::Clip::new(geometry::Rect::new(0, 0, 100, 60));
        assert!(clips_preserve_geometry(
            region,
            scene::Rounding::fixed(8.0),
            &[containing]
        ));
        let cutting = scene::Clip::new(geometry::Rect::new(20, 10, 70, 40));
        assert!(!clips_preserve_geometry(
            region,
            scene::Rounding::fixed(8.0),
            &[cutting]
        ));
    }
}

fn hwnd(window: &winit::window::Window) -> Result<HWND> {
    let handle = window
        .window_handle()
        .map_err(|_| windows::core::Error::from(E_HANDLE))?;
    match handle.as_raw() {
        RawWindowHandle::Win32(handle) => Ok(HWND(handle.hwnd.get() as *mut _)),
        _ => Err(windows::core::Error::from(E_HANDLE)),
    }
}
