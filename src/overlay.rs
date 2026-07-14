use std::{
    collections::{HashMap, HashSet},
    time::{Duration, Instant},
};

use crate::{animation, geometry, interaction, notification, scene, theme, window};

const DEFAULT_AFTERLIFE_LIMIT: usize = 8;

#[derive(Debug, Clone)]
pub(crate) struct Draft {
    id: interaction::Id,
    bounds: geometry::Rect,
    scene: scene::Scene,
    preference: Preference,
    popup_material_preference: PopupMaterialPreference,
    popup_border: scene::Color,
    text_caret_rect: Option<geometry::Rect>,
    placement: Option<geometry::PlacementRequest>,
    context_fingerprint: Option<crate::popup::ContextFingerprint>,
    accepts_input: bool,
    force_group_at_full_opacity: bool,
}

#[derive(Debug, Clone)]
struct Entry {
    id: interaction::Id,
    order: u64,
    bounds: geometry::Rect,
    scene: scene::Scene,
    backend: Backend,
    popup_material_preference: PopupMaterialPreference,
    popup_border: scene::Color,
    text_caret_rect: Option<geometry::Rect>,
    placement: Option<geometry::PlacementRequest>,
    context_fingerprint: Option<crate::popup::ContextFingerprint>,
    accepts_input: bool,
    opacity: f32,
    fade: PopupFade,
    state: State,
    elapsed: Duration,
    force_group_at_full_opacity: bool,
    demotion_marker: bool,
    frame_number: u64,
    lifecycle_epoch: Instant,
}

#[derive(Debug, Clone)]
struct Ghost {
    id: interaction::Id,
    original_order: u64,
    scene: scene::Scene,
    started_at: Instant,
    duration: Duration,
    from_opacity: f32,
}

#[derive(Debug, Clone)]
struct RetiringPopup {
    id: interaction::Id,
    original_order: u64,
    bounds: geometry::Rect,
    scene: scene::Scene,
    popup_material_preference: PopupMaterialPreference,
    popup_border: scene::Color,
    placement: Option<geometry::PlacementRequest>,
    context_fingerprint: Option<crate::popup::ContextFingerprint>,
    started_at: Instant,
    duration: Duration,
    from_opacity: f32,
}

/// The semantic overlay instance that may be updated in place. Contextual
/// panels share one public interaction id, so their captured responder path is
/// part of the identity that decides whether this is an update or a retarget.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
struct Identity {
    id: interaction::Id,
    context_fingerprint: Option<crate::popup::ContextFingerprint>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum State {
    Entering,
    Live,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) enum PopupFade {
    Entering {
        duration: Duration,
        started_at: Instant,
    },
    Stable,
    Exiting {
        duration: Duration,
        started_at: Instant,
        from_opacity: f32,
    },
}

impl PopupFade {
    pub(crate) fn opacity_at(self, now: Instant) -> f32 {
        match self {
            Self::Entering {
                duration,
                started_at,
            } => live_opacity(started_at, duration.as_millis() as u64, now).0,
            Self::Stable => 1.0,
            Self::Exiting {
                duration,
                started_at,
                from_opacity,
            } => exit_opacity_at(started_at, duration, from_opacity, now),
        }
    }
}

#[derive(Debug, Clone)]
pub(crate) struct Layer {
    id: interaction::Id,
    order: u64,
    bounds: geometry::Rect,
    scene: scene::Scene,
    opacity: f32,
    fade: PopupFade,
    lifecycle: Lifecycle,
    backend: Backend,
    popup_material_preference: PopupMaterialPreference,
    popup_border: scene::Color,
    text_caret_rect: Option<geometry::Rect>,
    placement: Option<geometry::PlacementRequest>,
    context_fingerprint: Option<crate::popup::ContextFingerprint>,
    accepts_input: bool,
    force_group_at_full_opacity: bool,
    demotion_marker: bool,
    frame_number: u64,
    lifecycle_epoch: Instant,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Lifecycle {
    Live { state: State, elapsed: Duration },
    Ghost { elapsed: Duration },
    RetiringPopup { elapsed: Duration },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum LayerKind {
    Live,
    Ghost,
    RetiringPopup,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Preference {
    InFrame,
    NativePopup,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Backend {
    InFrame,
    NativePopup,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum PopupMaterialPreference {
    System,
    OpaqueFallback,
    NoAccent,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Capabilities {
    InFrameOnly,
    AnimatedNativePopups,
    ImmediateNativePopups,
}

#[derive(Debug, Clone)]
pub(crate) struct Update {
    layers: Vec<Layer>,
    schedule: animation::Schedule,
}

#[derive(Debug, Clone)]
pub(crate) struct PopupPresentation {
    parent: window::Id,
    id: interaction::Id,
    local_bounds: geometry::Rect,
    placement: Option<geometry::PlacementRequest>,
    scene: scene::Scene,
    opacity: f32,
    fade: PopupFade,
    material: PopupMaterial,
    border: scene::Color,
    lifecycle_epoch: Instant,
    paint_only: bool,
    kind: LayerKind,
    context_fingerprint: Option<crate::popup::ContextFingerprint>,
    accepts_input: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum PopupMaterial {
    NativeWindow {
        dark: bool,
        tint: scene::Color,
        preference: PopupMaterialPreference,
    },
}

#[derive(Debug)]
pub(crate) struct Store {
    windows: HashMap<window::Id, WindowState>,
    afterlife_limit: usize,
}

#[derive(Debug, Default)]
struct WindowState {
    live: Vec<Live>,
    ghosts: Vec<Ghost>,
    retiring_popups: Vec<RetiringPopup>,
    next_order: u64,
    frame_number: u64,
}

#[derive(Debug, Clone)]
struct Live {
    id: interaction::Id,
    order: u64,
    bounds: geometry::Rect,
    scene: scene::Scene,
    backend: Backend,
    native_animation: bool,
    popup_material_preference: PopupMaterialPreference,
    popup_border: scene::Color,
    placement: Option<geometry::PlacementRequest>,
    context_fingerprint: Option<crate::popup::ContextFingerprint>,
    appeared_at: Instant,
    demotion_logged: bool,
}

impl Draft {
    pub(crate) fn new(id: interaction::Id, bounds: geometry::Rect, scene: scene::Scene) -> Self {
        Self {
            id,
            bounds,
            scene,
            preference: Preference::InFrame,
            popup_material_preference: PopupMaterialPreference::System,
            popup_border: scene::Color::rgba(0, 0, 0, 0),
            text_caret_rect: None,
            placement: None,
            context_fingerprint: None,
            accepts_input: true,
            force_group_at_full_opacity: false,
        }
    }

    pub(crate) fn prefer(mut self, preference: Preference) -> Self {
        self.preference = preference;
        self
    }

    pub(crate) fn force_group_at_full_opacity(mut self, force: bool) -> Self {
        self.force_group_at_full_opacity = force;
        self
    }

    pub(crate) fn popup_material_preference(mut self, preference: PopupMaterialPreference) -> Self {
        self.popup_material_preference = preference;
        self
    }

    pub(crate) fn popup_border(mut self, border: scene::Color) -> Self {
        self.popup_border = border;
        self
    }

    pub(crate) fn text_caret_rect(mut self, text_caret_rect: Option<geometry::Rect>) -> Self {
        self.text_caret_rect = text_caret_rect;
        self
    }

    pub(crate) fn placement(mut self, placement: Option<geometry::PlacementRequest>) -> Self {
        self.placement = placement;
        self
    }

    pub(crate) fn context_fingerprint(
        mut self,
        fingerprint: Option<crate::popup::ContextFingerprint>,
    ) -> Self {
        self.context_fingerprint = fingerprint;
        self
    }

    pub(crate) fn accepts_input(mut self, accepts_input: bool) -> Self {
        self.accepts_input = accepts_input;
        self
    }

    fn identity(&self) -> Identity {
        Identity {
            id: self.id,
            context_fingerprint: self.context_fingerprint,
        }
    }

    #[cfg(test)]
    pub(crate) fn scene(&self) -> &scene::Scene {
        &self.scene
    }
}

impl Entry {
    fn layer(&self) -> Layer {
        Layer {
            id: self.id,
            order: self.order,
            bounds: self.bounds,
            scene: self.scene.clone(),
            opacity: self.opacity,
            fade: self.fade,
            lifecycle: Lifecycle::Live {
                state: self.state,
                elapsed: self.elapsed,
            },
            backend: self.backend,
            popup_material_preference: self.popup_material_preference,
            popup_border: self.popup_border,
            text_caret_rect: self.text_caret_rect,
            placement: self.placement,
            context_fingerprint: self.context_fingerprint,
            accepts_input: self.accepts_input,
            force_group_at_full_opacity: self.force_group_at_full_opacity,
            demotion_marker: self.demotion_marker,
            frame_number: self.frame_number,
            lifecycle_epoch: self.lifecycle_epoch,
        }
    }
}

impl Live {
    fn identity(&self) -> Identity {
        Identity {
            id: self.id,
            context_fingerprint: self.context_fingerprint,
        }
    }
}

impl Ghost {
    fn layer_at(&self, now: Instant, frame_number: u64) -> Option<Layer> {
        if self.expired_at(now) {
            return None;
        }

        Some(Layer {
            id: self.id,
            order: self.original_order,
            bounds: geometry::Rect::from_size(self.scene.size()),
            scene: self.scene.clone(),
            opacity: self.opacity_at(now),
            fade: PopupFade::Exiting {
                duration: self.duration,
                started_at: self.started_at,
                from_opacity: self.from_opacity,
            },
            lifecycle: Lifecycle::Ghost {
                elapsed: now.saturating_duration_since(self.started_at),
            },
            backend: Backend::InFrame,
            popup_material_preference: PopupMaterialPreference::System,
            popup_border: scene::Color::rgba(0, 0, 0, 0),
            text_caret_rect: None,
            placement: None,
            context_fingerprint: None,
            accepts_input: false,
            force_group_at_full_opacity: false,
            demotion_marker: false,
            frame_number,
            lifecycle_epoch: self.started_at,
        })
    }

    fn opacity_at(&self, now: Instant) -> f32 {
        exit_opacity_at(self.started_at, self.duration, self.from_opacity, now)
    }

    fn expired_at(&self, now: Instant) -> bool {
        exit_expired_at(self.started_at, self.duration, now)
    }

    #[cfg(test)]
    fn id(&self) -> interaction::Id {
        self.id
    }
}

impl RetiringPopup {
    fn layer_at(&self, now: Instant, frame_number: u64) -> Option<Layer> {
        if self.expired_at(now) {
            return None;
        }

        Some(Layer {
            id: self.id,
            order: self.original_order,
            bounds: self.bounds,
            scene: self.scene.clone(),
            opacity: self.opacity_at(now),
            fade: PopupFade::Exiting {
                duration: self.duration,
                started_at: self.started_at,
                from_opacity: self.from_opacity,
            },
            lifecycle: Lifecycle::RetiringPopup {
                elapsed: now.saturating_duration_since(self.started_at),
            },
            backend: Backend::NativePopup,
            popup_material_preference: self.popup_material_preference,
            popup_border: self.popup_border,
            text_caret_rect: None,
            placement: self.placement,
            context_fingerprint: self.context_fingerprint,
            accepts_input: false,
            force_group_at_full_opacity: false,
            demotion_marker: false,
            frame_number,
            lifecycle_epoch: self.started_at,
        })
    }

    fn opacity_at(&self, now: Instant) -> f32 {
        exit_opacity_at(self.started_at, self.duration, self.from_opacity, now)
    }

    fn expired_at(&self, now: Instant) -> bool {
        exit_expired_at(self.started_at, self.duration, now)
    }
}

impl Layer {
    pub(crate) fn scene(&self) -> &scene::Scene {
        &self.scene
    }

    pub(crate) fn id(&self) -> interaction::Id {
        self.id
    }

    pub(crate) fn bounds(&self) -> geometry::Rect {
        self.bounds
    }

    pub(crate) fn opacity(&self) -> f32 {
        self.opacity
    }

    pub(crate) fn fade(&self) -> PopupFade {
        self.fade
    }

    pub(crate) fn kind(&self) -> LayerKind {
        match self.lifecycle {
            Lifecycle::Live { .. } => LayerKind::Live,
            Lifecycle::Ghost { .. } => LayerKind::Ghost,
            Lifecycle::RetiringPopup { .. } => LayerKind::RetiringPopup,
        }
    }

    pub(crate) fn backend(&self) -> Backend {
        self.backend
    }

    pub(crate) fn popup_material_preference(&self) -> PopupMaterialPreference {
        self.popup_material_preference
    }

    pub(crate) fn popup_border(&self) -> scene::Color {
        self.popup_border
    }

    pub(crate) fn text_caret_rect(&self) -> Option<geometry::Rect> {
        self.text_caret_rect
    }

    pub(crate) fn placement(&self) -> Option<geometry::PlacementRequest> {
        self.placement
    }

    pub(crate) fn context_fingerprint(&self) -> Option<crate::popup::ContextFingerprint> {
        self.context_fingerprint
    }

    pub(crate) fn accepts_input(&self) -> bool {
        self.accepts_input
    }

    pub(crate) fn state(&self) -> Option<State> {
        match self.lifecycle {
            Lifecycle::Live { state, .. } => Some(state),
            Lifecycle::Ghost { .. } | Lifecycle::RetiringPopup { .. } => None,
        }
    }

    pub(crate) fn elapsed(&self) -> Duration {
        match self.lifecycle {
            Lifecycle::Live { elapsed, .. }
            | Lifecycle::Ghost { elapsed }
            | Lifecycle::RetiringPopup { elapsed } => elapsed,
        }
    }

    pub(crate) fn force_group_at_full_opacity(&self) -> bool {
        self.force_group_at_full_opacity
    }

    pub(crate) fn demotion_marker(&self) -> bool {
        self.demotion_marker
    }

    pub(crate) fn frame_number(&self) -> u64 {
        self.frame_number
    }

    pub(crate) fn lifecycle_epoch(&self) -> Instant {
        self.lifecycle_epoch
    }
}

impl Capabilities {
    pub(crate) fn in_frame_only() -> Self {
        Self::InFrameOnly
    }

    pub(crate) fn with_native_popups() -> Self {
        Self::AnimatedNativePopups
    }

    pub(crate) fn with_immediate_native_popups() -> Self {
        Self::ImmediateNativePopups
    }

    pub(crate) fn native_popups_supported(self) -> bool {
        !matches!(self, Self::InFrameOnly)
    }

    pub(crate) fn native_popup_animation_supported(self) -> bool {
        matches!(self, Self::AnimatedNativePopups)
    }
}

impl Default for Capabilities {
    fn default() -> Self {
        Self::in_frame_only()
    }
}

pub(crate) fn resolve_backend(preference: Preference, capabilities: Capabilities) -> Backend {
    match preference {
        Preference::InFrame => Backend::InFrame,
        Preference::NativePopup if capabilities.native_popups_supported() => Backend::NativePopup,
        Preference::NativePopup => Backend::InFrame,
    }
}

impl Update {
    fn new(layers: Vec<Layer>, schedule: animation::Schedule) -> Self {
        Self { layers, schedule }
    }

    pub(crate) fn into_parts(self) -> (Vec<Layer>, animation::Schedule) {
        (self.layers, self.schedule)
    }
}

impl PopupPresentation {
    pub(crate) fn new(
        parent: window::Id,
        id: interaction::Id,
        local_bounds: geometry::Rect,
        placement: Option<geometry::PlacementRequest>,
        scene: scene::Scene,
        opacity: f32,
        fade: PopupFade,
        material: PopupMaterial,
        border: scene::Color,
        lifecycle_epoch: Instant,
        paint_only: bool,
        kind: LayerKind,
        context_fingerprint: Option<crate::popup::ContextFingerprint>,
        accepts_input: bool,
    ) -> Self {
        Self {
            parent,
            id,
            local_bounds,
            placement,
            scene,
            opacity,
            fade,
            material,
            border,
            lifecycle_epoch,
            paint_only,
            kind,
            context_fingerprint,
            accepts_input,
        }
    }

    pub(crate) fn parent(&self) -> window::Id {
        self.parent
    }

    pub(crate) fn id(&self) -> interaction::Id {
        self.id
    }

    pub(crate) fn local_bounds(&self) -> geometry::Rect {
        self.local_bounds
    }

    pub(crate) fn placement(&self) -> Option<geometry::PlacementRequest> {
        self.placement
    }

    pub(crate) fn scene(&self) -> &scene::Scene {
        &self.scene
    }

    pub(crate) fn opacity(&self) -> f32 {
        self.opacity
    }

    pub(crate) fn fade(&self) -> PopupFade {
        self.fade
    }

    pub(crate) fn material(&self) -> PopupMaterial {
        self.material
    }

    pub(crate) fn border(&self) -> scene::Color {
        self.border
    }

    pub(crate) fn lifecycle_epoch(&self) -> Instant {
        self.lifecycle_epoch
    }

    pub(crate) fn paint_only(&self) -> bool {
        self.paint_only
    }

    pub(crate) fn accepts_input(&self) -> bool {
        self.accepts_input
    }

    pub(crate) fn kind(&self) -> LayerKind {
        self.kind
    }

    pub(crate) fn context_fingerprint(&self) -> Option<crate::popup::ContextFingerprint> {
        self.context_fingerprint
    }
}

impl PopupMaterial {
    pub(crate) fn dark(self) -> bool {
        match self {
            Self::NativeWindow { dark, .. } => dark,
        }
    }

    pub(crate) fn tint(self) -> scene::Color {
        match self {
            Self::NativeWindow { tint, .. } => tint,
        }
    }

    pub(crate) fn preference(self) -> PopupMaterialPreference {
        match self {
            Self::NativeWindow { preference, .. } => preference,
        }
    }
}

impl Store {
    pub(crate) fn new() -> Self {
        Self {
            windows: HashMap::new(),
            afterlife_limit: DEFAULT_AFTERLIFE_LIMIT,
        }
    }

    pub(crate) fn clear(&mut self) {
        self.windows.clear();
    }

    pub(crate) fn update_window(
        &mut self,
        window: window::Id,
        drafts: Vec<Draft>,
        overlay: theme::Overlay,
        capabilities: Capabilities,
        now: Instant,
    ) -> Update {
        let state = self.windows.entry(window).or_default();
        state.frame_number = state.frame_number.saturating_add(1);
        let frame_number = state.frame_number;
        let previous_live = std::mem::take(&mut state.live);
        let mut previous_by_identity = previous_live
            .into_iter()
            .map(|live| (live.identity(), live))
            .collect::<HashMap<_, _>>();
        let current_identities = drafts.iter().map(Draft::identity).collect::<HashSet<_>>();

        state.ghosts.retain(|ghost| !ghost.expired_at(now));
        state.retiring_popups.retain(|popup| {
            !popup.expired_at(now)
                && !current_identities.contains(&Identity {
                    id: popup.id,
                    context_fingerprint: popup.context_fingerprint,
                })
        });
        for live in previous_by_identity
            .values()
            .filter(|live| !current_identities.contains(&live.identity()))
        {
            if overlay.exit_fade_ms == 0
                || (live.backend == Backend::NativePopup && !live.native_animation)
            {
                continue;
            }
            let duration = Duration::from_millis(overlay.exit_fade_ms);
            let from_opacity = live_opacity(live.appeared_at, overlay.enter_fade_ms, now).0;
            match live.backend {
                Backend::InFrame => state.ghosts.push(Ghost {
                    id: live.id,
                    original_order: live.order,
                    scene: live.scene.clone(),
                    started_at: now,
                    duration,
                    from_opacity,
                }),
                Backend::NativePopup => state.retiring_popups.push(RetiringPopup {
                    id: live.id,
                    original_order: live.order,
                    bounds: live.bounds,
                    scene: live.scene.clone(),
                    popup_material_preference: live.popup_material_preference,
                    popup_border: live.popup_border,
                    placement: live.placement,
                    context_fingerprint: live.context_fingerprint,
                    started_at: now,
                    duration,
                    from_opacity,
                }),
            }
        }
        if state.ghosts.len() > self.afterlife_limit {
            let drop_count = state.ghosts.len() - self.afterlife_limit;
            state.ghosts.drain(0..drop_count);
        }
        if state.retiring_popups.len() > self.afterlife_limit {
            let drop_count = state.retiring_popups.len() - self.afterlife_limit;
            state.retiring_popups.drain(0..drop_count);
        }

        let mut entries = Vec::with_capacity(drafts.len());
        for draft in drafts {
            let identity = draft.identity();
            let (order, appeared_at, demotion_logged) = previous_by_identity
                .remove(&identity)
                .map(|live| (live.order, live.appeared_at, live.demotion_logged))
                .unwrap_or_else(|| {
                    let order = state.next_order;
                    state.next_order = state.next_order.saturating_add(1);
                    (order, now, false)
                });
            let backend = resolve_backend(draft.preference, capabilities);
            log::debug!(
                target: "wgpu_l3::overlay::backend",
                "resolved overlay backend id={:?} preference={:?} material_preference={:?} backend={:?} native_popups={} native_animation={}",
                draft.id,
                draft.preference,
                draft.popup_material_preference,
                backend,
                capabilities.native_popups_supported(),
                capabilities.native_popup_animation_supported()
            );
            let native_animation =
                backend != Backend::NativePopup || capabilities.native_popup_animation_supported();
            let enter_fade_ms = if native_animation {
                overlay.enter_fade_ms
            } else {
                0
            };
            let (opacity, entering) = live_opacity(appeared_at, enter_fade_ms, now);
            let state_kind = if entering {
                State::Entering
            } else {
                State::Live
            };
            let demotion_marker = backend == Backend::InFrame
                && !entering
                && !demotion_logged
                && overlay.enter_fade_ms > 0;
            let live = Live {
                id: draft.id,
                order,
                bounds: draft.bounds,
                scene: draft.scene.clone(),
                backend,
                native_animation,
                popup_material_preference: draft.popup_material_preference,
                popup_border: draft.popup_border,
                placement: draft.placement,
                context_fingerprint: draft.context_fingerprint,
                appeared_at,
                demotion_logged: demotion_logged || demotion_marker,
            };
            state.live.push(live);
            entries.push(Entry {
                id: draft.id,
                order,
                bounds: draft.bounds,
                scene: draft.scene,
                backend,
                popup_material_preference: draft.popup_material_preference,
                popup_border: draft.popup_border,
                text_caret_rect: draft.text_caret_rect,
                placement: draft.placement,
                context_fingerprint: draft.context_fingerprint,
                accepts_input: draft.accepts_input,
                opacity,
                fade: if entering {
                    PopupFade::Entering {
                        duration: Duration::from_millis(enter_fade_ms),
                        started_at: appeared_at,
                    }
                } else {
                    PopupFade::Stable
                },
                state: state_kind,
                elapsed: now.saturating_duration_since(appeared_at),
                force_group_at_full_opacity: draft.force_group_at_full_opacity,
                demotion_marker,
                frame_number,
                lifecycle_epoch: appeared_at,
            });
        }

        let mut layers = state
            .ghosts
            .iter()
            .filter_map(|ghost| ghost.layer_at(now, frame_number))
            .chain(
                state
                    .retiring_popups
                    .iter()
                    .filter_map(|popup| popup.layer_at(now, frame_number)),
            )
            .chain(entries.iter().map(Entry::layer))
            .collect::<Vec<_>>();
        layers.sort_by_key(|layer| layer.order);

        let mut schedule = animation::Schedule::Idle;
        if entries
            .iter()
            .any(|entry| entry.backend == Backend::InFrame && entry.state == State::Entering)
            || state.ghosts.iter().any(|ghost| !ghost.expired_at(now))
        {
            schedule = animation::Schedule::NextFrame;
        }
        for entry in entries
            .iter()
            .filter(|entry| entry.backend == Backend::NativePopup)
        {
            if let PopupFade::Entering {
                duration,
                started_at,
            } = entry.fade
            {
                schedule = schedule.merge(animation::Schedule::At(started_at + duration));
            }
        }
        for popup in &state.retiring_popups {
            schedule = schedule.merge(animation::Schedule::At(popup.started_at + popup.duration));
        }

        Update::new(layers, schedule)
    }

    #[cfg(test)]
    fn ghost_count(&self, window: window::Id) -> usize {
        self.windows
            .get(&window)
            .map(|state| state.ghosts.len())
            .unwrap_or_default()
    }

    #[cfg(test)]
    fn ghost_ids(&self, window: window::Id) -> Vec<interaction::Id> {
        self.windows
            .get(&window)
            .map(|state| state.ghosts.iter().map(Ghost::id).collect())
            .unwrap_or_default()
    }

    #[cfg(test)]
    fn retiring_popup_count(&self, window: window::Id) -> usize {
        self.windows
            .get(&window)
            .map(|state| state.retiring_popups.len())
            .unwrap_or_default()
    }
}

impl notification::Listener<window::Departed> for Store {
    fn notify(&mut self, window: &window::Id) -> notification::Reaction {
        self.windows.remove(window);
        notification::Reaction::ignored()
    }
}

#[cfg(test)]
impl Store {
    pub(crate) fn residue_count(&self, window: window::Id) -> usize {
        usize::from(self.windows.contains_key(&window))
    }
}

impl Default for Store {
    fn default() -> Self {
        Self::new()
    }
}

fn live_opacity(appeared_at: Instant, duration_ms: u64, now: Instant) -> (f32, bool) {
    if duration_ms == 0 {
        return (1.0, false);
    }

    let duration = Duration::from_millis(duration_ms);
    let progress =
        now.saturating_duration_since(appeared_at).as_secs_f32() / duration.as_secs_f32();
    let entering = progress < 1.0;
    let mut opacity = animation::Easing::EaseOutCubic.sample(progress.clamp(0.0, 1.0));
    if entering {
        opacity = opacity.min(f32::from_bits(1.0_f32.to_bits() - 1));
    }

    (opacity, entering)
}

fn exit_opacity_at(
    started_at: Instant,
    duration: Duration,
    from_opacity: f32,
    now: Instant,
) -> f32 {
    if duration.is_zero() {
        return 0.0;
    }

    let progress = now.saturating_duration_since(started_at).as_secs_f32() / duration.as_secs_f32();
    let eased = animation::Easing::EaseOutCubic.sample(progress.clamp(0.0, 1.0));
    from_opacity * (1.0 - eased)
}

fn exit_expired_at(started_at: Instant, duration: Duration, now: Instant) -> bool {
    now.saturating_duration_since(started_at) >= duration
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::geometry;

    fn overlay_theme(enter: u64, exit: u64) -> theme::Overlay {
        theme::Overlay {
            enter_fade_ms: enter,
            exit_fade_ms: exit,
        }
    }

    fn draft(id: &'static str) -> Draft {
        Draft::new(
            interaction::Id::new(id),
            geometry::Rect::new(10, 20, 100, 40),
            scene::Scene::new(geometry::Size::new(100, 40)),
        )
    }

    fn update(
        store: &mut Store,
        window: window::Id,
        drafts: Vec<Draft>,
        overlay: theme::Overlay,
        now: Instant,
    ) -> Update {
        store.update_window(window, drafts, overlay, Capabilities::default(), now)
    }

    fn force_group_draft(id: &'static str) -> Draft {
        draft(id).force_group_at_full_opacity(true)
    }

    fn popup_draft(id: &'static str) -> Draft {
        draft(id).prefer(Preference::NativePopup)
    }

    fn context_popup_draft(owner: crate::composition::tree::NodeId) -> Draft {
        popup_draft("context_menu")
            .context_fingerprint(Some(crate::popup::ContextFingerprint::from_owner(owner)))
    }

    fn assert_close(actual: f32, expected: f32) {
        assert!(
            (actual - expected).abs() < 0.0001,
            "expected {actual} to be close to {expected}"
        );
    }

    #[test]
    fn native_popup_preference_falls_back_when_unsupported() {
        let mut store = Store::new();
        let window = window::Id::new(21);
        let update = store.update_window(
            window,
            vec![popup_draft("menu")],
            overlay_theme(0, 0),
            Capabilities::in_frame_only(),
            Instant::now(),
        );

        assert_eq!(update.layers[0].backend(), Backend::InFrame);
    }

    #[test]
    fn native_popup_preference_uses_native_when_supported() {
        let mut store = Store::new();
        let window = window::Id::new(22);
        let update = store.update_window(
            window,
            vec![popup_draft("menu")],
            overlay_theme(0, 0),
            Capabilities::with_native_popups(),
            Instant::now(),
        );

        assert_eq!(update.layers[0].backend(), Backend::NativePopup);
    }

    #[test]
    fn noninteractive_panel_policy_survives_native_backend_selection() {
        let mut store = Store::new();
        let window = window::Id::new(26);
        let update = store.update_window(
            window,
            vec![popup_draft("feedback").accepts_input(false)],
            overlay_theme(0, 0),
            Capabilities::with_native_popups(),
            Instant::now(),
        );

        assert_eq!(update.layers[0].backend(), Backend::NativePopup);
        assert!(!update.layers[0].accepts_input());
    }

    #[test]
    fn native_popup_preference_uses_native_for_glass_panels_when_supported() {
        let mut store = Store::new();
        let window = window::Id::new(24);
        let update = store.update_window(
            window,
            vec![popup_draft("command_palette")],
            overlay_theme(0, 0),
            Capabilities::with_native_popups(),
            Instant::now(),
        );

        assert_eq!(update.layers[0].backend(), Backend::NativePopup);
    }

    #[test]
    fn popup_material_preference_reaches_native_layer() {
        let mut store = Store::new();
        let window = window::Id::new(25);
        let update = store.update_window(
            window,
            vec![
                popup_draft("foreground_clarity")
                    .popup_material_preference(PopupMaterialPreference::NoAccent),
            ],
            overlay_theme(0, 0),
            Capabilities::with_native_popups(),
            Instant::now(),
        );

        assert_eq!(
            update.layers[0].popup_material_preference(),
            PopupMaterialPreference::NoAccent
        );
    }

    #[test]
    fn in_frame_preference_stays_in_frame_when_native_is_supported() {
        let mut store = Store::new();
        let window = window::Id::new(23);
        let update = store.update_window(
            window,
            vec![draft("command_palette")],
            overlay_theme(0, 0),
            Capabilities::with_native_popups(),
            Instant::now(),
        );

        assert_eq!(update.layers[0].backend(), Backend::InFrame);
    }

    #[test]
    fn live_entry_fades_in_and_settles() {
        let mut store = Store::new();
        let window = window::Id::new(1);
        let now = Instant::now();

        let first = update(
            &mut store,
            window,
            vec![draft("menu")],
            overlay_theme(90, 120),
            now,
        );
        assert_eq!(first.layers.len(), 1);
        assert_eq!(first.layers[0].id(), interaction::Id::new("menu"));
        assert_eq!(first.layers[0].kind(), LayerKind::Live);
        assert_eq!(first.layers[0].opacity, 0.0);
        assert_eq!(first.schedule, animation::Schedule::NextFrame);

        let settled = update(
            &mut store,
            window,
            vec![draft("menu")],
            overlay_theme(90, 120),
            now + Duration::from_millis(91),
        );
        assert_eq!(settled.layers[0].opacity, 1.0);
        assert_eq!(settled.schedule, animation::Schedule::Idle);
    }

    #[test]
    fn native_popup_entry_projects_one_compositor_fade_and_one_completion_deadline() {
        let mut store = Store::new();
        let window = window::Id::new(26);
        let now = Instant::now();
        let theme = overlay_theme(100, 120);

        let first = store.update_window(
            window,
            vec![popup_draft("menu")],
            theme,
            Capabilities::with_native_popups(),
            now,
        );

        assert_eq!(first.layers.len(), 1);
        assert_eq!(first.layers[0].backend(), Backend::NativePopup);
        assert_eq!(first.layers[0].opacity, 0.0);
        assert_eq!(first.layers[0].state(), Some(State::Entering));
        assert_eq!(
            first.layers[0].fade(),
            PopupFade::Entering {
                duration: Duration::from_millis(100),
                started_at: now
            }
        );
        assert_eq!(
            first.schedule,
            animation::Schedule::At(now + Duration::from_millis(100))
        );

        let middle = store.update_window(
            window,
            vec![popup_draft("menu")],
            theme,
            Capabilities::with_native_popups(),
            now + Duration::from_millis(50),
        );
        assert!(middle.layers[0].opacity > 0.0 && middle.layers[0].opacity < 1.0);

        let settled = store.update_window(
            window,
            vec![popup_draft("menu")],
            theme,
            Capabilities::with_native_popups(),
            now + Duration::from_millis(100),
        );
        assert_eq!(settled.layers[0].opacity, 1.0);
        assert_eq!(settled.layers[0].state(), Some(State::Live));
        assert_eq!(settled.schedule, animation::Schedule::Idle);
    }

    #[test]
    fn immediate_native_popup_skips_both_pseudo_fades_and_afterlife() {
        let mut store = Store::new();
        let window = window::Id::new(27);
        let now = Instant::now();
        let theme = overlay_theme(90, 120);
        let capabilities = Capabilities::with_immediate_native_popups();

        let opened =
            store.update_window(window, vec![popup_draft("menu")], theme, capabilities, now);
        assert_eq!(opened.layers.len(), 1);
        assert_eq!(opened.layers[0].opacity(), 1.0);
        assert_eq!(opened.layers[0].fade(), PopupFade::Stable);
        assert_eq!(opened.schedule, animation::Schedule::Idle);

        let closed = store.update_window(
            window,
            Vec::new(),
            theme,
            capabilities,
            now + Duration::from_millis(1),
        );
        assert!(closed.layers.is_empty());
        assert_eq!(store.retiring_popup_count(window), 0);
        assert_eq!(closed.schedule, animation::Schedule::Idle);
    }

    #[test]
    fn long_live_entry_fade_samples_tail_without_early_completion() {
        let mut store = Store::new();
        let window = window::Id::new(11);
        let now = Instant::now();
        let theme = overlay_theme(5_000, 120);

        update(&mut store, window, vec![draft("menu")], theme, now);
        let late = update(
            &mut store,
            window,
            vec![draft("menu")],
            theme,
            now + Duration::from_millis(4_000),
        );
        let tail = update(
            &mut store,
            window,
            vec![draft("menu")],
            theme,
            now + Duration::from_millis(4_999),
        );
        let settled = update(
            &mut store,
            window,
            vec![draft("menu")],
            theme,
            now + Duration::from_millis(5_000),
        );

        assert_close(
            late.layers[0].opacity,
            animation::Easing::EaseOutCubic.sample(0.8),
        );
        assert_eq!(late.layers[0].state(), Some(State::Entering));
        assert_eq!(late.schedule, animation::Schedule::NextFrame);
        assert!(tail.layers[0].opacity < 1.0);
        assert_eq!(tail.layers[0].state(), Some(State::Entering));
        assert_eq!(tail.schedule, animation::Schedule::NextFrame);
        assert_eq!(settled.layers[0].opacity, 1.0);
        assert_eq!(settled.layers[0].state(), Some(State::Live));
        assert_eq!(settled.schedule, animation::Schedule::Idle);
        assert!(settled.layers[0].demotion_marker);

        let later = update(
            &mut store,
            window,
            vec![draft("menu")],
            theme,
            now + Duration::from_millis(5_001),
        );
        assert!(!later.layers[0].demotion_marker);
    }

    #[test]
    fn force_group_flag_survives_to_live_layer_at_full_opacity() {
        let mut store = Store::new();
        let window = window::Id::new(12);
        let now = Instant::now();

        let update = update(
            &mut store,
            window,
            vec![force_group_draft("comparison")],
            overlay_theme(0, 120),
            now,
        );

        assert_eq!(update.layers.len(), 1);
        assert_eq!(update.layers[0].opacity, 1.0);
        assert_eq!(update.layers[0].kind(), LayerKind::Live);
        assert!(update.layers[0].force_group_at_full_opacity);
    }

    #[test]
    fn removed_entry_creates_fading_ghost() {
        let mut store = Store::new();
        let window = window::Id::new(2);
        let now = Instant::now();

        update(
            &mut store,
            window,
            vec![draft("palette")],
            overlay_theme(0, 120),
            now,
        );
        let fading = update(
            &mut store,
            window,
            Vec::new(),
            overlay_theme(0, 120),
            now + Duration::from_millis(10),
        );

        assert_eq!(store.ghost_count(window), 1);
        assert_eq!(fading.layers.len(), 1);
        assert_eq!(fading.layers[0].id(), interaction::Id::new("palette"));
        assert_eq!(fading.layers[0].kind(), LayerKind::Ghost);
        assert_eq!(fading.layers[0].opacity, 1.0);
        assert_eq!(fading.schedule, animation::Schedule::NextFrame);

        let mid_fade = update(
            &mut store,
            window,
            Vec::new(),
            overlay_theme(0, 120),
            now + Duration::from_millis(70),
        );
        assert!(mid_fade.layers[0].opacity > 0.0 && mid_fade.layers[0].opacity < 1.0);

        let expired = update(
            &mut store,
            window,
            Vec::new(),
            overlay_theme(0, 120),
            now + Duration::from_millis(131),
        );
        assert_eq!(store.ghost_count(window), 0);
        assert!(expired.layers.is_empty());
        assert_eq!(expired.schedule, animation::Schedule::Idle);
    }

    #[test]
    fn removed_native_popup_retires_on_its_native_surface_without_a_parent_ghost() {
        let mut store = Store::new();
        let window = window::Id::new(25);
        let now = Instant::now();
        let border = scene::Color::rgb(10, 20, 30);

        store.update_window(
            window,
            vec![
                popup_draft("menu")
                    .popup_material_preference(PopupMaterialPreference::NoAccent)
                    .popup_border(border),
            ],
            overlay_theme(0, 120),
            Capabilities::with_native_popups(),
            now,
        );
        let removed = store.update_window(
            window,
            Vec::new(),
            overlay_theme(0, 120),
            Capabilities::with_native_popups(),
            now + Duration::from_millis(10),
        );

        assert_eq!(store.ghost_count(window), 0);
        assert_eq!(store.retiring_popup_count(window), 1);
        assert_eq!(removed.layers.len(), 1);
        assert_eq!(removed.layers[0].kind(), LayerKind::RetiringPopup);
        assert_eq!(removed.layers[0].backend(), Backend::NativePopup);
        assert_eq!(
            removed.layers[0].popup_material_preference(),
            PopupMaterialPreference::NoAccent
        );
        assert_eq!(removed.layers[0].popup_border(), border);
        assert_eq!(removed.layers[0].opacity(), 1.0);
        assert_eq!(
            removed.layers[0].fade(),
            PopupFade::Exiting {
                duration: Duration::from_millis(120),
                started_at: now + Duration::from_millis(10),
                from_opacity: 1.0
            }
        );
        assert_eq!(
            removed.schedule,
            animation::Schedule::At(now + Duration::from_millis(130))
        );

        let middle = store.update_window(
            window,
            Vec::new(),
            overlay_theme(0, 120),
            Capabilities::with_native_popups(),
            now + Duration::from_millis(70),
        );
        assert!(middle.layers[0].opacity() > 0.0 && middle.layers[0].opacity() < 1.0);

        let expired = store.update_window(
            window,
            Vec::new(),
            overlay_theme(0, 120),
            Capabilities::with_native_popups(),
            now + Duration::from_millis(131),
        );
        assert_eq!(store.retiring_popup_count(window), 0);
        assert!(expired.layers.is_empty());
        assert_eq!(expired.schedule, animation::Schedule::Idle);
    }

    #[test]
    fn contextual_retarget_reuses_the_authored_menu_lifecycle() {
        let mut store = Store::new();
        let window = window::Id::new(26);
        let now = Instant::now();
        let theme = overlay_theme(100, 120);
        let mut next = 1;
        let first_owner = crate::composition::tree::NodeId::layout(&mut next);
        let second_owner = crate::composition::tree::NodeId::layout(&mut next);

        store.update_window(
            window,
            vec![context_popup_draft(first_owner)],
            theme,
            Capabilities::with_native_popups(),
            now,
        );
        let retargeted = store.update_window(
            window,
            vec![context_popup_draft(second_owner)],
            theme,
            Capabilities::with_native_popups(),
            now + Duration::from_millis(40),
        );

        assert_eq!(store.retiring_popup_count(window), 1);
        assert_eq!(retargeted.layers.len(), 2);
        assert_eq!(retargeted.layers[0].kind(), LayerKind::RetiringPopup);
        assert_eq!(retargeted.layers[1].kind(), LayerKind::Live);
        assert_eq!(retargeted.layers[1].state(), Some(State::Entering));
        assert_eq!(
            retargeted.layers[0].context_fingerprint(),
            Some(crate::popup::ContextFingerprint::from_owner(first_owner))
        );
        assert_eq!(
            retargeted.layers[1].context_fingerprint(),
            Some(crate::popup::ContextFingerprint::from_owner(second_owner))
        );
    }

    #[test]
    fn reopened_native_popup_replaces_its_retiring_surface_entry() {
        let mut store = Store::new();
        let window = window::Id::new(27);
        let now = Instant::now();
        let theme = overlay_theme(100, 120);

        store.update_window(
            window,
            vec![popup_draft("menu")],
            theme,
            Capabilities::with_native_popups(),
            now,
        );
        store.update_window(
            window,
            Vec::new(),
            theme,
            Capabilities::with_native_popups(),
            now + Duration::from_millis(50),
        );
        let reopened = store.update_window(
            window,
            vec![popup_draft("menu")],
            theme,
            Capabilities::with_native_popups(),
            now + Duration::from_millis(60),
        );

        assert_eq!(store.retiring_popup_count(window), 0);
        assert_eq!(reopened.layers.len(), 1);
        assert_eq!(reopened.layers[0].kind(), LayerKind::Live);
        assert_eq!(reopened.layers[0].backend(), Backend::NativePopup);
    }

    #[test]
    fn zero_exit_duration_allocates_no_ghost() {
        let mut store = Store::new();
        let window = window::Id::new(3);
        let now = Instant::now();

        update(
            &mut store,
            window,
            vec![draft("menu")],
            overlay_theme(0, 0),
            now,
        );
        let removed = update(
            &mut store,
            window,
            Vec::new(),
            overlay_theme(0, 0),
            now + Duration::from_millis(1),
        );

        assert_eq!(store.ghost_count(window), 0);
        assert!(removed.layers.is_empty());
        assert_eq!(removed.schedule, animation::Schedule::Idle);
    }

    #[test]
    fn reopened_entry_paints_above_its_ghost() {
        let mut store = Store::new();
        let window = window::Id::new(4);
        let now = Instant::now();

        update(
            &mut store,
            window,
            vec![draft("menu")],
            overlay_theme(0, 120),
            now,
        );
        update(
            &mut store,
            window,
            Vec::new(),
            overlay_theme(0, 120),
            now + Duration::from_millis(1),
        );
        let reopened = update(
            &mut store,
            window,
            vec![draft("menu")],
            overlay_theme(0, 120),
            now + Duration::from_millis(2),
        );

        assert_eq!(reopened.layers.len(), 2);
        assert_eq!(reopened.layers[0].id(), interaction::Id::new("menu"));
        assert_eq!(reopened.layers[1].id(), interaction::Id::new("menu"));
        assert_eq!(reopened.layers[0].kind(), LayerKind::Ghost);
        assert_eq!(reopened.layers[1].kind(), LayerKind::Live);
        assert!(reopened.layers[0].order < reopened.layers[1].order);
    }

    #[test]
    fn ghosts_are_retention_capped_oldest_first() {
        let mut store = Store::new();
        let window = window::Id::new(5);
        let now = Instant::now();

        for index in 0..10 {
            let id = match index {
                0 => "entry.0",
                1 => "entry.1",
                2 => "entry.2",
                3 => "entry.3",
                4 => "entry.4",
                5 => "entry.5",
                6 => "entry.6",
                7 => "entry.7",
                8 => "entry.8",
                _ => "entry.9",
            };
            let at = now + Duration::from_millis(index);
            update(
                &mut store,
                window,
                vec![draft(id)],
                overlay_theme(0, 1_000),
                at,
            );
            update(
                &mut store,
                window,
                Vec::new(),
                overlay_theme(0, 1_000),
                at + Duration::from_millis(1),
            );
        }

        assert_eq!(store.ghost_count(window), DEFAULT_AFTERLIFE_LIMIT);
        assert_eq!(
            store.ghost_ids(window),
            vec![
                interaction::Id::new("entry.2"),
                interaction::Id::new("entry.3"),
                interaction::Id::new("entry.4"),
                interaction::Id::new("entry.5"),
                interaction::Id::new("entry.6"),
                interaction::Id::new("entry.7"),
                interaction::Id::new("entry.8"),
                interaction::Id::new("entry.9"),
            ]
        );
    }

    #[test]
    fn overlay_lifecycle_laws_hold_through_10000_deterministic_updates() {
        const IDS: [&str; 8] = [
            "stress.0", "stress.1", "stress.2", "stress.3", "stress.4", "stress.5", "stress.6",
            "stress.7",
        ];

        let mut store = Store::new();
        let window = window::Id::new(90);
        let epoch = Instant::now();
        let theme = overlay_theme(7, 11);
        let mut random = 0x3c6e_f372_fe94_f82b_u64;

        for operation in 0..10_000_u64 {
            random = random.wrapping_mul(6364136223846793005).wrapping_add(1);
            let capabilities = if random & 1 == 0 {
                Capabilities::in_frame_only()
            } else {
                Capabilities::with_native_popups()
            };
            let mut selected = Vec::new();
            let count = ((random >> 8) % 4) as usize;
            for slot in 0..count {
                random = random.wrapping_mul(6364136223846793005).wrapping_add(1);
                let id = IDS[((random as usize) + slot) % IDS.len()];
                if selected.contains(&id) {
                    continue;
                }
                selected.push(id);
            }
            let drafts = selected
                .into_iter()
                .enumerate()
                .map(|(index, id)| {
                    if (random >> (index + 16)) & 1 == 0 {
                        draft(id)
                    } else {
                        popup_draft(id)
                    }
                })
                .collect::<Vec<_>>();
            let update = store.update_window(
                window,
                drafts,
                theme,
                capabilities,
                epoch + Duration::from_millis(operation),
            );

            assert!(
                update
                    .layers
                    .windows(2)
                    .all(|pair| pair[0].order <= pair[1].order),
                "layer order operation {operation}"
            );
            for layer in &update.layers {
                assert!(
                    layer.opacity().is_finite() && layer.opacity() >= 0.0 && layer.opacity() <= 1.0,
                    "opacity operation {operation}: {}",
                    layer.opacity()
                );
                match layer.kind() {
                    LayerKind::Live => {
                        if layer.state() == Some(State::Live) {
                            assert_eq!(layer.opacity(), 1.0);
                        }
                    }
                    LayerKind::Ghost => assert_eq!(layer.backend(), Backend::InFrame),
                    LayerKind::RetiringPopup => {
                        assert_eq!(layer.backend(), Backend::NativePopup)
                    }
                }
            }
            assert!(store.ghost_count(window) <= DEFAULT_AFTERLIFE_LIMIT);
            assert!(store.retiring_popup_count(window) <= DEFAULT_AFTERLIFE_LIMIT);
            let needs_frame = update.layers.iter().any(|layer| {
                layer.kind() != LayerKind::Live || layer.state() == Some(State::Entering)
            });
            assert_eq!(
                update.schedule,
                if needs_frame {
                    animation::Schedule::NextFrame
                } else {
                    animation::Schedule::Idle
                },
                "schedule operation {operation}"
            );

            if operation % 997 == 996 {
                <Store as notification::Listener<window::Departed>>::notify(&mut store, &window);
                assert_eq!(store.residue_count(window), 0);
            }
        }
    }
}
