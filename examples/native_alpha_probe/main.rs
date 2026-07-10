use std::sync::Arc;

use wgpu::util::DeviceExt;
use winit::{
    application::ApplicationHandler,
    dpi::{PhysicalPosition, PhysicalSize},
    event::{ElementState, WindowEvent},
    event_loop::{ActiveEventLoop, EventLoop},
    keyboard::{KeyCode, PhysicalKey},
    window::{Window, WindowAttributes, WindowId},
};

const SHADER: &str = r#"
struct VertexIn {
    @location(0) position: vec2<f32>,
};

struct VertexOut {
    @builtin(position) position: vec4<f32>,
};

@vertex
fn vs_main(in: VertexIn) -> VertexOut {
    var out: VertexOut;
    out.position = vec4<f32>(in.position, 0.0, 1.0);
    return out;
}

@fragment
fn fs_main() -> @location(0) vec4<f32> {
    return vec4<f32>(0.5, 0.0, 0.0, 0.5);
}
"#;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum BackendChoice {
    Dx12Visual,
    Vulkan,
}

impl BackendChoice {
    fn toggle(self) -> Self {
        match self {
            Self::Dx12Visual => Self::Vulkan,
            Self::Vulkan => Self::Dx12Visual,
        }
    }

    fn backends(self) -> wgpu::Backends {
        match self {
            Self::Dx12Visual => wgpu::Backends::DX12,
            Self::Vulkan => wgpu::Backends::VULKAN,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum AccentChoice {
    Off,
    Blur,
    Acrylic,
}

impl AccentChoice {
    fn toggle(self) -> Self {
        match self {
            Self::Off => Self::Blur,
            Self::Blur => Self::Acrylic,
            Self::Acrylic => Self::Off,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct ProbeConfig {
    backend: BackendChoice,
    accent: AccentChoice,
    no_redirection_bitmap: bool,
    owner: bool,
    popup_undecorated: bool,
    no_activate: bool,
    toolwindow: bool,
    always_on_top: bool,
    backdrop: bool,
    rounded: bool,
    shadow: bool,
}

impl Default for ProbeConfig {
    fn default() -> Self {
        Self {
            backend: BackendChoice::Dx12Visual,
            accent: AccentChoice::Off,
            no_redirection_bitmap: false,
            owner: false,
            popup_undecorated: false,
            no_activate: false,
            toolwindow: false,
            always_on_top: false,
            backdrop: false,
            rounded: false,
            shadow: false,
        }
    }
}

impl ProbeConfig {
    fn active_attributes(self) -> String {
        let mut attrs = Vec::new();
        if self.no_redirection_bitmap {
            attrs.push("nrb");
        }
        match self.accent {
            AccentChoice::Off => {}
            AccentChoice::Blur => attrs.push("accent-blur"),
            AccentChoice::Acrylic => attrs.push("accent-acrylic"),
        }
        if self.owner {
            attrs.push("owner");
        }
        if self.popup_undecorated {
            attrs.push("popup/undecorated");
        }
        if self.no_activate {
            attrs.push("noactivate");
        }
        if self.toolwindow {
            attrs.push("toolwindow");
        }
        if self.always_on_top {
            attrs.push("topmost");
        }
        if self.backdrop {
            attrs.push("backdrop");
        }
        if self.rounded {
            attrs.push("rounded");
        }
        if self.shadow {
            attrs.push("shadow");
        }
        if attrs.is_empty() {
            "boring".to_string()
        } else {
            attrs.join("+")
        }
    }
}

struct GpuState {
    surface: wgpu::Surface<'static>,
    device: wgpu::Device,
    queue: wgpu::Queue,
    config: wgpu::SurfaceConfiguration,
    pipeline: wgpu::RenderPipeline,
    vertex_buffer: wgpu::Buffer,
    max_texture_dimension_2d: u32,
}

impl GpuState {
    async fn new(window: Arc<Window>, config: ProbeConfig) -> Result<Self, String> {
        let mut backend_options = wgpu::BackendOptions::default();
        backend_options.dx12.presentation_system = wgpu::Dx12SwapchainKind::DxgiFromVisual;
        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
            backends: config.backend.backends(),
            flags: Default::default(),
            memory_budget_thresholds: Default::default(),
            backend_options,
            display: None,
        });
        let surface = instance.create_surface(window.clone()).map_err(to_string)?;
        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::default(),
                compatible_surface: Some(&surface),
                force_fallback_adapter: false,
            })
            .await
            .map_err(to_string)?;
        let info = adapter.get_info();
        let capabilities = surface.get_capabilities(&adapter);
        let adapter_limits = adapter.limits();
        let required_limits = wgpu::Limits::default().using_resolution(adapter_limits.clone());
        let surface_size = clamp_surface_size(
            window.inner_size(),
            required_limits.max_texture_dimension_2d,
        );
        log_surface_clamp(
            window.inner_size(),
            surface_size,
            required_limits.max_texture_dimension_2d,
        );
        let default_config = surface
            .get_default_config(&adapter, surface_size.width, surface_size.height)
            .ok_or_else(|| "surface has no default configuration".to_string())?;
        let mut surface_config = default_config;
        surface_config.alpha_mode = if capabilities
            .alpha_modes
            .contains(&wgpu::CompositeAlphaMode::PreMultiplied)
        {
            wgpu::CompositeAlphaMode::PreMultiplied
        } else {
            surface_config.alpha_mode
        };
        surface_config.present_mode = if capabilities
            .present_modes
            .contains(&wgpu::PresentMode::Fifo)
        {
            wgpu::PresentMode::Fifo
        } else {
            capabilities.present_modes[0]
        };
        surface_config.desired_maximum_frame_latency = 1;
        let (device, queue) = adapter
            .request_device(&wgpu::DeviceDescriptor {
                label: Some("native alpha probe"),
                required_features: wgpu::Features::empty(),
                experimental_features: Default::default(),
                required_limits,
                memory_hints: Default::default(),
                trace: wgpu::Trace::Off,
            })
            .await
            .map_err(to_string)?;
        surface.configure(&device, &surface_config);
        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("native_alpha_probe.wgsl"),
            source: wgpu::ShaderSource::Wgsl(SHADER.into()),
        });
        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Native Alpha Probe Pipeline Layout"),
            bind_group_layouts: &[],
            immediate_size: 0,
        });
        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Native Alpha Probe Pipeline"),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: Some("vs_main"),
                buffers: &[wgpu::VertexBufferLayout {
                    array_stride: std::mem::size_of::<[f32; 2]>() as u64,
                    step_mode: wgpu::VertexStepMode::Vertex,
                    attributes: &[wgpu::VertexAttribute {
                        format: wgpu::VertexFormat::Float32x2,
                        offset: 0,
                        shader_location: 0,
                    }],
                }],
                compilation_options: Default::default(),
            },
            primitive: wgpu::PrimitiveState::default(),
            depth_stencil: None,
            multisample: wgpu::MultisampleState::default(),
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: Some("fs_main"),
                targets: &[Some(wgpu::ColorTargetState {
                    format: surface_config.format,
                    blend: Some(wgpu::BlendState::REPLACE),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
                compilation_options: Default::default(),
            }),
            multiview_mask: None,
            cache: None,
        });
        let vertices: [[f32; 2]; 6] = [
            [-0.75, -0.5],
            [0.75, -0.5],
            [0.75, 0.5],
            [-0.75, -0.5],
            [0.75, 0.5],
            [-0.75, 0.5],
        ];
        let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Native Alpha Probe Vertices"),
            contents: bytemuck::cast_slice(&vertices),
            usage: wgpu::BufferUsages::VERTEX,
        });

        log::info!(
            target: "wgpu_l3::native_alpha_probe",
            "backend={:?} adapter_backend={:?} dx12_presentation=DxgiFromVisual alpha_supported={:?} alpha_selected={:?} attributes={}",
            config.backend,
            info.backend,
            capabilities.alpha_modes,
            surface_config.alpha_mode,
            config.active_attributes()
        );

        Ok(Self {
            surface,
            device,
            queue,
            config: surface_config,
            pipeline,
            vertex_buffer,
            max_texture_dimension_2d: adapter_limits.max_texture_dimension_2d,
        })
    }

    fn resize(&mut self, size: PhysicalSize<u32>) {
        if size.width == 0 || size.height == 0 {
            return;
        }
        let clamped = clamp_surface_size(size, self.max_texture_dimension_2d);
        log_surface_clamp(size, clamped, self.max_texture_dimension_2d);
        self.config.width = clamped.width;
        self.config.height = clamped.height;
        self.surface.configure(&self.device, &self.config);
    }

    fn render(&mut self) {
        use wgpu::CurrentSurfaceTexture::*;

        let frame = match self.surface.get_current_texture() {
            Success(frame) | Suboptimal(frame) => frame,
            Outdated | Timeout | Occluded | Validation => {
                log::warn!(target: "wgpu_l3::native_alpha_probe", "surface acquire skipped");
                self.surface.configure(&self.device, &self.config);
                return;
            }
            Lost => {
                log::warn!(target: "wgpu_l3::native_alpha_probe", "surface lost; reconfiguring");
                self.surface.configure(&self.device, &self.config);
                return;
            }
        };
        let view = frame
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());
        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("Native Alpha Probe Encoder"),
            });
        {
            let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Native Alpha Probe Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    depth_slice: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color::TRANSPARENT),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                occlusion_query_set: None,
                timestamp_writes: None,
                multiview_mask: None,
            });
            pass.set_pipeline(&self.pipeline);
            pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
            pass.draw(0..6, 0..1);
        }
        self.queue.submit(Some(encoder.finish()));
        frame.present();
    }
}

fn clamp_surface_size(size: PhysicalSize<u32>, max_texture_dimension_2d: u32) -> PhysicalSize<u32> {
    let max = max_texture_dimension_2d.max(1);
    PhysicalSize::new(size.width.max(1).min(max), size.height.max(1).min(max))
}

fn log_surface_clamp(
    requested: PhysicalSize<u32>,
    applied: PhysicalSize<u32>,
    max_texture_dimension_2d: u32,
) {
    if requested != applied {
        log::warn!(
            target: "wgpu_l3::native_alpha_probe",
            "clamped probe surface size from {}x{} to {}x{}; requested extent exceeds adapter max texture dimension {}",
            requested.width,
            requested.height,
            applied.width,
            applied.height,
            max_texture_dimension_2d
        );
    }
}

#[derive(Default)]
struct App {
    config: ProbeConfig,
    owner: Option<Arc<Window>>,
    window: Option<Arc<Window>>,
    gpu: Option<GpuState>,
}

impl App {
    fn rebuild(&mut self, event_loop: &ActiveEventLoop) {
        self.gpu = None;
        self.window = None;
        self.owner = None;

        let owner = if self.config.owner {
            let owner = Arc::new(
                event_loop
                    .create_window(
                        WindowAttributes::default()
                            .with_title("Native Alpha Probe Owner")
                            .with_inner_size(PhysicalSize::new(260, 120))
                            .with_position(PhysicalPosition::new(40, 40)),
                    )
                    .expect("owner window should be created"),
            );
            Some(owner)
        } else {
            None
        };

        let mut attributes = WindowAttributes::default()
            .with_title(self.title())
            .with_inner_size(PhysicalSize::new(420, 260))
            .with_position(PhysicalPosition::new(360, 160))
            .with_transparent(true)
            .with_decorations(!self.config.popup_undecorated)
            .with_active(!self.config.no_activate);
        if self.config.always_on_top {
            attributes = attributes.with_window_level(winit::window::WindowLevel::AlwaysOnTop);
        }
        attributes = configure_platform_attributes(attributes, owner.as_ref(), self.config);

        let window = Arc::new(
            event_loop
                .create_window(attributes)
                .expect("probe window should be created"),
        );
        configure_platform_window(&window, self.config);
        log_window_state(&window, self.config);

        let gpu = match pollster::block_on(GpuState::new(window.clone(), self.config)) {
            Ok(gpu) => gpu,
            Err(error) => {
                log::error!(target: "wgpu_l3::native_alpha_probe", "failed to initialize gpu: {error}");
                self.owner = owner;
                self.window = Some(window);
                return;
            }
        };

        self.owner = owner;
        self.window = Some(window);
        self.gpu = Some(gpu);
        self.request_redraw();
    }

    fn title(&self) -> String {
        format!(
            "Native Alpha Probe [{:?}; {}]",
            self.config.backend,
            self.config.active_attributes()
        )
    }

    fn request_redraw(&self) {
        if let Some(window) = &self.window {
            window.request_redraw();
        }
    }

    fn handle_key(&mut self, event_loop: &ActiveEventLoop, code: KeyCode) {
        match code {
            KeyCode::KeyV | KeyCode::KeyD => self.config.backend = self.config.backend.toggle(),
            KeyCode::KeyC => self.config.accent = self.config.accent.toggle(),
            KeyCode::KeyN => self.config.no_redirection_bitmap ^= true,
            KeyCode::KeyO => self.config.owner ^= true,
            KeyCode::KeyP => self.config.popup_undecorated ^= true,
            KeyCode::KeyA => self.config.no_activate ^= true,
            KeyCode::KeyT => self.config.toolwindow ^= true,
            KeyCode::KeyL => self.config.always_on_top ^= true,
            KeyCode::KeyB => self.config.backdrop ^= true,
            KeyCode::KeyR => self.config.rounded ^= true,
            KeyCode::KeyS => self.config.shadow ^= true,
            KeyCode::Digit0 => self.config = ProbeConfig::default(),
            KeyCode::Digit1 => {
                self.config.owner = true;
                self.config.toolwindow = true;
            }
            KeyCode::Digit2 => {
                self.config.no_redirection_bitmap = true;
                self.config.backdrop = true;
            }
            KeyCode::Escape | KeyCode::KeyQ => {
                event_loop.exit();
                return;
            }
            _ => return,
        }
        log::info!(
            target: "wgpu_l3::native_alpha_probe",
            "toggled {code:?}: {:?}",
            self.config
        );
        self.rebuild(event_loop);
    }
}

impl ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if self.window.is_none() {
            self.rebuild(event_loop);
        }
    }

    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        window_id: WindowId,
        event: WindowEvent,
    ) {
        if self
            .owner
            .as_ref()
            .is_some_and(|owner| owner.id() == window_id)
        {
            if matches!(event, WindowEvent::CloseRequested) {
                self.config.owner = false;
                self.rebuild(event_loop);
            }
            return;
        }

        let Some(window) = &self.window else {
            return;
        };
        if window.id() != window_id {
            return;
        }

        match event {
            WindowEvent::CloseRequested => event_loop.exit(),
            WindowEvent::Resized(size) => {
                if let Some(gpu) = &mut self.gpu {
                    gpu.resize(size);
                }
                self.request_redraw();
            }
            WindowEvent::RedrawRequested => {
                if let Some(gpu) = &mut self.gpu {
                    gpu.render();
                }
            }
            WindowEvent::KeyboardInput { event, .. }
                if event.state == ElementState::Pressed
                    && !event.repeat
                    && matches!(event.physical_key, PhysicalKey::Code(_)) =>
            {
                let PhysicalKey::Code(code) = event.physical_key else {
                    return;
                };
                self.handle_key(event_loop, code);
            }
            _ => {}
        }
    }

    fn about_to_wait(&mut self, _event_loop: &ActiveEventLoop) {
        self.request_redraw();
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();
    print_help();
    let event_loop = EventLoop::new()?;
    let mut app = App::default();
    event_loop.run_app(&mut app)?;
    Ok(())
}

fn print_help() {
    log::info!(
        target: "wgpu_l3::native_alpha_probe",
        "keys: V/D backend, C accent off/blur/acrylic, N no-redirection, O owner, P popup/undecorated, A noactivate, T toolwindow, L topmost, B backdrop, R rounded, S shadow, 0 reset, 1 owner+toolwindow, 2 nrb+backdrop, Q/Esc quit"
    );
}

fn to_string(error: impl std::fmt::Display) -> String {
    error.to_string()
}

#[cfg(target_os = "windows")]
fn configure_platform_attributes(
    mut attributes: WindowAttributes,
    owner: Option<&Arc<Window>>,
    config: ProbeConfig,
) -> WindowAttributes {
    use winit::platform::windows::{BackdropType, CornerPreference, WindowAttributesExtWindows};
    use winit::raw_window_handle::{HasWindowHandle, RawWindowHandle};

    attributes = attributes
        .with_no_redirection_bitmap(config.no_redirection_bitmap)
        .with_skip_taskbar(config.toolwindow)
        .with_undecorated_shadow(config.shadow);

    if config.backdrop {
        attributes = attributes.with_system_backdrop(BackdropType::TransientWindow);
    } else {
        attributes = attributes.with_system_backdrop(BackdropType::None);
    }
    if config.rounded {
        attributes = attributes.with_corner_preference(CornerPreference::Round);
    } else {
        attributes = attributes.with_corner_preference(CornerPreference::Default);
    }
    if let Some(owner) = owner {
        if let Ok(handle) = owner.window_handle() {
            if let RawWindowHandle::Win32(handle) = handle.as_raw() {
                attributes = attributes.with_owner_window(handle.hwnd.get());
            }
        }
    }

    attributes
}

#[cfg(not(target_os = "windows"))]
fn configure_platform_attributes(
    attributes: WindowAttributes,
    _owner: Option<&Arc<Window>>,
    _config: ProbeConfig,
) -> WindowAttributes {
    attributes
}

#[cfg(target_os = "windows")]
fn configure_platform_window(window: &Window, config: ProbeConfig) {
    use winit::raw_window_handle::{HasWindowHandle, RawWindowHandle};

    use windows_sys::Win32::Foundation::{HWND, LPARAM, LRESULT, WPARAM};
    use windows_sys::Win32::System::LibraryLoader::{GetModuleHandleA, GetProcAddress};
    use windows_sys::Win32::UI::Shell::SetWindowSubclass;
    use windows_sys::Win32::UI::WindowsAndMessaging::{
        GWL_EXSTYLE, GWL_STYLE, GetWindowLongPtrW, MA_NOACTIVATE, SetWindowLongPtrW,
        WM_MOUSEACTIVATE, WS_EX_NOACTIVATE, WS_EX_TOOLWINDOW, WS_POPUP,
    };
    use windows_sys::core::BOOL;

    const WCA_ACCENT_POLICY: u32 = 19;
    const ACCENT_DISABLED: i32 = 0;
    const ACCENT_ENABLE_BLURBEHIND: i32 = 3;
    const ACCENT_ENABLE_ACRYLICBLURBEHIND: i32 = 4;
    const ACCENT_ENABLE_GRADIENT_COLOR: i32 = 2;

    #[repr(C)]
    struct AccentPolicy {
        accent_state: i32,
        accent_flags: i32,
        gradient_color: u32,
        animation_id: i32,
    }

    #[repr(C)]
    struct WindowCompositionAttribData {
        attribute: u32,
        data: *mut core::ffi::c_void,
        size_of_data: usize,
    }

    type SetWindowCompositionAttributeFn =
        unsafe extern "system" fn(HWND, *mut WindowCompositionAttribData) -> BOOL;

    let Ok(handle) = window.window_handle() else {
        return;
    };
    let RawWindowHandle::Win32(handle) = handle.as_raw() else {
        return;
    };
    let hwnd = handle.hwnd.get() as HWND;

    unsafe {
        if config.popup_undecorated {
            let style = GetWindowLongPtrW(hwnd, GWL_STYLE);
            SetWindowLongPtrW(hwnd, GWL_STYLE, style | WS_POPUP as isize);
        }
        if config.no_activate || config.toolwindow {
            let mut exstyle = GetWindowLongPtrW(hwnd, GWL_EXSTYLE);
            if config.no_activate {
                exstyle |= WS_EX_NOACTIVATE as isize;
                let _ = SetWindowSubclass(hwnd, Some(mouse_activate_proc), 77, 0);
            }
            if config.toolwindow {
                exstyle |= WS_EX_TOOLWINDOW as isize;
            }
            SetWindowLongPtrW(hwnd, GWL_EXSTYLE, exstyle);
        }

        let accent_state = match config.accent {
            AccentChoice::Off => ACCENT_DISABLED,
            AccentChoice::Blur => ACCENT_ENABLE_BLURBEHIND,
            AccentChoice::Acrylic => ACCENT_ENABLE_ACRYLICBLURBEHIND,
        };
        let gradient_color = match config.accent {
            AccentChoice::Off => 0,
            AccentChoice::Blur | AccentChoice::Acrylic => 0xcc1e1c1c,
        };
        let mut policy = AccentPolicy {
            accent_state,
            accent_flags: ACCENT_ENABLE_GRADIENT_COLOR,
            gradient_color,
            animation_id: 0,
        };
        let mut data = WindowCompositionAttribData {
            attribute: WCA_ACCENT_POLICY,
            data: (&mut policy as *mut AccentPolicy).cast(),
            size_of_data: std::mem::size_of::<AccentPolicy>(),
        };
        if let Some(set_window_composition_attribute) = set_window_composition_attribute() {
            let result = set_window_composition_attribute(hwnd, &mut data);
            log::info!(
                target: "wgpu_l3::native_alpha_probe",
                "accent={:?} state={} gradient={gradient_color:#x} result={result}",
                config.accent,
                accent_state
            );
        } else {
            log::warn!(
                target: "wgpu_l3::native_alpha_probe",
                "SetWindowCompositionAttribute unavailable"
            );
        }
    }

    fn set_window_composition_attribute() -> Option<SetWindowCompositionAttributeFn> {
        let module = unsafe { GetModuleHandleA(c"user32.dll".as_ptr().cast()) };
        if module.is_null() {
            return None;
        }
        let proc =
            unsafe { GetProcAddress(module, c"SetWindowCompositionAttribute".as_ptr().cast()) }?;
        Some(unsafe {
            std::mem::transmute::<
                unsafe extern "system" fn() -> isize,
                SetWindowCompositionAttributeFn,
            >(proc)
        })
    }

    unsafe extern "system" fn mouse_activate_proc(
        hwnd: HWND,
        message: u32,
        wparam: WPARAM,
        lparam: LPARAM,
        _id: usize,
        _data: usize,
    ) -> LRESULT {
        if message == WM_MOUSEACTIVATE {
            return MA_NOACTIVATE as LRESULT;
        }
        unsafe { windows_sys::Win32::UI::Shell::DefSubclassProc(hwnd, message, wparam, lparam) }
    }
}

#[cfg(not(target_os = "windows"))]
fn configure_platform_window(_window: &Window, _config: ProbeConfig) {}

#[cfg(target_os = "windows")]
fn log_window_state(window: &Window, config: ProbeConfig) {
    use windows_sys::Win32::UI::WindowsAndMessaging::{GWL_EXSTYLE, GWL_STYLE, GetWindowLongPtrW};
    use winit::raw_window_handle::{HasWindowHandle, RawWindowHandle};

    let Ok(handle) = window.window_handle() else {
        return;
    };
    let RawWindowHandle::Win32(handle) = handle.as_raw() else {
        return;
    };
    let hwnd = handle.hwnd.get() as windows_sys::Win32::Foundation::HWND;
    unsafe {
        let style = GetWindowLongPtrW(hwnd, GWL_STYLE);
        let exstyle = GetWindowLongPtrW(hwnd, GWL_EXSTYLE);
        log::info!(
            target: "wgpu_l3::native_alpha_probe",
            "window_state attributes={} style={style:#x} exstyle={exstyle:#x}",
            config.active_attributes()
        );
    }
}

#[cfg(not(target_os = "windows"))]
fn log_window_state(_window: &Window, config: ProbeConfig) {
    log::info!(
        target: "wgpu_l3::native_alpha_probe",
        "window_state attributes={} platform_style_bits=unavailable",
        config.active_attributes()
    );
}
