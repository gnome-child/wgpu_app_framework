use crate::text;
use crate::{command, context as command_context, document, response::Response, target::Target};

use super::{FocusedDraft, put_clipboard_text};

impl Target<document::Copy> for FocusedDraft<'_> {
    fn state(&self, _: &(), _: &command_context::Context) -> command::State {
        if self.is_selectable()
            && self
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

        match put_clipboard_text(cx, selection) {
            Ok(()) => Response::output(document::Outcome::from_text_change(false, false, true)),
            Err(_) => Response::output(document::Outcome::unavailable_result()),
        }
    }
}

impl Target<document::Cut> for FocusedDraft<'_> {
    fn state(&self, args: &(), cx: &command_context::Context) -> command::State {
        if self.is_editable() {
            Target::<document::Copy>::state(self, args, cx)
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

        match put_clipboard_text(cx, selection) {
            Ok(()) => self.edit_response(text::edit::Edit::Delete, true),
            Err(_) => Response::output(document::Outcome::unavailable_result()),
        }
    }
}

impl Target<document::Paste> for FocusedDraft<'_> {
    fn state(&self, _: &(), cx: &command_context::Context) -> command::State {
        if self.is_editable() {
            document::Paste::availability(cx)
        } else {
            command::State::disabled()
        }
    }

    fn invoke(&mut self, _: (), cx: &mut command_context::Context) -> Response<document::Outcome> {
        let Some(clipboard) = cx.clipboard_mut() else {
            return Response::output(document::Outcome::unavailable_result());
        };

        match clipboard.text() {
            Ok(Some(text)) => self.edit_response(text::edit::Edit::insert(text), false),
            Ok(None) => Response::output(document::Outcome::from_text_change(false, false, false)),
            Err(_) => Response::output(document::Outcome::unavailable_result()),
        }
    }
}
