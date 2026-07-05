use crate::scratch::{command, context::Context, response::Response, target::Target};

use super::{Redo, Undo};

/// Capability for targets with their own local undo/redo history.
///
/// Implement this when a type should answer the framework's standard Undo/Redo
/// commands. A type can still hand-write `Target<Undo>` or `Target<Redo>` for
/// exceptional behavior, but normal history participants should opt in here.
pub trait Undoable {
    fn can_undo(&self) -> bool;

    fn can_redo(&self) -> bool;

    fn undo(&mut self, cx: &mut Context) -> Response<()>;

    fn redo(&mut self, cx: &mut Context) -> Response<()>;
}

impl<T: Undoable> Target<Undo> for T {
    fn state(&self, _: &(), _: &Context) -> command::State {
        undo_state(self.can_undo())
    }

    fn invoke(&mut self, _: (), cx: &mut Context) -> Response<()> {
        self.undo(cx)
    }
}

impl<T: Undoable> Target<Redo> for T {
    fn state(&self, _: &(), _: &Context) -> command::State {
        undo_state(self.can_redo())
    }

    fn invoke(&mut self, _: (), cx: &mut Context) -> Response<()> {
        self.redo(cx)
    }
}

fn undo_state(enabled: bool) -> command::State {
    if enabled {
        command::State::enabled()
    } else {
        command::State::disabled()
    }
}
