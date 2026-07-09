use crate::paint;
use winit::raw_window_handle::{HasWindowHandle, RawWindowHandle};

use windows_sys::Win32::Foundation::{HWND, LPARAM, LRESULT, WPARAM};
use windows_sys::Win32::Graphics::Dwm::{DWMWA_USE_IMMERSIVE_DARK_MODE, DwmSetWindowAttribute};
use windows_sys::Win32::UI::Shell::{DefSubclassProc, RemoveWindowSubclass, SetWindowSubclass};
use windows_sys::Win32::UI::WindowsAndMessaging::{
    GWL_EXSTYLE, GWL_STYLE, GetWindowLongPtrW, HWND_TOPMOST, MA_NOACTIVATE, SW_HIDE,
    SW_SHOWNOACTIVATE, SWP_FRAMECHANGED, SWP_NOACTIVATE, SWP_NOMOVE, SWP_NOOWNERZORDER, SWP_NOSIZE,
    SWP_NOZORDER, SWP_SHOWWINDOW, SetWindowLongPtrW, SetWindowPos, ShowWindow, WM_MOUSEACTIVATE,
    WS_BORDER, WS_CAPTION, WS_DLGFRAME, WS_EX_APPWINDOW, WS_EX_NOACTIVATE, WS_EX_TOOLWINDOW,
    WS_MAXIMIZEBOX, WS_MINIMIZEBOX, WS_POPUP, WS_SYSMENU, WS_THICKFRAME,
};

const POPUP_SUBCLASS_ID: usize = 1;

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
}

pub(super) fn configure_popup_bounds(
    window: &winit::window::Window,
    x: i32,
    y: i32,
    area: paint::area::Logical,
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

    unsafe { DefSubclassProc(hwnd, message, wparam, lparam) }
}
