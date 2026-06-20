use std::collections::HashMap;
use std::time::Instant;

use winit::{
    application::ApplicationHandler,
    event::{ElementState, KeyEvent, MouseButton, MouseScrollDelta, StartCause, WindowEvent},
    event_loop::ActiveEventLoop,
};

use crate::animation;
use crate::app::action_executor;
use crate::app::clipboard::SystemClipboard;
use crate::app::context;
use crate::app::input;
use crate::app::mailbox::{Mailbox, Message};
use crate::app::rendering::Driver;
use crate::app::sender::Sender;
use crate::app::state::WindowState;
use crate::app::view;
use crate::app::windows::Windows;
use crate::geometry::{area, point};
use crate::{action, event, text, ui, window};

use super::{Application, Error};

pub struct Runtime<A: Application> {
    app: A,
    rendering: Driver,
    windows: Windows,
    window_states: HashMap<window::Id, WindowState>,
    actions: action::Registry<A::Event>,
    text_engine: text::Engine,
    clipboard: SystemClipboard,
    mailbox: Mailbox<A::Event>,
    sender: Sender<A::Event>,
    animation_schedules: HashMap<window::Id, animation::Schedule>,
    last_frames: HashMap<window::Id, Instant>,
    cursors: HashMap<window::Id, ui::Cursor>,
    started: bool,
    error: Option<Error>,
}

impl<A: Application> Runtime<A> {
    pub fn new(app: A, sender: Sender<A::Event>) -> Self {
        Self {
            app,
            rendering: Driver::new(),
            windows: Windows::new(),
            window_states: HashMap::new(),
            actions: action::Registry::new(),
            text_engine: text::Engine::new(),
            clipboard: SystemClipboard::new(),
            mailbox: Mailbox::new(),
            sender,
            animation_schedules: HashMap::new(),
            last_frames: HashMap::new(),
            cursors: HashMap::new(),
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
                        actions: &mut self.actions,
                        text_engine: &mut self.text_engine,
                        clipboard: &mut self.clipboard,
                        mailbox: &mut self.mailbox,
                        sender: self.sender.clone(),
                        redraw_on_action_state_change: true,
                        event_loop,
                    });

                    self.app.event(&mut cx, event);
                }
                Message::RunAction(request) => {
                    self.run_action(request);
                }
                Message::ActionTaskCompleted { invocation, event } => {
                    self.complete_action_task(invocation, event);
                }
                Message::AppTaskCompleted(event) => {
                    self.complete_app_task(event);
                }
            }
        }
    }

    fn run_action(&mut self, request: action::Request) {
        let window = request.target().window_id();
        let request = self
            .window_states
            .get(&window)
            .map(|state| state.resolve_request(request.clone()))
            .unwrap_or(request);
        let windows = &self.windows;
        let mut request_redraw = |window| windows.request_redraw(window);

        let sender = self.sender.clone();
        if let Some(effect) = action_executor::execute(
            &mut self.actions,
            request,
            |invocation, task| action_executor::spawn_task(invocation, task, sender),
            &mut request_redraw,
        ) {
            action_executor::enqueue_effect(&mut self.mailbox, effect);
        }
    }

    fn complete_action_task(&mut self, invocation: action::Invocation, event: A::Event) {
        let windows = &self.windows;
        let mut request_redraw = |window| windows.request_redraw(window);

        action_executor::complete_task(&mut self.actions, invocation, &mut request_redraw);
        self.mailbox.push_app(event);
    }

    fn complete_app_task(&mut self, event: A::Event) {
        self.mailbox.push_app(event);
    }

    fn frame_for_window(&mut self, window: window::Id) -> animation::Frame {
        let now = Instant::now();
        let previous = self.last_frames.insert(window, now);

        animation::Frame::new(now, previous)
    }

    fn request_due_animation_redraws(&mut self, now: Instant) {
        let due = self
            .animation_schedules
            .iter()
            .filter_map(|(window, schedule)| schedule.is_due(now).then_some(*window))
            .collect::<Vec<_>>();

        for window in due {
            if self.windows.contains(window) {
                self.windows.request_redraw(window);
            } else {
                self.animation_schedules.remove(&window);
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

        let frame = self.frame_for_window(window);
        let mut tree = ui::Tree::new();

        self.actions.clear_context_states(window);

        {
            let mut cx = context::new(context::Parts {
                rendering: &mut self.rendering,
                windows: &mut self.windows,
                window_states: &mut self.window_states,
                actions: &mut self.actions,
                text_engine: &mut self.text_engine,
                clipboard: &mut self.clipboard,
                mailbox: &mut self.mailbox,
                sender: self.sender.clone(),
                redraw_on_action_state_change: false,
                event_loop,
            });

            self.app.view(&mut cx, window, &mut tree);
        }
        self.text_engine.reset_diagnostics();

        let Some(native_window) = self.windows.get(window) else {
            return;
        };
        let logical_area = native_window.canvas().logical_area();
        let state = self.window_states.entry(window).or_default();
        let scene = view::compose(
            window,
            &tree,
            state,
            &mut self.actions,
            &mut self.text_engine,
            logical_area,
            frame,
        );
        self.sync_ime_for_window(window);
        self.sync_cursor_for_window(window);

        let Some(native_window) = self.windows.get_mut(window) else {
            return;
        };

        use crate::render::frame::Status::*;
        match self.rendering.draw(native_window, &scene) {
            Ok(Presented) => {}
            Ok(Skipped(reason)) => {
                log::warn!("render pass was skipped: {:#?}", reason);
                native_window.request_redraw();
            }
            Err(error) => {
                self.fail(event_loop, error.into());
            }
        }

        self.drain_mailbox(event_loop);
    }

    fn close_window(&mut self, event_loop: &ActiveEventLoop, window: window::Id) {
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

        if outcome.redraw {
            self.windows.request_redraw(window);
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

        let outcome = match state {
            ElementState::Pressed => input::pointer_pressed(
                window_state,
                window,
                position,
                button,
                &mut self.text_engine,
            ),
            ElementState::Released => {
                input::pointer_released(&self.actions, window_state, window, position, button)
            }
        };

        self.dispatch_ui_events(event_loop, window, outcome.events);

        if let Some(request) = outcome.request {
            self.dispatch_message(event_loop, Message::RunAction(request));
        }

        if let Some(intent) = outcome.intent {
            self.handle_intent(window, intent);
        }

        if outcome.redraw {
            self.windows.request_redraw(window);
        }
        self.sync_ime_for_window(window);
        self.sync_cursor_for_window(window);
    }

    fn mouse_wheel(
        &mut self,
        event_loop: &ActiveEventLoop,
        window: window::Id,
        delta: MouseScrollDelta,
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
            MouseScrollDelta::LineDelta(x, y) => point::logical(x * 40.0, y * 40.0),
            MouseScrollDelta::PixelDelta(position) => {
                point::physical(position.x as f32, position.y as f32).to_logical(scale_factor)
            }
        };

        let Some(state) = self.window_states.get_mut(&window) else {
            return;
        };
        let outcome = input::scroll_wheel(state, position, delta, &mut self.text_engine);

        self.dispatch_ui_events(event_loop, window, outcome.events);
        if outcome.redraw {
            self.windows.request_redraw(window);
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
        let key = input::key(&event.logical_key);
        let Some(state) = self.window_states.get_mut(&window) else {
            return;
        };

        let outcome = match event.state {
            ElementState::Pressed => input::key_pressed_with_text(
                &self.actions,
                state,
                window,
                key,
                event.text.as_deref(),
                event.repeat,
                &mut self.text_engine,
            ),
            ElementState::Released => input::key_released(&self.actions, state, window, key),
        };

        self.dispatch_ui_events(event_loop, window, outcome.events);

        if let Some(request) = outcome.request {
            self.dispatch_message(event_loop, Message::RunAction(request));
        }

        if let Some(intent) = outcome.intent {
            self.handle_intent(window, intent);
        }

        if outcome.redraw {
            self.windows.request_redraw(window);
        }
        self.sync_ime_for_window(window);
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
            self.windows.request_redraw(window);
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
                if state.toggle_menu(menu, &self.actions, window, request.source) {
                    self.windows.request_redraw(window);
                }
            }
            ui::Intent::OpenSubmenu(menu) => {
                if state.open_submenu(menu, &self.actions, window, request.source) {
                    self.windows.request_redraw(window);
                }
            }
            ui::Intent::CloseSubmenu => {
                if state.close_submenu() {
                    self.windows.request_redraw(window);
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
            .map(WindowState::cursor_for_hovered)
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
    fn new_events(&mut self, _event_loop: &ActiveEventLoop, _cause: StartCause) {
        self.request_due_animation_redraws(Instant::now());
    }

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
            actions: &mut self.actions,
            text_engine: &mut self.text_engine,
            clipboard: &mut self.clipboard,
            mailbox: &mut self.mailbox,
            sender: self.sender.clone(),
            redraw_on_action_state_change: true,
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
                native_window.request_redraw();

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
                native_window.request_redraw();

                self.dispatch_ui_event(
                    event_loop,
                    window,
                    ui::Event::ScaleFactorChanged { scale_factor },
                );
            }
            WindowEvent::Focused(focused) => {
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

                let outcome = input::pointer_left(state);

                if outcome.redraw {
                    self.windows.request_redraw(window);
                }

                self.sync_cursor_for_window(window);
                self.dispatch_ui_events(event_loop, window, outcome.events);
            }
            WindowEvent::MouseInput { state, button, .. } => {
                self.pointer_button(event_loop, window, state, button);
            }
            WindowEvent::MouseWheel { delta, .. } => {
                self.mouse_wheel(event_loop, window, delta);
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
        self.request_due_animation_redraws(now);
        let schedule = self.refresh_animation_schedules(now);

        event_loop.set_control_flow(schedule.control_flow(now));
    }
}
