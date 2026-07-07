use crate::scratch::{
    command, context as command_context, document, response::Response, target::Target,
};
use crate::text;

use super::FocusedTextBox;

impl Target<document::SelectAll> for FocusedTextBox<'_> {
    fn state(&self, _: &(), _: &command_context::Context) -> command::State {
        if self.draft().is_some_and(|draft| !draft.text().is_empty()) {
            command::State::enabled()
        } else {
            command::State::disabled()
        }
    }

    fn invoke(&mut self, _: (), _: &mut command_context::Context) -> Response<document::Outcome> {
        self.edit_response(text::edit::Edit::SelectAll, false)
    }
}

impl Target<document::Delete> for FocusedTextBox<'_> {
    fn state(&self, _: &(), _: &command_context::Context) -> command::State {
        if self.draft().is_some_and(|draft| !draft.text().is_empty()) {
            command::State::enabled()
        } else {
            command::State::disabled()
        }
    }

    fn invoke(&mut self, _: (), _: &mut command_context::Context) -> Response<document::Outcome> {
        self.edit_response(text::edit::Edit::Delete, false)
    }
}
