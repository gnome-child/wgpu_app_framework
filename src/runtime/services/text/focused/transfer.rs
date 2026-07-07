use crate::text;
use crate::{
    clipboard, command, context as command_context, document, response::Response, target::Target,
};

use super::{FocusedTextBox, put_clipboard_text};

impl Target<document::Copy> for FocusedTextBox<'_> {
    fn state(&self, _: &(), _: &command_context::Context) -> command::State {
        if self
            .draft()
            .and_then(|draft| draft.selected_text())
            .is_some()
        {
            command::State::enabled()
        } else {
            command::State::disabled()
        }
    }

    fn invoke(&mut self, _: (), cx: &mut command_context::Context) -> Response<document::Outcome> {
        let Some(base) = self.base_text() else {
            return Response::output(document::Outcome::from_text_change(false, false, false));
        };
        let Some(selection) = self.selected_text(base) else {
            return Response::output(document::Outcome::from_text_change(false, false, false));
        };

        let clipboard_changed = put_clipboard_text(cx, selection);
        Response::output(document::Outcome::from_text_change(
            false,
            false,
            clipboard_changed,
        ))
    }
}

impl Target<document::Cut> for FocusedTextBox<'_> {
    fn state(&self, args: &(), cx: &command_context::Context) -> command::State {
        Target::<document::Copy>::state(self, args, cx)
    }

    fn invoke(&mut self, _: (), cx: &mut command_context::Context) -> Response<document::Outcome> {
        let Some(base) = self.base_text() else {
            return Response::output(document::Outcome::from_text_change(false, false, false));
        };
        let Some(selection) = self.selected_text(base) else {
            return Response::output(document::Outcome::from_text_change(false, false, false));
        };

        let clipboard_changed = put_clipboard_text(cx, selection);
        self.edit_response(text::edit::Edit::Delete, clipboard_changed)
    }
}

impl Target<document::Paste> for FocusedTextBox<'_> {
    fn state(&self, _: &(), cx: &command_context::Context) -> command::State {
        if cx.clipboard().is_some_and(clipboard::Clipboard::has_text) {
            command::State::enabled()
        } else {
            command::State::disabled()
        }
    }

    fn invoke(&mut self, _: (), cx: &mut command_context::Context) -> Response<document::Outcome> {
        let Some(text) = cx.clipboard_mut().and_then(|clipboard| clipboard.text()) else {
            return Response::output(document::Outcome::from_text_change(false, false, false));
        };

        self.edit_response(text::edit::Edit::insert(text), false)
    }
}
