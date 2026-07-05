use std::time::Instant;

use winit::event_loop::ActiveEventLoop;

use crate::app::mailbox::Mailbox;
use crate::app::rendering;
use crate::app::sender::Sender;
use crate::app::state::WindowState;
use crate::app::task_runner;
use crate::app::text as app_text;
use crate::app::text_input;
use crate::app::windows::Windows;
use crate::geometry::area;
use crate::{Command, Task, command, text, ui, window};

use super::{Result, frame};

pub struct Context<'a, T: Send + 'static> {
    rendering: &'a mut rendering::Driver,
    windows: &'a mut Windows,
    window_states: &'a mut std::collections::HashMap<window::Id, WindowState>,
    commands: &'a mut command::Registry,
    text_editor: &'a mut text::edit::Editor,
    text_engine: &'a mut text::layout::Engine,
    clipboard: &'a mut dyn text::edit::Clipboard,
    mailbox: &'a mut Mailbox<T>,
    sender: Sender<T>,
    redraw_on_command_state_change: bool,
    event_loop: &'a ActiveEventLoop,
}

pub struct Parts<'a, T: Send + 'static> {
    pub rendering: &'a mut rendering::Driver,
    pub windows: &'a mut Windows,
    pub window_states: &'a mut std::collections::HashMap<window::Id, WindowState>,
    pub commands: &'a mut command::Registry,
    pub text_editor: &'a mut text::edit::Editor,
    pub text_engine: &'a mut text::layout::Engine,
    pub clipboard: &'a mut dyn text::edit::Clipboard,
    pub mailbox: &'a mut Mailbox<T>,
    pub sender: Sender<T>,
    pub redraw_on_command_state_change: bool,
    pub event_loop: &'a ActiveEventLoop,
}

pub struct CommandDispatch<'a, T: Send + 'static> {
    cx: Context<'a, T>,
    call: Option<command::call::Any>,
    handled: bool,
    effects: Vec<QueuedCommandEffect>,
    projection_window: Option<window::Id>,
    changed: bool,
}

pub(crate) struct QueuedCommandEffect {
    pub(crate) command: command::Key,
    pub(crate) context: command::call::Context,
    pub(crate) effect: command::Effect,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TextCommandOutcome {
    label: &'static str,
    result: text::edit::ActionResult,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct Diagnostics {
    pub text: text::layout::Diagnostics,
    pub edit: text::edit::Diagnostics,
    pub scroll: ScrollDiagnostics,
    pub frame: frame::Diagnostics,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct ScrollDiagnostics {
    pub wheel_events: usize,
    pub thumb_drag_moves: usize,
    pub scroll_offset_changes: usize,
    pub scroll_redraw_requests: usize,
    pub queued_scroll_updates: usize,
    pub pending_scroll_updates: usize,
    pub pending_scroll_applications: usize,
    pub frame_scroll_commits: usize,
    pub generic_scroll_projections: usize,
    pub text_area_surfaces: usize,
    pub text_area_targets: usize,
    pub text_area_skipped_by_filter: usize,
    pub text_area_resolves: usize,
    pub text_area_projection_reuses: usize,
    pub text_area_projection_shifts: usize,
    pub text_area_projection_shift_misses: usize,
    pub text_area_projection_cold_jumps: usize,
    pub text_area_model_reuses: usize,
    pub text_area_model_updates: usize,
    pub text_area_idle_refinements: usize,
    pub text_area_idle_refinements_suppressed: usize,
    pub async_scroll_projection_sync_skips: usize,
    pub async_scroll_reconciles: usize,
    pub retained_scroll_translations: usize,
    pub retained_scroll_translated_items: usize,
    pub retained_scroll_chrome_repaints: usize,
    pub retained_scroll_target_repaint_fallbacks: usize,
    pub retained_scroll_layer_hits: usize,
    pub retained_scroll_layer_replaced_items: usize,
    pub retained_scroll_layer_text_prepare_skips: usize,
    pub retained_scroll_layer_missing: usize,
    pub retained_scroll_layer_metrics_misses: usize,
    pub retained_scroll_layer_coverage_misses: usize,
    pub retained_scroll_layer_geometry_misses: usize,
    pub retained_scroll_layer_projection_misses: usize,
    pub retained_scroll_layer_rebuilds: usize,
    pub projection_count: usize,
    pub last_scroll: LastScrollDiagnostics,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct LastScrollDiagnostics {
    pub wheel_events: usize,
    pub thumb_drag_moves: usize,
    pub scroll_offset_changes: usize,
    pub retained_scroll_layer_hits: usize,
    pub retained_scroll_layer_text_prepare_skips: usize,
    pub retained_scroll_target_repaint_fallbacks: usize,
    pub retained_scroll_layer_missing: usize,
    pub retained_scroll_layer_metrics_misses: usize,
    pub retained_scroll_layer_coverage_misses: usize,
    pub retained_scroll_layer_geometry_misses: usize,
    pub retained_scroll_layer_projection_misses: usize,
    pub retained_scroll_layer_rebuilds: usize,
}

impl ScrollDiagnostics {
    fn from_scroll(value: crate::app::scroll::Diagnostics) -> Self {
        Self {
            wheel_events: value.wheel_events,
            thumb_drag_moves: value.thumb_drag_moves,
            scroll_offset_changes: value.scroll_offset_changes,
            scroll_redraw_requests: value.scroll_redraw_requests,
            queued_scroll_updates: value.queued_scroll_updates,
            pending_scroll_updates: value.pending_scroll_updates,
            pending_scroll_applications: value.pending_scroll_applications,
            frame_scroll_commits: value.frame_scroll_commits,
            generic_scroll_projections: value.generic_scroll_projections,
            text_area_surfaces: value.text_area_surfaces,
            text_area_targets: value.text_area_targets,
            text_area_skipped_by_filter: value.text_area_skipped_by_filter,
            text_area_resolves: value.text_area_resolves,
            text_area_projection_reuses: value.text_area_projection_reuses,
            text_area_projection_shifts: value.text_area_projection_shifts,
            text_area_projection_shift_misses: value.text_area_projection_shift_misses,
            text_area_projection_cold_jumps: value.text_area_projection_cold_jumps,
            text_area_model_reuses: value.text_area_model_reuses,
            text_area_model_updates: value.text_area_model_updates,
            text_area_idle_refinements: value.text_area_idle_refinements,
            text_area_idle_refinements_suppressed: value.text_area_idle_refinements_suppressed,
            async_scroll_projection_sync_skips: value.async_scroll_projection_sync_skips,
            async_scroll_reconciles: value.async_scroll_reconciles,
            retained_scroll_translations: value.retained_scroll_translations,
            retained_scroll_translated_items: value.retained_scroll_translated_items,
            retained_scroll_chrome_repaints: value.retained_scroll_chrome_repaints,
            retained_scroll_target_repaint_fallbacks: value
                .retained_scroll_target_repaint_fallbacks,
            retained_scroll_layer_hits: value.retained_scroll_layer_hits,
            retained_scroll_layer_replaced_items: value.retained_scroll_layer_replaced_items,
            retained_scroll_layer_text_prepare_skips: value
                .retained_scroll_layer_text_prepare_skips,
            retained_scroll_layer_missing: value.retained_scroll_layer_missing,
            retained_scroll_layer_metrics_misses: value.retained_scroll_layer_metrics_misses,
            retained_scroll_layer_coverage_misses: value.retained_scroll_layer_coverage_misses,
            retained_scroll_layer_geometry_misses: value.retained_scroll_layer_geometry_misses,
            retained_scroll_layer_projection_misses: value.retained_scroll_layer_projection_misses,
            retained_scroll_layer_rebuilds: value.retained_scroll_layer_rebuilds,
            projection_count: value.projection_count,
            last_scroll: LastScrollDiagnostics {
                wheel_events: value.last_scroll.wheel_events,
                thumb_drag_moves: value.last_scroll.thumb_drag_moves,
                scroll_offset_changes: value.last_scroll.scroll_offset_changes,
                retained_scroll_layer_hits: value.last_scroll.retained_scroll_layer_hits,
                retained_scroll_layer_text_prepare_skips: value
                    .last_scroll
                    .retained_scroll_layer_text_prepare_skips,
                retained_scroll_target_repaint_fallbacks: value
                    .last_scroll
                    .retained_scroll_target_repaint_fallbacks,
                retained_scroll_layer_missing: value.last_scroll.retained_scroll_layer_missing,
                retained_scroll_layer_metrics_misses: value
                    .last_scroll
                    .retained_scroll_layer_metrics_misses,
                retained_scroll_layer_coverage_misses: value
                    .last_scroll
                    .retained_scroll_layer_coverage_misses,
                retained_scroll_layer_geometry_misses: value
                    .last_scroll
                    .retained_scroll_layer_geometry_misses,
                retained_scroll_layer_projection_misses: value
                    .last_scroll
                    .retained_scroll_layer_projection_misses,
                retained_scroll_layer_rebuilds: value.last_scroll.retained_scroll_layer_rebuilds,
            },
        }
    }
}

pub fn new<T: Send + 'static>(parts: Parts<'_, T>) -> Context<'_, T> {
    Context {
        rendering: parts.rendering,
        windows: parts.windows,
        window_states: parts.window_states,
        commands: parts.commands,
        text_editor: parts.text_editor,
        text_engine: parts.text_engine,
        clipboard: parts.clipboard,
        mailbox: parts.mailbox,
        sender: parts.sender,
        redraw_on_command_state_change: parts.redraw_on_command_state_change,
        event_loop: parts.event_loop,
    }
}

pub(crate) fn command_dispatch<T: Send + 'static>(
    parts: Parts<'_, T>,
    call: command::call::Any,
) -> CommandDispatch<'_, T> {
    CommandDispatch {
        cx: new(parts),
        call: Some(call),
        handled: false,
        effects: Vec::new(),
        projection_window: None,
        changed: false,
    }
}

pub(crate) fn command_projection<T: Send + 'static>(
    parts: Parts<'_, T>,
    window: window::Id,
) -> CommandDispatch<'_, T> {
    CommandDispatch {
        cx: new(parts),
        call: None,
        handled: false,
        effects: Vec::new(),
        projection_window: Some(window),
        changed: false,
    }
}

fn text_edit_should_ensure_caret_visible(
    pointer_placement: bool,
    text_changed: bool,
    selection_changed: bool,
    selection_only: bool,
) -> bool {
    if pointer_placement || selection_only {
        return false;
    }

    text_changed || selection_changed
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

    pub fn request_redraw(&mut self, window: window::Id) {
        if self.windows.contains(window) {
            self.window_states
                .entry(window)
                .or_default()
                .invalidate_frame(frame::RedrawKind::Full, Instant::now());
        }
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

    pub fn commands(&mut self, configure: impl FnOnce(&mut command::registry::Commands)) {
        self.try_commands(configure)
            .unwrap_or_else(|error| panic!("{error}"));
    }

    pub fn try_commands(
        &mut self,
        configure: impl FnOnce(&mut command::registry::Commands),
    ) -> std::result::Result<(), command::registry::RegisterError> {
        self.commands.try_commands(configure)
    }

    pub(crate) fn override_command_state_key(
        &mut self,
        command: command::Key,
        context: command::call::Context,
        state: command::State,
    ) {
        let window = context.window_id();

        if self.commands.set_state_key(command, context, state)
            && self.redraw_on_command_state_change
        {
            self.request_redraw(window);
        }
    }

    pub fn override_command_state<C: Command>(
        &mut self,
        context: command::call::Context,
        state: command::State,
    ) {
        self.override_command_state_key(command::Key::of::<C>(), context, state);
    }

    pub(crate) fn command_override_key(
        &mut self,
        window: window::Id,
        command: command::Key,
    ) -> CommandOverride<'_> {
        CommandOverride::new(
            self.commands,
            self.window_states.get_mut(&window),
            window,
            command,
            self.redraw_on_command_state_change,
        )
    }

    pub fn command_override<C: Command>(&mut self, window: window::Id) -> CommandOverride<'_> {
        self.command_override_key(window, command::Key::of::<C>())
    }

    pub(crate) fn command_state_key(
        &self,
        command: command::Key,
        context: command::call::Context,
    ) -> command::State {
        self.commands.state_key(command, context)
    }

    pub fn command_state<C: Command>(&self, context: command::call::Context) -> command::State {
        self.command_state_key(command::Key::of::<C>(), context)
    }

    pub fn diagnostics(&self, window: window::Id) -> Diagnostics {
        Diagnostics {
            text: self.text_engine.diagnostics(),
            edit: self.text_editor.diagnostics(),
            scroll: self
                .window_states
                .get(&window)
                .map(|state| ScrollDiagnostics::from_scroll(state.scroll.diagnostics()))
                .unwrap_or_default(),
            frame: self
                .window_states
                .get(&window)
                .map(WindowState::frame_diagnostics)
                .unwrap_or_default(),
        }
    }

    pub fn emit(&mut self, event: T) {
        self.mailbox.push_app(event);
    }

    pub fn spawn(&self, task: Task<T>) {
        task_runner::spawn(task, self.sender.clone());
    }

    pub fn apply_text_edit(
        &mut self,
        buffer: &mut text::Buffer,
        edit_state: &mut text::edit::State,
        edit: text::edit::Edit,
    ) -> bool {
        let result =
            self.text_editor
                .apply_edit_with_caret_map(buffer, edit_state, edit, self.text_engine);
        if result.text_changed {
            self.text_engine
                .invalidate_text_area_for_edit(buffer, &result.impacts);
        }
        result.buffer_changed()
    }

    pub fn apply_text_edit_for(
        &mut self,
        target: &ui::Path,
        buffer: &mut text::Buffer,
        edit: text::edit::Edit,
    ) -> bool {
        let Some(mut edit_state) = self.window_states.values().find_map(|state| {
            state
                .can_apply_text_edit(target, &edit)
                .then(|| state.text_edit_state_or_initial(target, buffer))
        }) else {
            return false;
        };

        let history_kind = app_text::HistoryKind::for_edit(&edit);
        let pointer_placement = matches!(edit, text::edit::Edit::Pointer { .. });
        let selection_only = matches!(edit, text::edit::Edit::SelectAll);
        let scroll_anchors = self
            .window_states
            .iter()
            .map(|(window, state)| (*window, state.text_area_scroll_anchor(target)))
            .collect::<std::collections::HashMap<_, _>>();
        let now = Instant::now();
        let result = self.text_editor.apply_edit_with_caret_map(
            buffer,
            &mut edit_state,
            edit,
            self.text_engine,
        );
        if result.text_changed {
            self.text_engine
                .invalidate_text_area_for_edit(buffer, &result.impacts);
        }

        if let Some(change) = result.change.clone() {
            for state in self.window_states.values_mut() {
                state.record_text_field_history(target, change.clone(), history_kind.clone(), now);
            }
        }

        if result.buffer_changed() {
            let ensure_caret = text_edit_should_ensure_caret_visible(
                pointer_placement,
                result.text_changed,
                result.selection_changed,
                selection_only,
            );
            for (window, state) in self.window_states.iter_mut() {
                state.store_text_edit_state(target, edit_state);
                if ensure_caret {
                    state.ensure_text_caret_visible_after_edit(
                        target,
                        now,
                        self.text_engine,
                        scroll_anchors.get(window).copied().flatten(),
                    );
                } else {
                    state.reset_text_field_caret_blink_without_scroll(target, now);
                }
            }
        }

        result.buffer_changed()
    }

    pub fn apply_text_command_for(
        &mut self,
        target: &ui::Path,
        buffer: &mut text::Buffer,
        command: text::edit::Action,
    ) -> text::edit::ActionResult {
        text_input::apply_command_for(
            self.window_states,
            self.text_editor,
            self.text_engine,
            self.clipboard,
            target,
            buffer,
            command,
        )
    }

    pub fn invoke_text_command<C>(
        &mut self,
        buffer: &mut text::Buffer,
        context: command::call::Context,
        source: command::call::Source,
    ) -> std::result::Result<
        command::Response<text::edit::ActionResult>,
        command::registry::Rejection,
    >
    where
        C: crate::widget::text_command::EditCommand,
    {
        let call = command::Call::<C>::for_context_with_target(
            (),
            context,
            crate::widget::text_command::text_target_kind(),
        )
        .expect("unit command args should validate")
        .with_source(source);
        self.invoke_text_call(buffer, call)
    }

    pub fn invoke_text_call<C>(
        &mut self,
        buffer: &mut text::Buffer,
        call: command::Call<C>,
    ) -> std::result::Result<
        command::Response<text::edit::ActionResult>,
        command::registry::Rejection,
    >
    where
        C: crate::widget::text_command::EditCommand,
    {
        let mut target = text_input::CommandTarget::new(
            self.window_states,
            self.text_editor,
            self.text_engine,
            self.clipboard,
            buffer,
        );

        self.commands.invoke_on(&mut target, call)
    }

    pub fn invoke_insert_text_call(
        &mut self,
        buffer: &mut text::Buffer,
        call: command::Call<crate::widget::text_command::InsertText>,
    ) -> std::result::Result<
        command::Response<text::edit::ActionResult>,
        command::registry::Rejection,
    > {
        let mut target = text_input::CommandTarget::new(
            self.window_states,
            self.text_editor,
            self.text_engine,
            self.clipboard,
            buffer,
        );

        self.commands.invoke_on(&mut target, call)
    }

    pub fn invoke_on<C, TTarget>(
        &mut self,
        target: &mut TTarget,
        call: command::Call<C>,
    ) -> std::result::Result<command::Response<C::Output>, command::registry::Rejection>
    where
        C: Command,
        TTarget: command::Target<C>,
    {
        self.commands.invoke_on(target, call)
    }

    pub fn sender(&self) -> Sender<T> {
        self.sender.clone()
    }

    pub fn invoke_call<C: Command>(&mut self, call: command::Call<C>) {
        self.mailbox.run_call(call);
    }

    pub fn invoke_command<C, TTarget>(&mut self, context: command::call::Context)
    where
        C: Command<Args = ()>,
        TTarget: command::Target<C> + 'static,
    {
        self.invoke_call(
            command::Call::<C>::for_context::<TTarget>((), context)
                .expect("unit command args should validate"),
        );
    }

    pub fn command_subject(&self, window: window::Id) -> command::call::Context {
        self.window_states
            .get(&window)
            .map(|state| state.command_context(window))
            .unwrap_or_else(|| command::call::Context::window(window))
    }

    pub fn set_command_subject(&mut self, window: window::Id, context: command::call::Context) {
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

    pub fn resolve_command_context(
        &self,
        window: window::Id,
        requested_scope: Option<command::call::Scope>,
    ) -> command::call::Context {
        if let Some(scope) = requested_scope {
            return command::call::Context::with_scope(window, scope);
        }

        self.command_subject(window)
    }
}

impl<'cx, T: Send + 'static> CommandDispatch<'cx, T> {
    pub fn target<TTarget>(&mut self, target: &mut TTarget) -> &mut Self
    where
        TTarget: 'static,
    {
        if let Some(window) = self.projection_window {
            let mut changed = false;
            for context in self.projection_contexts(window) {
                changed |= self.cx.commands.project_target_states(target, context);
            }
            self.record_projection_change(window, changed);
            return self;
        }

        if !self
            .call
            .as_ref()
            .is_some_and(|call| self.cx.commands.can_invoke_any_on::<TTarget>(call))
        {
            return self;
        }

        let call = self
            .call
            .take()
            .expect("call presence was checked before target dispatch");
        let command = call.command();
        let fallback_context = call.context();

        match self.cx.commands.invoke_any_on(target, call) {
            Ok(response) => {
                log::debug!(
                    "command dispatch target accepted command={} context={:?}",
                    response.command.as_str(),
                    response.context
                );
                self.handled = true;
                self.request_redraw(response.context.window_id());
                for effect in response.effects {
                    self.effects.push(QueuedCommandEffect {
                        command: response.command,
                        context: response.context.clone(),
                        effect,
                    });
                }
            }
            Err(error) => {
                log::debug!(
                    "command dispatch target rejected command={}: {error}",
                    command.as_str()
                );
                self.handled = true;
                if let Some(context) = fallback_context {
                    self.request_redraw(context.window_id());
                }
            }
        }

        self
    }

    pub fn text_buffer(&mut self, buffer: &mut text::Buffer) -> Option<TextCommandOutcome> {
        if let Some(window) = self.projection_window {
            let changed = self.cx.window_states.get(&window).is_some_and(|state| {
                text_input::publish_command_states(state, self.cx.commands, window)
            });
            self.record_projection_change(window, changed);
            return None;
        }

        if let Some(outcome) = self.insert_text_command(buffer) {
            return Some(outcome);
        }

        macro_rules! dispatch_text_command {
            ($command:ty, $edit:expr, $dispatch:expr, $buffer:expr) => {
                if let Some(outcome) = $dispatch.text_command::<$command>($buffer) {
                    return Some(outcome);
                }
            };
        }

        crate::widget::text_command::for_each_edit_command!(dispatch_text_command, self, buffer);
        None
    }

    pub fn request_redraw(&mut self, window: window::Id) {
        self.cx.request_redraw(window);
    }

    pub fn close_window(&mut self, window: window::Id) {
        self.cx.close_window(window);
    }

    pub fn emit(&mut self, event: T) {
        self.cx.emit(event);
    }

    pub fn spawn(&self, task: Task<T>) {
        self.cx.spawn(task);
    }

    pub fn sender(&self) -> Sender<T> {
        self.cx.sender()
    }

    pub fn apply_text_edit_for(
        &mut self,
        target: &ui::Path,
        buffer: &mut text::Buffer,
        edit: text::edit::Edit,
    ) -> bool {
        self.cx.apply_text_edit_for(target, buffer, edit)
    }

    pub(crate) fn finish(self) -> (bool, Option<command::call::Any>, Vec<QueuedCommandEffect>) {
        (self.handled, self.call, self.effects)
    }

    pub(crate) fn finish_projection(self) -> bool {
        self.changed
    }

    fn text_command<C>(&mut self, buffer: &mut text::Buffer) -> Option<TextCommandOutcome>
    where
        C: crate::widget::text_command::EditCommand,
    {
        if !self
            .call
            .as_ref()
            .is_some_and(|call| call.target() == crate::widget::text_command::text_target_kind())
        {
            return None;
        }

        let call = self.take_call::<C>()?;
        let context = call.requested_context()?;
        let command = C::edit_action();
        let label = crate::widget::text_command::edit_action_label(command);
        let result = match self.cx.invoke_text_call::<C>(buffer, call) {
            Ok(response) => response.into_output(),
            Err(error) => {
                log::debug!(
                    "text command dispatch rejected command={}: {error}",
                    C::NAME
                );
                text::edit::ActionResult {
                    unavailable: true,
                    ..text::edit::ActionResult::default()
                }
            }
        };

        self.handled = true;
        self.request_redraw(context.window_id());

        Some(TextCommandOutcome { label, result })
    }

    fn take_call<C: Command>(&mut self) -> Option<command::Call<C>> {
        let call = self.call.take()?;
        match call.into_call::<C>() {
            Ok(call) => Some(call),
            Err(call) => {
                self.call = Some(call);
                None
            }
        }
    }

    fn insert_text_command(&mut self, buffer: &mut text::Buffer) -> Option<TextCommandOutcome> {
        if !self
            .call
            .as_ref()
            .is_some_and(|call| call.target() == crate::widget::text_command::text_target_kind())
        {
            return None;
        }

        let Some(call) = self.take_call::<crate::widget::text_command::InsertText>() else {
            return None;
        };
        let context = call.requested_context();
        let result = match self.cx.invoke_insert_text_call(buffer, call) {
            Ok(response) => {
                let (result, effects) = response.into_parts();
                if let Some(context) = context.clone() {
                    self.request_redraw(context.window_id());
                    for effect in effects {
                        self.effects.push(QueuedCommandEffect {
                            command: command::Key::of::<crate::widget::text_command::InsertText>(),
                            context: context.clone(),
                            effect,
                        });
                    }
                }
                self.handled = true;
                result
            }
            Err(error) => {
                log::debug!("text insert command dispatch rejected command=insert_text: {error}");
                if let Some(context) = context {
                    self.request_redraw(context.window_id());
                }
                self.handled = true;
                text::edit::ActionResult {
                    unavailable: true,
                    ..text::edit::ActionResult::default()
                }
            }
        };

        Some(TextCommandOutcome {
            label: "insert text",
            result,
        })
    }

    fn record_projection_change(&mut self, window: window::Id, changed: bool) {
        self.changed |= changed;
        if changed && self.cx.redraw_on_command_state_change {
            self.cx.request_redraw(window);
        }
    }

    fn projection_contexts(&self, window: window::Id) -> Vec<command::call::Context> {
        let mut contexts = Vec::new();
        Self::push_context(&mut contexts, command::call::Context::window(window));

        let Some(state) = self.cx.window_states.get(&window) else {
            return contexts;
        };

        Self::push_context(&mut contexts, state.command_context(window));

        if let Some(composition) = state.composition.as_ref() {
            for path in composition.action_map().keys() {
                Self::push_context(&mut contexts, state.command_context_for_path(window, path));
            }
        }

        contexts
    }

    fn push_context(contexts: &mut Vec<command::call::Context>, context: command::call::Context) {
        if !contexts.contains(&context) {
            contexts.push(context);
        }
    }
}

impl TextCommandOutcome {
    pub fn label(self) -> &'static str {
        self.label
    }

    pub fn result(self) -> text::edit::ActionResult {
        self.result
    }
}

pub struct CommandOverride<'a> {
    commands: &'a mut command::Registry,
    window_state: Option<&'a mut WindowState>,
    window: window::Id,
    command: command::Key,
    state: command::State,
    changed: bool,
    redraw_on_command_state_change: bool,
}

impl<'a> CommandOverride<'a> {
    fn new(
        commands: &'a mut command::Registry,
        window_state: Option<&'a mut WindowState>,
        window: window::Id,
        command: command::Key,
        redraw_on_command_state_change: bool,
    ) -> Self {
        let state = commands.configured_state_key(command, command::call::Context::window(window));

        Self {
            commands,
            window_state,
            window,
            command,
            state,
            changed: false,
            redraw_on_command_state_change,
        }
    }

    pub fn available(mut self, available: bool) -> Self {
        self.state = self.state.clone().with_available(available);
        self.changed = true;
        self
    }

    pub fn active(mut self, active: bool) -> Self {
        self.state = self.state.clone().with_active(active);
        self.changed = true;
        self
    }

    pub fn running(mut self, running: bool) -> Self {
        self.state = self.state.clone().with_running(running);
        self.changed = true;
        self
    }
}

impl Drop for CommandOverride<'_> {
    fn drop(&mut self) {
        if !self.changed {
            return;
        }

        let changed = self.commands.set_state_key(
            self.command,
            command::call::Context::window(self.window),
            self.state.clone(),
        );

        if changed
            && self.redraw_on_command_state_change
            && let Some(state) = self.window_state.as_mut()
        {
            state.invalidate_frame(frame::RedrawKind::Full, Instant::now());
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pointer_text_edits_do_not_request_caret_visibility_scroll() {
        assert!(!text_edit_should_ensure_caret_visible(
            true, false, true, false,
        ));
    }

    #[test]
    fn keyboard_caret_edits_still_request_caret_visibility_scroll() {
        assert!(text_edit_should_ensure_caret_visible(
            false, false, true, false,
        ));
    }

    #[test]
    fn select_all_does_not_request_caret_visibility_scroll() {
        assert!(!text_edit_should_ensure_caret_visible(
            false, false, true, true,
        ));
    }
}
