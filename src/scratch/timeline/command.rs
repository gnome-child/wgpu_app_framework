use crate::scratch::{
    command,
    context::Context,
    error::Error,
    response::{self, Response},
    state,
};

use super::Undoable;
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

impl<M: state::State> Undoable for Service<'_, M> {
    fn can_undo(&self) -> bool {
        self.timeline().can_undo()
    }

    fn can_redo(&self) -> bool {
        self.timeline().can_redo()
    }

    fn undo(&mut self, _cx: &mut Context) -> Response<()> {
        if !self.timeline.undo(self.store.model_mut()) {
            return Response::failed(Error::Disabled {
                command: <Undo as command::Command>::NAME,
            });
        }

        self.store.commit(state::Reason::Undo);
        Response::changed(()).with_effect(response::Effect::Rebuild)
    }

    fn redo(&mut self, _cx: &mut Context) -> Response<()> {
        if !self.timeline.redo(self.store.model_mut()) {
            return Response::failed(Error::Disabled {
                command: <Redo as command::Command>::NAME,
            });
        }

        self.store.commit(state::Reason::Redo);
        Response::changed(()).with_effect(response::Effect::Rebuild)
    }
}

pub(in crate::scratch) fn register(commands: &mut command::Registry) {
    commands
        .register::<Undo>(
            command::Spec::new("Undo")
                .key_chord(command::KeyChord::standard(command::Standard::Undo)),
        )
        .register::<Redo>(
            command::Spec::new("Redo")
                .key_chord(command::KeyChord::standard(command::Standard::Redo)),
        );
}
