use std::collections::HashMap;
use std::time::Instant;

use crate::{command, text, ui, window};

use super::{state::WindowState, text as app_text};

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
    command: text::edit::Command,
) -> command::State {
    let enabled = state
        .composition
        .as_ref()
        .and_then(|composition| composition.text_surface(target))
        .is_some_and(|surface| can_apply_command(state, target, surface, command));

    command::State::available_if(enabled)
}

pub fn can_apply_command(
    state: &WindowState,
    target: &ui::Path,
    surface: &text::Surface,
    command: text::edit::Command,
) -> bool {
    if !is_editing_target(state, target) {
        return false;
    }

    match command {
        text::edit::Command::Undo => surface.is_editable() && state.text.can_undo(target),
        text::edit::Command::Redo => surface.is_editable() && state.text.can_redo(target),
        other => command_would_do_work(surface, other),
    }
}

pub fn publish_command_states(
    state: &WindowState,
    commands: &mut command::Registry,
    window: window::Id,
) -> bool {
    let Some(composition) = state.composition.as_ref() else {
        return false;
    };

    let mut changed = false;

    let target = CommandStateTarget::new(state);

    macro_rules! project_state {
        ($command:ty, $edit:expr, $commands:expr, $target:expr, $context:expr) => {
            changed |= $commands.project_command_state::<$command, _>($target, $context.clone());
        };
    }

    for path in composition.text_surfaces().keys() {
        let context = command::call::Context::path(window, path.clone());

        text::command::for_each_edit_command!(project_state, commands, &target, context);
        changed |= commands
            .project_command_state::<crate::text::command::InsertText, _>(&target, context.clone());
    }

    changed
}

struct CommandStateTarget<'a> {
    state: &'a WindowState,
}

impl<'a> CommandStateTarget<'a> {
    fn new(state: &'a WindowState) -> Self {
        Self { state }
    }
}

impl<C> command::Target<C> for CommandStateTarget<'_>
where
    C: text::command::EditCommand,
{
    fn state(&self, context: &command::call::Context) -> command::State {
        let command::call::Scope::Path(path) = context.scope() else {
            return command::State::unavailable();
        };

        command_state(self.state, path, C::edit_command())
    }

    fn invoke(
        &mut self,
        _args: C::Args,
        _invocation: command::call::Invocation<C>,
    ) -> command::Response<C::Output> {
        command::Response::output(text::edit::CommandResult {
            unavailable: true,
            ..text::edit::CommandResult::default()
        })
    }
}

impl command::Target<text::command::InsertText> for CommandStateTarget<'_> {
    fn state(&self, context: &command::call::Context) -> command::State {
        let command::call::Scope::Path(path) = context.scope() else {
            return command::State::unavailable();
        };

        insert_text_state(self.state, path)
    }

    fn invoke(
        &mut self,
        _args: String,
        _invocation: command::call::Invocation<text::command::InsertText>,
    ) -> command::Response<text::edit::CommandResult> {
        command::Response::output(text::edit::CommandResult {
            unavailable: true,
            ..text::edit::CommandResult::default()
        })
    }
}

pub(crate) struct CommandTarget<'a> {
    window_states: &'a mut HashMap<window::Id, WindowState>,
    text_editor: &'a mut text::edit::Editor,
    text_engine: &'a mut text::layout::Engine,
    clipboard: &'a mut dyn text::edit::Clipboard,
    buffer: &'a mut text::Buffer,
}

impl<'a> CommandTarget<'a> {
    pub(crate) fn new(
        window_states: &'a mut HashMap<window::Id, WindowState>,
        text_editor: &'a mut text::edit::Editor,
        text_engine: &'a mut text::layout::Engine,
        clipboard: &'a mut dyn text::edit::Clipboard,
        buffer: &'a mut text::Buffer,
    ) -> Self {
        Self {
            window_states,
            text_editor,
            text_engine,
            clipboard,
            buffer,
        }
    }
}

impl<C> command::Target<C> for CommandTarget<'_>
where
    C: text::command::EditCommand,
{
    fn state(&self, context: &command::call::Context) -> command::State {
        let command::call::Scope::Path(target) = context.scope() else {
            return command::State::unavailable();
        };

        self.window_states
            .values()
            .find_map(|state| {
                state
                    .composition
                    .as_ref()
                    .and_then(|composition| composition.text_surface(target))
                    .map(|_| command_state(state, target, C::edit_command()))
            })
            .unwrap_or_else(command::State::unavailable)
    }

    fn invoke(
        &mut self,
        _args: C::Args,
        invocation: command::call::Invocation<C>,
    ) -> command::Response<C::Output> {
        let command::call::Scope::Path(target) = invocation.context().scope() else {
            return command::Response::output(text::edit::CommandResult {
                unavailable: true,
                ..text::edit::CommandResult::default()
            });
        };

        command::Response::output(apply_command_for(
            self.window_states,
            self.text_editor,
            self.text_engine,
            self.clipboard,
            target,
            self.buffer,
            C::edit_command(),
        ))
    }
}

impl command::Target<text::command::InsertText> for CommandTarget<'_> {
    fn state(&self, context: &command::call::Context) -> command::State {
        let command::call::Scope::Path(target) = context.scope() else {
            return command::State::unavailable();
        };

        self.window_states
            .values()
            .find_map(|state| {
                state
                    .composition
                    .as_ref()
                    .and_then(|composition| composition.text_surface(target))
                    .map(|_| insert_text_state(state, target))
            })
            .unwrap_or_else(command::State::unavailable)
    }

    fn invoke(
        &mut self,
        text: String,
        invocation: command::call::Invocation<text::command::InsertText>,
    ) -> command::Response<text::edit::CommandResult> {
        let command::call::Scope::Path(target) = invocation.context().scope() else {
            return command::Response::output(text::edit::CommandResult {
                unavailable: true,
                ..text::edit::CommandResult::default()
            });
        };

        command::Response::output(apply_insert_text_for(
            self.window_states,
            self.text_editor,
            self.text_engine,
            target,
            self.buffer,
            text,
        ))
    }
}

fn insert_text_state(state: &WindowState, target: &ui::Path) -> command::State {
    let enabled = state
        .composition
        .as_ref()
        .and_then(|composition| composition.text_surface(target))
        .is_some_and(|surface| {
            is_editing_target(state, target)
                && surface.is_selectable()
                && surface.allows_text_mutation()
        });

    command::State::available_if(enabled)
}

pub(crate) fn apply_command_for(
    window_states: &mut HashMap<window::Id, WindowState>,
    text_editor: &mut text::edit::Editor,
    text_engine: &mut text::layout::Engine,
    clipboard: &mut dyn text::edit::Clipboard,
    target: &ui::Path,
    buffer: &mut text::Buffer,
    command: text::edit::Command,
) -> text::edit::CommandResult {
    if matches!(
        command,
        text::edit::Command::Undo | text::edit::Command::Redo
    ) {
        let scroll_anchors = window_states
            .iter()
            .map(|(window, state)| (*window, state.text_area_scroll_anchor(target)))
            .collect::<HashMap<_, _>>();
        let Some(result) = window_states.values_mut().find_map(|state| {
            let can_apply = state
                .text_surface(target)
                .is_some_and(|surface| can_apply_command(state, target, surface, command));

            can_apply.then(|| state.apply_text_history_command(target, buffer, command))
        }) else {
            return text::edit::CommandResult {
                unavailable: true,
                ..text::edit::CommandResult::default()
            };
        };

        if result.buffer_changed() {
            text_engine.invalidate_text_area_surfaces_for(buffer);
            let now = Instant::now();
            for (window, state) in window_states.iter_mut() {
                state.ensure_text_caret_visible_after_edit(
                    target,
                    now,
                    text_engine,
                    scroll_anchors.get(window).copied().flatten(),
                );
            }
        }

        return result;
    }

    if !window_states.values().any(|state| {
        state
            .text_surface(target)
            .is_some_and(|surface| can_apply_command(state, target, surface, command))
    }) {
        return text::edit::CommandResult {
            unavailable: true,
            ..text::edit::CommandResult::default()
        };
    }

    let selection_only = matches!(command, text::edit::Command::SelectAll);
    let scroll_anchors = window_states
        .iter()
        .map(|(window, state)| (*window, state.text_area_scroll_anchor(target)))
        .collect::<HashMap<_, _>>();
    let outcome = text_editor.apply_text_command_with_result(buffer, command, clipboard);
    if outcome.result.text_changed {
        text_engine.invalidate_text_area_for_edit(buffer, &outcome.impacts);
    }
    let result = outcome.result;

    let now = Instant::now();
    if let Some(change) = outcome.change {
        for state in window_states.values_mut() {
            state.record_text_field_history(
                target,
                change.clone(),
                app_text::HistoryKind::Boundary,
                now,
            );
        }
    }

    if result.buffer_changed() {
        for (window, state) in window_states.iter_mut() {
            if selection_only {
                state.reset_text_field_caret_blink_without_scroll(target, now);
            } else if result.text_changed || result.selection_changed {
                state.ensure_text_caret_visible_after_edit(
                    target,
                    now,
                    text_engine,
                    scroll_anchors.get(window).copied().flatten(),
                );
            } else {
                state.reset_text_field_caret_blink_without_scroll(target, now);
            }
        }
    }

    result
}

pub(crate) fn apply_insert_text_for(
    window_states: &mut HashMap<window::Id, WindowState>,
    text_editor: &mut text::edit::Editor,
    text_engine: &mut text::layout::Engine,
    target: &ui::Path,
    buffer: &mut text::Buffer,
    inserted: String,
) -> text::edit::CommandResult {
    let edit = text::edit::Edit::insert(inserted);
    if !window_states
        .values()
        .any(|state| state.can_apply_text_edit(target, &edit))
    {
        return text::edit::CommandResult {
            unavailable: true,
            ..text::edit::CommandResult::default()
        };
    }

    let scroll_anchors = window_states
        .iter()
        .map(|(window, state)| (*window, state.text_area_scroll_anchor(target)))
        .collect::<HashMap<_, _>>();
    let history_kind = app_text::HistoryKind::for_edit(&edit);
    let result = text_editor.apply_text_edit_with_result(buffer, edit);
    if result.text_changed {
        text_engine.invalidate_text_area_for_edit(buffer, &result.impacts);
    }
    if let Some(change) = result.change.clone() {
        let now = Instant::now();
        for state in window_states.values_mut() {
            state.record_text_field_history(target, change.clone(), history_kind.clone(), now);
        }
    }

    if result.buffer_changed() {
        let now = Instant::now();
        for (window, state) in window_states.iter_mut() {
            state.ensure_text_caret_visible_after_edit(
                target,
                now,
                text_engine,
                scroll_anchors.get(window).copied().flatten(),
            );
        }
    }

    text::edit::CommandResult {
        text_changed: result.text_changed,
        selection_changed: result.selection_changed,
        clipboard_changed: false,
        unavailable: false,
    }
}

fn command_would_do_work(surface: &text::Surface, command: text::edit::Command) -> bool {
    let buffer = surface.buffer();

    if surface.is_disabled() {
        return false;
    }

    match command {
        text::edit::Command::Copy => surface.allows_copy() && buffer.has_selection(),
        text::edit::Command::Cut => surface.allows_cut() && buffer.has_selection(),
        text::edit::Command::Delete | text::edit::Command::Paste => surface.is_editable(),
        text::edit::Command::SelectAll => {
            surface.is_selectable()
                && !buffer.is_empty()
                && buffer
                    .selected_range()
                    .is_none_or(|range| range.start != 0 || range.end != buffer.len())
        }
        text::edit::Command::Undo | text::edit::Command::Redo => false,
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
        .text_surfaces()
        .keys()
        .find(|path| state.focus.restores_to(path) && state.is_selectable_text_field(path))
        .cloned()
}

#[cfg(test)]
mod tests {
    use crate::app::focus;
    use crate::app::state::{Focus, FocusState};
    use crate::geometry::area;
    use crate::widget::menu;
    use crate::{command, ui::layout, widget};

    use super::*;

    const FIELD: ui::Id = ui::Id::new("field");
    const OTHER_FIELD: ui::Id = ui::Id::new("other_field");
    const ROOT: ui::Id = ui::Id::new("root");
    const MENU_BAR: ui::Id = ui::Id::new("menu_bar");
    const MENU_POPUP: ui::Id = ui::Id::new("menu_popup");
    const FILE: menu::Id = menu::Id::new("file");

    fn path(id: ui::Id) -> ui::Path {
        ui::Path::from(id)
    }

    fn child_path(id: ui::Id) -> ui::Path {
        ui::Path::new(vec![ROOT, id])
    }

    fn menu_title_path(index: usize) -> ui::Path {
        ui::Path::new(vec![
            ROOT,
            MENU_BAR,
            ui::Id::structural("menu_title", index),
        ])
    }

    fn window() -> window::Id {
        window::Id::new(1)
    }

    fn buffer_with_partial_selection() -> text::Buffer {
        let mut editor = text::edit::Editor::new();
        let mut buffer = text::Buffer::from_text("hello");

        editor.apply_text_edit(
            &mut buffer,
            text::edit::Edit::set_cursor(text::buffer::Cursor::new(0, 2)),
        );
        editor.apply_text_edit(
            &mut buffer,
            text::edit::Edit::extend_position(text::TextMotion::VisualRight),
        );

        buffer
    }

    fn buffer_with_full_selection() -> text::Buffer {
        let mut editor = text::edit::Editor::new();
        let mut buffer = text::Buffer::from_text("hello");

        editor.apply_text_edit(&mut buffer, text::edit::Edit::SelectAll);

        buffer
    }

    fn state(field: impl Into<text::Field>, focused: bool) -> WindowState {
        let mut tree = ui::Tree::new();
        let mut text_engine = text::layout::Engine::new();

        tree.set_root(
            widget::text_field(field)
                .key(FIELD)
                .with_size(layout::Size::Fixed(120.0), layout::Size::Fixed(32.0)),
        );
        let composition = tree
            .compose(area::logical(120.0, 32.0), &mut text_engine)
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
        let mut text_engine = text::layout::Engine::new();

        tree.set_root(
            ui::Node::container(layout::Axis::Vertical)
                .key(ROOT)
                .with_child(
                    widget::text_field(text::Buffer::from_text("first"))
                        .key(FIELD)
                        .with_size(layout::Size::Fixed(120.0), layout::Size::Fixed(32.0)),
                )
                .with_child(
                    widget::text_field(text::Buffer::from_text("second"))
                        .key(OTHER_FIELD)
                        .with_size(layout::Size::Fixed(120.0), layout::Size::Fixed(32.0)),
                ),
        );
        let composition = tree
            .compose(area::logical(120.0, 64.0), &mut text_engine)
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

    fn two_field_menu_state(focused: ui::Id) -> (WindowState, command::Registry) {
        let mut tree = ui::Tree::new();
        let mut registry = command::Registry::new();
        let mut text_engine = text::layout::Engine::new();

        registry.commands(|commands| {
            crate::text::command::define::<crate::text::command::SelectAll>(commands, |command| {
                command
            });
        });
        tree.set_root(
            ui::Node::container(layout::Axis::Vertical)
                .key(ROOT)
                .with_child(
                    widget::menu_bar(
                        menu::Bar::new().menu(
                            menu::Menu::new("File").key(FILE).section(
                                menu::Section::new()
                                    .item(menu::Item::text::<crate::text::command::SelectAll>()),
                            ),
                        ),
                    )
                    .key(MENU_BAR),
                )
                .with_child(
                    widget::text_field(text::Buffer::from_text("first"))
                        .key(FIELD)
                        .with_size(layout::Size::Fixed(120.0), layout::Size::Fixed(32.0)),
                )
                .with_child(
                    widget::text_field(text::Buffer::from_text("second"))
                        .key(OTHER_FIELD)
                        .with_size(layout::Size::Fixed(120.0), layout::Size::Fixed(32.0)),
                ),
        );
        let composition = tree
            .compose(area::logical(200.0, 96.0), &mut text_engine)
            .expect("two-field menu tree should compose");

        let mut state = WindowState {
            composition: Some(composition),
            focus: FocusState::focused(Focus::new(
                child_path(focused),
                ui::focus::Reason::Keyboard,
                ui::focus::Visibility::Visible,
            )),
            command: crate::app::command::State::with_subject(command::call::Scope::Path(
                child_path(focused),
            )),
            ..WindowState::default()
        };
        sync_session(&mut state);
        publish_command_states(&state, &mut registry, window());

        (state, registry)
    }

    #[test]
    fn no_editing_target_disables_text_commands() {
        let state = state(text::Buffer::from_text("hello"), false);

        for command in [
            text::edit::Command::SelectAll,
            text::edit::Command::Copy,
            text::edit::Command::Cut,
            text::edit::Command::Paste,
            text::edit::Command::Undo,
            text::edit::Command::Redo,
        ] {
            assert!(!command_state(&state, &path(FIELD), command).is_available());
        }
    }

    #[test]
    fn focused_caret_only_field_enables_select_all_and_paste() {
        let state = state(text::Buffer::from_text("hello"), true);

        assert!(command_state(&state, &path(FIELD), text::edit::Command::SelectAll).is_available());
        assert!(command_state(&state, &path(FIELD), text::edit::Command::Paste).is_available());
        assert!(!command_state(&state, &path(FIELD), text::edit::Command::Copy).is_available());
        assert!(!command_state(&state, &path(FIELD), text::edit::Command::Cut).is_available());
    }

    #[test]
    fn read_only_field_enables_selection_commands_but_not_mutation_commands() {
        let state = state(text::Field::new("hello").read_only(), true);

        assert!(command_state(&state, &path(FIELD), text::edit::Command::SelectAll).is_available());
        assert!(!command_state(&state, &path(FIELD), text::edit::Command::Paste).is_available());
        assert!(!command_state(&state, &path(FIELD), text::edit::Command::Cut).is_available());
        assert!(!command_state(&state, &path(FIELD), text::edit::Command::Undo).is_available());
        assert!(!command_state(&state, &path(FIELD), text::edit::Command::Redo).is_available());
    }

    #[test]
    fn selected_read_only_field_enables_copy_only_for_selected_text() {
        let state = state(
            text::Field::new(buffer_with_partial_selection()).read_only(),
            true,
        );

        assert!(command_state(&state, &path(FIELD), text::edit::Command::Copy).is_available());
        assert!(!command_state(&state, &path(FIELD), text::edit::Command::Cut).is_available());
    }

    #[test]
    fn obscured_field_disables_copy_and_cut_without_blocking_edit_commands() {
        let state = state(
            text::Field::new(buffer_with_partial_selection()).obscured_dot(),
            true,
        );

        assert!(!command_state(&state, &path(FIELD), text::edit::Command::Copy).is_available());
        assert!(!command_state(&state, &path(FIELD), text::edit::Command::Cut).is_available());
        assert!(command_state(&state, &path(FIELD), text::edit::Command::Paste).is_available());
        assert!(command_state(&state, &path(FIELD), text::edit::Command::SelectAll).is_available());
    }

    #[test]
    fn disabled_field_disables_all_text_commands() {
        let state = state(
            text::Field::new(buffer_with_partial_selection()).disabled(),
            false,
        );

        for command in [
            text::edit::Command::SelectAll,
            text::edit::Command::Copy,
            text::edit::Command::Cut,
            text::edit::Command::Paste,
            text::edit::Command::Undo,
            text::edit::Command::Redo,
        ] {
            assert!(!command_state(&state, &path(FIELD), command).is_available());
        }
    }

    #[test]
    fn fully_selected_field_disables_select_all() {
        let state = state(buffer_with_full_selection(), true);

        assert!(
            !command_state(&state, &path(FIELD), text::edit::Command::SelectAll).is_available()
        );
        assert!(command_state(&state, &path(FIELD), text::edit::Command::Copy).is_available());
        assert!(command_state(&state, &path(FIELD), text::edit::Command::Cut).is_available());
        assert!(command_state(&state, &path(FIELD), text::edit::Command::Paste).is_available());
    }

    #[test]
    fn partially_selected_field_keeps_select_all_enabled() {
        let state = state(buffer_with_partial_selection(), true);

        assert!(command_state(&state, &path(FIELD), text::edit::Command::SelectAll).is_available());
        assert!(command_state(&state, &path(FIELD), text::edit::Command::Copy).is_available());
        assert!(command_state(&state, &path(FIELD), text::edit::Command::Cut).is_available());
        assert!(command_state(&state, &path(FIELD), text::edit::Command::Paste).is_available());
    }

    #[test]
    fn undo_redo_availability_follows_active_field_history() {
        let mut state = state(text::Buffer::from_text("hello!"), true);
        let mut editor = text::edit::Editor::new();
        let mut before = text::Buffer::from_text("hello");
        let result = editor.apply_text_edit_with_result(&mut before, text::edit::Edit::insert("!"));

        state.record_text_field_history(
            &path(FIELD),
            result.change.expect("insert should change text"),
            app_text::HistoryKind::Typing("!".to_owned()),
            std::time::Instant::now(),
        );

        assert!(command_state(&state, &path(FIELD), text::edit::Command::Undo).is_available());
        assert!(!command_state(&state, &path(FIELD), text::edit::Command::Redo).is_available());

        state.apply_text_history_command(
            &path(FIELD),
            &mut text::Buffer::from_text("hello!"),
            text::edit::Command::Undo,
        );

        assert!(!command_state(&state, &path(FIELD), text::edit::Command::Undo).is_available());
        assert!(command_state(&state, &path(FIELD), text::edit::Command::Redo).is_available());
    }

    #[test]
    fn transient_menu_focus_preserves_editing_target() {
        let state = state_with_open_menu(text::Buffer::from_text("hello"));

        assert_eq!(editing_target(&state), Some(path(FIELD)));
        assert!(command_state(&state, &path(FIELD), text::edit::Command::SelectAll).is_available());
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
        state.command.subject = Some(command::call::Scope::Path(child_path(FIELD)));

        assert_eq!(editing_target(&state), Some(child_path(OTHER_FIELD)));
        assert_eq!(
            state.command_context(window()),
            command::call::Context::path(window(), child_path(OTHER_FIELD))
        );
        assert!(
            !command_state(&state, &child_path(FIELD), text::edit::Command::SelectAll)
                .is_available()
        );
        assert!(
            command_state(
                &state,
                &child_path(OTHER_FIELD),
                text::edit::Command::SelectAll
            )
            .is_available()
        );
    }

    #[test]
    fn closed_menu_title_focus_does_not_leave_stale_text_restore_scope() {
        let (mut state, mut registry) = two_field_menu_state(FIELD);
        let menu_title = menu_title_path(0);

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

        assert!(state.toggle_menu(
            FILE,
            &mut registry,
            window(),
            command::call::Source::Pointer,
        ));
        assert!(state.focus.restores_to(&child_path(OTHER_FIELD)));
        assert!(!state.focus.restores_to(&child_path(FIELD)));
    }

    #[test]
    fn publish_command_states_projects_single_text_policy() {
        let state = state(buffer_with_full_selection(), true);
        let mut registry = command::Registry::new();

        registry.register(command::definition::Definition::for_command::<
            crate::text::command::SelectAll,
            command::TestTarget,
        >());

        assert!(publish_command_states(&state, &mut registry, window()));
        assert!(
            !registry
                .configured_state::<crate::text::command::SelectAll>(command::call::Context::path(
                    window(),
                    path(FIELD),
                ))
                .is_available()
        );
    }
}
