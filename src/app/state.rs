use std::collections::HashMap;

use crate::geometry::point;
use crate::{action, layout, menu, pointer, ui, window};

use super::command;

#[derive(Debug, Default)]
pub struct WindowState {
    pub hovered: Option<ui::Path>,
    pub focus: Option<Focus>,
    pub pressed: Option<ui::Path>,
    pub pressed_source: Option<PressSource>,
    pub modifiers: ui::Modifiers,
    pub focus_order: Vec<ui::Path>,
    pub command_subject: Option<action::Scope>,
    pub pointer: pointer::Pointer,
    pub layout: Option<layout::Box>,
    pub actions: HashMap<ui::Path, action::Id>,
    pub action_targets: HashMap<ui::Path, ui::ActionTarget>,
    pub intents: HashMap<ui::Path, ui::Intent>,
    pub menus: HashMap<menu::Id, menu::Menu>,
    pub open_menu: Option<menu::Id>,
    pub open_submenu: Option<menu::Id>,
    pub responders: HashMap<ui::Path, Vec<action::Id>>,
    pub command_scopes: Vec<ui::Path>,
    pub command_scope_captures: HashMap<ui::Path, action::Context>,
    pub interactivity: HashMap<ui::Path, ui::Interactivity>,
    pub scrollables: HashMap<ui::Path, point::Logical>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Focus {
    pub path: ui::Path,
    pub reason: ui::focus::Reason,
    pub visibility: ui::focus::Visibility,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PressSource {
    Pointer,
    Keyboard,
}

impl WindowState {
    pub fn hit_test(&self, position: point::Logical) -> Option<ui::Path> {
        self.layout.as_ref().and_then(|layout| {
            layout.hit_test_where(position, |path| {
                self.interactivity
                    .get(path)
                    .is_some_and(|interactivity| interactivity.hit_test())
            })
        })
    }

    pub fn scroll_target(&self, position: point::Logical) -> Option<ui::Path> {
        self.layout.as_ref().and_then(|layout| {
            layout.hit_test_where(position, |path| self.scrollables.contains_key(path))
        })
    }

    pub fn is_focusable(&self, target: &ui::Path) -> bool {
        self.interactivity
            .get(target)
            .is_some_and(|interactivity| interactivity.focusable())
    }

    pub fn is_actionable(&self, target: &ui::Path) -> bool {
        self.interactivity
            .get(target)
            .is_some_and(|interactivity| interactivity.actionable())
    }

    pub fn action_target(&self, target: &ui::Path) -> ui::ActionTarget {
        self.action_targets.get(target).copied().unwrap_or_default()
    }

    pub fn intent(&self, target: &ui::Path) -> Option<ui::Intent> {
        self.intents.get(target).copied()
    }

    pub fn has_responder(&self, target: &ui::Path) -> bool {
        self.responders
            .get(target)
            .is_some_and(|actions| !actions.is_empty())
    }

    pub fn set_hovered(&mut self, target: Option<ui::Path>) -> Vec<ui::Event> {
        if self.hovered == target {
            return Vec::new();
        }

        let old = self.hovered.clone();
        self.hovered = target.clone();
        let mut events = Vec::new();

        if let Some(target) = old {
            events.push(ui::Event::PointerLeft { target });
        }

        if let Some(target) = target {
            events.push(ui::Event::PointerEntered { target });
        }

        events
    }

    pub fn pointer_down(
        &mut self,
        position: point::Logical,
        delta: point::Logical,
        target: Option<ui::Path>,
        button: pointer::Button,
    ) -> ui::Event {
        self.focus = target
            .clone()
            .filter(|target| self.is_focusable(target))
            .map(|path| {
                Focus::new(
                    path,
                    ui::focus::Reason::Pointer,
                    ui::focus::Visibility::Hidden,
                )
            });
        if let Some(target) = target.as_ref() {
            command::set_subject_from_path(self, target);
        }
        self.pressed = target.clone();
        self.pressed_source = target.as_ref().map(|_| PressSource::Pointer);

        ui::Event::PointerDown {
            position,
            delta,
            target,
            button,
        }
    }

    pub fn pointer_up(
        &mut self,
        position: point::Logical,
        delta: point::Logical,
        target: Option<ui::Path>,
        button: pointer::Button,
    ) -> (ui::Event, Option<ui::Path>) {
        let pressed = if self.pressed_source == Some(PressSource::Pointer) {
            self.pressed.take()
        } else {
            None
        };
        if self.pressed_source == Some(PressSource::Pointer) {
            self.pressed_source = None;
        }
        let routed_target = pressed.clone().or(target);
        let invoke = if button == pointer::Button::Primary {
            pressed
        } else {
            None
        }
        .filter(|target| self.is_actionable(target));

        (
            ui::Event::PointerUp {
                position,
                delta,
                target: routed_target,
                button,
            },
            invoke,
        )
    }

    pub fn toggle_menu<T>(
        &mut self,
        id: menu::Id,
        registry: &action::Registry<T>,
        window: window::Id,
    ) -> bool {
        if self.open_menu == Some(id) {
            return self.close_menu();
        }

        let Some(menu) = self.menus.get(&id) else {
            return false;
        };

        if !self.menu_can_open(menu, registry, window) {
            return false;
        }

        self.open_menu = Some(id);
        self.open_submenu = None;
        true
    }

    pub fn open_submenu<T>(
        &mut self,
        id: menu::Id,
        registry: &action::Registry<T>,
        window: window::Id,
    ) -> bool {
        if self.open_menu.is_none() || self.open_submenu == Some(id) {
            return false;
        }

        let Some(menu) = self.menus.get(&id) else {
            return false;
        };

        if !self.menu_can_open(menu, registry, window) {
            return false;
        }

        self.open_submenu = Some(id);
        true
    }

    pub fn close_submenu(&mut self) -> bool {
        let changed = self.open_submenu.is_some();
        self.open_submenu = None;
        changed
    }

    pub fn close_menu(&mut self) -> bool {
        let changed = self.open_menu.is_some() || self.open_submenu.is_some();
        self.open_menu = None;
        self.open_submenu = None;
        changed
    }

    pub fn dismiss_menu_for_target(&mut self, target: Option<&ui::Path>) -> bool {
        if self.open_menu.is_none() {
            return false;
        }

        if target.is_some_and(|target| self.is_menu_path(target)) {
            return false;
        }

        self.close_menu()
    }

    pub fn is_menu_path(&self, path: &ui::Path) -> bool {
        self.is_dropdown_path(path)
            || path.ids().iter().enumerate().any(|(index, _)| {
                let candidate = ui::Path::new(path.ids()[..=index].to_vec());
                matches!(
                    self.intent(&candidate),
                    Some(ui::Intent::OpenMenu(_) | ui::Intent::OpenSubmenu(_))
                )
            })
    }

    pub fn is_dropdown_path(&self, path: &ui::Path) -> bool {
        path.ids().iter().any(|id| *id == ui::widget::MENU_POPUP)
            || self.is_submenu_popup_path(path)
    }

    pub fn is_top_menu_popup_path(&self, path: &ui::Path) -> bool {
        path.ids().iter().any(|id| *id == ui::widget::MENU_POPUP)
            && !self.is_submenu_popup_path(path)
    }

    pub fn is_submenu_popup_path(&self, path: &ui::Path) -> bool {
        path.ids()
            .iter()
            .any(|id| *id == ui::widget::MENU_SUBMENU_POPUP)
    }

    pub fn focused_path(&self) -> Option<ui::Path> {
        self.focus.as_ref().map(|focus| focus.path.clone())
    }

    pub fn focus_visibility(&self) -> ui::focus::Visibility {
        self.focus
            .as_ref()
            .map(Focus::visibility)
            .unwrap_or(ui::focus::Visibility::Hidden)
    }

    pub fn set_focus(
        &mut self,
        path: ui::Path,
        reason: ui::focus::Reason,
        visibility: ui::focus::Visibility,
    ) -> bool {
        if !self.is_focusable(&path) {
            return self.clear_focus();
        }

        let focus = Focus::new(path.clone(), reason, visibility);

        let subject_changed = command::set_subject_from_path(self, &path);
        if self.focus.as_ref() == Some(&focus) {
            return subject_changed;
        }

        self.focus = Some(focus);
        true
    }

    pub fn clear_focus(&mut self) -> bool {
        let changed = self.focus.is_some();
        self.focus = None;
        changed
    }

    pub fn clear_stale_focus(&mut self) -> bool {
        let Some(path) = self.focused_path() else {
            return false;
        };

        if self.is_focusable(&path) {
            return false;
        }

        self.clear_focus()
    }

    pub fn command_context(&self, window: window::Id) -> action::Context {
        command::context(self, window)
    }

    pub fn action_context_for_path(&self, window: window::Id, path: &ui::Path) -> action::Context {
        command::context_for_path(self, window, path)
    }

    pub fn set_command_target(&mut self, context: action::Context) -> bool {
        command::set_subject(self, context)
    }

    pub fn clear_command_target(&mut self) -> bool {
        command::clear_subject(self)
    }

    pub fn clear_stale_command_target(&mut self) -> bool {
        command::clear_stale_subject(self)
    }

    pub fn update_command_scope_captures(&mut self, window: window::Id) {
        command::update_scope_captures(self, window);
    }

    pub fn resolve_request(&self, request: action::Request) -> action::Request {
        command::resolve_request(self, request)
    }
}

impl Focus {
    pub fn new(
        path: ui::Path,
        reason: ui::focus::Reason,
        visibility: ui::focus::Visibility,
    ) -> Self {
        Self {
            path,
            reason,
            visibility,
        }
    }

    pub fn visibility(&self) -> ui::focus::Visibility {
        self.visibility
    }
}

pub fn action_request(
    state: &WindowState,
    window: window::Id,
    origin: ui::Path,
    source: action::Source,
) -> Option<action::Request> {
    let action = match state.intent(&origin) {
        Some(ui::Intent::Action(action)) => action,
        Some(ui::Intent::OpenMenu(_) | ui::Intent::OpenSubmenu(_) | ui::Intent::CloseSubmenu) => {
            return None;
        }
        None => *state.actions.get(&origin)?,
    };
    let context = state.action_context_for_path(window, &origin);

    Some(action::Request::new(action, source, context).with_origin(origin))
}

pub fn intent(state: &WindowState, origin: ui::Path) -> Option<(ui::Path, ui::Intent)> {
    state.intent(&origin).map(|intent| (origin, intent))
}

impl WindowState {
    fn menu_can_open<T>(
        &self,
        menu: &menu::Menu,
        registry: &action::Registry<T>,
        window: window::Id,
    ) -> bool {
        menu.actions().any(|action| {
            let request = action::Request::new(
                action,
                action::Source::Pointer,
                self.command_context(window),
            );
            let request = self.resolve_request(request);

            registry.can_invoke(request.action(), request.target().clone())
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Action;
    use crate::geometry::{Rect, area};
    use crate::menu;

    const ROOT: ui::Id = ui::Id::new("root");
    const CHILD: ui::Id = ui::Id::new("child");
    const OUTSIDE: ui::Id = ui::Id::new("outside");
    const CLICK: action::Id = action::Id::new("click");
    const FILE: menu::Id = menu::Id::new("file");
    const PANELS: menu::Id = menu::Id::new("panels");

    fn path(id: ui::Id) -> ui::Path {
        ui::Path::from(id)
    }

    #[test]
    fn hover_changes_emit_leave_then_enter() {
        let mut state = WindowState {
            hovered: Some(path(ROOT)),
            ..WindowState::default()
        };

        let events = state.set_hovered(Some(path(CHILD)));

        assert_eq!(
            events,
            vec![
                ui::Event::PointerLeft { target: path(ROOT) },
                ui::Event::PointerEntered {
                    target: path(CHILD)
                }
            ]
        );
    }

    #[test]
    fn pointer_down_updates_focused_element() {
        let mut state = WindowState {
            interactivity: HashMap::from([(path(CHILD), ui::Interactivity::CONTROL)]),
            ..WindowState::default()
        };

        let event = state.pointer_down(
            point::logical(1.0, 2.0),
            point::logical(0.0, 0.0),
            Some(path(CHILD)),
            pointer::Button::Primary,
        );

        assert_eq!(state.focused_path(), Some(path(CHILD)));
        assert_eq!(
            state.focus.as_ref().map(|focus| focus.reason),
            Some(ui::focus::Reason::Pointer)
        );
        assert_eq!(state.focus_visibility(), ui::focus::Visibility::Hidden);
        assert_eq!(state.pressed, Some(path(CHILD)));
        assert_eq!(
            event,
            ui::Event::PointerDown {
                position: point::logical(1.0, 2.0),
                delta: point::logical(0.0, 0.0),
                target: Some(path(CHILD)),
                button: pointer::Button::Primary
            }
        );
    }

    #[test]
    fn passive_pointer_down_does_not_focus_element() {
        let mut state = WindowState::default();

        state.pointer_down(
            point::logical(1.0, 2.0),
            point::logical(0.0, 0.0),
            Some(path(CHILD)),
            pointer::Button::Primary,
        );

        assert_eq!(state.focused_path(), None);
        assert_eq!(state.pressed, Some(path(CHILD)));
    }

    #[test]
    fn programmatic_focus_can_choose_visible_or_hidden_indication() {
        let mut state = WindowState {
            interactivity: HashMap::from([(path(CHILD), ui::Interactivity::CONTROL)]),
            ..WindowState::default()
        };

        assert!(state.set_focus(
            path(CHILD),
            ui::focus::Reason::Programmatic,
            ui::focus::Visibility::Visible,
        ));
        assert_eq!(state.focused_path(), Some(path(CHILD)));
        assert_eq!(state.focus_visibility(), ui::focus::Visibility::Visible);

        assert!(state.set_focus(
            path(CHILD),
            ui::focus::Reason::Programmatic,
            ui::focus::Visibility::Hidden,
        ));
        assert_eq!(state.focus_visibility(), ui::focus::Visibility::Hidden);
    }

    #[test]
    fn stale_focused_paths_are_cleared_when_not_focusable() {
        let mut state = WindowState {
            focus: Some(Focus::new(
                path(CHILD),
                ui::focus::Reason::Keyboard,
                ui::focus::Visibility::Visible,
            )),
            ..WindowState::default()
        };

        assert!(state.clear_stale_focus());
        assert_eq!(state.focused_path(), None);
    }

    #[test]
    fn pointer_capture_routes_release_to_pressed_element() {
        let mut state = WindowState {
            pressed: Some(path(CHILD)),
            pressed_source: Some(PressSource::Pointer),
            interactivity: HashMap::from([(path(CHILD), ui::Interactivity::CONTROL)]),
            ..WindowState::default()
        };

        let (event, invoke) = state.pointer_up(
            point::logical(50.0, 50.0),
            point::logical(0.0, 0.0),
            Some(path(OUTSIDE)),
            pointer::Button::Primary,
        );

        assert_eq!(
            event,
            ui::Event::PointerUp {
                position: point::logical(50.0, 50.0),
                delta: point::logical(0.0, 0.0),
                target: Some(path(CHILD)),
                button: pointer::Button::Primary
            }
        );
        assert_eq!(invoke, Some(path(CHILD)));
    }

    #[test]
    fn non_primary_release_does_not_invoke_action() {
        let mut state = WindowState {
            pressed: Some(path(CHILD)),
            pressed_source: Some(PressSource::Pointer),
            interactivity: HashMap::from([(path(CHILD), ui::Interactivity::CONTROL)]),
            ..WindowState::default()
        };

        let (_, invoke) = state.pointer_up(
            point::logical(1.0, 1.0),
            point::logical(0.0, 0.0),
            Some(path(CHILD)),
            pointer::Button::Secondary,
        );

        assert_eq!(invoke, None);
    }

    #[test]
    fn passive_pressed_element_does_not_invoke_action() {
        let mut state = WindowState {
            pressed: Some(path(CHILD)),
            pressed_source: Some(PressSource::Pointer),
            ..WindowState::default()
        };

        let (_, invoke) = state.pointer_up(
            point::logical(1.0, 1.0),
            point::logical(0.0, 0.0),
            Some(path(CHILD)),
            pointer::Button::Primary,
        );

        assert_eq!(invoke, None);
    }

    #[test]
    fn pointer_release_over_pressed_action_emits_contextual_request() {
        let window = window::Id::new(1);
        let mut state = WindowState {
            layout: Some(layout::Box::new(
                CHILD,
                Rect::new(point::logical(0.0, 0.0), area::logical(10.0, 10.0)),
                Vec::new(),
            )),
            actions: HashMap::from([(path(CHILD), CLICK)]),
            interactivity: HashMap::from([(path(CHILD), ui::Interactivity::CONTROL)]),
            ..WindowState::default()
        };
        let mut registry = action::Registry::<()>::new();

        registry.register(Action::new(CLICK, "Click"));
        state.pointer_down(
            point::logical(1.0, 1.0),
            point::logical(0.0, 0.0),
            Some(path(CHILD)),
            pointer::Button::Primary,
        );
        let (_, target) = state.pointer_up(
            point::logical(1.0, 1.0),
            point::logical(0.0, 0.0),
            Some(path(CHILD)),
            pointer::Button::Primary,
        );
        let request = action_request(
            &state,
            window,
            target.expect("release should target pressed element"),
            action::Source::Pointer,
        )
        .filter(|request| registry.can_invoke(request.action(), request.target().clone()));

        assert_eq!(
            request,
            Some(
                action::Request::new(
                    CLICK,
                    action::Source::Pointer,
                    action::Context::path(window, path(CHILD))
                )
                .with_origin(path(CHILD))
            )
        );
    }

    #[test]
    fn disabled_action_bound_node_does_not_invoke() {
        let window = window::Id::new(1);
        let context = action::Context::path(window, path(CHILD));
        let mut registry = action::Registry::<()>::new();
        let state = WindowState {
            actions: HashMap::from([(path(CHILD), CLICK)]),
            ..WindowState::default()
        };

        registry.register(Action::new(CLICK, "Click"));
        registry.set_state(CLICK, context, action::State::disabled());

        assert_eq!(
            action_request(&state, window, path(CHILD), action::Source::Pointer)
                .filter(|request| registry.can_invoke(request.action(), request.target().clone())),
            None
        );
    }

    #[test]
    fn menu_opens_only_when_an_item_can_invoke_after_resolution() {
        let window = window::Id::new(1);
        let menu = menu::Menu::new(FILE, "File")
            .section(menu::Section::new().item(menu::Item::new(action::SELECT_ALL)));
        let mut registry = action::Registry::<()>::new();
        let mut state = WindowState {
            menus: HashMap::from([(FILE, menu)]),
            command_subject: Some(action::Scope::Path(path(CHILD))),
            responders: HashMap::from([(path(CHILD), vec![action::SELECT_ALL])]),
            ..WindowState::default()
        };

        registry.register(Action::new(action::SELECT_ALL, "Select All"));
        registry.set_state(
            action::SELECT_ALL,
            action::Context::window(window),
            action::State::disabled(),
        );

        assert!(!state.toggle_menu(FILE, &registry, window));
        assert_eq!(state.open_menu, None);

        registry.set_state(
            action::SELECT_ALL,
            action::Context::path(window, path(CHILD)),
            action::State::enabled(),
        );

        assert!(state.toggle_menu(FILE, &registry, window));
        assert_eq!(state.open_menu, Some(FILE));
    }

    #[test]
    fn menu_toggle_switches_and_closes_current_menu() {
        let window = window::Id::new(1);
        let edit = menu::Id::new("edit");
        let file_menu = menu::Menu::new(FILE, "File")
            .section(menu::Section::new().item(menu::Item::new(CLICK)));
        let edit_menu = menu::Menu::new(edit, "Edit")
            .section(menu::Section::new().item(menu::Item::new(CLICK)));
        let mut registry = action::Registry::<()>::new();
        let mut state = WindowState {
            menus: HashMap::from([(FILE, file_menu), (edit, edit_menu)]),
            ..WindowState::default()
        };

        registry.register(Action::new(CLICK, "Click"));

        assert!(state.toggle_menu(FILE, &registry, window));
        assert_eq!(state.open_menu, Some(FILE));
        state.open_submenu = Some(PANELS);
        assert!(state.toggle_menu(edit, &registry, window));
        assert_eq!(state.open_menu, Some(edit));
        assert_eq!(state.open_submenu, None);
        assert!(state.toggle_menu(edit, &registry, window));
        assert_eq!(state.open_menu, None);
    }

    #[test]
    fn submenu_opens_only_when_parent_menu_is_open_and_item_can_invoke() {
        let window = window::Id::new(1);
        let submenu = menu::Menu::new(PANELS, "Panels")
            .section(menu::Section::new().item(menu::Item::new(CLICK)));
        let mut registry = action::Registry::<()>::new();
        let mut state = WindowState {
            menus: HashMap::from([(PANELS, submenu)]),
            ..WindowState::default()
        };

        registry.register(Action::new(CLICK, "Click"));

        assert!(!state.open_submenu(PANELS, &registry, window));
        state.open_menu = Some(FILE);
        assert!(state.open_submenu(PANELS, &registry, window));
        assert_eq!(state.open_submenu, Some(PANELS));
    }

    #[test]
    fn closing_top_level_menu_also_closes_submenu() {
        let mut state = WindowState {
            open_menu: Some(FILE),
            open_submenu: Some(PANELS),
            ..WindowState::default()
        };

        assert!(state.close_menu());
        assert_eq!(state.open_menu, None);
        assert_eq!(state.open_submenu, None);
    }

    #[test]
    fn outside_pointer_target_dismisses_open_menu() {
        let mut state = WindowState {
            open_menu: Some(FILE),
            ..WindowState::default()
        };

        assert!(state.dismiss_menu_for_target(Some(&path(CHILD))));
        assert_eq!(state.open_menu, None);
    }

    #[test]
    fn menu_pointer_target_does_not_dismiss_open_menu() {
        let mut state = WindowState {
            open_menu: Some(FILE),
            intents: HashMap::from([(path(CHILD), ui::Intent::OpenMenu(FILE))]),
            ..WindowState::default()
        };

        assert!(!state.dismiss_menu_for_target(Some(&path(CHILD))));
        assert_eq!(state.open_menu, Some(FILE));
        assert!(state.is_menu_path(&path(CHILD)));
    }

    #[test]
    fn submenu_popup_target_does_not_dismiss_open_menu() {
        let submenu_row = ui::Path::new([ui::widget::MENU_SUBMENU_POPUP, CHILD]);
        let mut state = WindowState {
            open_menu: Some(FILE),
            open_submenu: Some(PANELS),
            ..WindowState::default()
        };

        assert!(!state.dismiss_menu_for_target(Some(&submenu_row)));
        assert_eq!(state.open_menu, Some(FILE));
        assert_eq!(state.open_submenu, Some(PANELS));
        assert!(state.is_menu_path(&submenu_row));
    }

    #[test]
    fn busy_action_bound_node_does_not_invoke() {
        let window = window::Id::new(1);
        let context = action::Context::path(window, path(CHILD));
        let mut registry = action::Registry::<()>::new();
        let state = WindowState {
            actions: HashMap::from([(path(CHILD), CLICK)]),
            ..WindowState::default()
        };

        registry.register(Action::new(CLICK, "Click"));
        registry.set_busy(CLICK, context, true);

        assert_eq!(
            action_request(&state, window, path(CHILD), action::Source::Pointer)
                .filter(|request| registry.can_invoke(request.action(), request.target().clone())),
            None
        );
    }

    #[test]
    fn command_target_survives_focus_changes() {
        let window = window::Id::new(1);
        let mut state = WindowState {
            command_subject: Some(action::Scope::Path(path(CHILD))),
            focus: Some(Focus::new(
                path(ROOT),
                ui::focus::Reason::Keyboard,
                ui::focus::Visibility::Visible,
            )),
            interactivity: HashMap::from([(path(OUTSIDE), ui::Interactivity::CONTROL)]),
            ..WindowState::default()
        };

        assert!(state.set_focus(
            path(OUTSIDE),
            ui::focus::Reason::Programmatic,
            ui::focus::Visibility::Hidden
        ));
        assert_eq!(
            state.command_context(window),
            action::Context::path(window, path(CHILD))
        );
    }

    #[test]
    fn transient_focus_does_not_replace_command_subject() {
        let window = window::Id::new(1);
        let mut state = WindowState {
            command_subject: Some(action::Scope::Path(path(CHILD))),
            interactivity: HashMap::from([(path(OUTSIDE), ui::Interactivity::CONTROL)]),
            responders: HashMap::from([(path(CHILD), vec![CLICK])]),
            ..WindowState::default()
        };

        assert!(state.set_focus(
            path(OUTSIDE),
            ui::focus::Reason::Keyboard,
            ui::focus::Visibility::Visible
        ));
        assert_eq!(
            state.command_context(window),
            action::Context::path(window, path(CHILD))
        );
    }

    #[test]
    fn responder_focus_replaces_command_subject() {
        let window = window::Id::new(1);
        let mut state = WindowState {
            command_subject: Some(action::Scope::Path(path(ROOT))),
            interactivity: HashMap::from([(path(CHILD), ui::Interactivity::CONTROL)]),
            responders: HashMap::from([(path(ROOT), vec![CLICK]), (path(CHILD), vec![CLICK])]),
            ..WindowState::default()
        };

        assert!(state.set_focus(
            path(CHILD),
            ui::focus::Reason::Keyboard,
            ui::focus::Visibility::Visible
        ));
        assert_eq!(
            state.command_context(window),
            action::Context::path(window, path(CHILD))
        );
    }

    #[test]
    fn focused_responder_automatically_becomes_command_subject() {
        let window = window::Id::new(1);
        let mut state = WindowState {
            interactivity: HashMap::from([(path(CHILD), ui::Interactivity::CONTROL)]),
            responders: HashMap::from([(path(CHILD), vec![CLICK])]),
            ..WindowState::default()
        };

        assert!(state.set_focus(
            path(CHILD),
            ui::focus::Reason::Keyboard,
            ui::focus::Visibility::Visible
        ));
        assert_eq!(
            state.command_context(window),
            action::Context::path(window, path(CHILD))
        );
    }

    #[test]
    fn refocusing_same_responder_restores_cleared_command_subject() {
        let window = window::Id::new(1);
        let mut state = WindowState {
            focus: Some(Focus::new(
                path(CHILD),
                ui::focus::Reason::Keyboard,
                ui::focus::Visibility::Visible,
            )),
            interactivity: HashMap::from([(path(CHILD), ui::Interactivity::CONTROL)]),
            responders: HashMap::from([(path(CHILD), vec![CLICK])]),
            ..WindowState::default()
        };

        assert_eq!(state.command_subject, None);
        assert!(state.set_focus(
            path(CHILD),
            ui::focus::Reason::Keyboard,
            ui::focus::Visibility::Visible
        ));
        assert_eq!(
            state.command_context(window),
            action::Context::path(window, path(CHILD))
        );
    }

    #[test]
    fn command_subject_falls_back_to_focus_then_window() {
        let window = window::Id::new(1);
        let mut state = WindowState {
            hovered: Some(path(ROOT)),
            focus: Some(Focus::new(
                path(CHILD),
                ui::focus::Reason::Keyboard,
                ui::focus::Visibility::Visible,
            )),
            responders: HashMap::from([(path(ROOT), vec![CLICK]), (path(CHILD), vec![CLICK])]),
            ..WindowState::default()
        };

        assert_eq!(
            state.command_context(window),
            action::Context::path(window, path(CHILD))
        );

        state.focus = None;
        assert_eq!(
            state.command_context(window),
            action::Context::window(window)
        );
    }

    #[test]
    fn hover_alone_does_not_become_command_subject() {
        let window = window::Id::new(1);
        let state = WindowState {
            hovered: Some(path(ROOT)),
            responders: HashMap::from([(path(ROOT), vec![CLICK])]),
            ..WindowState::default()
        };

        assert_eq!(
            state.command_context(window),
            action::Context::window(window)
        );
    }

    #[test]
    fn stale_command_target_is_cleared_when_path_disappears() {
        let window = window::Id::new(1);
        let mut state = WindowState {
            command_subject: Some(action::Scope::Path(path(CHILD))),
            interactivity: HashMap::from([(path(ROOT), ui::Interactivity::CONTROL)]),
            ..WindowState::default()
        };

        assert!(state.clear_stale_command_target());
        assert_eq!(state.command_subject, None);
        assert_eq!(
            state.command_context(window),
            action::Context::window(window)
        );
    }

    #[test]
    fn command_target_policy_resolves_stored_target() {
        let window = window::Id::new(1);
        let state = WindowState {
            command_subject: Some(action::Scope::Path(path(CHILD))),
            actions: HashMap::from([(path(ROOT), CLICK)]),
            action_targets: HashMap::from([(path(ROOT), ui::ActionTarget::Command)]),
            ..WindowState::default()
        };

        let request = action_request(&state, window, path(ROOT), action::Source::Pointer)
            .expect("command-target action should produce request");

        assert_eq!(request.origin(), Some(&path(ROOT)));
        assert_eq!(
            request.target(),
            &action::Context::path(window, path(CHILD))
        );
    }

    #[test]
    fn command_target_policy_resolves_window_without_subject() {
        let window = window::Id::new(1);
        let state = WindowState {
            actions: HashMap::from([(path(ROOT), CLICK)]),
            action_targets: HashMap::from([(path(ROOT), ui::ActionTarget::Command)]),
            ..WindowState::default()
        };

        let request = action_request(&state, window, path(ROOT), action::Source::Pointer)
            .expect("command-target action should produce request");

        assert_eq!(request.target(), &action::Context::window(window));
    }

    #[test]
    fn window_target_policy_resolves_window_context() {
        let window = window::Id::new(1);
        let state = WindowState {
            command_subject: Some(action::Scope::Path(path(CHILD))),
            actions: HashMap::from([(path(ROOT), CLICK)]),
            action_targets: HashMap::from([(path(ROOT), ui::ActionTarget::Window)]),
            ..WindowState::default()
        };

        let request = action_request(&state, window, path(ROOT), action::Source::Pointer)
            .expect("window-target action should produce request");

        assert_eq!(request.origin(), Some(&path(ROOT)));
        assert_eq!(request.target(), &action::Context::window(window));
    }

    #[test]
    fn captured_target_policy_resolves_nearest_scope_capture() {
        let window = window::Id::new(1);
        let scope = path(ROOT);
        let origin = ui::Path::new([ROOT, CHILD]);
        let subject = path(OUTSIDE);
        let state = WindowState {
            actions: HashMap::from([(origin.clone(), CLICK)]),
            action_targets: HashMap::from([(origin.clone(), ui::ActionTarget::Captured)]),
            command_scope_captures: HashMap::from([(
                scope,
                action::Context::path(window, subject.clone()),
            )]),
            ..WindowState::default()
        };

        let request = action_request(&state, window, origin, action::Source::Pointer)
            .expect("captured-target action should produce request");

        assert_eq!(request.target(), &action::Context::path(window, subject));
    }

    #[test]
    fn local_responder_inside_scope_becomes_command_target() {
        let window = window::Id::new(1);
        let local = ui::Path::new([ROOT, CHILD]);
        let button = ui::Path::new([ROOT, OUTSIDE]);
        let mut state = WindowState {
            interactivity: HashMap::from([(local.clone(), ui::Interactivity::CONTROL)]),
            responders: HashMap::from([(local.clone(), vec![CLICK])]),
            actions: HashMap::from([(button.clone(), CLICK)]),
            action_targets: HashMap::from([(button.clone(), ui::ActionTarget::Command)]),
            ..WindowState::default()
        };

        assert!(state.set_focus(
            local.clone(),
            ui::focus::Reason::Keyboard,
            ui::focus::Visibility::Visible
        ));
        let request = action_request(&state, window, button, action::Source::Pointer)
            .expect("command-target action should produce request");

        assert_eq!(request.target(), &action::Context::path(window, local));
    }

    #[test]
    fn responder_resolution_picks_nearest_handler() {
        let window = window::Id::new(1);
        let root = path(ROOT);
        let child = ui::Path::new([ROOT, CHILD]);
        let outside = ui::Path::new([ROOT, CHILD, OUTSIDE]);
        let state = WindowState {
            responders: HashMap::from([(root.clone(), vec![CLICK]), (child.clone(), vec![CLICK])]),
            ..WindowState::default()
        };
        let request = action::Request::new(
            CLICK,
            action::Source::Shortcut,
            action::Context::path(window, outside),
        );

        assert_eq!(
            state.resolve_request(request).target(),
            &action::Context::path(window, child)
        );
    }

    #[test]
    fn responder_resolution_falls_back_to_window_without_handler() {
        let window = window::Id::new(1);
        let state = WindowState {
            responders: HashMap::from([(path(ROOT), vec![action::COPY])]),
            ..WindowState::default()
        };
        let request = action::Request::new(
            CLICK,
            action::Source::Shortcut,
            action::Context::path(window, path(ROOT)),
        );

        assert_eq!(
            state.resolve_request(request).target(),
            &action::Context::window(window)
        );
    }

    #[test]
    fn origin_bound_action_resolves_to_itself() {
        let window = window::Id::new(1);
        let state = WindowState {
            actions: HashMap::from([(path(CHILD), CLICK)]),
            action_targets: HashMap::from([(path(CHILD), ui::ActionTarget::Origin)]),
            ..WindowState::default()
        };
        let request = action::Request::new(
            CLICK,
            action::Source::Pointer,
            action::Context::path(window, path(CHILD)),
        )
        .with_origin(path(CHILD));

        assert_eq!(
            state.resolve_request(request).target(),
            &action::Context::path(window, path(CHILD))
        );
    }

    #[test]
    fn disabled_responder_target_blocks_invocation_after_resolution() {
        let window = window::Id::new(1);
        let mut registry = action::Registry::<()>::new();
        let state = WindowState {
            responders: HashMap::from([(path(CHILD), vec![action::SELECT_ALL])]),
            ..WindowState::default()
        };
        let request = action::Request::new(
            action::SELECT_ALL,
            action::Source::Shortcut,
            action::Context::path(window, path(CHILD)),
        );

        registry.register(Action::new(action::SELECT_ALL, "Select All"));
        registry.set_state(
            action::SELECT_ALL,
            action::Context::path(window, path(CHILD)),
            action::State::disabled(),
        );
        let request = state.resolve_request(request);

        assert!(!registry.can_invoke(request.action(), request.target().clone()));
    }
}
