use crate::animation;
use crate::app::{state::WindowState, text_input};
use crate::geometry::area;
use crate::{action, paint, text, ui, window};

pub fn compose<T>(
    window: window::Id,
    tree: &ui::Tree,
    state: &mut WindowState,
    actions: &mut action::Registry<T>,
    text_engine: &mut text::Engine,
    logical_area: area::Logical,
    frame: animation::Frame,
) -> paint::Scene {
    let mut scene = paint::Scene::new();
    text_input::sync_session(state);

    if let Some(composition) = tree.compose(
        window,
        logical_area,
        actions,
        state.floating.surfaces(),
        text_engine,
    ) {
        state.open_menu = composition.open_menu();
        state.open_submenu = composition.open_submenu();
        state.composition = Some(composition);
        state.sync_menu_focus_scopes();
        state.clear_stale_focus();
        state.clear_stale_command_subject();
        text_input::sync_session(state);
        state.update_command_scope_captures(window);
        state.sync_text_field_states(text_engine);
        text_input::publish_action_states(state, actions, window);
        state.focus_first_floating_row(actions, window);
        state.sync_scroll_projections(text_engine, frame.now());
        state.refine_idle_scroll_models(text_engine, frame.now());
        let command_subject = state.command_context(window);

        let interaction = ui::Interaction::new(
            state.hovered.clone(),
            state.focused_path(),
            state.pressed.clone(),
        )
        .with_text_editing_target(text_input::editing_target(state))
        .with_focus_visibility(state.focus_visibility())
        .with_command_subject(command_subject)
        .with_command_scope_captures(state.command_scope_captures.clone())
        .with_open_menu(state.open_menu)
        .with_open_submenu(state.open_submenu)
        .with_pointer_position(state.pointer.position())
        .with_pointer_capture(state.pointer_capture.clone())
        .with_text_drop_caret(state.text_drop_caret())
        .with_drag_drop_operation(state.drag_drop.resolved_operation())
        .with_cursor_overlay(state.drag_drop.cursor_overlay());

        if let Some(composition) = state.composition.as_ref() {
            composition.paint_at(
                actions,
                window,
                interaction,
                &state.text_field_states,
                text_engine,
                frame,
                Some(&state.scroll),
                &mut scene,
            );
        }
    } else {
        state.composition = None;
        text_input::sync_session(state);
        state.sync_text_field_states(text_engine);
        state.clear_focus();
        state.clear_command_subject();
        state.command_scope_captures.clear();
        state.scroll.clear();
    }

    scene
}

#[cfg(test)]
mod tests {
    use crate::geometry::{area, point};
    use crate::widget;
    use crate::widget::menu;
    use crate::{Action, layout, paint, text};

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

    fn compose<T>(
        window: window::Id,
        tree: &ui::Tree,
        state: &mut WindowState,
        actions: &mut action::Registry<T>,
        logical_area: area::Logical,
    ) -> paint::Scene {
        let mut text_engine = text::Engine::new();

        super::compose(
            window,
            tree,
            state,
            actions,
            &mut text_engine,
            logical_area,
            crate::animation::Frame::new(std::time::Instant::now(), None),
        )
    }

    fn open_menu_surface(
        state: &mut WindowState,
        window: window::Id,
        menu: menu::Id,
        subject: ui::Path,
    ) {
        state.floating.open_top_menu(
            menu,
            action::Context::path(window, subject),
            action::Source::Pointer,
            ui::floating::FocusPolicy::PreserveCurrentFocus,
        );
        state.sync_open_menu_mirrors();
    }

    fn open_submenu_surface(
        state: &mut WindowState,
        window: window::Id,
        menu: menu::Id,
        submenu: menu::Id,
        subject: ui::Path,
    ) {
        open_menu_surface(state, window, menu, subject.clone());
        state.floating.show_submenu(
            submenu,
            action::Context::path(window, subject),
            action::Source::Pointer,
        );
        state.sync_open_menu_mirrors();
    }

    #[test]
    fn compose_updates_state_and_preserves_paint_order() {
        let window = window::Id::new(1);
        let mut state = WindowState::default();
        let mut registry = action::Registry::<()>::new();
        let mut tree = ui::Tree::new();

        registry.register(Action::new(CLICK, "Click"));
        tree.set_root(
            widget::panel(ROOT)
                .with_background(paint::Color::BLACK)
                .with_child(
                    widget::button(CHILD, CLICK)
                        .with_size(layout::Size::Fixed(10.0), layout::Size::Fixed(10.0)),
                ),
        );

        let scene = compose(
            window,
            &tree,
            &mut state,
            &mut registry,
            area::logical(100.0, 100.0),
        );
        let composition = state.composition.as_ref().expect("composition");

        assert!(state.composition.is_some());
        assert_eq!(
            composition.action(&ui::Path::new([ROOT, CHILD])),
            Some(CLICK)
        );
        assert_eq!(
            composition.command_subject(&ui::Path::new([ROOT, CHILD])),
            ui::CommandSubject::Origin
        );
        assert!(composition.responder_map().is_empty());
        assert!(composition.interactivity(&ui::Path::from(ROOT)).is_some());
        assert_eq!(composition.focus_order(), &[ui::Path::new([ROOT, CHILD])]);
        assert_eq!(scene.items().len(), 2);
    }

    #[test]
    fn compose_clears_stale_focused_paths_after_tree_rebuild() {
        let window = window::Id::new(1);
        let mut state = WindowState {
            focus: crate::app::state::FocusState::focused(crate::app::state::Focus::new(
                ui::Path::new([ROOT, CHILD]),
                ui::focus::Reason::Keyboard,
                ui::focus::Visibility::Visible,
            )),
            ..WindowState::default()
        };
        let mut registry = action::Registry::<()>::new();
        let mut tree = ui::Tree::new();

        tree.set_root(
            widget::panel(ROOT).with_child(
                widget::button(OTHER, CLICK)
                    .with_responder(CLICK)
                    .with_size(layout::Size::Fixed(10.0), layout::Size::Fixed(10.0)),
            ),
        );

        compose(
            window,
            &tree,
            &mut state,
            &mut registry,
            area::logical(100.0, 100.0),
        );

        assert_eq!(state.focused_path(), None);
    }

    #[test]
    fn compose_clears_stale_command_subject_after_tree_rebuild() {
        let window = window::Id::new(1);
        let mut state = WindowState {
            command_subject: Some(action::Scope::Path(ui::Path::new([ROOT, CHILD]))),
            ..WindowState::default()
        };
        let mut registry = action::Registry::<()>::new();
        let mut tree = ui::Tree::new();

        tree.set_root(
            widget::panel(ROOT).with_child(
                widget::button(OTHER, CLICK)
                    .with_size(layout::Size::Fixed(10.0), layout::Size::Fixed(10.0)),
            ),
        );

        compose(
            window,
            &tree,
            &mut state,
            &mut registry,
            area::logical(100.0, 100.0),
        );

        assert_eq!(state.command_subject, None);
    }

    #[test]
    fn compose_stores_responder_paths() {
        let window = window::Id::new(1);
        let mut state = WindowState::default();
        let mut registry = action::Registry::<()>::new();
        let mut tree = ui::Tree::new();

        tree.set_root(
            widget::panel(ROOT).with_child(
                ui::Node::leaf(CHILD)
                    .with_responder(action::SELECT_ALL)
                    .with_size(layout::Size::Fixed(10.0), layout::Size::Fixed(10.0)),
            ),
        );

        compose(
            window,
            &tree,
            &mut state,
            &mut registry,
            area::logical(100.0, 100.0),
        );
        let composition = state.composition.as_ref().expect("composition");

        assert_eq!(
            composition.responders(&ui::Path::new([ROOT, CHILD])),
            Some(&[action::SELECT_ALL][..])
        );
    }

    #[test]
    fn compose_publishes_responder_binding_state() {
        let window = window::Id::new(1);
        let mut state = WindowState::default();
        let mut registry = action::Registry::<()>::new();
        let mut tree = ui::Tree::new();
        let path = ui::Path::new([ROOT, CHILD]);
        let binding = action::Binding::new(action::SELECT_ALL)
            .enabled(false)
            .active(true)
            .busy(true);

        registry.register(Action::new(action::SELECT_ALL, "Select All"));
        tree.set_root(
            widget::panel(ROOT).with_child(
                ui::Node::leaf(CHILD)
                    .with_responder_binding(binding)
                    .with_size(layout::Size::Fixed(10.0), layout::Size::Fixed(10.0)),
            ),
        );

        compose(
            window,
            &tree,
            &mut state,
            &mut registry,
            area::logical(100.0, 100.0),
        );
        let composition = state.composition.as_ref().expect("composition");

        assert_eq!(
            composition.responders(&path),
            Some(&[action::SELECT_ALL][..])
        );
        assert_eq!(composition.responder_bindings(&path), Some(&[binding][..]));
        assert_eq!(
            registry.configured_state(action::SELECT_ALL, action::Context::path(window, path)),
            binding.state().expect("projected binding state")
        );
    }

    #[test]
    fn compose_disables_text_commands_without_text_editing_focus() {
        let window = window::Id::new(1);
        let path = ui::Path::new([ROOT, CHILD]);
        let mut state = WindowState::default();
        let mut registry = action::Registry::<()>::new();
        let mut tree = ui::Tree::new();

        registry.register(Action::new(action::SELECT_ALL, "Select All"));
        registry.register(Action::new(action::PASTE, "Paste"));
        tree.set_root(
            widget::panel(ROOT)
                .with_child(widget::text_field(CHILD, text::Buffer::from_text("hello"))),
        );

        compose(
            window,
            &tree,
            &mut state,
            &mut registry,
            area::logical(100.0, 100.0),
        );

        assert!(
            !registry
                .configured_state(
                    action::SELECT_ALL,
                    action::Context::path(window, path.clone())
                )
                .is_enabled()
        );
        assert!(
            !registry
                .configured_state(action::PASTE, action::Context::path(window, path))
                .is_enabled()
        );
    }

    #[test]
    fn compose_enables_text_commands_for_focused_text_field() {
        let window = window::Id::new(1);
        let path = ui::Path::new([ROOT, CHILD]);
        let mut state = WindowState {
            focus: crate::app::state::FocusState::focused(crate::app::state::Focus::new(
                path.clone(),
                ui::focus::Reason::Keyboard,
                ui::focus::Visibility::Visible,
            )),
            ..WindowState::default()
        };
        let mut registry = action::Registry::<()>::new();
        let mut tree = ui::Tree::new();

        registry.register(Action::new(action::SELECT_ALL, "Select All"));
        registry.register(Action::new(action::PASTE, "Paste"));
        tree.set_root(
            widget::panel(ROOT)
                .with_child(widget::text_field(CHILD, text::Buffer::from_text("hello"))),
        );

        compose(
            window,
            &tree,
            &mut state,
            &mut registry,
            area::logical(100.0, 100.0),
        );

        assert!(
            registry
                .configured_state(
                    action::SELECT_ALL,
                    action::Context::path(window, path.clone())
                )
                .is_enabled()
        );
        assert!(
            registry
                .configured_state(action::PASTE, action::Context::path(window, path))
                .is_enabled()
        );
    }

    #[test]
    fn open_menu_projects_disabled_select_all_when_text_is_fully_selected() {
        let window = window::Id::new(1);
        let field = ui::Path::new([ROOT, CHILD]);
        let mut engine = text::Engine::new();
        let mut buffer = text::Buffer::from_text("hello");
        let mut state = WindowState {
            focus: crate::app::state::FocusState::focused(crate::app::state::Focus::new(
                field.clone(),
                ui::focus::Reason::Keyboard,
                ui::focus::Visibility::Visible,
            )),
            ..WindowState::default()
        };
        let mut registry = action::Registry::<()>::new();
        let mut tree = ui::Tree::new();

        engine.apply_text_edit(&mut buffer, text::Edit::SelectAll);
        registry.register(Action::new(action::SELECT_ALL, "Select All"));
        tree.set_root(
            widget::panel(ROOT)
                .with_child(widget::menu_bar(
                    MENU_BAR,
                    menu::Bar::new().menu(
                        menu::Menu::new(FILE, "File").section(
                            menu::Section::new().item(menu::Item::new(action::SELECT_ALL)),
                        ),
                    ),
                ))
                .with_child(widget::text_field(CHILD, buffer)),
        );

        compose(
            window,
            &tree,
            &mut state,
            &mut registry,
            area::logical(300.0, 180.0),
        );
        open_menu_surface(&mut state, window, FILE, field.clone());
        compose(
            window,
            &tree,
            &mut state,
            &mut registry,
            area::logical(300.0, 180.0),
        );

        let row = ui::Path::new([ROOT, widget::MENU_POPUP, ui::Id::new("__menu_row_00")]);
        let composition = state.composition.as_ref().expect("composition");

        assert_eq!(composition.action(&row), Some(action::SELECT_ALL));
        assert!(
            !registry
                .configured_state(action::SELECT_ALL, action::Context::path(window, field))
                .is_enabled()
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
        let mut registry = action::Registry::<()>::new();
        let mut tree = ui::Tree::new();

        tree.set_root(
            widget::panel(ROOT)
                .with_child(
                    ui::Node::leaf(CHILD)
                        .with_responder(action::SELECT_ALL)
                        .with_size(layout::Size::Fixed(10.0), layout::Size::Fixed(10.0)),
                )
                .with_child(
                    widget::panel(OTHER)
                        .with_command_scope()
                        .with_size(layout::Size::Fixed(10.0), layout::Size::Fixed(10.0)),
                ),
        );

        compose(
            window,
            &tree,
            &mut state,
            &mut registry,
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
        let mut registry = action::Registry::<()>::new();
        let mut tree = ui::Tree::new();

        tree.set_root(widget::panel(ROOT));

        compose(
            window,
            &tree,
            &mut state,
            &mut registry,
            area::logical(100.0, 100.0),
        );

        assert!(state.command_scope_captures.is_empty());
    }

    #[test]
    fn compose_injects_open_menu_popup_from_menu_bar() {
        let window = window::Id::new(1);
        let subject = ui::Path::new([ROOT, CHILD]);
        let mut state = WindowState {
            command_subject: Some(action::Scope::Path(subject.clone())),
            ..WindowState::default()
        };
        let mut registry = action::Registry::<()>::new();
        let mut tree = ui::Tree::new();

        registry.register(Action::new(action::SELECT_ALL, "Select All"));
        registry.set_state(
            action::SELECT_ALL,
            action::Context::path(window, subject.clone()),
            action::State::enabled(),
        );
        open_menu_surface(&mut state, window, FILE, subject.clone());
        tree.set_root(
            widget::panel(ROOT)
                .with_child(widget::menu_bar(
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
            &mut registry,
            area::logical(300.0, 180.0),
        );
        let theme = crate::theme::Theme::default_dark();

        let row = ui::Path::new([ROOT, widget::MENU_POPUP, ui::Id::new("__menu_row_00")]);
        let scope = ui::Path::new([ROOT, widget::MENU_POPUP]);
        let composition = state.composition.as_ref().expect("composition");
        let layout = composition.layout();

        assert!(layout.find_path(&scope).is_some());
        let popup_rect = layout
            .find_path(&scope)
            .expect("menu popup root layout")
            .rect();
        let background_hit = layout.hit_test_where(
            point::logical(popup_rect.origin.x() + 1.0, popup_rect.origin.y() + 1.0),
            |path| {
                composition
                    .interactivity(path)
                    .is_some_and(|interactivity| interactivity.hit_test())
            },
        );

        assert_eq!(background_hit, Some(scope.clone()));
        assert!(
            composition
                .interactivity(&scope)
                .is_some_and(|interactivity| interactivity.hit_test()
                    && !interactivity.focusable()
                    && !interactivity.actionable())
        );
        assert_eq!(composition.action(&row), Some(action::SELECT_ALL));
        assert_eq!(
            composition.command_subject(&row),
            ui::CommandSubject::Captured
        );
        assert_eq!(
            state.command_scope_captures.get(&scope),
            Some(&action::Context::path(window, subject))
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
    fn pointer_opened_menu_preserves_text_field_focus_after_compose() {
        let window = window::Id::new(1);
        let field = ui::Path::new([ROOT, CHILD]);
        let mut state = WindowState {
            focus: crate::app::state::FocusState::focused(crate::app::state::Focus::new(
                field.clone(),
                ui::focus::Reason::Pointer,
                ui::focus::Visibility::Visible,
            )),
            command_subject: Some(action::Scope::Path(field.clone())),
            ..WindowState::default()
        };
        let mut registry = action::Registry::<()>::new();
        let mut tree = ui::Tree::new();

        registry.register(Action::new(action::SELECT_ALL, "Select All"));
        registry.set_state(
            action::SELECT_ALL,
            action::Context::path(window, field.clone()),
            action::State::enabled(),
        );
        open_menu_surface(&mut state, window, FILE, field.clone());
        tree.set_root(
            widget::panel(ROOT)
                .with_child(widget::menu_bar(
                    MENU_BAR,
                    menu::Bar::new().menu(
                        menu::Menu::new(FILE, "File").section(
                            menu::Section::new().item(menu::Item::new(action::SELECT_ALL)),
                        ),
                    ),
                ))
                .with_child(
                    widget::text_field(CHILD, text::Buffer::from_text("hello"))
                        .with_responder(action::SELECT_ALL),
                ),
        );

        compose(
            window,
            &tree,
            &mut state,
            &mut registry,
            area::logical(300.0, 180.0),
        );

        assert_eq!(state.focused_path(), Some(field));
        assert_eq!(state.focus_visibility(), ui::focus::Visibility::Visible);
    }

    #[test]
    fn keyboard_opened_menu_focuses_first_enabled_row() {
        let window = window::Id::new(1);
        let field = ui::Path::new([ROOT, CHILD]);
        let row = ui::Path::new([ROOT, widget::MENU_POPUP, ui::Id::new("__menu_row_00")]);
        let mut state = WindowState {
            focus: crate::app::state::FocusState::focused(crate::app::state::Focus::new(
                field.clone(),
                ui::focus::Reason::Keyboard,
                ui::focus::Visibility::Visible,
            )),
            command_subject: Some(action::Scope::Path(field.clone())),
            ..WindowState::default()
        };
        let mut registry = action::Registry::<()>::new();
        let mut tree = ui::Tree::new();

        registry.register(Action::new(action::SELECT_ALL, "Select All"));
        registry.set_state(
            action::SELECT_ALL,
            action::Context::path(window, field.clone()),
            action::State::enabled(),
        );
        tree.set_root(
            widget::panel(ROOT)
                .with_child(widget::menu_bar(
                    MENU_BAR,
                    menu::Bar::new().menu(
                        menu::Menu::new(FILE, "File").section(
                            menu::Section::new().item(menu::Item::new(action::SELECT_ALL)),
                        ),
                    ),
                ))
                .with_child(
                    widget::text_field(CHILD, text::Buffer::from_text("hello"))
                        .with_responder(action::SELECT_ALL),
                ),
        );

        compose(
            window,
            &tree,
            &mut state,
            &mut registry,
            area::logical(300.0, 180.0),
        );
        assert!(state.toggle_menu(FILE, &registry, window, action::Source::Keyboard));
        compose(
            window,
            &tree,
            &mut state,
            &mut registry,
            area::logical(300.0, 180.0),
        );

        assert_eq!(state.focused_path(), Some(row));
        assert_eq!(state.focus_visibility(), ui::focus::Visibility::Visible);
    }

    #[test]
    fn focused_open_menu_row_lowers_focus_background() {
        let window = window::Id::new(1);
        let subject = ui::Path::new([ROOT, CHILD]);
        let row = ui::Path::new([ROOT, widget::MENU_POPUP, ui::Id::new("__menu_row_00")]);
        let mut state = WindowState {
            command_subject: Some(action::Scope::Path(subject.clone())),
            focus: crate::app::state::FocusState::focused(crate::app::state::Focus::new(
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
            action::Context::path(window, subject.clone()),
            action::State::enabled(),
        );
        open_menu_surface(&mut state, window, FILE, subject.clone());
        tree.set_root(
            widget::panel(ROOT)
                .with_child(widget::menu_bar(
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
            &mut registry,
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
            command_subject: Some(action::Scope::Path(subject.clone())),
            ..WindowState::default()
        };
        let mut registry = action::Registry::<()>::new();
        let mut tree = ui::Tree::new();

        registry.register(Action::new(TOGGLE, "Toggle Preview"));
        registry.set_state(
            TOGGLE,
            action::Context::path(window, subject.clone()),
            action::State::active(),
        );
        open_menu_surface(&mut state, window, VIEW, subject.clone());
        tree.set_root(
            widget::panel(ROOT)
                .with_child(widget::menu_bar(
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
            &mut registry,
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
        open_submenu_surface(&mut state, window, VIEW, PANELS, subject.clone());
        tree.set_root(
            widget::panel(ROOT)
                .with_child(widget::menu_bar(
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
            &mut registry,
            area::logical(360.0, 220.0),
        );

        let submenu_popup = ui::Path::new([ROOT, widget::MENU_SUBMENU_POPUP]);
        let submenu_row = ui::Path::new([
            ROOT,
            widget::MENU_SUBMENU_POPUP,
            ui::Id::new("__menu_row_00"),
        ]);
        let top_submenu_row =
            ui::Path::new([ROOT, widget::MENU_POPUP, ui::Id::new("__menu_row_00")]);
        let composition = state.composition.as_ref().expect("composition");

        assert!(composition.layout().find_path(&submenu_popup).is_some());
        assert_eq!(composition.action(&submenu_row), Some(TOGGLE));
        assert_eq!(
            state.intent(&top_submenu_row),
            Some(ui::Intent::OpenSubmenu(PANELS))
        );
    }
}
