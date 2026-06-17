use crate::{action, text, ui, window};

use super::state::WindowState;

pub fn editing_target(state: &WindowState) -> Option<ui::Path> {
    state
        .composition
        .as_ref()?
        .text_fields()
        .keys()
        .find(|path| is_editing_target(state, path))
        .cloned()
}

pub fn is_editing_target(state: &WindowState, target: &ui::Path) -> bool {
    state.is_selectable_text_field(target)
        && (state.focused_path().as_ref() == Some(target) || state.focus.restores_to(target))
}

pub fn command_state(
    state: &WindowState,
    target: &ui::Path,
    command: text::Command,
) -> action::State {
    let enabled = state
        .composition
        .as_ref()
        .and_then(|composition| composition.text_field(target))
        .is_some_and(|field| can_apply_command(state, target, field, command));

    action::State::new(enabled, false)
}

pub fn can_apply_command(
    state: &WindowState,
    target: &ui::Path,
    field: &text::Field,
    command: text::Command,
) -> bool {
    if !is_editing_target(state, target) {
        return false;
    }

    match command {
        text::Command::Undo => field.is_editable()
            && state
                .text_field_states
                .get(target)
                .is_some_and(text::TextFieldState::can_undo),
        text::Command::Redo => field.is_editable()
            && state
                .text_field_states
                .get(target)
                .is_some_and(text::TextFieldState::can_redo),
        other => command_would_do_work(field, other),
    }
}

pub fn publish_action_states<T>(
    state: &WindowState,
    actions: &mut action::Registry<T>,
    window: window::Id,
) -> bool {
    let Some(composition) = state.composition.as_ref() else {
        return false;
    };

    let mut changed = false;

    for path in composition.text_fields().keys() {
        let context = action::Context::path(window, path.clone());

        for (action, command) in [
            (action::SELECT_ALL, text::Command::SelectAll),
            (action::COPY, text::Command::Copy),
            (action::CUT, text::Command::Cut),
            (action::PASTE, text::Command::Paste),
            (action::UNDO, text::Command::Undo),
            (action::REDO, text::Command::Redo),
        ] {
            changed |=
                actions.set_state(action, context.clone(), command_state(state, path, command));
        }
    }

    changed
}

fn command_would_do_work(field: &text::Field, command: text::Command) -> bool {
    let buffer = field.buffer();

    if field.is_disabled() {
        return false;
    }

    match command {
        text::Command::Copy => field.allows_copy() && buffer.has_selection(),
        text::Command::Cut => field.allows_cut() && buffer.has_selection(),
        text::Command::Paste => field.is_editable(),
        text::Command::SelectAll => {
            field.is_selectable()
                && !buffer.is_empty()
                && buffer
                    .selected_range()
                    .is_none_or(|range| range.start != 0 || range.end != buffer.text().len())
        }
        text::Command::Undo | text::Command::Redo => false,
    }
}

#[cfg(test)]
mod tests {
    use glyphon::cosmic_text::Motion;

    use crate::app::focus;
    use crate::app::state::{Focus, FocusState};
    use crate::geometry::area;
    use crate::{Action, layout, widget};

    use super::*;

    const FIELD: ui::Id = ui::Id::new("field");
    const MENU_POPUP: ui::Id = ui::Id::new("menu_popup");
    const SELECT_ALL: action::Id = action::SELECT_ALL;

    fn path(id: ui::Id) -> ui::Path {
        ui::Path::from(id)
    }

    fn window() -> window::Id {
        window::Id::new(1)
    }

    fn buffer_with_partial_selection() -> text::Buffer {
        let mut engine = text::Engine::new();
        let mut buffer = text::Buffer::from_text("hello");

        engine.apply_text_edit(&mut buffer, text::Edit::set_cursor(text::Cursor::new(0, 2)));
        engine.apply_text_edit(&mut buffer, text::Edit::extend_motion(Motion::Right));

        buffer
    }

    fn buffer_with_full_selection() -> text::Buffer {
        let mut engine = text::Engine::new();
        let mut buffer = text::Buffer::from_text("hello");

        engine.apply_text_edit(&mut buffer, text::Edit::SelectAll);

        buffer
    }

    fn state(field: impl Into<text::Field>, focused: bool) -> WindowState {
        let mut tree = ui::Tree::new();
        let mut registry = action::Registry::<()>::new();
        let mut text_engine = text::Engine::new();

        tree.set_root(
            widget::text_field(FIELD, field)
                .with_size(layout::Size::Fixed(120.0), layout::Size::Fixed(32.0)),
        );
        let composition = tree
            .compose(
                window(),
                area::logical(120.0, 32.0),
                &mut registry,
                &action::Context::window(window()),
                None,
                None,
                &mut text_engine,
            )
            .expect("text field tree should compose");

        WindowState {
            composition: Some(composition),
            focus: if focused {
                FocusState::focused(Focus::new(
                    path(FIELD),
                    ui::focus::Reason::Keyboard,
                    ui::focus::Visibility::Visible,
                ))
            } else {
                FocusState::default()
            },
            ..WindowState::default()
        }
    }

    fn state_with_open_menu(buffer: text::Buffer) -> WindowState {
        let mut state = state(buffer, true);

        state
            .focus
            .begin_transient(focus::TransientScope::Menu, path(MENU_POPUP));
        state.focus.set(Focus::new(
            path(MENU_POPUP),
            ui::focus::Reason::Keyboard,
            ui::focus::Visibility::Visible,
        ));

        state
    }

    #[test]
    fn no_editing_target_disables_text_commands() {
        let state = state(text::Buffer::from_text("hello"), false);

        for command in [
            text::Command::SelectAll,
            text::Command::Copy,
            text::Command::Cut,
            text::Command::Paste,
            text::Command::Undo,
            text::Command::Redo,
        ] {
            assert!(!command_state(&state, &path(FIELD), command).is_enabled());
        }
    }

    #[test]
    fn focused_caret_only_field_enables_select_all_and_paste() {
        let state = state(text::Buffer::from_text("hello"), true);

        assert!(command_state(&state, &path(FIELD), text::Command::SelectAll).is_enabled());
        assert!(command_state(&state, &path(FIELD), text::Command::Paste).is_enabled());
        assert!(!command_state(&state, &path(FIELD), text::Command::Copy).is_enabled());
        assert!(!command_state(&state, &path(FIELD), text::Command::Cut).is_enabled());
    }

    #[test]
    fn read_only_field_enables_selection_commands_but_not_mutation_commands() {
        let state = state(text::Field::new("hello").read_only(), true);

        assert!(command_state(&state, &path(FIELD), text::Command::SelectAll).is_enabled());
        assert!(!command_state(&state, &path(FIELD), text::Command::Paste).is_enabled());
        assert!(!command_state(&state, &path(FIELD), text::Command::Cut).is_enabled());
        assert!(!command_state(&state, &path(FIELD), text::Command::Undo).is_enabled());
        assert!(!command_state(&state, &path(FIELD), text::Command::Redo).is_enabled());
    }

    #[test]
    fn selected_read_only_field_enables_copy_only_for_selected_text() {
        let state = state(text::Field::new(buffer_with_partial_selection()).read_only(), true);

        assert!(command_state(&state, &path(FIELD), text::Command::Copy).is_enabled());
        assert!(!command_state(&state, &path(FIELD), text::Command::Cut).is_enabled());
    }

    #[test]
    fn obscured_field_disables_copy_and_cut_without_blocking_edit_commands() {
        let state = state(
            text::Field::new(buffer_with_partial_selection()).obscured_dot(),
            true,
        );

        assert!(!command_state(&state, &path(FIELD), text::Command::Copy).is_enabled());
        assert!(!command_state(&state, &path(FIELD), text::Command::Cut).is_enabled());
        assert!(command_state(&state, &path(FIELD), text::Command::Paste).is_enabled());
        assert!(command_state(&state, &path(FIELD), text::Command::SelectAll).is_enabled());
    }

    #[test]
    fn disabled_field_disables_all_text_commands() {
        let state = state(text::Field::new(buffer_with_partial_selection()).disabled(), false);

        for command in [
            text::Command::SelectAll,
            text::Command::Copy,
            text::Command::Cut,
            text::Command::Paste,
            text::Command::Undo,
            text::Command::Redo,
        ] {
            assert!(!command_state(&state, &path(FIELD), command).is_enabled());
        }
    }

    #[test]
    fn fully_selected_field_disables_select_all() {
        let state = state(buffer_with_full_selection(), true);

        assert!(!command_state(&state, &path(FIELD), text::Command::SelectAll).is_enabled());
        assert!(command_state(&state, &path(FIELD), text::Command::Copy).is_enabled());
        assert!(command_state(&state, &path(FIELD), text::Command::Cut).is_enabled());
        assert!(command_state(&state, &path(FIELD), text::Command::Paste).is_enabled());
    }

    #[test]
    fn partially_selected_field_keeps_select_all_enabled() {
        let state = state(buffer_with_partial_selection(), true);

        assert!(command_state(&state, &path(FIELD), text::Command::SelectAll).is_enabled());
        assert!(command_state(&state, &path(FIELD), text::Command::Copy).is_enabled());
        assert!(command_state(&state, &path(FIELD), text::Command::Cut).is_enabled());
        assert!(command_state(&state, &path(FIELD), text::Command::Paste).is_enabled());
    }

    #[test]
    fn undo_redo_availability_follows_active_field_history() {
        let mut state = state(text::Buffer::from_text("hello!"), true);
        let mut engine = text::Engine::new();
        let mut before = text::Buffer::from_text("hello");
        let result = engine.apply_text_edit_with_result(&mut before, text::Edit::insert("!"));

        state.record_text_field_history(
            &path(FIELD),
            result.change.expect("insert should change text"),
            text::HistoryKind::Typing,
        );

        assert!(command_state(&state, &path(FIELD), text::Command::Undo).is_enabled());
        assert!(!command_state(&state, &path(FIELD), text::Command::Redo).is_enabled());

        state.apply_text_history_command(
            &path(FIELD),
            &mut text::Buffer::from_text("hello!"),
            text::Command::Undo,
        );

        assert!(!command_state(&state, &path(FIELD), text::Command::Undo).is_enabled());
        assert!(command_state(&state, &path(FIELD), text::Command::Redo).is_enabled());
    }

    #[test]
    fn transient_menu_focus_preserves_editing_target() {
        let state = state_with_open_menu(text::Buffer::from_text("hello"));

        assert_eq!(editing_target(&state), Some(path(FIELD)));
        assert!(command_state(&state, &path(FIELD), text::Command::SelectAll).is_enabled());
    }

    #[test]
    fn publish_action_states_projects_single_text_policy() {
        let state = state(buffer_with_full_selection(), true);
        let mut registry = action::Registry::<()>::new();

        registry.register(Action::new(SELECT_ALL, "Select All"));

        assert!(publish_action_states(&state, &mut registry, window()));
        assert!(
            !registry
                .configured_state(SELECT_ALL, action::Context::path(window(), path(FIELD)))
                .is_enabled()
        );
    }
}
