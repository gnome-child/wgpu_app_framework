use super::PopupAccentMaterial;
use crate::geometry::area;
use winit::raw_window_handle::{HasWindowHandle, RawWindowHandle};

use std::collections::HashMap;
use std::sync::{Mutex, OnceLock};

use windows_sys::Win32::Foundation::{HWND, LPARAM, LRESULT, POINT, RECT, WPARAM};
use windows_sys::Win32::Graphics::Dwm::{
    DWMWA_BORDER_COLOR, DWMWA_CLOAK, DWMWA_USE_IMMERSIVE_DARK_MODE, DwmFlush, DwmSetWindowAttribute,
};
use windows_sys::Win32::Graphics::Gdi::{
    GetMonitorInfoW, MONITOR_DEFAULTTONEAREST, MONITORINFO, MonitorFromPoint,
};
use windows_sys::Win32::System::LibraryLoader::{GetModuleHandleA, GetProcAddress};
use windows_sys::Win32::UI::Shell::{DefSubclassProc, RemoveWindowSubclass, SetWindowSubclass};
use windows_sys::Win32::UI::WindowsAndMessaging::{
    GWL_EXSTYLE, GWL_STYLE, GetWindowLongPtrW, GetWindowRect, HTTRANSPARENT, HWND_TOPMOST,
    MA_NOACTIVATE, SW_HIDE, SW_SHOWNOACTIVATE, SWP_FRAMECHANGED, SWP_NOACTIVATE, SWP_NOMOVE,
    SWP_NOOWNERZORDER, SWP_NOSIZE, SWP_NOZORDER, SWP_SHOWWINDOW, SetWindowLongPtrW, SetWindowPos,
    ShowWindow, WM_MOUSEACTIVATE, WM_NCHITTEST, WS_BORDER, WS_CAPTION, WS_DLGFRAME,
    WS_EX_APPWINDOW, WS_EX_NOACTIVATE, WS_EX_TOOLWINDOW, WS_MAXIMIZEBOX, WS_MINIMIZEBOX, WS_POPUP,
    WS_SYSMENU, WS_THICKFRAME,
};
use windows_sys::core::BOOL;

const POPUP_SUBCLASS_ID: usize = 1;
const WCA_ACCENT_POLICY: u32 = 19;
const ACCENT_DISABLED: i32 = 0;
const ACCENT_ENABLE_ACRYLICBLURBEHIND: i32 = 4;
const ACCENT_ENABLE_GRADIENT_COLOR: i32 = 2;
const DWM_COLOR_NONE: u32 = 0xffff_fffe;

fn popup_hit_regions() -> &'static Mutex<HashMap<isize, crate::geometry::Rect>> {
    static REGIONS: OnceLock<Mutex<HashMap<isize, crate::geometry::Rect>>> = OnceLock::new();
    REGIONS.get_or_init(|| Mutex::new(HashMap::new()))
}

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

pub(super) fn enforce_popup_style(window: &winit::window::Window) {
    let Some(hwnd) = hwnd(window) else {
        log::warn!(target: "wgpu_l3::native_popup", "cannot enforce popup style without HWND");
        return;
    };

    unsafe {
        let current_style = GetWindowLongPtrW(hwnd, GWL_STYLE);
        let style_control_bits = WS_CAPTION
            | WS_SYSMENU
            | WS_THICKFRAME
            | WS_MINIMIZEBOX
            | WS_MAXIMIZEBOX
            | WS_BORDER
            | WS_DLGFRAME;
        let desired_style = (current_style | WS_POPUP as isize) & !(style_control_bits as isize);

        let current_exstyle = GetWindowLongPtrW(hwnd, GWL_EXSTYLE);
        let desired_exstyle =
            (current_exstyle | WS_EX_NOACTIVATE as isize | WS_EX_TOOLWINDOW as isize)
                & !(WS_EX_APPWINDOW as isize);

        let style_changed = desired_style != current_style;
        let exstyle_changed = desired_exstyle != current_exstyle;

        if style_changed {
            SetWindowLongPtrW(hwnd, GWL_STYLE, desired_style);
        }
        if exstyle_changed {
            SetWindowLongPtrW(hwnd, GWL_EXSTYLE, desired_exstyle);
        }
        if style_changed || exstyle_changed {
            SetWindowPos(
                hwnd,
                std::ptr::null_mut(),
                0,
                0,
                0,
                0,
                SWP_FRAMECHANGED | SWP_NOACTIVATE | SWP_NOMOVE | SWP_NOSIZE | SWP_NOZORDER,
            );
            log::debug!(
                target: "wgpu_l3::native_popup",
                "enforced popup style: style {current_style:#x} -> {desired_style:#x}, exstyle {current_exstyle:#x} -> {desired_exstyle:#x}"
            );
        } else {
            log::debug!(
                target: "wgpu_l3::native_popup",
                "popup style already enforced: style {current_style:#x}, exstyle {current_exstyle:#x}"
            );
        }
    }
}

pub(super) fn install_popup_subclass(window: &winit::window::Window) {
    let Some(hwnd) = hwnd(window) else {
        log::warn!(target: "wgpu_l3::native_popup", "cannot install popup subclass without HWND");
        return;
    };

    unsafe {
        if SetWindowSubclass(hwnd, Some(popup_subclass_proc), POPUP_SUBCLASS_ID, 0) == 0 {
            log::warn!(target: "wgpu_l3::native_popup", "failed to install popup subclass");
        } else {
            log::debug!(target: "wgpu_l3::native_popup", "installed popup subclass");
        }
    }
}

pub(super) fn set_popup_hit_rect(window: &winit::window::Window, rect: crate::geometry::Rect) {
    let Some(hwnd) = hwnd(window) else {
        return;
    };
    if let Ok(mut regions) = popup_hit_regions().lock() {
        regions.insert(hwnd as isize, rect);
    }
}

pub(super) fn remove_popup_subclass(window: &winit::window::Window) {
    let Some(hwnd) = hwnd(window) else {
        log::warn!(target: "wgpu_l3::native_popup", "cannot remove popup subclass without HWND");
        return;
    };

    unsafe {
        if RemoveWindowSubclass(hwnd, Some(popup_subclass_proc), POPUP_SUBCLASS_ID) == 0 {
            log::trace!(target: "wgpu_l3::native_popup", "popup subclass was not installed or already removed");
        } else {
            log::debug!(target: "wgpu_l3::native_popup", "removed popup subclass");
        }
    }
    if let Ok(mut regions) = popup_hit_regions().lock() {
        regions.remove(&(hwnd as isize));
    }
}

pub(super) fn configure_popup_bounds(
    window: &winit::window::Window,
    x: i32,
    y: i32,
    area: area::Logical,
) {
    let Some(hwnd) = hwnd(window) else {
        log::warn!(target: "wgpu_l3::native_popup", "cannot configure popup bounds without HWND");
        return;
    };
    let scale = window.scale_factor() as f32;
    let size = area.to_physical(scale).clamp_min(1);

    unsafe {
        SetWindowPos(
            hwnd,
            HWND_TOPMOST,
            x,
            y,
            size.width() as i32,
            size.height() as i32,
            SWP_NOACTIVATE | SWP_NOOWNERZORDER,
        );
    }
}

pub(super) fn popup_available_bounds(
    window: &winit::window::Window,
    anchor: crate::geometry::PlacementAnchor,
) -> Option<crate::geometry::Rect> {
    let parent_origin = window.inner_position().ok()?;
    let scale = window.scale_factor();
    let anchor = anchor.reference_point();
    let point = POINT {
        x: parent_origin
            .x
            .saturating_add((f64::from(anchor.x()) * scale).round() as i32),
        y: parent_origin
            .y
            .saturating_add((f64::from(anchor.y()) * scale).round() as i32),
    };
    let monitor = unsafe { MonitorFromPoint(point, MONITOR_DEFAULTTONEAREST) };
    if monitor.is_null() {
        return None;
    }

    let mut info: MONITORINFO = unsafe { core::mem::zeroed() };
    info.cbSize = core::mem::size_of::<MONITORINFO>() as u32;
    if unsafe { GetMonitorInfoW(monitor, &mut info) } == 0 {
        return None;
    }

    let work = info.rcWork;
    Some(super::physical_bounds_as_parent_logical(
        work.left,
        work.top,
        work.right.saturating_sub(work.left),
        work.bottom.saturating_sub(work.top),
        parent_origin.x,
        parent_origin.y,
        scale,
    ))
}

pub(super) fn set_popup_visible(window: &winit::window::Window, visible: bool) {
    let Some(hwnd) = hwnd(window) else {
        log::warn!(target: "wgpu_l3::native_popup", "cannot set popup visibility without HWND");
        return;
    };

    unsafe {
        if visible {
            ShowWindow(hwnd, SW_SHOWNOACTIVATE);
            SetWindowPos(
                hwnd,
                HWND_TOPMOST,
                0,
                0,
                0,
                0,
                SWP_NOMOVE | SWP_NOSIZE | SWP_NOACTIVATE | SWP_NOOWNERZORDER | SWP_SHOWWINDOW,
            );
        } else {
            ShowWindow(hwnd, SW_HIDE);
        }
    }
}

pub(super) fn hide_popup_before_teardown(window: &winit::window::Window) {
    let _ = set_popup_cloaked(window, true);
    set_popup_visible(window, false);
}

pub(super) fn prepare_popup_first_present(window: &winit::window::Window) -> Result<(), i32> {
    set_popup_cloaked(window, true)?;
    set_popup_visible(window, true);
    Ok(())
}

pub(super) fn expose_popup_after_present(window: &winit::window::Window) -> Result<(), i32> {
    set_popup_cloaked(window, false)
}

fn set_popup_cloaked(window: &winit::window::Window, cloaked: bool) -> Result<(), i32> {
    let Some(hwnd) = hwnd(window) else {
        log::warn!(
            target: "wgpu_l3::native_popup",
            "cannot set popup cloak without HWND"
        );
        return Err(-1);
    };
    let value: i32 = i32::from(cloaked);
    let result = unsafe {
        DwmSetWindowAttribute(
            hwnd,
            DWMWA_CLOAK as u32,
            &value as *const _ as *const core::ffi::c_void,
            std::mem::size_of::<i32>() as u32,
        )
    };
    if result < 0 {
        log::warn!(
            target: "wgpu_l3::native_popup",
            "failed to set popup cloak to {cloaked}: HRESULT {result:#x}"
        );
        Err(result)
    } else {
        log::debug!(
            target: "wgpu_l3::native_popup",
            "set popup cloak to {cloaked}"
        );
        Ok(())
    }
}

pub(super) fn synchronize_popup_presentation() -> Result<(), i32> {
    let result = unsafe { DwmFlush() };
    if result < 0 { Err(result) } else { Ok(()) }
}

pub(super) fn set_popup_dark_mode(window: &winit::window::Window, dark: bool) {
    let Some(hwnd) = hwnd(window) else {
        log::warn!(target: "wgpu_l3::native_popup", "cannot set popup dark mode without HWND");
        return;
    };

    let value: i32 = i32::from(dark);
    let result = unsafe {
        DwmSetWindowAttribute(
            hwnd,
            DWMWA_USE_IMMERSIVE_DARK_MODE as u32,
            &value as *const _ as *const core::ffi::c_void,
            std::mem::size_of::<i32>() as u32,
        )
    };

    if result < 0 {
        log::debug!(
            target: "wgpu_l3::native_popup",
            "failed to set popup immersive dark mode to {dark}: HRESULT {result:#x}"
        );
    } else {
        log::trace!(
            target: "wgpu_l3::native_popup",
            "set popup immersive dark mode to {dark}"
        );
    }
}

pub(super) fn set_popup_accent_material(
    window: &winit::window::Window,
    material: PopupAccentMaterial,
) -> bool {
    let Some(hwnd) = hwnd(window) else {
        log::warn!(target: "wgpu_l3::native_popup", "cannot set popup accent material without HWND");
        return false;
    };

    let (state, gradient_color) = match material {
        PopupAccentMaterial::Disabled => (ACCENT_DISABLED, 0),
        PopupAccentMaterial::Acrylic { tint } => (
            ACCENT_ENABLE_ACRYLICBLURBEHIND,
            super::accent_gradient_abgr(tint),
        ),
    };
    let mut policy = AccentPolicy {
        accent_state: state,
        accent_flags: ACCENT_ENABLE_GRADIENT_COLOR,
        gradient_color,
        animation_id: 0,
    };
    let mut data = WindowCompositionAttribData {
        attribute: WCA_ACCENT_POLICY,
        data: (&mut policy as *mut AccentPolicy).cast(),
        size_of_data: std::mem::size_of::<AccentPolicy>(),
    };
    let Some(set_window_composition_attribute) = set_window_composition_attribute() else {
        log::warn!(
            target: "wgpu_l3::native_popup",
            "SetWindowCompositionAttribute unavailable; cannot set popup accent material"
        );
        return false;
    };
    let result = unsafe { set_window_composition_attribute(hwnd, &mut data) };

    if result == 0 {
        log::warn!(
            target: "wgpu_l3::native_popup",
            "failed to set popup accent material {material:?}"
        );
        false
    } else {
        log::debug!(
            target: "wgpu_l3::native_popup",
            "set popup accent material {material:?} gradient={gradient_color:#x}"
        );
        true
    }
}

pub(super) fn set_popup_border_color(window: &winit::window::Window, colorref: u32) {
    let Some(hwnd) = hwnd(window) else {
        log::warn!(target: "wgpu_l3::native_popup", "cannot set popup border color without HWND");
        return;
    };
    let result = unsafe {
        DwmSetWindowAttribute(
            hwnd,
            DWMWA_BORDER_COLOR as u32,
            &colorref as *const _ as *const core::ffi::c_void,
            std::mem::size_of::<u32>() as u32,
        )
    };

    if result < 0 {
        log::debug!(
            target: "wgpu_l3::native_popup",
            "failed to set popup border COLORREF {colorref:#08x}: HRESULT {result:#x}"
        );
    } else {
        log::trace!(
            target: "wgpu_l3::native_popup",
            "set popup border COLORREF {colorref:#08x}"
        );
    }
}

pub(super) fn suppress_popup_border(window: &winit::window::Window) {
    set_popup_border_color(window, DWM_COLOR_NONE);
}

fn set_window_composition_attribute() -> Option<SetWindowCompositionAttributeFn> {
    let module = unsafe { GetModuleHandleA(c"user32.dll".as_ptr().cast()) };
    if module.is_null() {
        return None;
    }
    let proc = unsafe { GetProcAddress(module, c"SetWindowCompositionAttribute".as_ptr().cast()) }?;
    Some(unsafe {
        std::mem::transmute::<unsafe extern "system" fn() -> isize, SetWindowCompositionAttributeFn>(
            proc,
        )
    })
}

fn hwnd(window: &winit::window::Window) -> Option<HWND> {
    let handle = window.window_handle().ok()?;
    match handle.as_raw() {
        RawWindowHandle::Win32(handle) => Some(handle.hwnd.get() as HWND),
        _ => None,
    }
}

unsafe extern "system" fn popup_subclass_proc(
    hwnd: HWND,
    message: u32,
    wparam: WPARAM,
    lparam: LPARAM,
    _subclass_id: usize,
    _ref_data: usize,
) -> LRESULT {
    if message == WM_MOUSEACTIVATE {
        return MA_NOACTIVATE as LRESULT;
    }

    if message == WM_NCHITTEST {
        let mut window_rect = RECT::default();
        if unsafe { GetWindowRect(hwnd, &mut window_rect) } != 0
            && popup_hit_regions()
                .lock()
                .ok()
                .and_then(|regions| regions.get(&(hwnd as isize)).copied())
                .is_some_and(|rect| {
                    !rect_contains_point(
                        rect,
                        low_word_signed(lparam).saturating_sub(window_rect.left),
                        high_word_signed(lparam).saturating_sub(window_rect.top),
                    )
                })
        {
            return HTTRANSPARENT as LRESULT;
        }
    }

    unsafe { DefSubclassProc(hwnd, message, wparam, lparam) }
}

fn low_word_signed(value: LPARAM) -> i32 {
    (value as u16 as i16) as i32
}

fn high_word_signed(value: LPARAM) -> i32 {
    ((value as usize >> 16) as u16 as i16) as i32
}

fn rect_contains_point(rect: crate::geometry::Rect, x: i32, y: i32) -> bool {
    x >= rect.x() && x < rect.right() && y >= rect.y() && y < rect.bottom()
}

#[cfg(test)]
mod tests {
    use super::rect_contains_point;

    #[test]
    fn popup_shadow_margin_is_not_native_hit_geometry() {
        let panel = crate::geometry::Rect::new(12, 10, 100, 60);

        assert!(rect_contains_point(panel, 12, 10));
        assert!(rect_contains_point(panel, 111, 69));
        assert!(!rect_contains_point(panel, 11, 30));
        assert!(!rect_contains_point(panel, 40, 9));
        assert!(!rect_contains_point(panel, 112, 30));
        assert!(!rect_contains_point(panel, 40, 70));
    }
}
