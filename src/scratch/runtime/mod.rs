use super::{
    clipboard::Clipboard,
    command, composition, diagnostics, layout, responder, session,
    state::{self, Store},
    task,
    timeline::{self, Timeline},
    view,
};
mod access;
mod builder;
mod context;
mod dispatch;
mod input;
mod lifecycle;
mod pointer;
mod presentation;
mod retention;
mod routing;
mod services;
mod snapshot;
mod tasks;
mod transaction;
pub(super) mod work;

pub use context::Context;
pub use retention::Retention;
#[allow(unused_imports)]
pub use snapshot::{Persistence, Snapshot};

type Started<M> = Box<dyn for<'a> FnMut(&mut Context<'a, M>)>;
type Event<M, E> = Box<dyn for<'a> FnMut(&mut Context<'a, M>, E)>;
type ViewCallback<M, V> = Box<dyn Fn(&M, view::Context) -> V>;

pub struct Runtime<M: state::State, E: Send + 'static = (), V = ()> {
    store: Store<M>,
    timeline: Timeline<M>,
    session: session::Session,
    composition: composition::Store,
    layout: layout::engine::Engine,
    diagnostics: diagnostics::Store,
    clipboard: Clipboard,
    tasks: task::Queue<E>,
    registry: command::Registry,
    observers: command::Observers<M>,
    responders: responder::Builder<M>,
    gesture: Option<transaction::gesture::Gesture<M>>,
    history_group: Option<transaction::history::ActiveGroup>,
    started: Option<Started<M>>,
    event: Option<Event<M, E>>,
    view: Option<ViewCallback<M, V>>,
    started_ran: bool,
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
            layout: layout::engine::Engine::default(),
            diagnostics: diagnostics::Store::default(),
            clipboard: Clipboard::default(),
            tasks: task::Queue::default(),
            registry,
            observers: command::Observers::default(),
            responders: responder::Builder::default(),
            gesture: None,
            history_group: None,
            started: None,
            event: None,
            view: None,
            started_ran: false,
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
}
