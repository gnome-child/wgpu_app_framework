use super::{
    clipboard::Clipboard,
    command, composition, diagnostics, geometry, interaction, keymap, layout, overlay, responder,
    scene, session,
    state::{self, Store},
    task, theme,
    timeline::{self, Timeline},
    view,
};
use crate::animation;
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
}

impl PresentedGeometry {
    fn project_point(
        &self,
        node: composition::tree::NodeId,
        point: geometry::Point,
        clip: bool,
    ) -> Option<(geometry::Point, [i32; 2])> {
        let mut translation = [0_i32, 0_i32];
        let projections = self.layout.scroll_projections();
        for scroll in self.layout.scroll_ancestry(node) {
            let projection = projections
                .iter()
                .find(|projection| projection.node() == *scroll)?;
            let viewport = projection.viewport();
            let visible = viewport.visible_content();
            let visible = geometry::Rect::new(
                visible.x().saturating_add(translation[0]),
                visible.y().saturating_add(translation[1]),
                visible.width(),
                visible.height(),
            );
            if clip && !visible.contains(point) {
                return None;
            }
            let baseline = viewport.resolved_scroll();
            let current = self.stack.scroll_offset(*scroll).unwrap_or(baseline);
            translation[0] =
                translation[0].saturating_add(baseline.x().saturating_sub(current.x()));
            translation[1] =
                translation[1].saturating_add(baseline.y().saturating_sub(current.y()));
        }
        Some((
            geometry::Point::new(
                point.x().saturating_sub(translation[0]),
                point.y().saturating_sub(translation[1]),
            ),
            translation,
        ))
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
    ) -> geometry::Rect {
        let translation = self
            .project_point(node, geometry::Point::new(0, 0), false)
            .map(|(_, translation)| translation)
            .unwrap_or([0, 0]);
        geometry::Rect::new(
            rect.x().saturating_add(translation[0]),
            rect.y().saturating_add(translation[1]),
            rect.width(),
            rect.height(),
        )
    }

    fn context_available_for_node(
        &self,
        node: composition::tree::NodeId,
    ) -> Option<geometry::Rect> {
        self.layout
            .context_available_for_node(node)
            .map(|rect| self.translated_rect(node, rect))
    }

    fn focus_node_and_rect(
        &self,
        focus: session::Focus,
    ) -> Option<(composition::tree::NodeId, geometry::Rect)> {
        let frame = self.layout.frame_for_focus(focus)?;
        Some((
            frame.node_id(),
            self.translated_rect(frame.node_id(), frame.rect()),
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

    fn scroll_target_at_surface(
        &self,
        point: geometry::Point,
        delta: interaction::ScrollDelta,
        surface: crate::popup::Surface,
    ) -> Option<interaction::Target> {
        self.layout.scroll_target_at_surface_projected(
            point,
            delta,
            surface,
            &|node, point| {
                self.project_point(node, point, true)
                    .map(|(point, _)| point)
            },
            &|target, viewport| {
                self.layout
                    .scroll_projections()
                    .iter()
                    .find(|projection| projection.target() == target)
                    .and_then(|projection| self.stack.scroll_offset(projection.node()))
                    .unwrap_or_else(|| viewport.resolved_scroll())
            },
        )
    }
}

type VirtualMaterializations =
    HashMap<crate::interaction::Id, crate::virtual_list::Materialization>;
type VirtualMeasurements = HashMap<crate::interaction::Id, crate::virtual_list::Measurements>;

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
struct AnimationSchedules {
    paint: animation::Schedule,
    properties: animation::Schedule,
}

impl AnimationSchedules {
    fn combined(self) -> animation::Schedule {
        self.paint.merge(self.properties)
    }

    fn is_idle(self) -> bool {
        self.paint == animation::Schedule::Idle && self.properties == animation::Schedule::Idle
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
    visual_animations: visual::Animations,
    overlays: overlay::Store,
    overlay_capabilities: overlay::Capabilities,
    layout_cache: departed::WindowMap<CachedLayout>,
    presented_geometry: departed::WindowMap<PresentedGeometry>,
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
            visual_animations: visual::Animations::default(),
            overlays: overlay::Store::new(),
            overlay_capabilities: overlay::Capabilities::default(),
            layout_cache: departed::WindowMap::default(),
            presented_geometry: departed::WindowMap::default(),
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
