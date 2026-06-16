use winit::{
    event::MouseButton,
    keyboard::{Key as WinitKey, ModifiersState, NamedKey},
};

use crate::app::state::{PressSource, WindowState, action_request, intent};
use crate::geometry::point;
use crate::{action, pointer, ui, window};

#[derive(Debug, Default)]
pub struct Outcome {
    pub events: Vec<ui::Event>,
    pub request: Option<action::Request>,
    pub intent: Option<(ui::Path, ui::Intent)>,
    pub redraw: bool,
}

pub fn pointer_moved(state: &mut WindowState, position: point::Logical) -> Outcome {
    state
        .pointer
        .handle_event(pointer::Event::Moved { position });
    let delta = state.pointer.delta();

    if let Some((target, offset)) = state.pointer_capture_offset(position) {
        let mut events = vec![ui::Event::PointerMoved {
            position,
            delta,
            target: Some(target.clone()),
        }];
        if state
            .scroll_metrics(&target)
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

    Outcome {
        events,
        request: None,
        intent,
        redraw,
    }
}

pub fn pointer_pressed(
    state: &mut WindowState,
    position: point::Logical,
    button: pointer::Button,
) -> Outcome {
    state.pointer.handle_event(pointer::Event::Button {
        button,
        pressed: true,
    });

    if button == pointer::Button::Primary
        && let Some(hit) = state.widget_hit(position)
    {
        state.dismiss_menu_for_target(Some(hit.target()));
        let mut events = vec![ui::Event::PointerDown {
            position,
            delta: state.pointer.delta(),
            target: Some(hit.target().clone()),
            button,
        }];

        if state.start_pointer_capture(&hit, button, position) {
            return Outcome {
                events,
                request: None,
                intent: None,
                redraw: true,
            };
        }

        if let Some(part) = hit.part().scroll()
            && let Some(metrics) = state.scroll_metrics(hit.target())
            && let Some(offset) = metrics.page_offset(part, position)
            && offset != metrics.offset()
        {
            state.pressed = Some(hit.target().clone());
            state.pressed_source = Some(PressSource::Pointer);
            events.push(ui::Event::ScrollRequested {
                target: hit.target().clone(),
                offset,
            });
        }

        return Outcome {
            events,
            request: None,
            intent: None,
            redraw: true,
        };
    }

    let target = state.hit_test(position);
    state.dismiss_menu_for_target(target.as_ref());
    let event = state.pointer_down(position, state.pointer.delta(), target, button);

    Outcome {
        events: vec![event],
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
    let intent = invoke_target
        .as_ref()
        .and_then(|target| intent(state, target.clone()));
    let request = invoke_target.and_then(|target| {
        activation_request(registry, state, window, target, action::Source::Pointer)
    });
    let closed_menu = request
        .as_ref()
        .and_then(action::Request::origin)
        .is_some_and(|origin| state.is_menu_path(origin))
        && state.close_menu();

    Outcome {
        events: vec![event],
        request,
        intent,
        redraw: true || closed_menu,
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
        redraw: !events.is_empty() || cleared_pressed || cleared_capture,
        events,
        request: None,
        intent: None,
    }
}

pub fn scroll_wheel(
    state: &WindowState,
    position: point::Logical,
    delta: point::Logical,
) -> Outcome {
    let target = state.scroll_target(position);
    let mut events = vec![ui::Event::ScrollWheel {
        position,
        delta,
        target: target.clone(),
    }];

    if let Some(target) = target
        && let Some(metrics) = state.scroll_metrics(&target)
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
        redraw: false,
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
        WinitKey::Character(value) => {
            let mut chars = value.chars();
            let Some(character) = chars.next() else {
                return ui::Key::Other;
            };

            if chars.next().is_none() {
                ui::Key::Character(character.to_ascii_lowercase())
            } else {
                ui::Key::Other
            }
        }
        _ => ui::Key::Other,
    }
}

pub fn key_pressed<T>(
    registry: &action::Registry<T>,
    state: &mut WindowState,
    window: window::Id,
    key: ui::Key,
    repeat: bool,
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

    match key {
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
        events: vec![event],
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
            intent = target
                .as_ref()
                .and_then(|target| super::state::intent(state, target.clone()));
            request = target.and_then(|target| {
                activation_request(registry, state, window, target, action::Source::Keyboard)
            });
            redraw |= request
                .as_ref()
                .and_then(action::Request::origin)
                .is_some_and(|origin| state.is_menu_path(origin))
                && state.close_menu();
        }
    }

    Outcome {
        events: vec![event],
        request,
        intent,
        redraw,
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

    action_request(state, window, target, source)
        .filter(|request| registry.can_invoke(request.action(), request.target().clone()))
}

fn hover_menu_intent(state: &WindowState, target: &ui::Path) -> Option<(ui::Path, ui::Intent)> {
    menu_navigation_intent(state, target)
}

fn focus_menu_intent(state: &WindowState, target: &ui::Path) -> Option<(ui::Path, ui::Intent)> {
    menu_navigation_intent(state, target)
}

fn menu_navigation_intent(
    state: &WindowState,
    target: &ui::Path,
) -> Option<(ui::Path, ui::Intent)> {
    let intent = state.intent(target);

    match intent {
        Some(ui::Intent::OpenMenu(menu)) if state.open_menu.is_some_and(|open| open != menu) => {
            return Some((target.clone(), ui::Intent::OpenMenu(menu)));
        }
        Some(ui::Intent::OpenSubmenu(menu))
            if state.open_menu.is_some() && state.open_submenu != Some(menu) =>
        {
            return Some((target.clone(), ui::Intent::OpenSubmenu(menu)));
        }
        Some(ui::Intent::CloseSubmenu)
            if state.open_submenu.is_some() && state.is_top_menu_popup_path(target) =>
        {
            return Some((target.clone(), ui::Intent::CloseSubmenu));
        }
        Some(ui::Intent::Action(_))
            if state.open_submenu.is_some() && state.is_top_menu_popup_path(target) =>
        {
            return Some((target.clone(), ui::Intent::CloseSubmenu));
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

    if state.open_menu.is_some() {
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
        .filter(|request| registry.can_invoke(request.action(), request.target().clone()))
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

    use crate::Action;
    use crate::widget::{self, menu};

    use super::*;

    const CHILD: ui::Id = ui::Id::new("child");
    const SECOND: ui::Id = ui::Id::new("second");
    const CLICK: action::Id = action::Id::new("click");
    const FILE: menu::Id = menu::Id::new("file");
    const EDIT: menu::Id = menu::Id::new("edit");
    const PANELS: menu::Id = menu::Id::new("panels");

    fn path(id: ui::Id) -> ui::Path {
        ui::Path::from(id)
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
        let mut measurer = crate::text::Measurer::new();
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
        let mut measurer = crate::text::Measurer::new();
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
        action_targets: HashMap<ui::Path, ui::ActionTarget>,
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
            action_targets,
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
        action_targets: HashMap<ui::Path, ui::ActionTarget>,
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
                action_targets,
                intents,
                responders,
                interactivity,
                widget_metrics,
                focus_order,
            )),
            ..WindowState::default()
        }
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

        let outcome = pointer_pressed(
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
    fn wheel_event_routes_to_scrollable_under_pointer() {
        let state = scroll_state(point::logical(0.0, 12.0));

        let outcome = scroll_wheel(&state, point::logical(1.0, 1.0), point::logical(0.0, -20.0));

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
        let state = scroll_state(point::logical(0.0, 12.0));

        let outcome = scroll_wheel(
            &state,
            point::logical(50.0, 50.0),
            point::logical(0.0, -20.0),
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

        let pressed = pointer_pressed(
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

        let outcome = pointer_pressed(
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

        let thumb = pointer_pressed(
            &mut state,
            point::logical(35.0, 5.0),
            pointer::Button::Primary,
        );

        assert!(state.pointer_capture.is_none());
        assert_eq!(requested_offset(&thumb.events), None);

        let wheel = scroll_wheel(&state, point::logical(1.0, 1.0), point::logical(0.0, -20.0));
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
            Some((path(CHILD), ui::Intent::OpenMenu(FILE)))
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
    fn character_keys_are_normalized() {
        assert_eq!(
            key(&WinitKey::Character("A".into())),
            ui::Key::Character('a')
        );
        assert_eq!(key(&WinitKey::Character("ab".into())), ui::Key::Other);
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
            Some((path(SECOND), ui::Intent::OpenMenu(EDIT)))
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

        assert_eq!(outcome.intent, Some((row, ui::Intent::OpenSubmenu(PANELS))));
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

        assert_eq!(outcome.intent, Some((row, ui::Intent::CloseSubmenu)));
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

        assert_eq!(outcome.intent, Some((row, ui::Intent::CloseSubmenu)));
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
            Some((menu_row, ui::Intent::OpenSubmenu(PANELS)))
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
        state.focus = Some(crate::app::state::Focus::new(
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
    fn shortcut_press_emits_shortcut_request_for_command_target() {
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
    fn shortcut_and_command_button_use_same_automatic_subject() {
        let window = window::Id::new(1);
        let mut registry = action::Registry::<()>::new();
        let mut state = state_with_composition(
            single_box(CHILD),
            HashMap::from([(path(CHILD), action::SELECT_ALL)]),
            HashMap::from([(path(CHILD), ui::ActionTarget::Command)]),
            HashMap::new(),
            HashMap::from([(path(SECOND), vec![action::SELECT_ALL])]),
            HashMap::new(),
            HashMap::new(),
            Vec::new(),
        );
        state.focus = Some(crate::app::state::Focus::new(
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
    fn command_target_control_uses_stored_command_target() {
        let window = window::Id::new(1);
        let mut registry = action::Registry::<()>::new();
        let mut state = state_with_composition(
            single_box(CHILD),
            HashMap::from([(path(CHILD), CLICK)]),
            HashMap::from([(path(CHILD), ui::ActionTarget::Command)]),
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
            HashMap::from([(path(CHILD), ui::ActionTarget::Window)]),
            HashMap::new(),
            HashMap::new(),
            HashMap::from([(path(CHILD), ui::Interactivity::CONTROL)]),
            HashMap::new(),
            Vec::new(),
        );
        state.focus = Some(crate::app::state::Focus::new(
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
