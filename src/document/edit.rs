use crate::text;

use super::{ApplyEdit, ApplySelection, Copy, Cut, Delete, Document, Outcome, Paste, SelectAll};
use crate::{command, context::Context, response::Response, target::Target};

impl Target<ApplyEdit> for Document {
    fn state(&self, _: &text::Edit, _: &Context) -> command::State {
        command::State::enabled()
    }

    fn invoke(&mut self, edit: text::Edit, _: &mut Context) -> Response<Outcome> {
        let outcome = self.apply_edit(edit);
        if outcome.buffer_changed() {
            Response::changed(outcome)
        } else {
            Response::output(outcome)
        }
    }
}

impl Target<ApplySelection> for Document {
    fn state(&self, _: &text::selection::Operation, _: &Context) -> command::State {
        command::State::enabled()
    }

    fn invoke(
        &mut self,
        operation: text::selection::Operation,
        cx: &mut Context,
    ) -> Response<Outcome> {
        let outcome = if let Some(mut text) = cx.text_service() {
            self.apply_selection_with_caret_map(operation, &mut text)
        } else {
            self.apply_selection(operation)
        };
        if outcome.buffer_changed() {
            Response::changed(outcome)
        } else {
            Response::output(outcome)
        }
    }
}

impl Target<SelectAll> for Document {
    fn state(&self, _: &(), _: &Context) -> command::State {
        if self.buffer.is_empty() {
            command::State::disabled()
        } else {
            command::State::enabled()
        }
    }

    fn invoke(&mut self, _: (), cx: &mut Context) -> Response<Outcome> {
        invoke_text_command(self, text::edit::Action::SelectAll, cx)
    }
}

impl Target<Copy> for Document {
    fn state(&self, _: &(), _: &Context) -> command::State {
        if self.buffer.has_selection_for_state(self.text_state) {
            command::State::enabled()
        } else {
            command::State::disabled()
        }
    }

    fn invoke(&mut self, _: (), cx: &mut Context) -> Response<Outcome> {
        invoke_text_command(self, text::edit::Action::Copy, cx)
    }
}

impl Target<Cut> for Document {
    fn state(&self, _: &(), _: &Context) -> command::State {
        if self.buffer.has_selection_for_state(self.text_state) {
            command::State::enabled()
        } else {
            command::State::disabled()
        }
    }

    fn invoke(&mut self, _: (), cx: &mut Context) -> Response<Outcome> {
        invoke_text_command(self, text::edit::Action::Cut, cx)
    }
}

impl Target<Delete> for Document {
    fn state(&self, _: &(), _: &Context) -> command::State {
        if self.buffer.has_selection_for_state(self.text_state) || !self.buffer.is_empty() {
            command::State::enabled()
        } else {
            command::State::disabled()
        }
    }

    fn invoke(&mut self, _: (), cx: &mut Context) -> Response<Outcome> {
        invoke_text_command(self, text::edit::Action::Delete, cx)
    }
}

impl Target<Paste> for Document {
    fn state(&self, _: &(), cx: &Context) -> command::State {
        Paste::availability(cx)
    }

    fn invoke(&mut self, _: (), cx: &mut Context) -> Response<Outcome> {
        invoke_text_command(self, text::edit::Action::Paste, cx)
    }
}

fn invoke_text_command(
    document: &mut Document,
    action: text::edit::Action,
    cx: &mut Context,
) -> Response<Outcome> {
    let Some(mut clipboard) = cx.clipboard_mut() else {
        return Response::output(Outcome::unavailable_result());
    };
    let result = document.apply_action(action, &mut clipboard);

    if result.buffer_changed() {
        Response::changed(result)
    } else {
        Response::output(result)
    }
}
