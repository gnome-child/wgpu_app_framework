use crate::text;

use super::{ApplyEdit, ApplySelection, Copy, Cut, Delete, Document, Outcome, Paste, SelectAll};
use crate::{clipboard, command, context::Context, response::Response, target::Target};

impl Target<ApplyEdit> for Document {
    fn state(&self, _: &text::Edit, _: &Context) -> command::State {
        command::State::enabled()
    }

    fn invoke(&mut self, edit: text::Edit, _: &mut Context) -> Response<Outcome> {
        respond(self.apply_edit(edit))
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
        let outcome = if let Some(caret_map) = cx.caret_map() {
            self.apply_selection_with_caret_map(operation, &mut *caret_map.borrow_mut())
        } else {
            self.apply_selection(operation)
        };
        respond(outcome)
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
        let Some(_clipboard) = cx.clipboard_mut() else {
            return unavailable();
        };

        respond(self.apply_selection(text::selection::Operation::SelectAll))
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
        let Some(clipboard) = cx.clipboard_mut() else {
            return unavailable();
        };
        let Some(selection) = self.selected_text() else {
            return unchanged();
        };

        match clipboard.put(&clipboard::Text::new(selection)) {
            Ok(()) => Response::output(Outcome::from_text_change(false, false, true)),
            Err(_) => unavailable(),
        }
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
        let Some(clipboard) = cx.clipboard_mut() else {
            return unavailable();
        };
        let Some(selection) = self.selected_text() else {
            return unchanged();
        };

        match clipboard.put(&clipboard::Text::new(selection)) {
            Ok(()) => {
                let edit = self.apply_edit(text::Edit::insert(""));
                respond(Outcome::from_text_change(
                    edit.text_changed(),
                    edit.selection_changed(),
                    true,
                ))
            }
            Err(_) => unavailable(),
        }
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
        let Some(_clipboard) = cx.clipboard_mut() else {
            return unavailable();
        };

        respond(self.apply_edit(text::Edit::Delete))
    }
}

impl Target<Paste> for Document {
    fn state(&self, _: &(), cx: &Context) -> command::State {
        Paste::availability(cx)
    }

    fn invoke(&mut self, _: (), cx: &mut Context) -> Response<Outcome> {
        let Some(clipboard) = cx.clipboard_mut() else {
            return unavailable();
        };

        match clipboard.text() {
            Ok(Some(text))
                if !text::buffer::normalize_for_buffer(self.buffer(), &text).is_empty() =>
            {
                respond(self.apply_edit(text::Edit::insert(text)))
            }
            Ok(_) => unchanged(),
            Err(_) => unavailable(),
        }
    }
}

fn respond(outcome: Outcome) -> Response<Outcome> {
    if outcome.buffer_changed() {
        Response::changed(outcome)
    } else {
        Response::output(outcome)
    }
}

fn unchanged() -> Response<Outcome> {
    Response::output(Outcome::from_text_change(false, false, false))
}

fn unavailable() -> Response<Outcome> {
    Response::output(Outcome::unavailable_result())
}
