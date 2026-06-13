mod state;

use std::collections::{HashMap, VecDeque};

use thiserror::Error;
use winit::{
    application::ApplicationHandler,
    event::{ElementState, MouseButton, WindowEvent},
    event_loop::{ActiveEventLoop, EventLoop},
};

use crate::app::state::{WindowState, action_invocation_event, resolve_action_target};
use crate::geometry::{area, point};
use crate::{Action, action, native, paint, render, ui, window};

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, Error)]
pub enum Error {
    #[error(transparent)]
    EventLoop(#[from] winit::error::EventLoopError),

    #[error(transparent)]
    Native(#[from] native::Error),

    #[error(transparent)]
    Render(#[from] render::Error),
}

pub trait Application {
    fn started(&mut self, _cx: &mut Context<'_>) {}

    fn event(&mut self, _cx: &mut Context<'_>, _window: window::Id, _event: ui::Event) {}

    fn view(&mut self, _cx: &mut Context<'_>, _window: window::Id, _tree: &mut ui::Tree) {}
}

pub fn run<A: Application>(app: A) -> Result<()> {
    let event_loop = EventLoop::new()?;
    let mut runtime = Runtime::new(app);

    event_loop.run_app(&mut runtime)?;

    if let Some(error) = runtime.error {
        return Err(error);
    }

    Ok(())
}

pub struct Context<'a> {
    render_context: &'a render::Context,
    renderer: &'a mut Option<render::Renderer>,
    windows: &'a mut HashMap<window::Id, native::Window>,
    raw_windows: &'a mut HashMap<winit::window::WindowId, window::Id>,
    window_states: &'a mut HashMap<window::Id, WindowState>,
    next_window_id: &'a mut u64,
    actions: &'a mut action::Registry,
    pending_events: &'a mut VecDeque<(window::Id, ui::Event)>,
    redraw_on_action_state_change: bool,
    event_loop: &'a ActiveEventLoop,
}

impl Context<'_> {
    pub fn open_window(&mut self, options: window::Options) -> window::Id {
        self.try_open_window(options)
            .expect("failed to open framework window")
    }

    pub fn try_open_window(&mut self, options: window::Options) -> Result<window::Id> {
        let id = window::Id::new(*self.next_window_id);
        *self.next_window_id += 1;

        let native_options = native::window::Options {
            title: options.title,
            inner_area: options.inner_area,
            canvas_color: render::color_to_wgpu(options.canvas_color),
        };

        let mut native_window =
            native::Window::new(id, native_options, self.render_context, self.event_loop)?;

        if self.renderer.is_none() {
            let format = native_window.canvas().surface().config().format;
            *self.renderer = Some(render::Renderer::new(self.render_context, format));
        }

        let renderer = self
            .renderer
            .as_mut()
            .expect("renderer should be initialized after opening a window");

        use render::frame::Status::*;
        match renderer.clear(self.render_context, native_window.canvas_mut())? {
            Presented => {}
            Skipped(reason) => {
                log::warn!("initial frame was skipped: {:#?}", reason);
            }
        }

        native_window.set_visibility(true);
        native_window.request_redraw();

        self.raw_windows.insert(native_window.raw_id(), id);
        self.windows.insert(id, native_window);
        self.window_states.entry(id).or_default();

        Ok(id)
    }

    pub fn request_redraw(&self, window: window::Id) {
        if let Some(window) = self.windows.get(&window) {
            window.request_redraw();
        }
    }

    pub fn close_window(&mut self, window: window::Id) {
        if let Some(native_window) = self.windows.remove(&window) {
            self.raw_windows.remove(&native_window.raw_id());
        }

        self.window_states.remove(&window);

        if self.windows.is_empty() {
            self.event_loop.exit();
        }
    }

    pub fn window_logical_area(&self, window: window::Id) -> Option<area::Logical> {
        self.windows
            .get(&window)
            .map(|window| window.canvas().logical_area())
    }

    pub fn window_physical_area(&self, window: window::Id) -> Option<area::Physical> {
        self.windows.get(&window).map(native::Window::inner_area)
    }

    pub fn register_action(&mut self, action: Action) {
        self.actions.register(action);
    }

    pub fn set_action_state(
        &mut self,
        action: action::Id,
        context: action::Context,
        state: action::State,
    ) {
        if self.actions.set_state(action, context, state) && self.redraw_on_action_state_change {
            self.request_redraw(context.window);
        }
    }

    pub fn action_state(&self, action: action::Id, context: action::Context) -> action::State {
        self.actions.state(
            action,
            self.resolve_action_context(context.window, context.target),
        )
    }

    pub fn invoke_action(&mut self, action: action::Id, context: action::Context) {
        let context = self.resolve_action_context(context.window, context.target);

        if !self.actions.can_invoke(action, context) {
            return;
        }

        self.pending_events.push_back((
            context.window,
            ui::Event::ActionInvoked {
                action,
                source: action::Source::Programmatic,
                context,
            },
        ));
    }

    pub fn hovered(&self, window: window::Id) -> Option<ui::Id> {
        self.window_states
            .get(&window)
            .and_then(|state| state.hovered)
    }

    pub fn focused(&self, window: window::Id) -> Option<ui::Id> {
        self.window_states
            .get(&window)
            .and_then(|state| state.focused)
    }

    pub fn resolve_action_context(
        &self,
        window: window::Id,
        requested_target: Option<ui::Id>,
    ) -> action::Context {
        if requested_target.is_some() {
            return action::Context {
                window,
                target: requested_target,
            };
        }

        let target = resolve_action_target(self.window_states.get(&window), requested_target);

        action::Context { window, target }
    }
}

struct Runtime<A> {
    app: A,
    render_context: Option<render::Context>,
    renderer: Option<render::Renderer>,
    windows: HashMap<window::Id, native::Window>,
    raw_windows: HashMap<winit::window::WindowId, window::Id>,
    window_states: HashMap<window::Id, WindowState>,
    actions: action::Registry,
    next_window_id: u64,
    started: bool,
    error: Option<Error>,
}

impl<A> Runtime<A> {
    fn new(app: A) -> Self {
        Self {
            app,
            render_context: None,
            renderer: None,
            windows: HashMap::new(),
            raw_windows: HashMap::new(),
            window_states: HashMap::new(),
            actions: action::Registry::new(),
            next_window_id: 1,
            started: false,
            error: None,
        }
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
}

impl<A: Application> Runtime<A> {
    fn dispatch_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        window: window::Id,
        event: ui::Event,
    ) {
        let mut queue = VecDeque::from([(window, event)]);

        while let Some((window, event)) = queue.pop_front() {
            let Some(render_context) = self.render_context.as_ref() else {
                return;
            };
            let mut pending_events = VecDeque::new();

            {
                let mut cx = Context {
                    render_context,
                    renderer: &mut self.renderer,
                    windows: &mut self.windows,
                    raw_windows: &mut self.raw_windows,
                    window_states: &mut self.window_states,
                    next_window_id: &mut self.next_window_id,
                    actions: &mut self.actions,
                    pending_events: &mut pending_events,
                    redraw_on_action_state_change: true,
                    event_loop,
                };

                self.app.event(&mut cx, window, event);
            }

            queue.extend(pending_events);
        }
    }

    fn redraw_window(&mut self, event_loop: &ActiveEventLoop, window: window::Id) {
        let Some(render_context) = self.render_context.as_ref() else {
            return;
        };

        let mut tree = ui::Tree::new();
        let mut pending_events = VecDeque::new();

        self.actions.clear_context_states(window);

        {
            let mut cx = Context {
                render_context,
                renderer: &mut self.renderer,
                windows: &mut self.windows,
                raw_windows: &mut self.raw_windows,
                window_states: &mut self.window_states,
                next_window_id: &mut self.next_window_id,
                actions: &mut self.actions,
                pending_events: &mut pending_events,
                redraw_on_action_state_change: false,
                event_loop,
            };

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
            let interaction = ui::Interaction {
                hovered: state.hovered,
                focused: state.focused,
                pressed: state.pressed,
            };
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

        for (window, event) in pending_events {
            self.dispatch_event(event_loop, window, event);
        }
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
        let hover_events = state.set_hovered(target);
        state.cursor_position = Some(position);

        if !hover_events.is_empty() {
            if let Some(window) = self.windows.get(&window) {
                window.request_redraw();
            }
        }

        for event in hover_events {
            self.dispatch_event(event_loop, window, event);
        }

        self.dispatch_event(
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

                self.dispatch_event(event_loop, window, event);
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

                self.dispatch_event(event_loop, window, event);

                if let Some(target) = invoke_target {
                    if let Some(event) = action_invocation_event(
                        &self.actions,
                        &actions,
                        window,
                        target,
                        action::Source::Pointer,
                    ) {
                        self.dispatch_event(event_loop, window, event);
                    }
                }

                if let Some(native_window) = self.windows.get(&window) {
                    native_window.request_redraw();
                }
            }
        }
    }
}

impl<A: Application> ApplicationHandler for Runtime<A> {
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

        let mut pending_events = VecDeque::new();
        let mut cx = Context {
            render_context,
            renderer: &mut self.renderer,
            windows: &mut self.windows,
            raw_windows: &mut self.raw_windows,
            window_states: &mut self.window_states,
            next_window_id: &mut self.next_window_id,
            actions: &mut self.actions,
            pending_events: &mut pending_events,
            redraw_on_action_state_change: true,
            event_loop,
        };

        self.app.started(&mut cx);

        for (window, event) in pending_events {
            self.dispatch_event(event_loop, window, event);
        }
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
                self.dispatch_event(event_loop, window, ui::Event::CloseRequested);

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

                self.dispatch_event(
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

                self.dispatch_event(
                    event_loop,
                    window,
                    ui::Event::ScaleFactorChanged { scale_factor },
                );
            }
            WindowEvent::Focused(focused) => {
                self.dispatch_event(event_loop, window, ui::Event::Focused(focused));
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
