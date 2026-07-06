use super::super::{
    clipboard::Clipboard,
    command::{self, Command},
    responder, state, task, view,
};
use super::{Context, Retention, Runtime};

impl<M: state::State, E: Send + 'static, V> Runtime<M, E, V> {
    pub fn retention(mut self, retention: Retention) -> Self {
        self.store.set_change_limit(retention.change_limit());
        self.timeline.set_snapshot_limit(retention.snapshot_limit());
        self.session.set_draft_limit(retention.draft_limit());
        self
    }

    pub fn with_clipboard(mut self, clipboard: Clipboard) -> Self {
        self.clipboard = clipboard;
        self
    }

    pub fn commands(mut self, configure: impl FnOnce(&mut command::Registry)) -> Self {
        configure(&mut self.registry);
        self
    }

    pub fn responders(mut self, configure: impl FnOnce(&mut responder::Builder<M>)) -> Self {
        configure(&mut self.responders);
        self
    }

    pub fn observe<C>(
        mut self,
        callback: impl FnMut(&mut M, &C::Output, &mut command::Observation) + 'static,
    ) -> Self
    where
        C: Command,
    {
        self.observers.observe::<C>(callback);
        self
    }

    pub fn started(mut self, callback: impl for<'a> FnMut(&mut Context<'a, M>) + 'static) -> Self {
        self.started = Some(Box::new(callback));
        self
    }

    pub fn event<E2: Send + 'static>(
        self,
        callback: impl for<'a> FnMut(&mut Context<'a, M>, E2) + 'static,
    ) -> Runtime<M, E2, V> {
        Runtime {
            store: self.store,
            timeline: self.timeline,
            session: self.session,
            composition: self.composition,
            layout: self.layout,
            diagnostics: self.diagnostics,
            clipboard: self.clipboard,
            tasks: task::Queue::default(),
            registry: self.registry,
            observers: self.observers,
            responders: self.responders,
            gesture: self.gesture,
            history_group: self.history_group,
            started: self.started,
            event: Some(Box::new(callback)),
            view: self.view,
            started_ran: self.started_ran,
        }
    }

    pub fn view<V2>(
        self,
        callback: impl Fn(&M, view::Context) -> V2 + 'static,
    ) -> Runtime<M, E, V2> {
        Runtime {
            store: self.store,
            timeline: self.timeline,
            session: self.session,
            composition: self.composition,
            layout: self.layout,
            diagnostics: self.diagnostics,
            clipboard: self.clipboard,
            tasks: self.tasks,
            registry: self.registry,
            observers: self.observers,
            responders: self.responders,
            gesture: self.gesture,
            history_group: self.history_group,
            started: self.started,
            event: self.event,
            view: Some(Box::new(callback)),
            started_ran: self.started_ran,
        }
    }
}
