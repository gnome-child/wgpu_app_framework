use crate::text;

use super::super::Runtime;
use crate::scratch::{
    clipboard, command, context as command_context, document, draft, error::Error, input,
    interaction, response, session, state, window,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum TextBoxShortcut {
    SelectAll,
    ClearSelection,
    Copy,
    Cut,
    Paste,
    Undo,
    Redo,
}

impl<M: state::State, E: Send + 'static, V> Runtime<M, E, V> {
    pub(in crate::scratch::runtime::input) fn handle_text_commit(
        &mut self,
        window: window::Id,
        text: String,
    ) -> std::result::Result<input::Outcome, Error> {
        let Some(focus) = self.session.focused(window) else {
            return Ok(input::Outcome::ignored());
        };

        if self.text_box_base_text(window, focus).is_some() {
            return self.handle_text_box_edit(window, focus, text::edit::Edit::ime_commit(text));
        }

        if text.is_empty() {
            return if self.session.clear_text_input(window) {
                Ok(self.window_outcome(window, false, response::Effect::Repaint))
            } else {
                Ok(input::Outcome::ignored())
            };
        }

        self.handle_text_edit(
            window,
            text::edit::Edit::ime_commit(text),
            command_context::Source::Input,
        )
    }

    fn text_box_base_text(&self, window: window::Id, focus: session::Focus) -> Option<String> {
        self.composition
            .get(window)?
            .view()
            .text_box_text(focus)
            .map(str::to_owned)
    }

    pub(in crate::scratch::runtime::input) fn text_input_target(
        &self,
        window: window::Id,
        focus: session::Focus,
    ) -> interaction::Target {
        self.composition
            .get(window)
            .and_then(|composition| composition.view().text_input_target(focus))
            .unwrap_or_else(|| interaction::Target::text_area(focus))
    }

    fn handle_text_box_edit(
        &mut self,
        window: window::Id,
        focus: session::Focus,
        edit: text::edit::Edit,
    ) -> std::result::Result<input::Outcome, Error> {
        let Some(base) = self.text_box_base_text(window, focus) else {
            return Ok(input::Outcome::ignored());
        };
        let Some(change) = self.session.edit_text_draft(window, focus, base, edit) else {
            return Ok(input::Outcome::ignored());
        };

        self.finish_text_box_change(window, focus, change)
    }

    fn finish_text_box_change(
        &mut self,
        window: window::Id,
        focus: session::Focus,
        change: draft::Change,
    ) -> std::result::Result<input::Outcome, Error> {
        let mut handled = change.changed() || change.submit();
        let mut changed_state = false;
        let mut effect = if change.changed() {
            response::Effect::Repaint
        } else {
            response::Effect::None
        };

        if change.text_changed() || change.submit() {
            let action = self.composition.get(window).and_then(|composition| {
                composition
                    .view()
                    .text_commit_action(focus, change.text().to_owned())
            });

            if let Some(action) = action {
                let outcome = self.handle_view(window, action)?;
                handled |= outcome.is_handled();
                changed_state |= outcome.changed_state();
                effect = effect.then(outcome.effect().clone());
            }
        }

        if handled {
            Ok(self.window_outcome(window, changed_state, effect))
        } else {
            Ok(input::Outcome::ignored())
        }
    }

    pub(in crate::scratch::runtime::input) fn handle_text_box_key_shortcut(
        &mut self,
        window: window::Id,
        key: input::Key,
        modifiers: input::Modifiers,
    ) -> std::result::Result<Option<input::Outcome>, Error> {
        let Some(shortcut) = text_box_shortcut_for_key(key, modifiers) else {
            return Ok(None);
        };

        self.handle_text_box_shortcut(window, shortcut)
    }

    fn handle_text_box_shortcut(
        &mut self,
        window: window::Id,
        shortcut: TextBoxShortcut,
    ) -> std::result::Result<Option<input::Outcome>, Error> {
        let Some(focus) = self.session.focused(window) else {
            return Ok(None);
        };
        let Some(base) = self.text_box_base_text(window, focus) else {
            return Ok(None);
        };

        let outcome = match shortcut {
            TextBoxShortcut::SelectAll => {
                self.handle_text_box_edit(window, focus, text::edit::Edit::SelectAll)?
            }
            TextBoxShortcut::ClearSelection => self.handle_text_box_edit(
                window,
                focus,
                text::edit::Edit::MovePosition(text::edit::Motion::VisualRight),
            )?,
            TextBoxShortcut::Copy => {
                if let Some(selection) = self.text_box_selected_text(window, focus, base) {
                    self.clipboard.put(&clipboard::Text::new(selection));
                }

                self.window_outcome(window, false, response::Effect::None)
            }
            TextBoxShortcut::Cut => {
                if let Some(selection) = self.text_box_selected_text(window, focus, base) {
                    self.clipboard.put(&clipboard::Text::new(selection));

                    self.handle_text_box_edit(window, focus, text::edit::Edit::Delete)?
                } else {
                    self.window_outcome(window, false, response::Effect::None)
                }
            }
            TextBoxShortcut::Paste => {
                if let Some(text) = self.clipboard.text() {
                    self.handle_text_box_edit(window, focus, text::edit::Edit::insert(text))?
                } else {
                    self.window_outcome(window, false, response::Effect::None)
                }
            }
            TextBoxShortcut::Undo => self.handle_text_box_history(window, focus, false)?,
            TextBoxShortcut::Redo => self.handle_text_box_history(window, focus, true)?,
        };

        Ok(Some(outcome))
    }

    fn handle_text_box_history(
        &mut self,
        window: window::Id,
        focus: session::Focus,
        redo: bool,
    ) -> std::result::Result<input::Outcome, Error> {
        let change = if redo {
            self.session.redo_text_draft(window, focus)
        } else {
            self.session.undo_text_draft(window, focus)
        };

        let Some(change) = change else {
            return Ok(self.window_outcome(window, false, response::Effect::None));
        };

        self.finish_text_box_change(window, focus, change)
    }

    fn text_box_selected_text(
        &self,
        window: window::Id,
        focus: session::Focus,
        base: String,
    ) -> Option<String> {
        let target = interaction::Target::text_area(focus);

        self.session
            .interaction(window)
            .and_then(|interaction| interaction.text_input().draft_for(&target).cloned())
            .unwrap_or_else(|| draft::State::new(base))
            .selected_text()
    }

    pub(in crate::scratch::runtime::input) fn handle_text_edit(
        &mut self,
        window: window::Id,
        edit: text::edit::Edit,
        source: command_context::Source,
    ) -> std::result::Result<input::Outcome, Error> {
        let Some(focus) = self.session.focused(window) else {
            return Ok(input::Outcome::ignored());
        };

        if self.text_box_base_text(window, focus).is_some() {
            return self.handle_text_box_edit(window, focus, edit);
        }

        let reveal_target = self.text_input_target(window, focus);
        let cleared_preedit = self.session.clear_text_input(window);
        let response = self.invoke_focused_with_source(
            window,
            command::Trigger::<document::ApplyEdit>::command(edit),
            source,
        );
        let changed = response.changed_state();
        let reveal = response
            .output_ref()
            .is_some_and(|outcome| outcome.buffer_changed());
        let mut effect = response.effect.clone();
        if reveal && self.session.reveal_scroll(window, reveal_target) {
            effect = effect.then(response::Effect::Repaint);
        }
        if cleared_preedit {
            effect = effect.then(response::Effect::Repaint);
            self.apply_window_update(window, false, &response::Effect::Repaint);
        }

        response
            .output
            .map(|_| input::Outcome::handled(changed, effect))
    }

    pub(in crate::scratch::runtime::input) fn handle_shortcut(
        &mut self,
        window: window::Id,
        shortcut: command::KeyChord,
    ) -> std::result::Result<input::Outcome, Error> {
        if let Some(text_box_outcome) =
            self.handle_text_box_shortcut_for_chord(window, shortcut.as_str())?
        {
            return Ok(text_box_outcome);
        }

        let Some(command) = self.registry.shortcut_command(shortcut)? else {
            return Ok(input::Outcome::ignored());
        };
        let command_type = command.command_type();
        let command_name = command.command_name();

        let source = command_context::Source::Shortcut;
        let Some(transaction) = self.transact_any_command(
            self.session.focused(window),
            Some(window),
            command_type,
            command_name,
            source,
            |registry, chain, cx| registry.invoke_shortcut(shortcut, chain, cx),
        )?
        else {
            return Ok(input::Outcome::ignored());
        };

        let changed = transaction.changed_state;
        let effect = transaction.effect;
        transaction
            .response
            .into_result()
            .map(|_| self.window_outcome(window, changed, effect))
    }

    fn handle_text_box_shortcut_for_chord(
        &mut self,
        window: window::Id,
        shortcut: &'static str,
    ) -> std::result::Result<Option<input::Outcome>, Error> {
        let Some(shortcut) = text_box_shortcut_for_chord(shortcut) else {
            return Ok(None);
        };

        self.handle_text_box_shortcut(window, shortcut)
    }

    pub(in crate::scratch::runtime::input) fn handle_text_drop(
        &mut self,
        window: window::Id,
        text_drop: input::TextDrop,
    ) -> std::result::Result<input::Outcome, Error> {
        let before = self.store.prepare_snapshot();
        let focus = self.session.focused(window);
        let reveal_target = focus.map(|focus| self.text_input_target(window, focus));
        let (edit, source_cleanup) = text_drop.into_edits();
        let task_sink = self.tasks.sink();
        let mut cx = command_context::Context::with_services_source(
            &mut self.clipboard,
            task_sink,
            command_context::Source::Input,
        )
        .with_text_service(self.layout.text_service());
        let mut chain = self.responders.chain_for(&mut self.store, focus);

        let response = self
            .registry
            .invoke::<document::ApplyEdit>(&mut chain, edit, &mut cx);
        let mut changed = response.changed_state();
        let mut effect = response.effect.clone();

        if let Err(error) = response.output {
            drop(chain);
            if changed {
                self.store.discard_retained_snapshot();
            } else {
                self.store.restore_prepared_snapshot(before);
            }
            return Err(error);
        }

        if changed && let Some(source_cleanup) = source_cleanup {
            let cleanup_response =
                self.registry
                    .invoke::<document::ApplyEdit>(&mut chain, source_cleanup, &mut cx);

            changed |= cleanup_response.changed_state();
            effect = effect.then(cleanup_response.effect.clone());

            if let Err(error) = cleanup_response.output {
                drop(chain);
                if changed {
                    self.store.discard_retained_snapshot();
                } else {
                    self.store.restore_prepared_snapshot(before);
                }
                return Err(error);
            }
        }

        drop(chain);

        if changed {
            if let Some(target) = reveal_target
                && self.session.reveal_scroll(window, target)
            {
                effect = effect.then(response::Effect::Repaint);
            }
            self.timeline.record(before.into_model());
            self.store
                .commit_retaining_current(state::Reason::event("text_drop"));
        } else {
            self.store.restore_prepared_snapshot(before);
        }

        Ok(self.window_outcome(window, changed, effect))
    }
}

fn text_box_shortcut_for_key(
    key: input::Key,
    modifiers: input::Modifiers,
) -> Option<TextBoxShortcut> {
    if modifiers.alt() || modifiers.super_key() || !modifiers.control() {
        return None;
    }

    match (key.normalized(), modifiers.shift()) {
        (input::Key::Character('a'), false) | (input::Key::Character('/'), false) => {
            Some(TextBoxShortcut::SelectAll)
        }
        (input::Key::Character('a'), true) | (input::Key::Character('\\'), false) => {
            Some(TextBoxShortcut::ClearSelection)
        }
        (input::Key::Character('c'), false) => Some(TextBoxShortcut::Copy),
        (input::Key::Character('x'), false) => Some(TextBoxShortcut::Cut),
        (input::Key::Character('v'), false) => Some(TextBoxShortcut::Paste),
        (input::Key::Character('z'), false) => Some(TextBoxShortcut::Undo),
        (input::Key::Character('z'), true) | (input::Key::Character('y'), false) => {
            Some(TextBoxShortcut::Redo)
        }
        _ => None,
    }
}

fn text_box_shortcut_for_chord(shortcut: &'static str) -> Option<TextBoxShortcut> {
    match shortcut.to_ascii_lowercase().as_str() {
        "ctrl+a" | "control+a" | "ctrl+/" | "control+/" => Some(TextBoxShortcut::SelectAll),
        "ctrl+shift+a" | "control+shift+a" | "ctrl+\\" | "control+\\" => {
            Some(TextBoxShortcut::ClearSelection)
        }
        "ctrl+c" | "control+c" => Some(TextBoxShortcut::Copy),
        "ctrl+x" | "control+x" => Some(TextBoxShortcut::Cut),
        "ctrl+v" | "control+v" => Some(TextBoxShortcut::Paste),
        "ctrl+z" | "control+z" => Some(TextBoxShortcut::Undo),
        "ctrl+shift+z" | "control+shift+z" | "ctrl+y" | "control+y" => {
            Some(TextBoxShortcut::Redo)
        }
        _ => None,
    }
}
