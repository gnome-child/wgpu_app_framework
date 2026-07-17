use crate::geometry::area;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};

use crate::{ime as app_ime, interaction, overlay, pointer, render, scene};

use super::super::{session, window as app_window};

mod adapter;
#[cfg(target_os = "windows")]
mod composition;
mod context;
mod error;
mod ime;
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
    renderers: HashMap<render::surface::Format, render::Renderer>,
    active_presentations: HashMap<app_window::Id, crate::shell::Presentation>,
    pending_presentations: HashMap<app_window::Id, PendingPresentation>,
    windows: HashMap<app_window::Id, window::Window>,
    popups: HashMap<PopupKey, PopupWindow>,
    popup_pool: HashMap<app_window::Id, Vec<PopupHost>>,
    popup_pool_capacity: HashMap<app_window::Id, usize>,
    popup_prewarm: HashMap<app_window::Id, PopupPrewarmState>,
    next_popup_generation: u64,
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

struct PendingPresentation<T = crate::shell::Presentation> {
    preparing: T,
    latest: Option<T>,
}

struct PendingCompletion<T> {
    prepared: T,
    successor: Option<T>,
}

impl<T> PendingPresentation<T> {
    fn new(preparing: T) -> Self {
        Self {
            preparing,
            latest: None,
        }
    }

    fn enqueue_by(&mut self, presentation: T, same_identity: impl Fn(&T, &T) -> bool) {
        if same_identity(&self.preparing, &presentation)
            || self
                .latest
                .as_ref()
                .is_some_and(|latest| same_identity(latest, &presentation))
        {
            return;
        }
        self.latest = Some(presentation);
    }

    fn newest(&self) -> &T {
        self.latest.as_ref().unwrap_or(&self.preparing)
    }

    fn into_newest(self) -> T {
        self.latest.unwrap_or(self.preparing)
    }

    fn complete(self) -> PendingCompletion<T> {
        PendingCompletion {
            prepared: self.preparing,
            successor: self.latest,
        }
    }
}

impl PendingPresentation<crate::shell::Presentation> {
    fn enqueue(&mut self, presentation: crate::shell::Presentation) {
        self.enqueue_by(presentation, |left, right| {
            Arc::ptr_eq(left.stack(), right.stack())
        });
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
struct PopupKey {
    parent: app_window::Id,
    id: interaction::Id,
    context_fingerprint: Option<crate::popup::ContextFingerprint>,
}

struct PopupWindow {
    host: PopupHost,
    accepts_input: bool,
    realization: Option<crate::popup::Realization>,
    pending_realization: Option<crate::popup::Realization>,
    generation: crate::popup::Generation,
    reconfiguring: bool,
    geometry: PopupGeometryState,
    pending_geometry: Option<PopupGeometry>,
    accent: PopupAccentState,
    border: PopupBorderState,
    presentation_prepared: bool,
    exposed: bool,
    last_presented_stack: Option<Arc<scene::Stack>>,
    first_present: PopupFirstPresentTrace,
    material_readiness: PopupMaterialReadiness,
    material: Option<overlay::PopupMaterial>,
    material_realization: Option<PopupMaterialRealization>,
    source_scene: Option<crate::scene::Scene>,
    context_fingerprint: Option<crate::popup::ContextFingerprint>,
}

struct PopupHost {
    window: window::Window,
    presentation_mode: PopupPresentationMode,
    reused: bool,
    #[cfg(target_os = "windows")]
    composition: Option<composition::Host>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum PopupMaterialReadiness {
    NotRequired,
    Pending(u64),
    Committed(u64),
    Ready(u64),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum PopupPrewarmState {
    Armed,
    Scheduled,
    Complete,
}

impl PopupMaterialReadiness {
    fn observe(&mut self, observed: Self) {
        match observed {
            Self::NotRequired => *self = Self::NotRequired,
            Self::Pending(generation) => {
                if self.generation() != Some(generation) {
                    *self = Self::Pending(generation);
                }
            }
            Self::Committed(generation) => {
                if matches!(self, Self::Pending(current) if *current == generation) {
                    *self = Self::Committed(generation);
                }
            }
            Self::Ready(generation) => {
                if matches!(self, Self::Committed(current) if *current == generation) {
                    *self = Self::Ready(generation);
                }
            }
        }
    }

    fn generation(self) -> Option<u64> {
        match self {
            Self::NotRequired => None,
            Self::Pending(generation) | Self::Committed(generation) | Self::Ready(generation) => {
                Some(generation)
            }
        }
    }

    fn mark_ready(&mut self, generation: u64) -> bool {
        if *self != Self::Committed(generation) {
            return false;
        }
        *self = Self::Ready(generation);
        true
    }
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
    realization: crate::popup::Realization,
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
    generation: crate::popup::Generation,
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
    ContentReady,
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
        Self {
            parent,
            id,
            context_fingerprint: None,
        }
    }

    fn for_presentation(presentation: &overlay::PopupPresentation) -> Self {
        Self {
            parent: presentation.parent(),
            id: presentation.id(),
            context_fingerprint: presentation.context_fingerprint(),
        }
    }
}

impl PopupWindow {
    fn new(
        mut host: PopupHost,
        lifecycle_epoch: Instant,
        generation: crate::popup::Generation,
    ) -> Self {
        let material_readiness = host.readiness_for_session();
        Self {
            host,
            accepts_input: false,
            realization: None,
            pending_realization: None,
            generation,
            reconfiguring: false,
            geometry: PopupGeometryState::default(),
            pending_geometry: None,
            accent: PopupAccentState::default(),
            border: PopupBorderState::default(),
            presentation_prepared: false,
            exposed: false,
            last_presented_stack: None,
            first_present: PopupFirstPresentTrace::new(lifecycle_epoch, generation),
            material_readiness,
            material: None,
            material_realization: None,
            source_scene: None,
            context_fingerprint: None,
        }
    }

    fn into_host(self) -> PopupHost {
        self.host
    }
}

impl PopupHost {
    fn new(
        window: window::Window,
        presentation_mode: PopupPresentationMode,
        #[cfg(target_os = "windows")] composition: Option<composition::Host>,
    ) -> Self {
        Self {
            window,
            presentation_mode,
            reused: false,
            #[cfg(target_os = "windows")]
            composition,
        }
    }

    fn readiness_for_session(&mut self) -> PopupMaterialReadiness {
        #[cfg(target_os = "windows")]
        let readiness = self
            .composition
            .as_mut()
            .map(composition::Host::material_readiness)
            .map(|readiness| match readiness {
                composition::MaterialReadiness::NotRequired => PopupMaterialReadiness::NotRequired,
                composition::MaterialReadiness::Pending(generation) => {
                    PopupMaterialReadiness::Pending(generation)
                }
                composition::MaterialReadiness::Committed(generation) => {
                    PopupMaterialReadiness::Committed(generation)
                }
            })
            .unwrap_or(PopupMaterialReadiness::NotRequired);
        #[cfg(not(target_os = "windows"))]
        let readiness = PopupMaterialReadiness::NotRequired;

        readiness_for_reused_session(readiness)
    }
}

fn readiness_for_reused_session(readiness: PopupMaterialReadiness) -> PopupMaterialReadiness {
    match readiness {
        PopupMaterialReadiness::Ready(generation)
        | PopupMaterialReadiness::Committed(generation) => {
            PopupMaterialReadiness::Pending(generation)
        }
        PopupMaterialReadiness::Pending(generation) => PopupMaterialReadiness::Pending(generation),
        PopupMaterialReadiness::NotRequired => PopupMaterialReadiness::NotRequired,
    }
}

impl Drop for PopupHost {
    fn drop(&mut self) {
        self.window.hide_popup_before_teardown();
        self.window.remove_popup_subclass();
        log::debug!(
            target: "wgpu_l3::native_popup",
            "popup lifecycle stage=hidden-before-teardown"
        );
    }
}

impl PopupEventTarget {
    pub(in crate::platform) fn parent(self) -> app_window::Id {
        self.realization.parent()
    }

    pub(in crate::platform) fn id(self) -> interaction::Id {
        self.realization.popup()
    }

    pub(in crate::platform) fn realization(self) -> crate::popup::Realization {
        self.realization
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
    fn new(created_at: Instant, generation: crate::popup::Generation) -> Self {
        Self {
            created_at,
            generation,
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
    fn logical_area(self) -> area::Logical {
        area::logical(self.width, self.height)
    }

    #[cfg(test)]
    fn rounded_physical_area(self) -> area::Physical {
        let scale = f64::from_bits(self.scale_factor_bits);
        area::physical(
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

    fn alpha_preference(self) -> render::surface::CompositeAlphaPreference {
        match self {
            Self::CompositionBacked => render::surface::CompositeAlphaPreference::PreMultiplied,
            Self::RedirectedFallback => render::surface::CompositeAlphaPreference::PreMultiplied,
        }
    }

    fn realization_for(
        self,
        support: render::surface::WindowsPopupSupport,
        preference: overlay::PopupMaterialPreference,
    ) -> PopupMaterialRealization {
        match preference {
            overlay::PopupMaterialPreference::OpaqueFallback => {
                PopupMaterialRealization::OpaqueFallback
            }
            overlay::PopupMaterialPreference::NoAccent => {
                if support.is_available() {
                    PopupMaterialRealization::TransparentNoAccent
                } else {
                    PopupMaterialRealization::OpaqueFallback
                }
            }
            overlay::PopupMaterialPreference::System => {
                if support.is_available() {
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
        support: render::surface::WindowsPopupSupport,
    ) -> Option<&'static str> {
        match self {
            Self::WindowsAccentAcrylic | Self::TransparentNoAccent => None,
            Self::OpaqueFallback => support.fallback_reason(),
        }
    }
}

impl Native {
    pub fn new() -> Self {
        Self {
            context: None,
            renderers: HashMap::new(),
            active_presentations: HashMap::new(),
            pending_presentations: HashMap::new(),
            windows: HashMap::new(),
            popups: HashMap::new(),
            popup_pool: HashMap::new(),
            popup_pool_capacity: HashMap::new(),
            popup_prewarm: HashMap::new(),
            next_popup_generation: 0,
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
        ApplyDue, CursorHost, Native, POPUP_SYS_SETTLE_DELAY, PendingPresentation,
        PopupAccentState, PopupBorderState, PopupGeometry, PopupGeometryState, PopupKey,
        PopupMaterialReadiness, PopupMaterialRealization, PopupPresentationMode, popup_accent_due,
        popup_border_due, popup_geometry_due, readiness_for_reused_session,
    };
    use crate::geometry::area;
    use crate::overlay::PopupMaterialPreference;
    use crate::platform::native::sys::PopupAccentMaterial;
    use crate::scene;
    use std::time::{Duration, Instant};

    #[test]
    fn pending_presentations_keep_one_preparing_and_only_the_latest_successor() {
        let mut pending = PendingPresentation::new((1_u32, "preparing"));

        pending.enqueue_by((2, "first latest"), |left, right| left.0 == right.0);
        pending.enqueue_by((3, "newest latest"), |left, right| left.0 == right.0);

        assert_eq!(pending.preparing, (1, "preparing"));
        assert_eq!(pending.latest, Some((3, "newest latest")));
        assert_eq!(pending.newest(), &(3, "newest latest"));

        pending.enqueue_by((1, "duplicate preparing"), |left, right| left.0 == right.0);
        pending.enqueue_by((3, "duplicate latest"), |left, right| left.0 == right.0);

        assert_eq!(pending.preparing, (1, "preparing"));
        assert_eq!(pending.latest, Some((3, "newest latest")));

        pending.enqueue_by((4, "replacement latest"), |left, right| left.0 == right.0);
        assert_eq!(pending.preparing, (1, "preparing"));
        assert_eq!(pending.latest, Some((4, "replacement latest")));

        let completed = pending.complete();
        assert_eq!(completed.prepared, (1, "preparing"));
        assert_eq!(completed.successor, Some((4, "replacement latest")));

        let completed = PendingPresentation::new((4, "ready")).complete();
        assert_eq!(completed.prepared, (4, "ready"));
        assert_eq!(completed.successor, None);
    }

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
    fn reused_session_preserves_the_hosts_material_generation() {
        assert_eq!(
            readiness_for_reused_session(PopupMaterialReadiness::Ready(7)),
            PopupMaterialReadiness::Pending(7)
        );
        assert_eq!(
            readiness_for_reused_session(PopupMaterialReadiness::Committed(8)),
            PopupMaterialReadiness::Pending(8)
        );
        assert_eq!(
            readiness_for_reused_session(PopupMaterialReadiness::Pending(9)),
            PopupMaterialReadiness::Pending(9)
        );
        assert_eq!(
            readiness_for_reused_session(PopupMaterialReadiness::NotRequired),
            PopupMaterialReadiness::NotRequired
        );
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
    fn contextual_popup_hosts_are_keyed_by_captured_owner() {
        let parent = crate::window::Id::new(42);
        let id = crate::interaction::Id::new("context_menu");
        let mut next = 1;
        let first_owner = crate::composition::tree::NodeId::layout(&mut next);
        let second_owner = crate::composition::tree::NodeId::layout(&mut next);
        let first = PopupKey {
            parent,
            id,
            context_fingerprint: Some(crate::popup::ContextFingerprint::from_owner(first_owner)),
        };
        let second = PopupKey {
            parent,
            id,
            context_fingerprint: Some(crate::popup::ContextFingerprint::from_owner(second_owner)),
        };

        assert_ne!(
            first, second,
            "context retargeting must enter a fresh host while the old authored-menu host retires"
        );
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

        assert_eq!(geometry.logical_area(), area::logical(240.0, 180.0));
        assert_eq!(geometry.rounded_physical_area(), area::physical(360, 270));
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
            crate::render::surface::CompositeAlphaPreference::PreMultiplied
        );

        assert!(!PopupPresentationMode::RedirectedFallback.no_redirection_bitmap());
        assert_eq!(
            PopupPresentationMode::RedirectedFallback.alpha_preference(),
            crate::render::surface::CompositeAlphaPreference::PreMultiplied
        );
    }

    #[test]
    fn popup_material_realization_requires_premultiplied_alpha() {
        assert_eq!(
            PopupPresentationMode::CompositionBacked.realization_for(
                crate::render::surface::WindowsPopupSupport::Available,
                PopupMaterialPreference::System
            ),
            PopupMaterialRealization::WindowsAccentAcrylic
        );
        assert_eq!(
            PopupPresentationMode::CompositionBacked.realization_for(
                crate::render::surface::WindowsPopupSupport::PremultipliedAlphaUnavailable,
                PopupMaterialPreference::System
            ),
            PopupMaterialRealization::OpaqueFallback
        );
        assert_eq!(
            PopupPresentationMode::RedirectedFallback.realization_for(
                crate::render::surface::WindowsPopupSupport::Available,
                PopupMaterialPreference::System
            ),
            PopupMaterialRealization::WindowsAccentAcrylic
        );
        assert_eq!(
            PopupPresentationMode::RedirectedFallback.realization_for(
                crate::render::surface::WindowsPopupSupport::NonSrgbFormatUnavailable,
                PopupMaterialPreference::System
            ),
            PopupMaterialRealization::OpaqueFallback
        );
    }

    #[test]
    fn popup_material_diagnostics_can_force_realization() {
        assert_eq!(
            PopupPresentationMode::CompositionBacked.realization_for(
                crate::render::surface::WindowsPopupSupport::Available,
                PopupMaterialPreference::OpaqueFallback
            ),
            PopupMaterialRealization::OpaqueFallback
        );
        assert_eq!(
            PopupPresentationMode::CompositionBacked.realization_for(
                crate::render::surface::WindowsPopupSupport::Available,
                PopupMaterialPreference::NoAccent
            ),
            PopupMaterialRealization::TransparentNoAccent
        );
        assert_eq!(
            PopupPresentationMode::RedirectedFallback.realization_for(
                crate::render::surface::WindowsPopupSupport::Available,
                PopupMaterialPreference::NoAccent
            ),
            PopupMaterialRealization::TransparentNoAccent
        );
    }
}
