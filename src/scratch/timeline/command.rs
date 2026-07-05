use crate::scratch::{
    command,
    context::Context,
    error::Error,
    response::{self, Response},
    state,
    target::Target,
};

use super::service::Service;

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

pub(in crate::scratch) fn register(commands: &mut command::Registry) {
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
