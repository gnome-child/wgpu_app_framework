use std::collections::HashMap;

use crate::text;
use winit::{
    dpi::{PhysicalPosition, PhysicalSize},
    event::{ElementState, Ime, MouseButton, MouseScrollDelta, WindowEvent as WinitWindowEvent},
    keyboard::{Key as WinitKey, ModifiersState, NamedKey},
};

use super::super::{geometry, host, input, interaction, pointer, window};

pub struct Events {
    modifiers: input::Modifiers,
    default_scale_factor: f64,
    windows: HashMap<window::Id, WindowEvents>,
}

struct WindowEvents {
    scale_factor: f64,
    pointer: geometry::Point,
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
            WinitWindowEvent::MouseInput { state, button, .. } => match state {
                ElementState::Pressed => host::WindowEvent::PointerDown {
                    point: self.pointer(window),
                    button: pointer_button(*button),
                    modifiers: self.modifiers,
                },
                ElementState::Released => host::WindowEvent::PointerUp {
                    point: self.pointer(window),
                    button: pointer_button(*button),
                },
            },
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
            WinitWindowEvent::Ime(ime) => ime_window_event(ime)?,
            WinitWindowEvent::RedrawRequested => host::WindowEvent::RedrawRequested,
            _ => return None,
        };

        Some(host::Event::window(window, event))
    }

    pub fn popup_window_event(
        &mut self,
        parent: window::Id,
        bounds: geometry::Rect,
        panel_offset_physical: (i32, i32),
        popup_scale_factor: f64,
        event: &WinitWindowEvent,
    ) -> Option<host::Event> {
        let event = match event {
            WinitWindowEvent::CursorMoved { position, .. } => {
                let point = popup_point_from_physical(
                    *position,
                    popup_scale_factor,
                    bounds,
                    panel_offset_physical,
                );
                self.window_state(parent).pointer = point;
                host::WindowEvent::PointerMoved { point }
            }
            WinitWindowEvent::CursorLeft { .. } => host::WindowEvent::PointerLeft,
            WinitWindowEvent::MouseInput { state, button, .. } => match state {
                ElementState::Pressed => host::WindowEvent::PointerDown {
                    point: self.pointer(parent),
                    button: pointer_button(*button),
                    modifiers: self.modifiers,
                },
                ElementState::Released => host::WindowEvent::PointerUp {
                    point: self.pointer(parent),
                    button: pointer_button(*button),
                },
            },
            WinitWindowEvent::MouseWheel { delta, .. } => host::WindowEvent::Scrolled {
                point: self.pointer(parent),
                delta: scroll_delta(*delta, popup_scale_factor),
            },
            WinitWindowEvent::Ime(ime) => ime_window_event(ime)?,
            WinitWindowEvent::RedrawRequested => host::WindowEvent::RedrawRequested,
            _ => return None,
        };

        Some(host::Event::window(parent, event))
    }

    fn window_state(&mut self, window: window::Id) -> &mut WindowEvents {
        self.windows.entry(window).or_insert(WindowEvents {
            scale_factor: self.default_scale_factor,
            pointer: origin(),
        })
    }
}

fn ime_window_event(ime: &Ime) -> Option<host::WindowEvent> {
    match ime {
        Ime::Commit(text) => Some(host::WindowEvent::TextCommitted { text: text.clone() }),
        Ime::Preedit(text, selection) => Some(host::WindowEvent::TextPreedit {
            preedit: text::edit::Preedit::new(text.clone(), *selection),
        }),
        Ime::Disabled => Some(host::WindowEvent::TextPreedit {
            preedit: text::edit::Preedit::new("", None),
        }),
        Ime::Enabled => None,
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
        WinitKey::Named(NamedKey::F2) => input::Key::F2,
        WinitKey::Named(NamedKey::F4) => input::Key::F4,
        WinitKey::Named(NamedKey::F10) => input::Key::F10,
        WinitKey::Named(NamedKey::ContextMenu) => input::Key::ContextMenu,
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

pub fn popup_point_from_physical(
    position: PhysicalPosition<f64>,
    scale_factor: f64,
    bounds: geometry::Rect,
    panel_offset_physical: (i32, i32),
) -> geometry::Point {
    let local = point_from_physical(
        PhysicalPosition::new(
            position.x - f64::from(panel_offset_physical.0),
            position.y - f64::from(panel_offset_physical.1),
        ),
        scale_factor,
    );
    geometry::Point::new(
        bounds.x().saturating_add(local.x()),
        bounds.y().saturating_add(local.y()),
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

pub fn pointer_button(button: MouseButton) -> pointer::Button {
    match button {
        MouseButton::Left => pointer::Button::Primary,
        MouseButton::Right => pointer::Button::Secondary,
        MouseButton::Middle => pointer::Button::Middle,
        MouseButton::Back => pointer::Button::Back,
        MouseButton::Forward => pointer::Button::Forward,
        MouseButton::Other(value) => pointer::Button::Other(value),
    }
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
