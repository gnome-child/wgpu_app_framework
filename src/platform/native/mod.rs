use std::collections::HashMap;

use crate::{geometry, interaction, overlay, render};

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
    material: Option<overlay::PopupMaterial>,
    presentation_mode: PopupPresentationMode,
    material_realization: Option<PopupMaterialRealization>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(in crate::platform::native) enum PopupPresentationMode {
    CompositionBacked,
    RedirectedFallback,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum PopupMaterialRealization {
    WindowsAccentAcrylic,
    OpaqueFallback,
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
    fn new(window: window::Window, presentation_mode: PopupPresentationMode) -> Self {
        Self {
            window,
            bounds: geometry::Rect::new(0, 0, 0, 0),
            geometry: PopupGeometryState::default(),
            visible: false,
            material: None,
            presentation_mode,
            material_realization: None,
        }
    }
}

impl Drop for PopupWindow {
    fn drop(&mut self) {
        self.window.remove_popup_subclass();
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

impl PopupPresentationMode {
    fn from_render_context(context: &render::Context) -> Self {
        if context.windows_popup_composition_supported() {
            Self::CompositionBacked
        } else {
            Self::RedirectedFallback
        }
    }

    pub(in crate::platform::native) fn no_redirection_bitmap(self) -> bool {
        matches!(self, Self::CompositionBacked)
    }

    fn alpha_preference(self) -> render::CompositeAlphaPreference {
        match self {
            Self::CompositionBacked => render::CompositeAlphaPreference::PreMultiplied,
            Self::RedirectedFallback => render::CompositeAlphaPreference::Default,
        }
    }

    fn realization_for(self, alpha_mode: wgpu::CompositeAlphaMode) -> PopupMaterialRealization {
        match self {
            Self::CompositionBacked if alpha_mode == wgpu::CompositeAlphaMode::PreMultiplied => {
                PopupMaterialRealization::WindowsAccentAcrylic
            }
            Self::CompositionBacked | Self::RedirectedFallback => {
                PopupMaterialRealization::OpaqueFallback
            }
        }
    }
}

impl PopupMaterialRealization {
    fn uses_os_material(self) -> bool {
        matches!(self, Self::WindowsAccentAcrylic)
    }

    fn fallback_reason(
        self,
        mode: PopupPresentationMode,
        alpha_mode: wgpu::CompositeAlphaMode,
    ) -> Option<&'static str> {
        match (self, mode, alpha_mode) {
            (Self::WindowsAccentAcrylic, _, _) => None,
            (Self::OpaqueFallback, PopupPresentationMode::RedirectedFallback, _) => {
                Some("composition-backed popup presentation unavailable")
            }
            (Self::OpaqueFallback, PopupPresentationMode::CompositionBacked, _) => {
                Some("premultiplied alpha surface unavailable")
            }
        }
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
    use super::{
        PopupGeometry, PopupGeometryState, PopupMaterialRealization, PopupPresentationMode,
    };

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

    #[test]
    fn popup_presentation_mode_pairs_no_redirection_with_premultiplied_alpha() {
        assert!(PopupPresentationMode::CompositionBacked.no_redirection_bitmap());
        assert_eq!(
            PopupPresentationMode::CompositionBacked.alpha_preference(),
            crate::render::CompositeAlphaPreference::PreMultiplied
        );

        assert!(!PopupPresentationMode::RedirectedFallback.no_redirection_bitmap());
        assert_eq!(
            PopupPresentationMode::RedirectedFallback.alpha_preference(),
            crate::render::CompositeAlphaPreference::Default
        );
    }

    #[test]
    fn popup_material_realization_requires_composition_and_premultiplied_alpha() {
        assert_eq!(
            PopupPresentationMode::CompositionBacked
                .realization_for(wgpu::CompositeAlphaMode::PreMultiplied),
            PopupMaterialRealization::WindowsAccentAcrylic
        );
        assert_eq!(
            PopupPresentationMode::CompositionBacked
                .realization_for(wgpu::CompositeAlphaMode::Opaque),
            PopupMaterialRealization::OpaqueFallback
        );
        assert_eq!(
            PopupPresentationMode::RedirectedFallback
                .realization_for(wgpu::CompositeAlphaMode::PreMultiplied),
            PopupMaterialRealization::OpaqueFallback
        );
    }
}
