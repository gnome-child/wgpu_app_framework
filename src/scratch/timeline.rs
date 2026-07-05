use super::{
    command,
    context::Context,
    error::Error,
    response::{self, Response},
    state::{self, Store},
    target::Target,
};

pub(super) const DEFAULT_SNAPSHOT_LIMIT: usize = 256;

pub struct Undo;

pub struct Redo;

impl command::Command for Undo {
    type Args = ();
    type Output = ();

    const NAME: &'static str = "edit.undo";
    const HISTORY: command::History = command::History::Committed;
}

impl command::Command for Redo {
    type Args = ();
    type Output = ();

    const NAME: &'static str = "edit.redo";
    const HISTORY: command::History = command::History::Committed;
}

pub struct Timeline<M: state::State> {
    past: Vec<M>,
    future: Vec<M>,
    snapshot_limit: usize,
}

pub(super) struct Service<'a, M: state::State> {
    store: &'a mut Store<M>,
    timeline: &'a mut Timeline<M>,
}

impl<M: state::State> Default for Timeline<M> {
    fn default() -> Self {
        Self {
            past: Vec::new(),
            future: Vec::new(),
            snapshot_limit: DEFAULT_SNAPSHOT_LIMIT,
        }
    }
}

impl<M: state::State> Timeline<M> {
    pub(super) fn record(&mut self, previous: M) {
        self.past.push(previous);
        self.prune_past();
        self.future.clear();
    }

    pub(super) fn undo(&mut self, current: &mut M) -> bool {
        let Some(previous) = self.past.pop() else {
            return false;
        };

        self.future.push(current.clone());
        self.prune_future();
        *current = previous;

        true
    }

    pub(super) fn redo(&mut self, current: &mut M) -> bool {
        let Some(next) = self.future.pop() else {
            return false;
        };

        self.past.push(current.clone());
        self.prune_past();
        *current = next;

        true
    }

    pub fn can_undo(&self) -> bool {
        !self.past.is_empty()
    }

    pub fn can_redo(&self) -> bool {
        !self.future.is_empty()
    }

    pub fn undo_depth(&self) -> usize {
        self.past.len()
    }

    pub fn redo_depth(&self) -> usize {
        self.future.len()
    }

    pub fn snapshot_limit(&self) -> usize {
        self.snapshot_limit
    }

    pub(super) fn set_snapshot_limit(&mut self, limit: usize) {
        self.snapshot_limit = limit;
        self.prune_past();
        self.prune_future();
    }

    pub(super) fn clear(&mut self) {
        self.past.clear();
        self.future.clear();
    }

    fn prune_past(&mut self) {
        if self.past.len() > self.snapshot_limit {
            let drop_count = self.past.len() - self.snapshot_limit;
            self.past.drain(0..drop_count);
        }
    }

    fn prune_future(&mut self) {
        if self.future.len() > self.snapshot_limit {
            let drop_count = self.future.len() - self.snapshot_limit;
            self.future.drain(0..drop_count);
        }
    }
}

impl<'a, M: state::State> Service<'a, M> {
    pub(super) fn new(store: &'a mut Store<M>, timeline: &'a mut Timeline<M>) -> Self {
        Self { store, timeline }
    }

    fn timeline(&self) -> &Timeline<M> {
        &*self.timeline
    }
}

impl<M: state::State> Target<Undo> for Service<'_, M> {
    fn state(&self, _args: &(), _cx: &Context) -> command::State {
        history_state(self.timeline().can_undo())
    }

    fn invoke(&mut self, _args: (), _cx: &mut Context) -> Response<()> {
        if !self.timeline.undo(self.store.model_mut()) {
            return Response::failed(Error::Disabled {
                command: <Undo as command::Command>::NAME,
            });
        }

        self.store.commit(state::Reason::Undo);
        Response::changed(()).with_effect(response::Effect::Repaint)
    }
}

impl<M: state::State> Target<Redo> for Service<'_, M> {
    fn state(&self, _args: &(), _cx: &Context) -> command::State {
        history_state(self.timeline().can_redo())
    }

    fn invoke(&mut self, _args: (), _cx: &mut Context) -> Response<()> {
        if !self.timeline.redo(self.store.model_mut()) {
            return Response::failed(Error::Disabled {
                command: <Redo as command::Command>::NAME,
            });
        }

        self.store.commit(state::Reason::Redo);
        Response::changed(()).with_effect(response::Effect::Repaint)
    }
}

pub(super) fn register(commands: &mut command::Registry) {
    commands
        .register::<Undo>(command::Spec::new("Undo").shortcut("Ctrl+Z"))
        .register::<Redo>(command::Spec::new("Redo").shortcut("Ctrl+Shift+Z"));
}

fn history_state(enabled: bool) -> command::State {
    if enabled {
        command::State::enabled()
    } else {
        command::State::disabled()
    }
}
