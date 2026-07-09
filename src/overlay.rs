use std::{
    collections::{HashMap, HashSet},
    time::{Duration, Instant},
};

use crate::{animation, interaction, scene, theme, window};

const DEFAULT_GHOST_LIMIT: usize = 8;

#[derive(Debug, Clone)]
pub(crate) struct Draft {
    id: interaction::Id,
    scene: scene::Scene,
    force_group_at_full_opacity: bool,
}

#[derive(Debug, Clone)]
struct Entry {
    #[allow(dead_code)]
    id: interaction::Id,
    order: u64,
    scene: scene::Scene,
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
    scene: scene::Scene,
    opacity: f32,
    kind: LayerKind,
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

#[derive(Debug, Clone)]
pub(crate) struct Update {
    layers: Vec<Layer>,
    schedule: animation::Schedule,
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
    appeared_at: Instant,
    demotion_logged: bool,
}

impl Draft {
    pub(crate) fn new(id: interaction::Id, scene: scene::Scene) -> Self {
        Self {
            id,
            scene,
            force_group_at_full_opacity: false,
        }
    }

    pub(crate) fn force_group_at_full_opacity(mut self, force: bool) -> Self {
        self.force_group_at_full_opacity = force;
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
            scene: self.scene.clone(),
            opacity: self.opacity,
            kind: LayerKind::Live,
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
            scene: self.scene.clone(),
            opacity: self.opacity_at(now),
            kind: LayerKind::Ghost,
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
    pub(crate) fn id(&self) -> interaction::Id {
        self.id
    }
}

impl Layer {
    #[cfg(test)]
    pub(crate) fn id(&self) -> interaction::Id {
        self.id
    }

    pub(crate) fn scene(&self) -> &scene::Scene {
        &self.scene
    }

    pub(crate) fn opacity(&self) -> f32 {
        self.opacity
    }

    pub(crate) fn kind(&self) -> LayerKind {
        self.kind
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

impl Store {
    pub(crate) fn new() -> Self {
        Self {
            windows: HashMap::new(),
            ghost_limit: DEFAULT_GHOST_LIMIT,
        }
    }

    pub(crate) fn update_window(
        &mut self,
        window: window::Id,
        drafts: Vec<Draft>,
        overlay: theme::Overlay,
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
            .filter(|live| !current_ids.contains(&live.id))
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
            let (opacity, entering) = live_opacity(appeared_at, overlay.enter_fade_ms, now);
            let state_kind = if entering {
                State::Entering
            } else {
                State::Live
            };
            let demotion_marker = !entering && !demotion_logged && overlay.enter_fade_ms > 0;
            let live = Live {
                id: draft.id,
                order,
                scene: draft.scene.clone(),
                appeared_at,
                demotion_logged: demotion_logged || demotion_marker,
            };
            state.live.push(live);
            entries.push(Entry {
                id: draft.id,
                order,
                scene: draft.scene,
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
    pub(crate) fn ghost_count(&self, window: window::Id) -> usize {
        self.windows
            .get(&window)
            .map(|state| state.ghosts.len())
            .unwrap_or_default()
    }

    #[cfg(test)]
    pub(crate) fn ghost_ids(&self, window: window::Id) -> Vec<interaction::Id> {
        self.windows
            .get(&window)
            .map(|state| state.ghosts.iter().map(Ghost::id).collect())
            .unwrap_or_default()
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
            scene::Scene::new(geometry::Size::new(100, 40)),
        )
    }

    fn force_group_draft(id: &'static str) -> Draft {
        draft(id).force_group_at_full_opacity(true)
    }

    fn assert_close(actual: f32, expected: f32) {
        assert!(
            (actual - expected).abs() < 0.0001,
            "expected {actual} to be close to {expected}"
        );
    }

    #[test]
    fn live_entry_fades_in_and_settles() {
        let mut store = Store::new();
        let window = window::Id::new(1);
        let now = Instant::now();

        let first = store.update_window(window, vec![draft("menu")], overlay_theme(90, 120), now);
        assert_eq!(first.layers.len(), 1);
        assert_eq!(first.layers[0].id(), interaction::Id::new("menu"));
        assert_eq!(first.layers[0].kind, LayerKind::Live);
        assert_eq!(first.layers[0].opacity, 0.0);
        assert_eq!(first.schedule, animation::Schedule::NextFrame);

        let settled = store.update_window(
            window,
            vec![draft("menu")],
            overlay_theme(90, 120),
            now + Duration::from_millis(91),
        );
        assert_eq!(settled.layers[0].opacity, 1.0);
        assert_eq!(settled.schedule, animation::Schedule::Idle);
    }

    #[test]
    fn long_live_entry_fade_samples_tail_without_early_completion() {
        let mut store = Store::new();
        let window = window::Id::new(11);
        let now = Instant::now();
        let theme = overlay_theme(5_000, 120);

        store.update_window(window, vec![draft("menu")], theme, now);
        let late = store.update_window(
            window,
            vec![draft("menu")],
            theme,
            now + Duration::from_millis(4_000),
        );
        let tail = store.update_window(
            window,
            vec![draft("menu")],
            theme,
            now + Duration::from_millis(4_999),
        );
        let settled = store.update_window(
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

        let later = store.update_window(
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

        let update = store.update_window(
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

        store.update_window(window, vec![draft("palette")], overlay_theme(0, 120), now);
        let fading = store.update_window(
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

        let mid_fade = store.update_window(
            window,
            Vec::new(),
            overlay_theme(0, 120),
            now + Duration::from_millis(70),
        );
        assert!(mid_fade.layers[0].opacity > 0.0 && mid_fade.layers[0].opacity < 1.0);

        let expired = store.update_window(
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
    fn zero_exit_duration_allocates_no_ghost() {
        let mut store = Store::new();
        let window = window::Id::new(3);
        let now = Instant::now();

        store.update_window(window, vec![draft("menu")], overlay_theme(0, 0), now);
        let removed = store.update_window(
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

        store.update_window(window, vec![draft("menu")], overlay_theme(0, 120), now);
        store.update_window(
            window,
            Vec::new(),
            overlay_theme(0, 120),
            now + Duration::from_millis(1),
        );
        let reopened = store.update_window(
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
            store.update_window(window, vec![draft(id)], overlay_theme(0, 1_000), at);
            store.update_window(
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
