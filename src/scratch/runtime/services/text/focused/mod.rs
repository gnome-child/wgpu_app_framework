use crate::text;

use crate::scratch::{
    clipboard, composition, context as command_context, document, draft, interaction,
    response::{Effect, Response},
    session, window,
};

mod edit;
mod history;
mod transfer;

pub(super) struct FocusedTextBox<'a> {
    session: &'a mut session::Session,
    composition: &'a composition::Store,
    window: window::Id,
    focus: session::Focus,
}

impl<'a> FocusedTextBox<'a> {
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
            .text_box_text(self.focus)
            .map(str::to_owned)
    }

    fn draft(&self) -> Option<draft::State> {
        let base = self.base_text()?;
        let target = interaction::Target::text_area(self.focus);

        Some(
            self.session
                .interaction(self.window)
                .and_then(|interaction| interaction.text_input().draft_for(&target).cloned())
                .unwrap_or_else(|| draft::State::new(base)),
        )
    }

    fn selected_text(&self, base: String) -> Option<String> {
        let target = interaction::Target::text_area(self.focus);

        self.session
            .interaction(self.window)
            .and_then(|interaction| interaction.text_input().draft_for(&target).cloned())
            .unwrap_or_else(|| draft::State::new(base))
            .selected_text()
    }

    fn edit_response(
        &mut self,
        edit: text::edit::Edit,
        clipboard_changed: bool,
    ) -> Response<document::Outcome> {
        let Some(base) = self.base_text() else {
            return Response::output(document::Outcome::from_text_change(
                false,
                false,
                clipboard_changed,
            ));
        };
        let Some(change) = self
            .session
            .edit_text_draft(self.window, self.focus, base, edit)
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

        Response::output(output).with_effect(effect_for_change(&change))
    }

    fn history_response(&mut self, redo: bool) -> Response<()> {
        let change = if redo {
            self.session.redo_text_draft(self.window, self.focus)
        } else {
            self.session.undo_text_draft(self.window, self.focus)
        };

        let Some(change) = change else {
            return Response::output(());
        };

        Response::output(()).with_effect(effect_for_change(&change))
    }
}

fn put_clipboard_text(cx: &mut command_context::Context, text: String) -> bool {
    let Some(clipboard) = cx.clipboard_mut() else {
        return false;
    };

    clipboard.put(&clipboard::Text::new(text));
    true
}

fn effect_for_change(change: &draft::Change) -> Effect {
    if change.changed() {
        Effect::Layout
    } else {
        Effect::None
    }
}
