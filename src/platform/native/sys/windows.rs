use crate::paint;
use winit::raw_window_handle::{HasWindowHandle, RawWindowHandle};

use windows_sys::Win32::Foundation::HWND;
use windows_sys::Win32::UI::WindowsAndMessaging::{
    GWL_EXSTYLE, GetWindowLongPtrW, HWND_TOPMOST, SW_HIDE, SW_SHOWNOACTIVATE, SWP_NOACTIVATE,
    SWP_NOMOVE, SWP_NOOWNERZORDER, SWP_NOSIZE, SWP_SHOWWINDOW, SetWindowLongPtrW, SetWindowPos,
    ShowWindow, WS_EX_APPWINDOW, WS_EX_NOACTIVATE, WS_EX_TOOLWINDOW,
};

pub(super) fn enforce_popup_style(window: &winit::window::Window) {
    let Some(hwnd) = hwnd(window) else {
        log::warn!(target: "wgpu_l3::native_popup", "cannot enforce popup style without HWND");
        return;
    };

    unsafe {
        let current = GetWindowLongPtrW(hwnd, GWL_EXSTYLE);
        let desired = (current | WS_EX_NOACTIVATE as isize | WS_EX_TOOLWINDOW as isize)
            & !(WS_EX_APPWINDOW as isize);
        if desired != current {
            SetWindowLongPtrW(hwnd, GWL_EXSTYLE, desired);
            log::debug!(
                target: "wgpu_l3::native_popup",
                "enforced popup exstyle: {current:#x} -> {desired:#x}"
            );
        } else {
            log::debug!(
                target: "wgpu_l3::native_popup",
                "popup exstyle already enforced: {current:#x}"
            );
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

fn hwnd(window: &winit::window::Window) -> Option<HWND> {
    let handle = window.window_handle().ok()?;
    match handle.as_raw() {
        RawWindowHandle::Win32(handle) => Some(handle.hwnd.get() as HWND),
        _ => None,
    }
}
