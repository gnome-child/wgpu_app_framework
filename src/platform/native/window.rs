use super::{Native, NativeError, PopupPresentationMode};
use crate::geometry::area;
use crate::window as app_window;
use crate::{pointer, render};
use std::sync::Arc;
use winit::dpi::{LogicalPosition, LogicalSize};
use winit::{
    event::WindowEvent as WinitWindowEvent,
    event_loop::ActiveEventLoop,
    window::{CursorIcon, WindowAttributes},
};

use super::{CursorHost, PopupKey};

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
    pub popup_presentation_mode: Option<PopupPresentationMode>,
}

pub(in crate::platform::native) enum InitialSize {
    Logical(area::Logical),
}

impl Window {
    pub fn open(
        options: Options,
        event_loop: &ActiveEventLoop,
    ) -> Result<Handle, winit::error::OsError> {
        let mut window_attributes = WindowAttributes::default()
            .with_title(options.title)
            .with_visible(false);
        window_attributes = configure_base_attributes(
            window_attributes,
            options.kind,
            options.owner.as_deref(),
            options.popup_presentation_mode,
        );

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

    pub fn inner_area(&self) -> area::Physical {
        area::physical(
            self.handle.inner_size().width,
            self.handle.inner_size().height,
        )
    }

    pub fn scale_factor(&self) -> f64 {
        self.handle.scale_factor()
    }

    pub fn display_name(&self) -> Option<String> {
        self.handle
            .current_monitor()
            .and_then(|monitor| monitor.name())
    }

    pub fn display_refresh_millihertz(&self) -> Option<u32> {
        self.handle
            .current_monitor()
            .and_then(|monitor| monitor.refresh_rate_millihertz())
    }

    pub fn set_visibility(&self, visible: bool) {
        self.handle.set_visible(visible);
    }

    pub fn prepare_popup_first_present(&self) -> Result<(), i32> {
        super::sys::prepare_popup_first_present(&self.handle)
    }

    pub fn expose_popup_after_present(&self) -> Result<(), i32> {
        super::sys::expose_popup_after_present(&self.handle)
    }

    pub fn remove_popup_subclass(&self) {
        super::sys::remove_popup_subclass(&self.handle);
    }

    pub fn hide_popup_before_teardown(&self) {
        super::sys::hide_popup_before_teardown(&self.handle);
    }

    pub fn configure_popup_bounds(&self, x: i32, y: i32, area: area::Logical) {
        super::sys::configure_popup_bounds(&self.handle, x, y, area);
    }

    pub fn set_popup_hit_rect(&self, rect: crate::geometry::Rect) {
        super::sys::set_popup_hit_rect(&self.handle, rect);
    }

    pub fn set_popup_material_theme(&self, dark: bool) {
        let theme = if dark {
            winit::window::Theme::Dark
        } else {
            winit::window::Theme::Light
        };
        self.handle.set_theme(Some(theme));
        super::sys::set_popup_dark_mode(&self.handle, dark);
    }

    pub fn set_popup_accent_material(&self, material: super::sys::PopupAccentMaterial) -> bool {
        super::sys::set_popup_accent_material(&self.handle, material)
    }

    pub fn set_popup_border_color(&self, color: crate::scene::Color) {
        super::sys::set_popup_border_color(&self.handle, color);
    }

    pub fn handle(&self) -> Handle {
        Arc::clone(&self.handle)
    }

    pub fn set_ime_allowed(&self, allowed: bool) {
        self.handle.set_ime_allowed(allowed);
    }

    pub fn set_ime_cursor_area(&self, area: crate::geometry::Rect) {
        self.handle.set_ime_cursor_area(
            LogicalPosition::new(area.x(), area.y()),
            LogicalSize::new(area.width(), area.height()),
        );
    }

    pub fn set_cursor(&self, cursor: pointer::Cursor) {
        let icon = match cursor {
            pointer::Cursor::Default => CursorIcon::Default,
            pointer::Cursor::Text => CursorIcon::Text,
            pointer::Cursor::ResizeHorizontal => CursorIcon::EwResize,
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
        area: area::Physical,
        scale_factor: f32,
    ) {
        self.canvas.resize(render_context, area, scale_factor);
    }
}

fn configure_base_attributes(
    attributes: WindowAttributes,
    kind: app_window::Kind,
    owner: Option<&winit::window::Window>,
    popup_presentation_mode: Option<PopupPresentationMode>,
) -> WindowAttributes {
    match kind {
        app_window::Kind::Application => attributes,
        app_window::Kind::Popup => popup_attributes(
            attributes,
            owner,
            popup_presentation_mode.unwrap_or(PopupPresentationMode::RedirectedFallback),
        ),
    }
}

fn popup_attributes(
    attributes: WindowAttributes,
    owner: Option<&winit::window::Window>,
    mode: PopupPresentationMode,
) -> WindowAttributes {
    let attributes = attributes
        .with_decorations(false)
        .with_transparent(true)
        .with_blur(false)
        .with_window_level(winit::window::WindowLevel::AlwaysOnTop)
        .with_active(false);

    configure_platform_popup_attributes(attributes, owner, mode)
}

#[cfg(target_os = "windows")]
fn configure_platform_popup_attributes(
    attributes: WindowAttributes,
    owner: Option<&winit::window::Window>,
    mode: PopupPresentationMode,
) -> WindowAttributes {
    use winit::platform::windows::CornerPreference;
    use winit::platform::windows::WindowAttributesExtWindows;
    use winit::raw_window_handle::{HasWindowHandle, RawWindowHandle};

    let composition_backed = mode == PopupPresentationMode::CompositionBacked;
    let mut attributes = attributes
        .with_skip_taskbar(true)
        .with_no_redirection_bitmap(mode.no_redirection_bitmap())
        .with_undecorated_shadow(!composition_backed)
        .with_corner_preference(if composition_backed {
            CornerPreference::DoNotRound
        } else {
            CornerPreference::Round
        });

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
    _mode: PopupPresentationMode,
) -> WindowAttributes {
    use winit::platform::macos::WindowAttributesExtMacOS;

    attributes
        .with_accepts_first_mouse(true)
        .with_has_shadow(true)
}

#[cfg(all(unix, not(target_os = "macos")))]
fn configure_platform_popup_attributes(
    attributes: WindowAttributes,
    _owner: Option<&winit::window::Window>,
    _mode: PopupPresentationMode,
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
    _mode: PopupPresentationMode,
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
        let realization = popup.realization?;
        if !popup.exposed || !popup.accepts_input || realization.generation() != popup.generation {
            return None;
        }
        Some(super::PopupEventTarget {
            realization,
            scale_factor: popup.host.window.scale_factor(),
            first_present_elapsed_micros: popup.first_present.elapsed_micros(),
            first_present_stage: popup.first_present.stage(),
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
        &mut self,
        window: app_window::Id,
        cursor: pointer::Cursor,
    ) -> Result<(), NativeError> {
        if !self.windows.contains_key(&window) {
            log::warn!("cursor update requested for missing native window: {window:?}");
            return Err(NativeError::MissingWindow { window });
        }

        self.cursor_values.insert(window, cursor);
        self.apply_cursor_to_host(window, self.cursor_host(window), cursor);
        Ok(())
    }

    pub(in crate::platform) fn route_cursor_host_event(
        &mut self,
        raw: winit::window::WindowId,
        event: &WinitWindowEvent,
    ) {
        let target = self
            .raw_windows
            .get(&raw)
            .copied()
            .map(|parent| (parent, CursorHost::Parent))
            .or_else(|| {
                self.raw_popups
                    .get(&raw)
                    .copied()
                    .map(|key| (key.parent, CursorHost::Popup(key)))
            });
        let Some((parent, host)) = target else {
            return;
        };

        match event {
            WinitWindowEvent::CursorEntered { .. } | WinitWindowEvent::CursorMoved { .. } => {
                self.set_cursor_host(parent, host);
            }
            WinitWindowEvent::CursorLeft { .. } if self.cursor_host(parent) == host => {
                self.set_cursor_host(parent, CursorHost::Outside);
            }
            _ => {}
        }
    }

    pub(in crate::platform::native) fn set_cursor_host(
        &mut self,
        parent: app_window::Id,
        next: CursorHost,
    ) {
        let previous = self.cursor_host(parent);
        if previous == next {
            return;
        }

        self.apply_cursor_to_host(parent, previous, pointer::Cursor::Default);
        self.cursor_hosts.insert(parent, next);
        let cursor = self
            .cursor_values
            .get(&parent)
            .copied()
            .unwrap_or(pointer::Cursor::Default);
        self.apply_cursor_to_host(parent, next, cursor);
    }

    pub(in crate::platform::native) fn rehome_cursor_from_popup(&mut self, key: PopupKey) {
        if self.cursor_host(key.parent) == CursorHost::Popup(key) {
            self.set_cursor_host(key.parent, CursorHost::Parent);
        }
    }

    fn cursor_host(&self, parent: app_window::Id) -> CursorHost {
        self.cursor_hosts
            .get(&parent)
            .copied()
            .unwrap_or(CursorHost::Parent)
    }

    fn apply_cursor_to_host(
        &self,
        parent: app_window::Id,
        host: CursorHost,
        cursor: pointer::Cursor,
    ) {
        let window = match host {
            CursorHost::Parent => self.windows.get(&parent),
            CursorHost::Popup(key) => self.popups.get(&key).map(|popup| &popup.host.window),
            CursorHost::Outside => None,
        };
        if let Some(window) = window {
            window.set_cursor(cursor);
        }
    }

    #[cfg(test)]
    pub fn track_window_for_test(&mut self, raw: winit::window::WindowId, window: app_window::Id) {
        self.raw_windows.insert(raw, window);
    }
}
