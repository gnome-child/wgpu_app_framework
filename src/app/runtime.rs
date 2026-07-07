use std::collections::{HashMap, HashSet};
use std::time::{Duration, Instant};

use winit::{
    application::ApplicationHandler,
    event::{
        ElementState, KeyEvent, MouseButton, MouseScrollDelta, StartCause, TouchPhase, WindowEvent,
    },
    event_loop::ActiveEventLoop,
};

use crate::animation;
use crate::app::clipboard::SystemClipboard;
use crate::app::command as command_layer;
use crate::app::context;
use crate::app::frame;
use crate::app::input;
use crate::app::key_repeat;
use crate::app::mailbox::{Mailbox, Message};
use crate::app::rendering::Driver;
use crate::app::sender::Sender;
use crate::app::state::WindowState;
use crate::app::view;
use crate::app::windows::Windows;
use crate::geometry::{area, point};
use crate::{command, event, native, paint, text, ui, window};

use super::{Application, Error, Options};

pub struct Runtime<A: Application> {
    app: A,
    rendering: Driver,
    windows: Windows,
    window_states: HashMap<window::Id, WindowState>,
    commands: command::Registry,
    text_editor: text::edit::Editor,
    text_engine: text::layout::Engine,
    clipboard: SystemClipboard,
    mailbox: Mailbox<A::Event>,
    sender: Sender<A::Event>,
    key_repeat_policy: key_repeat::KeyRepeatPolicy,
    key_repeat: key_repeat::State,
    animation_schedules: HashMap<window::Id, animation::Schedule>,
    last_frames: HashMap<window::Id, Instant>,
    cursors: HashMap<window::Id, ui::Cursor>,
    native_pointer_captures: HashSet<window::Id>,
    started: bool,
    error: Option<Error>,
}

impl<A: Application> Runtime<A> {
    pub fn new(app: A, sender: Sender<A::Event>, options: Options) -> Self {
        Self {
            app,
            rendering: Driver::new(),
            windows: Windows::new(),
            window_states: HashMap::new(),
            commands: command::Registry::new(),
            text_editor: text::edit::Editor::new(),
            text_engine: text::layout::Engine::new(),
            clipboard: SystemClipboard::new(),
            mailbox: Mailbox::new(),
            sender,
            key_repeat_policy: options.key_repeat,
            key_repeat: key_repeat::State::new(
                options.key_repeat.timer_settings().unwrap_or_default(),
            ),
            animation_schedules: HashMap::new(),
            last_frames: HashMap::new(),
            cursors: HashMap::new(),
            native_pointer_captures: HashSet::new(),
            started: false,
            error: None,
        }
    }

    pub fn take_error(&mut self) -> Option<Error> {
        self.error.take()
    }

    fn fail(&mut self, event_loop: &ActiveEventLoop, error: Error) {
        self.error = Some(error);
        event_loop.exit();
    }

    fn dispatch_event(&mut self, event_loop: &ActiveEventLoop, event: event::Event<A::Event>) {
        self.dispatch_message(event_loop, Message::Event(event));
    }

    fn dispatch_message(&mut self, event_loop: &ActiveEventLoop, message: Message<A::Event>) {
        self.mailbox.push_message(message);
        self.drain_mailbox(event_loop);
    }

    fn dispatch_ui_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        window: window::Id,
        event: ui::Event,
    ) {
        self.dispatch_event(event_loop, event::Event::Ui { window, event });
    }

    fn drain_mailbox(&mut self, event_loop: &ActiveEventLoop) {
        if !self.rendering.ready() {
            return;
        }

        while let Some(message) = self.mailbox.pop() {
            match message {
                Message::Event(event) => {
                    let mut cx = context::new(context::Parts {
                        rendering: &mut self.rendering,
                        windows: &mut self.windows,
                        window_states: &mut self.window_states,
                        commands: &mut self.commands,
                        text_editor: &mut self.text_editor,
                        text_engine: &mut self.text_engine,
                        clipboard: &mut self.clipboard,
                        mailbox: &mut self.mailbox,
                        sender: self.sender.clone(),
                        redraw_on_command_state_change: true,
                        event_loop,
                    });

                    self.app.event(&mut cx, event);
                }
                Message::RunCommand(request) => {
                    self.run_command(event_loop, request);
                }
                Message::RunCall(call) => {
                    self.run_any_call(event_loop, call);
                }
                Message::CommandTaskCompleted {
                    command,
                    context,
                    response,
                } => {
                    self.complete_command_task(event_loop, command, context, response);
                }
                Message::AppTaskCompleted(event) => {
                    self.complete_app_task(event);
                }
            }
        }
    }

    fn run_command(&mut self, event_loop: &ActiveEventLoop, request: command::call::Raw) {
        let window = request.context().window_id();
        let Some(request) = self.window_states.get(&window).and_then(|state| {
            command_layer::resolve_executable_request(state, &self.commands, request.clone())
        }) else {
            log::debug!("command request rejected before dispatch: unresolved command target");
            return;
        };

        match self.commands.prepare_call(request.clone()) {
            Ok(call) => {
                let (handled, remaining) = self.dispatch_command_call(event_loop, call);
                if !handled && let Some(call) = remaining {
                    log::debug!(
                        "command request was prepared but no target handled it: {:?}",
                        call
                    );
                }
            }
            Err(error) => {
                log::debug!("command request rejected before dispatch: {error}");
            }
        }
    }

    fn run_any_call(&mut self, event_loop: &ActiveEventLoop, call: command::call::Any) {
        let (handled, remaining) = self.dispatch_command_call(event_loop, call);
        if !handled && let Some(call) = remaining {
            log::debug!("pending command call had no target: {:?}", call);
        }
    }

    fn dispatch_command_call(
        &mut self,
        event_loop: &ActiveEventLoop,
        call: command::call::Any,
    ) -> (bool, Option<command::call::Any>) {
        let mut dispatch = context::command_dispatch(
            context::Parts {
                rendering: &mut self.rendering,
                windows: &mut self.windows,
                window_states: &mut self.window_states,
                commands: &mut self.commands,
                text_editor: &mut self.text_editor,
                text_engine: &mut self.text_engine,
                clipboard: &mut self.clipboard,
                mailbox: &mut self.mailbox,
                sender: self.sender.clone(),
                redraw_on_command_state_change: true,
                event_loop,
            },
            call,
        );

        self.app.command_targets(&mut dispatch);
        let (handled, remaining, effects) = dispatch.finish();

        for effect in effects {
            self.enqueue_command_effect(event_loop, effect.command, effect.context, effect.effect);
        }

        (handled, remaining)
    }

    fn complete_command_task(
        &mut self,
        event_loop: &ActiveEventLoop,
        command: command::Key,
        context: command::call::Context,
        response: Result<command::Response<()>, command::registry::Rejection>,
    ) {
        if self
            .commands
            .set_running_key(command, context.clone(), false)
        {
            self.invalidate_full(context.window_id());
        }

        match response {
            Ok(response) => {
                let (_, effects) = response.into_parts();
                for effect in effects {
                    self.enqueue_command_effect(event_loop, command, context.clone(), effect);
                }
            }
            Err(error) => {
                log::debug!(
                    "command task completed with rejection command={} context={context:?}: {error}",
                    command.as_str()
                );
            }
        }
    }

    fn enqueue_command_effect(
        &mut self,
        event_loop: &ActiveEventLoop,
        command: command::Key,
        context: command::call::Context,
        effect: command::Effect,
    ) {
        match effect {
            command::Effect::None => {}
            command::Effect::Runtime(effect) => match effect {
                command::effect::RuntimeEffect::Notify(message) => {
                    log::debug!("command runtime notification: {message}");
                }
                command::effect::RuntimeEffect::RequestRedraw => {
                    self.invalidate_full(context.window_id());
                }
                command::effect::RuntimeEffect::ClipboardWrite(text) => {
                    use crate::text::edit::Clipboard;

                    if let Err(error) = self.clipboard.write_text(&text) {
                        log::debug!("command clipboard write failed: {error:?}");
                    }
                }
            },
            command::Effect::Batch(effects) => {
                for effect in effects {
                    self.enqueue_command_effect(event_loop, command, context.clone(), effect);
                }
            }
            command::Effect::Call(call) => {
                self.run_any_call(event_loop, call.with_fallback_window(context.window_id()));
            }
            command::Effect::Task(task) => {
                if self
                    .commands
                    .set_running_key(command, context.clone(), true)
                {
                    self.invalidate_full(context.window_id());
                }
                let sender = self.sender.clone();
                let task_context = context.clone();
                std::thread::spawn(move || {
                    use crate::app::MailboxSender;

                    let response = task.run();
                    let _ = sender.send_message(Message::CommandTaskCompleted {
                        command,
                        context: task_context,
                        response,
                    });
                });
            }
        }
    }

    fn complete_app_task(&mut self, event: A::Event) {
        self.mailbox.push_app(event);
    }

    fn frame_for_window(&mut self, window: window::Id) -> animation::Frame {
        let now = Instant::now();
        let previous = self.last_frames.insert(window, now);

        animation::Frame::new(now, previous)
    }

    fn invalidate_due_animation_frames(&mut self, now: Instant) {
        let due = self
            .animation_schedules
            .iter()
            .filter_map(|(window, schedule)| schedule.is_due(now).then_some(*window))
            .collect::<Vec<_>>();

        for window in due {
            if !self.windows.contains(window) {
                self.animation_schedules.remove(&window);
                continue;
            }

            if self
                .window_states
                .get(&window)
                .is_some_and(WindowState::smooth_scroll_active)
            {
                self.invalidate_scroll(window);
            } else {
                self.invalidate_full(window);
            }
        }
    }

    fn refresh_animation_schedules(&mut self, now: Instant) -> animation::Schedule {
        self.animation_schedules
            .retain(|window, _| self.windows.contains(*window));

        for (window, state) in &self.window_states {
            if !self.windows.contains(*window) {
                continue;
            }

            let schedule = state.animation_schedule(now);
            if schedule == animation::Schedule::Idle {
                self.animation_schedules.remove(window);
            } else {
                self.animation_schedules.insert(*window, schedule);
            }
        }

        self.animation_schedules
            .values()
            .copied()
            .fold(animation::Schedule::Idle, animation::Schedule::merge)
    }

    fn redraw_window(&mut self, event_loop: &ActiveEventLoop, window: window::Id) {
        if !self.rendering.ready() {
            return;
        }

        let total_start = Instant::now();
        let frame = self.frame_for_window(window);
        let Some(work) = self
            .window_states
            .get_mut(&window)
            .and_then(|state| state.frame.begin_redraw(frame.now()))
        else {
            return;
        };
        let mut redraw_kind = work.kind();
        let dirty_to_frame = work.dirty_to_frame();

        let due_selection_drag_autoscroll = self
            .animation_schedules
            .get(&window)
            .is_some_and(|schedule| schedule.is_due(frame.now()))
            && self
                .window_states
                .get(&window)
                .is_some_and(WindowState::text_selection_drag_autoscroll_active);
        if due_selection_drag_autoscroll {
            self.advance_text_selection_drag_autoscroll(event_loop, window);
            redraw_kind = frame::RedrawKind::Full;
        }

        let can_scroll_only = redraw_kind.is_scroll_only()
            && self
                .window_states
                .get(&window)
                .is_some_and(|state| state.composition.is_some() && !state.scroll.is_empty());

        let (paint_result, actual_kind) = if can_scroll_only {
            self.text_editor.reset_diagnostics();
            self.text_engine.reset_diagnostics();
            let state = self.window_states.entry(window).or_default();
            match view::paint_scroll_only(
                window,
                state,
                &mut self.commands,
                &mut self.text_engine,
                frame,
            ) {
                Some(result) => (result, frame::RedrawKind::ScrollOnly),
                None => {
                    if let Some(state) = self.window_states.get_mut(&window) {
                        state.frame.record_scroll_only_fallback_to_full();
                    }
                    (
                        self.full_redraw(event_loop, window, frame),
                        frame::RedrawKind::Full,
                    )
                }
            }
        } else {
            (
                self.full_redraw(event_loop, window, frame),
                frame::RedrawKind::Full,
            )
        };

        self.sync_ime_for_window(window);
        self.sync_cursor_for_window(window);

        let Some(native_window) = self.windows.get_mut(window) else {
            return;
        };

        use crate::render::frame::Status::*;
        let draw_report = match self.rendering.draw(
            native_window,
            &paint_result.scene,
            &paint_result.layer_updates,
        ) {
            Ok(report) => report,
            Err(error) => {
                self.fail(event_loop, error.into());
                return;
            }
        };
        let presented = match draw_report.status {
            Presented => true,
            Skipped(reason) => {
                log::warn!("render pass was skipped: {:#?}", reason);
                if let Some(state) = self.window_states.get_mut(&window) {
                    state.frame.record_render_skip();
                    state.invalidate_frame(frame::RedrawKind::Full, Instant::now());
                }
                false
            }
        };

        if !presented {
            self.drain_mailbox(event_loop);
            return;
        }

        let presented_at = Instant::now();
        if let Some(state) = self.window_states.get_mut(&window) {
            let latency = state.take_scroll_input_latency(presented_at);
            let total = total_start.elapsed();
            let text_diagnostics = self.text_engine.diagnostics();
            let scroll_diagnostics = state.scroll.diagnostics();
            state.frame.record_presented(
                actual_kind,
                frame::FrameTimings {
                    stages: paint_result.timings,
                    render: draw_report.timings.total,
                    render_stages: frame::RenderTimings {
                        acquire: draw_report.timings.surface_acquire,
                        batching: draw_report.timings.scene_batching,
                        quad_prepare: draw_report.timings.quad_prepare,
                        text_prepare: draw_report.timings.text_prepare,
                        scene_text_prepare: draw_report.timings.scene_text_prepare,
                        layer_update_text_prepare: draw_report.timings.layer_update_text_prepare,
                        filter_prepare: draw_report.timings.filter_prepare,
                        encode_submit: draw_report.timings.encode_submit,
                    },
                    render_stats: frame::RenderStats {
                        scene_items: draw_report.stats.scene_items,
                        render_batches: draw_report.stats.render_batches,
                        glyph_batches: draw_report.stats.glyph_batches,
                        text_surfaces: draw_report.stats.text_surfaces,
                        inline_text_cache_hits: draw_report.stats.inline_text_cache_hits,
                        inline_text_cache_misses: draw_report.stats.inline_text_cache_misses,
                        inline_text_shape_calls: draw_report.stats.inline_text_shape_calls,
                        inline_icon_cache_hits: draw_report.stats.inline_icon_cache_hits,
                        inline_icon_cache_misses: draw_report.stats.inline_icon_cache_misses,
                        inline_icon_shape_calls: draw_report.stats.inline_icon_shape_calls,
                        quad_vertices: draw_report.stats.quad_vertices,
                        clip_batches: draw_report.stats.clip_batches,
                        filters: draw_report.stats.filters,
                        layer_items: draw_report.stats.layer_items,
                        layer_updates: draw_report.stats.layer_updates,
                    },
                    total,
                    scroll_input_to_present: latency,
                    dirty_to_present: dirty_to_frame.map(|latency| latency + total),
                },
            );
            trace_presented_scroll_frame(
                actual_kind,
                paint_result.timings,
                draw_report,
                total,
                scroll_diagnostics,
                text_diagnostics,
            );
            if state.smooth_scroll_active() {
                state.invalidate_frame(frame::RedrawKind::ScrollOnly, Instant::now());
            }
        }

        self.drain_mailbox(event_loop);
    }

    fn full_redraw(
        &mut self,
        event_loop: &ActiveEventLoop,
        window: window::Id,
        frame: animation::Frame,
    ) -> view::PaintResult {
        let mut tree = ui::Tree::new();

        self.commands.clear_context_states(window);

        {
            let mut cx = context::new(context::Parts {
                rendering: &mut self.rendering,
                windows: &mut self.windows,
                window_states: &mut self.window_states,
                commands: &mut self.commands,
                text_editor: &mut self.text_editor,
                text_engine: &mut self.text_engine,
                clipboard: &mut self.clipboard,
                mailbox: &mut self.mailbox,
                sender: self.sender.clone(),
                redraw_on_command_state_change: false,
                event_loop,
            });

            self.app.view(&mut cx, window, &mut tree);
        }
        self.project_command_states(event_loop, window);
        self.text_editor.reset_diagnostics();
        self.text_engine.reset_diagnostics();

        let Some(native_window) = self.windows.get(window) else {
            return view::PaintResult {
                scene: paint::Scene::new(),
                timings: frame::StageTimings::default(),
                layer_updates: Vec::new(),
            };
        };
        let logical_area = native_window.canvas().logical_area();
        let mut paint_result = view::compose_with_timings(
            window,
            &tree,
            self.window_states.entry(window).or_default(),
            &mut self.commands,
            &mut self.text_engine,
            logical_area,
            frame,
        );

        if self.project_command_states(event_loop, window) {
            paint_result = view::compose_with_timings(
                window,
                &tree,
                self.window_states.entry(window).or_default(),
                &mut self.commands,
                &mut self.text_engine,
                logical_area,
                frame,
            );
        }

        paint_result
    }

    fn project_command_states(&mut self, event_loop: &ActiveEventLoop, window: window::Id) -> bool {
        let mut dispatch = context::command_projection(
            context::Parts {
                rendering: &mut self.rendering,
                windows: &mut self.windows,
                window_states: &mut self.window_states,
                commands: &mut self.commands,
                text_editor: &mut self.text_editor,
                text_engine: &mut self.text_engine,
                clipboard: &mut self.clipboard,
                mailbox: &mut self.mailbox,
                sender: self.sender.clone(),
                redraw_on_command_state_change: false,
                event_loop,
            },
            window,
        );

        self.app.command_targets(&mut dispatch);
        dispatch.finish_projection()
    }

    fn advance_text_selection_drag_autoscroll(
        &mut self,
        event_loop: &ActiveEventLoop,
        window: window::Id,
    ) -> bool {
        let Some(state) = self.window_states.get_mut(&window) else {
            return false;
        };
        let outcome =
            input::text_selection_drag_autoscroll_with_text_engine(state, &mut self.text_engine);
        let redraw = outcome.redraw;

        self.sync_cursor_for_window(window);
        self.dispatch_ui_events(event_loop, window, outcome.events);
        redraw
    }

    fn close_window(&mut self, event_loop: &ActiveEventLoop, window: window::Id) {
        self.key_repeat.clear_window(window);
        self.release_native_pointer_capture(window);
        self.windows.remove(window);
        self.window_states.remove(&window);
        self.animation_schedules.remove(&window);
        self.last_frames.remove(&window);
        self.cursors.remove(&window);

        if self.windows.is_empty() {
            event_loop.exit();
        }
    }

    fn pointer_moved(
        &mut self,
        event_loop: &ActiveEventLoop,
        window: window::Id,
        position: point::Logical,
    ) {
        let Some(state) = self.window_states.get_mut(&window) else {
            return;
        };
        let outcome = input::pointer_moved_with_text_engine(state, position, &mut self.text_engine);

        let has_scroll_capture = self
            .window_states
            .get(&window)
            .is_some_and(WindowState::pointer_capture_is_scroll_thumb);
        if outcome.redraw && has_scroll_capture {
            if let Some(state) = self.window_states.get_mut(&window) {
                state.mark_scroll_input(Instant::now());
            }
            self.invalidate_scroll(window);
        } else if outcome.redraw {
            self.invalidate_full(window);
        }

        self.sync_cursor_for_window(window);
        self.dispatch_ui_events(event_loop, window, outcome.events);

        if let Some(intent) = outcome.intent {
            self.handle_intent(window, intent);
        }
    }

    fn pointer_button(
        &mut self,
        event_loop: &ActiveEventLoop,
        window: window::Id,
        state: ElementState,
        button: MouseButton,
    ) {
        let button = input::pointer_button(button);

        let position = self
            .window_states
            .get(&window)
            .and_then(|state| state.pointer.position())
            .unwrap_or_else(|| point::logical(0.0, 0.0));

        let Some(window_state) = self.window_states.get_mut(&window) else {
            return;
        };

        let had_scroll_capture = window_state.pointer_capture_is_scroll_thumb();
        let outcome = match state {
            ElementState::Pressed => input::pointer_pressed(
                window_state,
                window,
                position,
                button,
                &mut self.text_engine,
            ),
            ElementState::Released => {
                input::pointer_released(&self.commands, window_state, window, position, button)
            }
        };
        let has_scroll_capture = window_state.pointer_capture_is_scroll_thumb();
        self.sync_native_pointer_capture_for_capture(
            window,
            had_scroll_capture,
            has_scroll_capture,
        );

        self.dispatch_ui_events(event_loop, window, outcome.events);

        if let Some(request) = outcome.request {
            self.dispatch_message(event_loop, Message::RunCommand(request));
        }

        if let Some(intent) = outcome.intent {
            self.handle_intent(window, intent);
        }

        if outcome.redraw {
            self.invalidate_full(window);
        }
        self.sync_ime_for_window(window);
        self.sync_cursor_for_window(window);
    }

    fn mouse_wheel(
        &mut self,
        event_loop: &ActiveEventLoop,
        window: window::Id,
        delta: MouseScrollDelta,
        phase: TouchPhase,
    ) {
        let Some(native_window) = self.windows.get(window) else {
            return;
        };
        let scale_factor = native_window.scale_factor() as f32;
        let position = self
            .window_states
            .get(&window)
            .and_then(|state| state.pointer.position())
            .unwrap_or_else(|| point::logical(0.0, 0.0));
        let delta = match delta {
            MouseScrollDelta::LineDelta(x, y) => {
                let delta = point::logical(x, y);
                trace_scroll_input(format_args!("wheel raw=line {delta:?}"));
                crate::app::scroll::WheelDelta::lines(delta)
            }
            MouseScrollDelta::PixelDelta(position) => {
                let delta =
                    point::physical(position.x as f32, position.y as f32).to_logical(scale_factor);
                trace_scroll_input(format_args!("wheel raw=pixel {delta:?} phase={phase:?}"));
                crate::app::scroll::WheelDelta::pixels_with_phase(delta, wheel_phase(phase))
            }
        };

        let Some(state) = self.window_states.get_mut(&window) else {
            return;
        };
        let now = Instant::now();
        state.mark_scroll_input(now);
        let outcome = input::scroll_wheel(state, position, delta, &mut self.text_engine, now);

        self.dispatch_ui_events(event_loop, window, outcome.events);
        if outcome.redraw {
            self.invalidate_scroll(window);
        }
        self.sync_ime_for_window(window);
        self.sync_cursor_for_window(window);
    }

    fn modifiers_changed(&mut self, window: window::Id, modifiers: winit::event::Modifiers) {
        let Some(state) = self.window_states.get_mut(&window) else {
            return;
        };

        state.modifiers = input::modifiers(modifiers.state());
    }

    fn keyboard_input(
        &mut self,
        event_loop: &ActiveEventLoop,
        window: window::Id,
        event: KeyEvent,
    ) {
        let use_client_timer = self.key_repeat_policy.uses_client_timer();
        let suppress_backend_repeat = !self.key_repeat_policy.accepts_backend_repeat();

        if event.state == ElementState::Pressed && event.repeat && suppress_backend_repeat {
            return;
        }

        let key = input::key(&event.logical_key);
        let physical_key = event.physical_key;
        let inserted_text = event.text.as_deref();
        let repeat_text = event.text.as_ref().map(ToString::to_string);
        let now = Instant::now();

        if event.state == ElementState::Released {
            self.key_repeat.release(window, physical_key);
        }

        let Some(state) = self.window_states.get_mut(&window) else {
            return;
        };

        let outcome = match event.state {
            ElementState::Pressed => {
                let repeat = self.key_repeat_policy.accepts_backend_repeat() && event.repeat;
                let outcome = input::key_pressed_with_text(
                    &self.commands,
                    state,
                    window,
                    key,
                    inserted_text,
                    repeat,
                    &mut self.text_engine,
                );
                if use_client_timer {
                    self.key_repeat.press(
                        window,
                        physical_key,
                        key,
                        repeat_text,
                        outcome.repeatable_key,
                        now,
                    );
                }
                outcome
            }
            ElementState::Released => input::key_released(&self.commands, state, window, key),
        };

        self.finish_keyboard_outcome(event_loop, window, outcome);
    }

    fn finish_keyboard_outcome(
        &mut self,
        event_loop: &ActiveEventLoop,
        window: window::Id,
        outcome: input::Outcome,
    ) {
        self.dispatch_ui_events(event_loop, window, outcome.events);

        if let Some(request) = outcome.request {
            self.dispatch_message(event_loop, Message::RunCommand(request));
        }

        if let Some(intent) = outcome.intent {
            self.handle_intent(window, intent);
        }

        if outcome.redraw {
            self.invalidate_full(window);
        }
        self.sync_ime_for_window(window);
    }

    fn dispatch_due_key_repeat(&mut self, event_loop: &ActiveEventLoop, now: Instant) {
        if !self.key_repeat_policy.uses_client_timer() {
            return;
        }

        let Some(pulse) = self.key_repeat.due(now) else {
            return;
        };

        if !self.windows.contains(pulse.window) {
            self.key_repeat.clear_window(pulse.window);
            return;
        }

        let Some(state) = self.window_states.get_mut(&pulse.window) else {
            self.key_repeat.clear_window(pulse.window);
            return;
        };
        let outcome = input::key_pressed_with_text(
            &self.commands,
            state,
            pulse.window,
            pulse.key,
            pulse.text.as_deref(),
            true,
            &mut self.text_engine,
        );

        self.finish_keyboard_outcome(event_loop, pulse.window, outcome);
    }

    fn ime_input(
        &mut self,
        event_loop: &ActiveEventLoop,
        window: window::Id,
        event: winit::event::Ime,
    ) {
        let Some(state) = self.window_states.get_mut(&window) else {
            return;
        };
        let outcome = input::ime(state, event);

        self.dispatch_ui_events(event_loop, window, outcome.events);

        if outcome.redraw {
            self.invalidate_full(window);
        }
        self.sync_ime_for_window(window);
    }

    fn handle_intent(&mut self, window: window::Id, request: input::IntentRequest) {
        let Some(state) = self.window_states.get_mut(&window) else {
            return;
        };

        match request.intent {
            ui::Intent::Action(_) => {}
            ui::Intent::OpenMenu(menu) => {
                if state.toggle_menu(menu, &mut self.commands, window, request.source) {
                    self.invalidate_full(window);
                }
            }
            ui::Intent::OpenSubmenu(menu) => {
                if state.open_submenu(menu, &mut self.commands, window, request.source) {
                    self.invalidate_full(window);
                }
            }
            ui::Intent::CloseSubmenu => {
                if state.close_submenu() {
                    self.invalidate_full(window);
                }
            }
        }
    }

    fn sync_ime_for_window(&mut self, window: window::Id) {
        let Some(state) = self.window_states.get(&window) else {
            return;
        };
        let enabled = state.text_input_enabled();
        let cursor_rect = enabled
            .then(|| state.focused_text_field_caret_rect(&mut self.text_engine))
            .flatten();
        let Some(native_window) = self.windows.get(window) else {
            return;
        };

        native_window.set_ime_allowed(enabled);
        if let Some(rect) = cursor_rect {
            native_window.set_ime_cursor_area(rect);
        }
    }

    fn sync_cursor_for_window(&mut self, window: window::Id) {
        let cursor = self
            .window_states
            .get(&window)
            .map(|state| state.cursor_for_pointer(&mut self.text_engine))
            .unwrap_or_default();

        let Some(native_window) = self.windows.get(window) else {
            self.cursors.remove(&window);
            return;
        };

        if self.cursors.get(&window).copied() == Some(cursor) {
            return;
        }

        native_window.set_cursor(cursor);
        self.cursors.insert(window, cursor);
    }

    #[track_caller]
    fn invalidate_scroll(&mut self, window: window::Id) {
        if self.windows.contains(window) {
            if std::env::var_os("WGPU_L3_SCROLL_TRACE").is_some() {
                let location = std::panic::Location::caller();
                eprintln!(
                    "[wgpu_l3 frame] invalidate scroll window={window:?} at {}:{}",
                    location.file(),
                    location.line()
                );
            }
            self.window_states
                .entry(window)
                .or_default()
                .invalidate_frame(frame::RedrawKind::ScrollOnly, Instant::now());
        }
    }

    #[track_caller]
    fn invalidate_full(&mut self, window: window::Id) {
        if self.windows.contains(window) {
            if std::env::var_os("WGPU_L3_SCROLL_TRACE").is_some() {
                let location = std::panic::Location::caller();
                eprintln!(
                    "[wgpu_l3 frame] invalidate full window={window:?} at {}:{}",
                    location.file(),
                    location.line()
                );
            }
            self.window_states
                .entry(window)
                .or_default()
                .invalidate_frame(frame::RedrawKind::Full, Instant::now());
        }
    }

    fn flush_frame_invalidations(&mut self) {
        let windows = self.window_states.keys().copied().collect::<Vec<_>>();
        let mut requests = Vec::new();

        for window in windows {
            if !self.windows.contains(window) {
                continue;
            }

            let Some(state) = self.window_states.get_mut(&window) else {
                continue;
            };
            if !state.frame.needs_native_redraw() {
                continue;
            }

            if state.frame.pending_redraw_kind() == Some(frame::RedrawKind::ScrollOnly) {
                state.scroll.record_scroll_redraw_request();
            }
            state.frame.native_redraw_requested();
            requests.push(window);
        }

        for window in requests {
            self.windows.request_redraw(window);
        }
    }

    fn sync_native_pointer_capture_for_capture(
        &mut self,
        window: window::Id,
        had_capture: bool,
        has_capture: bool,
    ) {
        if had_capture != has_capture {
            self.set_native_pointer_capture(window, has_capture);
        }
    }

    fn set_native_pointer_capture(&mut self, window: window::Id, captured: bool) {
        if captured {
            if self.native_pointer_captures.contains(&window) {
                return;
            }
            let Some(native_window) = self.windows.get(window) else {
                return;
            };
            if native_window.capture_pointer(native::PointerCaptureKind::EventStream)
                == native::PointerCaptureStatus::Active
            {
                self.native_pointer_captures.insert(window);
            }
        } else {
            self.release_native_pointer_capture(window);
        }
    }

    fn release_native_pointer_capture(&mut self, window: window::Id) {
        let was_captured = self.native_pointer_captures.remove(&window);
        if !was_captured {
            return;
        }
        if let Some(native_window) = self.windows.get(window) {
            native_window.release_pointer_capture();
        }
    }

    fn dispatch_ui_events(
        &mut self,
        event_loop: &ActiveEventLoop,
        window: window::Id,
        events: Vec<ui::Event>,
    ) {
        for event in events {
            self.dispatch_ui_event(event_loop, window, event);
        }
    }
}

impl<A: Application> ApplicationHandler<Message<A::Event>> for Runtime<A> {
    fn new_events(&mut self, _event_loop: &ActiveEventLoop, _cause: StartCause) {}

    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if !self.rendering.ready() {
            if let Err(error) = self.rendering.initialize() {
                self.fail(event_loop, error.into());
                return;
            }
        }

        if self.started {
            return;
        }

        self.started = true;

        let mut cx = context::new(context::Parts {
            rendering: &mut self.rendering,
            windows: &mut self.windows,
            window_states: &mut self.window_states,
            commands: &mut self.commands,
            text_editor: &mut self.text_editor,
            text_engine: &mut self.text_engine,
            clipboard: &mut self.clipboard,
            mailbox: &mut self.mailbox,
            sender: self.sender.clone(),
            redraw_on_command_state_change: true,
            event_loop,
        });

        self.app.started(&mut cx);

        self.drain_mailbox(event_loop);
    }

    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        raw_window: winit::window::WindowId,
        event: WindowEvent,
    ) {
        let Some(window) = self.windows.raw_id(raw_window) else {
            return;
        };

        match event {
            WindowEvent::CloseRequested => {
                self.dispatch_ui_event(event_loop, window, ui::Event::CloseRequested);

                if self.windows.contains(window) {
                    self.close_window(event_loop, window);
                }
            }
            WindowEvent::Resized(size) => {
                let area = area::physical(size.width, size.height);
                let Some(native_window) = self.windows.get_mut(window) else {
                    return;
                };

                let scale_factor = native_window.scale_factor() as f32;
                self.rendering.resize(native_window, area, scale_factor);
                self.invalidate_full(window);

                self.dispatch_ui_event(
                    event_loop,
                    window,
                    ui::Event::Resized { area, scale_factor },
                );
            }
            WindowEvent::ScaleFactorChanged { scale_factor, .. } => {
                let Some(native_window) = self.windows.get_mut(window) else {
                    return;
                };

                let area = native_window.inner_area();
                let scale_factor = scale_factor as f32;
                self.rendering.resize(native_window, area, scale_factor);
                self.invalidate_full(window);

                self.dispatch_ui_event(
                    event_loop,
                    window,
                    ui::Event::ScaleFactorChanged { scale_factor },
                );
            }
            WindowEvent::Focused(focused) => {
                if !focused {
                    self.key_repeat.clear_window(window);
                    let had_scroll_capture = self
                        .window_states
                        .get(&window)
                        .is_some_and(WindowState::pointer_capture_is_scroll_thumb);
                    if let Some(state) = self.window_states.get_mut(&window) {
                        state.clear_pointer_capture();
                        state.pressed = None;
                        state.pressed_source = None;
                    }
                    self.sync_native_pointer_capture_for_capture(window, had_scroll_capture, false);
                }
                self.dispatch_ui_event(event_loop, window, ui::Event::Focused(focused));
            }
            WindowEvent::CursorMoved { position, .. } => {
                let Some(native_window) = self.windows.get(window) else {
                    return;
                };

                let position = point::physical(position.x as f32, position.y as f32)
                    .to_logical(native_window.scale_factor() as f32);

                self.pointer_moved(event_loop, window, position);
            }
            WindowEvent::CursorLeft { .. } => {
                let Some(state) = self.window_states.get_mut(&window) else {
                    return;
                };

                let had_scroll_capture = state.pointer_capture_is_scroll_thumb();
                let outcome = input::pointer_left(state);
                let has_scroll_capture = state.pointer_capture_is_scroll_thumb();

                if outcome.redraw {
                    self.invalidate_full(window);
                }

                self.sync_native_pointer_capture_for_capture(
                    window,
                    had_scroll_capture,
                    has_scroll_capture,
                );
                self.sync_cursor_for_window(window);
                self.dispatch_ui_events(event_loop, window, outcome.events);
            }
            WindowEvent::MouseInput { state, button, .. } => {
                self.pointer_button(event_loop, window, state, button);
            }
            WindowEvent::MouseWheel { delta, phase, .. } => {
                self.mouse_wheel(event_loop, window, delta, phase);
            }
            WindowEvent::ModifiersChanged(modifiers) => {
                self.modifiers_changed(window, modifiers);
            }
            WindowEvent::KeyboardInput {
                event,
                is_synthetic: false,
                ..
            } => {
                self.keyboard_input(event_loop, window, event);
            }
            WindowEvent::Ime(event) => {
                self.ime_input(event_loop, window, event);
            }
            WindowEvent::RedrawRequested => {
                self.redraw_window(event_loop, window);
            }
            _ => {}
        }
    }

    fn user_event(&mut self, event_loop: &ActiveEventLoop, message: Message<A::Event>) {
        self.dispatch_message(event_loop, message);
    }

    fn about_to_wait(&mut self, event_loop: &ActiveEventLoop) {
        let now = Instant::now();
        self.dispatch_due_key_repeat(event_loop, now);
        self.invalidate_due_animation_frames(now);
        let key_repeat_schedule = if self.key_repeat_policy.uses_client_timer() {
            self.key_repeat.schedule()
        } else {
            animation::Schedule::Idle
        };
        let schedule = self
            .refresh_animation_schedules(now)
            .merge(key_repeat_schedule);

        self.flush_frame_invalidations();
        event_loop.set_control_flow(schedule.control_flow(now));
    }
}

fn trace_presented_scroll_frame(
    kind: frame::RedrawKind,
    paint_timings: frame::StageTimings,
    draw_report: crate::render::renderer::DrawReport,
    total: Duration,
    scroll: crate::app::scroll::Diagnostics,
    text: text::layout::Diagnostics,
) {
    if kind != frame::RedrawKind::ScrollOnly || std::env::var_os("WGPU_L3_SCROLL_TRACE").is_none() {
        return;
    }

    eprintln!(
        concat!(
            "[wgpu_l3 scroll-frame] ",
            "commit={}us projection={}us paint={}us ",
            "render={}us text={}us scene_text={}us layer_text={}us total={}us ",
            "items={} text_surfaces={} glyph_batches={} layers={}/{} ",
            "metrics_calls={} layout_calls={} interaction_surfaces={} segments={} overscan={} ",
            "text_surface_build calls={} hit/miss={}/{} lines={} bytes={} ",
            "build_us anchor/text/buffer/attrs/size/shape/meta/total={}/{}/{}/{}/{}/{}/{}/{} ",
            "line_cache={}/{} shaped={} ",
            "text_projection resolves={} reuses={} shifts={} shift_miss={} cold={} ",
            "scroll_events wheel={} thumb={} commits={} shifts={} shift_miss={} cold={} ",
            "retained hit={} miss={}/{}/{}/{} rebuild={} fallback={} skips={}"
        ),
        duration_micros(paint_timings.scroll_commit),
        duration_micros(paint_timings.scroll_projection_sync),
        duration_micros(paint_timings.paint),
        duration_micros(draw_report.timings.total),
        duration_micros(draw_report.timings.text_prepare),
        duration_micros(draw_report.timings.scene_text_prepare),
        duration_micros(draw_report.timings.layer_update_text_prepare),
        duration_micros(total),
        draw_report.stats.scene_items,
        draw_report.stats.text_surfaces,
        draw_report.stats.glyph_batches,
        draw_report.stats.layer_items,
        draw_report.stats.layer_updates,
        text.text_area_metrics_layout_calls,
        text.text_area_paint_layout_calls,
        text.text_area_interaction_surfaces,
        text.text_area_layout_segments,
        text.text_area_overscan_segments,
        text.text_area_render_surface_calls,
        text.text_area_render_surface_cache_hits,
        text.text_area_render_surface_cache_misses,
        text.text_area_render_surface_source_lines,
        text.text_area_render_surface_source_bytes,
        text.text_area_render_surface_anchor_us,
        text.text_area_render_surface_text_us,
        text.text_area_render_surface_buffer_us,
        text.text_area_render_surface_attrs_us,
        text.text_area_render_surface_size_us,
        text.text_area_render_surface_shape_us,
        text.text_area_render_surface_metadata_us,
        text.text_area_render_surface_total_us,
        text.text_area_line_cache_hits,
        text.text_area_line_cache_misses,
        text.text_area_shaped_visual_lines,
        scroll.text_area_resolves,
        scroll.text_area_projection_reuses,
        scroll.text_area_projection_shifts,
        scroll.text_area_projection_shift_misses,
        scroll.text_area_projection_cold_jumps,
        scroll.last_scroll.wheel_events,
        scroll.last_scroll.thumb_drag_moves,
        scroll.frame_scroll_commits,
        scroll.text_area_projection_shifts,
        scroll.text_area_projection_shift_misses,
        scroll.text_area_projection_cold_jumps,
        scroll.last_scroll.retained_scroll_layer_hits,
        scroll.last_scroll.retained_scroll_layer_missing,
        scroll.last_scroll.retained_scroll_layer_metrics_misses,
        scroll.last_scroll.retained_scroll_layer_coverage_misses,
        scroll.last_scroll.retained_scroll_layer_geometry_misses,
        scroll.last_scroll.retained_scroll_layer_rebuilds,
        scroll.last_scroll.retained_scroll_target_repaint_fallbacks,
        scroll.async_scroll_projection_sync_skips,
    );
}

fn wheel_phase(phase: TouchPhase) -> crate::app::scroll::WheelPhase {
    match phase {
        TouchPhase::Started => crate::app::scroll::WheelPhase::Started,
        TouchPhase::Moved => crate::app::scroll::WheelPhase::Moved,
        TouchPhase::Ended => crate::app::scroll::WheelPhase::Ended,
        TouchPhase::Cancelled => crate::app::scroll::WheelPhase::Cancelled,
    }
}

fn trace_scroll_input(args: std::fmt::Arguments<'_>) {
    if std::env::var_os("WGPU_L3_SCROLL_TRACE").is_some() {
        eprintln!("[wgpu_l3 scroll-input] {args}");
    }
}

fn duration_micros(duration: Duration) -> u128 {
    duration.as_micros()
}
