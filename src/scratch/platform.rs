use std::{collections::HashMap, fmt, path::PathBuf};

use crate::geometry::area;
use crate::text;
use crate::{native, paint, render};
use thiserror::Error as ThisError;
use winit::{
    application::ApplicationHandler,
    dpi::{PhysicalPosition, PhysicalSize},
    event::{ElementState, Ime, MouseButton, MouseScrollDelta, WindowEvent as WinitWindowEvent},
    event_loop::{ActiveEventLoop, ControlFlow, EventLoop},
    keyboard::{Key as WinitKey, ModifiersState, NamedKey},
};

use super::{
    Error as FrameworkError, geometry, host, input, interaction, scene, session, shell,
    state::State, window,
};

pub struct Platform<M: State, E: Send + 'static = (), B = ()> {
    host: host::Host<M, E>,
    backend: B,
    active_requests: Vec<session::Request>,
    poll_scheduled: bool,
}

pub struct Runner<M: State, E: Send + 'static = (), B: Backend = Native> {
    platform: Platform<M, E, B>,
    events: Events,
    started: bool,
    error: Option<Error<B::Error>>,
}

pub struct Events {
    modifiers: input::Modifiers,
    default_scale_factor: f64,
    windows: HashMap<window::Id, WindowEvents>,
}

struct WindowEvents {
    scale_factor: f64,
    pointer: geometry::Point,
}

pub trait Backend {
    type Error;
    type Context<'a>;

    fn open_window(
        &mut self,
        context: &mut Self::Context<'_>,
        window: &Window,
    ) -> Result<(), Self::Error>;

    fn close_window(
        &mut self,
        context: &mut Self::Context<'_>,
        window: window::Id,
    ) -> Result<(), Self::Error>;

    fn present(
        &mut self,
        context: &mut Self::Context<'_>,
        presentation: &shell::Presentation,
    ) -> Result<(), Self::Error>;

    fn request(
        &mut self,
        context: &mut Self::Context<'_>,
        request: session::Request,
    ) -> Result<(), Self::Error>;

    fn schedule_poll(&mut self, context: &mut Self::Context<'_>) -> Result<(), Self::Error>;
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Window {
    id: window::Id,
    title: String,
    size: geometry::Size,
    canvas_color: scene::Color,
}

pub struct Native {
    context: Option<render::Context>,
    renderer: Option<render::Renderer>,
    windows: HashMap<window::Id, native::Window>,
    raw_windows: HashMap<winit::window::WindowId, window::Id>,
    requests: Vec<session::Request>,
    poll_requested: bool,
}

pub struct NativeContext<'a> {
    event_loop: &'a ActiveEventLoop,
}

#[derive(Debug)]
pub enum Error<E> {
    Framework(FrameworkError),
    Backend(E),
}

#[derive(Debug)]
pub enum RunError<E> {
    EventLoop(winit::error::EventLoopError),
    Platform(Error<E>),
}

#[derive(Debug, ThisError)]
pub enum NativeError {
    #[error("native window error")]
    Native(#[from] native::Error),

    #[error("render error")]
    Render(#[from] render::Error),

    #[error("native window is not open: {window:?}")]
    MissingWindow { window: window::Id },
}

impl<M: State, E: Send + 'static, B: Backend> Platform<M, E, B> {
    pub fn new(shell: shell::Shell<M, E>, backend: B) -> Self {
        Self::with_host(host::Host::new(shell), backend)
    }

    pub fn with_host(host: host::Host<M, E>, backend: B) -> Self {
        Self {
            host,
            backend,
            active_requests: Vec::new(),
            poll_scheduled: false,
        }
    }

    pub fn host(&self) -> &host::Host<M, E> {
        &self.host
    }

    pub fn host_mut(&mut self) -> &mut host::Host<M, E> {
        &mut self.host
    }

    pub fn backend(&self) -> &B {
        &self.backend
    }

    pub fn backend_mut(&mut self) -> &mut B {
        &mut self.backend
    }

    pub fn into_parts(self) -> (host::Host<M, E>, B) {
        (self.host, self.backend)
    }

    pub fn start(&mut self) -> Result<(), Error<B::Error>>
    where
        for<'a> B::Context<'a>: Default,
    {
        let mut context: B::Context<'_> = Default::default();
        self.handle_event_with(&mut context, host::Event::Started)
    }

    pub fn poll(&mut self) -> Result<(), Error<B::Error>>
    where
        for<'a> B::Context<'a>: Default,
    {
        let mut context: B::Context<'_> = Default::default();
        self.handle_event_with(&mut context, host::Event::Poll)
    }

    pub fn drain(&mut self) -> Result<(), Error<B::Error>>
    where
        for<'a> B::Context<'a>: Default,
    {
        let mut context: B::Context<'_> = Default::default();
        self.drain_with(&mut context)
    }

    pub fn handle_event(&mut self, event: host::Event) -> Result<(), Error<B::Error>>
    where
        for<'a> B::Context<'a>: Default,
    {
        let mut context: B::Context<'_> = Default::default();
        self.handle_event_with(&mut context, event)
    }

    pub fn start_with(&mut self, context: &mut B::Context<'_>) -> Result<(), Error<B::Error>> {
        self.handle_event_with(context, host::Event::Started)
    }

    pub fn poll_with(&mut self, context: &mut B::Context<'_>) -> Result<(), Error<B::Error>> {
        self.handle_event_with(context, host::Event::Poll)
    }

    pub fn drain_with(&mut self, context: &mut B::Context<'_>) -> Result<(), Error<B::Error>> {
        let work = self.host.drain();
        self.apply_work(context, &work).map_err(Error::Backend)
    }

    pub fn handle_event_with(
        &mut self,
        context: &mut B::Context<'_>,
        event: host::Event,
    ) -> Result<(), Error<B::Error>> {
        if matches!(&event, host::Event::Poll) {
            self.poll_scheduled = false;
        }

        let work = self.host.handle_event(event).map_err(Error::Framework)?;
        self.apply_work(context, &work).map_err(Error::Backend)
    }

    fn apply_work(
        &mut self,
        context: &mut B::Context<'_>,
        work: &shell::Work,
    ) -> Result<(), B::Error> {
        for window in work.closed_windows() {
            self.backend.close_window(context, *window)?;
        }

        for window in work.opened_windows() {
            self.backend
                .open_window(context, &Window::from_shell(window))?;
        }

        for presentation in work.presentations() {
            self.backend.present(context, presentation)?;
        }

        self.sync_requests(context, work.requests())?;
        self.sync_poll(context, work.needs_poll())?;

        Ok(())
    }

    fn sync_requests(
        &mut self,
        context: &mut B::Context<'_>,
        requests: &[session::Request],
    ) -> Result<(), B::Error> {
        self.active_requests
            .retain(|request| requests.contains(request));

        for request in requests {
            if self.active_requests.contains(request) {
                continue;
            }

            self.backend.request(context, *request)?;
            self.active_requests.push(*request);
        }

        Ok(())
    }

    fn sync_poll(
        &mut self,
        context: &mut B::Context<'_>,
        needs_poll: bool,
    ) -> Result<(), B::Error> {
        if !needs_poll {
            self.poll_scheduled = false;
            return Ok(());
        }

        if self.poll_scheduled {
            return Ok(());
        }

        self.backend.schedule_poll(context)?;
        self.poll_scheduled = true;
        Ok(())
    }
}

impl<M: State, E: Send + 'static> Runner<M, E, Native> {
    pub fn new(shell: shell::Shell<M, E>) -> Self {
        Self::with_platform(Platform::new(shell, Native::new()))
    }

    pub fn run(mut self) -> Result<(), RunError<NativeError>> {
        let event_loop = EventLoop::<E>::with_user_event().build()?;

        event_loop.run_app(&mut self)?;

        if let Some(error) = self.take_error() {
            return Err(error.into());
        }

        Ok(())
    }

    pub fn translate_window_event(
        &mut self,
        raw_window: winit::window::WindowId,
        event: &WinitWindowEvent,
    ) -> Option<host::Event> {
        let window = self.platform.backend().window_for_raw(raw_window)?;
        self.events.window_event(window, event)
    }

    fn sync_native_event_state(&mut self) {
        let windows = self
            .platform
            .host()
            .windows()
            .iter()
            .map(|window| window.id())
            .collect::<Vec<_>>();

        self.events
            .retain_windows(|window| windows.contains(&window));

        for window in windows {
            if let Some(scale_factor) = self.platform.backend().scale_factor(window) {
                self.events.set_window_scale_factor(window, scale_factor);
            }
        }
    }

    fn handle_native_requests(
        &mut self,
        event_loop: &ActiveEventLoop,
    ) -> Result<(), Error<NativeError>> {
        let requests = self.platform.backend_mut().take_requests();

        for request in requests {
            let path = native_file_dialog(request.kind());
            let event = file_dialog_selected(request, path);
            let mut context = NativeContext::new(event_loop);
            self.platform.handle_event_with(&mut context, event)?;
            self.sync_native_event_state();
        }

        Ok(())
    }

    fn finish_native_pass(&mut self, event_loop: &ActiveEventLoop) {
        self.sync_native_event_state();

        if let Err(error) = self.handle_native_requests(event_loop) {
            self.fail(event_loop, error);
            return;
        }

        if !self.exit_if_finished(event_loop) {
            self.sync_control_flow(event_loop);
        }
    }

    fn fail(&mut self, event_loop: &ActiveEventLoop, error: Error<NativeError>) {
        self.error = Some(error);
        event_loop.exit();
    }

    fn sync_control_flow(&self, event_loop: &ActiveEventLoop) {
        if event_loop.exiting() {
            return;
        }

        let control_flow = if self.platform.backend().poll_requested() {
            ControlFlow::Poll
        } else {
            ControlFlow::Wait
        };
        event_loop.set_control_flow(control_flow);
    }

    fn exit_if_finished(&self, event_loop: &ActiveEventLoop) -> bool {
        if self.started && self.platform.host().windows().is_empty() {
            event_loop.exit();
            true
        } else {
            false
        }
    }
}

impl<M: State, E: Send + 'static, B: Backend> Runner<M, E, B> {
    pub fn with_platform(platform: Platform<M, E, B>) -> Self {
        Self {
            platform,
            events: Events::new(),
            started: false,
            error: None,
        }
    }

    pub fn platform(&self) -> &Platform<M, E, B> {
        &self.platform
    }

    pub fn platform_mut(&mut self) -> &mut Platform<M, E, B> {
        &mut self.platform
    }

    pub fn events(&self) -> &Events {
        &self.events
    }

    pub fn events_mut(&mut self) -> &mut Events {
        &mut self.events
    }

    pub fn started(&self) -> bool {
        self.started
    }

    pub fn error(&self) -> Option<&Error<B::Error>> {
        self.error.as_ref()
    }

    pub fn take_error(&mut self) -> Option<Error<B::Error>> {
        self.error.take()
    }

    pub fn into_platform(self) -> Platform<M, E, B> {
        self.platform
    }

    pub fn start(&mut self) -> Result<(), Error<B::Error>>
    where
        for<'a> B::Context<'a>: Default,
    {
        if self.started {
            return Ok(());
        }

        self.platform.start()?;
        self.started = true;
        Ok(())
    }

    pub fn handle_event(&mut self, event: host::Event) -> Result<(), Error<B::Error>>
    where
        for<'a> B::Context<'a>: Default,
    {
        self.platform.handle_event(event)
    }

    pub fn emit(&mut self, event: E) -> Result<(), Error<B::Error>>
    where
        for<'a> B::Context<'a>: Default,
    {
        self.platform
            .host_mut()
            .shell_mut()
            .runtime_mut()
            .emit(event);
        self.platform.drain()
    }

    pub fn poll(&mut self) -> Result<(), Error<B::Error>>
    where
        for<'a> B::Context<'a>: Default,
    {
        self.platform.poll()
    }
}

impl Window {
    fn from_shell(window: &shell::Window) -> Self {
        Self {
            id: window.id(),
            title: window.title().to_owned(),
            size: window.size(),
            canvas_color: window.canvas_color(),
        }
    }

    pub fn id(&self) -> window::Id {
        self.id
    }

    pub fn title(&self) -> &str {
        &self.title
    }

    pub fn size(&self) -> geometry::Size {
        self.size
    }

    pub fn canvas_color(&self) -> scene::Color {
        self.canvas_color
    }
}

impl Native {
    pub fn new() -> Self {
        Self {
            context: None,
            renderer: None,
            windows: HashMap::new(),
            raw_windows: HashMap::new(),
            requests: Vec::new(),
            poll_requested: false,
        }
    }

    pub fn initialize(&mut self) -> Result<(), NativeError> {
        self.ensure_context()
    }

    pub fn ready(&self) -> bool {
        self.context.is_some()
    }

    pub fn contains(&self, window: window::Id) -> bool {
        self.windows.contains_key(&window)
    }

    pub fn window_for_raw(&self, raw: winit::window::WindowId) -> Option<window::Id> {
        self.raw_windows.get(&raw).copied()
    }

    pub fn scale_factor(&self, window: window::Id) -> Option<f64> {
        self.windows.get(&window).map(native::Window::scale_factor)
    }

    pub fn is_empty(&self) -> bool {
        self.windows.is_empty()
    }

    pub fn requests(&self) -> &[session::Request] {
        &self.requests
    }

    pub fn take_requests(&mut self) -> Vec<session::Request> {
        std::mem::take(&mut self.requests)
    }

    pub fn clear_requests(&mut self) {
        self.requests.clear();
    }

    pub fn poll_requested(&self) -> bool {
        self.poll_requested
    }

    pub fn take_poll_requested(&mut self) -> bool {
        let requested = self.poll_requested;
        self.poll_requested = false;
        requested
    }

    pub fn request_redraw(&self, window: window::Id) -> Result<(), NativeError> {
        let Some(window) = self.windows.get(&window) else {
            return Err(NativeError::MissingWindow { window });
        };

        window.request_redraw();
        Ok(())
    }

    #[cfg(test)]
    pub fn track_window_for_test(&mut self, raw: winit::window::WindowId, window: window::Id) {
        self.raw_windows.insert(raw, window);
    }

    #[cfg(test)]
    pub fn track_request_for_test(&mut self, request: session::Request) {
        self.requests.push(request);
    }

    fn ensure_context(&mut self) -> Result<(), NativeError> {
        if self.context.is_none() {
            self.context = Some(pollster::block_on(render::Context::new(
                render_context_options(),
            ))?);
        }

        Ok(())
    }

    fn ensure_renderer(&mut self, format: wgpu::TextureFormat) {
        if self.renderer.is_some() {
            return;
        }

        let context = self
            .context
            .as_ref()
            .expect("render context should exist before creating renderer");
        self.renderer = Some(render::Renderer::new(context, format));
    }

    fn create_native_window(
        &mut self,
        context: &NativeContext<'_>,
        window: &Window,
    ) -> Result<native::Window, NativeError> {
        self.ensure_context()?;

        let native_options = native::window::Options {
            title: window.title().to_owned(),
            inner_size: native::window::InitialSize::Logical(logical_area(window.size())),
        };
        let handle = native::Window::open(native_options, context.event_loop())?;
        let render_context = self
            .context
            .as_ref()
            .expect("render context should exist before creating window canvas");
        let inner_size = handle.inner_size();
        let canvas = render::Canvas::new(
            render::canvas::Options {
                area: area::physical(inner_size.width, inner_size.height).clamp_min(1),
                scale_factor: handle.scale_factor() as f32,
                color: render::color_to_wgpu(paint_color(window.canvas_color())),
            },
            render_context,
            handle.clone(),
        )?;

        Ok(native::Window::new(handle, canvas))
    }

    fn clear_window(&mut self, native_window: &mut native::Window) -> Result<(), NativeError> {
        let format = native_window.canvas().surface().config().format;
        self.ensure_renderer(format);

        let context = self
            .context
            .as_ref()
            .expect("render context should exist before clearing");
        let renderer = self
            .renderer
            .as_mut()
            .expect("renderer should exist before clearing");
        renderer.clear(context, native_window.canvas_mut())?;

        Ok(())
    }

    fn sync_window_surface(
        &mut self,
        window: window::Id,
    ) -> Result<wgpu::TextureFormat, NativeError> {
        self.ensure_context()?;
        let native_window = self
            .windows
            .get_mut(&window)
            .ok_or(NativeError::MissingWindow { window })?;
        let area = native_window.inner_area().clamp_min(1);
        let scale_factor = native_window.scale_factor() as f32;
        let needs_resize = native_window.canvas().physical_area() != area
            || (native_window.canvas().scale_factor() - scale_factor).abs() > f32::EPSILON;

        if needs_resize {
            let context = self
                .context
                .as_ref()
                .expect("render context should exist before resizing");
            native_window.resize(context, area, scale_factor);
        }

        Ok(native_window.canvas().surface().config().format)
    }
}

impl Default for Native {
    fn default() -> Self {
        Self::new()
    }
}

impl<'a> NativeContext<'a> {
    pub fn new(event_loop: &'a ActiveEventLoop) -> Self {
        NativeContext { event_loop }
    }

    pub fn event_loop(&self) -> &ActiveEventLoop {
        self.event_loop
    }
}

impl Backend for Native {
    type Error = NativeError;
    type Context<'a> = NativeContext<'a>;

    fn open_window(
        &mut self,
        context: &mut Self::Context<'_>,
        window: &Window,
    ) -> Result<(), Self::Error> {
        let mut native_window = self.create_native_window(context, window)?;
        self.clear_window(&mut native_window)?;
        native_window.set_ime_allowed(true);
        native_window.set_visibility(true);

        self.raw_windows.insert(native_window.raw_id(), window.id());
        self.windows.insert(window.id(), native_window);

        Ok(())
    }

    fn close_window(
        &mut self,
        _context: &mut Self::Context<'_>,
        window: window::Id,
    ) -> Result<(), Self::Error> {
        let native_window = self
            .windows
            .remove(&window)
            .ok_or(NativeError::MissingWindow { window })?;
        self.raw_windows.remove(&native_window.raw_id());
        Ok(())
    }

    fn present(
        &mut self,
        _context: &mut Self::Context<'_>,
        presentation: &shell::Presentation,
    ) -> Result<(), Self::Error> {
        let window = presentation.window();
        let format = self.sync_window_surface(window)?;
        self.ensure_renderer(format);

        let context = self
            .context
            .as_ref()
            .expect("render context should exist before presenting");
        let renderer = self
            .renderer
            .as_mut()
            .expect("renderer should exist before presenting");
        let native_window = self
            .windows
            .get_mut(&window)
            .ok_or(NativeError::MissingWindow { window })?;

        renderer.draw(
            context,
            native_window.canvas_mut(),
            presentation.scene(),
            &[],
        )?;

        Ok(())
    }

    fn request(
        &mut self,
        _context: &mut Self::Context<'_>,
        request: session::Request,
    ) -> Result<(), Self::Error> {
        if !self.requests.contains(&request) {
            self.requests.push(request);
        }

        Ok(())
    }

    fn schedule_poll(&mut self, _context: &mut Self::Context<'_>) -> Result<(), Self::Error> {
        self.poll_requested = true;
        Ok(())
    }
}

impl Events {
    pub fn new() -> Self {
        Self {
            modifiers: input::Modifiers::default(),
            default_scale_factor: 1.0,
            windows: HashMap::new(),
        }
    }

    pub fn with_scale_factor(mut self, scale_factor: f64) -> Self {
        self.set_scale_factor(scale_factor);
        self
    }

    pub fn set_scale_factor(&mut self, scale_factor: f64) {
        self.default_scale_factor = normalized_scale_factor(scale_factor);
    }

    pub fn set_window_scale_factor(&mut self, window: window::Id, scale_factor: f64) {
        self.window_state(window).scale_factor = normalized_scale_factor(scale_factor);
    }

    pub fn modifiers(&self) -> input::Modifiers {
        self.modifiers
    }

    pub fn pointer(&self, window: window::Id) -> geometry::Point {
        self.windows
            .get(&window)
            .map(|state| state.pointer)
            .unwrap_or_else(origin)
    }

    pub fn scale_factor(&self, window: window::Id) -> f64 {
        self.windows
            .get(&window)
            .map(|state| state.scale_factor)
            .unwrap_or(self.default_scale_factor)
    }

    pub fn retain_windows(&mut self, mut retain: impl FnMut(window::Id) -> bool) {
        self.windows.retain(|window, _| retain(*window));
    }

    pub fn window_event(
        &mut self,
        window: window::Id,
        event: &WinitWindowEvent,
    ) -> Option<host::Event> {
        let scale_factor = self.scale_factor(window);
        let event = match event {
            WinitWindowEvent::CloseRequested => host::WindowEvent::CloseRequested,
            WinitWindowEvent::Resized(size) => host::WindowEvent::Resized {
                size: size_from_physical(*size, scale_factor),
            },
            WinitWindowEvent::ScaleFactorChanged { scale_factor, .. } => {
                self.window_state(window).scale_factor = normalized_scale_factor(*scale_factor);
                return None;
            }
            WinitWindowEvent::CursorMoved { position, .. } => {
                let point = point_from_physical(*position, scale_factor);
                self.window_state(window).pointer = point;
                host::WindowEvent::PointerMoved { point }
            }
            WinitWindowEvent::CursorLeft { .. } => host::WindowEvent::PointerLeft,
            WinitWindowEvent::MouseInput {
                state,
                button: MouseButton::Left,
                ..
            } => match state {
                ElementState::Pressed => host::WindowEvent::PointerDown {
                    point: self.pointer(window),
                },
                ElementState::Released => host::WindowEvent::PointerUp {
                    point: self.pointer(window),
                },
            },
            WinitWindowEvent::MouseInput { .. } => return None,
            WinitWindowEvent::MouseWheel { delta, .. } => host::WindowEvent::Scrolled {
                point: self.pointer(window),
                delta: scroll_delta(*delta, scale_factor),
            },
            WinitWindowEvent::ModifiersChanged(next) => {
                self.modifiers = modifiers(next.state());
                return None;
            }
            WinitWindowEvent::KeyboardInput {
                event,
                is_synthetic: false,
                ..
            } if event.state == ElementState::Pressed => host::WindowEvent::KeyDown {
                key: key(&event.logical_key),
                modifiers: self.modifiers,
                text: key_text(event.text.as_deref()),
            },
            WinitWindowEvent::Ime(Ime::Commit(text)) => {
                host::WindowEvent::TextCommitted { text: text.clone() }
            }
            WinitWindowEvent::Ime(Ime::Preedit(text, selection)) => {
                host::WindowEvent::TextPreedit {
                    preedit: text::Preedit::new(text.clone(), *selection),
                }
            }
            WinitWindowEvent::Ime(Ime::Disabled) => host::WindowEvent::TextPreedit {
                preedit: text::Preedit::new("", None),
            },
            WinitWindowEvent::RedrawRequested => host::WindowEvent::RedrawRequested,
            _ => return None,
        };

        Some(host::Event::window(window, event))
    }

    fn window_state(&mut self, window: window::Id) -> &mut WindowEvents {
        self.windows.entry(window).or_insert(WindowEvents {
            scale_factor: self.default_scale_factor,
            pointer: origin(),
        })
    }
}

impl Default for Events {
    fn default() -> Self {
        Self::new()
    }
}

pub fn key(key: &WinitKey) -> input::Key {
    match key.as_ref() {
        WinitKey::Named(NamedKey::Tab) => input::Key::Tab,
        WinitKey::Named(NamedKey::Enter) => input::Key::Enter,
        WinitKey::Named(NamedKey::Space) => input::Key::Space,
        WinitKey::Named(NamedKey::Escape) => input::Key::Escape,
        WinitKey::Named(NamedKey::Backspace) => input::Key::Backspace,
        WinitKey::Named(NamedKey::Delete) => input::Key::Delete,
        WinitKey::Named(NamedKey::ArrowLeft) => input::Key::ArrowLeft,
        WinitKey::Named(NamedKey::ArrowRight) => input::Key::ArrowRight,
        WinitKey::Named(NamedKey::ArrowUp) => input::Key::ArrowUp,
        WinitKey::Named(NamedKey::ArrowDown) => input::Key::ArrowDown,
        WinitKey::Named(NamedKey::Home) => input::Key::Home,
        WinitKey::Named(NamedKey::End) => input::Key::End,
        WinitKey::Named(NamedKey::PageUp) => input::Key::PageUp,
        WinitKey::Named(NamedKey::PageDown) => input::Key::PageDown,
        WinitKey::Named(NamedKey::F4) => input::Key::F4,
        WinitKey::Character(value) => {
            let mut chars = value.chars();
            match (chars.next(), chars.next()) {
                (Some(character), None) => input::Key::Character(character),
                _ => input::Key::Other,
            }
        }
        _ => input::Key::Other,
    }
}

pub fn key_text(text: Option<&str>) -> Option<String> {
    text.filter(|text| text.chars().all(|character| !character.is_control()))
        .map(str::to_owned)
}

pub fn modifiers(modifiers: ModifiersState) -> input::Modifiers {
    input::Modifiers::new(
        modifiers.shift_key(),
        modifiers.control_key(),
        modifiers.alt_key(),
        modifiers.super_key(),
    )
}

pub fn size_from_physical(size: PhysicalSize<u32>, scale_factor: f64) -> geometry::Size {
    let scale_factor = normalized_scale_factor(scale_factor);
    geometry::Size::new(
        logical_i32(size.width as f64 / scale_factor),
        logical_i32(size.height as f64 / scale_factor),
    )
}

pub fn point_from_physical(position: PhysicalPosition<f64>, scale_factor: f64) -> geometry::Point {
    let scale_factor = normalized_scale_factor(scale_factor);
    geometry::Point::new(
        logical_i32(position.x / scale_factor),
        logical_i32(position.y / scale_factor),
    )
}

pub fn scroll_delta(delta: MouseScrollDelta, scale_factor: f64) -> interaction::ScrollDelta {
    const LINE_SCROLL_LOGICAL_PIXELS: f64 = 28.0;

    match delta {
        MouseScrollDelta::LineDelta(x, y) => interaction::ScrollDelta::new(
            logical_i32(x as f64 * LINE_SCROLL_LOGICAL_PIXELS),
            logical_i32(-(y as f64) * LINE_SCROLL_LOGICAL_PIXELS),
        ),
        MouseScrollDelta::PixelDelta(position) => {
            let scale_factor = normalized_scale_factor(scale_factor);
            interaction::ScrollDelta::new(
                logical_i32(position.x / scale_factor),
                logical_i32(-position.y / scale_factor),
            )
        }
    }
}

fn render_context_options() -> render::context::Options {
    render::context::Options {
        device_label: "wgpu_l3 scratch device",
        backends: wgpu::Backends::all(),
        power_preference: wgpu::PowerPreference::HighPerformance,
        force_fallback_adapter: false,
        required_features: wgpu::Features::empty(),
        required_limits: wgpu::Limits::default(),
    }
}

fn logical_area(size: geometry::Size) -> area::Logical {
    area::logical(size.width().max(1) as f32, size.height().max(1) as f32)
}

fn paint_color(color: scene::Color) -> paint::Color {
    let (r, g, b, a) = color.channels();
    paint::Color::rgba(
        r as f32 / 255.0,
        g as f32 / 255.0,
        b as f32 / 255.0,
        a as f32 / 255.0,
    )
}

fn normalized_scale_factor(scale_factor: f64) -> f64 {
    if scale_factor.is_finite() && scale_factor > 0.0 {
        scale_factor
    } else {
        1.0
    }
}

fn logical_i32(value: f64) -> i32 {
    if !value.is_finite() {
        return 0;
    }

    value.round().clamp(i32::MIN as f64, i32::MAX as f64) as i32
}

fn origin() -> geometry::Point {
    geometry::Point::new(0, 0)
}

impl<E: fmt::Display> fmt::Display for Error<E> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Framework(error) => write!(formatter, "framework error: {error}"),
            Self::Backend(error) => write!(formatter, "backend error: {error}"),
        }
    }
}

impl<E> std::error::Error for Error<E>
where
    E: std::error::Error + fmt::Debug + fmt::Display + 'static,
{
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Framework(error) => Some(error),
            Self::Backend(error) => Some(error),
        }
    }
}

impl<E: fmt::Display> fmt::Display for RunError<E> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::EventLoop(error) => write!(formatter, "event loop error: {error}"),
            Self::Platform(error) => write!(formatter, "platform error: {error}"),
        }
    }
}

impl<E> std::error::Error for RunError<E>
where
    E: std::error::Error + fmt::Debug + fmt::Display + 'static,
{
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::EventLoop(error) => Some(error),
            Self::Platform(error) => Some(error),
        }
    }
}

impl<E> From<winit::error::EventLoopError> for RunError<E> {
    fn from(error: winit::error::EventLoopError) -> Self {
        Self::EventLoop(error)
    }
}

impl<E> From<Error<E>> for RunError<E> {
    fn from(error: Error<E>) -> Self {
        Self::Platform(error)
    }
}

pub fn run<M: State, E: Send + 'static>(
    shell: shell::Shell<M, E>,
) -> Result<(), RunError<NativeError>> {
    Runner::new(shell).run()
}

pub(super) fn file_dialog_selected(
    request: session::Request,
    path: Option<PathBuf>,
) -> host::Event {
    match request.kind() {
        session::RequestKind::FileDialog(_) => host::Event::FilePathSelected {
            window: request.window(),
            path,
        },
    }
}

fn native_file_dialog(kind: session::RequestKind) -> Option<PathBuf> {
    match kind {
        session::RequestKind::FileDialog(session::FileDialog::Open) => {
            rfd::FileDialog::new().pick_file()
        }
        session::RequestKind::FileDialog(session::FileDialog::SaveAs) => {
            rfd::FileDialog::new().save_file()
        }
    }
}

impl<M: State, E: Send + 'static> ApplicationHandler<E> for Runner<M, E, Native> {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if self.started {
            self.sync_control_flow(event_loop);
            return;
        }

        let mut context = NativeContext::new(event_loop);
        if let Err(error) = self.platform.start_with(&mut context) {
            self.fail(event_loop, error);
            return;
        }

        self.started = true;
        self.finish_native_pass(event_loop);
    }

    fn user_event(&mut self, event_loop: &ActiveEventLoop, event: E) {
        self.platform
            .host_mut()
            .shell_mut()
            .runtime_mut()
            .emit(event);

        let mut context = NativeContext::new(event_loop);
        if let Err(error) = self.platform.drain_with(&mut context) {
            self.fail(event_loop, error);
            return;
        }

        self.finish_native_pass(event_loop);
    }

    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        raw_window: winit::window::WindowId,
        event: WinitWindowEvent,
    ) {
        let Some(event) = self.translate_window_event(raw_window, &event) else {
            return;
        };

        let mut context = NativeContext::new(event_loop);
        if let Err(error) = self.platform.handle_event_with(&mut context, event) {
            self.fail(event_loop, error);
            return;
        }

        self.finish_native_pass(event_loop);
    }

    fn about_to_wait(&mut self, event_loop: &ActiveEventLoop) {
        if self.platform.backend_mut().take_poll_requested() {
            let mut context = NativeContext::new(event_loop);
            if let Err(error) = self.platform.poll_with(&mut context) {
                self.fail(event_loop, error);
                return;
            }
        }

        self.finish_native_pass(event_loop);
    }
}
