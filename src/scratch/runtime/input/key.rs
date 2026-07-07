use super::super::Runtime;
use crate::scratch::{context as command_context, error::Error, input, state, view, window};

fn text_for_key(
    key: input::Key,
    modifiers: input::Modifiers,
    inserted_text: Option<&str>,
) -> Option<String> {
    if modifiers.control() || modifiers.alt() || modifiers.super_key() {
        return None;
    }

    if let Some(text) =
        inserted_text.filter(|text| text.chars().all(|character| !character.is_control()))
    {
        return Some(text.to_owned());
    }

    match key {
        input::Key::Space => Some(" ".to_owned()),
        input::Key::Character(character) if !character.is_control() => Some(character.to_string()),
        _ => None,
    }
}

impl<M: state::State, E: Send + 'static, V> Runtime<M, E, V> {
    pub(in crate::scratch::runtime::input) fn handle_key_down(
        &mut self,
        window: window::Id,
        key: input::Key,
        modifiers: input::Modifiers,
        text: Option<String>,
    ) -> std::result::Result<input::Outcome, Error> {
        if key == input::Key::Escape {
            return self.handle_input(window, input::Input::cancel());
        }

        if key == input::Key::Tab
            && !modifiers.control()
            && !modifiers.alt()
            && !modifiers.super_key()
        {
            return self.handle_tab_focus(window, modifiers.shift());
        }

        if let Some(outcome) = self.handle_command_palette_key(window, key, modifiers)? {
            return Ok(outcome);
        }

        if let Some(outcome) = self.handle_text_box_key_shortcut(window, key, modifiers)? {
            return Ok(outcome);
        }

        if let Some(shortcut) = self
            .registry
            .shortcut_for_key(key, modifiers, self.keymap)?
        {
            let outcome = self.handle_shortcut(window, shortcut)?;
            if outcome.is_handled() {
                return Ok(outcome);
            }
        }

        if matches!(key, input::Key::Enter | input::Key::Space)
            && !modifiers.control()
            && !modifiers.alt()
            && !modifiers.super_key()
            && let Some(outcome) = self.handle_focused_activation(window)?
        {
            return Ok(outcome);
        }

        if let Some(text) = text_for_key(key, modifiers, text.as_deref()) {
            return self.handle_text_commit(window, text);
        }

        let Some(edit) = self.keymap.edit_for_key(key, modifiers) else {
            return Ok(input::Outcome::ignored());
        };

        self.handle_text_edit(window, edit, command_context::Source::Keyboard)
    }

    fn handle_tab_focus(
        &mut self,
        window: window::Id,
        reverse: bool,
    ) -> std::result::Result<input::Outcome, Error> {
        let direction = if reverse {
            view::action::FocusDirection::Backward
        } else {
            view::action::FocusDirection::Forward
        };
        let Some(next) = self.composition.get(window).and_then(|composition| {
            composition.next_focus(self.session.focused(window), direction)
        }) else {
            return Ok(input::Outcome::ignored());
        };

        self.focus_committing_text_box(window, next)
    }

    fn handle_focused_activation(
        &mut self,
        window: window::Id,
    ) -> std::result::Result<Option<input::Outcome>, Error> {
        let Some(focus) = self.session.focused(window) else {
            return Ok(None);
        };
        let Some(action) = self
            .composition
            .get(window)
            .and_then(|composition| composition.focus_action(&focus))
        else {
            return Ok(None);
        };

        self.handle_view(window, action).map(Some)
    }
}
