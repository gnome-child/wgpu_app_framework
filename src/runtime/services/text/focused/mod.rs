use crate::text;

use crate::{
    clipboard, composition, context as command_context, document, draft,
    response::{Effect, Response},
    session, window,
};

mod edit;
mod history;
mod transfer;

pub(super) struct FocusedDraft<'a> {
    session: &'a mut session::Session,
    composition: &'a composition::Store,
    window: window::Id,
    focus: session::Focus,
}

impl<'a> FocusedDraft<'a> {
    pub(super) fn new(
        session: &'a mut session::Session,
        composition: &'a composition::Store,
        window: window::Id,
        focus: session::Focus,
    ) -> Self {
        Self {
            session,
            composition,
            window,
            focus,
        }
    }

    fn base_text(&self) -> Option<String> {
        self.composition
            .get(self.window)?
            .view()
            .draft_text(self.focus)
    }

    fn input(&self) -> text::Input {
        self.composition
            .get(self.window)
            .and_then(|composition| composition.view().draft_input(self.focus))
            .unwrap_or_else(text::Input::unrestricted)
    }

    fn mode(&self) -> text::surface::FieldMode {
        self.composition
            .get(self.window)
            .and_then(|composition| composition.view().text_surface_mode(self.focus))
            .unwrap_or(text::surface::FieldMode::Editable)
    }

    fn is_editable(&self) -> bool {
        self.mode() == text::surface::FieldMode::Editable
    }

    fn is_selectable(&self) -> bool {
        self.mode() != text::surface::FieldMode::Disabled
    }

    fn draft(&self) -> Option<draft::State> {
        let base = self.base_text()?;
        let target = self.focus.text_target()?;

        Some(
            self.session
                .interaction(self.window)
                .and_then(|interaction| interaction.text_input().draft_for(&target).cloned())
                .unwrap_or_else(|| draft::State::new(base)),
        )
    }

    fn selected_text(&self, base: String) -> Option<String> {
        let target = self.focus.text_target()?;

        self.session
            .interaction(self.window)
            .and_then(|interaction| interaction.text_input().draft_for(&target).cloned())
            .unwrap_or_else(|| draft::State::new(base))
            .selected_text()
    }

    fn edit_response(
        &mut self,
        edit: text::Edit,
        clipboard_changed: bool,
    ) -> Response<document::Outcome> {
        if !self.is_editable() {
            return Response::output(document::Outcome::from_text_change(
                false,
                false,
                clipboard_changed,
            ));
        }
        let Some(base) = self.base_text() else {
            return Response::output(document::Outcome::from_text_change(
                false,
                false,
                clipboard_changed,
            ));
        };
        let input = self.input();
        let had_input_feedback = self
            .session
            .text_input_feedback(self.window, self.focus)
            .is_some();
        let Some(change) = self
            .session
            .edit_text_draft(self.window, self.focus, base, edit, input)
        else {
            return Response::output(document::Outcome::from_text_change(
                false,
                false,
                clipboard_changed,
            ));
        };
        let output = document::Outcome::from_text_change(
            change.text_changed(),
            change.selection_changed(),
            clipboard_changed,
        );

        Response::output(output).with_effect(effect_for_change(&change, had_input_feedback))
    }

    fn selection_response(
        &mut self,
        operation: text::selection::Operation,
        clipboard_changed: bool,
    ) -> Response<document::Outcome> {
        if !self.is_selectable() {
            return Response::output(document::Outcome::from_text_change(
                false,
                false,
                clipboard_changed,
            ));
        }
        let Some(base) = self.base_text() else {
            return Response::output(document::Outcome::from_text_change(
                false,
                false,
                clipboard_changed,
            ));
        };
        let had_input_feedback = self
            .session
            .text_input_feedback(self.window, self.focus)
            .is_some();
        let Some(change) = self
            .session
            .select_text_draft(self.window, self.focus, base, operation)
        else {
            return Response::output(document::Outcome::from_text_change(
                false,
                false,
                clipboard_changed,
            ));
        };
        let output = document::Outcome::from_text_change(
            false,
            change.selection_changed(),
            clipboard_changed,
        );

        Response::output(output).with_effect(effect_for_change(&change, had_input_feedback))
    }

    fn history_response(&mut self, redo: bool) -> Response<()> {
        let had_input_feedback = self
            .session
            .text_input_feedback(self.window, self.focus)
            .is_some();
        let change = if redo {
            self.session.redo_text_draft(self.window, self.focus)
        } else {
            self.session.undo_text_draft(self.window, self.focus)
        };

        let Some(change) = change else {
            return Response::output(());
        };

        Response::output(()).with_effect(effect_for_change(&change, had_input_feedback))
    }
}

fn put_clipboard_text(cx: &command_context::Context, text: String) -> clipboard::Result<()> {
    let Some(clipboard) = cx.clipboard() else {
        return Err(clipboard::Error::Unavailable);
    };

    clipboard.put(&clipboard::Text::new(text))
}

fn effect_for_change(change: &draft::Change, removed_input_feedback: bool) -> Effect {
    if change.text_changed() && removed_input_feedback {
        Effect::Rebuild
    } else if change.changed() {
        Effect::Layout
    } else {
        Effect::None
    }
}
