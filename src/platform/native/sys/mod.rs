#[cfg(target_os = "windows")]
mod windows;

use crate::paint;
use crate::scene;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(in crate::platform::native) enum PopupAccentMaterial {
    Disabled,
    Acrylic { tint: scene::Color },
}

pub(in crate::platform::native) fn enforce_popup_style(window: &winit::window::Window) {
    #[cfg(target_os = "windows")]
    windows::enforce_popup_style(window);

    #[cfg(not(target_os = "windows"))]
    {
        let _ = window;
    }
}

pub(in crate::platform::native) fn install_popup_subclass(window: &winit::window::Window) {
    #[cfg(target_os = "windows")]
    windows::install_popup_subclass(window);

    #[cfg(not(target_os = "windows"))]
    {
        let _ = window;
    }
}

pub(in crate::platform::native) fn remove_popup_subclass(window: &winit::window::Window) {
    #[cfg(target_os = "windows")]
    windows::remove_popup_subclass(window);

    #[cfg(not(target_os = "windows"))]
    {
        let _ = window;
    }
}

pub(in crate::platform::native) fn configure_popup_bounds(
    window: &winit::window::Window,
    x: i32,
    y: i32,
    area: paint::area::Logical,
) {
    #[cfg(target_os = "windows")]
    windows::configure_popup_bounds(window, x, y, area);

    #[cfg(not(target_os = "windows"))]
    {
        use winit::dpi::{LogicalSize, PhysicalPosition};

        window.set_outer_position(PhysicalPosition::new(x, y));
        let _ =
            window.request_inner_size(LogicalSize::new(area.width() as f64, area.height() as f64));
    }
}

pub(in crate::platform::native) fn set_popup_visible(
    window: &winit::window::Window,
    visible: bool,
) {
    #[cfg(target_os = "windows")]
    windows::set_popup_visible(window, visible);

    #[cfg(not(target_os = "windows"))]
    window.set_visible(visible);
}

pub(in crate::platform::native) fn set_popup_dark_mode(window: &winit::window::Window, dark: bool) {
    #[cfg(target_os = "windows")]
    windows::set_popup_dark_mode(window, dark);

    #[cfg(not(target_os = "windows"))]
    {
        let _ = window;
        let _ = dark;
    }
}

pub(in crate::platform::native) fn set_popup_accent_material(
    window: &winit::window::Window,
    material: PopupAccentMaterial,
) {
    #[cfg(target_os = "windows")]
    windows::set_popup_accent_material(window, material);

    #[cfg(not(target_os = "windows"))]
    {
        let _ = window;
        let _ = material;
    }
}

pub(in crate::platform::native) fn accent_gradient_abgr(color: scene::Color) -> u32 {
    let (r, g, b, a) = color.channels();
    ((a as u32) << 24) | ((b as u32) << 16) | ((g as u32) << 8) | r as u32
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn accent_gradient_color_uses_abgr_order() {
        assert_eq!(
            accent_gradient_abgr(scene::Color::rgba(0x11, 0x22, 0x33, 0x44)),
            0x44332211
        );
    }
}
