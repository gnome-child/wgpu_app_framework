use super::{
    clipboard::Clipboard,
    command, composition, diagnostics, geometry, keymap, layout, overlay, responder, session,
    state::{self, Store},
    task, theme,
    timeline::{self, Timeline},
    view,
};
use crate::animation;
mod access;
mod builder;
mod context;
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
    layout: layout::Layout,
}

pub struct Runtime<M: state::State, E: Send + 'static = (), V = ()> {
    store: Store<M>,
    timeline: Timeline<M>,
    session: session::Session,
    composition: composition::Store,
    layout: layout::Engine,
    diagnostics: diagnostics::Store,
    clipboard: Clipboard,
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
    animation_schedules: departed::WindowMap<animation::Schedule>,
    visual_animations: visual::Animations,
    overlays: overlay::Store,
    overlay_capabilities: overlay::Capabilities,
    layout_cache: departed::WindowMap<CachedLayout>,
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
            diagnostics: diagnostics::Store::default(),
            clipboard: Clipboard::default(),
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
