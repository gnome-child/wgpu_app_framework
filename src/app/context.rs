use std::collections::HashMap;

use winit::event_loop::ActiveEventLoop;

use crate::app::mailbox::Mailbox;
use crate::app::state::{WindowState, resolve_action_path};
use crate::geometry::area;
use crate::{Action, action, native, render, ui, window};

use super::Result;

pub struct Context<'a, T> {
    render_context: &'a render::Context,
    renderer: &'a mut Option<render::Renderer>,
    windows: &'a mut HashMap<window::Id, native::Window>,
    raw_windows: &'a mut HashMap<winit::window::WindowId, window::Id>,
    window_states: &'a mut HashMap<window::Id, WindowState>,
    next_window_id: &'a mut u64,
    actions: &'a mut action::Registry<T>,
    mailbox: &'a mut Mailbox<T>,
    redraw_on_action_state_change: bool,
    event_loop: &'a ActiveEventLoop,
}

pub struct Parts<'a, T> {
    pub render_context: &'a render::Context,
    pub renderer: &'a mut Option<render::Renderer>,
    pub windows: &'a mut HashMap<window::Id, native::Window>,
    pub raw_windows: &'a mut HashMap<winit::window::WindowId, window::Id>,
    pub window_states: &'a mut HashMap<window::Id, WindowState>,
    pub next_window_id: &'a mut u64,
    pub actions: &'a mut action::Registry<T>,
    pub mailbox: &'a mut Mailbox<T>,
    pub redraw_on_action_state_change: bool,
    pub event_loop: &'a ActiveEventLoop,
}

pub fn new<T>(parts: Parts<'_, T>) -> Context<'_, T> {
    Context {
        render_context: parts.render_context,
        renderer: parts.renderer,
        windows: parts.windows,
        raw_windows: parts.raw_windows,
        window_states: parts.window_states,
        next_window_id: parts.next_window_id,
        actions: parts.actions,
        mailbox: parts.mailbox,
        redraw_on_action_state_change: parts.redraw_on_action_state_change,
        event_loop: parts.event_loop,
    }
}

impl<T> Context<'_, T> {
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
        };

        let handle = native::Window::open(native_options, self.event_loop)?;
        let canvas = render::Canvas::new(
            render::canvas::Options {
                area: area::physical(handle.inner_size().width, handle.inner_size().height),
                scale_factor: handle.scale_factor() as f32,
                color: render::color_to_wgpu(options.canvas_color),
            },
            self.render_context,
            handle.clone(),
        )?;
        let mut native_window = native::Window::new(handle, canvas);

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
            &*self.windows,
            window,
            action,
            self.redraw_on_action_state_change,
        )
    }

    pub fn action_state(&self, action: action::Id, context: action::Context) -> action::State {
        self.actions.state(action, context)
    }

    pub fn emit(&mut self, event: T) {
        self.mailbox.push_app(event);
    }

    pub fn invoke_action(&mut self, action: action::Id, context: action::Context) {
        self.mailbox.run_action(action::Invocation::new(
            action,
            action::Source::Programmatic,
            context,
        ));
    }

    pub fn hovered(&self, window: window::Id) -> Option<ui::Path> {
        self.window_states
            .get(&window)
            .and_then(|state| state.hovered.clone())
    }

    pub fn focused(&self, window: window::Id) -> Option<ui::Path> {
        self.window_states
            .get(&window)
            .and_then(|state| state.focused.clone())
    }

    pub fn resolve_action_context(
        &self,
        window: window::Id,
        requested_scope: Option<action::Scope>,
    ) -> action::Context {
        if let Some(scope) = requested_scope {
            return action::Context::with_scope(window, scope);
        }

        let scope = resolve_action_path(self.window_states.get(&window), None)
            .map(action::Scope::Path)
            .unwrap_or(action::Scope::Window);

        action::Context::with_scope(window, scope)
    }
}

pub struct ActionState<'a, T> {
    actions: &'a mut action::Registry<T>,
    windows: &'a HashMap<window::Id, native::Window>,
    window: window::Id,
    action: action::Id,
    state: action::State,
    changed: bool,
    redraw_on_action_state_change: bool,
}

impl<'a, T> ActionState<'a, T> {
    fn new(
        actions: &'a mut action::Registry<T>,
        windows: &'a HashMap<window::Id, native::Window>,
        window: window::Id,
        action: action::Id,
        redraw_on_action_state_change: bool,
    ) -> Self {
        let state = actions.state(action, action::Context::window(window));

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
            if let Some(window) = self.windows.get(&self.window) {
                window.request_redraw();
            }
        }
    }
}
