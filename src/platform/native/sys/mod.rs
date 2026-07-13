#[cfg(target_os = "windows")]
mod windows;

use crate::{geometry, paint, scene};

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

pub(in crate::platform::native) fn set_popup_hit_rect(
    window: &winit::window::Window,
    rect: geometry::Rect,
) {
    #[cfg(target_os = "windows")]
    windows::set_popup_hit_rect(window, rect);

    #[cfg(not(target_os = "windows"))]
    {
        let _ = (window, rect);
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

pub(in crate::platform::native) fn popup_available_bounds(
    window: &winit::window::Window,
    anchor: geometry::PlacementAnchor,
) -> Option<geometry::Rect> {
    #[cfg(target_os = "windows")]
    return windows::popup_available_bounds(window, anchor)
        .or_else(|| monitor_available_bounds(window));

    #[cfg(not(target_os = "windows"))]
    return monitor_available_bounds(window);
}

fn monitor_available_bounds(window: &winit::window::Window) -> Option<geometry::Rect> {
    let monitor = window.current_monitor()?;
    let origin = window.inner_position().ok()?;
    let position = monitor.position();
    let size = monitor.size();
    Some(physical_bounds_as_parent_logical(
        position.x,
        position.y,
        size.width as i32,
        size.height as i32,
        origin.x,
        origin.y,
        window.scale_factor(),
    ))
}

fn physical_bounds_as_parent_logical(
    x: i32,
    y: i32,
    width: i32,
    height: i32,
    parent_x: i32,
    parent_y: i32,
    scale: f64,
) -> geometry::Rect {
    let logical = |value: i32| ((f64::from(value)) / scale).round() as i32;
    geometry::Rect::new(
        logical(x.saturating_sub(parent_x)),
        logical(y.saturating_sub(parent_y)),
        logical(width),
        logical(height),
    )
}

#[cfg(test)]
mod tests {
    use super::physical_bounds_as_parent_logical;
    use crate::geometry::Rect;

    #[test]
    fn monitor_work_area_projects_to_parent_logical_space_at_supported_scales() {
        for scale in [1.0, 1.25, 1.5, 2.0] {
            assert_eq!(
                physical_bounds_as_parent_logical(
                    100,
                    50,
                    (800.0 * scale) as i32,
                    (600.0 * scale) as i32,
                    100,
                    50,
                    scale,
                ),
                Rect::new(0, 0, 800, 600)
            );
        }
    }

    #[test]
    fn monitor_work_area_may_extend_beyond_the_parent_surface() {
        assert_eq!(
            physical_bounds_as_parent_logical(-1920, 0, 1920, 1040, -1200, 100, 1.0),
            Rect::new(-720, -100, 1920, 1040)
        );
    }
}

/// Makes a popup surface presentable without exposing uninitialized or stale
/// swapchain content to the user.
pub(in crate::platform::native) fn prepare_popup_first_present(
    window: &winit::window::Window,
) -> Result<(), i32> {
    #[cfg(target_os = "windows")]
    return windows::prepare_popup_first_present(window);

    #[cfg(not(target_os = "windows"))]
    {
        // Other native backends retain their established show-before-present
        // behavior until they grow a platform concealment primitive.
        window.set_visible(true);
        Ok(())
    }
}

/// Exposes a popup only after its current show-cycle frame was presented.
pub(in crate::platform::native) fn expose_popup_after_present(
    window: &winit::window::Window,
) -> Result<(), i32> {
    #[cfg(target_os = "windows")]
    return windows::expose_popup_after_present(window);

    #[cfg(not(target_os = "windows"))]
    {
        let _ = window;
        Ok(())
    }
}

pub(in crate::platform::native) fn hide_popup_before_teardown(window: &winit::window::Window) {
    #[cfg(target_os = "windows")]
    windows::hide_popup_before_teardown(window);

    #[cfg(not(target_os = "windows"))]
    window.set_visible(false);
}

pub(in crate::platform::native) fn synchronize_popup_presentation() -> Result<(), i32> {
    #[cfg(target_os = "windows")]
    return windows::synchronize_popup_presentation();

    #[cfg(not(target_os = "windows"))]
    Ok(())
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
) -> bool {
    #[cfg(target_os = "windows")]
    return windows::set_popup_accent_material(window, material);

    #[cfg(not(target_os = "windows"))]
    {
        let _ = window;
        let _ = material;
        false
    }
}

pub(in crate::platform::native) fn set_popup_border_color(
    window: &winit::window::Window,
    color: scene::Color,
) {
    #[cfg(target_os = "windows")]
    windows::set_popup_border_color(window, popup_border_colorref(color));

    #[cfg(not(target_os = "windows"))]
    {
        let _ = window;
        let _ = color;
    }
}

pub(in crate::platform::native) fn suppress_popup_border(window: &winit::window::Window) {
    #[cfg(target_os = "windows")]
    windows::suppress_popup_border(window);

    #[cfg(not(target_os = "windows"))]
    let _ = window;
}

pub(in crate::platform::native) fn accent_gradient_abgr(color: scene::Color) -> u32 {
    let (r, g, b, a) = color.channels();
    crate::color::aabbggrr(r, g, b, a)
}

pub(in crate::platform::native) fn popup_border_colorref(color: scene::Color) -> u32 {
    let (r, g, b, _) = color.channels();
    crate::color::bbggrr(r, g, b)
}
