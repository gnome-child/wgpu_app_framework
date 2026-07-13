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
    material_projection: Vec<(composition::NodeId, ProjectedRegion)>,
    shadow_projection: Option<ProjectedShadow>,
    material_generation: u64,
    material_commit: Option<MaterialCommit>,
    committed_generation: Option<u64>,
    fade: Option<FadeState>,
    pending_entrance: Option<std::time::Duration>,
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
            material_projection: Vec::new(),
            shadow_projection: None,
            material_generation: 0,
            material_commit: None,
            committed_generation: None,
            fade: None,
            pending_entrance: None,
        })
    }
}

impl SurfaceSeed {
    pub(super) fn target(&self) -> wgpu::SurfaceTargetUnsafe {
        wgpu::SurfaceTargetUnsafe::CompositionVisual(self.visual.as_raw())
    }
}

impl Host {
    pub(super) fn prepare_entrance(&mut self, duration: std::time::Duration) -> Result<()> {
        if self.pending_entrance == Some(duration) {
            return Ok(());
        }
        self.root.StopAnimation(&HSTRING::from("Opacity"))?;
        self.root.SetOpacity(PREWARM_OPACITY)?;
        self.fade = None;
        self.pending_entrance = Some(duration);
        log::debug!(
            target: "wgpu_l3::native_popup",
            "composition entrance prepared opacity={PREWARM_OPACITY:.3} duration_us={} application_redraws=0 dwm_flushes=0",
            duration.as_micros()
        );
        Ok(())
    }

    pub(super) fn start_prepared_entrance(&mut self, now: std::time::Instant) -> Result<()> {
        let Some(duration) = self.pending_entrance.take() else {
            return Ok(());
        };
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
        ancestor_opacity: f32,
        panel_offset_dips: paint::point::Logical,
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
                project_region(request, scale_factor, ancestor_opacity, panel_offset_dips)
                    .map(|projected| (request.id(), projected))
            })
            .collect::<Vec<_>>();
        let desired_shadow = shadow
            .zip(desired.first().map(|(_, projected)| *projected))
            .and_then(|(recipe, silhouette)| project_shadow(recipe, silhouette));
        if desired == self.material_projection && desired_shadow == self.shadow_projection {
            let readiness = self.poll_material_readiness();
            return MaterialSync {
                reports: self.material_reports(),
                readiness,
            };
        }

        let prior_count = self.material_regions.len();
        let mut created = 0_usize;
        let mut updated = 0_usize;
        let mut retained = HashSet::new();
        let mut applied = Vec::new();
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
            let region = if let Some(region) = self.material_regions.get_mut(&id) {
                updated += 1;
                region
            } else {
                match RegionVisual::new(&self.compositor) {
                    Ok(region) => {
                        created += 1;
                        self.material_regions.entry(id).or_insert(region)
                    }
                    Err(error) => {
                        log::warn!(target: "wgpu_l3::native_popup", "material region {id:?} creation failed: {error}");
                        continue;
                    }
                }
            };
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

        self.material_regions.retain(|id, _| retained.contains(id));
        let projected_shadow = shadow
            .zip(applied.first().map(|(_, projected)| *projected))
            .and_then(|(recipe, silhouette)| project_shadow(recipe, silhouette));
        if let Err(error) = self.sync_shadow(projected_shadow) {
            log::warn!(target: "wgpu_l3::native_popup", "cannot synchronize composition popup shadow: {error}");
        }
        self.material_projection = applied;
        self.shadow_projection = projected_shadow;
        let Some(generation) = self.material_generation.checked_add(1) else {
            log::warn!(target: "wgpu_l3::native_popup", "composition material generation exhausted; using framework fallback");
            self.disable_material();
            return MaterialSync {
                reports: Vec::new(),
                readiness: MaterialReadiness::NotRequired,
            };
        };
        self.material_generation = generation;
        self.material_commit = None;
        self.committed_generation = None;
        let readiness = if self.material_projection.is_empty() {
            MaterialReadiness::NotRequired
        } else {
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
        };
        let removed = prior_count
            .saturating_add(created)
            .saturating_sub(self.material_regions.len());
        log::debug!(
            target: "wgpu_l3::native_popup",
            "composition material-region sync requested={} realized={} created={} updated={} removed={} elapsed_us={}",
            requests.len(),
            self.material_projection.len(),
            created,
            updated,
            removed,
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

fn project_shadow(recipe: scene::Shadow, silhouette: ProjectedRegion) -> Option<ProjectedShadow> {
    let (_, _, _, alpha) = recipe.color().channels();
    if alpha == 0 {
        return None;
    }
    let spread = recipe.spread().max(0.0);
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
        blur_radius: recipe.blur().max(0.0),
        offset: Vector3 {
            X: recipe.offset().x(),
            Y: recipe.offset().y(),
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
    ancestor_opacity: f32,
    panel_offset_dips: paint::point::Logical,
) -> Option<ProjectedRegion> {
    if !matches!(request.material(), scene::Material::Glass(_))
        || !clips_preserve_geometry(request.rect(), request.rounding(), request.clips())
    {
        return None;
    }
    let mut projected = project_geometry(
        request.rect(),
        request.rounding(),
        if ancestor_opacity > f32::EPSILON {
            request.opacity() / ancestor_opacity
        } else {
            0.0
        },
        scale_factor,
    )?;
    projected.offset.X += panel_offset_dips.x();
    projected.offset.Y += panel_offset_dips.y();
    Some(projected)
}

fn project_geometry(
    source: crate::geometry::Rect,
    source_rounding: scene::Rounding,
    opacity: f32,
    scale_factor: f32,
) -> Option<ProjectedRegion> {
    let grid = paint::Grid::new(scale_factor);
    let rect = super::paint::into_paint_rounded_rect_at_scale(source, source_rounding, grid);
    let rounding = rect.rounding.resolve(rect.area);
    if !rounding
        .iter()
        .all(|radius| (radius - rounding[0]).abs() <= f32::EPSILON)
    {
        return None;
    }
    Some(ProjectedRegion {
        offset: Vector3 {
            X: rect.origin.x(),
            Y: rect.origin.y(),
            Z: 0.0,
        },
        size: Vector2 {
            X: rect.area.width(),
            Y: rect.area.height(),
        },
        radius: rounding[0],
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
    fn region_projection_emits_snapped_dips_at_all_campaign_scales() {
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
                assert_eq!(
                    (edge * scale).fract(),
                    0.0,
                    "DIP edge resolves to a physical pixel at scale {scale}"
                );
            }
            assert_eq!(projected.opacity, 0.65);
            assert!((projected.radius - 7.0).abs() < f32::EPSILON);
        }
    }

    #[test]
    fn composition_projection_never_reuses_the_physical_projection() {
        let scale = 1.25;
        let projected = project_geometry(
            geometry::Rect::new(48, 16, 80, 40),
            scene::Rounding::fixed(8.0),
            1.0,
            scale,
        )
        .expect("uniform popup silhouette should project");

        assert_eq!(projected.offset.X, 48.0, "Composition consumes DIPs");
        assert_eq!(
            projected.offset.X * scale,
            60.0,
            "48 DIPs resolve to the intended 60 physical pixels"
        );
        assert_ne!(
            60.0 * scale,
            projected.offset.X * scale,
            "the renderer's 60px projection must not be fed back as 60 DIPs"
        );
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
                let shadow =
                    project_shadow(recipe, silhouette).expect("production shadow is visible");
                let spread = recipe.spread();
                assert_eq!(shadow.mask_offset.X, silhouette.offset.X - spread);
                assert_eq!(shadow.mask_offset.Y, silhouette.offset.Y - spread);
                assert_eq!(shadow.mask_size.X, silhouette.size.X + spread * 2.0);
                assert_eq!(shadow.mask_size.Y, silhouette.size.Y + spread * 2.0);
                assert_eq!(shadow.mask_radius, silhouette.radius + spread);
                assert_eq!(shadow.blur_radius, recipe.blur());
                assert_eq!(shadow.offset.Y, recipe.offset().y());
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
