use super::super::Runtime;
use crate::{
    command::Error, context as command_context, input, keymap, session, state, view, window,
};

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
    pub(in crate::runtime::input) fn handle_key_down(
        &mut self,
        window: window::Id,
        key: input::Key,
        modifiers: input::Modifiers,
        text: Option<String>,
    ) -> std::result::Result<input::Outcome, Error> {
        if key == input::Key::Escape {
            return self.handle_input(window, input::Input::cancel());
        }

        if key == input::Key::ContextMenu
            || (key == input::Key::F10
                && modifiers.shift()
                && !modifiers.control()
                && !modifiers.alt()
                && !modifiers.super_key())
        {
            return self.open_context_menu_for_focus(window);
        }

        if let Some(outcome) = self.handle_command_palette_scope_key(window, key, modifiers)? {
            return Ok(outcome);
        }

        if let Some(outcome) = self.handle_table_edit_key(window, key, modifiers)? {
            return Ok(outcome);
        }

        if let Some(outcome) = self.handle_virtual_selection_key(window, key, modifiers) {
            return Ok(outcome);
        }

        if key == input::Key::Tab
            && !modifiers.control()
            && !modifiers.alt()
            && !modifiers.super_key()
        {
            return self.handle_tab_focus(window, modifiers.shift());
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

        if self
            .session
            .focused(window)
            .and_then(session::Focus::text_target)
            .is_none()
            && let Some(outcome) = self.handle_scroll_container_key(window, key, modifiers)
        {
            return Ok(outcome);
        }

        let Some(operation) = self.keymap.text_operation_for_key(key, modifiers) else {
            return Ok(input::Outcome::ignored());
        };

        match operation {
            keymap::TextOperation::Selection(operation) => {
                self.handle_text_selection(window, operation, command_context::Source::Keyboard)
            }
            keymap::TextOperation::Edit(edit) => {
                self.handle_text_edit(window, edit, command_context::Source::Keyboard)
            }
        }
    }

    fn handle_scroll_container_key(
        &mut self,
        window: window::Id,
        key: input::Key,
        modifiers: input::Modifiers,
    ) -> Option<input::Outcome> {
        if modifiers.shift() || modifiers.control() || modifiers.alt() || modifiers.super_key() {
            return None;
        }
        let (axis, intent) = match key {
            input::Key::ArrowLeft => (
                crate::interaction::ScrollbarAxis::Horizontal,
                KeyboardScrollIntent::PhysicalBackward,
            ),
            input::Key::ArrowRight => (
                crate::interaction::ScrollbarAxis::Horizontal,
                KeyboardScrollIntent::PhysicalForward,
            ),
            input::Key::ArrowUp => (
                crate::interaction::ScrollbarAxis::Vertical,
                KeyboardScrollIntent::StepBackward,
            ),
            input::Key::ArrowDown => (
                crate::interaction::ScrollbarAxis::Vertical,
                KeyboardScrollIntent::StepForward,
            ),
            input::Key::PageUp => (
                crate::interaction::ScrollbarAxis::Vertical,
                KeyboardScrollIntent::PageBackward,
            ),
            input::Key::PageDown => (
                crate::interaction::ScrollbarAxis::Vertical,
                KeyboardScrollIntent::PageForward,
            ),
            input::Key::Home => (
                crate::interaction::ScrollbarAxis::Vertical,
                KeyboardScrollIntent::ToStart,
            ),
            input::Key::End => (
                crate::interaction::ScrollbarAxis::Vertical,
                KeyboardScrollIntent::ToEnd,
            ),
            _ => return None,
        };
        let focus = self.session.focused(window)?;
        let targets = self
            .presented_geometry
            .get(&window)?
            .scroll_target_chain_for_focus(focus, axis);
        for (target, direction) in targets {
            let reversed = axis == crate::interaction::ScrollbarAxis::Horizontal
                && direction == crate::scroll::Direction::RightToLeft;
            let operation = intent.operation(reversed);
            if let Some(outcome) = self.apply_scroll_operation(
                window,
                target,
                axis,
                operation,
                reversed,
                crate::interaction::ScrollSource::Keyboard,
            ) {
                return Some(outcome);
            }
        }
        None
    }

    pub(in crate::runtime::input) fn handle_tab_focus(
        &mut self,
        window: window::Id,
        reverse: bool,
    ) -> std::result::Result<input::Outcome, Error> {
        let direction = if reverse {
            view::FocusDirection::Backward
        } else {
            view::FocusDirection::Forward
        };
        let current = self.session.focused(window);
        let Some(next) = self.composition.get(window).and_then(|composition| {
            current
                .and_then(session::Focus::table_cell_identity)
                .map(|cell| composition.next_focus_outside_table(current?, direction, cell.table()))
                .unwrap_or_else(|| composition.next_focus(current, direction))
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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum KeyboardScrollIntent {
    PhysicalBackward,
    PhysicalForward,
    StepBackward,
    StepForward,
    PageBackward,
    PageForward,
    ToStart,
    ToEnd,
}

impl KeyboardScrollIntent {
    fn operation(self, reversed: bool) -> crate::interaction::ScrollOperation {
        use crate::interaction::ScrollOperation;

        match self {
            Self::PhysicalBackward if reversed => ScrollOperation::StepForward,
            Self::PhysicalBackward => ScrollOperation::StepBackward,
            Self::PhysicalForward if reversed => ScrollOperation::StepBackward,
            Self::PhysicalForward => ScrollOperation::StepForward,
            Self::StepBackward => ScrollOperation::StepBackward,
            Self::StepForward => ScrollOperation::StepForward,
            Self::PageBackward => ScrollOperation::PageBackward,
            Self::PageForward => ScrollOperation::PageForward,
            Self::ToStart => ScrollOperation::ToStart,
            Self::ToEnd => ScrollOperation::ToEnd,
        }
    }
}
