use std::collections::HashMap;

use crate::{geometry, interaction, render};

use super::super::{session, window as app_window};

mod adapter;
mod color;
mod context;
mod error;
mod paint;
mod poll;
mod popup;
mod request;
mod surface;
mod sys;
mod window;

pub use context::NativeContext;
pub use error::NativeError;

pub struct Native {
    context: Option<render::Context>,
    renderer: Option<render::Renderer>,
    windows: HashMap<app_window::Id, window::Window>,
    popups: HashMap<PopupKey, PopupWindow>,
    raw_windows: HashMap<winit::window::WindowId, app_window::Id>,
    raw_popups: HashMap<winit::window::WindowId, PopupKey>,
    requests: Vec<session::Request>,
    poll_requested: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
struct PopupKey {
    parent: app_window::Id,
    id: interaction::Id,
}

struct PopupWindow {
    window: window::Window,
    bounds: geometry::Rect,
    geometry: PopupGeometryState,
    visible: bool,
}

#[derive(Debug, Clone, Copy)]
pub(in crate::platform) struct PopupEventTarget {
    parent: app_window::Id,
    id: interaction::Id,
    bounds: geometry::Rect,
    scale_factor: f64,
}

#[derive(Debug, Default)]
struct PopupGeometryState {
    applied: Option<PopupGeometry>,
}

#[derive(Debug, Clone, Copy, PartialEq)]
struct PopupGeometry {
    x: i32,
    y: i32,
    width: f32,
    height: f32,
    scale_factor_bits: u64,
}

impl PopupKey {
    fn new(parent: app_window::Id, id: interaction::Id) -> Self {
        Self { parent, id }
    }
}

impl PopupWindow {
    fn new(window: window::Window) -> Self {
        Self {
            window,
            bounds: geometry::Rect::new(0, 0, 0, 0),
            geometry: PopupGeometryState::default(),
            visible: false,
        }
    }
}

impl PopupEventTarget {
    pub(in crate::platform) fn parent(self) -> app_window::Id {
        self.parent
    }

    pub(in crate::platform) fn id(self) -> interaction::Id {
        self.id
    }

    pub(in crate::platform) fn bounds(self) -> geometry::Rect {
        self.bounds
    }

    pub(in crate::platform) fn scale_factor(self) -> f64 {
        self.scale_factor
    }
}

impl PopupGeometryState {
    fn needs_apply(&self, desired: PopupGeometry) -> bool {
        self.applied != Some(desired)
    }

    fn mark_applied(&mut self, geometry: PopupGeometry) {
        self.applied = Some(geometry);
    }
}

impl PopupGeometry {
    fn logical_area(self) -> crate::paint::area::Logical {
        crate::paint::area::logical(self.width, self.height)
    }
}

impl Native {
    pub fn new() -> Self {
        Self {
            context: None,
            renderer: None,
            windows: HashMap::new(),
            popups: HashMap::new(),
            raw_windows: HashMap::new(),
            raw_popups: HashMap::new(),
            requests: Vec::new(),
            poll_requested: false,
        }
    }

    pub fn initialize(&mut self) -> Result<(), NativeError> {
        self.ensure_context()
    }

    pub fn ready(&self) -> bool {
        self.context.is_some()
    }
}

impl Default for Native {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::{PopupGeometry, PopupGeometryState};

    fn geometry(x: i32, y: i32, scale: f64) -> PopupGeometry {
        PopupGeometry {
            x,
            y,
            width: 240.0,
            height: 180.0,
            scale_factor_bits: scale.to_bits(),
        }
    }

    #[test]
    fn popup_geometry_state_skips_unchanged_redraws() {
        let mut state = PopupGeometryState::default();
        let desired = geometry(10, 20, 1.0);

        assert!(state.needs_apply(desired));
        state.mark_applied(desired);

        assert!(
            !state.needs_apply(desired),
            "fade/redraw frames with unchanged geometry must be draw-only"
        );
    }

    #[test]
    fn popup_geometry_state_reapplies_real_geometry_changes() {
        let mut state = PopupGeometryState::default();
        let desired = geometry(10, 20, 1.0);
        state.mark_applied(desired);

        assert!(
            state.needs_apply(geometry(11, 20, 1.0)),
            "parent move or anchor change must reconfigure popup position"
        );
        assert!(
            state.needs_apply(geometry(10, 20, 1.5)),
            "popup monitor scale changes must reconfigure popup size"
        );
    }
}
