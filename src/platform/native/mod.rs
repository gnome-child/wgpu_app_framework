use std::collections::HashMap;
use std::time::{Duration, Instant};

use crate::{geometry, ime as app_ime, interaction, overlay, pointer, render, scene};

use super::super::{session, window as app_window};

mod adapter;
mod color;
#[cfg(target_os = "windows")]
mod composition;
mod context;
mod error;
mod ime;
mod paint;
mod poll;
mod popup;
mod request;
mod settle;
mod surface;
mod sys;
mod window;

pub use context::NativeContext;
pub use error::NativeError;

use settle::{ApplyDue, SysApplicator};

pub struct Native {
    context: Option<render::Context>,
    renderers: HashMap<wgpu::TextureFormat, render::Renderer>,
    windows: HashMap<app_window::Id, window::Window>,
    popups: HashMap<PopupKey, PopupWindow>,
    raw_windows: HashMap<winit::window::WindowId, app_window::Id>,
    raw_popups: HashMap<winit::window::WindowId, PopupKey>,
    cursor_hosts: HashMap<app_window::Id, CursorHost>,
    cursor_values: HashMap<app_window::Id, pointer::Cursor>,
    ime_targets: HashMap<app_window::Id, app_ime::Target>,
    requests: Vec<session::Request>,
    poll_requested: bool,
    #[cfg(target_os = "windows")]
    composition: Option<composition::Runtime>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
struct PopupKey {
    parent: app_window::Id,
    id: interaction::Id,
}

struct PopupWindow {
    window: window::Window,
    bounds: geometry::Rect,
    panel_offset_physical: (i32, i32),
    geometry: PopupGeometryState,
    accent: PopupAccentState,
    border: PopupBorderState,
    presentation_prepared: bool,
    exposed: bool,
    first_present: PopupFirstPresentTrace,
    material: Option<overlay::PopupMaterial>,
    presentation_mode: PopupPresentationMode,
    material_realization: Option<PopupMaterialRealization>,
    #[cfg(target_os = "windows")]
    composition: Option<composition::Host>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum CursorHost {
    Parent,
    Popup(PopupKey),
    Outside,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ImeHost {
    Parent,
    Popup(PopupKey),
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
    panel_offset_physical: (i32, i32),
    scale_factor: f64,
    first_present_elapsed_micros: u128,
    first_present_stage: &'static str,
}

type PopupGeometryState = SysApplicator<PopupGeometry>;
type PopupAccentState = SysApplicator<sys::PopupAccentMaterial>;
type PopupBorderState = SysApplicator<scene::Color>;

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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum PopupFirstPresentAction {
    None,
    RequestRedraw,
    Expose,
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
            panel_offset_physical: (0, 0),
            geometry: PopupGeometryState::default(),
            accent: PopupAccentState::default(),
            border: PopupBorderState::default(),
            presentation_prepared: false,
            exposed: false,
            first_present: PopupFirstPresentTrace::new(),
            material: None,
            presentation_mode,
            material_realization: None,
            #[cfg(target_os = "windows")]
            composition: None,
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

    pub(in crate::platform) fn panel_offset_physical(self) -> (i32, i32) {
        self.panel_offset_physical
    }

    pub(in crate::platform) fn scale_factor(self) -> f64 {
        self.scale_factor
    }

    pub(in crate::platform) fn first_present_elapsed_micros(self) -> u128 {
        self.first_present_elapsed_micros
    }

    pub(in crate::platform) fn first_present_stage(self) -> &'static str {
        self.first_present_stage
    }
}

// Windows accent re-application rebuilds compositor-side material state. Keep
// parameter churn settle-rate while preserving instant material presence.
const POPUP_SYS_SETTLE_DELAY: Duration = Duration::from_millis(150);

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

    fn stage(&self) -> &'static str {
        match self.state {
            PopupFirstPresentState::AwaitingFirst => "awaiting-first",
            PopupFirstPresentState::AwaitingConfirmation => "awaiting-confirmation",
            PopupFirstPresentState::Complete => "complete",
        }
    }
}

fn accent_presence(material: sys::PopupAccentMaterial) -> PopupAccentPresence {
    match material {
        sys::PopupAccentMaterial::Disabled => PopupAccentPresence::Disabled,
        sys::PopupAccentMaterial::Acrylic { .. } => PopupAccentPresence::Acrylic,
    }
}

fn popup_geometry_due(state: &PopupGeometryState, now: Instant) -> Option<ApplyDue> {
    state.due(now, Duration::ZERO, |_, _| true)
}

fn popup_accent_due(state: &PopupAccentState, now: Instant) -> Option<ApplyDue> {
    state.due(now, POPUP_SYS_SETTLE_DELAY, |applied, desired| {
        accent_presence(applied) != accent_presence(desired)
    })
}

fn popup_border_due(state: &PopupBorderState, now: Instant) -> Option<ApplyDue> {
    state.due(now, POPUP_SYS_SETTLE_DELAY, |_, _| false)
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
            cursor_hosts: HashMap::new(),
            cursor_values: HashMap::new(),
            ime_targets: HashMap::new(),
            requests: Vec::new(),
            poll_requested: false,
            #[cfg(target_os = "windows")]
            composition: None,
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
        ApplyDue, CursorHost, Native, POPUP_SYS_SETTLE_DELAY, PopupAccentState, PopupBorderState,
        PopupGeometry, PopupGeometryState, PopupKey, PopupMaterialRealization,
        PopupPresentationMode, popup_accent_due, popup_border_due, popup_geometry_due,
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
    fn cursor_routing_moves_unchanged_logical_value_between_physical_hosts() {
        let parent = crate::window::Id::new(41);
        let key = PopupKey::new(parent, crate::interaction::Id::new("palette"));
        let mut native = Native::new();
        native.cursor_hosts.insert(parent, CursorHost::Parent);
        native
            .cursor_values
            .insert(parent, crate::pointer::Cursor::Text);

        native.set_cursor_host(parent, CursorHost::Popup(key));

        assert_eq!(
            native.cursor_hosts.get(&parent),
            Some(&CursorHost::Popup(key))
        );
        assert_eq!(
            native.cursor_values.get(&parent),
            Some(&crate::pointer::Cursor::Text)
        );

        native.set_cursor_host(parent, CursorHost::Outside);
        assert_eq!(native.cursor_hosts.get(&parent), Some(&CursorHost::Outside));

        native.rehome_cursor_from_popup(key);
        assert_eq!(native.cursor_hosts.get(&parent), Some(&CursorHost::Outside));
        native.set_cursor_host(parent, CursorHost::Popup(key));
        native.rehome_cursor_from_popup(key);
        assert_eq!(native.cursor_hosts.get(&parent), Some(&CursorHost::Parent));
    }

    #[test]
    fn popup_geometry_state_skips_unchanged_redraws() {
        let now = Instant::now();
        let mut state = PopupGeometryState::default();
        let desired = geometry(10, 20, 1.0);

        assert!(state.set_desired(desired, now));
        assert_eq!(popup_geometry_due(&state, now), Some(ApplyDue::Initial));
        state.mark_applied(desired);

        assert!(
            !state.set_desired(desired, now + Duration::from_millis(1))
                && popup_geometry_due(&state, now + Duration::from_millis(1)).is_none(),
            "fade/redraw frames with unchanged geometry must be draw-only"
        );
    }

    #[test]
    fn popup_geometry_state_reapplies_real_geometry_changes() {
        let now = Instant::now();
        let mut state = PopupGeometryState::default();
        let desired = geometry(10, 20, 1.0);
        state.set_desired(desired, now);
        state.mark_applied(desired);

        assert!(
            state.set_desired(geometry(11, 20, 1.0), now)
                && popup_geometry_due(&state, now) == Some(ApplyDue::Immediate),
            "parent move or anchor change must reconfigure popup position"
        );
        state.set_desired(desired, now);
        state.mark_applied(desired);
        assert!(
            state.set_desired(geometry(10, 20, 1.5), now)
                && popup_geometry_due(&state, now) == Some(ApplyDue::Immediate),
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
        assert_eq!(popup_accent_due(&state, now), Some(ApplyDue::Initial));
        state.mark_applied(desired);
        assert_eq!(popup_accent_due(&state, now), None);
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
            popup_accent_due(&state, now + Duration::from_millis(1)),
            Some(ApplyDue::Immediate)
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
        assert_eq!(
            popup_accent_due(&state, now + Duration::from_millis(1)),
            None
        );
        assert!(state.pending());

        assert!(state.set_desired(third, now + Duration::from_millis(20)));
        assert_eq!(
            popup_accent_due(&state, now + Duration::from_millis(149)),
            None
        );
        assert_eq!(
            popup_accent_due(
                &state,
                now + Duration::from_millis(20) + POPUP_SYS_SETTLE_DELAY
            ),
            Some(ApplyDue::Settled)
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
            popup_accent_due(&state, now + POPUP_SYS_SETTLE_DELAY),
            Some(ApplyDue::Settled)
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
        assert_eq!(popup_accent_due(&state, now + POPUP_SYS_SETTLE_DELAY), None);
        assert!(!state.pending());
    }

    #[test]
    fn popup_border_state_applies_creation_then_settles_theme_changes() {
        let now = Instant::now();
        let mut state = PopupBorderState::default();
        let first = scene::Color::rgb(0x11, 0x22, 0x33);
        let second = scene::Color::rgb(0x44, 0x55, 0x66);

        assert!(state.set_desired(first, now));
        assert_eq!(popup_border_due(&state, now), Some(ApplyDue::Initial));
        state.mark_applied(first);
        assert!(!state.pending());

        assert!(state.set_desired(second, now + Duration::from_millis(1)));
        assert_eq!(
            popup_border_due(&state, now + Duration::from_millis(1)),
            None
        );
        assert!(state.pending());
        let changed_at = state.changed_instant();
        assert!(!state.set_desired(second, now + Duration::from_millis(100)));
        assert_eq!(state.changed_instant(), changed_at);
        assert_eq!(
            popup_border_due(
                &state,
                now + Duration::from_millis(1) + POPUP_SYS_SETTLE_DELAY
            ),
            Some(ApplyDue::Settled)
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
