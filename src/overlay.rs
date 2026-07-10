use std::{
    collections::{HashMap, HashSet},
    time::{Duration, Instant},
};

use crate::{animation, geometry, interaction, notification, scene, theme, window};

const DEFAULT_GHOST_LIMIT: usize = 8;

#[derive(Debug, Clone)]
pub(crate) struct Draft {
    id: interaction::Id,
    bounds: geometry::Rect,
    scene: scene::Scene,
    preference: Preference,
    popup_material_preference: PopupMaterialPreference,
    popup_border: scene::Color,
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
    opacity: f32,
    state: State,
    elapsed: Duration,
    force_group_at_full_opacity: bool,
    demotion_marker: bool,
    frame_number: u64,
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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum State {
    Entering,
    Live,
}

#[derive(Debug, Clone)]
pub(crate) struct Layer {
    #[allow(dead_code)]
    id: interaction::Id,
    order: u64,
    bounds: geometry::Rect,
    scene: scene::Scene,
    opacity: f32,
    kind: LayerKind,
    backend: Backend,
    popup_material_preference: PopupMaterialPreference,
    popup_border: scene::Color,
    state: Option<State>,
    elapsed: Option<Duration>,
    force_group_at_full_opacity: bool,
    demotion_marker: bool,
    frame_number: u64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum LayerKind {
    Live,
    Ghost,
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
pub(crate) struct Capabilities {
    native_popups: bool,
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
    bounds: geometry::Rect,
    scene: scene::Scene,
    opaque_fallback_scene: scene::Scene,
    material: PopupMaterial,
    border: scene::Color,
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
    ghost_limit: usize,
}

#[derive(Debug, Default)]
struct WindowState {
    live: Vec<Live>,
    ghosts: Vec<Ghost>,
    next_order: u64,
    frame_number: u64,
}

#[derive(Debug, Clone)]
struct Live {
    id: interaction::Id,
    order: u64,
    scene: scene::Scene,
    backend: Backend,
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

    pub(crate) fn id(&self) -> interaction::Id {
        self.id
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
            kind: LayerKind::Live,
            backend: self.backend,
            popup_material_preference: self.popup_material_preference,
            popup_border: self.popup_border,
            state: Some(self.state),
            elapsed: Some(self.elapsed),
            force_group_at_full_opacity: self.force_group_at_full_opacity,
            demotion_marker: self.demotion_marker,
            frame_number: self.frame_number,
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
            kind: LayerKind::Ghost,
            backend: Backend::InFrame,
            popup_material_preference: PopupMaterialPreference::System,
            popup_border: scene::Color::rgba(0, 0, 0, 0),
            state: None,
            elapsed: Some(now.saturating_duration_since(self.started_at)),
            force_group_at_full_opacity: false,
            demotion_marker: false,
            frame_number,
        })
    }

    fn opacity_at(&self, now: Instant) -> f32 {
        if self.duration.is_zero() {
            return 0.0;
        }

        let progress = now.saturating_duration_since(self.started_at).as_secs_f32()
            / self.duration.as_secs_f32();
        let eased = animation::Easing::EaseOutCubic.sample(progress.clamp(0.0, 1.0));

        self.from_opacity * (1.0 - eased)
    }

    fn expired_at(&self, now: Instant) -> bool {
        now.saturating_duration_since(self.started_at) >= self.duration
    }

    #[cfg(test)]
    fn id(&self) -> interaction::Id {
        self.id
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

    pub(crate) fn kind(&self) -> LayerKind {
        self.kind
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

    pub(crate) fn state(&self) -> Option<State> {
        self.state
    }

    pub(crate) fn elapsed(&self) -> Option<Duration> {
        self.elapsed
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
}

impl Capabilities {
    pub(crate) fn in_frame_only() -> Self {
        Self {
            native_popups: false,
        }
    }

    pub(crate) fn with_native_popups() -> Self {
        Self {
            native_popups: true,
        }
    }

    pub(crate) fn native_popups_supported(self) -> bool {
        self.native_popups
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

    pub(crate) fn layers(&self) -> &[Layer] {
        &self.layers
    }

    pub(crate) fn schedule(&self) -> animation::Schedule {
        self.schedule
    }
}

impl PopupPresentation {
    pub(crate) fn new(
        parent: window::Id,
        id: interaction::Id,
        bounds: geometry::Rect,
        scene: scene::Scene,
        opaque_fallback_scene: scene::Scene,
        material: PopupMaterial,
        border: scene::Color,
    ) -> Self {
        Self {
            parent,
            id,
            bounds,
            scene,
            opaque_fallback_scene,
            material,
            border,
        }
    }

    pub(crate) fn parent(&self) -> window::Id {
        self.parent
    }

    pub(crate) fn id(&self) -> interaction::Id {
        self.id
    }

    pub(crate) fn bounds(&self) -> geometry::Rect {
        self.bounds
    }

    pub(crate) fn scene(&self) -> &scene::Scene {
        &self.scene
    }

    pub(crate) fn opaque_fallback_scene(&self) -> &scene::Scene {
        &self.opaque_fallback_scene
    }

    pub(crate) fn material(&self) -> PopupMaterial {
        self.material
    }

    pub(crate) fn border(&self) -> scene::Color {
        self.border
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
            ghost_limit: DEFAULT_GHOST_LIMIT,
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
        let mut previous_by_id = previous_live
            .into_iter()
            .map(|live| (live.id, live))
            .collect::<HashMap<_, _>>();
        let current_ids = drafts.iter().map(Draft::id).collect::<HashSet<_>>();

        state.ghosts.retain(|ghost| !ghost.expired_at(now));
        for live in previous_by_id
            .values()
            .filter(|live| live.backend == Backend::InFrame && !current_ids.contains(&live.id))
        {
            if overlay.exit_fade_ms > 0 {
                state.ghosts.push(Ghost {
                    id: live.id,
                    original_order: live.order,
                    scene: live.scene.clone(),
                    started_at: now,
                    duration: Duration::from_millis(overlay.exit_fade_ms),
                    from_opacity: live_opacity(live.appeared_at, overlay.enter_fade_ms, now).0,
                });
            }
        }
        if state.ghosts.len() > self.ghost_limit {
            let drop_count = state.ghosts.len() - self.ghost_limit;
            state.ghosts.drain(0..drop_count);
        }

        let mut entries = Vec::with_capacity(drafts.len());
        for draft in drafts {
            let (order, appeared_at, demotion_logged) = previous_by_id
                .remove(&draft.id)
                .map(|live| (live.order, live.appeared_at, live.demotion_logged))
                .unwrap_or_else(|| {
                    let order = state.next_order;
                    state.next_order = state.next_order.saturating_add(1);
                    (order, now, false)
                });
            let backend = resolve_backend(draft.preference, capabilities);
            log::debug!(
                target: "wgpu_l3::overlay::backend",
                "resolved overlay backend id={:?} preference={:?} material_preference={:?} backend={:?} native_popups={}",
                draft.id,
                draft.preference,
                draft.popup_material_preference,
                backend,
                capabilities.native_popups_supported()
            );
            let (opacity, entering) = if backend == Backend::NativePopup {
                (1.0, false)
            } else {
                live_opacity(appeared_at, overlay.enter_fade_ms, now)
            };
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
                scene: draft.scene.clone(),
                backend,
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
                opacity,
                state: state_kind,
                elapsed: now.saturating_duration_since(appeared_at),
                force_group_at_full_opacity: draft.force_group_at_full_opacity,
                demotion_marker,
                frame_number,
            });
        }

        let mut layers = state
            .ghosts
            .iter()
            .filter_map(|ghost| ghost.layer_at(now, frame_number))
            .chain(entries.iter().map(Entry::layer))
            .collect::<Vec<_>>();
        layers.sort_by_key(|layer| layer.order);

        let schedule = if entries.iter().any(|entry| entry.state == State::Entering)
            || state.ghosts.iter().any(|ghost| !ghost.expired_at(now))
        {
            animation::Schedule::NextFrame
        } else {
            animation::Schedule::Idle
        };

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
        assert_eq!(first.layers[0].kind, LayerKind::Live);
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
    fn native_popup_entry_skips_enter_fade_until_premultiplied_audit() {
        let mut store = Store::new();
        let window = window::Id::new(26);
        let now = Instant::now();

        let first = store.update_window(
            window,
            vec![popup_draft("menu")],
            overlay_theme(5_000, 120),
            Capabilities::with_native_popups(),
            now,
        );

        assert_eq!(first.layers.len(), 1);
        assert_eq!(first.layers[0].backend(), Backend::NativePopup);
        assert_eq!(first.layers[0].opacity, 1.0);
        assert_eq!(first.layers[0].state, Some(State::Live));
        assert_eq!(first.schedule, animation::Schedule::Idle);
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
        assert_eq!(late.layers[0].state, Some(State::Entering));
        assert_eq!(late.schedule, animation::Schedule::NextFrame);
        assert!(tail.layers[0].opacity < 1.0);
        assert_eq!(tail.layers[0].state, Some(State::Entering));
        assert_eq!(tail.schedule, animation::Schedule::NextFrame);
        assert_eq!(settled.layers[0].opacity, 1.0);
        assert_eq!(settled.layers[0].state, Some(State::Live));
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
        assert_eq!(update.layers[0].kind, LayerKind::Live);
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
        assert_eq!(fading.layers[0].kind, LayerKind::Ghost);
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
    fn removed_native_popup_entry_allocates_no_ghost() {
        let mut store = Store::new();
        let window = window::Id::new(25);
        let now = Instant::now();

        store.update_window(
            window,
            vec![popup_draft("menu")],
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
        assert!(removed.layers.is_empty());
        assert_eq!(removed.schedule, animation::Schedule::Idle);
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
        assert_eq!(reopened.layers[0].kind, LayerKind::Ghost);
        assert_eq!(reopened.layers[1].kind, LayerKind::Live);
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

        assert_eq!(store.ghost_count(window), DEFAULT_GHOST_LIMIT);
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
}
