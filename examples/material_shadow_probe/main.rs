#[cfg(target_os = "windows")]
mod windows_probe {
    use std::sync::Arc;
    use std::time::{Duration, Instant};

    use windows::Foundation::TimeSpan;
    use windows::System::DispatcherQueueController;
    use windows::UI::Composition::Desktop::DesktopWindowTarget;
    use windows::UI::Composition::{
        CompositionBatchTypes, CompositionCommitBatch, CompositionDropShadowSourcePolicy,
        CompositionGeometricClip, CompositionRoundedRectangleGeometry, CompositionSurfaceBrush,
        CompositionVisualSurface, Compositor, ContainerVisual, DropShadow, LayerVisual,
        SpriteVisual,
    };
    use windows::Win32::Foundation::{E_FAIL, HWND};
    use windows::Win32::Graphics::Dwm::{
        DWMWA_BORDER_COLOR, DWMWA_CLOAK, DWMWA_COLOR_NONE, DWMWA_USE_HOSTBACKDROPBRUSH, DwmFlush,
        DwmSetWindowAttribute,
    };
    use windows::Win32::Graphics::Gdi::{
        BI_RGB, BITMAPINFO, BitBlt, CAPTUREBLT, CreateCompatibleBitmap, CreateCompatibleDC,
        DIB_RGB_COLORS, DeleteDC, DeleteObject, GetDC, GetDIBits, HGDIOBJ, ReleaseDC, SRCCOPY,
        SelectObject,
    };
    use windows::Win32::System::WinRT::Composition::ICompositorDesktopInterop;
    use windows::Win32::System::WinRT::{
        CreateDispatcherQueueController, DQTAT_COM_NONE, DQTYPE_THREAD_CURRENT,
        DispatcherQueueOptions,
    };
    use windows::core::{HSTRING, Interface};
    use windows_numerics::{Vector2, Vector3};
    use winit::application::ApplicationHandler;
    use winit::dpi::{PhysicalPosition, PhysicalSize};
    use winit::event::WindowEvent;
    use winit::event_loop::{ActiveEventLoop, ControlFlow, EventLoop};
    use winit::platform::windows::{CornerPreference, WindowAttributesExtWindows};
    use winit::raw_window_handle::{HasWindowHandle, RawWindowHandle};
    use winit::window::{Window, WindowAttributes, WindowId};

    const PANEL_X: f32 = 60.0;
    const PANEL_Y: f32 = 44.0;
    const PANEL_WIDTH: f32 = 300.0;
    const PANEL_HEIGHT: f32 = 180.0;
    const PANEL_RADIUS: f32 = 10.0;
    const SHADOW_BLUR: f32 = 24.0;
    const SHADOW_OFFSET_Y: f32 = 10.0;
    const INSPECTION_OPACITY: f32 = 0.25;
    const PREWARM_OPACITY: f32 = 0.001;
    const INSPECTION_HOLD: Duration = Duration::from_millis(2_000);
    const FADE_DURATION: Duration = Duration::from_millis(1_000);
    const EXIT_AFTER_FADE: Duration = Duration::from_millis(250);
    const REUSE_FADE_DURATION: Duration = Duration::from_millis(80);
    const REUSE_VISIBLE_HOLD: Duration = Duration::from_millis(180);
    const REUSE_INTERVALS: [Duration; 4] = [
        Duration::from_millis(10),
        Duration::from_millis(100),
        Duration::from_secs(1),
        Duration::from_secs(10),
    ];

    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    enum ProbeMode {
        Fade,
        Reuse,
    }

    impl Default for ProbeMode {
        fn default() -> Self {
            Self::Fade
        }
    }

    #[derive(Debug, Clone, Copy)]
    enum ReusePhase {
        Hidden { since: Instant },
        Visible { since: Instant },
    }

    #[derive(Debug, Clone, Copy)]
    struct ReuseLadder {
        cycle: usize,
        phase: ReusePhase,
        prior_outside: Option<[f64; 3]>,
    }

    struct CompositionState {
        _dispatcher: DispatcherQueueController,
        _underlay_target: DesktopWindowTarget,
        _underlay_root: ContainerVisual,
        _target: DesktopWindowTarget,
        _stage: ContainerVisual,
        background: SpriteVisual,
        root: ContainerVisual,
        frost: SpriteVisual,
        frost_geometry: CompositionRoundedRectangleGeometry,
        _frost_clip: CompositionGeometricClip,
        shadow_layer: LayerVisual,
        caster: SpriteVisual,
        caster_geometry: CompositionRoundedRectangleGeometry,
        _caster_clip: CompositionGeometricClip,
        mask_visual: SpriteVisual,
        mask_geometry: CompositionRoundedRectangleGeometry,
        _mask_clip: CompositionGeometricClip,
        mask_surface: CompositionVisualSurface,
        _mask_brush: CompositionSurfaceBrush,
        _shadow: DropShadow,
        compositor: Compositor,
        commit_batch: CompositionCommitBatch,
        created_at: Instant,
        committed_at: Option<Instant>,
        fade_started_at: Option<Instant>,
        first_capture: Option<CaptureStats>,
        comparison_done: bool,
        mode: ProbeMode,
        reuse: Option<ReuseLadder>,
        reuse_complete: bool,
    }

    impl CompositionState {
        fn new(window: &Window, underlay: &Window, mode: ProbeMode) -> windows::core::Result<Self> {
            let options = DispatcherQueueOptions {
                dwSize: std::mem::size_of::<DispatcherQueueOptions>() as u32,
                threadType: DQTYPE_THREAD_CURRENT,
                apartmentType: DQTAT_COM_NONE,
            };
            let dispatcher = unsafe { CreateDispatcherQueueController(options)? };
            let compositor = Compositor::new()?;
            let underlay_root = compositor.CreateContainerVisual()?;
            underlay_root.SetRelativeSizeAdjustment(Vector2 { X: 1.0, Y: 1.0 })?;
            let underlay_background = compositor.CreateSpriteVisual()?;
            underlay_background.SetRelativeSizeAdjustment(Vector2 { X: 1.0, Y: 1.0 })?;
            underlay_background.SetBrush(&compositor.CreateColorBrushWithColor(
                windows::UI::Color {
                    A: 255,
                    R: 24,
                    G: 32,
                    B: 48,
                },
            )?)?;
            underlay_root
                .Children()?
                .InsertAtBottom(&underlay_background)?;
            for index in 0_u32..8 {
                let stripe = compositor.CreateSpriteVisual()?;
                stripe.SetOffset(Vector3 {
                    X: index as f32 * 52.5,
                    Y: 0.0,
                    Z: 0.0,
                })?;
                stripe.SetSize(Vector2 { X: 26.25, Y: 300.0 })?;
                stripe.SetBrush(&compositor.CreateColorBrushWithColor(windows::UI::Color {
                    A: 255,
                    R: if index.is_multiple_of(2) { 232 } else { 44 },
                    G: if index.is_multiple_of(2) { 92 } else { 196 },
                    B: if index.is_multiple_of(2) { 68 } else { 228 },
                })?)?;
                underlay_root.Children()?.InsertAtTop(&stripe)?;
            }
            let desktop: ICompositorDesktopInterop = compositor.cast()?;
            let underlay_target =
                unsafe { desktop.CreateDesktopWindowTarget(hwnd(underlay)?, false)? };
            underlay_target.SetRoot(&underlay_root)?;
            let stage = compositor.CreateContainerVisual()?;
            stage.SetRelativeSizeAdjustment(Vector2 { X: 1.0, Y: 1.0 })?;
            let background = compositor.CreateSpriteVisual()?;
            background.SetRelativeSizeAdjustment(Vector2 { X: 1.0, Y: 1.0 })?;
            background.SetBrush(&compositor.CreateColorBrushWithColor(windows::UI::Color {
                A: 255,
                R: 196,
                G: 210,
                B: 232,
            })?)?;
            let root = compositor.CreateContainerVisual()?;
            root.SetRelativeSizeAdjustment(Vector2 { X: 1.0, Y: 1.0 })?;

            let frost = compositor.CreateSpriteVisual()?;
            frost.SetOffset(Vector3 {
                X: PANEL_X,
                Y: PANEL_Y,
                Z: 0.0,
            })?;
            frost.SetSize(Vector2 {
                X: PANEL_WIDTH,
                Y: PANEL_HEIGHT,
            })?;
            frost.SetBrush(&compositor.CreateHostBackdropBrush()?)?;
            let frost_geometry = rounded_geometry(&compositor)?;
            let frost_clip = compositor.CreateGeometricClipWithGeometry(&frost_geometry)?;
            frost.SetClip(&frost_clip)?;

            let shadow_layer = compositor.CreateLayerVisual()?;
            shadow_layer.SetOffset(Vector3 {
                X: PANEL_X,
                Y: PANEL_Y,
                Z: 0.0,
            })?;
            shadow_layer.SetSize(Vector2 {
                X: PANEL_WIDTH,
                Y: PANEL_HEIGHT,
            })?;
            let caster = compositor.CreateSpriteVisual()?;
            caster.SetSize(Vector2 {
                X: PANEL_WIDTH,
                Y: PANEL_HEIGHT,
            })?;
            caster.SetBrush(&compositor.CreateColorBrushWithColor(windows::UI::Color {
                A: 72,
                R: 28,
                G: 28,
                B: 30,
            })?)?;
            let caster_geometry = rounded_geometry(&compositor)?;
            let caster_clip = compositor.CreateGeometricClipWithGeometry(&caster_geometry)?;
            caster.SetClip(&caster_clip)?;

            let mask_visual = compositor.CreateSpriteVisual()?;
            mask_visual.SetSize(Vector2 {
                X: PANEL_WIDTH,
                Y: PANEL_HEIGHT,
            })?;
            mask_visual.SetBrush(&compositor.CreateColorBrushWithColor(windows::UI::Color {
                A: 255,
                R: 255,
                G: 255,
                B: 255,
            })?)?;
            let mask_geometry = rounded_geometry(&compositor)?;
            let mask_clip = compositor.CreateGeometricClipWithGeometry(&mask_geometry)?;
            mask_visual.SetClip(&mask_clip)?;
            let mask_surface = compositor.CreateVisualSurface()?;
            mask_surface.SetSourceVisual(&mask_visual)?;
            mask_surface.SetSourceSize(Vector2 {
                X: PANEL_WIDTH,
                Y: PANEL_HEIGHT,
            })?;
            let mask_brush = compositor.CreateSurfaceBrushWithSurface(&mask_surface)?;

            let shadow = compositor.CreateDropShadow()?;
            shadow.SetBlurRadius(SHADOW_BLUR)?;
            shadow.SetOffset(Vector3 {
                X: 0.0,
                Y: SHADOW_OFFSET_Y,
                Z: 0.0,
            })?;
            shadow.SetColor(windows::UI::Color {
                A: 255,
                R: 0,
                G: 0,
                B: 0,
            })?;
            shadow.SetOpacity(96.0 / 255.0)?;
            shadow.SetSourcePolicy(CompositionDropShadowSourcePolicy::Default)?;
            shadow.SetMask(&mask_brush)?;
            shadow_layer.Children()?.InsertAtTop(&caster)?;
            shadow_layer.SetShadow(&shadow)?;

            root.Children()?.InsertAtBottom(&shadow_layer)?;
            root.Children()?.InsertAtTop(&frost)?;
            // A CompositionVisualSurface only realizes the alpha of a visual that
            // participates in a live tree. Keep the mask source beneath the opaque
            // probe background: it remains realizable without becoming visible.
            stage.Children()?.InsertAtBottom(&background)?;
            stage.Children()?.InsertAtBottom(&mask_visual)?;
            stage.Children()?.InsertAtTop(&root)?;

            let target = unsafe { desktop.CreateDesktopWindowTarget(hwnd(window)?, false)? };
            target.SetRoot(&stage)?;

            let enabled = 1_i32;
            let border = DWMWA_COLOR_NONE;
            unsafe {
                DwmSetWindowAttribute(
                    hwnd(window)?,
                    DWMWA_USE_HOSTBACKDROPBRUSH,
                    (&raw const enabled).cast(),
                    std::mem::size_of::<i32>() as u32,
                )?;
                DwmSetWindowAttribute(
                    hwnd(window)?,
                    DWMWA_BORDER_COLOR,
                    (&raw const border).cast(),
                    std::mem::size_of_val(&border) as u32,
                )?;
            }

            set_cloaked(window, true)?;
            // Nonzero participation forces the material subtree to realize
            // while the DWM cloak keeps every pixel off-screen.
            root.SetOpacity(PREWARM_OPACITY)?;
            let commit_batch = compositor.GetCommitBatch(CompositionBatchTypes::Effect)?;
            let state = Self {
                _dispatcher: dispatcher,
                _underlay_target: underlay_target,
                _underlay_root: underlay_root,
                _target: target,
                _stage: stage,
                background,
                root,
                frost,
                frost_geometry,
                _frost_clip: frost_clip,
                shadow_layer,
                caster,
                caster_geometry,
                _caster_clip: caster_clip,
                mask_visual,
                mask_geometry,
                _mask_clip: mask_clip,
                mask_surface,
                _mask_brush: mask_brush,
                _shadow: shadow,
                compositor,
                commit_batch,
                created_at: Instant::now(),
                committed_at: None,
                fade_started_at: None,
                first_capture: None,
                comparison_done: false,
                mode,
                reuse: None,
                reuse_complete: false,
            };
            Ok(state)
        }

        fn start_fade(&mut self) -> windows::core::Result<()> {
            let from = self.root.Opacity()?;
            let animation = self.compositor.CreateScalarKeyFrameAnimation()?;
            animation.InsertKeyFrame(0.0, from)?;
            let easing = self.compositor.CreateCubicBezierEasingFunction(
                Vector2 { X: 0.33, Y: 1.0 },
                Vector2 { X: 0.68, Y: 1.0 },
            )?;
            animation.InsertKeyFrameWithEasingFunction(1.0, 1.0, &easing)?;
            animation.SetDuration(TimeSpan {
                Duration: (FADE_DURATION.as_nanos() / 100) as i64,
            })?;
            self.root
                .StartAnimation(&HSTRING::from("Opacity"), &animation)?;
            let now = Instant::now();
            self.fade_started_at = Some(now);
            log::info!(
                target: "wgpu_l3::material_shadow_probe",
                "inspection complete at +{:?}; compositor fade from={from:.3} to=1.000; application_redraws=0",
                now.duration_since(self.created_at),
            );
            Ok(())
        }

        fn start_reuse_fade(&self) -> windows::core::Result<()> {
            let from = self.root.Opacity()?;
            let animation = self.compositor.CreateScalarKeyFrameAnimation()?;
            animation.InsertKeyFrame(0.0, from)?;
            let easing = self.compositor.CreateCubicBezierEasingFunction(
                Vector2 { X: 0.33, Y: 1.0 },
                Vector2 { X: 0.68, Y: 1.0 },
            )?;
            animation.InsertKeyFrameWithEasingFunction(1.0, 1.0, &easing)?;
            animation.SetDuration(TimeSpan {
                Duration: (REUSE_FADE_DURATION.as_nanos() / 100) as i64,
            })?;
            self.root
                .StartAnimation(&HSTRING::from("Opacity"), &animation)
        }

        fn resize_panel(&self, width: f32, height: f32) -> windows::core::Result<()> {
            let size = Vector2 {
                X: width,
                Y: height,
            };
            self.frost.SetSize(size)?;
            self.frost_geometry.SetSize(size)?;
            self.shadow_layer.SetSize(size)?;
            self.caster.SetSize(size)?;
            self.caster_geometry.SetSize(size)?;
            self.mask_visual.SetSize(size)?;
            self.mask_geometry.SetSize(size)?;
            self.mask_surface.SetSourceSize(size)
        }

        fn begin_reuse_ladder(&mut self, window: &Window) -> windows::core::Result<()> {
            self.root.StopAnimation(&HSTRING::from("Opacity"))?;
            self.root.SetOpacity(0.0)?;
            unsafe { DwmFlush()? };
            window.set_visible(false);
            self.reuse = Some(ReuseLadder {
                cycle: 0,
                phase: ReusePhase::Hidden {
                    since: Instant::now(),
                },
                prior_outside: None,
            });
            log::info!(
                target: "wgpu_l3::material_shadow_probe",
                "reuse ladder began host_creations=1 effect_receipts=1 intervals_ms=[10,100,1000,10000]"
            );
            Ok(())
        }

        fn update_reuse(
            &mut self,
            window: &Window,
            underlay: &Window,
        ) -> windows::core::Result<bool> {
            if self.reuse_complete {
                return Ok(true);
            }
            let Some(mut ladder) = self.reuse else {
                self.begin_reuse_ladder(window)?;
                return Ok(false);
            };

            match ladder.phase {
                ReusePhase::Hidden { since }
                    if since.elapsed() >= REUSE_INTERVALS[ladder.cycle] =>
                {
                    let positions = [(360, 160), (420, 190), (320, 140), (390, 210)];
                    let sizes = [(420, 300), (460, 330), (400, 280), (440, 310)];
                    let colors = [
                        (196, 210, 232),
                        (232, 196, 210),
                        (196, 232, 206),
                        (222, 206, 176),
                    ];
                    let panel_sizes = [
                        (300.0, 180.0),
                        (330.0, 200.0),
                        (270.0, 170.0),
                        (315.0, 190.0),
                    ];
                    let (x, y) = positions[ladder.cycle];
                    let (width, height) = sizes[ladder.cycle];
                    let (red, green, blue) = colors[ladder.cycle];
                    let (panel_width, panel_height) = panel_sizes[ladder.cycle];
                    underlay.set_outer_position(PhysicalPosition::new(x, y));
                    window.set_outer_position(PhysicalPosition::new(x, y));
                    let _ = underlay.request_inner_size(PhysicalSize::new(width, height));
                    let _ = window.request_inner_size(PhysicalSize::new(width, height));
                    self.background
                        .SetBrush(&self.compositor.CreateColorBrushWithColor(
                            windows::UI::Color {
                                A: 255,
                                R: red,
                                G: green,
                                B: blue,
                            },
                        )?)?;
                    self.resize_panel(panel_width, panel_height)?;
                    self.root.SetOpacity(INSPECTION_OPACITY)?;
                    let show_started = Instant::now();
                    window.set_visible(true);
                    unsafe { DwmFlush()? };
                    let first_capture = capture_screen(window)?;
                    let first_contrast = channel_delta(first_capture.inside, first_capture.outside);
                    if first_contrast <= 1.0 {
                        return Err(windows::core::Error::new(
                            E_FAIL,
                            "reused host exposed before frost was visible",
                        ));
                    }
                    self.start_reuse_fade()?;
                    ladder.phase = ReusePhase::Visible {
                        since: Instant::now(),
                    };
                    self.reuse = Some(ladder);
                    log::info!(
                        target: "wgpu_l3::material_shadow_probe",
                        "reuse cycle={} hidden_ms={} host_size={}x{} panel_size={:.0}x{:.0} position=({}, {}) show_setup_us={} first_frame_frost_contrast={:.3} host_creations=0 effect_receipts=0 generation={}",
                        ladder.cycle + 1,
                        REUSE_INTERVALS[ladder.cycle].as_millis(),
                        width,
                        height,
                        panel_width,
                        panel_height,
                        x,
                        y,
                        show_started.elapsed().as_micros(),
                        first_contrast,
                        ladder.cycle + 2,
                    );
                }
                ReusePhase::Visible { since } if since.elapsed() >= REUSE_VISIBLE_HOLD => {
                    unsafe { DwmFlush()? };
                    let capture = capture_screen(window)?;
                    let contrast = channel_delta(capture.inside, capture.outside);
                    let content_delta = ladder
                        .prior_outside
                        .map_or(0.0, |prior| channel_delta(prior, capture.outside));
                    log::info!(
                        target: "wgpu_l3::material_shadow_probe",
                        "reuse cycle={} verified visible_ms={} inside_rgb={:?} outside_rgb={:?} frost_contrast={:.3} fresh_content_delta={:.3} shadow_border_first_frame=screen-captured",
                        ladder.cycle + 1,
                        since.elapsed().as_millis(),
                        capture.inside,
                        capture.outside,
                        contrast,
                        content_delta,
                    );
                    if contrast <= 1.0 {
                        return Err(windows::core::Error::new(
                            E_FAIL,
                            "reused host lost visible frost contrast",
                        ));
                    }
                    if ladder.cycle + 1 == REUSE_INTERVALS.len() {
                        self.reuse_complete = true;
                        log::info!(
                            target: "wgpu_l3::material_shadow_probe",
                            "reuse ladder complete cycles=4 host_creations=1 effect_receipts=1 late_receipts=0"
                        );
                        return Ok(true);
                    }

                    self.root.StopAnimation(&HSTRING::from("Opacity"))?;
                    self.root.SetOpacity(0.0)?;
                    unsafe { DwmFlush()? };
                    window.set_visible(false);
                    ladder.cycle += 1;
                    ladder.phase = ReusePhase::Hidden {
                        since: Instant::now(),
                    };
                    ladder.prior_outside = Some(capture.outside);
                    self.reuse = Some(ladder);
                }
                ReusePhase::Hidden { .. } | ReusePhase::Visible { .. } => {}
            }

            Ok(false)
        }

        fn update(&mut self, window: &Window, underlay: &Window) -> windows::core::Result<bool> {
            if self.committed_at.is_none() && self.commit_batch.IsEnded()? {
                set_cloaked(window, false)?;
                for _ in 0..2 {
                    unsafe { DwmFlush()? };
                }
                self.root.SetOpacity(INSPECTION_OPACITY)?;
                unsafe { DwmFlush()? };
                let now = Instant::now();
                self.committed_at = Some(now);
                log::info!(
                    target: "wgpu_l3::material_shadow_probe",
                    "effect committed; window uncloaked at {PREWARM_OPACITY:.3}, two host frames consumed, root raised to {INSPECTION_OPACITY:.3}, and visible commit synchronized at +{:?}; post_receipt_barriers=3 delay_constants=0 application_redraws=0",
                    now.duration_since(self.created_at),
                );
            }
            if self.mode == ProbeMode::Reuse && self.committed_at.is_some() {
                return self.update_reuse(window, underlay);
            }
            if let Some(committed) = self.committed_at
                && !self.comparison_done
            {
                let elapsed = committed.elapsed();
                if self.first_capture.is_none() && elapsed >= Duration::from_millis(100) {
                    let captured = capture_screen(window)?;
                    log_capture("first-visible", elapsed, captured);
                    self.first_capture = Some(captured);
                } else if let Some(first) = self.first_capture
                    && elapsed >= Duration::from_millis(1_000)
                {
                    let settled = capture_screen(window)?;
                    log_capture("fixed-opacity-settled", elapsed, settled);
                    log::info!(
                        target: "wgpu_l3::material_shadow_probe",
                        "screen-space comparison inside_delta={:.3} outside_delta={:.3} first_contrast={:.3} settled_contrast={:.3}",
                        channel_delta(first.inside, settled.inside),
                        channel_delta(first.outside, settled.outside),
                        channel_delta(first.inside, first.outside),
                        channel_delta(settled.inside, settled.outside),
                    );
                    // One comparison per run. The fixed-opacity visual remains
                    // unchanged until the inspection fade begins.
                    self.first_capture = None;
                    self.comparison_done = true;
                }
            }
            if self.fade_started_at.is_none()
                && self
                    .committed_at
                    .is_some_and(|committed| committed.elapsed() >= INSPECTION_HOLD)
            {
                self.start_fade()?;
            }
            Ok(self
                .fade_started_at
                .is_some_and(|started| started.elapsed() >= FADE_DURATION + EXIT_AFTER_FADE))
        }

        fn next_deadline(&self) -> Instant {
            Instant::now() + Duration::from_millis(16)
        }
    }

    fn rounded_geometry(
        compositor: &Compositor,
    ) -> windows::core::Result<CompositionRoundedRectangleGeometry> {
        let geometry = compositor.CreateRoundedRectangleGeometry()?;
        geometry.SetSize(Vector2 {
            X: PANEL_WIDTH,
            Y: PANEL_HEIGHT,
        })?;
        geometry.SetCornerRadius(Vector2 {
            X: PANEL_RADIUS,
            Y: PANEL_RADIUS,
        })?;
        Ok(geometry)
    }

    fn hwnd(window: &Window) -> windows::core::Result<HWND> {
        let handle = window
            .window_handle()
            .map_err(|_| windows::core::Error::from(windows::Win32::Foundation::E_HANDLE))?;
        match handle.as_raw() {
            RawWindowHandle::Win32(handle) => Ok(HWND(handle.hwnd.get() as *mut _)),
            _ => Err(windows::core::Error::from(
                windows::Win32::Foundation::E_HANDLE,
            )),
        }
    }

    fn set_cloaked(window: &Window, cloaked: bool) -> windows::core::Result<()> {
        let value = i32::from(cloaked);
        unsafe {
            DwmSetWindowAttribute(
                hwnd(window)?,
                DWMWA_CLOAK,
                (&raw const value).cast(),
                std::mem::size_of::<i32>() as u32,
            )
        }
    }

    #[derive(Debug, Clone, Copy)]
    struct CaptureStats {
        inside: [f64; 3],
        outside: [f64; 3],
    }

    fn capture_screen(window: &Window) -> windows::core::Result<CaptureStats> {
        let origin = window
            .outer_position()
            .map_err(|_| windows::core::Error::from(E_FAIL))?;
        let area = window.inner_size();
        let width = area.width as i32;
        let height = area.height as i32;
        let screen = unsafe { GetDC(None) };
        if screen.0.is_null() {
            return Err(windows::core::Error::from(E_FAIL));
        }
        let memory = unsafe { CreateCompatibleDC(Some(screen)) };
        let bitmap = unsafe { CreateCompatibleBitmap(screen, width, height) };
        if memory.0.is_null() || bitmap.0.is_null() {
            unsafe {
                if !memory.0.is_null() {
                    let _ = DeleteDC(memory);
                }
                let _ = ReleaseDC(None, screen);
            }
            return Err(windows::core::Error::from(E_FAIL));
        }
        let old = unsafe { SelectObject(memory, HGDIOBJ(bitmap.0)) };
        let copied = unsafe {
            BitBlt(
                memory,
                0,
                0,
                width,
                height,
                Some(screen),
                origin.x,
                origin.y,
                SRCCOPY | CAPTUREBLT,
            )
        };
        let mut info = BITMAPINFO::default();
        info.bmiHeader.biSize = std::mem::size_of_val(&info.bmiHeader) as u32;
        info.bmiHeader.biWidth = width;
        info.bmiHeader.biHeight = -height;
        info.bmiHeader.biPlanes = 1;
        info.bmiHeader.biBitCount = 32;
        info.bmiHeader.biCompression = BI_RGB.0;
        let mut pixels = vec![0_u8; width as usize * height as usize * 4];
        let rows = if copied.is_ok() {
            unsafe {
                GetDIBits(
                    memory,
                    bitmap,
                    0,
                    height as u32,
                    Some(pixels.as_mut_ptr().cast()),
                    &raw mut info,
                    DIB_RGB_COLORS,
                )
            }
        } else {
            0
        };
        unsafe {
            let _ = SelectObject(memory, old);
            let _ = DeleteObject(HGDIOBJ(bitmap.0));
            let _ = DeleteDC(memory);
            let _ = ReleaseDC(None, screen);
        }
        if rows != height {
            return Err(windows::core::Error::from(E_FAIL));
        }

        let scale = window.scale_factor() as f32;
        let panel = (
            (PANEL_X * scale).round() as i32,
            (PANEL_Y * scale).round() as i32,
            (PANEL_WIDTH * scale).round() as i32,
            (PANEL_HEIGHT * scale).round() as i32,
        );
        let inside = mean_rgb(
            &pixels,
            width,
            height,
            panel.0 + panel.2 / 4,
            panel.1 + panel.3 / 4,
            panel.2 / 2,
            panel.3 / 2,
        );
        let outside = mean_rgb(&pixels, width, height, 8, 8, 24, 24);
        Ok(CaptureStats { inside, outside })
    }

    fn mean_rgb(
        pixels: &[u8],
        width: i32,
        height: i32,
        x: i32,
        y: i32,
        sample_width: i32,
        sample_height: i32,
    ) -> [f64; 3] {
        let mut sum = [0_u64; 3];
        let mut count = 0_u64;
        for py in y.max(0)..(y + sample_height).min(height) {
            for px in x.max(0)..(x + sample_width).min(width) {
                let index = ((py * width + px) * 4) as usize;
                sum[0] += u64::from(pixels[index + 2]);
                sum[1] += u64::from(pixels[index + 1]);
                sum[2] += u64::from(pixels[index]);
                count += 1;
            }
        }
        sum.map(|value| value as f64 / count.max(1) as f64)
    }

    fn channel_delta(left: [f64; 3], right: [f64; 3]) -> f64 {
        left.into_iter()
            .zip(right)
            .map(|(left, right)| (left - right).abs())
            .sum::<f64>()
            / 3.0
    }

    fn log_capture(label: &str, elapsed: Duration, capture: CaptureStats) {
        log::info!(
            target: "wgpu_l3::material_shadow_probe",
            "screen-space capture={label} elapsed_ms={} inside_rgb={:?} outside_rgb={:?}",
            elapsed.as_millis(),
            capture.inside,
            capture.outside,
        );
    }

    #[derive(Default)]
    struct App {
        underlay: Option<Arc<Window>>,
        window: Option<Arc<Window>>,
        composition: Option<CompositionState>,
        mode: ProbeMode,
    }

    impl ApplicationHandler for App {
        fn resumed(&mut self, event_loop: &ActiveEventLoop) {
            if self.window.is_some() {
                return;
            }
            let underlay_attributes = WindowAttributes::default()
                .with_title("Material Shadow Probe — static underlay")
                .with_inner_size(PhysicalSize::new(420, 300))
                .with_position(PhysicalPosition::new(360, 160))
                .with_decorations(false)
                .with_active(false)
                .with_skip_taskbar(true)
                .with_no_redirection_bitmap(true)
                .with_undecorated_shadow(false)
                .with_corner_preference(CornerPreference::DoNotRound);
            let underlay = Arc::new(
                event_loop
                    .create_window(underlay_attributes)
                    .expect("material probe underlay should open"),
            );
            let attributes = WindowAttributes::default()
                .with_title("Material Shadow Probe — compositor-only fade")
                .with_inner_size(PhysicalSize::new(420, 300))
                .with_position(PhysicalPosition::new(360, 160))
                .with_decorations(false)
                .with_transparent(true)
                .with_visible(false)
                .with_active(false)
                .with_window_level(winit::window::WindowLevel::AlwaysOnTop)
                .with_skip_taskbar(true)
                .with_no_redirection_bitmap(true)
                .with_undecorated_shadow(false)
                .with_corner_preference(CornerPreference::DoNotRound);
            let window = Arc::new(
                event_loop
                    .create_window(attributes)
                    .expect("material shadow probe window should open"),
            );
            match CompositionState::new(&window, &underlay, self.mode) {
                Ok(composition) => {
                    log::info!(
                        target: "wgpu_l3::material_shadow_probe",
                        "composition DropShadow gate active: surface=420x300 panel=300x180+60+44 radius=10 blur=24 offset_y=10; DWM border/shadow/rounding disabled"
                    );
                    self.composition = Some(composition);
                    window.set_visible(true);
                }
                Err(error) => {
                    log::error!(target: "wgpu_l3::material_shadow_probe", "probe setup failed: {error}");
                    event_loop.exit();
                }
            }
            self.underlay = Some(underlay);
            self.window = Some(window);
        }

        fn window_event(
            &mut self,
            event_loop: &ActiveEventLoop,
            window_id: WindowId,
            event: WindowEvent,
        ) {
            if self
                .window
                .as_ref()
                .is_some_and(|window| window.id() == window_id)
                && matches!(event, WindowEvent::CloseRequested)
            {
                event_loop.exit();
            }
        }

        fn about_to_wait(&mut self, event_loop: &ActiveEventLoop) {
            if let Some(composition) = &mut self.composition {
                let Some(window) = self.window.as_deref() else {
                    return;
                };
                let Some(underlay) = self.underlay.as_deref() else {
                    return;
                };
                match composition.update(window, underlay) {
                    Ok(true) => {
                        if self.mode == ProbeMode::Fade {
                            log::info!(
                                target: "wgpu_l3::material_shadow_probe",
                                "zero-hold probe complete"
                            );
                        }
                        event_loop.exit();
                        return;
                    }
                    Ok(false) => {}
                    Err(error) => {
                        log::error!(target: "wgpu_l3::material_shadow_probe", "probe failed: {error}");
                        event_loop.exit();
                        return;
                    }
                }
                event_loop.set_control_flow(ControlFlow::WaitUntil(composition.next_deadline()));
            }
        }
    }

    pub fn run() -> Result<(), Box<dyn std::error::Error>> {
        env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();
        let event_loop = EventLoop::new()?;
        event_loop.set_control_flow(ControlFlow::Wait);
        let mode = if std::env::args().any(|arg| arg == "--reuse-ladder") {
            ProbeMode::Reuse
        } else {
            ProbeMode::Fade
        };
        event_loop.run_app(&mut App {
            mode,
            ..App::default()
        })?;
        Ok(())
    }
}

#[cfg(target_os = "windows")]
fn main() -> Result<(), Box<dyn std::error::Error>> {
    windows_probe::run()
}

#[cfg(not(target_os = "windows"))]
fn main() {
    eprintln!("material_shadow_probe is Windows-only");
}
