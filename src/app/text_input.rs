use crate::{action, text, ui, window};

use super::state::WindowState;

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Session {
    target: Option<ui::Path>,
}

impl Session {
    pub fn target(&self) -> Option<&ui::Path> {
        self.target.as_ref()
    }

    fn set_target(&mut self, target: Option<ui::Path>) -> bool {
        if self.target == target {
            return false;
        }

        self.target = target;
        true
    }
}

pub fn sync_session(state: &mut WindowState) -> bool {
    let target = resolve_session_target(state);

    state.text_input_session.set_target(target)
}

pub fn editing_target(state: &WindowState) -> Option<ui::Path> {
    state
        .text_input_session
        .target()
        .filter(|target| state.is_selectable_text_field(target))
        .cloned()
}

pub fn is_editing_target(state: &WindowState, target: &ui::Path) -> bool {
    state.text_input_session.target() == Some(target) && state.is_selectable_text_field(target)
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
        text::Command::Undo => {
            field.is_editable()
                && state
                    .text_field_states
                    .get(target)
                    .is_some_and(text::TextFieldState::can_undo)
        }
        text::Command::Redo => {
            field.is_editable()
                && state
                    .text_field_states
                    .get(target)
                    .is_some_and(text::TextFieldState::can_redo)
        }
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

pub fn command_for_action(action: action::Id) -> Option<text::Command> {
    match action {
        action::SELECT_ALL => Some(text::Command::SelectAll),
        action::COPY => Some(text::Command::Copy),
        action::CUT => Some(text::Command::Cut),
        action::PASTE => Some(text::Command::Paste),
        action::UNDO => Some(text::Command::Undo),
        action::REDO => Some(text::Command::Redo),
        _ => None,
    }
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

fn resolve_session_target(state: &WindowState) -> Option<ui::Path> {
    if let Some(path) = state
        .focused_path()
        .filter(|path| state.is_selectable_text_field(path))
    {
        return Some(path);
    }

    if let Some(target) = state
        .text_input_session
        .target()
        .filter(|target| state.is_selectable_text_field(target))
        && state
            .focused_path()
            .is_some_and(|path| state.focus_preserves_text_input_session(&path))
    {
        return Some(target.clone());
    }

    state
        .composition
        .as_ref()?
        .text_fields()
        .keys()
        .find(|path| state.focus.restores_to(path) && state.is_selectable_text_field(path))
        .cloned()
}

#[cfg(test)]
mod tests {
    use glyphon::cosmic_text::Motion;

    use crate::app::focus;
    use crate::app::state::{Focus, FocusState};
    use crate::geometry::area;
    use crate::widget::menu;
    use crate::{Action, layout, widget};

    use super::*;

    const FIELD: ui::Id = ui::Id::new("field");
    const OTHER_FIELD: ui::Id = ui::Id::new("other_field");
    const ROOT: ui::Id = ui::Id::new("root");
    const MENU_BAR: ui::Id = ui::Id::new("menu_bar");
    const MENU_POPUP: ui::Id = ui::Id::new("menu_popup");
    const FILE: menu::Id = menu::Id::new("file");
    const SELECT_ALL: action::Id = action::SELECT_ALL;

    fn path(id: ui::Id) -> ui::Path {
        ui::Path::from(id)
    }

    fn child_path(id: ui::Id) -> ui::Path {
        ui::Path::new(vec![ROOT, id])
    }

    fn menu_title_path(menu: menu::Id) -> ui::Path {
        ui::Path::new(vec![ROOT, MENU_BAR, ui::Id::new(menu.as_str())])
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
                &[],
                &mut text_engine,
            )
            .expect("text field tree should compose");

        let mut state = WindowState {
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
        };
        sync_session(&mut state);
        state
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
        sync_session(&mut state);

        state
    }

    fn two_field_state(focused: ui::Id) -> WindowState {
        let mut tree = ui::Tree::new();
        let mut registry = action::Registry::<()>::new();
        let mut text_engine = text::Engine::new();

        tree.set_root(
            ui::Node::container(ROOT, layout::Axis::Vertical)
                .with_child(
                    widget::text_field(FIELD, text::Buffer::from_text("first"))
                        .with_size(layout::Size::Fixed(120.0), layout::Size::Fixed(32.0)),
                )
                .with_child(
                    widget::text_field(OTHER_FIELD, text::Buffer::from_text("second"))
                        .with_size(layout::Size::Fixed(120.0), layout::Size::Fixed(32.0)),
                ),
        );
        let composition = tree
            .compose(
                window(),
                area::logical(120.0, 64.0),
                &mut registry,
                &[],
                &mut text_engine,
            )
            .expect("two-field tree should compose");

        let mut state = WindowState {
            composition: Some(composition),
            focus: FocusState::focused(Focus::new(
                child_path(focused),
                ui::focus::Reason::Keyboard,
                ui::focus::Visibility::Visible,
            )),
            ..WindowState::default()
        };
        sync_session(&mut state);
        state
    }

    fn two_field_menu_state(focused: ui::Id) -> (WindowState, action::Registry<()>) {
        let mut tree = ui::Tree::new();
        let mut registry = action::Registry::<()>::new();
        let mut text_engine = text::Engine::new();

        registry.register(Action::new(SELECT_ALL, "Select All"));
        tree.set_root(
            ui::Node::container(ROOT, layout::Axis::Vertical)
                .with_child(widget::menu_bar(
                    MENU_BAR,
                    menu::Bar::new().menu(
                        menu::Menu::new(FILE, "File")
                            .section(menu::Section::new().item(menu::Item::new(SELECT_ALL))),
                    ),
                ))
                .with_child(
                    widget::text_field(FIELD, text::Buffer::from_text("first"))
                        .with_size(layout::Size::Fixed(120.0), layout::Size::Fixed(32.0)),
                )
                .with_child(
                    widget::text_field(OTHER_FIELD, text::Buffer::from_text("second"))
                        .with_size(layout::Size::Fixed(120.0), layout::Size::Fixed(32.0)),
                ),
        );
        let composition = tree
            .compose(
                window(),
                area::logical(200.0, 96.0),
                &mut registry,
                &[],
                &mut text_engine,
            )
            .expect("two-field menu tree should compose");

        let mut state = WindowState {
            composition: Some(composition),
            focus: FocusState::focused(Focus::new(
                child_path(focused),
                ui::focus::Reason::Keyboard,
                ui::focus::Visibility::Visible,
            )),
            command_subject: Some(action::Scope::Path(child_path(focused))),
            ..WindowState::default()
        };
        sync_session(&mut state);
        publish_action_states(&state, &mut registry, window());

        (state, registry)
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
        let state = state(
            text::Field::new(buffer_with_partial_selection()).read_only(),
            true,
        );

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
        let state = state(
            text::Field::new(buffer_with_partial_selection()).disabled(),
            false,
        );

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
    fn focus_move_changes_session_target() {
        let mut state = two_field_state(FIELD);

        assert_eq!(editing_target(&state), Some(child_path(FIELD)));

        assert!(state.set_focus(
            child_path(OTHER_FIELD),
            ui::focus::Reason::Keyboard,
            ui::focus::Visibility::Visible,
        ));

        assert_eq!(editing_target(&state), Some(child_path(OTHER_FIELD)));
    }

    #[test]
    fn stale_command_subject_cannot_override_text_session() {
        let mut state = two_field_state(OTHER_FIELD);
        state.command_subject = Some(action::Scope::Path(child_path(FIELD)));

        assert_eq!(editing_target(&state), Some(child_path(OTHER_FIELD)));
        assert_eq!(
            state.command_context(window()),
            action::Context::path(window(), child_path(OTHER_FIELD))
        );
        assert!(!command_state(&state, &child_path(FIELD), text::Command::SelectAll).is_enabled());
        assert!(
            command_state(&state, &child_path(OTHER_FIELD), text::Command::SelectAll).is_enabled()
        );
    }

    #[test]
    fn closed_menu_title_focus_does_not_leave_stale_text_restore_scope() {
        let (mut state, registry) = two_field_menu_state(FIELD);
        let menu_title = menu_title_path(FILE);

        assert!(state.set_focus(
            menu_title.clone(),
            ui::focus::Reason::Keyboard,
            ui::focus::Visibility::Visible,
        ));
        assert_eq!(editing_target(&state), Some(child_path(FIELD)));
        assert!(!state.focus.restores_to(&child_path(FIELD)));

        assert!(state.set_focus(
            child_path(OTHER_FIELD),
            ui::focus::Reason::Keyboard,
            ui::focus::Visibility::Visible,
        ));
        assert_eq!(editing_target(&state), Some(child_path(OTHER_FIELD)));

        assert!(state.set_focus(
            menu_title,
            ui::focus::Reason::Keyboard,
            ui::focus::Visibility::Visible,
        ));
        assert_eq!(editing_target(&state), Some(child_path(OTHER_FIELD)));

        assert!(state.toggle_menu(FILE, &registry, window(), action::Source::Pointer,));
        assert!(state.focus.restores_to(&child_path(OTHER_FIELD)));
        assert!(!state.focus.restores_to(&child_path(FIELD)));
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
