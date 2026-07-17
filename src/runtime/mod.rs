use super::{
    clipboard::Clipboard,
    command, composition, diagnostics, geometry, interaction, keymap, layout, overlay, responder,
    scene, session,
    state::{self, Store},
    task, theme,
    timeline::{self, Timeline},
    view,
};
use crate::{animation, window};
use std::{
    collections::HashMap,
    ops::{Deref, DerefMut},
    sync::Arc,
};
mod access;
mod builder;
mod context;
mod context_menu;
mod departed;
mod dispatch;
mod fuzzy;
mod input;
mod lifecycle;
mod palette;
mod pointer;
mod presentation;
mod retention;
mod routing;
mod services;
mod snapshot;
mod tasks;
mod transaction;
mod visual;
pub(super) mod work;

pub use context::Context;
pub use retention::Retention;
pub use snapshot::{Persistence, Snapshot};

type Started<M> = Box<dyn for<'a> FnMut(&mut Context<'a, M>)>;
type Event<M, E> = Box<dyn for<'a> FnMut(&mut Context<'a, M>, E)>;
type ThemeCallback<M> = Box<dyn Fn(&M) -> theme::Theme>;
type ViewCallback<M, V> = Box<dyn Fn(&M, view::Context) -> V>;

#[derive(Clone)]
struct CachedLayout {
    size: geometry::Size,
    theme: theme::Theme,
    popup_surfaces: layout::PopupSurfaces,
    layout: layout::Layout,
}

#[derive(Clone)]
struct PresentedGeometry {
    layout: Arc<layout::Layout>,
    stack: Arc<scene::Stack>,
    spatial: scene::SpatialSnapshot,
}

impl PresentedGeometry {
    fn project_point(
        &self,
        node: composition::tree::NodeId,
        point: geometry::Point,
        clip: bool,
    ) -> Option<(geometry::Point, [i32; 2])> {
        self.spatial.project_point(node, point, clip)
    }

    fn hit_test_on_surface(
        &self,
        point: geometry::Point,
        surface: crate::popup::Surface,
    ) -> Option<layout::Hit> {
        self.layout
            .hit_test_on_surface_projected(point, surface, &|node, point| {
                self.project_point(node, point, true)
            })
    }

    fn translated_rect(
        &self,
        node: composition::tree::NodeId,
        rect: geometry::Rect,
    ) -> Option<geometry::Rect> {
        self.spatial.translated_rect(node, rect)
    }

    fn context_available_for_node(
        &self,
        node: composition::tree::NodeId,
    ) -> Option<geometry::Rect> {
        self.layout
            .context_available_for_node(node)
            .and_then(|rect| self.translated_rect(node, rect))
    }

    fn focus_node_and_rect(
        &self,
        focus: session::Focus,
    ) -> Option<(composition::tree::NodeId, geometry::Rect)> {
        let frame = self.layout.frame_for_focus(focus)?;
        Some((
            frame.node_id(),
            self.translated_rect(frame.node_id(), frame.rect())?,
        ))
    }

    fn context_node_at_surface(
        &self,
        point: geometry::Point,
        surface: crate::popup::Surface,
    ) -> Option<composition::tree::NodeId> {
        self.layout
            .context_node_at_surface_projected(point, surface, &|node, point| {
                self.project_point(node, point, true)
                    .map(|(point, _)| point)
            })
    }

    fn drag_action_for_target(
        &self,
        target: &interaction::Target,
        point: geometry::Point,
        engine: &mut layout::Engine,
    ) -> Option<(view::Role, Option<view::Action>)> {
        self.layout
            .drag_action_for_target_projected(target, point, engine, &|node, point| {
                self.project_point(node, point, false)
                    .map(|(point, _)| point)
            })
    }

    fn scroll_target_chain_at_surface(
        &self,
        point: geometry::Point,
        surface: crate::popup::Surface,
    ) -> Vec<interaction::Target> {
        self.layout
            .scroll_target_chain_at_surface_projected(point, surface, &|node, point| {
                self.project_point(node, point, true)
                    .map(|(point, _)| point)
            })
    }

    fn scroll_target_chain_for_focus(
        &self,
        focus: session::Focus,
        axis: interaction::ScrollbarAxis,
    ) -> Vec<(interaction::Target, crate::scroll::Direction)> {
        self.layout.scroll_target_chain_for_focus(focus, axis)
    }
}

type VirtualMaterializations = HashMap<crate::interaction::Id, crate::list::Materialization>;
type VirtualMeasurements = HashMap<crate::interaction::Id, crate::list::Measurements>;

const KINETIC_FRAME_INTERVAL: std::time::Duration = std::time::Duration::from_millis(4);

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
struct AnimationSchedules {
    paint: animation::Schedule,
    properties: animation::Schedule,
}

#[derive(Debug, Clone)]
struct KineticScroll {
    targets: Vec<interaction::Target>,
    source: interaction::ScrollSource,
    velocity: interaction::Delta,
    last_tick: std::time::Instant,
}

impl AnimationSchedules {
    fn combined(self) -> animation::Schedule {
        self.paint.merge(self.properties)
    }

    fn is_idle(self) -> bool {
        self.paint == animation::Schedule::Idle && self.properties == animation::Schedule::Idle
    }
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
struct ResidencySchedule {
    latest_request_generation: u64,
    latest_request_urgency: Option<scene::ResidencyUrgency>,
    candidate_requested: Option<scene::ResidencyUrgency>,
    selected: Option<ResidencyCandidate>,
    queued: Option<ResidencyCandidate>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct ResidencyCandidate {
    epoch: window::PresentationEpoch,
    request_generation: u64,
    urgency: scene::ResidencyUrgency,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct ResidencyRetirement {
    schedule_follow_up: bool,
    finished: bool,
}

impl ResidencySchedule {
    fn request_latest(&mut self, urgency: scene::ResidencyUrgency) -> bool {
        self.latest_request_generation = self.latest_request_generation.saturating_add(1);
        self.latest_request_urgency = Some(urgency);
        if let Some(requested) = self.candidate_requested.as_mut() {
            *requested = (*requested).max(urgency);
            return false;
        }
        if let Some(selected) = self.selected {
            if self.queued.is_some_and(|queued| queued.urgency >= urgency)
                || selected.urgency >= urgency
            {
                return false;
            }
        }
        self.candidate_requested = Some(urgency);
        true
    }

    fn select(&mut self, epoch: window::PresentationEpoch) -> Option<scene::ResidencyUrgency> {
        let urgency = self.candidate_requested.take()?;
        let candidate = ResidencyCandidate {
            epoch,
            request_generation: self.latest_request_generation,
            urgency,
        };
        if self.selected.is_none() {
            self.selected = Some(candidate);
        } else if self.queued.is_none() {
            debug_assert!(
                self.selected
                    .is_some_and(|selected| selected.urgency < candidate.urgency),
                "only a more urgent candidate may bypass the selected residency front"
            );
            self.queued = Some(candidate);
        } else {
            debug_assert!(
                self.queued.is_some_and(|queued| queued.urgency < urgency),
                "only a more urgent residency candidate may replace queued work"
            );
            self.queued = Some(candidate);
        }
        Some(urgency)
    }

    fn candidate_requested(self) -> bool {
        self.candidate_requested.is_some()
    }

    fn candidate_requested_urgency(self) -> Option<scene::ResidencyUrgency> {
        self.candidate_requested
    }

    fn selected_epoch(self) -> Option<window::PresentationEpoch> {
        self.selected.map(|candidate| candidate.epoch)
    }

    fn retire_selected(&mut self, epoch: window::PresentationEpoch) -> Option<ResidencyRetirement> {
        let retired = self.selected.filter(|candidate| candidate.epoch == epoch)?;
        self.selected = self.queued.take();
        let newest_selected_generation = self
            .selected
            .map_or(retired.request_generation, |candidate| {
                candidate.request_generation
            });
        let schedule_follow_up = self.latest_request_generation > newest_selected_generation
            && self.candidate_requested.is_none();
        if schedule_follow_up {
            self.candidate_requested = self.latest_request_urgency;
        }
        Some(ResidencyRetirement {
            schedule_follow_up,
            finished: self.selected.is_none() && self.candidate_requested.is_none(),
        })
    }
}

#[cfg(test)]
mod residency_schedule_tests {
    use super::{ResidencySchedule, scene::ResidencyUrgency};

    #[test]
    fn same_urgency_requests_coalesce_before_candidate_construction() {
        let first = crate::window::PresentationEpoch::initial().next();
        let newest = first.next();
        let mut schedule = ResidencySchedule::default();

        assert!(schedule.request_latest(ResidencyUrgency::Required));
        assert_eq!(schedule.select(first), Some(ResidencyUrgency::Required));
        for _ in 0..12 {
            assert!(
                !schedule.request_latest(ResidencyUrgency::Required),
                "same-urgency intent must coalesce before an expensive candidate is constructed"
            );
        }
        assert_eq!(schedule.select(newest), None);

        let first_retirement = schedule
            .retire_selected(first)
            .expect("front candidate should retire");
        assert!(first_retirement.schedule_follow_up);
        assert!(!first_retirement.finished);
        assert_eq!(schedule.selected_epoch(), None);
        assert_eq!(schedule.select(newest), Some(ResidencyUrgency::Required));
    }

    #[test]
    fn unrelated_completion_cannot_retire_the_selected_front() {
        let selected = crate::window::PresentationEpoch::initial().next();
        let unrelated = selected.next();
        let follow_up = unrelated.next();
        let mut schedule = ResidencySchedule::default();

        assert!(schedule.request_latest(ResidencyUrgency::Required));
        assert_eq!(schedule.select(selected), Some(ResidencyUrgency::Required));
        assert!(!schedule.request_latest(ResidencyUrgency::Required));
        assert_eq!(
            schedule.retire_selected(unrelated),
            None,
            "a newer unrelated submitted frame must not retire residency work selected under another epoch"
        );
        assert_eq!(schedule.selected_epoch(), Some(selected));

        let retirement = schedule
            .retire_selected(selected)
            .expect("the selected candidate must retire by its own identity");
        assert!(retirement.schedule_follow_up);
        assert_eq!(schedule.select(follow_up), Some(ResidencyUrgency::Required));
    }

    #[test]
    fn required_request_bypasses_a_selected_speculative_candidate() {
        let front = crate::window::PresentationEpoch::initial().next();
        let required = front.next();
        let mut schedule = ResidencySchedule::default();

        assert!(schedule.request_latest(ResidencyUrgency::Proactive));
        assert_eq!(schedule.select(front), Some(ResidencyUrgency::Proactive));
        assert!(
            !schedule.request_latest(ResidencyUrgency::Proactive),
            "same-urgency speculative work must coalesce before realization"
        );
        assert_eq!(schedule.select(required), None);

        assert!(
            schedule.request_latest(ResidencyUrgency::Required),
            "required residency must be able to bypass selected speculative work"
        );
        assert_eq!(schedule.select(required), Some(ResidencyUrgency::Required));
        assert_eq!(schedule.selected_epoch(), Some(front));
    }
}

pub struct Runtime<M: state::State, E: Send + 'static = (), V = ()> {
    store: Store<M>,
    timeline: Timeline<M>,
    session: session::Session,
    composition: composition::Store,
    layout: layout::Engine,
    scene: scene::Store,
    diagnostics: diagnostics::Store,
    clipboard: ConfiguredClipboard,
    tasks: task::Queue<E>,
    registry: command::Registry,
    keymap: keymap::Profile,
    observers: command::Observers<M>,
    responders: responder::Builder<M>,
    gesture: Option<transaction::gesture::Gesture<M>>,
    history_group: Option<transaction::history::ActiveGroup>,
    started: Option<Started<M>>,
    event: Option<Event<M, E>>,
    theme: Option<ThemeCallback<M>>,
    view: Option<ViewCallback<M, V>>,
    started_ran: bool,
    animation_schedules: departed::WindowMap<AnimationSchedules>,
    kinetic_scrolls: departed::WindowMap<KineticScroll>,
    visual_animations: visual::Animations,
    overlays: overlay::Store,
    overlay_capabilities: overlay::Capabilities,
    layout_cache: departed::WindowMap<CachedLayout>,
    presented_geometry: departed::WindowMap<PresentedGeometry>,
    residency_schedules: departed::WindowMap<ResidencySchedule>,
    virtual_materializations: departed::WindowMap<VirtualMaterializations>,
    virtual_measurements: departed::WindowMap<VirtualMeasurements>,
}

impl<M: state::State> Runtime<M> {
    pub fn new(model: M) -> Self {
        let mut registry = command::Registry::default();
        session::register(&mut registry);
        timeline::register(&mut registry);

        Self {
            store: Store::new(model),
            timeline: Timeline::default(),
            session: session::Session::default(),
            composition: composition::Store::default(),
            layout: layout::Engine::default(),
            scene: scene::Store::default(),
            diagnostics: diagnostics::Store::default(),
            clipboard: ConfiguredClipboard::default(),
            tasks: task::Queue::default(),
            registry,
            keymap: keymap::Profile::default(),
            observers: command::Observers::default(),
            responders: responder::Builder::default(),
            gesture: None,
            history_group: None,
            started: None,
            event: None,
            theme: None,
            view: None,
            started_ran: false,
            animation_schedules: departed::WindowMap::default(),
            kinetic_scrolls: departed::WindowMap::default(),
            visual_animations: visual::Animations::default(),
            overlays: overlay::Store::new(),
            overlay_capabilities: overlay::Capabilities::default(),
            layout_cache: departed::WindowMap::default(),
            presented_geometry: departed::WindowMap::default(),
            residency_schedules: departed::WindowMap::default(),
            virtual_materializations: departed::WindowMap::default(),
            virtual_measurements: departed::WindowMap::default(),
        }
    }
}

enum ConfiguredClipboard {
    Default(Clipboard),
    Explicit(Clipboard),
}

impl ConfiguredClipboard {
    fn explicit(clipboard: Clipboard) -> Self {
        Self::Explicit(clipboard)
    }

    fn use_system_default(&mut self) {
        if matches!(self, Self::Default(_)) {
            *self = Self::Default(Clipboard::system());
        }
    }
}

impl Default for ConfiguredClipboard {
    fn default() -> Self {
        Self::Default(Clipboard::default())
    }
}

impl Deref for ConfiguredClipboard {
    type Target = Clipboard;

    fn deref(&self) -> &Self::Target {
        match self {
            Self::Default(clipboard) | Self::Explicit(clipboard) => clipboard,
        }
    }
}

impl DerefMut for ConfiguredClipboard {
    fn deref_mut(&mut self) -> &mut Self::Target {
        match self {
            Self::Default(clipboard) | Self::Explicit(clipboard) => clipboard,
        }
    }
}

impl<M: state::State, E: Send + 'static, V> Runtime<M, E, V> {
    pub fn change(&mut self, reason: state::Reason, mutate: impl FnOnce(&mut M)) -> state::Change {
        let before = self.store.prepare_snapshot();
        mutate(self.store.model_mut());
        self.timeline.record(before.into_model());
        let change = self.store.commit_retaining_current(reason);
        self.request_all_redraws();
        change
    }

    pub fn undo(&mut self) -> bool {
        let trigger = self.trigger::<timeline::Undo>(());
        self.invoke(trigger).is_ok()
    }

    pub fn redo(&mut self) -> bool {
        let trigger = self.trigger::<timeline::Redo>(());
        self.invoke(trigger).is_ok()
    }

    pub(crate) fn active_theme(&self) -> theme::Theme {
        self.theme
            .as_ref()
            .map(|theme| theme(self.store.model()))
            .unwrap_or_default()
    }

    pub(crate) fn set_overlay_capabilities(&mut self, capabilities: overlay::Capabilities) {
        self.overlay_capabilities = capabilities;
    }
}
