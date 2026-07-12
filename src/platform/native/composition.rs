use crate::render;

use windows::System::DispatcherQueueController;
use windows::UI::Composition::Desktop::DesktopWindowTarget;
use windows::UI::Composition::{
    CompositionSurfaceBrush, Compositor, ContainerVisual, ICompositionSurface, SpriteVisual,
};
use windows::Win32::Foundation::{E_HANDLE, E_NOINTERFACE, HWND};
use windows::Win32::Graphics::DirectComposition::{
    DCompositionCreateDevice2, IDCompositionDevice, IDCompositionVisual,
};
use windows::Win32::System::WinRT::Composition::{ICompositorDesktopInterop, ICompositorInterop};
use windows::Win32::System::WinRT::{
    CreateDispatcherQueueController, DQTAT_COM_NONE, DQTYPE_THREAD_CURRENT, DispatcherQueueOptions,
};
use windows::core::{Interface, Result};
use windows_numerics::Vector2;
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
    _content: SpriteVisual,
    _content_brush: CompositionSurfaceBrush,
    _wrapped_surface: ICompositionSurface,
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

        let root = self.compositor.CreateContainerVisual()?;
        root.SetRelativeSizeAdjustment(Vector2 { X: 1.0, Y: 1.0 })?;
        root.Children()?.InsertAtTop(&content)?;

        let desktop: ICompositorDesktopInterop = self.compositor.cast()?;
        let target = unsafe { desktop.CreateDesktopWindowTarget(hwnd(window)?, false)? };
        target.SetRoot(&root)?;

        Ok(Host {
            _classic_visual: seed.visual,
            _target: target,
            root,
            _content: content,
            _content_brush: content_brush,
            _wrapped_surface: wrapped_surface,
        })
    }
}

impl SurfaceSeed {
    pub(super) fn target(&self) -> wgpu::SurfaceTargetUnsafe {
        wgpu::SurfaceTargetUnsafe::CompositionVisual(self.visual.as_raw())
    }
}

impl Host {
    pub(super) fn set_opacity(&self, opacity: f32) -> Result<()> {
        self.root.SetOpacity(opacity.clamp(0.0, 1.0))
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
