use crate::{context as command_context, response::Response, timeline};

use super::FocusedTextBox;

impl timeline::Undoable for FocusedTextBox<'_> {
    fn can_undo(&self) -> bool {
        self.is_editable() && self.draft().is_some_and(|draft| draft.can_undo())
    }

    fn can_redo(&self) -> bool {
        self.is_editable() && self.draft().is_some_and(|draft| draft.can_redo())
    }

    fn undo(&mut self, _: &mut command_context::Context) -> Response<()> {
        self.history_response(false)
    }

    fn redo(&mut self, _: &mut command_context::Context) -> Response<()> {
        self.history_response(true)
    }
}
