use winit::event_loop::ActiveEventLoop;

use crate::app::mailbox::Mailbox;
use crate::app::rendering;
use crate::app::sender::Sender;
use crate::app::state::WindowState;
use crate::app::task_runner;
use crate::app::windows::Windows;
use crate::geometry::area;
use crate::{Action, Task, action, ui, window};

use super::Result;

pub struct Context<'a, T: Send + 'static> {
    rendering: &'a mut rendering::Driver,
    windows: &'a mut Windows,
    window_states: &'a mut std::collections::HashMap<window::Id, WindowState>,
    actions: &'a mut action::Registry<T>,
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
    pub mailbox: &'a mut Mailbox<T>,
    pub sender: Sender<T>,
    pub redraw_on_action_state_change: bool,
    pub event_loop: &'a ActiveEventLoop,
}

pub fn new<T: Send + 'static>(parts: Parts<'_, T>) -> Context<'_, T> {
    Context {
        rendering: parts.rendering,
        windows: parts.windows,
        window_states: parts.window_states,
        actions: parts.actions,
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

    pub fn emit(&mut self, event: T) {
        self.mailbox.push_app(event);
    }

    pub fn spawn(&self, task: Task<T>) {
        task_runner::spawn(task, self.sender.clone());
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

    pub fn command_target(&self, window: window::Id) -> action::Context {
        self.command_subject(window)
    }

    pub fn command_subject(&self, window: window::Id) -> action::Context {
        self.window_states
            .get(&window)
            .map(|state| state.command_context(window))
            .unwrap_or_else(|| action::Context::window(window))
    }

    pub fn set_command_target(&mut self, window: window::Id, context: action::Context) {
        self.set_command_subject(window, context);
    }

    pub fn set_command_subject(&mut self, window: window::Id, context: action::Context) {
        if context.window_id() != window {
            return;
        }

        let Some(state) = self.window_states.get_mut(&window) else {
            return;
        };

        if state.set_command_target(context) {
            self.request_redraw(window);
        }
    }

    pub fn clear_command_target(&mut self, window: window::Id) {
        self.clear_command_subject(window);
    }

    pub fn clear_command_subject(&mut self, window: window::Id) {
        let Some(state) = self.window_states.get_mut(&window) else {
            return;
        };

        if state.clear_command_target() {
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
