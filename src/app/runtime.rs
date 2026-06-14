use std::collections::HashMap;

use winit::{
    application::ApplicationHandler,
    event::{ElementState, MouseButton, WindowEvent},
    event_loop::ActiveEventLoop,
};

use crate::app::context;
use crate::app::mailbox::{Mailbox, Message};
use crate::app::sender::Sender;
use crate::app::state::{WindowState, action_invocation};
use crate::geometry::{area, point};
use crate::{action, event, native, paint, render, ui, window};

use super::{Application, Error};

pub struct Runtime<A: Application> {
    app: A,
    render_context: Option<render::Context>,
    renderer: Option<render::Renderer>,
    windows: HashMap<window::Id, native::Window>,
    raw_windows: HashMap<winit::window::WindowId, window::Id>,
    window_states: HashMap<window::Id, WindowState>,
    actions: action::Registry<A::Event>,
    mailbox: Mailbox<A::Event>,
    sender: Sender<A::Event>,
    next_window_id: u64,
    started: bool,
    error: Option<Error>,
}

impl<A: Application> Runtime<A> {
    pub fn new(app: A, sender: Sender<A::Event>) -> Self {
        Self {
            app,
            render_context: None,
            renderer: None,
            windows: HashMap::new(),
            raw_windows: HashMap::new(),
            window_states: HashMap::new(),
            actions: action::Registry::new(),
            mailbox: Mailbox::new(),
            sender,
            next_window_id: 1,
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

    fn render_options() -> render::context::Options {
        render::context::Options {
            device_label: "wgpu_l3 device",
            backends: wgpu::Backends::all(),
            power_preference: wgpu::PowerPreference::HighPerformance,
            force_fallback_adapter: false,
            required_features: wgpu::Features::empty(),
            required_limits: wgpu::Limits::default(),
        }
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
        if self.render_context.is_none() {
            return;
        }

        while let Some(message) = self.mailbox.pop() {
            match message {
                Message::Event(event) => {
                    let render_context = self
                        .render_context
                        .as_ref()
                        .expect("render context should exist while draining mailbox");
                    let mut cx = context::new(context::Parts {
                        render_context,
                        renderer: &mut self.renderer,
                        windows: &mut self.windows,
                        raw_windows: &mut self.raw_windows,
                        window_states: &mut self.window_states,
                        next_window_id: &mut self.next_window_id,
                        actions: &mut self.actions,
                        mailbox: &mut self.mailbox,
                        sender: self.sender.clone(),
                        redraw_on_action_state_change: true,
                        event_loop,
                    });

                    self.app.event(&mut cx, event);
                }
                Message::RunAction(invocation) => {
                    self.run_action(invocation);
                }
            }
        }
    }

    fn run_action(&mut self, invocation: action::Invocation) {
        let action = invocation.action();
        let context = invocation.context().clone();
        let window = context.window_id();

        if !self.actions.can_invoke(action, context) {
            return;
        }

        self.request_redraw_if_open(window);
        let Some(effect) = self.actions.execute(invocation) else {
            return;
        };
        self.request_redraw_if_open(window);
        self.enqueue_effect(effect);
    }

    fn enqueue_effect(&mut self, effect: action::Effect<A::Event>) {
        match effect {
            action::Effect::None => {}
            action::Effect::Emit(event) => {
                self.mailbox.push_app(event);
            }
            action::Effect::Batch(events) => {
                for event in events {
                    self.mailbox.push_app(event);
                }
            }
        }
    }

    fn request_redraw_if_open(&self, window: window::Id) {
        if let Some(window) = self.windows.get(&window) {
            window.request_redraw();
        }
    }

    fn redraw_window(&mut self, event_loop: &ActiveEventLoop, window: window::Id) {
        let Some(render_context) = self.render_context.as_ref() else {
            return;
        };

        let mut tree = ui::Tree::new();

        self.actions.clear_context_states(window);

        {
            let mut cx = context::new(context::Parts {
                render_context,
                renderer: &mut self.renderer,
                windows: &mut self.windows,
                raw_windows: &mut self.raw_windows,
                window_states: &mut self.window_states,
                next_window_id: &mut self.next_window_id,
                actions: &mut self.actions,
                mailbox: &mut self.mailbox,
                sender: self.sender.clone(),
                redraw_on_action_state_change: false,
                event_loop,
            });

            self.app.view(&mut cx, window, &mut tree);
        }

        let Some(native_window) = self.windows.get(&window) else {
            return;
        };
        let logical_area = native_window.canvas().logical_area();
        let mut scene = paint::Scene::new();
        let state = self.window_states.entry(window).or_default();
        state.actions = tree.actions();
        state.interactivity = tree.interactivity();

        if let Some(layout) = tree.layout(logical_area) {
            let interaction = ui::Interaction::new(
                state.hovered.clone(),
                state.focused.clone(),
                state.pressed.clone(),
            );
            state.layout = Some(layout.clone());

            tree.paint(&layout, &self.actions, window, interaction, &mut scene);
        } else {
            state.layout = None;
        }

        let Some(native_window) = self.windows.get_mut(&window) else {
            return;
        };

        if self.renderer.is_none() {
            let format = native_window.canvas().surface().config().format;
            self.renderer = Some(render::Renderer::new(render_context, format));
        }

        let renderer = self
            .renderer
            .as_mut()
            .expect("renderer should be initialized before redraw");

        use render::frame::Status::*;
        match renderer.draw(render_context, native_window.canvas_mut(), &scene) {
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
        if let Some(native_window) = self.windows.remove(&window) {
            self.raw_windows.remove(&native_window.raw_id());
        }

        self.window_states.remove(&window);

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
        let target = state.hit_test(position);
        let hover_events = state.set_hovered(target.clone());
        state.cursor_position = Some(position);

        if !hover_events.is_empty() {
            if let Some(window) = self.windows.get(&window) {
                window.request_redraw();
            }
        }

        for event in hover_events {
            self.dispatch_ui_event(event_loop, window, event);
        }

        self.dispatch_ui_event(
            event_loop,
            window,
            ui::Event::PointerMoved { position, target },
        );
    }

    fn pointer_button(
        &mut self,
        event_loop: &ActiveEventLoop,
        window: window::Id,
        state: ElementState,
        button: MouseButton,
    ) {
        let Some(button) = pointer_button(button) else {
            return;
        };

        match state {
            ElementState::Pressed => {
                let Some(window_state) = self.window_states.get_mut(&window) else {
                    return;
                };
                let position = window_state
                    .cursor_position
                    .unwrap_or_else(|| point::logical(0.0, 0.0));
                let target = window_state.hit_test(position);
                let event = window_state.pointer_down(position, target, button);

                if let Some(native_window) = self.windows.get(&window) {
                    native_window.request_redraw();
                }

                self.dispatch_ui_event(event_loop, window, event);
            }
            ElementState::Released => {
                let Some(window_state) = self.window_states.get_mut(&window) else {
                    return;
                };
                let position = window_state
                    .cursor_position
                    .unwrap_or_else(|| point::logical(0.0, 0.0));
                let target = window_state.hit_test(position);
                let (event, invoke_target) = window_state.pointer_up(position, target, button);
                let actions = window_state.actions.clone();

                self.dispatch_ui_event(event_loop, window, event);

                if let Some(target) = invoke_target {
                    if let Some(invocation) = action_invocation(
                        &self.actions,
                        &actions,
                        window,
                        target,
                        action::Source::Pointer,
                    ) {
                        self.dispatch_message(event_loop, Message::RunAction(invocation));
                    }
                }

                if let Some(native_window) = self.windows.get(&window) {
                    native_window.request_redraw();
                }
            }
        }
    }
}

impl<A: Application> ApplicationHandler<Message<A::Event>> for Runtime<A> {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if self.render_context.is_none() {
            match pollster::block_on(render::Context::new(Self::render_options())) {
                Ok(render_context) => {
                    self.render_context = Some(render_context);
                }
                Err(error) => {
                    self.fail(event_loop, error.into());
                    return;
                }
            }
        }

        if self.started {
            return;
        }

        self.started = true;

        let Some(render_context) = self.render_context.as_ref() else {
            return;
        };

        let mut cx = context::new(context::Parts {
            render_context,
            renderer: &mut self.renderer,
            windows: &mut self.windows,
            raw_windows: &mut self.raw_windows,
            window_states: &mut self.window_states,
            next_window_id: &mut self.next_window_id,
            actions: &mut self.actions,
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
        let Some(&window) = self.raw_windows.get(&raw_window) else {
            return;
        };

        match event {
            WindowEvent::CloseRequested => {
                self.dispatch_ui_event(event_loop, window, ui::Event::CloseRequested);

                if self.windows.contains_key(&window) {
                    self.close_window(event_loop, window);
                }
            }
            WindowEvent::Resized(size) => {
                let area = area::physical(size.width, size.height);
                let Some(render_context) = self.render_context.as_ref() else {
                    return;
                };

                let Some(native_window) = self.windows.get_mut(&window) else {
                    return;
                };

                let scale_factor = native_window.scale_factor() as f32;
                native_window.resize(render_context, area, scale_factor);
                native_window.request_redraw();

                self.dispatch_ui_event(
                    event_loop,
                    window,
                    ui::Event::Resized { area, scale_factor },
                );
            }
            WindowEvent::ScaleFactorChanged { scale_factor, .. } => {
                let Some(render_context) = self.render_context.as_ref() else {
                    return;
                };

                let Some(native_window) = self.windows.get_mut(&window) else {
                    return;
                };

                let area = native_window.inner_area();
                let scale_factor = scale_factor as f32;
                native_window.resize(render_context, area, scale_factor);
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
                let Some(native_window) = self.windows.get(&window) else {
                    return;
                };

                let position = point::physical(position.x as f32, position.y as f32)
                    .to_logical(native_window.scale_factor() as f32);

                self.pointer_moved(event_loop, window, position);
            }
            WindowEvent::MouseInput { state, button, .. } => {
                self.pointer_button(event_loop, window, state, button);
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
}

fn pointer_button(button: MouseButton) -> Option<ui::Button> {
    match button {
        MouseButton::Left => Some(ui::Button::Left),
        MouseButton::Right => Some(ui::Button::Right),
        MouseButton::Middle => Some(ui::Button::Middle),
        MouseButton::Back | MouseButton::Forward => None,
        MouseButton::Other(value) => Some(ui::Button::Other(value)),
    }
}
