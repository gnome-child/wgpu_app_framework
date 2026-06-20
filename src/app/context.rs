use std::time::Instant;

use winit::event_loop::ActiveEventLoop;

use crate::app::mailbox::Mailbox;
use crate::app::rendering;
use crate::app::sender::Sender;
use crate::app::state::WindowState;
use crate::app::task_runner;
use crate::app::text_input;
use crate::app::windows::Windows;
use crate::geometry::area;
use crate::{Action, Task, action, text, ui, window};

use super::Result;

pub struct Context<'a, T: Send + 'static> {
    rendering: &'a mut rendering::Driver,
    windows: &'a mut Windows,
    window_states: &'a mut std::collections::HashMap<window::Id, WindowState>,
    actions: &'a mut action::Registry<T>,
    text_engine: &'a mut text::Engine,
    clipboard: &'a mut dyn text::Clipboard,
    mailbox: &'a mut Mailbox<T>,
    sender: Sender<T>,
    redraw_on_action_state_change: bool,
    event_loop: &'a ActiveEventLoop,
}

pub struct Parts<'a, T: Send + 'static> {
    pub rendering: &'a mut rendering::Driver,
    pub windows: &'a mut Windows,
    pub window_states: &'a mut std::collections::HashMap<window::Id, WindowState>,
    pub actions: &'a mut action::Registry<T>,
    pub text_engine: &'a mut text::Engine,
    pub clipboard: &'a mut dyn text::Clipboard,
    pub mailbox: &'a mut Mailbox<T>,
    pub sender: Sender<T>,
    pub redraw_on_action_state_change: bool,
    pub event_loop: &'a ActiveEventLoop,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct Diagnostics {
    pub text: text::Diagnostics,
    pub scroll: ScrollDiagnostics,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct ScrollDiagnostics {
    pub generic_scroll_projections: usize,
    pub text_area_surfaces: usize,
    pub text_area_targets: usize,
    pub text_area_skipped_by_filter: usize,
    pub text_area_resolves: usize,
    pub text_area_model_reuses: usize,
    pub text_area_model_updates: usize,
    pub text_area_idle_refinements: usize,
    pub projection_count: usize,
}

impl ScrollDiagnostics {
    fn from_scroll(value: crate::app::scroll::Diagnostics) -> Self {
        Self {
            generic_scroll_projections: value.generic_scroll_projections,
            text_area_surfaces: value.text_area_surfaces,
            text_area_targets: value.text_area_targets,
            text_area_skipped_by_filter: value.text_area_skipped_by_filter,
            text_area_resolves: value.text_area_resolves,
            text_area_model_reuses: value.text_area_model_reuses,
            text_area_model_updates: value.text_area_model_updates,
            text_area_idle_refinements: value.text_area_idle_refinements,
            projection_count: value.projection_count,
        }
    }
}

pub fn new<T: Send + 'static>(parts: Parts<'_, T>) -> Context<'_, T> {
    Context {
        rendering: parts.rendering,
        windows: parts.windows,
        window_states: parts.window_states,
        actions: parts.actions,
        text_engine: parts.text_engine,
        clipboard: parts.clipboard,
        mailbox: parts.mailbox,
        sender: parts.sender,
        redraw_on_action_state_change: parts.redraw_on_action_state_change,
        event_loop: parts.event_loop,
    }
}

impl<T: Send + 'static> Context<'_, T> {
    pub fn open_window(&mut self, options: window::Options) -> window::Id {
        self.try_open_window(options)
            .expect("failed to open framework window")
    }

    pub fn try_open_window(&mut self, options: window::Options) -> Result<window::Id> {
        let id = self
            .windows
            .open(options, self.rendering, self.event_loop)?;

        self.window_states.entry(id).or_default();

        Ok(id)
    }

    pub fn request_redraw(&self, window: window::Id) {
        self.windows.request_redraw(window);
    }

    pub fn close_window(&mut self, window: window::Id) {
        self.windows.remove(window);
        self.window_states.remove(&window);

        if self.windows.is_empty() {
            self.event_loop.exit();
        }
    }

    pub fn window_logical_area(&self, window: window::Id) -> Option<area::Logical> {
        self.windows
            .get(window)
            .map(|window| window.canvas().logical_area())
    }

    pub fn window_physical_area(&self, window: window::Id) -> Option<area::Physical> {
        self.windows.get(window).map(|window| window.inner_area())
    }

    pub fn register_action(&mut self, action: Action<T>) {
        self.actions.register(action);
    }

    pub fn set_action_state(
        &mut self,
        action: action::Id,
        context: action::Context,
        state: action::State,
    ) {
        let window = context.window_id();

        if self.actions.set_state(action, context, state) && self.redraw_on_action_state_change {
            self.request_redraw(window);
        }
    }

    pub fn action(&mut self, window: window::Id, action: action::Id) -> ActionState<'_, T> {
        ActionState::new(
            self.actions,
            self.windows,
            window,
            action,
            self.redraw_on_action_state_change,
        )
    }

    pub fn action_state(&self, action: action::Id, context: action::Context) -> action::State {
        self.actions.state(action, context)
    }

    pub fn diagnostics(&self, window: window::Id) -> Diagnostics {
        Diagnostics {
            text: self.text_engine.diagnostics(),
            scroll: self
                .window_states
                .get(&window)
                .map(|state| ScrollDiagnostics::from_scroll(state.scroll.diagnostics()))
                .unwrap_or_default(),
        }
    }

    pub fn emit(&mut self, event: T) {
        self.mailbox.push_app(event);
    }

    pub fn spawn(&self, task: Task<T>) {
        task_runner::spawn(task, self.sender.clone());
    }

    pub fn apply_text_edit(&mut self, buffer: &mut text::Buffer, edit: text::Edit) -> bool {
        self.text_engine.apply_text_edit(buffer, edit)
    }

    pub fn apply_text_edit_for(
        &mut self,
        target: &ui::Path,
        buffer: &mut text::Buffer,
        edit: text::Edit,
    ) -> bool {
        if !self
            .window_states
            .values()
            .any(|state| state.can_apply_text_edit(target, &edit))
        {
            return false;
        }

        let history_kind = edit.history_kind();
        let reveal_caret = text_edit_reveals_caret(&edit);
        let now = Instant::now();
        let result = self.text_engine.apply_text_edit_with_result(buffer, edit);

        if let Some(change) = result.change.clone() {
            for state in self.window_states.values_mut() {
                state.record_text_field_history(target, change.clone(), history_kind.clone(), now);
            }
        }

        if result.buffer_changed() {
            for state in self.window_states.values_mut() {
                if reveal_caret {
                    state.reset_text_field_caret_blink_if_needed(target, now);
                    state.reveal_text_field_caret(target, &mut self.text_engine);
                } else {
                    state.reset_text_field_caret_blink_without_reveal(target, now);
                }
            }
        }

        result.buffer_changed()
    }

    pub fn apply_text_command_for(
        &mut self,
        target: &ui::Path,
        buffer: &mut text::Buffer,
        command: text::Command,
    ) -> text::CommandResult {
        if matches!(command, text::Command::Undo | text::Command::Redo) {
            let Some(result) = self.window_states.values_mut().find_map(|state| {
                let can_apply = state.text_surface(target).is_some_and(|surface| {
                    text_input::can_apply_command(state, target, surface, command.clone())
                });

                can_apply.then(|| state.apply_text_history_command(target, buffer, command.clone()))
            }) else {
                return text::CommandResult {
                    unavailable: true,
                    ..text::CommandResult::default()
                };
            };

            if result.buffer_changed() {
                self.text_engine.invalidate_text_area_surfaces_for(buffer);
                let now = Instant::now();
                for state in self.window_states.values_mut() {
                    state.reset_text_field_caret_blink_if_needed(target, now);
                    state.reveal_text_field_caret(target, &mut self.text_engine);
                }
            }

            return result;
        }

        if !self.window_states.values().any(|state| {
            state.text_surface(target).is_some_and(|surface| {
                text_input::can_apply_command(state, target, surface, command.clone())
            })
        }) {
            return text::CommandResult {
                unavailable: true,
                ..text::CommandResult::default()
            };
        }

        let reveal_caret = text_command_reveals_caret(command);
        let outcome =
            self.text_engine
                .apply_text_command_with_result(buffer, command, self.clipboard);
        let result = outcome.result;

        let now = Instant::now();
        if let Some(change) = outcome.change {
            for state in self.window_states.values_mut() {
                state.record_text_field_history(
                    target,
                    change.clone(),
                    text::HistoryKind::Boundary,
                    now,
                );
            }
        }

        if result.buffer_changed() {
            for state in self.window_states.values_mut() {
                if reveal_caret {
                    state.reset_text_field_caret_blink_if_needed(target, now);
                    state.reveal_text_field_caret(target, &mut self.text_engine);
                } else {
                    state.reset_text_field_caret_blink_without_reveal(target, now);
                }
            }
        }

        result
    }

    pub fn sender(&self) -> Sender<T> {
        self.sender.clone()
    }

    pub fn invoke_action(&mut self, action: action::Id, context: action::Context) {
        self.mailbox.run_action(action::Request::new(
            action,
            action::Source::Programmatic,
            context,
        ));
    }

    pub fn command_subject(&self, window: window::Id) -> action::Context {
        self.window_states
            .get(&window)
            .map(|state| state.command_context(window))
            .unwrap_or_else(|| action::Context::window(window))
    }

    pub fn set_command_subject(&mut self, window: window::Id, context: action::Context) {
        if context.window_id() != window {
            return;
        }

        let Some(state) = self.window_states.get_mut(&window) else {
            return;
        };

        if state.set_command_subject(context) {
            self.request_redraw(window);
        }
    }

    pub fn clear_command_subject(&mut self, window: window::Id) {
        let Some(state) = self.window_states.get_mut(&window) else {
            return;
        };

        if state.clear_command_subject() {
            self.request_redraw(window);
        }
    }

    pub fn hovered(&self, window: window::Id) -> Option<ui::Path> {
        self.window_states
            .get(&window)
            .and_then(|state| state.hovered.clone())
    }

    pub fn focused(&self, window: window::Id) -> Option<ui::Path> {
        self.window_states
            .get(&window)
            .and_then(|state| state.focused_path())
    }

    pub fn focus(&mut self, window: window::Id, path: ui::Path, visibility: ui::focus::Visibility) {
        let Some(state) = self.window_states.get_mut(&window) else {
            return;
        };

        if state.set_focus(path, ui::focus::Reason::Programmatic, visibility) {
            self.request_redraw(window);
        }
    }

    pub fn clear_focus(&mut self, window: window::Id) {
        let Some(state) = self.window_states.get_mut(&window) else {
            return;
        };

        if state.clear_focus() {
            self.request_redraw(window);
        }
    }

    pub fn resolve_action_context(
        &self,
        window: window::Id,
        requested_scope: Option<action::Scope>,
    ) -> action::Context {
        if let Some(scope) = requested_scope {
            return action::Context::with_scope(window, scope);
        }

        self.command_subject(window)
    }
}

pub struct ActionState<'a, T> {
    actions: &'a mut action::Registry<T>,
    windows: &'a Windows,
    window: window::Id,
    action: action::Id,
    state: action::State,
    changed: bool,
    redraw_on_action_state_change: bool,
}

impl<'a, T> ActionState<'a, T> {
    fn new(
        actions: &'a mut action::Registry<T>,
        windows: &'a Windows,
        window: window::Id,
        action: action::Id,
        redraw_on_action_state_change: bool,
    ) -> Self {
        let state = actions.configured_state(action, action::Context::window(window));

        Self {
            actions,
            windows,
            window,
            action,
            state,
            changed: false,
            redraw_on_action_state_change,
        }
    }

    pub fn enabled(mut self, enabled: bool) -> Self {
        self.state = self.state.with_enabled(enabled);
        self.changed = true;
        self
    }

    pub fn active(mut self, active: bool) -> Self {
        self.state = self.state.with_active(active);
        self.changed = true;
        self
    }

    pub fn busy(mut self, busy: bool) -> Self {
        self.state = self.state.with_busy(busy);
        self.changed = true;
        self
    }
}

impl<T> Drop for ActionState<'_, T> {
    fn drop(&mut self) {
        if !self.changed {
            return;
        }

        let changed = self.actions.set_state(
            self.action,
            action::Context::window(self.window),
            self.state,
        );

        if changed && self.redraw_on_action_state_change {
            self.windows.request_redraw(self.window);
        }
    }
}

fn text_edit_reveals_caret(edit: &text::Edit) -> bool {
    matches!(
        edit,
        text::Edit::Insert(_)
            | text::Edit::ImeCommit(_)
            | text::Edit::ReplaceRange { .. }
            | text::Edit::MoveRange { .. }
            | text::Edit::Backspace
            | text::Edit::Delete
            | text::Edit::InsertLineBreak
            | text::Edit::MovePosition(_)
            | text::Edit::ExtendPosition(_)
            | text::Edit::DeleteWordBackward
            | text::Edit::DeleteWordForward
            | text::Edit::SetPosition(_)
    )
}

fn text_command_reveals_caret(command: text::Command) -> bool {
    matches!(
        command,
        text::Command::Cut | text::Command::Paste | text::Command::Undo | text::Command::Redo
    )
}
