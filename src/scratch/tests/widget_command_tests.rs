use super::*;

#[test]
fn widget_trigger_binding_activates_from_non_command_element() {
    let mut app = Runtime::new(SourceState::default())
        .commands(|commands| {
            commands.register::<RecordSource>(command::Spec::new("Record"));
        })
        .responders(|responders| {
            responders.app().target::<RecordSource>();
        })
        .started(|cx| {
            cx.open_window(window::Options::new("Widget"));
        })
        .view(|_, _| {
            widget::view(|ui| {
                ui.add(
                    widget::Element::new()
                        .label("Record")
                        .trigger::<RecordSource>(()),
                );
            })
        });

    app.start();

    let window = app.session().windows()[0].id();
    let projected = app.present(window).expect("window should have a view");
    let command = projected
        .command::<RecordSource>()
        .expect("trigger-bound element should collect a command");
    assert_eq!(command.source(), context::Source::Button);

    let mut layout_engine = layout::Engine::new();
    let layout = layout::Layout::compose(
        &projected,
        geometry::Size::new(320, 120),
        &mut layout_engine,
    );
    let hit = layout
        .hit_test(geometry::Point::new(10, 10))
        .expect("trigger-bound element should be hit");

    assert_eq!(hit.frame().role(), view::Role::Panel);
    assert_eq!(
        hit.target()
            .expect("trigger-bound element should expose a command target")
            .kind(),
        interaction::Kind::Command
    );

    app.handle_view(
        window,
        hit.action()
            .expect("trigger-bound element should expose a command action")
            .clone(),
    )
    .expect("trigger-bound element should activate");

    assert_eq!(app.state().sources, vec![context::Source::Button]);
}

#[test]
fn widget_button_trigger_hit_tests_as_button_and_invokes_command() {
    let mut app = Runtime::new(SourceState::default())
        .commands(|commands| {
            commands.register::<RecordSource>(command::Spec::new("Record"));
        })
        .responders(|responders| {
            responders.app().target::<RecordSource>();
        })
        .started(|cx| {
            cx.open_window(window::Options::new("Button"));
        })
        .view(|_, _| {
            widget::view(|ui| {
                ui.button(widget::Button::new("Record").trigger::<RecordSource>(()));
            })
        });

    app.start();

    let window = app.session().windows()[0].id();
    let projected = app.present(window).expect("window should have a view");
    let mut layout_engine = layout::Engine::new();
    let layout = layout::Layout::compose(
        &projected,
        geometry::Size::new(320, 120),
        &mut layout_engine,
    );
    let hit = layout
        .hit_test(geometry::Point::new(10, 10))
        .expect("button should be hit");

    assert_eq!(hit.frame().role(), view::Role::Button);
    assert_eq!(
        hit.target()
            .expect("button should expose a command target")
            .kind(),
        interaction::Kind::Command
    );

    app.handle_view(
        window,
        hit.action()
            .expect("button should expose a command action")
            .clone(),
    )
    .expect("button trigger should activate");

    assert_eq!(app.state().sources, vec![context::Source::Button]);
}

#[test]
fn hidden_command_bound_widgets_are_pruned_after_resolution() {
    let mut app = Runtime::new(SourceState::default())
        .started(|cx| {
            cx.open_window(window::Options::new("Hidden"));
        })
        .view(|_, _| {
            widget::view(|ui| {
                ui.menu_bar(|ui| {
                    ui.menu("menu.hidden", "Actions", |ui| {
                        ui.add(widget::Command::<HiddenRecordSource>::menu());
                    });
                });
                ui.button(widget::Button::new("Hidden").trigger::<HiddenRecordSource>(()));
            })
        });

    app.start();

    let window = app.session().windows()[0].id();
    let projected = app.present(window).expect("window should have a view");

    assert!(projected.command::<HiddenRecordSource>().is_none());
    assert!(projected.buttons().is_empty());
    assert_eq!(projected.menus().len(), 1);

    let mut layout_engine = layout::Engine::new();
    let layout = layout::Layout::compose(
        &projected,
        geometry::Size::new(320, 120),
        &mut layout_engine,
    );

    assert!(layout.find_role(view::Role::Button).is_empty());
    assert!(layout.find_role(view::Role::Command).is_empty());

    let menu = projected.menus()[0]
        .menu_action()
        .expect("menu should still be visible");
    app.handle_view(window, menu)
        .expect("menu action should be handled");
    let projected = app
        .present(window)
        .expect("window should still have a view");

    assert_eq!(projected.popups().len(), 1);
    assert!(projected.command::<HiddenRecordSource>().is_none());
}

#[test]
fn disabled_command_bound_widgets_are_visible_but_not_activating() {
    let mut app = Runtime::new(SourceState::default())
        .commands(|commands| {
            commands.register::<DisabledRecordSource>(command::Spec::new("Disabled"));
        })
        .responders(|responders| {
            responders.app().target::<DisabledRecordSource>();
        })
        .started(|cx| {
            cx.open_window(window::Options::new("Disabled"));
        })
        .view(|_, _| {
            widget::view(|ui| {
                ui.button(widget::Button::new("Disabled").trigger::<DisabledRecordSource>(()));
            })
        });

    app.start();

    let window = app.session().windows()[0].id();
    let projected = app.present(window).expect("window should have a view");
    let command = projected
        .command::<DisabledRecordSource>()
        .expect("disabled command should remain in presentation");

    assert!(!command.is_enabled());
    assert_eq!(projected.buttons().len(), 1);

    let mut layout_engine = layout::Engine::new();
    let layout = layout::Layout::compose(
        &projected,
        geometry::Size::new(320, 120),
        &mut layout_engine,
    );
    let hit = layout
        .hit_test(geometry::Point::new(10, 10))
        .expect("disabled button should still be hit-testable");

    assert_eq!(hit.frame().role(), view::Role::Button);
    assert_eq!(
        hit.target()
            .expect("disabled button should keep a stable command target")
            .kind(),
        interaction::Kind::Command
    );
    assert!(hit.action().is_none());

    app.pointer_down_at(
        window,
        geometry::Size::new(320, 120),
        geometry::Point::new(10, 10),
    )
    .expect("pointer down should be handled by the disabled control target");
    app.pointer_up_at(
        window,
        geometry::Size::new(320, 120),
        geometry::Point::new(10, 10),
    )
    .expect("pointer up should not invoke the disabled command");

    assert!(app.state().sources.is_empty());
}

#[test]
fn command_bound_controls_use_node_identity_for_press_release_matching() {
    let mut app = Runtime::new(CloneCountState::default())
        .commands(|commands| {
            commands.register::<OpenNamed>(command::Spec::new("Open Named"));
        })
        .responders(|responders| {
            responders.app().target::<OpenNamed>();
        })
        .started(|cx| {
            cx.open_window(window::Options::new("Command Args"));
        })
        .view(|_, _| {
            widget::view(|ui| {
                ui.column(|ui| {
                    ui.button(
                        widget::Button::new("First").trigger::<OpenNamed>("first".to_owned()),
                    );
                    ui.button(
                        widget::Button::new("Second").trigger::<OpenNamed>("second".to_owned()),
                    );
                });
            })
        });

    app.start();

    let window = app.session().windows()[0].id();
    let size = geometry::Size::new(240, 120);
    app.present(window).expect("window should have a view");

    app.pointer_down_at(window, size, geometry::Point::new(10, 10))
        .expect("first button should receive pointer down");
    app.pointer_up_at(window, size, geometry::Point::new(10, 40))
        .expect("release over a different button should be handled without activation");

    assert_eq!(app.state().value, 0);

    app.pointer_down_at(window, size, geometry::Point::new(10, 40))
        .expect("second button should receive pointer down");
    app.pointer_up_at(window, size, geometry::Point::new(10, 40))
        .expect("second button release should activate");

    assert_eq!(app.state().value, "second".len());
}
