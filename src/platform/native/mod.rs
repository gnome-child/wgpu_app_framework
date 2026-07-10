use std::collections::HashMap;
use std::time::{Duration, Instant};

use crate::{geometry, interaction, overlay, render, scene};

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
    renderers: HashMap<wgpu::TextureFormat, render::Renderer>,
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
    accent: PopupAccentState,
    border: PopupBorderState,
    visible: bool,
    first_present: PopupFirstPresentTrace,
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
    TransparentNoAccent,
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

#[derive(Debug, Default)]
struct PopupAccentState {
    desired: Option<sys::PopupAccentMaterial>,
    applied: Option<sys::PopupAccentMaterial>,
    desired_changed_at: Option<Instant>,
}

#[derive(Debug, Default)]
struct PopupBorderState {
    desired: Option<scene::Color>,
    applied: Option<scene::Color>,
    desired_changed_at: Option<Instant>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum PopupAccentDue {
    Immediate,
    Settled,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum PopupBorderDue {
    Immediate,
    Settled,
}

#[derive(Debug)]
struct PopupFirstPresentTrace {
    created_at: Instant,
    configured: bool,
    acquire_attempts: u32,
    state: PopupFirstPresentState,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum PopupFirstPresentState {
    AwaitingFirst,
    AwaitingConfirmation,
    Complete,
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
            accent: PopupAccentState::default(),
            border: PopupBorderState::default(),
            visible: false,
            first_present: PopupFirstPresentTrace::new(),
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

// Windows accent re-application rebuilds compositor-side material state. Keep
// parameter churn settle-rate while preserving instant material presence.
const POPUP_ACCENT_SETTLE_DELAY: Duration = Duration::from_millis(150);
const POPUP_BORDER_SETTLE_DELAY: Duration = Duration::from_millis(150);

impl PopupGeometryState {
    fn needs_apply(&self, desired: PopupGeometry) -> bool {
        self.applied != Some(desired)
    }

    fn mark_applied(&mut self, geometry: PopupGeometry) {
        self.applied = Some(geometry);
    }
}

impl PopupAccentState {
    fn set_desired(&mut self, desired: sys::PopupAccentMaterial, now: Instant) -> bool {
        if self.desired == Some(desired) {
            return false;
        }

        self.desired = Some(desired);
        self.desired_changed_at = (self.applied != Some(desired)).then_some(now);
        true
    }

    fn due(&self, now: Instant) -> Option<PopupAccentDue> {
        let desired = self.desired?;
        if self.applied == Some(desired) {
            return None;
        }

        let Some(applied) = self.applied else {
            return Some(PopupAccentDue::Immediate);
        };

        if accent_presence(applied) != accent_presence(desired) {
            return Some(PopupAccentDue::Immediate);
        }

        let changed_at = self.desired_changed_at.unwrap_or(now);
        (now.duration_since(changed_at) >= POPUP_ACCENT_SETTLE_DELAY)
            .then_some(PopupAccentDue::Settled)
    }

    fn desired(&self) -> Option<sys::PopupAccentMaterial> {
        self.desired
    }

    fn mark_applied(&mut self, material: sys::PopupAccentMaterial) {
        self.applied = Some(material);
        self.desired = Some(material);
        self.desired_changed_at = None;
    }

    fn pending(&self) -> bool {
        self.desired.is_some() && self.desired != self.applied
    }

    fn changed_instant(&self) -> Option<Instant> {
        self.desired_changed_at
    }
}

impl PopupBorderState {
    fn set_desired(&mut self, desired: scene::Color, now: Instant) -> bool {
        if self.desired == Some(desired) {
            return false;
        }

        self.desired = Some(desired);
        self.desired_changed_at = (self.applied != Some(desired)).then_some(now);
        true
    }

    fn due(&self, now: Instant) -> Option<PopupBorderDue> {
        let desired = self.desired?;
        if self.applied == Some(desired) {
            return None;
        }

        if self.applied.is_none() {
            return Some(PopupBorderDue::Immediate);
        }

        let changed_at = self.desired_changed_at.unwrap_or(now);
        (now.duration_since(changed_at) >= POPUP_BORDER_SETTLE_DELAY)
            .then_some(PopupBorderDue::Settled)
    }

    fn desired(&self) -> Option<scene::Color> {
        self.desired
    }

    fn mark_applied(&mut self, border: scene::Color) {
        self.applied = Some(border);
        self.desired = Some(border);
        self.desired_changed_at = None;
    }

    fn pending(&self) -> bool {
        self.desired.is_some() && self.desired != self.applied
    }

    fn changed_instant(&self) -> Option<Instant> {
        self.desired_changed_at
    }
}

impl PopupFirstPresentTrace {
    fn new() -> Self {
        Self {
            created_at: Instant::now(),
            configured: false,
            acquire_attempts: 0,
            state: PopupFirstPresentState::AwaitingFirst,
        }
    }

    fn elapsed_micros(&self) -> u128 {
        self.created_at.elapsed().as_micros()
    }
}

fn accent_presence(material: sys::PopupAccentMaterial) -> PopupAccentPresence {
    match material {
        sys::PopupAccentMaterial::Disabled => PopupAccentPresence::Disabled,
        sys::PopupAccentMaterial::Acrylic { .. } => PopupAccentPresence::Acrylic,
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum PopupAccentPresence {
    Disabled,
    Acrylic,
}

impl PopupGeometry {
    fn logical_area(self) -> crate::paint::area::Logical {
        crate::paint::area::logical(self.width, self.height)
    }

    #[cfg(test)]
    fn rounded_physical_area(self) -> crate::paint::area::Physical {
        let scale = f64::from_bits(self.scale_factor_bits);
        crate::paint::area::physical(
            ((self.width as f64) * scale).round() as u32,
            ((self.height as f64) * scale).round() as u32,
        )
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
            Self::RedirectedFallback => render::CompositeAlphaPreference::PreMultiplied,
        }
    }

    fn realization_for(
        self,
        format: wgpu::TextureFormat,
        alpha_mode: wgpu::CompositeAlphaMode,
        preference: overlay::PopupMaterialPreference,
    ) -> PopupMaterialRealization {
        match preference {
            overlay::PopupMaterialPreference::OpaqueFallback => {
                PopupMaterialRealization::OpaqueFallback
            }
            overlay::PopupMaterialPreference::NoAccent => {
                if render::supports_windows_premultiplied_popup_pack(format, alpha_mode) {
                    PopupMaterialRealization::TransparentNoAccent
                } else {
                    PopupMaterialRealization::OpaqueFallback
                }
            }
            overlay::PopupMaterialPreference::System => {
                if render::supports_windows_premultiplied_popup_pack(format, alpha_mode) {
                    PopupMaterialRealization::WindowsAccentAcrylic
                } else {
                    PopupMaterialRealization::OpaqueFallback
                }
            }
        }
    }
}

impl PopupMaterialRealization {
    fn uses_os_material(self) -> bool {
        matches!(self, Self::WindowsAccentAcrylic)
    }

    fn uses_native_material_scene(self) -> bool {
        matches!(self, Self::WindowsAccentAcrylic | Self::TransparentNoAccent)
    }

    fn fallback_reason(
        self,
        mode: PopupPresentationMode,
        format: wgpu::TextureFormat,
        alpha_mode: wgpu::CompositeAlphaMode,
    ) -> Option<&'static str> {
        match (self, mode, format, alpha_mode) {
            (Self::WindowsAccentAcrylic, _, _, _) => None,
            (Self::TransparentNoAccent, _, _, _) => None,
            (Self::OpaqueFallback, _, _, wgpu::CompositeAlphaMode::PreMultiplied)
                if !matches!(
                    format,
                    wgpu::TextureFormat::Bgra8Unorm | wgpu::TextureFormat::Rgba8Unorm
                ) =>
            {
                Some("non-sRGB premultiplied popup surface format unavailable")
            }
            (Self::OpaqueFallback, _, _, wgpu::CompositeAlphaMode::PreMultiplied) => None,
            (Self::OpaqueFallback, _, _, _) => Some("premultiplied alpha surface unavailable"),
        }
    }
}

impl Native {
    pub fn new() -> Self {
        Self {
            context: None,
            renderers: HashMap::new(),
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
        POPUP_ACCENT_SETTLE_DELAY, POPUP_BORDER_SETTLE_DELAY, PopupAccentDue, PopupAccentState,
        PopupBorderDue, PopupBorderState, PopupGeometry, PopupGeometryState,
        PopupMaterialRealization, PopupPresentationMode,
    };
    use crate::overlay::PopupMaterialPreference;
    use crate::platform::native::sys::PopupAccentMaterial;
    use crate::scene;
    use std::time::{Duration, Instant};

    fn geometry(x: i32, y: i32, scale: f64) -> PopupGeometry {
        PopupGeometry {
            x,
            y,
            width: 240.0,
            height: 180.0,
            scale_factor_bits: scale.to_bits(),
        }
    }

    fn acrylic(red: u8) -> PopupAccentMaterial {
        PopupAccentMaterial::Acrylic {
            tint: scene::Color::rgba(red, 20, 30, 180),
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
    fn popup_geometry_scale_chain_rounds_logical_size_to_physical_size() {
        let geometry = geometry(10, 20, 1.5);

        assert_eq!(
            geometry.logical_area(),
            crate::paint::area::logical(240.0, 180.0)
        );
        assert_eq!(
            geometry.rounded_physical_area(),
            crate::paint::area::physical(360, 270)
        );
    }

    #[test]
    fn popup_accent_state_applies_first_material_immediately() {
        let now = Instant::now();
        let mut state = PopupAccentState::default();
        let desired = acrylic(10);

        assert!(state.set_desired(desired, now));
        assert_eq!(state.due(now), Some(PopupAccentDue::Immediate));
        state.mark_applied(desired);
        assert_eq!(state.due(now), None);
        assert!(!state.pending());
    }

    #[test]
    fn popup_accent_state_applies_presence_changes_immediately() {
        let now = Instant::now();
        let mut state = PopupAccentState::default();
        state.set_desired(PopupAccentMaterial::Disabled, now);
        state.mark_applied(PopupAccentMaterial::Disabled);

        let desired = acrylic(10);
        assert!(state.set_desired(desired, now + Duration::from_millis(1)));
        assert_eq!(
            state.due(now + Duration::from_millis(1)),
            Some(PopupAccentDue::Immediate)
        );
    }

    #[test]
    fn popup_accent_state_debounces_tint_only_changes_to_latest() {
        let now = Instant::now();
        let mut state = PopupAccentState::default();
        let first = acrylic(10);
        state.set_desired(first, now);
        state.mark_applied(first);

        let second = acrylic(11);
        let third = acrylic(12);
        assert!(state.set_desired(second, now + Duration::from_millis(1)));
        assert_eq!(state.due(now + Duration::from_millis(1)), None);
        assert!(state.pending());

        assert!(state.set_desired(third, now + Duration::from_millis(20)));
        assert_eq!(state.due(now + Duration::from_millis(149)), None);
        assert_eq!(
            state.due(now + Duration::from_millis(20) + POPUP_ACCENT_SETTLE_DELAY),
            Some(PopupAccentDue::Settled)
        );
        assert_eq!(state.desired(), Some(third));
    }

    #[test]
    fn popup_accent_state_repeated_desired_does_not_extend_quiet_time() {
        let now = Instant::now();
        let mut state = PopupAccentState::default();
        let first = acrylic(10);
        let second = acrylic(11);
        state.set_desired(first, now);
        state.mark_applied(first);
        state.set_desired(second, now);
        let changed_at = state.changed_instant();

        assert!(!state.set_desired(second, now + Duration::from_millis(120)));
        assert_eq!(state.changed_instant(), changed_at);
        assert_eq!(
            state.due(now + POPUP_ACCENT_SETTLE_DELAY),
            Some(PopupAccentDue::Settled)
        );
    }

    #[test]
    fn popup_accent_state_reverting_to_applied_value_clears_pending() {
        let now = Instant::now();
        let mut state = PopupAccentState::default();
        let first = acrylic(10);
        state.set_desired(first, now);
        state.mark_applied(first);
        state.set_desired(acrylic(11), now + Duration::from_millis(1));

        assert!(state.set_desired(first, now + Duration::from_millis(2)));
        assert_eq!(state.due(now + POPUP_ACCENT_SETTLE_DELAY), None);
        assert!(!state.pending());
    }

    #[test]
    fn popup_border_state_applies_creation_then_settles_theme_changes() {
        let now = Instant::now();
        let mut state = PopupBorderState::default();
        let first = scene::Color::rgb(0x11, 0x22, 0x33);
        let second = scene::Color::rgb(0x44, 0x55, 0x66);

        assert!(state.set_desired(first, now));
        assert_eq!(state.due(now), Some(PopupBorderDue::Immediate));
        state.mark_applied(first);
        assert!(!state.pending());

        assert!(state.set_desired(second, now + Duration::from_millis(1)));
        assert_eq!(state.due(now + Duration::from_millis(1)), None);
        assert!(state.pending());
        let changed_at = state.changed_instant();
        assert!(!state.set_desired(second, now + Duration::from_millis(100)));
        assert_eq!(state.changed_instant(), changed_at);
        assert_eq!(
            state.due(now + Duration::from_millis(1) + POPUP_BORDER_SETTLE_DELAY),
            Some(PopupBorderDue::Settled)
        );
        assert_eq!(state.desired(), Some(second));
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
            crate::render::CompositeAlphaPreference::PreMultiplied
        );
    }

    #[test]
    fn popup_material_realization_requires_premultiplied_alpha() {
        assert_eq!(
            PopupPresentationMode::CompositionBacked.realization_for(
                wgpu::TextureFormat::Bgra8Unorm,
                wgpu::CompositeAlphaMode::PreMultiplied,
                PopupMaterialPreference::System
            ),
            PopupMaterialRealization::WindowsAccentAcrylic
        );
        assert_eq!(
            PopupPresentationMode::CompositionBacked.realization_for(
                wgpu::TextureFormat::Bgra8Unorm,
                wgpu::CompositeAlphaMode::Opaque,
                PopupMaterialPreference::System
            ),
            PopupMaterialRealization::OpaqueFallback
        );
        assert_eq!(
            PopupPresentationMode::RedirectedFallback.realization_for(
                wgpu::TextureFormat::Bgra8Unorm,
                wgpu::CompositeAlphaMode::PreMultiplied,
                PopupMaterialPreference::System
            ),
            PopupMaterialRealization::WindowsAccentAcrylic
        );
        assert_eq!(
            PopupPresentationMode::RedirectedFallback.realization_for(
                wgpu::TextureFormat::Bgra8UnormSrgb,
                wgpu::CompositeAlphaMode::PreMultiplied,
                PopupMaterialPreference::System
            ),
            PopupMaterialRealization::OpaqueFallback
        );
    }

    #[test]
    fn popup_material_diagnostics_can_force_realization() {
        assert_eq!(
            PopupPresentationMode::CompositionBacked.realization_for(
                wgpu::TextureFormat::Bgra8Unorm,
                wgpu::CompositeAlphaMode::PreMultiplied,
                PopupMaterialPreference::OpaqueFallback
            ),
            PopupMaterialRealization::OpaqueFallback
        );
        assert_eq!(
            PopupPresentationMode::CompositionBacked.realization_for(
                wgpu::TextureFormat::Bgra8Unorm,
                wgpu::CompositeAlphaMode::PreMultiplied,
                PopupMaterialPreference::NoAccent
            ),
            PopupMaterialRealization::TransparentNoAccent
        );
        assert_eq!(
            PopupPresentationMode::RedirectedFallback.realization_for(
                wgpu::TextureFormat::Bgra8Unorm,
                wgpu::CompositeAlphaMode::PreMultiplied,
                PopupMaterialPreference::NoAccent
            ),
            PopupMaterialRealization::TransparentNoAccent
        );
    }
}
