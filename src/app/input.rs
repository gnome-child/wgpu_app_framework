use std::time::Instant;

use winit::{
    event::{Ime, MouseButton},
    keyboard::{Key as WinitKey, ModifiersState, NamedKey},
};

use crate::app::state::{PressSource, WindowState, action_request};
use crate::geometry::point;
use crate::{action, pointer, text, ui, window};

use super::text_input;

#[derive(Debug, Default)]
pub struct Outcome {
    pub events: Vec<ui::Event>,
    pub request: Option<action::Request>,
    pub intent: Option<IntentRequest>,
    pub redraw: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub struct IntentRequest {
    pub origin: ui::Path,
    pub intent: ui::Intent,
    pub source: action::Source,
}

impl IntentRequest {
    fn new(origin: ui::Path, intent: ui::Intent, source: action::Source) -> Self {
        Self {
            origin,
            intent,
            source,
        }
    }
}

#[cfg(test)]
pub fn pointer_moved(state: &mut WindowState, position: point::Logical) -> Outcome {
    let mut text_engine = text::Engine::new();

    pointer_moved_with_text_engine(state, position, &mut text_engine)
}

pub fn pointer_moved_with_text_engine(
    state: &mut WindowState,
    position: point::Logical,
    text_engine: &mut text::Engine,
) -> Outcome {
    state
        .pointer
        .handle_event(pointer::Event::Moved { position });
    let delta = state.pointer.delta();

    if let Some((target, offset)) = state.pointer_capture_offset(position, text_engine) {
        let mut events = vec![ui::Event::PointerMoved {
            position,
            delta,
            target: Some(target.clone()),
        }];
        if state
            .text_surface(&target)
            .is_some_and(text::Surface::is_area)
        {
            state.scroll_text_area_to(&target, offset, text_engine);
        } else if state
            .scroll_metrics_for(&target, text_engine)
            .is_some_and(|metrics| metrics.offset() != offset)
        {
            events.push(ui::Event::ScrollRequested { target, offset });
        }

        return Outcome {
            events,
            request: None,
            intent: None,
            redraw: true,
        };
    }

    let target = state.hit_test(position);
    let hover_events = state.set_hovered(target.clone());
    let intent = target
        .as_ref()
        .and_then(|target| hover_menu_intent(state, target));

    let redraw = !hover_events.is_empty() || intent.is_some();
    let mut events = hover_events;
    events.push(ui::Event::PointerMoved {
        position,
        delta,
        target,
    });

    let _text_drag_redraw = state.update_text_drag(position, text_engine);
    if state.drag_drop.active_text().is_some() {
        return Outcome {
            events,
            request: None,
            intent,
            redraw: true,
        };
    }

    if let Some((target, edit)) = state.text_field_drag_edit_at(position, text_engine) {
        events.push(ui::Event::TextEditRequested { target, edit });
        return Outcome {
            events,
            request: None,
            intent,
            redraw: true,
        };
    }

    Outcome {
        events,
        request: None,
        intent,
        redraw,
    }
}

pub fn pointer_pressed(
    state: &mut WindowState,
    window: window::Id,
    position: point::Logical,
    button: pointer::Button,
    text_engine: &mut text::Engine,
) -> Outcome {
    state.pointer.handle_event(pointer::Event::Button {
        button,
        pressed: true,
    });

    if button == pointer::Button::Primary
        && let Some(hit) = state.widget_hit(position, text_engine)
    {
        state.dismiss_menu_for_target(Some(hit.target()));
        let mut events = vec![ui::Event::PointerDown {
            position,
            delta: state.pointer.delta(),
            target: Some(hit.target().clone()),
            button,
        }];

        if state.start_pointer_capture(&hit, button, position, text_engine) {
            return Outcome {
                events,
                request: None,
                intent: None,
                redraw: true,
            };
        }

        if let Some(part) = hit.part().scroll()
            && let Some(metrics) = state.scroll_metrics_for(hit.target(), text_engine)
            && let Some(offset) = metrics.page_offset(part, position)
            && offset != metrics.offset()
        {
            state.pressed = Some(hit.target().clone());
            state.pressed_source = Some(PressSource::Pointer);
            if state
                .text_surface(hit.target())
                .is_some_and(text::Surface::is_area)
            {
                state.scroll_text_area_to(hit.target(), offset, text_engine);
            } else {
                events.push(ui::Event::ScrollRequested {
                    target: hit.target().clone(),
                    offset,
                });
            }
        }

        return Outcome {
            events,
            request: None,
            intent: None,
            redraw: true,
        };
    }

    let target = state.hit_test(position);
    if button == pointer::Button::Secondary
        && let Some(target) = target.as_ref()
        && state.is_selectable_text_field(target)
    {
        let event = state.pointer_down(
            position,
            state.pointer.delta(),
            Some(target.clone()),
            button,
        );
        state.open_text_context_menu(window, target.clone(), position, action::Source::Pointer);

        return Outcome {
            events: vec![event],
            request: None,
            intent: None,
            redraw: true,
        };
    }

    state.dismiss_menu_for_target(target.as_ref());
    let event = state.pointer_down(position, state.pointer.delta(), target, button);
    let mut events = vec![event];

    if button == pointer::Button::Primary
        && let ui::Event::PointerDown {
            target: Some(target),
            ..
        } = &events[0]
        && let Some(edit) = state.text_field_edit_at(target, position, text_engine)
    {
        events.push(ui::Event::TextEditRequested {
            target: target.clone(),
            edit,
        });
    }

    Outcome {
        events,
        request: None,
        intent: None,
        redraw: true,
    }
}

pub fn pointer_released<T>(
    registry: &action::Registry<T>,
    state: &mut WindowState,
    window: window::Id,
    position: point::Logical,
    button: pointer::Button,
) -> Outcome {
    state.pointer.handle_event(pointer::Event::Button {
        button,
        pressed: false,
    });
    let text_drop_event = if button == pointer::Button::Primary {
        state.finish_text_drop()
    } else {
        None
    };
    let cleared_text_drag = if text_drop_event.is_none() {
        state.clear_text_drag_drop()
    } else {
        false
    };
    let text_click_event = if button == pointer::Button::Primary && text_drop_event.is_none() {
        state
            .finish_text_pointer_gesture()
            .map(|(target, edit)| ui::Event::TextEditRequested { target, edit })
    } else {
        state.cancel_text_pointer_gesture();
        None
    };

    if let Some(capture) = state.pointer_capture.clone() {
        state.clear_pointer_capture();
        state.pressed = None;
        state.pressed_source = None;

        return Outcome {
            events: vec![ui::Event::PointerUp {
                position,
                delta: state.pointer.delta(),
                target: Some(capture.target().clone()),
                button,
            }],
            request: None,
            intent: None,
            redraw: true,
        };
    }

    let target = state.hit_test(position);
    let (event, invoke_target) = state.pointer_up(position, state.pointer.delta(), target, button);
    if let Some(text_drop_event) = text_drop_event {
        return Outcome {
            events: vec![event, text_drop_event],
            request: None,
            intent: None,
            redraw: true,
        };
    }

    let intent = invoke_target.as_ref().and_then(|target| {
        state
            .intent(target)
            .map(|intent| IntentRequest::new(target.clone(), intent, action::Source::Pointer))
    });
    let request = invoke_target.and_then(|target| {
        activation_request(registry, state, window, target, action::Source::Pointer)
    });
    let menu_restore_visibility =
        request
            .as_ref()
            .and_then(|request| match request.target().scope() {
                action::Scope::Path(target) => {
                    Some(state.focus_visibility_for_activation(target, action::Source::Pointer))
                }
                action::Scope::Window => Some(ui::focus::Visibility::Hidden),
            });
    let closed_menu = request
        .as_ref()
        .and_then(action::Request::origin)
        .is_some_and(|origin| state.is_menu_path(origin))
        && state.close_menu_with_focus_visibility(menu_restore_visibility);
    let restored_focus = request.as_ref().is_some_and(|request| {
        restore_action_target_focus(state, request, action::Source::Pointer)
    });
    let mut events = vec![event];
    if let Some(text_click_event) = text_click_event {
        events.push(text_click_event);
    }

    Outcome {
        events,
        request,
        intent,
        redraw: true || closed_menu || restored_focus || cleared_text_drag,
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

pub fn pointer_left(state: &mut WindowState) -> Outcome {
    state.pointer.handle_event(pointer::Event::Left);
    let cleared_text_gesture = state.cancel_text_pointer_gesture();
    let cleared_text_drag = state.clear_text_drag_drop();
    let events = state.set_hovered(None);
    let cleared_capture = state.clear_pointer_capture();
    let cleared_pressed = if state.pressed_source == Some(PressSource::Pointer) {
        state.pressed = None;
        state.pressed_source = None;
        true
    } else {
        false
    };

    Outcome {
        redraw: !events.is_empty()
            || cleared_pressed
            || cleared_capture
            || cleared_text_drag
            || cleared_text_gesture,
        events,
        request: None,
        intent: None,
    }
}

pub fn scroll_wheel(
    state: &mut WindowState,
    position: point::Logical,
    delta: point::Logical,
    text_engine: &mut text::Engine,
) -> Outcome {
    let target = state.scroll_target(position, text_engine);
    let area_scrolled = target.as_ref().is_some_and(|target| {
        state
            .text_surface(target)
            .is_some_and(text::Surface::is_area)
    }) && state.scroll_text_area_at(position, delta, text_engine);
    let mut events = vec![ui::Event::ScrollWheel {
        position,
        delta,
        target: target.clone(),
    }];

    if !area_scrolled
        && let Some(target) = target
        && let Some(metrics) = state.scroll_metrics_for(&target, text_engine)
    {
        let offset = metrics.wheel_offset(delta);
        if offset != metrics.offset() {
            events.push(ui::Event::ScrollRequested { target, offset });
        }
    }

    Outcome {
        events,
        request: None,
        intent: None,
        redraw: area_scrolled,
    }
}

pub fn modifiers(modifiers: ModifiersState) -> ui::Modifiers {
    ui::Modifiers::new(
        modifiers.shift_key(),
        modifiers.control_key(),
        modifiers.alt_key(),
        modifiers.super_key(),
    )
}

pub fn key(key: &WinitKey) -> ui::Key {
    match key {
        WinitKey::Named(NamedKey::Tab) => ui::Key::Tab,
        WinitKey::Named(NamedKey::Enter) => ui::Key::Enter,
        WinitKey::Named(NamedKey::Space) => ui::Key::Space,
        WinitKey::Named(NamedKey::Escape) => ui::Key::Escape,
        WinitKey::Named(NamedKey::Backspace) => ui::Key::Backspace,
        WinitKey::Named(NamedKey::Delete) => ui::Key::Delete,
        WinitKey::Named(NamedKey::ArrowLeft) => ui::Key::ArrowLeft,
        WinitKey::Named(NamedKey::ArrowRight) => ui::Key::ArrowRight,
        WinitKey::Named(NamedKey::ArrowUp) => ui::Key::ArrowUp,
        WinitKey::Named(NamedKey::ArrowDown) => ui::Key::ArrowDown,
        WinitKey::Named(NamedKey::Home) => ui::Key::Home,
        WinitKey::Named(NamedKey::End) => ui::Key::End,
        WinitKey::Named(NamedKey::PageUp) => ui::Key::PageUp,
        WinitKey::Named(NamedKey::PageDown) => ui::Key::PageDown,
        WinitKey::Named(NamedKey::F10) => ui::Key::F10,
        WinitKey::Named(NamedKey::ContextMenu) => ui::Key::ContextMenu,
        WinitKey::Character(value) => {
            let mut chars = value.chars();
            let Some(character) = chars.next() else {
                return ui::Key::Other;
            };

            if chars.next().is_none() {
                ui::Key::Character(character)
            } else {
                ui::Key::Other
            }
        }
        _ => ui::Key::Other,
    }
}

fn text_edit_for_key(
    key: ui::Key,
    modifiers: ui::Modifiers,
    inserted_text: Option<&str>,
    multiline: bool,
) -> Option<text::Edit> {
    if modifiers.super_key() {
        return None;
    }

    let jump = jump_modifier_pressed(modifiers);
    let motion = match key {
        ui::Key::ArrowLeft if jump => Some(text::TextMotion::WordPrevious),
        ui::Key::ArrowRight if jump => Some(text::TextMotion::WordNext),
        ui::Key::ArrowLeft => Some(text::TextMotion::VisualLeft),
        ui::Key::ArrowRight => Some(text::TextMotion::VisualRight),
        ui::Key::ArrowUp if multiline => Some(text::TextMotion::VisualUp),
        ui::Key::ArrowDown if multiline => Some(text::TextMotion::VisualDown),
        ui::Key::PageUp if multiline => Some(text::TextMotion::PageUp),
        ui::Key::PageDown if multiline => Some(text::TextMotion::PageDown),
        ui::Key::Home if multiline && jump => Some(text::TextMotion::DocumentStart),
        ui::Key::End if multiline && jump => Some(text::TextMotion::DocumentEnd),
        ui::Key::Home => Some(text::TextMotion::LineStart),
        ui::Key::End => Some(text::TextMotion::LineEnd),
        _ => None,
    };

    if let Some(motion) = motion {
        return Some(if modifiers.shift() {
            text::Edit::extend_position(motion)
        } else {
            text::Edit::move_position(motion)
        });
    }

    match key {
        ui::Key::Backspace if jump => Some(text::Edit::delete_word_backward()),
        ui::Key::Delete if jump => Some(text::Edit::delete_word_forward()),
        _ if modifiers.control() || modifiers.alt() => None,
        ui::Key::Backspace => Some(text::Edit::backspace()),
        ui::Key::Delete => Some(text::Edit::delete()),
        ui::Key::Enter if multiline => Some(text::Edit::insert_line_break()),
        _ if inserted_text.is_some_and(|text| text.chars().all(|c| !c.is_control())) => {
            Some(text::Edit::insert(inserted_text.unwrap_or_default()))
        }
        ui::Key::Space => Some(text::Edit::insert(" ")),
        ui::Key::Character(character) if !character.is_control() => {
            Some(text::Edit::insert(character.to_string()))
        }
        _ => None,
    }
}

fn jump_modifier_pressed(modifiers: ui::Modifiers) -> bool {
    if cfg!(target_os = "macos") {
        modifiers.alt()
    } else {
        modifiers.control()
    }
}

#[cfg(test)]
pub fn key_pressed<T>(
    registry: &action::Registry<T>,
    state: &mut WindowState,
    window: window::Id,
    key: ui::Key,
    repeat: bool,
) -> Outcome {
    let mut text_engine = text::Engine::new();

    key_pressed_with_text(registry, state, window, key, None, repeat, &mut text_engine)
}

pub fn key_pressed_with_text<T>(
    registry: &action::Registry<T>,
    state: &mut WindowState,
    window: window::Id,
    key: ui::Key,
    inserted_text: Option<&str>,
    repeat: bool,
    text_engine: &mut text::Engine,
) -> Outcome {
    let target = state.focused_path();
    let mut redraw = false;
    let mut intent = None;

    let event = ui::Event::KeyDown {
        key,
        modifiers: state.modifiers,
        target: target.clone(),
        repeat,
    };
    let mut events = vec![event];

    let target_is_multiline = target
        .as_ref()
        .and_then(|target| state.text_surface(target))
        .is_some_and(|surface| surface.buffer().is_multiline());

    if let Some(edit) = text_edit_for_key(key, state.modifiers, inserted_text, target_is_multiline)
        && let Some(target) = target
            .as_ref()
            .filter(|target| state.can_apply_text_edit(target, &edit))
    {
        if state.is_editable_text_field(target) {
            state.reset_text_field_caret_blink(target, Instant::now());
        }
        events.push(ui::Event::TextEditRequested {
            target: target.clone(),
            edit,
        });

        return Outcome {
            events,
            request: None,
            intent: None,
            redraw: true,
        };
    }

    match key {
        ui::Key::ContextMenu if !repeat => {
            redraw |= open_keyboard_text_context_menu(state, window, text_engine);
        }
        ui::Key::F10 if state.modifiers.shift() && !repeat => {
            redraw |= open_keyboard_text_context_menu(state, window, text_engine);
        }
        ui::Key::ArrowUp | ui::Key::ArrowDown if state.floating.has_open_surface() => {
            let reverse = key == ui::Key::ArrowUp;
            if let Some(path) = next_focus(registry, state, window, reverse) {
                redraw |= state.set_focus(
                    path.clone(),
                    ui::focus::Reason::Keyboard,
                    ui::focus::Visibility::Visible,
                );
                intent = focus_menu_intent(state, &path);
            }
        }
        ui::Key::Tab => {
            let reverse = state.modifiers.shift();
            if let Some(path) = next_focus(registry, state, window, reverse) {
                redraw |= state.set_focus(
                    path.clone(),
                    ui::focus::Reason::Keyboard,
                    ui::focus::Visibility::Visible,
                );
                intent = focus_menu_intent(state, &path);
            }
        }
        ui::Key::Enter | ui::Key::Space if !repeat => {
            if let Some(path) = invokable_focused_path(registry, state, window) {
                redraw |= state.pressed.as_ref() != Some(&path)
                    || state.pressed_source != Some(PressSource::Keyboard);
                state.pressed = Some(path);
                state.pressed_source = Some(PressSource::Keyboard);
            }
        }
        ui::Key::Escape if !repeat => {
            redraw |= if state.open_submenu.is_some() {
                state.close_submenu()
            } else {
                state.close_menu()
            };
        }
        _ => {}
    }

    let request = if !repeat {
        shortcut_request(registry, state, window, key)
    } else {
        None
    };

    Outcome {
        events,
        request,
        intent,
        redraw,
    }
}

pub fn key_released<T>(
    registry: &action::Registry<T>,
    state: &mut WindowState,
    window: window::Id,
    key: ui::Key,
) -> Outcome {
    let target = state.focused_path();
    let event = ui::Event::KeyUp {
        key,
        modifiers: state.modifiers,
        target: target.clone(),
    };
    let mut redraw = false;
    let mut request = None;
    let mut intent = None;

    if matches!(key, ui::Key::Enter | ui::Key::Space)
        && state.pressed_source == Some(PressSource::Keyboard)
    {
        let pressed = state.pressed.take();
        state.pressed_source = None;
        redraw = pressed.is_some();

        if pressed == target {
            intent = target.as_ref().and_then(|target| {
                state.intent(target).map(|intent| {
                    IntentRequest::new(target.clone(), intent, action::Source::Keyboard)
                })
            });
            request = target.and_then(|target| {
                activation_request(registry, state, window, target, action::Source::Keyboard)
            });
            redraw |= request
                .as_ref()
                .and_then(action::Request::origin)
                .is_some_and(|origin| state.is_menu_path(origin))
                && state.close_menu_with_focus_visibility(Some(ui::focus::Visibility::Visible));
            redraw |= request.as_ref().is_some_and(|request| {
                restore_action_target_focus(state, request, action::Source::Keyboard)
            });
        }
    }

    Outcome {
        events: vec![event],
        request,
        intent,
        redraw,
    }
}

pub fn ime(state: &mut WindowState, event: Ime) -> Outcome {
    match event {
        Ime::Enabled => Outcome::default(),
        Ime::Preedit(text, selection) => {
            let preedit = (!text.is_empty()).then(|| text::Preedit::new(text, selection));
            let redraw = state.set_focused_text_field_preedit(preedit).is_some();

            Outcome {
                redraw,
                ..Outcome::default()
            }
        }
        Ime::Commit(text) => {
            let Some(target) = state.focused_editable_text_field() else {
                return Outcome::default();
            };

            state.set_focused_text_field_preedit(None);
            state.reset_text_field_caret_blink(&target, Instant::now());

            Outcome {
                events: vec![ui::Event::TextEditRequested {
                    target,
                    edit: text::Edit::ime_commit(text),
                }],
                redraw: true,
                ..Outcome::default()
            }
        }
        Ime::Disabled => Outcome {
            redraw: state.clear_text_field_preedits(),
            ..Outcome::default()
        },
    }
}

fn activation_request<T>(
    registry: &action::Registry<T>,
    state: &WindowState,
    window: window::Id,
    target: ui::Path,
    source: action::Source,
) -> Option<action::Request> {
    if matches!(
        state.intent(&target),
        Some(ui::Intent::OpenMenu(_) | ui::Intent::OpenSubmenu(_) | ui::Intent::CloseSubmenu)
    ) {
        return None;
    }

    action_request(state, window, target, source).filter(|request| registry.can_execute(request))
}

fn restore_action_target_focus(
    state: &mut WindowState,
    request: &action::Request,
    source: action::Source,
) -> bool {
    let Some(origin) = request.origin() else {
        return false;
    };
    let action::Scope::Path(target) = request.target().scope() else {
        return false;
    };
    if state.is_menu_path(origin) {
        return false;
    }
    if !state.is_focusable(target) {
        return false;
    }
    if state.command_subject(origin) == ui::CommandSubject::Origin {
        return false;
    }

    let visibility = state.focus_visibility_for_activation(target, source);

    state.set_focus(target.clone(), ui::focus::Reason::Programmatic, visibility)
}

fn hover_menu_intent(state: &WindowState, target: &ui::Path) -> Option<IntentRequest> {
    menu_navigation_intent(state, target, action::Source::Pointer)
}

fn focus_menu_intent(state: &WindowState, target: &ui::Path) -> Option<IntentRequest> {
    menu_navigation_intent(state, target, action::Source::Keyboard)
}

fn menu_navigation_intent(
    state: &WindowState,
    target: &ui::Path,
    source: action::Source,
) -> Option<IntentRequest> {
    let intent = state.intent(target);

    match intent {
        Some(ui::Intent::OpenMenu(menu)) if state.open_menu.is_some_and(|open| open != menu) => {
            return Some(IntentRequest::new(
                target.clone(),
                ui::Intent::OpenMenu(menu),
                source,
            ));
        }
        Some(ui::Intent::OpenSubmenu(menu))
            if state.open_menu.is_some() && state.open_submenu != Some(menu) =>
        {
            return Some(IntentRequest::new(
                target.clone(),
                ui::Intent::OpenSubmenu(menu),
                source,
            ));
        }
        Some(ui::Intent::CloseSubmenu)
            if state.open_submenu.is_some() && state.is_top_menu_popup_path(target) =>
        {
            return Some(IntentRequest::new(
                target.clone(),
                ui::Intent::CloseSubmenu,
                source,
            ));
        }
        Some(ui::Intent::Action(_))
            if state.open_submenu.is_some() && state.is_top_menu_popup_path(target) =>
        {
            return Some(IntentRequest::new(
                target.clone(),
                ui::Intent::CloseSubmenu,
                source,
            ));
        }
        _ => {}
    }

    None
}

fn next_focus<T>(
    registry: &action::Registry<T>,
    state: &WindowState,
    window: window::Id,
    reverse: bool,
) -> Option<ui::Path> {
    let order = focusable_paths(registry, state, window);
    if order.is_empty() {
        return None;
    }

    let current = state
        .focused_path()
        .and_then(|focused| order.iter().position(|path| path == &focused));
    let next = match (current, reverse) {
        (Some(0), true) | (None, true) => order.len() - 1,
        (Some(index), true) => index - 1,
        (Some(index), false) => (index + 1) % order.len(),
        (None, false) => 0,
    };

    Some(order[next].clone())
}

fn focusable_paths<T>(
    registry: &action::Registry<T>,
    state: &WindowState,
    window: window::Id,
) -> Vec<ui::Path> {
    let paths = state
        .composition
        .as_ref()
        .map(ui::Composition::focus_order)
        .unwrap_or(&[])
        .iter()
        .filter(|path| can_focus(registry, state, window, path))
        .cloned()
        .collect::<Vec<_>>();

    if state.open_menu.is_some() || state.floating.has_open_surface() {
        let menu_paths = paths
            .iter()
            .filter(|path| state.is_dropdown_path(path))
            .cloned()
            .collect::<Vec<_>>();

        if !menu_paths.is_empty() {
            return menu_paths;
        }
    }

    paths
}

fn open_keyboard_text_context_menu(
    state: &mut WindowState,
    window: window::Id,
    text_engine: &mut text::Engine,
) -> bool {
    let Some(target) = text_input::editing_target(state) else {
        return false;
    };
    let anchor = state
        .focused_text_field_caret_rect(text_engine)
        .or_else(|| state.text_field_rect(&target))
        .map(|rect| rect.origin)
        .unwrap_or_else(|| point::logical(0.0, 0.0));

    state.open_text_context_menu(window, target, anchor, action::Source::Keyboard)
}

fn can_focus<T>(
    registry: &action::Registry<T>,
    state: &WindowState,
    window: window::Id,
    path: &ui::Path,
) -> bool {
    if !state.is_focusable(path) {
        return false;
    }

    let Some(action) = state
        .composition
        .as_ref()
        .and_then(|composition| composition.action(path))
    else {
        return true;
    };

    registry.can_invoke(action, state.action_context_for_path(window, path))
}

fn invokable_focused_path<T>(
    registry: &action::Registry<T>,
    state: &WindowState,
    window: window::Id,
) -> Option<ui::Path> {
    let target = state.focused_path()?;
    if matches!(
        state.intent(&target),
        Some(ui::Intent::OpenMenu(_) | ui::Intent::OpenSubmenu(_))
    ) {
        return Some(target);
    }

    action_request(state, window, target.clone(), action::Source::Keyboard)
        .filter(|request| registry.can_execute(request))
        .map(|_| target)
}

fn shortcut_request<T>(
    registry: &action::Registry<T>,
    state: &WindowState,
    window: window::Id,
    key: ui::Key,
) -> Option<action::Request> {
    let shortcut = action::Shortcut::new(key.normalized(), state.modifiers);
    let action = registry.shortcut_action(shortcut)?;
    let context = state.command_context(window);

    Some(action::Request::new(
        action,
        action::Source::Shortcut,
        context,
    ))
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;
    use std::time::{Duration, Instant};

    use crate::Action;
    use crate::app::state::{Focus, FocusState};
    use crate::widget::{self, menu};

    use super::*;

    const CHILD: ui::Id = ui::Id::new("child");
    const SECOND: ui::Id = ui::Id::new("second");
    const ROOT: ui::Id = ui::Id::new("root");
    const MENU_BAR: ui::Id = ui::Id::new("menu_bar");
    const ORIGIN_BUTTON: ui::Id = ui::Id::new("origin_button");
    const CLICK: action::Id = action::Id::new("click");
    const FILE: menu::Id = menu::Id::new("file");
    const EDIT: menu::Id = menu::Id::new("edit");
    const PANELS: menu::Id = menu::Id::new("panels");

    fn path(id: ui::Id) -> ui::Path {
        ui::Path::from(id)
    }

    fn root_path(id: ui::Id) -> ui::Path {
        ui::Path::new([ROOT, id])
    }

    fn text_drag_copy_modifier() -> ui::Modifiers {
        ui::Modifiers::new(
            false,
            !cfg!(target_os = "macos"),
            cfg!(target_os = "macos"),
            false,
        )
    }

    fn scroll_state(offset: point::Logical) -> WindowState {
        let mut tree = ui::Tree::new();
        tree.set_root(
            widget::scroll_view(CHILD)
                .with_scroll_offset(offset)
                .with_size(
                    crate::layout::Size::Fixed(40.0),
                    crate::layout::Size::Fixed(40.0),
                )
                .with_child(
                    ui::Node::leaf(SECOND)
                        .with_size(crate::layout::Size::Fill, crate::layout::Size::Fixed(30.0))
                        .with_interactivity(ui::Interactivity::NONE.with_hit_test(true)),
                )
                .with_child(
                    ui::Node::leaf(ui::Id::new("third"))
                        .with_size(crate::layout::Size::Fill, crate::layout::Size::Fixed(30.0))
                        .with_interactivity(ui::Interactivity::NONE.with_hit_test(true)),
                )
                .with_child(
                    ui::Node::leaf(ui::Id::new("fourth"))
                        .with_size(crate::layout::Size::Fill, crate::layout::Size::Fixed(30.0))
                        .with_interactivity(ui::Interactivity::NONE.with_hit_test(true)),
                ),
        );
        let mut measurer = crate::text::Engine::new();
        let layout = tree
            .layout(crate::geometry::area::logical(40.0, 40.0), &mut measurer)
            .expect("scroll test tree should layout");
        let widget_metrics = tree.widget_metrics(&layout);

        state_with_composition(
            layout,
            HashMap::new(),
            HashMap::new(),
            HashMap::new(),
            HashMap::new(),
            HashMap::new(),
            widget_metrics,
            Vec::new(),
        )
    }

    fn non_overflow_scroll_state() -> WindowState {
        let mut tree = ui::Tree::new();
        tree.set_root(
            widget::scroll_view(CHILD)
                .with_size(
                    crate::layout::Size::Fixed(40.0),
                    crate::layout::Size::Fixed(40.0),
                )
                .with_child(
                    ui::Node::leaf(SECOND)
                        .with_size(crate::layout::Size::Fill, crate::layout::Size::Fixed(20.0))
                        .with_interactivity(ui::Interactivity::NONE.with_hit_test(true)),
                ),
        );
        let mut measurer = crate::text::Engine::new();
        let layout = tree
            .layout(crate::geometry::area::logical(40.0, 40.0), &mut measurer)
            .expect("scroll test tree should layout");
        let widget_metrics = tree.widget_metrics(&layout);

        state_with_composition(
            layout,
            HashMap::new(),
            HashMap::new(),
            HashMap::new(),
            HashMap::new(),
            HashMap::new(),
            widget_metrics,
            Vec::new(),
        )
    }

    fn requested_offset(events: &[ui::Event]) -> Option<point::Logical> {
        events.iter().find_map(|event| match event {
            ui::Event::ScrollRequested { offset, .. } => Some(*offset),
            _ => None,
        })
    }

    fn single_box(id: ui::Id) -> crate::ui::Frame {
        crate::layout::Frame::<ui::Path>::new(
            ui::Path::root(id),
            crate::geometry::Rect::new(
                point::logical(0.0, 0.0),
                crate::geometry::area::logical(20.0, 20.0),
            ),
            Vec::new(),
        )
    }

    fn path_box(path: ui::Path) -> crate::ui::Frame {
        crate::layout::Frame::<ui::Path>::with_path(
            path,
            crate::geometry::Rect::new(
                point::logical(0.0, 0.0),
                crate::geometry::area::logical(20.0, 20.0),
            ),
            Vec::new(),
        )
    }

    fn composition(
        layout: crate::ui::Frame,
        actions: HashMap<ui::Path, action::Id>,
        command_subjects: HashMap<ui::Path, ui::CommandSubject>,
        intents: HashMap<ui::Path, ui::Intent>,
        responders: HashMap<ui::Path, Vec<action::Id>>,
        interactivity: HashMap<ui::Path, ui::Interactivity>,
        widget_metrics: HashMap<ui::Path, widget::Metrics>,
        focus_order: Vec<ui::Path>,
    ) -> ui::Composition {
        ui::Composition::for_test(
            layout,
            HashMap::new(),
            actions,
            command_subjects,
            intents,
            responders,
            Vec::new(),
            interactivity,
            widget_metrics,
            focus_order,
        )
    }

    fn state_with_composition(
        layout: crate::ui::Frame,
        actions: HashMap<ui::Path, action::Id>,
        command_subjects: HashMap<ui::Path, ui::CommandSubject>,
        intents: HashMap<ui::Path, ui::Intent>,
        responders: HashMap<ui::Path, Vec<action::Id>>,
        interactivity: HashMap<ui::Path, ui::Interactivity>,
        widget_metrics: HashMap<ui::Path, widget::Metrics>,
        focus_order: Vec<ui::Path>,
    ) -> WindowState {
        WindowState {
            composition: Some(composition(
                layout,
                actions,
                command_subjects,
                intents,
                responders,
                interactivity,
                widget_metrics,
                focus_order,
            )),
            ..WindowState::default()
        }
    }

    fn text_field_state() -> WindowState {
        text_field_state_with_field(crate::text::Buffer::from_text("hello"))
    }

    fn text_field_state_with_field(field: impl Into<crate::text::Field>) -> WindowState {
        let window = window::Id::new(1);
        let mut tree = ui::Tree::new();
        let mut registry = action::Registry::<()>::new();
        let mut measurer = crate::text::Engine::new();

        tree.set_root(widget::text_field(CHILD, field).with_size(
            crate::layout::Size::Fixed(100.0),
            crate::layout::Size::Fixed(24.0),
        ));

        let composition = tree
            .compose(
                window,
                crate::geometry::area::logical(100.0, 24.0),
                &mut registry,
                &[],
                &mut measurer,
            )
            .expect("text field tree should compose");

        WindowState {
            composition: Some(composition),
            focus: FocusState::focused(Focus::new(
                path(CHILD),
                ui::focus::Reason::Keyboard,
                ui::focus::Visibility::Visible,
            )),
            ..WindowState::default()
        }
    }

    fn two_text_field_state(
        first: impl Into<crate::text::Field>,
        second: impl Into<crate::text::Field>,
    ) -> WindowState {
        let window = window::Id::new(1);
        let mut tree = ui::Tree::new();
        let mut registry = action::Registry::<()>::new();
        let mut measurer = crate::text::Engine::new();

        tree.set_root(
            ui::Node::container(ROOT, crate::layout::Axis::Vertical)
                .with_child(widget::text_field(CHILD, first).with_size(
                    crate::layout::Size::Fixed(100.0),
                    crate::layout::Size::Fixed(24.0),
                ))
                .with_child(widget::text_field(SECOND, second).with_size(
                    crate::layout::Size::Fixed(100.0),
                    crate::layout::Size::Fixed(24.0),
                )),
        );

        let composition = tree
            .compose(
                window,
                crate::geometry::area::logical(100.0, 48.0),
                &mut registry,
                &[],
                &mut measurer,
            )
            .expect("two text field tree should compose");

        WindowState {
            composition: Some(composition),
            focus: FocusState::focused(Focus::new(
                root_path(CHILD),
                ui::focus::Reason::Keyboard,
                ui::focus::Visibility::Visible,
            )),
            ..WindowState::default()
        }
    }

    fn text_field_state_with_registry(
        buffer: crate::text::Buffer,
        registry: &mut action::Registry<()>,
    ) -> WindowState {
        let window = window::Id::new(1);
        let mut tree = ui::Tree::new();
        let mut measurer = crate::text::Engine::new();

        tree.set_root(widget::text_field(CHILD, buffer).with_size(
            crate::layout::Size::Fixed(100.0),
            crate::layout::Size::Fixed(24.0),
        ));

        let composition = tree
            .compose(
                window,
                crate::geometry::area::logical(100.0, 24.0),
                registry,
                &[],
                &mut measurer,
            )
            .expect("text field tree should compose");

        let mut state = WindowState {
            composition: Some(composition),
            focus: FocusState::focused(Focus::new(
                path(CHILD),
                ui::focus::Reason::Keyboard,
                ui::focus::Visibility::Visible,
            )),
            ..WindowState::default()
        };
        crate::app::text_input::sync_session(&mut state);
        crate::app::text_input::publish_action_states(&state, registry, window);
        state
    }

    fn text_field_command_tree(open_menu: bool) -> WindowState {
        let window = window::Id::new(1);
        let field = root_path(CHILD);
        let mut tree = ui::Tree::new();
        let mut registry = action::Registry::<()>::new();
        let mut measurer = crate::text::Engine::new();
        let mut floating = crate::app::floating::State::default();

        registry.register(Action::new(action::SELECT_ALL, "Select All"));
        registry.register(Action::new(CLICK, "Click"));
        tree.set_root(
            ui::Node::container(ROOT, crate::layout::Axis::Vertical)
                .with_child(widget::menu_bar(
                    MENU_BAR,
                    menu::Bar::new().menu(
                        menu::Menu::new(FILE, "File").section(
                            menu::Section::new().item(menu::Item::new(action::SELECT_ALL)),
                        ),
                    ),
                ))
                .with_child(
                    widget::text_field(CHILD, crate::text::Buffer::from_text("hello")).with_size(
                        crate::layout::Size::Fixed(100.0),
                        crate::layout::Size::Fixed(24.0),
                    ),
                )
                .with_child(
                    ui::Node::leaf(SECOND)
                        .with_action(action::SELECT_ALL)
                        .with_command_subject(ui::CommandSubject::Current)
                        .with_interactivity(ui::Interactivity::CONTROL)
                        .with_size(
                            crate::layout::Size::Fixed(20.0),
                            crate::layout::Size::Fixed(20.0),
                        ),
                )
                .with_child(
                    ui::Node::leaf(ORIGIN_BUTTON)
                        .with_action(CLICK)
                        .with_interactivity(ui::Interactivity::CONTROL)
                        .with_size(
                            crate::layout::Size::Fixed(20.0),
                            crate::layout::Size::Fixed(20.0),
                        ),
                ),
        );
        if open_menu {
            floating.open_top_menu(
                FILE,
                action::Context::path(window, field.clone()),
                action::Source::Pointer,
                ui::floating::FocusPolicy::PreserveCurrentFocus,
            );
        }
        let composition = tree
            .compose(
                window,
                crate::geometry::area::logical(180.0, 120.0),
                &mut registry,
                floating.surfaces(),
                &mut measurer,
            )
            .expect("command text field tree should compose");

        let mut state = WindowState {
            composition: Some(composition),
            focus: FocusState::focused(Focus::new(
                field.clone(),
                ui::focus::Reason::Keyboard,
                ui::focus::Visibility::Visible,
            )),
            command_subject: Some(action::Scope::Path(field)),
            floating,
            ..WindowState::default()
        };
        state.sync_open_menu_mirrors();
        state.update_command_scope_captures(window);
        state.sync_menu_focus_scopes();
        crate::app::text_input::sync_session(&mut state);
        state
    }

    fn select_all_menu_row(state: &WindowState) -> ui::Path {
        state
            .composition
            .as_ref()
            .expect("state should have composition")
            .actions()
            .iter()
            .find_map(|(path, action)| {
                (*action == action::SELECT_ALL && state.is_menu_path(path)).then(|| path.clone())
            })
            .expect("open menu should contain select all row")
    }

    fn pointer_pressed_for_test(
        state: &mut WindowState,
        position: point::Logical,
        button: pointer::Button,
    ) -> Outcome {
        let mut text_engine = crate::text::Engine::new();
        pointer_pressed(
            state,
            window::Id::new(1),
            position,
            button,
            &mut text_engine,
        )
    }

    fn primary_click_for_test(
        state: &mut WindowState,
        text_engine: &mut crate::text::Engine,
        position: point::Logical,
    ) -> Outcome {
        let window = window::Id::new(1);
        let registry = action::Registry::<()>::new();
        let outcome = pointer_pressed(
            state,
            window,
            position,
            pointer::Button::Primary,
            text_engine,
        );

        pointer_released(&registry, state, window, position, pointer::Button::Primary);
        outcome
    }

    fn assert_text_pointer_kind(outcome: &Outcome, expected: crate::text::PointerEditKind) {
        assert!(
            outcome.events.iter().any(|event| matches!(
                event,
                ui::Event::TextEditRequested {
                    edit: crate::text::Edit::Pointer { kind, .. },
                    ..
                } if *kind == expected
            )),
            "expected {expected:?} in events: {:?}",
            outcome.events
        );
    }

    #[test]
    fn pressed_control_focuses_and_requests_redraw() {
        let mut state = state_with_composition(
            single_box(CHILD),
            HashMap::new(),
            HashMap::new(),
            HashMap::new(),
            HashMap::new(),
            HashMap::from([(path(CHILD), ui::Interactivity::CONTROL)]),
            HashMap::new(),
            Vec::new(),
        );

        let outcome = pointer_pressed_for_test(
            &mut state,
            point::logical(1.0, 1.0),
            pointer::Button::Primary,
        );

        assert!(outcome.redraw);
        assert!(state.pointer.primary_down());
        assert_eq!(state.focused_path(), Some(path(CHILD)));
        assert_eq!(
            state.focus.as_ref().map(|focus| focus.reason),
            Some(ui::focus::Reason::Pointer)
        );
        assert_eq!(state.focus_visibility(), ui::focus::Visibility::Hidden);
        assert_eq!(state.pressed, Some(path(CHILD)));
    }

    #[test]
    fn pointer_down_on_menu_title_preserves_text_field_focus() {
        let mut state = text_field_command_tree(false);
        let title = state
            .composition
            .as_ref()
            .expect("composition")
            .intents()
            .iter()
            .find_map(|(path, intent)| {
                (*intent == ui::Intent::OpenMenu(FILE)).then(|| path.clone())
            })
            .expect("menu title path");
        let field = root_path(CHILD);

        let event = state.pointer_down(
            point::logical(1.0, 1.0),
            point::logical(0.0, 0.0),
            Some(title.clone()),
            pointer::Button::Primary,
        );

        assert_eq!(state.focused_path(), Some(field.clone()));
        assert_eq!(state.command_subject, Some(action::Scope::Path(field)));
        assert_eq!(state.pressed, Some(title.clone()));
        assert_eq!(
            event,
            ui::Event::PointerDown {
                position: point::logical(1.0, 1.0),
                delta: point::logical(0.0, 0.0),
                target: Some(title),
                button: pointer::Button::Primary,
            }
        );
    }

    #[test]
    fn floating_menu_captured_context_overrides_stale_command_subject() {
        let window = window::Id::new(1);
        let mut state = text_field_command_tree(true);
        let row = select_all_menu_row(&state);
        let field = root_path(CHILD);
        let stale = root_path(ORIGIN_BUTTON);

        state.command_subject = Some(action::Scope::Path(stale));
        state.update_command_scope_captures(window);

        let request = action_request(&state, window, row, action::Source::Pointer)
            .expect("captured menu row should request select all");

        assert_eq!(request.action(), action::SELECT_ALL);
        assert_eq!(request.target(), &action::Context::path(window, field));
    }

    #[test]
    fn wheel_event_routes_to_scrollable_under_pointer() {
        let mut state = scroll_state(point::logical(0.0, 12.0));
        let mut text_engine = text::Engine::new();

        let outcome = scroll_wheel(
            &mut state,
            point::logical(1.0, 1.0),
            point::logical(0.0, -20.0),
            &mut text_engine,
        );

        assert!(!outcome.redraw);
        assert_eq!(
            outcome.events,
            vec![
                ui::Event::ScrollWheel {
                    position: point::logical(1.0, 1.0),
                    delta: point::logical(0.0, -20.0),
                    target: Some(path(CHILD))
                },
                ui::Event::ScrollRequested {
                    target: path(CHILD),
                    offset: point::logical(0.0, 32.0)
                }
            ]
        );
    }

    #[test]
    fn wheel_event_outside_scrollable_has_no_target() {
        let mut state = scroll_state(point::logical(0.0, 12.0));
        let mut text_engine = text::Engine::new();

        let outcome = scroll_wheel(
            &mut state,
            point::logical(50.0, 50.0),
            point::logical(0.0, -20.0),
            &mut text_engine,
        );

        assert_eq!(
            outcome.events,
            vec![ui::Event::ScrollWheel {
                position: point::logical(50.0, 50.0),
                delta: point::logical(0.0, -20.0),
                target: None
            }]
        );
    }

    #[test]
    fn thumb_drag_emits_clamped_scroll_request_and_preserves_capture() {
        let mut state = scroll_state(point::logical(0.0, 0.0));

        let pressed = pointer_pressed_for_test(
            &mut state,
            point::logical(35.0, 5.0),
            pointer::Button::Primary,
        );
        assert!(pressed.redraw);
        assert!(state.pointer_capture.is_some());
        assert_eq!(state.pressed, Some(path(CHILD)));

        let moved = pointer_moved(&mut state, point::logical(35.0, 20.0));
        let offset = requested_offset(&moved.events).expect("drag should request scroll");

        assert_eq!(offset.x(), 0.0);
        assert!((offset.y() - 34.09091).abs() < 0.001);
        assert!(state.pointer_capture.is_some());
    }

    #[test]
    fn scrollbar_track_click_pages_by_one_viewport() {
        let mut state = scroll_state(point::logical(0.0, 0.0));

        let outcome = pointer_pressed_for_test(
            &mut state,
            point::logical(35.0, 35.0),
            pointer::Button::Primary,
        );

        assert_eq!(
            requested_offset(&outcome.events),
            Some(point::logical(0.0, 40.0))
        );
        assert!(state.pointer_capture.is_none());
    }

    #[test]
    fn non_overflowing_scrollbar_draws_but_ignores_drag_and_track_input() {
        let mut state = non_overflow_scroll_state();

        let thumb = pointer_pressed_for_test(
            &mut state,
            point::logical(35.0, 5.0),
            pointer::Button::Primary,
        );

        assert!(state.pointer_capture.is_none());
        assert_eq!(requested_offset(&thumb.events), None);

        let mut text_engine = text::Engine::new();
        let wheel = scroll_wheel(
            &mut state,
            point::logical(1.0, 1.0),
            point::logical(0.0, -20.0),
            &mut text_engine,
        );
        assert_eq!(requested_offset(&wheel.events), None);
    }

    #[test]
    fn released_control_returns_contextual_action_request() {
        let window = window::Id::new(1);
        let mut registry = action::Registry::<()>::new();
        let mut state = state_with_composition(
            single_box(CHILD),
            HashMap::from([(path(CHILD), CLICK)]),
            HashMap::new(),
            HashMap::new(),
            HashMap::new(),
            HashMap::from([(path(CHILD), ui::Interactivity::CONTROL)]),
            HashMap::new(),
            Vec::new(),
        );
        state.pressed = Some(path(CHILD));
        state.pressed_source = Some(PressSource::Pointer);
        registry.register(Action::new(CLICK, "Click"));

        let outcome = pointer_released(
            &registry,
            &mut state,
            window,
            point::logical(1.0, 1.0),
            pointer::Button::Primary,
        );

        assert_eq!(
            outcome.request,
            Some(
                action::Request::new(
                    CLICK,
                    action::Source::Pointer,
                    action::Context::path(window, path(CHILD))
                )
                .with_origin(path(CHILD))
            )
        );
        assert!(!state.pointer.primary_down());
    }

    #[test]
    fn released_menu_title_returns_open_menu_intent() {
        let window = window::Id::new(1);
        let registry = action::Registry::<()>::new();
        let mut state = state_with_composition(
            single_box(CHILD),
            HashMap::new(),
            HashMap::new(),
            HashMap::from([(path(CHILD), ui::Intent::OpenMenu(FILE))]),
            HashMap::new(),
            HashMap::from([(path(CHILD), ui::Interactivity::CONTROL)]),
            HashMap::new(),
            Vec::new(),
        );
        state.pressed = Some(path(CHILD));
        state.pressed_source = Some(PressSource::Pointer);

        let outcome = pointer_released(
            &registry,
            &mut state,
            window,
            point::logical(1.0, 1.0),
            pointer::Button::Primary,
        );

        assert_eq!(outcome.request, None);
        assert_eq!(
            outcome.intent,
            Some(IntentRequest::new(
                path(CHILD),
                ui::Intent::OpenMenu(FILE),
                action::Source::Pointer,
            ))
        );
    }

    #[test]
    fn menu_action_request_closes_open_menu() {
        let window = window::Id::new(1);
        let mut registry = action::Registry::<()>::new();
        let row = ui::Path::new([widget::MENU_POPUP, CHILD]);
        let mut state = state_with_composition(
            crate::layout::Frame::<ui::Path>::with_path(
                ui::Path::from(widget::MENU_POPUP),
                crate::geometry::Rect::new(
                    point::logical(0.0, 0.0),
                    crate::geometry::area::logical(20.0, 20.0),
                ),
                vec![crate::layout::Frame::<ui::Path>::with_path(
                    row.clone(),
                    crate::geometry::Rect::new(
                        point::logical(0.0, 0.0),
                        crate::geometry::area::logical(10.0, 10.0),
                    ),
                    Vec::new(),
                )],
            ),
            HashMap::from([(row.clone(), CLICK)]),
            HashMap::new(),
            HashMap::from([(row.clone(), ui::Intent::Action(CLICK))]),
            HashMap::new(),
            HashMap::from([(row.clone(), ui::Interactivity::CONTROL)]),
            HashMap::new(),
            Vec::new(),
        );
        state.pressed = Some(row.clone());
        state.pressed_source = Some(PressSource::Pointer);
        state.open_menu = Some(FILE);
        registry.register(Action::new(CLICK, "Click"));

        let outcome = pointer_released(
            &registry,
            &mut state,
            window,
            point::logical(1.0, 1.0),
            pointer::Button::Primary,
        );

        assert!(outcome.request.is_some());
        assert_eq!(state.open_menu, None);
    }

    #[test]
    fn pointer_menu_action_restores_focus_to_text_field_command_target() {
        let window = window::Id::new(1);
        let mut registry = action::Registry::<()>::new();
        let mut state = text_field_command_tree(true);
        let field = root_path(CHILD);
        let row = select_all_menu_row(&state);
        registry.register(Action::new(action::SELECT_ALL, "Select All"));
        assert!(state.set_focus(
            row.clone(),
            ui::focus::Reason::Pointer,
            ui::focus::Visibility::Hidden,
        ));
        state.pressed = Some(row);
        state.pressed_source = Some(PressSource::Pointer);

        let outcome = pointer_released(
            &registry,
            &mut state,
            window,
            point::logical(1.0, 1.0),
            pointer::Button::Primary,
        );

        assert_eq!(state.open_menu, None);
        assert_eq!(state.focused_path(), Some(field.clone()));
        assert_eq!(state.focus_visibility(), ui::focus::Visibility::Visible);
        assert_eq!(
            outcome.request.as_ref().map(action::Request::target),
            Some(&action::Context::path(window, field))
        );
    }

    #[test]
    fn keyboard_menu_action_restores_visible_focus_to_text_field_command_target() {
        let window = window::Id::new(1);
        let mut registry = action::Registry::<()>::new();
        let mut state = text_field_command_tree(true);
        let field = root_path(CHILD);
        let row = select_all_menu_row(&state);
        registry.register(Action::new(action::SELECT_ALL, "Select All"));
        assert!(state.set_focus(
            row.clone(),
            ui::focus::Reason::Keyboard,
            ui::focus::Visibility::Visible,
        ));
        state.pressed = Some(row);
        state.pressed_source = Some(PressSource::Keyboard);

        let outcome = key_released(&registry, &mut state, window, ui::Key::Enter);

        assert_eq!(state.open_menu, None);
        assert_eq!(state.focused_path(), Some(field.clone()));
        assert_eq!(state.focus_visibility(), ui::focus::Visibility::Visible);
        assert_eq!(
            outcome.request.as_ref().map(action::Request::target),
            Some(&action::Context::path(window, field))
        );
    }

    #[test]
    fn command_subject_button_restores_focus_to_text_field_command_target() {
        let window = window::Id::new(1);
        let mut registry = action::Registry::<()>::new();
        let mut state = text_field_command_tree(false);
        let field = root_path(CHILD);
        let button = root_path(SECOND);
        registry.register(Action::new(action::SELECT_ALL, "Select All"));
        state.focus = FocusState::focused(Focus::new(
            button.clone(),
            ui::focus::Reason::Pointer,
            ui::focus::Visibility::Hidden,
        ));
        state.pressed = Some(button);
        state.pressed_source = Some(PressSource::Pointer);

        let outcome = pointer_released(
            &registry,
            &mut state,
            window,
            point::logical(1.0, 1.0),
            pointer::Button::Primary,
        );

        assert_eq!(state.focused_path(), Some(field.clone()));
        assert_eq!(state.focus_visibility(), ui::focus::Visibility::Visible);
        assert_eq!(
            outcome.request.as_ref().map(action::Request::target),
            Some(&action::Context::path(window, field))
        );
    }

    #[test]
    fn origin_targeted_button_keeps_its_own_focus_after_activation() {
        let window = window::Id::new(1);
        let mut registry = action::Registry::<()>::new();
        let mut state = text_field_command_tree(false);
        let button = root_path(ORIGIN_BUTTON);
        registry.register(Action::new(CLICK, "Click"));
        state.focus = FocusState::focused(Focus::new(
            button.clone(),
            ui::focus::Reason::Pointer,
            ui::focus::Visibility::Hidden,
        ));
        state.pressed = Some(button.clone());
        state.pressed_source = Some(PressSource::Pointer);

        let outcome = pointer_released(
            &registry,
            &mut state,
            window,
            point::logical(1.0, 1.0),
            pointer::Button::Primary,
        );

        assert_eq!(state.focused_path(), Some(button.clone()));
        assert_eq!(
            outcome.request.as_ref().map(action::Request::target),
            Some(&action::Context::path(window, button))
        );
    }

    #[test]
    fn winit_mouse_buttons_map_to_pointer_buttons() {
        assert_eq!(pointer_button(MouseButton::Left), pointer::Button::Primary);
        assert_eq!(
            pointer_button(MouseButton::Right),
            pointer::Button::Secondary
        );
        assert_eq!(pointer_button(MouseButton::Middle), pointer::Button::Middle);
        assert_eq!(pointer_button(MouseButton::Back), pointer::Button::Back);
        assert_eq!(
            pointer_button(MouseButton::Forward),
            pointer::Button::Forward
        );
        assert_eq!(
            pointer_button(MouseButton::Other(9)),
            pointer::Button::Other(9)
        );
    }

    #[test]
    fn character_keys_preserve_text_case_and_normalize_on_demand() {
        assert_eq!(
            key(&WinitKey::Character("A".into())),
            ui::Key::Character('A')
        );
        assert_eq!(
            key(&WinitKey::Character("A".into())).normalized(),
            ui::Key::Character('a')
        );
        assert_eq!(key(&WinitKey::Character("ab".into())), ui::Key::Other);
    }

    #[test]
    fn focused_text_field_character_key_emits_insert_edit_request() {
        let window = window::Id::new(1);
        let registry = action::Registry::<()>::new();
        let mut state = text_field_state();

        let outcome = key_pressed(
            &registry,
            &mut state,
            window,
            ui::Key::Character('A'),
            false,
        );

        assert!(outcome.events.contains(&ui::Event::TextEditRequested {
            target: path(CHILD),
            edit: crate::text::Edit::insert("A"),
        }));
    }
    #[test]
    fn focused_text_field_plain_character_edit_consumes_matching_shortcut() {
        let window = window::Id::new(1);
        let mut registry = action::Registry::<()>::new();
        let mut state = text_field_state();
        registry.register(Action::new(action::SELECT_ALL, "Plain A").with_shortcut(
            action::Shortcut::new(
                ui::Key::Character('a'),
                ui::Modifiers::new(false, false, false, false),
            ),
        ));

        let outcome = key_pressed(
            &registry,
            &mut state,
            window,
            ui::Key::Character('a'),
            false,
        );

        assert_eq!(outcome.request, None);
        assert!(outcome.events.contains(&ui::Event::TextEditRequested {
            target: path(CHILD),
            edit: crate::text::Edit::insert("a"),
        }));
    }

    #[test]
    fn read_only_text_field_suppresses_character_edit_requests() {
        let window = window::Id::new(1);
        let registry = action::Registry::<()>::new();
        let mut state = text_field_state_with_field(crate::text::Field::new("hello").read_only());

        let outcome = key_pressed(
            &registry,
            &mut state,
            window,
            ui::Key::Character('A'),
            false,
        );

        assert!(
            !outcome
                .events
                .iter()
                .any(|event| matches!(event, ui::Event::TextEditRequested { .. }))
        );
    }

    #[test]
    fn read_only_text_field_still_emits_selection_navigation_requests() {
        let window = window::Id::new(1);
        let registry = action::Registry::<()>::new();
        let mut state = text_field_state_with_field(crate::text::Field::new("hello").read_only());

        let outcome = key_pressed(&registry, &mut state, window, ui::Key::ArrowLeft, false);

        assert!(outcome.events.contains(&ui::Event::TextEditRequested {
            target: path(CHILD),
            edit: crate::text::Edit::motion(glyphon::cosmic_text::Motion::Left),
        }));
    }

    #[test]
    fn read_only_text_field_does_not_enable_ime_or_commit_text() {
        let mut state = text_field_state_with_field(crate::text::Field::new("hello").read_only());

        assert!(!state.text_input_enabled());
        let preedit = ime(&mut state, Ime::Preedit("compose".into(), Some((0, 1))));
        let commit = ime(&mut state, Ime::Commit("x".into()));

        assert!(!preedit.redraw);
        assert!(commit.events.is_empty());
    }

    #[test]
    fn focused_text_field_key_edit_resets_caret_blink() {
        let window = window::Id::new(1);
        let registry = action::Registry::<()>::new();
        let mut state = text_field_state();
        let old_epoch = Instant::now() - Duration::from_millis(500);
        state.text_field_states.insert(
            path(CHILD),
            crate::text::TextFieldState::new_at(0.0, old_epoch),
        );

        assert!(
            !state
                .text_field_states
                .get(&path(CHILD))
                .expect("text field state should exist")
                .caret_visible(Instant::now())
        );

        key_pressed(
            &registry,
            &mut state,
            window,
            ui::Key::Character('A'),
            false,
        );

        assert!(
            state
                .text_field_states
                .get(&path(CHILD))
                .expect("text field state should exist")
                .caret_visible(Instant::now())
        );
    }

    #[test]
    fn ime_commit_on_focused_text_field_emits_insert_edit_request() {
        let mut state = text_field_state();
        state.text_field_states.insert(
            path(CHILD),
            crate::text::TextFieldState::default()
                .with_preedit(Some(crate::text::Preedit::new("preedit", Some((0, 3))))),
        );

        let outcome = ime(&mut state, Ime::Commit("a\nb".into()));

        assert_eq!(
            outcome.events,
            vec![ui::Event::TextEditRequested {
                target: path(CHILD),
                edit: crate::text::Edit::ime_commit("a\nb")
            }]
        );
        assert!(outcome.redraw);
        assert_eq!(
            state
                .text_field_states
                .get(&path(CHILD))
                .and_then(crate::text::TextFieldState::preedit),
            None
        );
    }

    #[test]
    fn ime_preedit_is_stored_on_focused_text_field_and_cleared_when_disabled() {
        let mut state = text_field_state();

        let preedit = ime(&mut state, Ime::Preedit("compose".into(), Some((1, 4))));

        assert!(preedit.redraw);
        let stored = state
            .text_field_states
            .get(&path(CHILD))
            .and_then(crate::text::TextFieldState::preedit)
            .expect("preedit should be stored on focused text field");
        assert_eq!(stored.text(), "compose");
        assert_eq!(stored.selection(), Some((1, 4)));

        let disabled = ime(&mut state, Ime::Disabled);

        assert!(disabled.redraw);
        assert_eq!(
            state
                .text_field_states
                .get(&path(CHILD))
                .and_then(crate::text::TextFieldState::preedit),
            None
        );
    }

    #[test]
    fn focused_text_field_enables_text_input_and_exposes_caret_rect() {
        let mut state = text_field_state();
        let mut text_engine = crate::text::Engine::new();

        assert!(state.text_input_enabled());
        let rect = state
            .focused_text_field_caret_rect(&mut text_engine)
            .expect("focused text field should expose shaped caret rect");
        assert_eq!(rect.area.width(), 1.0);
        assert!(rect.area.height() > 0.0);

        state.clear_focus();

        assert!(!state.text_input_enabled());
        assert_eq!(state.focused_text_field_caret_rect(&mut text_engine), None);
    }

    #[test]
    fn focused_text_field_arrow_key_emits_cosmic_motion_edit_request() {
        let window = window::Id::new(1);
        let registry = action::Registry::<()>::new();
        let mut state = text_field_state();

        let outcome = key_pressed(&registry, &mut state, window, ui::Key::ArrowLeft, false);

        assert!(outcome.events.contains(&ui::Event::TextEditRequested {
            target: path(CHILD),
            edit: crate::text::Edit::motion(glyphon::cosmic_text::Motion::Left),
        }));
    }

    #[test]
    fn focused_text_field_shift_arrow_extends_selection() {
        let window = window::Id::new(1);
        let registry = action::Registry::<()>::new();
        let mut state = text_field_state();
        state.modifiers = ui::Modifiers::new(true, false, false, false);

        let outcome = key_pressed(&registry, &mut state, window, ui::Key::ArrowRight, false);

        assert!(outcome.events.contains(&ui::Event::TextEditRequested {
            target: path(CHILD),
            edit: crate::text::Edit::extend_motion(glyphon::cosmic_text::Motion::Right),
        }));
    }

    #[test]
    fn focused_text_field_jump_arrow_emits_word_motion() {
        let window = window::Id::new(1);
        let registry = action::Registry::<()>::new();
        let mut state = text_field_state();
        state.modifiers = if cfg!(target_os = "macos") {
            ui::Modifiers::new(false, false, true, false)
        } else {
            ui::Modifiers::new(false, true, false, false)
        };

        let outcome = key_pressed(&registry, &mut state, window, ui::Key::ArrowLeft, false);

        assert!(outcome.events.contains(&ui::Event::TextEditRequested {
            target: path(CHILD),
            edit: crate::text::Edit::motion(glyphon::cosmic_text::Motion::LeftWord),
        }));
    }

    #[test]
    fn focused_text_field_jump_delete_emits_word_delete() {
        let window = window::Id::new(1);
        let registry = action::Registry::<()>::new();
        let mut state = text_field_state();
        state.modifiers = if cfg!(target_os = "macos") {
            ui::Modifiers::new(false, false, true, false)
        } else {
            ui::Modifiers::new(false, true, false, false)
        };

        let outcome = key_pressed(&registry, &mut state, window, ui::Key::Backspace, false);

        assert!(outcome.events.contains(&ui::Event::TextEditRequested {
            target: path(CHILD),
            edit: crate::text::Edit::delete_word_backward(),
        }));
    }

    #[test]
    fn pointer_press_on_text_field_requests_cursor_placement() {
        let mut state = text_field_state();

        let outcome = pointer_pressed_for_test(
            &mut state,
            point::logical(50.0, 12.0),
            pointer::Button::Primary,
        );

        assert_eq!(state.focused_path(), Some(path(CHILD)));
        assert!(outcome.events.iter().any(|event| matches!(
            event,
            ui::Event::TextEditRequested {
                target,
                edit: crate::text::Edit::Pointer {
                    kind: crate::text::PointerEditKind::Click,
                    position
                }
            } if target == &path(CHILD)
                && position.index <= "hello".len()
        )));
    }

    #[test]
    fn repeated_text_clicks_cycle_word_and_full_selection_after_triple_click() {
        let mut state = text_field_state();
        let mut text_engine = crate::text::Engine::new();

        let first =
            primary_click_for_test(&mut state, &mut text_engine, point::logical(10.0, 12.0));
        let second =
            primary_click_for_test(&mut state, &mut text_engine, point::logical(10.0, 12.0));
        let third =
            primary_click_for_test(&mut state, &mut text_engine, point::logical(10.0, 12.0));
        let fourth =
            primary_click_for_test(&mut state, &mut text_engine, point::logical(10.0, 12.0));
        let fifth =
            primary_click_for_test(&mut state, &mut text_engine, point::logical(10.0, 12.0));

        assert_text_pointer_kind(&first, crate::text::PointerEditKind::Click);
        assert_text_pointer_kind(&second, crate::text::PointerEditKind::DoubleClick);
        assert_text_pointer_kind(&third, crate::text::PointerEditKind::TripleClick);
        assert_text_pointer_kind(&fourth, crate::text::PointerEditKind::DoubleClick);
        assert_text_pointer_kind(&fifth, crate::text::PointerEditKind::TripleClick);
    }

    #[test]
    fn text_click_count_resets_when_pointer_moves_too_far() {
        let mut state = text_field_state();
        let mut text_engine = crate::text::Engine::new();

        let first =
            primary_click_for_test(&mut state, &mut text_engine, point::logical(10.0, 12.0));
        let second =
            primary_click_for_test(&mut state, &mut text_engine, point::logical(30.0, 12.0));

        assert_text_pointer_kind(&first, crate::text::PointerEditKind::Click);
        assert_text_pointer_kind(&second, crate::text::PointerEditKind::Click);
    }

    #[test]
    fn secondary_click_on_text_field_opens_context_menu_without_moving_caret() {
        let mut state = text_field_state();

        let outcome = pointer_pressed_for_test(
            &mut state,
            point::logical(10.0, 12.0),
            pointer::Button::Secondary,
        );

        assert!(state.floating.context_menu().is_some_and(|surface| {
            surface.context_menu_target() == Some(&path(CHILD))
                && surface.anchor() == ui::floating::Anchor::Point(point::logical(10.0, 12.0))
        }));
        assert_eq!(state.focused_path(), Some(path(CHILD)));
        assert_eq!(state.focus_visibility(), ui::focus::Visibility::Visible);
        assert!(
            !outcome
                .events
                .iter()
                .any(|event| matches!(event, ui::Event::TextEditRequested { .. }))
        );
    }

    #[test]
    fn secondary_click_on_disabled_text_field_does_not_open_context_menu() {
        let mut state = text_field_state_with_field(crate::text::Field::new("Disabled").disabled());

        pointer_pressed_for_test(
            &mut state,
            point::logical(10.0, 12.0),
            pointer::Button::Secondary,
        );

        assert!(state.floating.context_menu().is_none());
        assert_eq!(state.focused_path(), None);
    }

    #[test]
    fn context_menu_key_opens_text_context_menu_for_active_session() {
        let registry = action::Registry::<()>::new();
        let window = window::Id::new(1);
        let mut state = text_field_state();
        let mut text_engine = crate::text::Engine::new();
        crate::app::text_input::sync_session(&mut state);

        let outcome = key_pressed_with_text(
            &registry,
            &mut state,
            window,
            ui::Key::ContextMenu,
            None,
            false,
            &mut text_engine,
        );

        assert!(outcome.redraw);
        assert!(state.floating.context_menu().is_some_and(|surface| {
            surface.context_menu_target() == Some(&path(CHILD))
                && surface.source() == action::Source::Keyboard
        }));
    }

    #[test]
    fn pointer_drag_on_text_field_requests_drag_edit_until_release() {
        let window = window::Id::new(1);
        let registry = action::Registry::<()>::new();
        let mut state = text_field_state();
        let mut text_engine = crate::text::Engine::new();

        pointer_pressed(
            &mut state,
            window,
            point::logical(10.0, 12.0),
            pointer::Button::Primary,
            &mut text_engine,
        );
        let moved = pointer_moved_with_text_engine(
            &mut state,
            point::logical(80.0, 12.0),
            &mut text_engine,
        );

        assert!(moved.events.iter().any(|event| matches!(
            event,
            ui::Event::TextEditRequested {
                target,
                edit: crate::text::Edit::Pointer {
                    kind: crate::text::PointerEditKind::Drag,
                    position
                }
            } if target == &path(CHILD)
                && position.index <= "hello".len()
        )));

        pointer_released(
            &registry,
            &mut state,
            window,
            point::logical(80.0, 12.0),
            pointer::Button::Primary,
        );

        assert_eq!(state.text_pointer_gesture, None);
    }

    fn selected_text_buffer(selection_end: usize) -> crate::text::Buffer {
        let mut text_engine = crate::text::Engine::new();
        let mut buffer = crate::text::Buffer::from_text("hello");
        text_engine.apply_text_edit(
            &mut buffer,
            crate::text::Edit::set_cursor(crate::text::Cursor::new(0, 0)),
        );
        text_engine.apply_text_edit(
            &mut buffer,
            crate::text::Edit::pointer(
                crate::text::PointerEditKind::Drag,
                crate::text::Cursor::new(0, selection_end),
            ),
        );
        buffer
    }

    #[test]
    fn click_inside_fully_selected_text_collapses_selection_on_release() {
        let window = window::Id::new(1);
        let registry = action::Registry::<()>::new();
        let mut text_engine = crate::text::Engine::new();
        let mut state = text_field_state_with_field(selected_text_buffer(5));

        let pressed = pointer_pressed(
            &mut state,
            window,
            point::logical(10.0, 12.0),
            pointer::Button::Primary,
            &mut text_engine,
        );
        assert!(
            !pressed
                .events
                .iter()
                .any(|event| matches!(event, ui::Event::TextEditRequested { .. }))
        );

        let released = pointer_released(
            &registry,
            &mut state,
            window,
            point::logical(10.0, 12.0),
            pointer::Button::Primary,
        );

        assert!(released.events.iter().any(|event| matches!(
            event,
            ui::Event::TextEditRequested {
                target,
                edit: crate::text::Edit::Pointer {
                    kind: crate::text::PointerEditKind::Click,
                    ..
                },
            } if target == &path(CHILD)
        )));
        assert_eq!(state.text_pointer_gesture, None);
        assert!(state.drag_drop.active_text().is_none());
    }

    #[test]
    fn click_inside_partially_selected_text_places_caret_on_release() {
        let window = window::Id::new(1);
        let registry = action::Registry::<()>::new();
        let mut text_engine = crate::text::Engine::new();
        let mut state = text_field_state_with_field(selected_text_buffer(2));

        pointer_pressed(
            &mut state,
            window,
            point::logical(10.0, 12.0),
            pointer::Button::Primary,
            &mut text_engine,
        );
        let released = pointer_released(
            &registry,
            &mut state,
            window,
            point::logical(10.0, 12.0),
            pointer::Button::Primary,
        );

        assert!(released.events.iter().any(|event| matches!(
            event,
            ui::Event::TextEditRequested {
                target,
                edit: crate::text::Edit::Pointer {
                    kind: crate::text::PointerEditKind::Click,
                    position,
                },
            } if target == &path(CHILD) && position.index <= 2
        )));
        assert_eq!(state.text_pointer_gesture, None);
        assert!(state.drag_drop.active_text().is_none());
    }

    #[test]
    fn movement_below_text_drag_threshold_still_resolves_as_click() {
        let window = window::Id::new(1);
        let registry = action::Registry::<()>::new();
        let mut text_engine = crate::text::Engine::new();
        let mut state = text_field_state_with_field(selected_text_buffer(2));

        pointer_pressed(
            &mut state,
            window,
            point::logical(10.0, 12.0),
            pointer::Button::Primary,
            &mut text_engine,
        );
        let moved = pointer_moved_with_text_engine(
            &mut state,
            point::logical(12.0, 12.0),
            &mut text_engine,
        );
        assert!(state.drag_drop.active_text().is_none());
        assert!(
            !moved
                .events
                .iter()
                .any(|event| matches!(event, ui::Event::TextEditRequested { .. }))
        );

        let released = pointer_released(
            &registry,
            &mut state,
            window,
            point::logical(12.0, 12.0),
            pointer::Button::Primary,
        );

        assert!(released.events.iter().any(|event| matches!(
            event,
            ui::Event::TextEditRequested {
                target,
                edit: crate::text::Edit::Pointer {
                    kind: crate::text::PointerEditKind::Click,
                    ..
                },
            } if target == &path(CHILD)
        )));
        assert_eq!(state.text_pointer_gesture, None);
    }

    #[test]
    fn selection_drag_pointer_move_requests_redraw() {
        let window = window::Id::new(1);
        let mut text_engine = crate::text::Engine::new();
        let mut state = text_field_state();
        state.hovered = Some(path(CHILD));

        pointer_pressed(
            &mut state,
            window,
            point::logical(4.0, 12.0),
            pointer::Button::Primary,
            &mut text_engine,
        );
        let moved = pointer_moved_with_text_engine(
            &mut state,
            point::logical(80.0, 12.0),
            &mut text_engine,
        );

        assert!(moved.redraw);
        assert!(moved.events.iter().any(|event| matches!(
            event,
            ui::Event::TextEditRequested {
                target,
                edit: crate::text::Edit::Pointer {
                    kind: crate::text::PointerEditKind::Drag,
                    ..
                },
            } if target == &path(CHILD)
        )));
    }
    #[test]
    fn pointer_drag_inside_selected_text_starts_text_drop_session() {
        let window = window::Id::new(1);
        let registry = action::Registry::<()>::new();
        let mut text_engine = crate::text::Engine::new();
        let mut state = text_field_state_with_field(selected_text_buffer(2));

        let pressed = pointer_pressed(
            &mut state,
            window,
            point::logical(10.0, 12.0),
            pointer::Button::Primary,
            &mut text_engine,
        );
        assert!(
            !pressed
                .events
                .iter()
                .any(|event| matches!(event, ui::Event::TextEditRequested { .. }))
        );

        let moved = pointer_moved_with_text_engine(
            &mut state,
            point::logical(80.0, 12.0),
            &mut text_engine,
        );

        assert!(state.drag_drop.active_text().is_some());
        assert!(state.drag_drop.text_target().is_some());
        assert!(state.text_drop_caret().is_some());
        assert_eq!(state.text_pointer_gesture, None);
        assert!(
            !moved
                .events
                .iter()
                .any(|event| matches!(event, ui::Event::TextEditRequested { .. }))
        );

        let released = pointer_released(
            &registry,
            &mut state,
            window,
            point::logical(80.0, 12.0),
            pointer::Button::Primary,
        );

        assert!(released.events.iter().any(|event| matches!(
            event,
            ui::Event::TextDropRequested {
                source_cleanup: None,
                target,
                edit: crate::text::Edit::MoveRange { range, .. },
                operation: ui::drag_drop::Operation::Move,
            } if target == &path(CHILD) && range == &(0..2)
        )));
        assert!(state.drag_drop.active_text().is_none());
        assert!(state.drag_drop.text_target().is_none());
    }

    #[test]
    fn active_text_drag_retargets_after_starting_inside_original_selection() {
        let window = window::Id::new(1);
        let registry = action::Registry::<()>::new();
        let mut text_engine = crate::text::Engine::new();
        let mut state = text_field_state_with_field(selected_text_buffer(2));

        pointer_pressed(
            &mut state,
            window,
            point::logical(10.0, 12.0),
            pointer::Button::Primary,
            &mut text_engine,
        );
        pointer_moved_with_text_engine(&mut state, point::logical(15.0, 12.0), &mut text_engine);

        assert!(state.drag_drop.active_text().is_some());
        assert_eq!(
            state.drag_drop.resolved_operation(),
            ui::drag_drop::Operation::None
        );
        assert!(state.drag_drop.text_target().is_none());
        assert_eq!(state.text_drop_caret(), None);

        pointer_moved_with_text_engine(&mut state, point::logical(80.0, 12.0), &mut text_engine);

        assert_eq!(
            state.drag_drop.resolved_operation(),
            ui::drag_drop::Operation::Move
        );
        assert!(state.drag_drop.text_target().is_some());
        assert!(state.text_drop_caret().is_some());

        let released = pointer_released(
            &registry,
            &mut state,
            window,
            point::logical(80.0, 12.0),
            pointer::Button::Primary,
        );

        assert!(released.events.iter().any(|event| matches!(
            event,
            ui::Event::TextDropRequested {
                target,
                edit: crate::text::Edit::MoveRange { range, .. },
                operation: ui::drag_drop::Operation::Move,
                ..
            } if target == &path(CHILD) && range == &(0..2)
        )));
    }

    #[test]
    fn active_text_drag_retargets_from_invalid_to_valid_cross_field_target() {
        let window = window::Id::new(1);
        let registry = action::Registry::<()>::new();
        let mut text_engine = crate::text::Engine::new();
        let mut state = two_text_field_state(
            selected_text_buffer(2),
            crate::text::Buffer::from_text("world"),
        );

        pointer_pressed(
            &mut state,
            window,
            point::logical(10.0, 12.0),
            pointer::Button::Primary,
            &mut text_engine,
        );
        pointer_moved_with_text_engine(&mut state, point::logical(80.0, 80.0), &mut text_engine);

        assert!(state.drag_drop.active_text().is_some());
        assert_eq!(
            state.drag_drop.resolved_operation(),
            ui::drag_drop::Operation::None
        );
        assert!(state.drag_drop.text_target().is_none());

        pointer_moved_with_text_engine(&mut state, point::logical(80.0, 36.0), &mut text_engine);

        assert_eq!(
            state.drag_drop.resolved_operation(),
            ui::drag_drop::Operation::Move
        );
        assert!(state.drag_drop.text_target().is_some());

        let released = pointer_released(
            &registry,
            &mut state,
            window,
            point::logical(80.0, 36.0),
            pointer::Button::Primary,
        );

        assert!(released.events.iter().any(|event| matches!(
            event,
            ui::Event::TextDropRequested {
                source_cleanup: Some((source, _)),
                target,
                operation: ui::drag_drop::Operation::Move,
                ..
            } if source == &root_path(CHILD) && target == &root_path(SECOND)
        )));
    }

    #[test]
    fn active_text_drag_requests_redraw_even_when_drop_target_is_unchanged() {
        let window = window::Id::new(1);
        let mut text_engine = crate::text::Engine::new();
        let mut state = text_field_state_with_field(selected_text_buffer(2));

        pointer_pressed(
            &mut state,
            window,
            point::logical(10.0, 12.0),
            pointer::Button::Primary,
            &mut text_engine,
        );
        pointer_moved_with_text_engine(&mut state, point::logical(80.0, 80.0), &mut text_engine);

        assert!(state.drag_drop.active_text().is_some());
        assert!(state.drag_drop.text_target().is_none());

        let moved = pointer_moved_with_text_engine(
            &mut state,
            point::logical(80.0, 80.0),
            &mut text_engine,
        );

        assert!(moved.redraw);
        assert!(state.drag_drop.active_text().is_some());
        assert!(state.drag_drop.text_target().is_none());
    }

    #[test]
    fn cross_field_text_drop_moves_by_default() {
        let window = window::Id::new(1);
        let registry = action::Registry::<()>::new();
        let mut text_engine = crate::text::Engine::new();
        let mut buffer = crate::text::Buffer::from_text("hello");
        text_engine.apply_text_edit(
            &mut buffer,
            crate::text::Edit::set_cursor(crate::text::Cursor::new(0, 0)),
        );
        text_engine.apply_text_edit(
            &mut buffer,
            crate::text::Edit::pointer(
                crate::text::PointerEditKind::Drag,
                crate::text::Cursor::new(0, 2),
            ),
        );
        let mut state = two_text_field_state(buffer, crate::text::Buffer::from_text("world"));

        pointer_pressed(
            &mut state,
            window,
            point::logical(10.0, 12.0),
            pointer::Button::Primary,
            &mut text_engine,
        );
        pointer_moved_with_text_engine(&mut state, point::logical(80.0, 36.0), &mut text_engine);
        assert_eq!(
            state.drag_drop.resolved_operation(),
            ui::drag_drop::Operation::Move
        );

        let released = pointer_released(
            &registry,
            &mut state,
            window,
            point::logical(80.0, 36.0),
            pointer::Button::Primary,
        );

        assert!(released.events.iter().any(|event| matches!(
            event,
            ui::Event::TextDropRequested {
                source_cleanup: Some((source, crate::text::Edit::ReplaceRange { range, text })),
                target,
                edit: crate::text::Edit::ReplaceRange { .. },
                operation: ui::drag_drop::Operation::Move,
            } if source == &root_path(CHILD)
                && target == &root_path(SECOND)
                && range == &(0..2)
                && text.is_empty()
        )));
    }

    #[test]
    fn cross_field_text_drop_copy_modifier_preserves_source() {
        let window = window::Id::new(1);
        let registry = action::Registry::<()>::new();
        let mut text_engine = crate::text::Engine::new();
        let mut buffer = crate::text::Buffer::from_text("hello");
        text_engine.apply_text_edit(
            &mut buffer,
            crate::text::Edit::set_cursor(crate::text::Cursor::new(0, 0)),
        );
        text_engine.apply_text_edit(
            &mut buffer,
            crate::text::Edit::pointer(
                crate::text::PointerEditKind::Drag,
                crate::text::Cursor::new(0, 2),
            ),
        );
        let mut state = two_text_field_state(buffer, crate::text::Buffer::from_text("world"));
        state.modifiers = text_drag_copy_modifier();

        pointer_pressed(
            &mut state,
            window,
            point::logical(10.0, 12.0),
            pointer::Button::Primary,
            &mut text_engine,
        );
        pointer_moved_with_text_engine(&mut state, point::logical(80.0, 36.0), &mut text_engine);
        assert_eq!(
            state.drag_drop.resolved_operation(),
            ui::drag_drop::Operation::Copy
        );

        let released = pointer_released(
            &registry,
            &mut state,
            window,
            point::logical(80.0, 36.0),
            pointer::Button::Primary,
        );

        assert!(released.events.iter().any(|event| matches!(
            event,
            ui::Event::TextDropRequested {
                source_cleanup: None,
                target,
                edit: crate::text::Edit::ReplaceRange { .. },
                operation: ui::drag_drop::Operation::Copy,
            } if target == &root_path(SECOND)
        )));
    }

    #[test]
    fn text_drop_on_read_only_target_is_rejected() {
        let window = window::Id::new(1);
        let registry = action::Registry::<()>::new();
        let mut text_engine = crate::text::Engine::new();
        let mut buffer = crate::text::Buffer::from_text("hello");
        text_engine.apply_text_edit(
            &mut buffer,
            crate::text::Edit::set_cursor(crate::text::Cursor::new(0, 0)),
        );
        text_engine.apply_text_edit(
            &mut buffer,
            crate::text::Edit::pointer(
                crate::text::PointerEditKind::Drag,
                crate::text::Cursor::new(0, 2),
            ),
        );
        let mut state = two_text_field_state(
            buffer,
            crate::text::Field::new(crate::text::Buffer::from_text("world")).read_only(),
        );

        pointer_pressed(
            &mut state,
            window,
            point::logical(10.0, 12.0),
            pointer::Button::Primary,
            &mut text_engine,
        );
        pointer_moved_with_text_engine(&mut state, point::logical(80.0, 36.0), &mut text_engine);

        assert!(state.drag_drop.active_text().is_some());
        assert_eq!(
            state.drag_drop.resolved_operation(),
            ui::drag_drop::Operation::None
        );
        assert!(state.drag_drop.text_target().is_none());
        assert_eq!(state.text_drop_caret(), None);

        let released = pointer_released(
            &registry,
            &mut state,
            window,
            point::logical(80.0, 36.0),
            pointer::Button::Primary,
        );

        assert!(
            !released
                .events
                .iter()
                .any(|event| matches!(event, ui::Event::TextDropRequested { .. }))
        );
        assert!(state.drag_drop.active_text().is_none());
    }

    #[test]
    fn escape_dismisses_open_menu() {
        let window = window::Id::new(1);
        let registry = action::Registry::<()>::new();
        let mut state = WindowState {
            open_menu: Some(FILE),
            ..WindowState::default()
        };

        let outcome = key_pressed(&registry, &mut state, window, ui::Key::Escape, false);

        assert!(outcome.redraw);
        assert_eq!(state.open_menu, None);
    }

    #[test]
    fn escape_closes_submenu_before_top_level_menu() {
        let window = window::Id::new(1);
        let registry = action::Registry::<()>::new();
        let mut state = WindowState {
            open_menu: Some(FILE),
            open_submenu: Some(PANELS),
            ..WindowState::default()
        };

        let first = key_pressed(&registry, &mut state, window, ui::Key::Escape, false);

        assert!(first.redraw);
        assert_eq!(state.open_menu, Some(FILE));
        assert_eq!(state.open_submenu, None);

        let second = key_pressed(&registry, &mut state, window, ui::Key::Escape, false);

        assert!(second.redraw);
        assert_eq!(state.open_menu, None);
    }

    #[test]
    fn pointer_movement_uses_pointer_delta_and_hover_order() {
        let mut state = state_with_composition(
            single_box(SECOND),
            HashMap::new(),
            HashMap::new(),
            HashMap::new(),
            HashMap::new(),
            HashMap::from([(path(SECOND), ui::Interactivity::CONTROL)]),
            HashMap::new(),
            Vec::new(),
        );
        state.hovered = Some(path(CHILD));
        let entered = pointer_moved(&mut state, point::logical(2.0, 3.0));

        assert_eq!(
            entered.events,
            vec![
                ui::Event::PointerLeft {
                    target: path(CHILD)
                },
                ui::Event::PointerEntered {
                    target: path(SECOND)
                },
                ui::Event::PointerMoved {
                    position: point::logical(2.0, 3.0),
                    delta: point::logical(0.0, 0.0),
                    target: Some(path(SECOND)),
                }
            ]
        );

        let outcome = pointer_moved(&mut state, point::logical(5.0, 8.0));

        assert_eq!(state.pointer.position(), Some(point::logical(5.0, 8.0)));
        assert_eq!(
            state.pointer.previous_position(),
            Some(point::logical(2.0, 3.0))
        );
        assert_eq!(state.pointer.delta(), point::logical(3.0, 5.0));
        assert_eq!(
            outcome.events,
            vec![ui::Event::PointerMoved {
                position: point::logical(5.0, 8.0),
                delta: point::logical(3.0, 5.0),
                target: Some(path(SECOND)),
            }]
        );
    }

    #[test]
    fn hovering_other_menu_title_while_menu_is_open_emits_open_menu_intent() {
        let mut state = state_with_composition(
            single_box(SECOND),
            HashMap::new(),
            HashMap::new(),
            HashMap::from([(path(SECOND), ui::Intent::OpenMenu(EDIT))]),
            HashMap::new(),
            HashMap::from([(path(SECOND), ui::Interactivity::CONTROL)]),
            HashMap::new(),
            Vec::new(),
        );
        state.open_menu = Some(FILE);

        let outcome = pointer_moved(&mut state, point::logical(2.0, 3.0));

        assert_eq!(
            outcome.intent,
            Some(IntentRequest::new(
                path(SECOND),
                ui::Intent::OpenMenu(EDIT),
                action::Source::Pointer,
            ))
        );
        assert!(outcome.redraw);
    }

    #[test]
    fn hovering_same_menu_title_does_not_toggle_open_menu() {
        let mut state = state_with_composition(
            single_box(CHILD),
            HashMap::new(),
            HashMap::new(),
            HashMap::from([(path(CHILD), ui::Intent::OpenMenu(FILE))]),
            HashMap::new(),
            HashMap::from([(path(CHILD), ui::Interactivity::CONTROL)]),
            HashMap::new(),
            Vec::new(),
        );
        state.open_menu = Some(FILE);

        let outcome = pointer_moved(&mut state, point::logical(2.0, 3.0));

        assert_eq!(outcome.intent, None);
        assert_eq!(state.open_menu, Some(FILE));
    }

    #[test]
    fn hovering_submenu_row_while_menu_is_open_emits_open_submenu_intent() {
        let row = ui::Path::new([widget::MENU_POPUP, CHILD]);
        let mut state = state_with_composition(
            path_box(row.clone()),
            HashMap::new(),
            HashMap::new(),
            HashMap::from([(row.clone(), ui::Intent::OpenSubmenu(PANELS))]),
            HashMap::new(),
            HashMap::from([(row.clone(), ui::Interactivity::CONTROL)]),
            HashMap::new(),
            Vec::new(),
        );
        state.open_menu = Some(FILE);

        let outcome = pointer_moved(&mut state, point::logical(2.0, 3.0));

        assert_eq!(
            outcome.intent,
            Some(IntentRequest::new(
                row,
                ui::Intent::OpenSubmenu(PANELS),
                action::Source::Pointer,
            ))
        );
    }

    #[test]
    fn hovering_top_menu_action_row_closes_open_submenu() {
        let row = ui::Path::new([widget::MENU_POPUP, CHILD]);
        let mut state = state_with_composition(
            path_box(row.clone()),
            HashMap::new(),
            HashMap::new(),
            HashMap::from([(row.clone(), ui::Intent::Action(CLICK))]),
            HashMap::new(),
            HashMap::from([(row.clone(), ui::Interactivity::CONTROL)]),
            HashMap::new(),
            Vec::new(),
        );
        state.open_menu = Some(FILE);
        state.open_submenu = Some(PANELS);

        let outcome = pointer_moved(&mut state, point::logical(2.0, 3.0));

        assert_eq!(
            outcome.intent,
            Some(IntentRequest::new(
                row,
                ui::Intent::CloseSubmenu,
                action::Source::Pointer,
            ))
        );
    }

    #[test]
    fn hovering_top_menu_separator_row_closes_open_submenu() {
        let row = ui::Path::new([widget::MENU_POPUP, CHILD]);
        let mut state = state_with_composition(
            path_box(row.clone()),
            HashMap::new(),
            HashMap::new(),
            HashMap::from([(row.clone(), ui::Intent::CloseSubmenu)]),
            HashMap::new(),
            HashMap::from([(row.clone(), ui::Interactivity::NONE.with_hit_test(true))]),
            HashMap::new(),
            Vec::new(),
        );
        state.open_menu = Some(FILE);
        state.open_submenu = Some(PANELS);

        let outcome = pointer_moved(&mut state, point::logical(2.0, 3.0));

        assert_eq!(
            outcome.intent,
            Some(IntentRequest::new(
                row,
                ui::Intent::CloseSubmenu,
                action::Source::Pointer,
            ))
        );
    }

    #[test]
    fn hovering_top_menu_popup_background_preserves_open_submenu() {
        let popup = ui::Path::from(widget::MENU_POPUP);
        let mut state = state_with_composition(
            path_box(popup.clone()),
            HashMap::new(),
            HashMap::new(),
            HashMap::new(),
            HashMap::new(),
            HashMap::from([(popup.clone(), ui::Interactivity::NONE.with_hit_test(true))]),
            HashMap::new(),
            Vec::new(),
        );
        state.open_menu = Some(FILE);
        state.open_submenu = Some(PANELS);

        let outcome = pointer_moved(&mut state, point::logical(2.0, 3.0));

        assert_eq!(outcome.intent, None);
        assert_eq!(state.open_submenu, Some(PANELS));
    }

    #[test]
    fn hovering_submenu_popup_preserves_open_submenu() {
        let popup = ui::Path::from(widget::MENU_SUBMENU_POPUP);
        let mut state = state_with_composition(
            path_box(popup.clone()),
            HashMap::new(),
            HashMap::new(),
            HashMap::new(),
            HashMap::new(),
            HashMap::from([(popup.clone(), ui::Interactivity::NONE.with_hit_test(true))]),
            HashMap::new(),
            Vec::new(),
        );
        state.open_menu = Some(FILE);
        state.open_submenu = Some(PANELS);

        let outcome = pointer_moved(&mut state, point::logical(2.0, 3.0));

        assert_eq!(outcome.intent, None);
        assert_eq!(state.open_submenu, Some(PANELS));
    }

    #[test]
    fn menu_title_hover_without_open_menu_does_not_emit_intent() {
        let mut state = state_with_composition(
            single_box(CHILD),
            HashMap::new(),
            HashMap::new(),
            HashMap::from([(path(CHILD), ui::Intent::OpenMenu(FILE))]),
            HashMap::new(),
            HashMap::from([(path(CHILD), ui::Interactivity::CONTROL)]),
            HashMap::new(),
            Vec::new(),
        );

        let outcome = pointer_moved(&mut state, point::logical(2.0, 3.0));

        assert_eq!(outcome.intent, None);
    }

    #[test]
    fn pointer_left_clears_pointer_and_hover() {
        let mut state = WindowState {
            hovered: Some(path(CHILD)),
            ..WindowState::default()
        };
        state.pointer.handle_event(pointer::Event::Moved {
            position: point::logical(2.0, 3.0),
        });
        state.pointer.handle_event(pointer::Event::Button {
            button: pointer::Button::Primary,
            pressed: true,
        });

        let outcome = pointer_left(&mut state);

        assert_eq!(state.pointer.position(), None);
        assert_eq!(state.pointer.delta(), point::logical(0.0, 0.0));
        assert!(!state.pointer.primary_down());
        assert_eq!(state.hovered, None);
        assert_eq!(
            outcome.events,
            vec![ui::Event::PointerLeft {
                target: path(CHILD)
            }]
        );
        assert!(outcome.redraw);
    }

    #[test]
    fn pointer_left_clears_pointer_press_state() {
        let mut state = WindowState {
            pressed: Some(path(CHILD)),
            pressed_source: Some(PressSource::Pointer),
            ..WindowState::default()
        };

        let outcome = pointer_left(&mut state);

        assert_eq!(state.pressed, None);
        assert_eq!(state.pressed_source, None);
        assert!(outcome.redraw);
    }

    #[test]
    fn tab_moves_focus_in_layout_order_and_shows_outline() {
        let window = window::Id::new(1);
        let registry = action::Registry::<()>::new();
        let mut state = state_with_composition(
            single_box(CHILD),
            HashMap::new(),
            HashMap::new(),
            HashMap::new(),
            HashMap::new(),
            HashMap::from([
                (path(CHILD), ui::Interactivity::CONTROL),
                (path(SECOND), ui::Interactivity::CONTROL),
            ]),
            HashMap::new(),
            vec![path(CHILD), path(SECOND)],
        );

        let outcome = key_pressed(&registry, &mut state, window, ui::Key::Tab, false);

        assert!(outcome.redraw);
        assert_eq!(state.focused_path(), Some(path(CHILD)));
        assert_eq!(
            state.focus.as_ref().map(|focus| focus.reason),
            Some(ui::focus::Reason::Keyboard)
        );
        assert_eq!(state.focus_visibility(), ui::focus::Visibility::Visible);
        assert_eq!(
            outcome.events,
            vec![ui::Event::KeyDown {
                key: ui::Key::Tab,
                modifiers: ui::Modifiers::default(),
                target: None,
                repeat: false,
            }]
        );

        key_pressed(&registry, &mut state, window, ui::Key::Tab, false);

        assert_eq!(state.focused_path(), Some(path(SECOND)));
    }

    #[test]
    fn shift_tab_moves_focus_backward() {
        let window = window::Id::new(1);
        let registry = action::Registry::<()>::new();
        let mut state = state_with_composition(
            single_box(CHILD),
            HashMap::new(),
            HashMap::new(),
            HashMap::new(),
            HashMap::new(),
            HashMap::from([
                (path(CHILD), ui::Interactivity::CONTROL),
                (path(SECOND), ui::Interactivity::CONTROL),
            ]),
            HashMap::new(),
            vec![path(CHILD), path(SECOND)],
        );
        state.modifiers = ui::Modifiers::new(true, false, false, false);

        key_pressed(&registry, &mut state, window, ui::Key::Tab, false);

        assert_eq!(state.focused_path(), Some(path(SECOND)));
    }

    #[test]
    fn disabled_action_bound_controls_are_skipped_by_tab_focus() {
        let window = window::Id::new(1);
        let mut registry = action::Registry::<()>::new();
        let mut state = state_with_composition(
            single_box(CHILD),
            HashMap::from([(path(CHILD), CLICK)]),
            HashMap::new(),
            HashMap::new(),
            HashMap::new(),
            HashMap::from([
                (path(CHILD), ui::Interactivity::CONTROL),
                (path(SECOND), ui::Interactivity::CONTROL),
            ]),
            HashMap::new(),
            vec![path(CHILD), path(SECOND)],
        );

        registry.register(Action::new(CLICK, "Click"));
        registry.set_state(
            CLICK,
            action::Context::path(window, path(CHILD)),
            action::State::disabled(),
        );
        key_pressed(&registry, &mut state, window, ui::Key::Tab, false);

        assert_eq!(state.focused_path(), Some(path(SECOND)));
    }

    #[test]
    fn tab_focus_is_trapped_to_open_dropdown_rows() {
        let window = window::Id::new(1);
        let registry = action::Registry::<()>::new();
        let menu_row = ui::Path::new([widget::MENU_POPUP, CHILD]);
        let mut state = state_with_composition(
            path_box(menu_row.clone()),
            HashMap::new(),
            HashMap::new(),
            HashMap::new(),
            HashMap::new(),
            HashMap::from([
                (path(SECOND), ui::Interactivity::CONTROL),
                (menu_row.clone(), ui::Interactivity::CONTROL),
            ]),
            HashMap::new(),
            vec![path(SECOND), menu_row.clone()],
        );
        state.open_menu = Some(FILE);

        key_pressed(&registry, &mut state, window, ui::Key::Tab, false);

        assert_eq!(state.focused_path(), Some(menu_row));
    }

    #[test]
    fn focusing_submenu_row_emits_open_submenu_intent() {
        let window = window::Id::new(1);
        let registry = action::Registry::<()>::new();
        let menu_row = ui::Path::new([widget::MENU_POPUP, CHILD]);
        let mut state = state_with_composition(
            path_box(menu_row.clone()),
            HashMap::new(),
            HashMap::new(),
            HashMap::from([(menu_row.clone(), ui::Intent::OpenSubmenu(PANELS))]),
            HashMap::new(),
            HashMap::from([(menu_row.clone(), ui::Interactivity::CONTROL)]),
            HashMap::new(),
            vec![menu_row.clone()],
        );
        state.open_menu = Some(FILE);

        let outcome = key_pressed(&registry, &mut state, window, ui::Key::Tab, false);

        assert_eq!(
            outcome.intent,
            Some(IntentRequest::new(
                menu_row,
                ui::Intent::OpenSubmenu(PANELS),
                action::Source::Keyboard,
            ))
        );
    }

    #[test]
    fn enter_releases_focused_action_with_keyboard_source() {
        let window = window::Id::new(1);
        let mut registry = action::Registry::<()>::new();
        let mut state = state_with_composition(
            single_box(CHILD),
            HashMap::from([(path(CHILD), CLICK)]),
            HashMap::new(),
            HashMap::new(),
            HashMap::new(),
            HashMap::from([(path(CHILD), ui::Interactivity::CONTROL)]),
            HashMap::new(),
            Vec::new(),
        );
        state.focus = FocusState::focused(Focus::new(
            path(CHILD),
            ui::focus::Reason::Keyboard,
            ui::focus::Visibility::Visible,
        ));

        registry.register(Action::new(CLICK, "Click"));
        let pressed = key_pressed(&registry, &mut state, window, ui::Key::Enter, false);

        assert!(pressed.redraw);
        assert_eq!(state.pressed, Some(path(CHILD)));
        assert_eq!(state.pressed_source, Some(PressSource::Keyboard));

        let released = key_released(&registry, &mut state, window, ui::Key::Enter);

        assert!(released.redraw);
        assert_eq!(state.pressed, None);
        assert_eq!(state.pressed_source, None);
        assert_eq!(
            released.request,
            Some(
                action::Request::new(
                    CLICK,
                    action::Source::Keyboard,
                    action::Context::path(window, path(CHILD))
                )
                .with_origin(path(CHILD))
            )
        );
    }

    #[test]
    fn shortcut_press_emits_shortcut_request_for_command_subject() {
        let window = window::Id::new(1);
        let mut registry = action::Registry::<()>::new();
        let mut state = state_with_composition(
            single_box(SECOND),
            HashMap::new(),
            HashMap::new(),
            HashMap::new(),
            HashMap::from([(path(SECOND), vec![action::SELECT_ALL])]),
            HashMap::new(),
            HashMap::new(),
            Vec::new(),
        );
        state.modifiers = ui::Modifiers::new(false, true, false, false);
        state.command_subject = Some(action::Scope::Path(path(SECOND)));

        registry.register(
            Action::new(action::SELECT_ALL, "Select All")
                .with_shortcut(action::Shortcut::control('a')),
        );

        let outcome = key_pressed(
            &registry,
            &mut state,
            window,
            ui::Key::Character('a'),
            false,
        );

        assert_eq!(
            outcome.request,
            Some(action::Request::new(
                action::SELECT_ALL,
                action::Source::Shortcut,
                action::Context::path(window, path(SECOND))
            ))
        );
    }

    #[test]
    fn copy_shortcut_targets_focused_text_field_command_subject() {
        let window = window::Id::new(1);
        let mut registry = action::Registry::<()>::new();
        let mut state = text_field_state();
        state.modifiers = ui::Modifiers::new(false, true, false, false);

        registry.register(
            Action::new(action::COPY, "Copy").with_shortcut(action::Shortcut::control('c')),
        );

        let outcome = key_pressed(
            &registry,
            &mut state,
            window,
            ui::Key::Character('c'),
            false,
        );

        assert_eq!(
            outcome.request,
            Some(action::Request::new(
                action::COPY,
                action::Source::Shortcut,
                action::Context::path(window, path(CHILD))
            ))
        );
        assert!(!outcome.events.iter().any(|event| {
            matches!(
                event,
                ui::Event::TextEditRequested {
                    edit: crate::text::Edit::Insert(_),
                    ..
                }
            )
        }));
    }

    #[test]
    fn copy_shortcut_targeting_text_field_without_selection_is_not_executable() {
        let window = window::Id::new(1);
        let mut registry = action::Registry::<()>::new();
        registry.register(
            Action::new(action::COPY, "Copy").with_shortcut(action::Shortcut::control('c')),
        );
        let mut state =
            text_field_state_with_registry(crate::text::Buffer::from_text("hello"), &mut registry);
        state.modifiers = ui::Modifiers::new(false, true, false, false);

        let request = key_pressed(
            &registry,
            &mut state,
            window,
            ui::Key::Character('c'),
            false,
        )
        .request
        .expect("copy shortcut should create a request for the text field");

        assert_eq!(
            request.target(),
            &action::Context::path(window, path(CHILD))
        );
        assert!(!registry.can_execute(&request));
    }

    #[test]
    fn copy_shortcut_targeting_selected_text_field_remains_executable() {
        let window = window::Id::new(1);
        let mut registry = action::Registry::<()>::new();
        let mut engine = crate::text::Engine::new();
        let mut buffer = crate::text::Buffer::from_text("hello");
        registry.register(
            Action::new(action::COPY, "Copy").with_shortcut(action::Shortcut::control('c')),
        );
        engine.apply_text_edit(&mut buffer, crate::text::Edit::SelectAll);
        let mut state = text_field_state_with_registry(buffer, &mut registry);
        state.modifiers = ui::Modifiers::new(false, true, false, false);

        let request = key_pressed(
            &registry,
            &mut state,
            window,
            ui::Key::Character('c'),
            false,
        )
        .request
        .expect("copy shortcut should create a request for the text field");

        assert_eq!(
            request.target(),
            &action::Context::path(window, path(CHILD))
        );
        assert!(registry.can_execute(&request));
    }

    #[test]
    fn undo_shortcut_targeting_text_field_without_history_is_not_executable() {
        let window = window::Id::new(1);
        let mut registry = action::Registry::<()>::new();
        registry.register(
            Action::new(action::UNDO, "Undo").with_shortcut(action::Shortcut::control('z')),
        );
        let mut state =
            text_field_state_with_registry(crate::text::Buffer::from_text("hello"), &mut registry);
        state.modifiers = ui::Modifiers::new(false, true, false, false);

        let request = key_pressed(
            &registry,
            &mut state,
            window,
            ui::Key::Character('z'),
            false,
        )
        .request
        .expect("undo shortcut should create a request for the text field");

        assert_eq!(
            request.target(),
            &action::Context::path(window, path(CHILD))
        );
        assert!(!registry.can_execute(&request));
    }

    #[test]
    fn undo_shortcut_targeting_text_field_with_history_is_executable() {
        let window = window::Id::new(1);
        let mut registry = action::Registry::<()>::new();
        let mut engine = crate::text::Engine::new();
        let mut buffer = crate::text::Buffer::from_text("hello");

        registry.register(
            Action::new(action::UNDO, "Undo").with_shortcut(action::Shortcut::control('z')),
        );
        let result =
            engine.apply_text_edit_with_result(&mut buffer, crate::text::Edit::insert("!"));
        let mut state = text_field_state_with_registry(buffer, &mut registry);
        state.record_text_field_history(
            &path(CHILD),
            result.change.expect("insert should change text"),
            crate::text::HistoryKind::Typing("!".to_owned()),
            Instant::now(),
        );
        crate::app::text_input::publish_action_states(&state, &mut registry, window);
        state.modifiers = ui::Modifiers::new(false, true, false, false);

        let request = key_pressed(
            &registry,
            &mut state,
            window,
            ui::Key::Character('z'),
            false,
        )
        .request
        .expect("undo shortcut should create a request for the text field");

        assert_eq!(
            request.target(),
            &action::Context::path(window, path(CHILD))
        );
        assert!(registry.can_execute(&request));
    }

    #[test]
    fn redo_shortcuts_target_text_field_redo_history() {
        let window = window::Id::new(1);
        let mut registry = action::Registry::<()>::new();
        let mut engine = crate::text::Engine::new();
        let mut buffer = crate::text::Buffer::from_text("hello");

        registry.register(
            Action::new(action::REDO, "Redo")
                .with_shortcut(action::Shortcut::control_shift('z'))
                .with_shortcut(action::Shortcut::control('y')),
        );
        let result =
            engine.apply_text_edit_with_result(&mut buffer, crate::text::Edit::insert("!"));
        let mut state = text_field_state_with_registry(buffer.clone(), &mut registry);
        state.record_text_field_history(
            &path(CHILD),
            result.change.expect("insert should change text"),
            crate::text::HistoryKind::Typing("!".to_owned()),
            Instant::now(),
        );
        state.apply_text_history_command(&path(CHILD), &mut buffer, crate::text::Command::Undo);
        crate::app::text_input::publish_action_states(&state, &mut registry, window);

        state.modifiers = ui::Modifiers::new(true, true, false, false);
        let shift_z = key_pressed(
            &registry,
            &mut state,
            window,
            ui::Key::Character('z'),
            false,
        )
        .request
        .expect("redo shortcut should create a request");
        assert!(registry.can_execute(&shift_z));

        state.modifiers = ui::Modifiers::new(false, true, false, false);
        let control_y = key_pressed(
            &registry,
            &mut state,
            window,
            ui::Key::Character('y'),
            false,
        )
        .request
        .expect("redo shortcut should create a request");
        assert!(registry.can_execute(&control_y));
    }

    #[test]
    fn select_all_shortcut_targeting_fully_selected_text_field_is_not_executable() {
        let window = window::Id::new(1);
        let mut registry = action::Registry::<()>::new();
        let mut engine = crate::text::Engine::new();
        let mut buffer = crate::text::Buffer::from_text("hello");

        registry.register(
            Action::new(action::SELECT_ALL, "Select All")
                .with_shortcut(action::Shortcut::control('a')),
        );
        engine.apply_text_edit(&mut buffer, crate::text::Edit::SelectAll);
        let mut state = text_field_state_with_registry(buffer, &mut registry);
        state.modifiers = ui::Modifiers::new(false, true, false, false);

        let request = key_pressed(
            &registry,
            &mut state,
            window,
            ui::Key::Character('a'),
            false,
        )
        .request
        .expect("select all shortcut should create a request for the text field");

        assert_eq!(
            request.target(),
            &action::Context::path(window, path(CHILD))
        );
        assert!(!registry.can_execute(&request));
    }

    #[test]
    fn shortcut_and_command_button_use_same_automatic_subject() {
        let window = window::Id::new(1);
        let mut registry = action::Registry::<()>::new();
        let mut state = state_with_composition(
            single_box(CHILD),
            HashMap::from([(path(CHILD), action::SELECT_ALL)]),
            HashMap::from([(path(CHILD), ui::CommandSubject::Current)]),
            HashMap::new(),
            HashMap::from([(path(SECOND), vec![action::SELECT_ALL])]),
            HashMap::new(),
            HashMap::new(),
            Vec::new(),
        );
        state.focus = FocusState::focused(Focus::new(
            path(SECOND),
            ui::focus::Reason::Keyboard,
            ui::focus::Visibility::Visible,
        ));
        state.modifiers = ui::Modifiers::new(false, true, false, false);

        registry.register(
            Action::new(action::SELECT_ALL, "Select All")
                .with_shortcut(action::Shortcut::control('a')),
        );

        let shortcut = key_pressed(
            &registry,
            &mut state,
            window,
            ui::Key::Character('a'),
            false,
        )
        .request
        .expect("shortcut should request the action");
        let button = action_request(&state, window, path(CHILD), action::Source::Pointer)
            .expect("command button should request the action");

        assert_eq!(shortcut.target(), button.target());
        assert_eq!(
            shortcut.target(),
            &action::Context::path(window, path(SECOND))
        );
    }

    #[test]
    fn repeated_shortcut_press_does_not_emit_request() {
        let window = window::Id::new(1);
        let mut registry = action::Registry::<()>::new();
        let mut state = state_with_composition(
            single_box(SECOND),
            HashMap::new(),
            HashMap::new(),
            HashMap::new(),
            HashMap::from([(path(SECOND), vec![action::SELECT_ALL])]),
            HashMap::new(),
            HashMap::new(),
            Vec::new(),
        );
        state.modifiers = ui::Modifiers::new(false, true, false, false);
        state.command_subject = Some(action::Scope::Path(path(SECOND)));

        registry.register(
            Action::new(action::SELECT_ALL, "Select All")
                .with_shortcut(action::Shortcut::control('a')),
        );

        let outcome = key_pressed(&registry, &mut state, window, ui::Key::Character('a'), true);

        assert_eq!(outcome.request, None);
    }

    #[test]
    fn command_subject_control_uses_stored_command_subject() {
        let window = window::Id::new(1);
        let mut registry = action::Registry::<()>::new();
        let mut state = state_with_composition(
            single_box(CHILD),
            HashMap::from([(path(CHILD), CLICK)]),
            HashMap::from([(path(CHILD), ui::CommandSubject::Current)]),
            HashMap::new(),
            HashMap::new(),
            HashMap::from([(path(CHILD), ui::Interactivity::CONTROL)]),
            HashMap::new(),
            Vec::new(),
        );
        state.pressed = Some(path(CHILD));
        state.pressed_source = Some(PressSource::Pointer);
        state.command_subject = Some(action::Scope::Path(path(SECOND)));
        registry.register(Action::new(CLICK, "Click"));

        let outcome = pointer_released(
            &registry,
            &mut state,
            window,
            point::logical(1.0, 1.0),
            pointer::Button::Primary,
        );

        assert_eq!(
            outcome.request,
            Some(
                action::Request::new(
                    CLICK,
                    action::Source::Pointer,
                    action::Context::path(window, path(SECOND))
                )
                .with_origin(path(CHILD))
            )
        );
    }

    #[test]
    fn window_target_control_uses_window_context() {
        let window = window::Id::new(1);
        let mut registry = action::Registry::<()>::new();
        let mut state = state_with_composition(
            single_box(CHILD),
            HashMap::from([(path(CHILD), CLICK)]),
            HashMap::from([(path(CHILD), ui::CommandSubject::Window)]),
            HashMap::new(),
            HashMap::new(),
            HashMap::from([(path(CHILD), ui::Interactivity::CONTROL)]),
            HashMap::new(),
            Vec::new(),
        );
        state.focus = FocusState::focused(Focus::new(
            path(CHILD),
            ui::focus::Reason::Keyboard,
            ui::focus::Visibility::Visible,
        ));
        state.command_subject = Some(action::Scope::Path(path(SECOND)));

        registry.register(Action::new(CLICK, "Click"));
        key_pressed(&registry, &mut state, window, ui::Key::Space, false);
        let outcome = key_released(&registry, &mut state, window, ui::Key::Space);

        assert_eq!(
            outcome.request,
            Some(
                action::Request::new(
                    CLICK,
                    action::Source::Keyboard,
                    action::Context::window(window)
                )
                .with_origin(path(CHILD))
            )
        );
    }
}
