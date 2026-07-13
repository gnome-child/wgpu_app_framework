#[cfg(target_os = "windows")]
mod windows_probe {
    use std::sync::Arc;
    use std::time::{Duration, Instant};

    use windows::Foundation::TimeSpan;
    use windows::System::DispatcherQueueController;
    use windows::UI::Composition::Desktop::DesktopWindowTarget;
    use windows::UI::Composition::{
        CompositionDropShadowSourcePolicy, CompositionGeometricClip,
        CompositionRoundedRectangleGeometry, CompositionSurfaceBrush, CompositionVisualSurface,
        Compositor, ContainerVisual, DropShadow, LayerVisual, SpriteVisual,
    };
    use windows::Win32::Foundation::HWND;
    use windows::Win32::Graphics::Dwm::{
        DWMWA_BORDER_COLOR, DWMWA_COLOR_NONE, DWMWA_USE_HOSTBACKDROPBRUSH, DwmSetWindowAttribute,
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
    const FADE_DURATION: Duration = Duration::from_millis(650);
    const HOLD_DURATION: Duration = Duration::from_millis(450);

    struct CompositionState {
        _dispatcher: DispatcherQueueController,
        _target: DesktopWindowTarget,
        _stage: ContainerVisual,
        _background: SpriteVisual,
        root: ContainerVisual,
        _frost: SpriteVisual,
        _frost_geometry: CompositionRoundedRectangleGeometry,
        _frost_clip: CompositionGeometricClip,
        _shadow_layer: LayerVisual,
        _caster: SpriteVisual,
        _caster_geometry: CompositionRoundedRectangleGeometry,
        _caster_clip: CompositionGeometricClip,
        _mask_visual: SpriteVisual,
        _mask_geometry: CompositionRoundedRectangleGeometry,
        _mask_clip: CompositionGeometricClip,
        _mask_surface: CompositionVisualSurface,
        _mask_brush: CompositionSurfaceBrush,
        _shadow: DropShadow,
        compositor: Compositor,
        phase_started: Instant,
        visible: bool,
        animation_count: u64,
    }

    impl CompositionState {
        fn new(window: &Window) -> windows::core::Result<Self> {
            let options = DispatcherQueueOptions {
                dwSize: std::mem::size_of::<DispatcherQueueOptions>() as u32,
                threadType: DQTYPE_THREAD_CURRENT,
                apartmentType: DQTAT_COM_NONE,
            };
            let dispatcher = unsafe { CreateDispatcherQueueController(options)? };
            let compositor = Compositor::new()?;
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

            let desktop: ICompositorDesktopInterop = compositor.cast()?;
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

            root.SetOpacity(0.0)?;
            let mut state = Self {
                _dispatcher: dispatcher,
                _target: target,
                _stage: stage,
                _background: background,
                root,
                _frost: frost,
                _frost_geometry: frost_geometry,
                _frost_clip: frost_clip,
                _shadow_layer: shadow_layer,
                _caster: caster,
                _caster_geometry: caster_geometry,
                _caster_clip: caster_clip,
                _mask_visual: mask_visual,
                _mask_geometry: mask_geometry,
                _mask_clip: mask_clip,
                _mask_surface: mask_surface,
                _mask_brush: mask_brush,
                _shadow: shadow,
                compositor,
                phase_started: Instant::now(),
                visible: false,
                animation_count: 0,
            };
            state.animate_to(true)?;
            Ok(state)
        }

        fn animate_to(&mut self, visible: bool) -> windows::core::Result<()> {
            let from = self.root.Opacity()?;
            let to = if visible { 1.0 } else { 0.0 };
            let animation = self.compositor.CreateScalarKeyFrameAnimation()?;
            animation.InsertKeyFrame(0.0, from)?;
            let easing = self.compositor.CreateCubicBezierEasingFunction(
                Vector2 { X: 0.33, Y: 1.0 },
                Vector2 { X: 0.68, Y: 1.0 },
            )?;
            animation.InsertKeyFrameWithEasingFunction(1.0, to, &easing)?;
            animation.SetDuration(TimeSpan {
                Duration: (FADE_DURATION.as_nanos() / 100) as i64,
            })?;
            self.root
                .StartAnimation(&HSTRING::from("Opacity"), &animation)?;
            self.visible = visible;
            self.phase_started = Instant::now();
            self.animation_count = self.animation_count.saturating_add(1);
            log::info!(
                target: "wgpu_l3::material_shadow_probe",
                "root animation={} from={from:.3} to={to:.3}; no application redraw scheduled",
                self.animation_count
            );
            Ok(())
        }

        fn update(&mut self) -> windows::core::Result<()> {
            let deadline = FADE_DURATION + HOLD_DURATION;
            if self.phase_started.elapsed() >= deadline {
                self.animate_to(!self.visible)?;
            }
            Ok(())
        }

        fn next_deadline(&self) -> Instant {
            self.phase_started + FADE_DURATION + HOLD_DURATION
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

    #[derive(Default)]
    struct App {
        window: Option<Arc<Window>>,
        composition: Option<CompositionState>,
    }

    impl ApplicationHandler for App {
        fn resumed(&mut self, event_loop: &ActiveEventLoop) {
            if self.window.is_some() {
                return;
            }
            let attributes = WindowAttributes::default()
                .with_title("Material Shadow Probe — compositor-only fade")
                .with_inner_size(PhysicalSize::new(420, 300))
                .with_position(PhysicalPosition::new(360, 160))
                .with_decorations(false)
                .with_transparent(true)
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
            match CompositionState::new(&window) {
                Ok(composition) => {
                    log::info!(
                        target: "wgpu_l3::material_shadow_probe",
                        "composition DropShadow gate active: surface=420x300 panel=300x180+60+44 radius=10 blur=24 offset_y=10; DWM border/shadow/rounding disabled"
                    );
                    self.composition = Some(composition);
                }
                Err(error) => {
                    log::error!(target: "wgpu_l3::material_shadow_probe", "probe setup failed: {error}");
                    event_loop.exit();
                }
            }
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
                if let Err(error) = composition.update() {
                    log::error!(target: "wgpu_l3::material_shadow_probe", "animation failed: {error}");
                    event_loop.exit();
                    return;
                }
                event_loop.set_control_flow(ControlFlow::WaitUntil(composition.next_deadline()));
            }
        }
    }

    pub fn run() -> Result<(), Box<dyn std::error::Error>> {
        env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();
        let event_loop = EventLoop::new()?;
        event_loop.set_control_flow(ControlFlow::Wait);
        event_loop.run_app(&mut App::default())?;
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
