use crate::app::menu_ui;
use crate::app::state::WindowState;
use crate::geometry::area;
use crate::{action, layout, paint, ui, window};

pub fn compose<T>(
    window: window::Id,
    tree: &ui::Tree,
    state: &mut WindowState,
    actions: &action::Registry<T>,
    logical_area: area::Logical,
) -> paint::Scene {
    let mut scene = paint::Scene::new();
    let mut tree = tree.clone();

    state.menus = tree.menus();
    if state
        .open_menu
        .is_some_and(|menu| !state.menus.contains_key(&menu))
    {
        state.open_menu = None;
    }
    if state.open_menu.is_none() {
        state.open_submenu = None;
    }
    if state
        .open_submenu
        .is_some_and(|menu| !state.menus.contains_key(&menu))
    {
        state.open_submenu = None;
    }

    let command_target = state.command_context(window);
    if let Some(open_menu) = state.open_menu
        && let Some(menu) = state.menus.get(&open_menu)
        && let Some(base_layout) = tree.layout(logical_area)
        && let Some(popup) = menu_ui::popup(&tree, &base_layout, menu, actions, &command_target)
    {
        tree.push_popup(popup);
    }
    if let Some(open_submenu) = state.open_submenu
        && let Some(menu) = state.menus.get(&open_submenu)
        && let Some(menu_layout) = tree.layout(logical_area)
        && let Some(popup) =
            menu_ui::submenu_popup(&tree, &menu_layout, menu, actions, &command_target)
    {
        tree.push_popup(popup);
    }

    state.actions = tree.actions();
    state.action_targets = tree.action_targets();
    state.intents = tree.intents();
    state.responders = tree.responders();
    state.command_scopes = tree.command_scopes();
    state.interactivity = tree.interactivity();
    state.scrollables = tree.scrollables();

    if let Some(layout) = tree.layout(logical_area) {
        state.focus_order = focus_order(&layout, &state.interactivity);
        state.clear_stale_focus();
        state.clear_stale_command_target();
        state.update_command_scope_captures(window);
        let command_target = state.command_context(window);

        let interaction = ui::Interaction::new(
            state.hovered.clone(),
            state.focused_path(),
            state.pressed.clone(),
        )
        .with_focus_visibility(state.focus_visibility())
        .with_command_target(command_target)
        .with_command_scope_captures(state.command_scope_captures.clone())
        .with_open_menu(state.open_menu)
        .with_open_submenu(state.open_submenu);
        state.layout = Some(layout.clone());

        tree.paint(&layout, actions, window, interaction, &mut scene);
    } else {
        state.layout = None;
        state.focus_order.clear();
        state.clear_focus();
        state.clear_command_target();
        state.command_scopes.clear();
        state.command_scope_captures.clear();
        state.scrollables.clear();
    }

    scene
}

fn focus_order(
    layout: &layout::Box,
    interactivity: &std::collections::HashMap<ui::Path, ui::Interactivity>,
) -> Vec<ui::Path> {
    let mut order = Vec::new();
    collect_focus_order(layout, interactivity, &mut order);
    order
}

fn collect_focus_order(
    layout: &layout::Box,
    interactivity: &std::collections::HashMap<ui::Path, ui::Interactivity>,
    order: &mut Vec<ui::Path>,
) {
    if interactivity
        .get(layout.path())
        .is_some_and(|interactivity| interactivity.focusable())
    {
        order.push(layout.path().clone());
    }

    for child in layout.children() {
        collect_focus_order(child, interactivity, order);
    }
}

#[cfg(test)]
mod tests {
    use crate::geometry::area;
    use crate::{Action, layout, menu, paint};

    use super::*;

    const ROOT: ui::Id = ui::Id::new("root");
    const CHILD: ui::Id = ui::Id::new("child");
    const OTHER: ui::Id = ui::Id::new("other");
    const CLICK: action::Id = action::Id::new("click");
    const TOGGLE: action::Id = action::Id::new("toggle");
    const MENU_BAR: ui::Id = ui::Id::new("menu_bar");
    const FILE: menu::Id = menu::Id::new("file");
    const VIEW: menu::Id = menu::Id::new("view");
    const PANELS: menu::Id = menu::Id::new("panels");

    #[test]
    fn compose_updates_state_and_preserves_paint_order() {
        let window = window::Id::new(1);
        let mut state = WindowState::default();
        let mut registry = action::Registry::<()>::new();
        let mut tree = ui::Tree::new();

        registry.register(Action::new(CLICK, "Click"));
        tree.set_root(
            ui::control::panel(ROOT)
                .with_background(paint::Color::BLACK)
                .with_child(
                    ui::control::button(CHILD, CLICK)
                        .with_size(layout::Size::Fixed(10.0), layout::Size::Fixed(10.0)),
                ),
        );

        let scene = compose(
            window,
            &tree,
            &mut state,
            &registry,
            area::logical(100.0, 100.0),
        );

        assert!(state.layout.is_some());
        assert_eq!(
            state.actions.get(&ui::Path::new([ROOT, CHILD])),
            Some(&CLICK)
        );
        assert_eq!(
            state.action_targets.get(&ui::Path::new([ROOT, CHILD])),
            Some(&ui::ActionTarget::Origin)
        );
        assert!(state.responders.is_empty());
        assert!(state.interactivity.contains_key(&ui::Path::from(ROOT)));
        assert_eq!(state.focus_order, vec![ui::Path::new([ROOT, CHILD])]);
        assert_eq!(scene.items().len(), 2);
    }

    #[test]
    fn compose_clears_stale_focused_paths_after_tree_rebuild() {
        let window = window::Id::new(1);
        let mut state = WindowState {
            focus: Some(crate::app::state::Focus::new(
                ui::Path::new([ROOT, CHILD]),
                ui::focus::Reason::Keyboard,
                ui::focus::Visibility::Visible,
            )),
            ..WindowState::default()
        };
        let registry = action::Registry::<()>::new();
        let mut tree = ui::Tree::new();

        tree.set_root(
            ui::control::panel(ROOT).with_child(
                ui::control::button(OTHER, CLICK)
                    .with_responder(CLICK)
                    .with_size(layout::Size::Fixed(10.0), layout::Size::Fixed(10.0)),
            ),
        );

        compose(
            window,
            &tree,
            &mut state,
            &registry,
            area::logical(100.0, 100.0),
        );

        assert_eq!(state.focused_path(), None);
    }

    #[test]
    fn compose_clears_stale_command_target_after_tree_rebuild() {
        let window = window::Id::new(1);
        let mut state = WindowState {
            command_subject: Some(action::Scope::Path(ui::Path::new([ROOT, CHILD]))),
            ..WindowState::default()
        };
        let registry = action::Registry::<()>::new();
        let mut tree = ui::Tree::new();

        tree.set_root(
            ui::control::panel(ROOT).with_child(
                ui::control::button(OTHER, CLICK)
                    .with_size(layout::Size::Fixed(10.0), layout::Size::Fixed(10.0)),
            ),
        );

        compose(
            window,
            &tree,
            &mut state,
            &registry,
            area::logical(100.0, 100.0),
        );

        assert_eq!(state.command_subject, None);
    }

    #[test]
    fn compose_stores_responder_paths() {
        let window = window::Id::new(1);
        let mut state = WindowState::default();
        let registry = action::Registry::<()>::new();
        let mut tree = ui::Tree::new();

        tree.set_root(
            ui::control::panel(ROOT).with_child(
                ui::Node::leaf(CHILD)
                    .with_responder(action::SELECT_ALL)
                    .with_size(layout::Size::Fixed(10.0), layout::Size::Fixed(10.0)),
            ),
        );

        compose(
            window,
            &tree,
            &mut state,
            &registry,
            area::logical(100.0, 100.0),
        );

        assert_eq!(
            state.responders.get(&ui::Path::new([ROOT, CHILD])),
            Some(&vec![action::SELECT_ALL])
        );
    }

    #[test]
    fn compose_captures_inherited_command_subject_for_scope() {
        let window = window::Id::new(1);
        let subject = ui::Path::new([ROOT, CHILD]);
        let scope = ui::Path::new([ROOT, OTHER]);
        let mut state = WindowState {
            command_subject: Some(action::Scope::Path(subject.clone())),
            ..WindowState::default()
        };
        let registry = action::Registry::<()>::new();
        let mut tree = ui::Tree::new();

        tree.set_root(
            ui::control::panel(ROOT)
                .with_child(
                    ui::Node::leaf(CHILD)
                        .with_responder(action::SELECT_ALL)
                        .with_size(layout::Size::Fixed(10.0), layout::Size::Fixed(10.0)),
                )
                .with_child(
                    ui::control::panel(OTHER)
                        .with_command_scope()
                        .with_size(layout::Size::Fixed(10.0), layout::Size::Fixed(10.0)),
                ),
        );

        compose(
            window,
            &tree,
            &mut state,
            &registry,
            area::logical(100.0, 100.0),
        );

        assert_eq!(
            state.command_scope_captures.get(&scope),
            Some(&action::Context::path(window, subject))
        );
    }

    #[test]
    fn compose_clears_stale_scope_captures() {
        let window = window::Id::new(1);
        let scope = ui::Path::new([ROOT, OTHER]);
        let subject = ui::Path::new([ROOT, CHILD]);
        let mut state = WindowState {
            command_scope_captures: std::collections::HashMap::from([(
                scope,
                action::Context::path(window, subject),
            )]),
            ..WindowState::default()
        };
        let registry = action::Registry::<()>::new();
        let mut tree = ui::Tree::new();

        tree.set_root(ui::control::panel(ROOT));

        compose(
            window,
            &tree,
            &mut state,
            &registry,
            area::logical(100.0, 100.0),
        );

        assert!(state.command_scope_captures.is_empty());
    }

    #[test]
    fn compose_injects_open_menu_popup_from_menu_bar() {
        let window = window::Id::new(1);
        let mut state = WindowState {
            open_menu: Some(FILE),
            command_subject: Some(action::Scope::Path(ui::Path::new([ROOT, CHILD]))),
            ..WindowState::default()
        };
        let mut registry = action::Registry::<()>::new();
        let mut tree = ui::Tree::new();

        registry.register(Action::new(action::SELECT_ALL, "Select All"));
        registry.set_state(
            action::SELECT_ALL,
            action::Context::path(window, ui::Path::new([ROOT, CHILD])),
            action::State::enabled(),
        );
        tree.set_root(
            ui::control::panel(ROOT)
                .with_child(ui::widget::menu_bar(
                    MENU_BAR,
                    menu::Bar::new().menu(
                        menu::Menu::new(FILE, "File").section(
                            menu::Section::new().item(menu::Item::new(action::SELECT_ALL)),
                        ),
                    ),
                ))
                .with_child(
                    ui::Node::leaf(CHILD)
                        .with_responder(action::SELECT_ALL)
                        .with_interactivity(ui::Interactivity::CONTROL),
                ),
        );

        let scene = compose(
            window,
            &tree,
            &mut state,
            &registry,
            area::logical(300.0, 180.0),
        );
        let theme = crate::theme::Theme::default_dark();

        let row = ui::Path::new([ROOT, ui::widget::MENU_POPUP, ui::Id::new("__menu_row_00")]);
        let scope = ui::Path::new([ROOT, ui::widget::MENU_POPUP]);

        assert!(
            state
                .layout
                .as_ref()
                .and_then(|layout| layout.find_path(&scope))
                .is_some()
        );
        assert_eq!(state.actions.get(&row), Some(&action::SELECT_ALL));
        assert_eq!(
            state.action_targets.get(&row),
            Some(&ui::ActionTarget::Captured)
        );
        assert_eq!(
            state.command_scope_captures.get(&scope),
            Some(&action::Context::path(window, ui::Path::new([ROOT, CHILD])))
        );

        let backdrop_index = scene
            .items()
            .iter()
            .position(|item| matches!(item, paint::Item::Backdrop(_)))
            .expect("open menu should lower a backdrop item");
        let paint::Item::Backdrop(backdrop) = &scene.items()[backdrop_index] else {
            unreachable!();
        };
        let paint::BackdropFilter::Blur { amount } = backdrop.filter;
        assert_eq!(amount, theme.floating_panel().backdrop_blur());

        let paint::Item::Quad(quad) = &scene.items()[backdrop_index + 1] else {
            panic!("popup material fill should follow popup backdrop");
        };
        assert_eq!(
            quad.style.fill,
            Some(paint::Fill::Brush(theme.floating_panel().backdrop_fill()))
        );
    }

    #[test]
    fn focused_open_menu_row_lowers_focus_background() {
        let window = window::Id::new(1);
        let subject = ui::Path::new([ROOT, CHILD]);
        let row = ui::Path::new([ROOT, ui::widget::MENU_POPUP, ui::Id::new("__menu_row_00")]);
        let mut state = WindowState {
            open_menu: Some(FILE),
            command_subject: Some(action::Scope::Path(subject.clone())),
            focus: Some(crate::app::state::Focus::new(
                row,
                ui::focus::Reason::Keyboard,
                ui::focus::Visibility::Visible,
            )),
            ..WindowState::default()
        };
        let mut registry = action::Registry::<()>::new();
        let mut tree = ui::Tree::new();

        registry.register(Action::new(action::SELECT_ALL, "Select All"));
        registry.set_state(
            action::SELECT_ALL,
            action::Context::path(window, subject),
            action::State::enabled(),
        );
        tree.set_root(
            ui::control::panel(ROOT)
                .with_child(ui::widget::menu_bar(
                    MENU_BAR,
                    menu::Bar::new().menu(
                        menu::Menu::new(FILE, "File").section(
                            menu::Section::new().item(menu::Item::new(action::SELECT_ALL)),
                        ),
                    ),
                ))
                .with_child(
                    ui::Node::leaf(CHILD)
                        .with_responder(action::SELECT_ALL)
                        .with_interactivity(ui::Interactivity::CONTROL),
                ),
        );

        let scene = compose(
            window,
            &tree,
            &mut state,
            &registry,
            area::logical(300.0, 180.0),
        );
        let theme = crate::theme::Theme::default_dark();

        assert!(scene.items().iter().any(|item| {
            matches!(
                item,
                paint::Item::Quad(quad)
                    if quad.style.fill == Some(paint::Fill::Brush(theme.menu().row_hover_tint()))
            )
        }));
    }

    #[test]
    fn active_menu_item_lowers_check_glyph() {
        let window = window::Id::new(1);
        let subject = ui::Path::new([ROOT, CHILD]);
        let mut state = WindowState {
            open_menu: Some(VIEW),
            command_subject: Some(action::Scope::Path(subject.clone())),
            ..WindowState::default()
        };
        let mut registry = action::Registry::<()>::new();
        let mut tree = ui::Tree::new();

        registry.register(Action::new(TOGGLE, "Toggle Preview"));
        registry.set_state(
            TOGGLE,
            action::Context::path(window, subject),
            action::State::active(),
        );
        tree.set_root(
            ui::control::panel(ROOT)
                .with_child(ui::widget::menu_bar(
                    MENU_BAR,
                    menu::Bar::new().menu(
                        menu::Menu::new(VIEW, "View")
                            .section(menu::Section::new().item(menu::Item::new(TOGGLE))),
                    ),
                ))
                .with_child(ui::Node::leaf(CHILD).with_responder(TOGGLE)),
        );

        let scene = compose(
            window,
            &tree,
            &mut state,
            &registry,
            area::logical(300.0, 180.0),
        );

        assert!(scene.items().iter().any(|item| {
            matches!(
                item,
                paint::Item::Icon(icon)
                    if icon.icon == crate::Icon::phosphor(crate::icon::Id::new("check"))
            )
        }));
    }

    #[test]
    fn compose_injects_open_submenu_popup() {
        let window = window::Id::new(1);
        let subject = ui::Path::new([ROOT, CHILD]);
        let mut state = WindowState {
            open_menu: Some(VIEW),
            open_submenu: Some(PANELS),
            command_subject: Some(action::Scope::Path(subject.clone())),
            ..WindowState::default()
        };
        let mut registry = action::Registry::<()>::new();
        let mut tree = ui::Tree::new();

        registry.register(Action::new(TOGGLE, "Toggle Preview"));
        registry.set_state(
            TOGGLE,
            action::Context::path(window, subject.clone()),
            action::State::enabled(),
        );
        tree.set_root(
            ui::control::panel(ROOT)
                .with_child(ui::widget::menu_bar(
                    MENU_BAR,
                    menu::Bar::new().menu(
                        menu::Menu::new(VIEW, "View").section(
                            menu::Section::new().submenu(
                                menu::Menu::new(PANELS, "Panels")
                                    .section(menu::Section::new().item(menu::Item::new(TOGGLE))),
                            ),
                        ),
                    ),
                ))
                .with_child(ui::Node::leaf(CHILD).with_responder(TOGGLE)),
        );

        compose(
            window,
            &tree,
            &mut state,
            &registry,
            area::logical(360.0, 220.0),
        );

        let submenu_popup = ui::Path::new([ROOT, ui::widget::MENU_SUBMENU_POPUP]);
        let submenu_row = ui::Path::new([
            ROOT,
            ui::widget::MENU_SUBMENU_POPUP,
            ui::Id::new("__menu_row_00"),
        ]);
        let top_submenu_row =
            ui::Path::new([ROOT, ui::widget::MENU_POPUP, ui::Id::new("__menu_row_00")]);

        assert!(
            state
                .layout
                .as_ref()
                .and_then(|layout| layout.find_path(&submenu_popup))
                .is_some()
        );
        assert_eq!(state.actions.get(&submenu_row), Some(&TOGGLE));
        assert_eq!(
            state.intent(&top_submenu_row),
            Some(ui::Intent::OpenSubmenu(PANELS))
        );
    }
}
