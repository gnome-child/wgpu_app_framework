use super::{Native, NativeError};
use crate::window as app_window;
use crate::{paint, pointer, render};
use std::sync::Arc;
use winit::dpi::LogicalSize;
use winit::{
    event_loop::ActiveEventLoop,
    window::{CursorIcon, WindowAttributes},
};

pub(in crate::platform::native) type Handle = Arc<winit::window::Window>;

pub(in crate::platform::native) struct Window {
    handle: Handle,
    canvas: render::Canvas,
}

pub(in crate::platform::native) struct Options {
    pub title: String,
    pub inner_size: InitialSize,
    pub kind: app_window::Kind,
    pub owner: Option<Handle>,
}

pub(in crate::platform::native) enum InitialSize {
    Logical(paint::area::Logical),
}

impl Window {
    pub fn open(
        options: Options,
        event_loop: &ActiveEventLoop,
    ) -> Result<Handle, winit::error::OsError> {
        let mut window_attributes = WindowAttributes::default()
            .with_title(options.title)
            .with_visible(false);
        window_attributes =
            configure_base_attributes(window_attributes, options.kind, options.owner.as_deref());

        match options.inner_size {
            InitialSize::Logical(inner_area) => {
                window_attributes = window_attributes.with_inner_size(LogicalSize::new(
                    inner_area.width() as f64,
                    inner_area.height() as f64,
                ));
            }
        }

        let handle = Arc::new(event_loop.create_window(window_attributes)?);
        if options.kind == app_window::Kind::Popup {
            super::sys::enforce_popup_style(&handle);
            super::sys::install_popup_subclass(&handle);
        }

        Ok(handle)
    }

    pub fn new(handle: Handle, canvas: render::Canvas) -> Self {
        Self { handle, canvas }
    }

    pub fn raw_id(&self) -> winit::window::WindowId {
        self.handle.id()
    }

    pub fn request_redraw(&self) {
        self.handle.request_redraw();
    }

    pub fn inner_area(&self) -> paint::area::Physical {
        paint::area::physical(
            self.handle.inner_size().width,
            self.handle.inner_size().height,
        )
    }

    pub fn scale_factor(&self) -> f64 {
        self.handle.scale_factor()
    }

    pub fn set_visibility(&self, visible: bool) {
        self.handle.set_visible(visible);
    }

    pub fn set_popup_visibility(&self, visible: bool) {
        super::sys::set_popup_visible(&self.handle, visible);
    }

    pub fn remove_popup_subclass(&self) {
        super::sys::remove_popup_subclass(&self.handle);
    }

    pub fn configure_popup_bounds(&self, x: i32, y: i32, area: paint::area::Logical) {
        super::sys::configure_popup_bounds(&self.handle, x, y, area);
    }

    pub fn handle(&self) -> Handle {
        Arc::clone(&self.handle)
    }

    pub fn set_ime_allowed(&self, allowed: bool) {
        self.handle.set_ime_allowed(allowed);
    }

    pub fn set_cursor(&self, cursor: pointer::Cursor) {
        let icon = match cursor {
            pointer::Cursor::Default => CursorIcon::Default,
            pointer::Cursor::Text => CursorIcon::Text,
        };
        self.handle.set_cursor(icon);
    }

    pub fn canvas(&self) -> &render::Canvas {
        &self.canvas
    }

    pub fn canvas_mut(&mut self) -> &mut render::Canvas {
        &mut self.canvas
    }

    pub fn resize(
        &mut self,
        render_context: &render::Context,
        area: paint::area::Physical,
        scale_factor: f32,
    ) {
        self.canvas.resize(render_context, area, scale_factor);
    }
}

fn configure_base_attributes(
    attributes: WindowAttributes,
    kind: app_window::Kind,
    owner: Option<&winit::window::Window>,
) -> WindowAttributes {
    match kind {
        app_window::Kind::Application => attributes,
        app_window::Kind::Popup => popup_attributes(attributes, owner),
    }
}

fn popup_attributes(
    attributes: WindowAttributes,
    owner: Option<&winit::window::Window>,
) -> WindowAttributes {
    let attributes = attributes
        .with_decorations(false)
        .with_transparent(true)
        .with_blur(false)
        .with_window_level(winit::window::WindowLevel::AlwaysOnTop)
        .with_active(false);

    configure_platform_popup_attributes(attributes, owner)
}

#[cfg(target_os = "windows")]
fn configure_platform_popup_attributes(
    attributes: WindowAttributes,
    owner: Option<&winit::window::Window>,
) -> WindowAttributes {
    use winit::platform::windows::WindowAttributesExtWindows;
    use winit::raw_window_handle::{HasWindowHandle, RawWindowHandle};

    let mut attributes = attributes
        .with_skip_taskbar(true)
        .with_undecorated_shadow(false);

    if let Some(owner) = owner {
        if let Ok(handle) = owner.window_handle() {
            if let RawWindowHandle::Win32(handle) = handle.as_raw() {
                attributes = attributes.with_owner_window(handle.hwnd.get());
            }
        }
    }

    attributes
}

#[cfg(target_os = "macos")]
fn configure_platform_popup_attributes(
    attributes: WindowAttributes,
    _owner: Option<&winit::window::Window>,
) -> WindowAttributes {
    use winit::platform::macos::WindowAttributesExtMacOS;

    attributes
        .with_accepts_first_mouse(true)
        .with_has_shadow(false)
}

#[cfg(all(unix, not(target_os = "macos")))]
fn configure_platform_popup_attributes(
    attributes: WindowAttributes,
    _owner: Option<&winit::window::Window>,
) -> WindowAttributes {
    use winit::platform::x11::{WindowAttributesExtX11, WindowType};

    attributes
        .with_override_redirect(true)
        .with_x11_window_type(vec![WindowType::PopupMenu])
}

#[cfg(not(any(
    target_os = "windows",
    target_os = "macos",
    all(unix, not(target_os = "macos"))
)))]
fn configure_platform_popup_attributes(
    attributes: WindowAttributes,
    _owner: Option<&winit::window::Window>,
) -> WindowAttributes {
    attributes
}

impl Native {
    pub fn contains(&self, window: app_window::Id) -> bool {
        self.windows.contains_key(&window)
    }

    pub fn window_for_raw(&self, raw: winit::window::WindowId) -> Option<app_window::Id> {
        self.raw_windows.get(&raw).copied()
    }

    pub(in crate::platform) fn popup_for_raw(
        &self,
        raw: winit::window::WindowId,
    ) -> Option<super::PopupEventTarget> {
        let key = self.raw_popups.get(&raw).copied()?;
        let popup = self.popups.get(&key)?;
        Some(super::PopupEventTarget {
            parent: key.parent,
            id: key.id,
            bounds: popup.bounds,
            scale_factor: popup.window.scale_factor(),
        })
    }

    pub fn scale_factor(&self, window: app_window::Id) -> Option<f64> {
        self.windows.get(&window).map(Window::scale_factor)
    }

    pub fn is_empty(&self) -> bool {
        self.windows.is_empty()
    }

    pub fn request_redraw(&self, window: app_window::Id) -> Result<(), NativeError> {
        let Some(window) = self.windows.get(&window) else {
            log::warn!("redraw requested for missing native window: {window:?}");
            return Err(NativeError::MissingWindow { window });
        };

        window.request_redraw();
        Ok(())
    }

    pub fn set_cursor(
        &self,
        window: app_window::Id,
        cursor: pointer::Cursor,
    ) -> Result<(), NativeError> {
        let Some(window) = self.windows.get(&window) else {
            log::warn!("cursor update requested for missing native window: {window:?}");
            return Err(NativeError::MissingWindow { window });
        };

        window.set_cursor(cursor);
        Ok(())
    }

    #[cfg(test)]
    pub fn track_window_for_test(&mut self, raw: winit::window::WindowId, window: app_window::Id) {
        self.raw_windows.insert(raw, window);
    }
}
