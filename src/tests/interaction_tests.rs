use super::*;
use crate::virtual_list;

#[derive(Clone)]
struct ReplacingInteractionState {
    level: f64,
    invocations: usize,
    visible: bool,
    available: bool,
}

impl State for ReplacingInteractionState {}

impl Target<SetLevel> for ReplacingInteractionState {
    fn state(&self, _: &f64, _: &Context) -> command::State {
        if self.available {
            command::State::enabled()
        } else {
            command::State::disabled()
        }
    }

    fn invoke(&mut self, level: f64, _: &mut Context) -> Response<f64> {
        self.level = level;
        self.invocations += 1;
        Response::changed(level)
    }
}

#[derive(Clone, Default)]
struct HiddenLocalTarget;

#[derive(Clone, Default)]
struct DisabledLocalTarget;

#[derive(Clone, Default)]
struct HiddenLocalRouteState {
    local: HiddenLocalTarget,
    app_invocations: usize,
}

impl State for HiddenLocalRouteState {}

impl Target<RecordSource> for HiddenLocalTarget {
    fn state(&self, _: &(), _: &Context) -> command::State {
        command::State::hidden()
    }

    fn invoke(&mut self, _: (), _: &mut Context) -> Response<()> {
        unreachable!("a hidden exact target must not invoke")
    }
}

impl Target<RecordSource> for DisabledLocalTarget {
    fn state(&self, _: &(), _: &Context) -> command::State {
        command::State::disabled()
    }

    fn invoke(&mut self, _: (), _: &mut Context) -> Response<()> {
        unreachable!("a disabled broad owner must not invoke")
    }
}

impl Target<RecordSource> for HiddenLocalRouteState {
    fn state(&self, _: &(), _: &Context) -> command::State {
        command::State::enabled()
    }

    fn invoke(&mut self, _: (), _: &mut Context) -> Response<()> {
        self.app_invocations += 1;
        Response::changed(())
    }
}

struct ContextRow;

impl Command for ContextRow {
    type Args = virtual_list::Key;
    type Output = ();

    const NAME: &'static str = "test.context_row";
}

struct ContextToggle;

impl Command for ContextToggle {
    type Args = (crate::table::Cell, bool);
    type Output = ();

    const NAME: &'static str = "test.context_toggle";
}

struct CommitContextName;

impl Command for CommitContextName {
    type Args = (crate::table::Cell, String);
    type Output = ();

    const NAME: &'static str = "test.commit_context_name";
}

#[derive(Clone, Default)]
struct TableContextState {
    visible: bool,
    name: String,
    row_invoked: Option<virtual_list::Key>,
    toggled: Option<(crate::table::Cell, bool)>,
}

impl State for TableContextState {}

impl Target<ContextRow> for TableContextState {
    fn state(&self, _: &virtual_list::Key, _: &Context) -> command::State {
        command::State::enabled()
    }

    fn invoke(&mut self, key: virtual_list::Key, _: &mut Context) -> Response<()> {
        self.row_invoked = Some(key);
        Response::changed(())
    }
}

impl Target<ContextToggle> for TableContextState {
    fn state(&self, _: &(crate::table::Cell, bool), _: &Context) -> command::State {
        command::State::enabled()
    }

    fn invoke(&mut self, args: (crate::table::Cell, bool), _: &mut Context) -> Response<()> {
        self.toggled = Some(args);
        Response::changed(())
    }
}

impl Target<CommitContextName> for TableContextState {
    fn state(&self, _: &(crate::table::Cell, String), _: &Context) -> command::State {
        command::State::enabled()
    }

    fn invoke(&mut self, (_, name): (crate::table::Cell, String), _: &mut Context) -> Response<()> {
        self.name = name;
        Response::changed(())
    }
}

#[derive(Clone)]
struct ContextRecord {
    name: String,
    enabled: bool,
}

#[derive(Clone)]
struct PinnedContextState {
    keys: Vec<u64>,
}

impl State for PinnedContextState {}

impl Target<ContextRow> for PinnedContextState {
    fn state(&self, _: &virtual_list::Key, _: &Context) -> command::State {
        command::State::enabled()
    }

    fn invoke(&mut self, _: virtual_list::Key, _: &mut Context) -> Response<()> {
        Response::output(())
    }
}

fn table_context_app() -> Runtime<TableContextState, (), View> {
    Runtime::new(TableContextState {
        visible: true,
        name: "Seventeen".to_owned(),
        ..TableContextState::default()
    })
    .commands(|commands| {
        commands.install(document::Editing::standard());
        commands
            .register::<ContextRow>(command::Spec::new("Open row"))
            .register::<ContextToggle>(command::Spec::new("Toggle enabled"))
            .register::<CommitContextName>(command::Spec::new("Commit name"));
    })
    .responders(|responders| {
        responders
            .app()
            .target::<ContextRow>()
            .target::<ContextToggle>()
            .target::<CommitContextName>();
    })
    .view(|state, _| {
        let len = usize::from(state.visible);
        let name = state.name.clone();
        let source = crate::table::Source::new(
            len,
            |_| virtual_list::Key::new(17),
            move |key| (len == 1 && key == virtual_list::Key::new(17)).then_some(0),
            move |_| ContextRecord {
                name: name.clone(),
                enabled: false,
            },
        );
        let columns = vec![
            crate::table::Column::text(
                "name",
                "Name",
                view::Dimension::fixed(120),
                |record: &ContextRecord| &record.name,
            )
            .editable::<CommitContextName>(|cell, value| (cell, value))
            .unsortable()
            .build(),
            crate::table::Column::boolean(
                "enabled",
                "Enabled",
                view::Dimension::fixed(80),
                |record: &ContextRecord| &record.enabled,
            )
            .toggle::<ContextToggle>(|cell, value| (cell, value))
            .unsortable()
            .build(),
        ];
        widget::view(|ui| {
            ui.add(
                crate::Table::typed("context.table", 24, columns, source)
                    .context_rows::<ContextRow>(|key| key)
                    .width(view::Dimension::fixed(200))
                    .height(view::Dimension::fixed(90)),
            );
        })
    })
    .started(|cx| {
        cx.open_window(window::Options::new("Table context"));
    })
}

fn contextual_binding_app() -> Runtime<ReplacingInteractionState, (), View> {
    Runtime::new(ReplacingInteractionState {
        level: 0.0,
        invocations: 0,
        visible: true,
        available: true,
    })
    .commands(|commands| {
        commands.register::<SetLevel>(command::Spec::new("Set level"));
    })
    .responders(|responders| {
        responders.app().target::<SetLevel>();
    })
    .view(|state, _| {
        widget::view(|ui| {
            ui.column(|ui| {
                if state.visible {
                    ui.add(widget::Binding::<SetLevel>::button_with_args(42.0));
                }
                ui.label("Unmarked");
            });
        })
    })
    .started(|cx| {
        cx.open_window(window::Options::new("Context menu"));
    })
}

#[test]
fn contextual_binding_derives_one_existing_menu_row_and_preserves_arguments() {
    let mut app = contextual_binding_app();
    app.start();
    let window = app.session().windows()[0].id();
    let size = geometry::Size::new(320, 180);
    let initial = app.show_scene(window, size).expect("view should render");
    let owner = labeled_frame(initial.layout(), view::Role::Binding, "Set level");
    let focus_before = app.session().focused(window);

    let opened = app
        .open_context_menu_at(window, size, frame_point(owner))
        .expect("context request should be handled");

    assert!(opened.is_handled());
    assert_eq!(app.session().focused(window), focus_before);
    assert!(
        app.session()
            .interaction(window)
            .and_then(Interaction::open_menu)
            .is_some_and(interaction::Menu::is_context)
    );

    let projected = app.present(window).expect("context menu should project");
    let menu_bindings = projected
        .bindings()
        .into_iter()
        .filter(|binding| binding.source() == context::Source::Menu)
        .collect::<Vec<_>>();
    assert_eq!(menu_bindings.len(), 1);
    let action = menu_bindings[0].action();

    app.handle_view(window, action)
        .expect("derived menu binding should invoke normally");

    assert_eq!(app.state().level, 42.0);
    assert_eq!(app.state().invocations, 1);
    assert_eq!(
        app.session()
            .interaction(window)
            .and_then(Interaction::open_menu),
        None
    );
}

#[test]
fn open_context_menu_reprojects_live_command_state() {
    let mut app = contextual_binding_app();
    app.start();
    let window = app.session().windows()[0].id();
    let size = geometry::Size::new(320, 180);
    let initial = app.show_scene(window, size).expect("view should render");
    let owner = labeled_frame(initial.layout(), view::Role::Binding, "Set level");
    app.open_context_menu_at(window, size, frame_point(owner))
        .expect("context menu should open");

    app.change(
        state::Reason::programmatic("disable-context-action"),
        |state| {
            state.available = false;
        },
    );
    let projected = app.present(window).expect("open menu should reproject");
    let action = projected
        .bindings()
        .into_iter()
        .find(|binding| binding.source() == context::Source::Menu)
        .expect("disabled context action should remain visible");

    assert!(!action.state().is_enabled());
    assert!(
        app.session()
            .interaction(window)
            .and_then(Interaction::open_menu)
            .is_some_and(interaction::Menu::is_context)
    );
}

#[test]
fn unmarked_space_does_not_scan_the_application_command_world() {
    let mut app = contextual_binding_app();
    app.start();
    let window = app.session().windows()[0].id();
    let size = geometry::Size::new(320, 180);
    let initial = app.show_scene(window, size).expect("view should render");
    let unmarked = labeled_frame(initial.layout(), view::Role::Label, "Unmarked");

    let outcome = app
        .open_context_menu_at(window, size, frame_point(unmarked))
        .expect("context request should be valid input");

    assert!(!outcome.is_handled());
    assert_eq!(
        app.session()
            .interaction(window)
            .and_then(Interaction::open_menu),
        None
    );
}

#[test]
fn explicitly_contextual_root_projects_its_bounded_application_targets() {
    let mut app = Runtime::new(SourceState::default())
        .commands(|commands| {
            commands.register::<RecordSource>(command::Spec::new("Application action"));
        })
        .responders(|responders| {
            responders.app().target::<RecordSource>();
        })
        .view(|_, _| {
            widget::view_node(widget::context_menu(
                widget::Root::new().child(widget::Label::new("Application canvas")),
            ))
        })
        .started(|cx| {
            cx.open_window(window::Options::new("Application context"));
        });
    app.start();
    let window = app.session().windows()[0].id();
    let size = geometry::Size::new(320, 180);
    let initial = app.show_scene(window, size).expect("root should render");
    let canvas = labeled_frame(initial.layout(), view::Role::Label, "Application canvas");

    app.open_context_menu_at(window, size, frame_point(canvas))
        .expect("explicit application context should open");
    let projected = app
        .present(window)
        .expect("application menu should project");
    let actions = projected
        .bindings()
        .into_iter()
        .filter(|binding| binding.source() == context::Source::Menu)
        .collect::<Vec<_>>();

    assert_eq!(actions.len(), 1);
    assert_eq!(
        actions[0].command_type(),
        std::any::TypeId::of::<RecordSource>()
    );
}

#[test]
fn removing_a_context_owner_prunes_the_shared_menu_session() {
    let mut app = contextual_binding_app();
    app.start();
    let window = app.session().windows()[0].id();
    let size = geometry::Size::new(320, 180);
    let initial = app.show_scene(window, size).expect("view should render");
    let owner = labeled_frame(initial.layout(), view::Role::Binding, "Set level");
    let captured_focus = session::Focus::text("captured-before-context-menu");
    app.handle_input(window, Input::focus(captured_focus))
        .expect("focus should be captured before the context menu opens");
    app.open_context_menu_at(window, size, frame_point(owner))
        .expect("context menu should open");
    assert_eq!(app.session().command_focus(window), Some(captured_focus));

    app.change(
        state::Reason::programmatic("remove-context-owner"),
        |state| {
            state.visible = false;
        },
    );
    app.show_scene(window, size)
        .expect("view should reconcile owner removal");

    assert_eq!(
        app.session()
            .interaction(window)
            .and_then(Interaction::open_menu),
        None
    );
    let live_focus = session::Focus::text("live-after-context-menu");
    app.handle_input(window, Input::focus(live_focus))
        .expect("live focus should be accepted after the menu is pruned");
    assert_eq!(
        app.session().command_focus(window),
        Some(live_focus),
        "pruning a contextual menu must retire its captured command focus"
    );
}

#[test]
fn nested_context_owners_form_one_broad_to_exact_inspection_path() {
    let mut app = Runtime::new(SourceState::default())
        .commands(|commands| {
            commands
                .register::<RecordSource>(command::Spec::new("Inner action"))
                .register::<DisabledRecordSource>(command::Spec::new("Outer action"));
        })
        .responders(|responders| {
            responders
                .object("outer", |state| state)
                .target::<DisabledRecordSource>();
            responders
                .object("inner", |state| state)
                .target::<RecordSource>();
        })
        .view(|_, _| {
            widget::view_node(widget::context_menu(
                widget::Element::new()
                    .id("outer")
                    .child(widget::context_menu(
                        widget::Element::new()
                            .id("inner")
                            .child(widget::Label::new("Nearest")),
                    )),
            ))
        })
        .started(|cx| {
            cx.open_window(window::Options::new("Nearest context"));
        });
    app.start();
    let window = app.session().windows()[0].id();
    let size = geometry::Size::new(320, 180);
    let initial = app.show_scene(window, size).expect("view should render");
    let nearest = labeled_frame(initial.layout(), view::Role::Label, "Nearest");
    let point = frame_point(nearest);
    let node = initial
        .layout()
        .context_node_at(point)
        .expect("nearest label should have context geometry");
    let owner = app
        .composition(window)
        .and_then(|composition| composition.context_path_for_node(node).pop())
        .expect("nearest label should inherit a contextual owner");
    assert_eq!(owner.responder(), Some(interaction::Id::new("inner")));

    let opened = app
        .open_context_menu_at(window, size, point)
        .expect("nearest context owner should open");
    assert!(opened.is_handled());
    let projected = app.present(window).expect("context menu should project");
    let menu_bindings = projected
        .bindings()
        .into_iter()
        .filter(|binding| binding.source() == context::Source::Menu)
        .collect::<Vec<_>>();

    assert_eq!(menu_bindings.len(), 2);
    assert_eq!(
        menu_bindings[0].command_type(),
        std::any::TypeId::of::<DisabledRecordSource>()
    );
    assert_eq!(
        menu_bindings[1].command_type(),
        std::any::TypeId::of::<RecordSource>()
    );
    let action = menu_bindings[1].action();
    app.handle_view(window, action)
        .expect("exact responder action should invoke");
    assert_eq!(app.state().sources, vec![context::Source::Menu]);
}

#[test]
fn disabled_broad_context_claim_consumes_the_exact_duplicate() {
    #[derive(Clone, Default)]
    struct ConsumptionState {
        broad: DisabledLocalTarget,
        exact_invocations: usize,
    }

    impl State for ConsumptionState {}

    impl Target<RecordSource> for ConsumptionState {
        fn state(&self, _: &(), _: &Context) -> command::State {
            command::State::enabled()
        }

        fn invoke(&mut self, _: (), _: &mut Context) -> Response<()> {
            self.exact_invocations += 1;
            Response::changed(())
        }
    }

    let mut app = Runtime::new(ConsumptionState::default())
        .commands(|commands| {
            commands.register::<RecordSource>(command::Spec::new("Same command"));
        })
        .responders(|responders| {
            responders
                .object("broad", |state| &mut state.broad)
                .target::<RecordSource>();
            responders
                .object("exact", |state| state)
                .target::<RecordSource>();
        })
        .view(|_, _| {
            widget::view_node(widget::context_menu(
                widget::Element::new()
                    .id("broad")
                    .child(widget::context_menu(
                        widget::Element::new()
                            .id("exact")
                            .child(widget::Label::new("Consumed")),
                    )),
            ))
        })
        .started(|cx| {
            cx.open_window(window::Options::new("Consumed context"));
        });
    app.start();
    let window = app.session().windows()[0].id();
    let size = geometry::Size::new(320, 180);
    let initial = app.show_scene(window, size).expect("view should render");
    let label = labeled_frame(initial.layout(), view::Role::Label, "Consumed");

    app.open_context_menu_at(window, size, frame_point(label))
        .expect("inspection context should open");
    let projected = app.present(window).expect("context menu should project");
    let actions = projected
        .bindings()
        .into_iter()
        .filter(|binding| binding.source() == context::Source::Menu)
        .collect::<Vec<_>>();

    assert_eq!(actions.len(), 1);
    assert!(!actions[0].state().is_enabled());
    assert_eq!(app.state().exact_invocations, 0);
}

#[test]
fn text_context_uses_the_existing_local_service_without_moving_focus() {
    let focus = session::Focus::text("context-text");
    let mut app = Runtime::new(SourceState::default())
        .commands(|commands| {
            commands.install(document::Editing::standard());
        })
        .view(move |_, _| widget::view_node(widget::TextBox::new("selectable text").focus(focus)))
        .started(|cx| {
            cx.open_window(window::Options::new("Text context"));
        });
    app.start();
    let window = app.session().windows()[0].id();
    let size = geometry::Size::new(320, 180);
    let initial = app
        .show_scene(window, size)
        .expect("text view should render");
    let text_box = initial
        .layout()
        .find_role(view::Role::TextBox)
        .into_iter()
        .next()
        .expect("text box should be laid out");

    app.open_context_menu_at(window, size, frame_point(text_box))
        .expect("text context should open");
    assert_eq!(app.session().focused(window), None);
    let projected = app.present(window).expect("text context should project");
    let commands = projected
        .bindings()
        .into_iter()
        .filter(|binding| binding.source() == context::Source::Menu)
        .map(view::Binding::command_name)
        .collect::<Vec<_>>();

    assert_eq!(
        commands,
        vec![
            "edit.select_all",
            "edit.copy",
            "edit.cut",
            "edit.delete",
            "edit.paste",
            "edit.undo",
            "edit.redo",
        ]
    );
    assert!(!commands.contains(&"document.apply_edit"));
    assert_eq!(app.session().focused(window), None);

    app.handle_input(window, Input::cancel())
        .expect("pointer context should close");
    app.handle_input(window, Input::focus(focus))
        .expect("text focus should be accepted");
    let keyboard = app
        .handle_input(
            window,
            Input::key_down(
                input::Key::F10,
                input::Modifiers::new(true, false, false, false),
            ),
        )
        .expect("Shift+F10 should request the same context menu");
    assert!(keyboard.is_handled());
    assert!(
        app.session()
            .interaction(window)
            .and_then(Interaction::open_menu)
            .is_some_and(interaction::Menu::is_context)
    );
    assert_eq!(app.session().focused(window), Some(focus));

    app.handle_input(window, Input::cancel())
        .expect("keyboard context should close");
    let menu_key = app
        .handle_input(
            window,
            Input::key_down(input::Key::ContextMenu, input::Modifiers::default()),
        )
        .expect("Menu key should request the same context menu");
    assert!(menu_key.is_handled());
    assert!(
        app.session()
            .interaction(window)
            .and_then(Interaction::open_menu)
            .is_some_and(interaction::Menu::is_context)
    );
}

#[test]
fn hidden_exact_target_does_not_fall_through_to_the_application() {
    let mut app = Runtime::new(HiddenLocalRouteState::default())
        .commands(|commands| {
            commands.register::<RecordSource>(command::Spec::new("Record"));
        })
        .responders(|responders| {
            responders.app().target::<RecordSource>();
            responders
                .object("hidden-local", |state| &mut state.local)
                .target::<RecordSource>();
        })
        .view(|_, _| {
            widget::view_node(widget::context_menu(
                widget::Element::new()
                    .id("hidden-local")
                    .child(widget::Label::new("Hidden local")),
            ))
        })
        .started(|cx| {
            cx.open_window(window::Options::new("Hidden context"));
        });
    app.start();
    let window = app.session().windows()[0].id();
    let size = geometry::Size::new(320, 180);
    let initial = app.show_scene(window, size).expect("view should render");
    let label = labeled_frame(initial.layout(), view::Role::Label, "Hidden local");

    let outcome = app
        .open_context_menu_at(window, size, frame_point(label))
        .expect("context request should remain valid input");

    assert!(!outcome.is_handled());
    assert_eq!(app.state().app_invocations, 0);
    assert_eq!(
        app.session()
            .interaction(window)
            .and_then(Interaction::open_menu),
        None
    );
}

#[test]
fn table_context_cells_override_row_actions_and_virtual_removal_prunes_the_menu() {
    let mut app = table_context_app();
    app.start();
    let window = app.session().windows()[0].id();
    let size = geometry::Size::new(320, 180);
    let initial = app.show_scene(window, size).expect("table should render");
    let key = virtual_list::Key::new(17);
    let name = crate::table::Cell::new("context.table".into(), key, "name".into());
    let enabled = crate::table::Cell::new("context.table".into(), key, "enabled".into());
    let row = initial
        .layout()
        .frames()
        .iter()
        .find(|frame| frame.table_row().is_some())
        .expect("virtual row should be materialized");
    assert_eq!(
        row.target(),
        None,
        "context-only rows must not become buttons"
    );
    let name_frame = initial
        .layout()
        .frames()
        .iter()
        .find(|frame| frame.table_cell() == Some(name) && frame.role() == view::Role::TextBox)
        .expect("text cell should be laid out");
    let context_node = initial
        .layout()
        .context_node_at(frame_point(name_frame))
        .expect("text cell should expose context geometry");
    let context_path = app
        .composition(window)
        .expect("table composition should exist")
        .context_path_for_node(context_node);
    let table_layer = context_path
        .iter()
        .position(|owner| owner.service() == view::ContextService::Table)
        .expect("table domain should own one service layer");
    let row_layer = context_path
        .iter()
        .position(|owner| owner.row().is_some())
        .expect("focal row should own one semantic layer");
    let cell_layer = context_path
        .iter()
        .position(|owner| owner.cell() == Some(name))
        .expect("cell identity should own one semantic layer");
    let member_layer = context_path
        .iter()
        .position(|owner| {
            owner.focus() == Some(session::Focus::table_cell(name))
                && owner.service() == view::ContextService::Text
        })
        .expect("exact text member should own one service layer");
    assert!(table_layer < row_layer && row_layer < cell_layer && cell_layer < member_layer);
    assert_eq!(context_path[table_layer].row(), None);
    assert_eq!(context_path[table_layer].cell(), None);
    assert_eq!(context_path[row_layer].cell(), None);
    assert_eq!(
        context_path[row_layer].service(),
        view::ContextService::None
    );
    assert_eq!(context_path[cell_layer].focus(), None);
    assert_eq!(
        context_path[cell_layer].service(),
        view::ContextService::None
    );
    assert_eq!(context_path[member_layer].cell(), None);

    app.open_context_menu_at(window, size, frame_point(name_frame))
        .expect("row context should open through an unmarked cell");
    let projected = app.present(window).expect("row menu should project");
    let actions = projected
        .bindings()
        .into_iter()
        .filter(|binding| binding.source() == context::Source::Menu)
        .collect::<Vec<_>>();
    assert_eq!(
        actions[0].command_type(),
        std::any::TypeId::of::<document::SelectAll>(),
        "the containing table domain is the first inspection section"
    );
    assert_eq!(
        actions[1].command_type(),
        std::any::TypeId::of::<ContextRow>(),
        "the focal row follows its containing table"
    );
    assert_eq!(
        actions
            .iter()
            .filter(|binding| {
                binding.command_type() == std::any::TypeId::of::<document::SelectAll>()
            })
            .count(),
        1,
        "the broad table claim consumes the text facet's Select All"
    );
    assert!(
        actions
            .iter()
            .any(|binding| { binding.command_type() == std::any::TypeId::of::<document::Copy>() })
    );
    assert!(
        actions.iter().all(|binding| {
            binding.command_type() != std::any::TypeId::of::<CommitContextName>()
        }),
        "the edit-commit input binding is not a contextual control action"
    );
    let context_panel = projected
        .root()
        .children()
        .iter()
        .find(|node| node.role() == view::Role::FloatingPanel)
        .expect("context menu should project one floating panel");
    assert_eq!(
        context_panel
            .children()
            .iter()
            .filter(|node| node.role() == view::Role::Separator)
            .count(),
        2,
        "nonempty table, row, and text sections derive exactly two dividers"
    );
    let row_action = actions
        .into_iter()
        .find(|binding| binding.command_type() == std::any::TypeId::of::<ContextRow>())
        .expect("row should contribute one context action");
    app.handle_view(window, row_action.action())
        .expect("row action should invoke");
    assert_eq!(app.state().row_invoked, Some(key));

    let current = app.show_scene(window, size).expect("table should rebuild");
    let enabled_frame = current
        .layout()
        .frames()
        .iter()
        .find(|frame| frame.table_cell() == Some(enabled) && frame.role() == view::Role::Checkbox)
        .expect("Boolean cell should be laid out");
    app.open_context_menu_at(window, size, frame_point(enabled_frame))
        .expect("cell context should open");
    let projected = app.present(window).expect("cell menu should project");
    let actions = projected
        .bindings()
        .into_iter()
        .filter(|binding| binding.source() == context::Source::Menu)
        .collect::<Vec<_>>();
    assert_eq!(actions.len(), 3);
    assert_eq!(
        actions[0].command_type(),
        std::any::TypeId::of::<document::SelectAll>()
    );
    assert_eq!(
        actions[1].command_type(),
        std::any::TypeId::of::<ContextRow>()
    );
    assert_eq!(
        actions[2].command_type(),
        std::any::TypeId::of::<ContextToggle>()
    );
    app.handle_view(window, actions[2].action())
        .expect("Boolean context action should invoke");
    assert_eq!(app.state().toggled, Some((enabled, true)));

    let current = app
        .show_scene(window, size)
        .expect("table should remain visible");
    let name_frame = current
        .layout()
        .frames()
        .iter()
        .find(|frame| frame.table_cell() == Some(name) && frame.role() == view::Role::TextBox)
        .expect("text cell should still be laid out");
    app.open_context_menu_at(window, size, frame_point(name_frame))
        .expect("row context should reopen");
    app.change(state::Reason::programmatic("remove-context-row"), |state| {
        state.visible = false;
    });
    app.show_scene(window, size)
        .expect("virtual row removal should reconcile");
    assert_eq!(
        app.session()
            .interaction(window)
            .and_then(Interaction::open_menu),
        None
    );
}

#[test]
fn active_table_editor_uses_task_order_and_owns_select_all() {
    let mut app = table_context_app();
    app.start();
    let window = app.session().windows()[0].id();
    let size = geometry::Size::new(320, 180);
    let cell = crate::table::Cell::new(
        "context.table".into(),
        virtual_list::Key::new(17),
        "name".into(),
    );
    app.show_scene(window, size).expect("table should render");
    app.handle_input(window, Input::focus(session::Focus::table_cell(cell)))
        .expect("the editable text cell should take focus");
    app.handle_input(
        window,
        Input::key_down(input::Key::F2, input::Modifiers::default()),
    )
    .expect("the editable text cell should enter its task session");
    let editing = app
        .show_scene(window, size)
        .expect("active editor should render");
    let editor = editing
        .layout()
        .frames()
        .iter()
        .find(|frame| frame.role() == view::Role::TextBox && frame.table_cell() == Some(cell))
        .expect("active table editor should be laid out");

    app.handle_input(window, Input::shortcut("Ctrl+A"))
        .expect("live keyboard routing should reach the active text task");
    assert_eq!(
        text_draft(&app, window, session::Focus::table_cell(cell))
            .selected_text()
            .as_deref(),
        Some("Seventeen")
    );
    assert!(
        !app.session()
            .selection(window, "context.table".into())
            .is_some_and(crate::selection::Selection::is_all),
        "the live text task consumes the keyboard command before the table"
    );

    app.open_context_menu_at(window, size, frame_point(editor))
        .expect("active editor context should open");
    let projected = app.present(window).expect("editor context should project");
    let actions = projected
        .bindings()
        .into_iter()
        .filter(|binding| binding.source() == context::Source::Menu)
        .collect::<Vec<_>>();

    assert_eq!(
        actions[0].command_type(),
        std::any::TypeId::of::<document::SelectAll>(),
        "Task traversal starts at the active editor"
    );
    let row_position = actions
        .iter()
        .position(|binding| binding.command_type() == std::any::TypeId::of::<ContextRow>())
        .expect("the focal row remains in the active task path");
    for command_type in [
        std::any::TypeId::of::<document::SelectAll>(),
        std::any::TypeId::of::<document::Copy>(),
        std::any::TypeId::of::<document::Cut>(),
        std::any::TypeId::of::<document::Delete>(),
        std::any::TypeId::of::<document::Paste>(),
    ] {
        let position = actions
            .iter()
            .position(|binding| binding.command_type() == command_type)
            .expect("the active text task should expose every standard edit command");
        assert!(
            position < row_position,
            "the exact text task must precede broader row commands"
        );
    }
    assert!(
        actions
            .iter()
            .any(|binding| { binding.command_type() == std::any::TypeId::of::<ContextRow>() }),
        "the focal row remains a broader task layer"
    );
    assert_eq!(
        actions
            .iter()
            .filter(|binding| {
                binding.command_type() == std::any::TypeId::of::<document::SelectAll>()
            })
            .count(),
        1
    );
    assert!(
        actions.iter().all(|binding| {
            binding.command_type() != std::any::TypeId::of::<CommitContextName>()
        })
    );

    app.handle_view(window, actions[0].action())
        .expect("editor Select All should invoke");
    let focus = session::Focus::table_cell(cell);
    assert_eq!(
        text_draft(&app, window, focus).selected_text().as_deref(),
        Some("Seventeen")
    );
    assert!(
        !app.session()
            .selection(window, "context.table".into())
            .is_some_and(crate::selection::Selection::is_all),
        "the editor claim prevents the table service from consuming Select All"
    );
}

#[test]
fn text_editor_menu_open_state_is_framework_owned_interaction() {
    let mut app = text_editor::app(text_editor::State::default());

    app.start();

    let window = app.session().windows()[0].id();
    let projected = app.present(window).expect("window should have a view");
    let file = projected
        .menus()
        .into_iter()
        .find(|menu| menu.label_text() == Some("File"))
        .expect("file menu should be in the view");
    let action = file.menu_action().expect("menu should expose an action");

    assert!(app.clear_redraw_request(window));

    let outcome = app
        .handle_view(window, action.clone())
        .expect("menu action should be handled");

    assert!(outcome.is_handled());
    assert!(!outcome.changed_state());
    assert!(outcome.effect().contains_invalidation());
    let interaction: &Interaction = app
        .session()
        .interaction(window)
        .expect("window should have interaction state");

    assert_eq!(
        interaction.open_menu().map(|menu| menu.label()),
        Some("File")
    );
    assert!(app.session().windows()[0].redraw_requested());
    assert_eq!(app.revision(), state::Revision::initial());
    let projected = app
        .present(window)
        .expect("window should still have a view");

    assert_eq!(projected.floating_panels().len(), 1);
    assert_eq!(projected.floating_panels()[0].label_text(), None);

    app.clear_redraw_request(window);
    app.handle_view(window, action)
        .expect("second menu action should be handled");

    assert_eq!(
        app.session()
            .interaction(window)
            .and_then(|interaction| interaction.open_menu()),
        None
    );
    let projected = app
        .present(window)
        .expect("window should still have a view");

    assert!(projected.floating_panels().is_empty());
    assert!(app.session().windows()[0].redraw_requested());
}

#[test]
fn table_context_preserves_multiselection_and_table_consumes_select_all() {
    let mut app = Runtime::new(SourceState::default())
        .commands(|commands| {
            commands.register::<document::SelectAll>(
                command::Spec::new("Select All")
                    .key_chord(command::KeyChord::standard(command::Standard::SelectAll)),
            );
        })
        .view(|_, _| {
            let source = crate::table::Source::new(
                3,
                |index| virtual_list::Key::new(index as u64),
                |key| usize::try_from(key.value()).ok().filter(|index| *index < 3),
                |index| ContextRecord {
                    name: format!("Row {index}"),
                    enabled: index % 2 == 0,
                },
            );
            let columns = vec![
                crate::table::Column::text(
                    "name",
                    "Name",
                    view::Dimension::fixed(140),
                    |record: &ContextRecord| &record.name,
                )
                .unsortable()
                .build(),
            ];
            widget::view_node(
                crate::Table::typed("selection.context", 24, columns, source)
                    .width(view::Dimension::fixed(180))
                    .height(view::Dimension::fixed(120)),
            )
        })
        .started(|cx| {
            cx.open_window(window::Options::new("Selection context"));
        });
    app.start();
    let window = app.session().windows()[0].id();
    let size = geometry::Size::new(320, 180);
    let rendered = app.show_scene(window, size).expect("table should render");
    let row_point = |key| {
        let cell = crate::table::Cell::new(
            "selection.context".into(),
            virtual_list::Key::new(key),
            "name".into(),
        );
        frame_point(
            rendered
                .layout()
                .frames()
                .iter()
                .find(|frame| frame.table_cell() == Some(cell))
                .expect("row cell should materialize"),
        )
    };
    let primary = input::Modifiers::new(false, true, false, false);
    app.pointer_down_at_with_modifiers(window, size, row_point(0), input::Modifiers::default())
        .expect("first row should select");
    app.pointer_down_at_with_modifiers(window, size, row_point(1), primary)
        .expect("primary-click should toggle a second row");
    let selection = app
        .session()
        .selection(window, "selection.context".into())
        .expect("table should retain keyed selection");
    assert_eq!(selection.len(), 2);

    let body = rendered.layout().find_role(view::Role::VirtualList)[0].rect();
    app.open_context_menu_at(
        window,
        size,
        geometry::Point::new(body.x() + 1, body.bottom() - 1),
    )
    .expect("empty table space should open the table domain");
    assert_eq!(
        app.session()
            .selection(window, "selection.context".into())
            .map(crate::selection::Selection::len),
        Some(2),
        "secondary click on empty table space retains membership"
    );
    let empty_space = app
        .present(window)
        .expect("table-only context should project");
    let empty_actions = empty_space
        .bindings()
        .into_iter()
        .filter(|binding| binding.source() == context::Source::Menu)
        .collect::<Vec<_>>();
    assert_eq!(empty_actions.len(), 1);
    assert_eq!(
        empty_actions[0].command_type(),
        std::any::TypeId::of::<document::SelectAll>()
    );
    app.handle_input(window, Input::cancel())
        .expect("table-only context should close");

    app.open_context_menu_at(window, size, row_point(1))
        .expect("selected focal row should open context");
    assert_eq!(
        app.session()
            .selection(window, "selection.context".into())
            .map(crate::selection::Selection::len),
        Some(2),
        "secondary click on selected membership preserves the set"
    );
    let projected = app.present(window).expect("context menu should project");
    let select_all = projected
        .bindings()
        .into_iter()
        .find(|binding| {
            binding.source() == context::Source::Menu
                && binding.command_type() == std::any::TypeId::of::<document::SelectAll>()
        })
        .expect("the table domain should own canonical Select All");
    app.handle_view(window, select_all.action())
        .expect("Select All should invoke through the table service");
    assert!(
        app.session()
            .selection(window, "selection.context".into())
            .is_some_and(crate::selection::Selection::is_all)
    );
}

#[test]
fn context_capture_pins_a_dematerialized_focal_row_but_not_a_deleted_one() {
    let mut app = Runtime::new(PinnedContextState {
        keys: (0..50).collect(),
    })
    .commands(|commands| {
        commands.register::<ContextRow>(command::Spec::new("Open row"));
    })
    .responders(|responders| {
        responders.app().target::<ContextRow>();
    })
    .view(|state, _| {
        let keys = state.keys.clone();
        let source = crate::table::Source::new(
            keys.len(),
            {
                let keys = keys.clone();
                move |index| virtual_list::Key::new(keys[index])
            },
            {
                let keys = keys.clone();
                move |key| keys.iter().position(|candidate| *candidate == key.value())
            },
            move |index| ContextRecord {
                name: format!("Row {}", keys[index]),
                enabled: false,
            },
        );
        let columns = vec![
            crate::table::Column::text(
                "name",
                "Name",
                view::Dimension::fixed(140),
                |record: &ContextRecord| &record.name,
            )
            .unsortable()
            .build(),
        ];
        widget::view_node(
            crate::Table::typed("pinned.context", 24, columns, source)
                .context_rows::<ContextRow>(|key| key)
                .width(view::Dimension::fixed(180))
                .height(view::Dimension::fixed(96)),
        )
    })
    .started(|cx| {
        cx.open_window(window::Options::new("Pinned context"));
    });
    app.start();
    let window = app.session().windows()[0].id();
    let size = geometry::Size::new(260, 120);
    let initial = app.show_scene(window, size).expect("table should render");
    let focal = crate::table::Cell::new(
        "pinned.context".into(),
        virtual_list::Key::new(0),
        "name".into(),
    );
    let focal_point = frame_point(
        initial
            .layout()
            .frames()
            .iter()
            .find(|frame| frame.table_cell() == Some(focal))
            .expect("focal row should materialize"),
    );
    let list_rect = initial.layout().find_role(view::Role::VirtualList)[0].rect();
    app.open_context_menu_at(window, size, focal_point)
        .expect("focal context should open");
    app.scroll_at(
        window,
        size,
        geometry::Point::new(list_rect.x() + 1, list_rect.y() + 1),
        interaction::ScrollDelta::vertical(720),
    )
    .expect("table should scroll while context is captured");
    app.show_scene(window, size)
        .expect("scrolled table should rebuild with its context pin");
    assert!(
        app.session()
            .interaction(window)
            .and_then(Interaction::open_menu)
            .is_some_and(interaction::Menu::is_context),
        "dematerialization alone must not dismiss the focal context"
    );

    app.change(state::Reason::programmatic("delete-focal-row"), |state| {
        state.keys.retain(|key| *key != 0);
    });
    app.show_scene(window, size)
        .expect("provider deletion should reconcile");
    assert_eq!(
        app.session()
            .interaction(window)
            .and_then(Interaction::open_menu),
        None,
        "provider deletion ends the captured subject and its menu"
    );
}

#[test]
fn opening_command_palette_replaces_an_open_menu_session() {
    let mut app = text_editor::app(text_editor::State::default());

    app.start();

    let window = app.session().windows()[0].id();
    let projected = app.present(window).expect("window should have a view");
    let file = projected
        .menus()
        .into_iter()
        .find(|menu| menu.label_text() == Some("File"))
        .expect("file menu should be in the view");
    app.handle_view(
        window,
        file.menu_action().expect("menu should expose an action"),
    )
    .expect("menu action should be handled");

    app.handle_input(window, Input::shortcut("Ctrl+Shift+P"))
        .expect("palette shortcut should dispatch");

    let interaction = app
        .session()
        .interaction(window)
        .expect("window should retain interaction state");
    assert_eq!(interaction.open_menu(), None);
    assert!(interaction.command_palette().is_some());
}

#[test]
fn opening_a_menu_replaces_an_open_command_palette_session() {
    let mut app = text_editor::app(text_editor::State::default());

    app.start();

    let window = app.session().windows()[0].id();
    let projected = app.present(window).expect("window should have a view");
    let file = projected
        .menus()
        .into_iter()
        .find(|menu| menu.label_text() == Some("File"))
        .expect("file menu should be in the view");
    let action = file.menu_action().expect("menu should expose an action");
    app.handle_input(window, Input::shortcut("Ctrl+Shift+P"))
        .expect("palette shortcut should dispatch");

    app.handle_view(window, action)
        .expect("menu action should be handled");

    let interaction = app
        .session()
        .interaction(window)
        .expect("window should retain interaction state");
    assert_eq!(
        interaction.open_menu().map(|menu| menu.label()),
        Some("File")
    );
    assert!(interaction.command_palette().is_none());
}

#[test]
fn hovering_another_menu_title_switches_open_menu() {
    let mut app = text_editor::app(text_editor::State::default());

    app.start();

    let window = app.session().windows()[0].id();
    let size = geometry::Size::new(800, 600);
    let initial = app
        .show_scene(window, size)
        .expect("text editor should render");
    let file = labeled_frame(initial.layout(), view::Role::Menu, "File");
    let edit = labeled_frame(initial.layout(), view::Role::Menu, "Edit");

    app.pointer_down_at(window, size, frame_point(file))
        .expect("file menu pointer down should be handled");
    app.pointer_up_at(window, size, frame_point(file))
        .expect("file menu pointer up should open the menu");
    app.show_scene(window, size)
        .expect("open file menu should render");

    let switched = app
        .pointer_move_at(window, size, frame_point(edit))
        .expect("edit menu hover should be handled");

    assert!(switched.is_handled());
    assert_eq!(
        app.session()
            .interaction(window)
            .and_then(|interaction| interaction.open_menu())
            .map(|menu| menu.label()),
        Some("Edit")
    );
}

#[test]
fn pointer_down_outside_menu_surface_closes_open_menu() {
    let mut app = text_editor::app(text_editor::State::default());

    app.start();

    let window = app.session().windows()[0].id();
    let size = geometry::Size::new(800, 600);
    let initial = app
        .show_scene(window, size)
        .expect("text editor should render");
    let file = labeled_frame(initial.layout(), view::Role::Menu, "File");

    app.pointer_down_at(window, size, frame_point(file))
        .expect("file menu pointer down should be handled");
    app.pointer_up_at(window, size, frame_point(file))
        .expect("file menu pointer up should open the menu");
    let opened = app
        .show_scene(window, size)
        .expect("open file menu should render");
    let text_area = opened
        .layout()
        .find_role(view::Role::TextArea)
        .into_iter()
        .next()
        .expect("text area should be laid out");

    let outside_popup = geometry::Point::new(
        size.width().saturating_sub(2),
        text_area.rect().y().saturating_add(80),
    );

    app.pointer_down_at(window, size, outside_popup)
        .expect("outside pointer down should be handled");

    assert_eq!(
        app.session()
            .interaction(window)
            .and_then(|interaction| interaction.open_menu()),
        None
    );
}

#[test]
fn parent_pointer_left_does_not_close_open_menu() {
    let mut app = text_editor::app(text_editor::State::default());

    app.start();

    let window = app.session().windows()[0].id();
    let size = geometry::Size::new(800, 600);
    let initial = app
        .show_scene(window, size)
        .expect("text editor should render");
    let file = labeled_frame(initial.layout(), view::Role::Menu, "File");

    app.pointer_down_at(window, size, frame_point(file))
        .expect("file menu pointer down should be handled");
    app.pointer_up_at(window, size, frame_point(file))
        .expect("file menu pointer up should open the menu");

    app.pointer_left_at(window)
        .expect("parent pointer leave should be handled");

    assert_eq!(
        app.session()
            .interaction(window)
            .and_then(|interaction| interaction.open_menu())
            .map(|menu| menu.label()),
        Some("File")
    );
}

#[test]
fn menu_command_activation_closes_framework_owned_menu_state() {
    let mut app = text_editor::app(text_editor::State::default());

    app.start();

    let window = app.session().windows()[0].id();
    let projected = app.present(window).expect("window should have a view");
    let file = projected
        .menus()
        .into_iter()
        .find(|menu| menu.label_text() == Some("File"))
        .expect("file menu should be in the view");
    app.handle_view(
        window,
        file.menu_action().expect("menu should expose an action"),
    )
    .expect("menu action should be handled");

    let projected = app
        .present(window)
        .expect("window should still have a view");
    let open = projected
        .binding::<document::OpenFile>()
        .expect("open command should be in the view")
        .action();

    app.handle_view(window, open)
        .expect("open action should be handled");

    assert_eq!(
        app.session()
            .interaction(window)
            .and_then(|interaction| interaction.open_menu()),
        None
    );
    assert_eq!(
        app.session().file_dialog(window),
        Some(session::FileDialog::Open)
    );
    assert_eq!(app.state().last_status, "choosing file");
    assert_eq!(app.revision().get(), 1);
}

#[test]
fn cancel_input_closes_open_menu_before_clearing_focus() {
    let mut app = text_editor::app(text_editor::State::default());

    app.start();

    let window = app.session().windows()[0].id();
    let projected = app.present(window).expect("window should have a view");
    let focus = projected.text_areas()[0]
        .focus()
        .expect("text area should declare a focus target");
    app.handle_input(window, Input::focus(focus))
        .expect("focus input should be handled");
    let file = projected
        .menus()
        .into_iter()
        .find(|menu| menu.label_text() == Some("File"))
        .expect("file menu should be in the view");
    app.handle_view(
        window,
        file.menu_action().expect("menu should expose an action"),
    )
    .expect("menu action should be handled");

    assert_eq!(app.session().focused(window), Some(focus));
    assert_eq!(
        app.session()
            .interaction(window)
            .and_then(|interaction| interaction.open_menu())
            .map(|menu| menu.label()),
        Some("File")
    );
    assert!(app.clear_redraw_request(window));

    let outcome = app
        .handle_input(window, Input::cancel())
        .expect("cancel input should be handled");

    assert!(outcome.is_handled());
    assert!(!outcome.changed_state());
    assert!(outcome.effect().contains_invalidation());
    assert_eq!(
        app.session()
            .interaction(window)
            .and_then(|interaction| interaction.open_menu()),
        None
    );
    assert_eq!(app.session().focused(window), Some(focus));
    assert_eq!(app.revision(), state::Revision::initial());
}

#[test]
fn cancel_input_clears_focus_when_no_menu_is_open() {
    let mut app = text_editor::app(text_editor::State::default());

    app.start();

    let window = app.session().windows()[0].id();
    let projected = app.present(window).expect("window should have a view");
    let focus = projected.text_areas()[0]
        .focus()
        .expect("text area should declare a focus target");
    app.handle_input(window, Input::focus(focus))
        .expect("focus input should be handled");
    assert_eq!(app.session().focused(window), Some(focus));
    assert!(app.clear_redraw_request(window));

    let outcome = app
        .handle_input(window, Input::cancel())
        .expect("cancel input should be handled");

    assert!(outcome.is_handled());
    assert!(!outcome.changed_state());
    assert!(outcome.effect().contains_invalidation());
    assert_eq!(app.session().focused(window), None);
    assert!(app.session().windows()[0].redraw_requested());
    assert_eq!(app.revision(), state::Revision::initial());

    let outcome = app
        .handle_input(window, Input::cancel())
        .expect("second cancel input should be ignored");

    assert!(!outcome.is_handled());
    assert!(!outcome.changed_state());
    assert_eq!(outcome.effect(), &response::Effect::None);
}

#[test]
fn pointer_actions_update_framework_owned_hover_and_press_state() {
    let mut app = text_editor::app(text_editor::State::default());

    app.start();

    let window = app.session().windows()[0].id();
    let projected = app.present(window).expect("window should have a view");
    let file = projected
        .menus()
        .into_iter()
        .find(|menu| menu.label_text() == Some("File"))
        .expect("file menu should be in the view");
    let target = file
        .pointer_target()
        .expect("file menu should have a pointer target");
    let pointer_move = file
        .pointer_move_action()
        .expect("file menu should expose pointer move");
    let pointer_down = file
        .pointer_down_action()
        .expect("file menu should expose pointer down");

    let moved = app
        .handle_view(window, pointer_move)
        .expect("pointer move should be handled");

    assert!(moved.is_handled());
    assert!(!moved.changed_state());
    assert!(moved.effect().contains_invalidation());
    let interaction: &Interaction = app
        .session()
        .interaction(window)
        .expect("window should have interaction state");
    assert_eq!(interaction.pointer().hovered(), Some(&target));
    assert_eq!(interaction.pointer().pressed(), None);
    assert_eq!(interaction.pointer().capture(), None);

    let pressed = app
        .handle_view(window, pointer_down)
        .expect("pointer down should be handled");

    assert!(pressed.is_handled());
    assert!(!pressed.changed_state());
    assert!(pressed.effect().contains_invalidation());
    let interaction = app
        .session()
        .interaction(window)
        .expect("window should have interaction state");
    assert_eq!(interaction.pointer().hovered(), Some(&target));
    assert_eq!(interaction.pointer().pressed(), Some(&target));
    assert_eq!(interaction.pointer().capture(), None);

    let left = app
        .handle_view(window, view::Action::pointer_left())
        .expect("pointer left should be handled");

    assert!(left.is_handled());
    assert!(!left.changed_state());
    assert!(left.effect().contains_invalidation());
    let interaction = app
        .session()
        .interaction(window)
        .expect("window should have interaction state");
    assert_eq!(interaction.pointer().hovered(), None);
    assert_eq!(interaction.pointer().pressed(), None);
    assert_eq!(interaction.pointer().capture(), None);
    assert_eq!(app.revision(), state::Revision::initial());
}

#[test]
fn rebuilding_away_captured_command_prunes_pointer_and_history_gesture() {
    let mut app = Runtime::new(ReplacingInteractionState {
        level: 0.0,
        invocations: 0,
        visible: true,
        available: true,
    })
    .commands(|commands| {
        commands.register::<SetLevel>(command::Spec::new("Set Level"));
    })
    .responders(|responders| {
        responders.app().target::<SetLevel>();
    })
    .started(|cx| {
        cx.open_window(window::Options::new("Replacing Interaction"));
    })
    .view(|state, _| {
        widget::view(|ui| {
            if state.visible {
                ui.slider(
                    widget::Slider::new("Level", state.level, 0.0..=10.0).on_change::<SetLevel>(),
                );
            } else {
                ui.label("Control removed");
            }
        })
    });

    app.start();

    let window = app.session().windows()[0].id();
    let size = geometry::Size::new(240, 80);
    let initial = app
        .show_scene(window, size)
        .expect("initial slider should render");
    let slider = initial
        .layout()
        .find_role(view::Role::Slider)
        .into_iter()
        .next()
        .expect("slider should be laid out");
    let track = layout::slider_track_rect(slider.rect(), slider.label_width(), &Theme::default());
    let press = geometry::Point::new(track.x() + track.width() / 2, track.y() + 1);

    app.pointer_down_at(window, size, press)
        .expect("slider press should begin a captured gesture");
    assert_eq!(app.state().invocations, 1);
    assert_eq!(app.window_residues(window).gesture, 1);

    app.change(state::Reason::programmatic("remove_control"), |state| {
        state.visible = false;
    });
    app.show_scene(window, size)
        .expect("replacement view should reconcile");

    let pointer = app
        .session()
        .interaction(window)
        .expect("window should retain interaction state")
        .pointer();
    assert_eq!(pointer.hovered(), None);
    assert_eq!(pointer.pressed(), None);
    assert_eq!(pointer.capture(), None);
    assert_eq!(
        app.window_residues(window).gesture,
        0,
        "a gesture whose captured target was removed must not outlive capture"
    );

    let invocations = app.state().invocations;
    app.pointer_up_at(window, size, press)
        .expect("late release should be safely routed");
    assert_eq!(app.state().invocations, invocations);
}

#[test]
fn text_area_pointer_down_starts_framework_pointer_capture() {
    let mut app = text_editor::app(text_editor::State::default());

    app.start();

    let window = app.session().windows()[0].id();
    let projected = app.present(window).expect("window should have a view");
    let text_area = text_area_node(projected.root()).expect("text area should be in the view");
    let target = text_area
        .pointer_target()
        .expect("text area should have a pointer target");

    assert!(target.captures());

    let outcome = app
        .handle_view(
            window,
            text_area
                .pointer_down_action()
                .expect("text area should expose pointer down"),
        )
        .expect("text area pointer down should be handled");

    assert!(outcome.is_handled());
    assert!(!outcome.changed_state());
    assert!(outcome.effect().contains_invalidation());
    let pointer = app
        .session()
        .interaction(window)
        .expect("window should have interaction state")
        .pointer();

    assert_eq!(pointer.hovered(), Some(&target));
    assert_eq!(pointer.pressed(), Some(&target));
    assert_eq!(
        pointer.capture().map(|capture| capture.target()),
        Some(&target)
    );
    assert_eq!(app.revision(), state::Revision::initial());
}

#[test]
fn text_area_scroll_action_updates_framework_owned_scroll_state() {
    let mut app = text_editor::app(text_editor::State::default());

    app.start();

    let window = app.session().windows()[0].id();
    let projected = app.present(window).expect("window should have a view");
    let text_area = text_area_node(projected.root()).expect("text area should be in the view");
    let target = text_area
        .pointer_target()
        .expect("text area should have a scroll target");
    let revision = app.revision();

    let scrolled = app
        .handle_view(
            window,
            text_area
                .scroll_action(interaction::ScrollDelta::vertical(120))
                .expect("text area should expose scroll"),
        )
        .expect("scroll should be handled");

    assert!(scrolled.is_handled());
    assert!(!scrolled.changed_state());
    assert!(scrolled.effect().contains_invalidation());
    assert_eq!(app.revision(), revision);
    let scroll = app
        .session()
        .interaction(window)
        .expect("window should have interaction state")
        .scroll();
    assert_eq!(scroll.offset(&target), interaction::ScrollOffset::default());
    assert_eq!(
        scroll.desired_offset(&target),
        interaction::ScrollOffset::new(0, 120)
    );
    {
        let diagnostics = app
            .diagnostics(window)
            .expect("window should have diagnostics after scrolling");
        assert_eq!(diagnostics.scroll.wheel_events, 1);
        assert_eq!(diagnostics.scroll.scroll_offset_changes, 1);
        assert_eq!(diagnostics.scroll.scroll_redraw_requests, 1);
    }

    let scrolled_again = app
        .handle_input(
            window,
            Input::scroll(target.clone(), interaction::ScrollDelta::new(8, -20)),
        )
        .expect("scroll input should be handled");

    assert!(scrolled_again.is_handled());
    assert!(!scrolled_again.changed_state());
    assert!(scrolled_again.effect().contains_invalidation());
    assert_eq!(app.revision(), revision);
    let scroll = app
        .session()
        .interaction(window)
        .expect("window should have interaction state")
        .scroll();
    assert_eq!(scroll.offset(&target), interaction::ScrollOffset::default());
    assert_eq!(
        scroll.desired_offset(&target),
        interaction::ScrollOffset::new(8, 100)
    );
    {
        let diagnostics = app
            .diagnostics(window)
            .expect("window should retain diagnostics after scrolling again");
        assert_eq!(diagnostics.scroll.wheel_events, 2);
        assert_eq!(diagnostics.scroll.scroll_offset_changes, 2);
        assert_eq!(diagnostics.scroll.scroll_redraw_requests, 2);
    }
}

#[test]
fn text_area_interaction_id_scrolls_without_focus() {
    let document = (0..120)
        .map(|line| format!("preview line {line:03}"))
        .collect::<Vec<_>>()
        .join("\n");
    let buffer = text::Buffer::from_multiline_text(document);
    let edit_state = buffer.initial_state();
    let mut app = Runtime::new(SourceState::default())
        .started(|cx| {
            cx.open_window(window::Options::new("Preview"));
        })
        .view(move |_, _| {
            View::new(
                view::Node::root().child(
                    view::Node::text_area_state(view::TextArea::from_buffer(
                        buffer.clone(),
                        edit_state,
                    ))
                    .with_interaction_id("preview"),
                ),
            )
        });

    app.start();

    let window = app.session().windows()[0].id();
    let size = geometry::Size::new(800, 600);
    let presentation = app
        .show_scene(window, size)
        .expect("initial preview scene should render");
    let text_area = presentation
        .layout()
        .find_role(view::Role::TextArea)
        .into_iter()
        .next()
        .expect("preview text area should be laid out");
    let target = text_area
        .target()
        .expect("preview text area should expose a scroll target")
        .clone();
    let point = geometry::Point::new(text_area.rect().x() + 4, text_area.rect().y() + 4);

    assert_eq!(app.session().focused(window), None);
    assert_eq!(target, interaction::Target::text_area_id("preview"));

    let scrolled = app
        .scroll_at(window, size, point, interaction::ScrollDelta::vertical(96))
        .expect("preview scroll should route by hit test");

    assert!(scrolled.is_handled());
    assert!(!scrolled.changed_state());
    assert_eq!(scrolled.effect(), &response::Effect::None);

    let presentation = app
        .show_scene(window, size)
        .expect("scrolled preview scene should render");
    let presented_text_area = presentation
        .layout()
        .find_role(view::Role::TextArea)
        .into_iter()
        .next()
        .expect("presented preview should retain its text area");
    assert_eq!(
        presented_text_area.resolved_scroll(),
        Some(Default::default())
    );
    let projection = presentation
        .layout()
        .scroll_projections()
        .iter()
        .find(|projection| projection.target() == &target)
        .expect("preview text area should retain its scroll projection");
    assert_eq!(
        presentation.properties().scroll_offset(projection.node()),
        Some(interaction::ScrollOffset::new(0, 96))
    );

    assert_eq!(app.session().focused(window), None);
    assert_eq!(
        app.session()
            .interaction(window)
            .expect("window should have interaction state")
            .scroll()
            .offset(&target),
        interaction::ScrollOffset::new(0, 96)
    );
    let text_area = presentation
        .layout()
        .find_role(view::Role::TextArea)
        .into_iter()
        .next()
        .expect("preview text area should be laid out after scrolling");
    assert_eq!(
        text_area
            .text_area_layout()
            .expect("preview should use text area layout")
            .layout()
            .scroll_y(),
        0.0
    );
}

#[test]
fn text_input_preedit_is_framework_owned_and_projected_into_text_area() {
    let mut app = text_editor::app(text_editor::State::default());

    app.start();

    let window = app.session().windows()[0].id();
    let projected = app.present(window).expect("window should have a view");
    let text_area = text_area_node(projected.root()).expect("text area should be in the view");
    let focus = text_area
        .text_area_model()
        .and_then(view::TextArea::focus)
        .expect("text area should declare a focus target");
    let target = text_area
        .pointer_target()
        .expect("text area should have an interaction target");
    let preedit = text::Preedit::new("世界", Some((0, "世".len())));

    app.handle_input(window, Input::focus(focus))
        .expect("focus input should be handled");
    let outcome = app
        .handle_input(window, Input::text_preedit(preedit.clone()))
        .expect("preedit input should be handled");

    assert!(outcome.is_handled());
    assert!(!outcome.changed_state());
    assert!(outcome.effect().contains_invalidation());
    assert_eq!(app.revision(), state::Revision::initial());
    let text_input = app
        .session()
        .interaction(window)
        .expect("window should have interaction state")
        .text_input();
    assert_eq!(text_input.target(), Some(&target));
    assert_eq!(text_input.preedit(), Some(&preedit));

    let projected = app
        .present(window)
        .expect("window should project interaction into a view");
    let text_area = text_area_node(projected.root()).expect("text area should be in the view");

    assert_eq!(
        text_area
            .text_area_model()
            .expect("node should contain text area")
            .preedit(),
        Some(&preedit)
    );
}

#[test]
fn text_input_commit_routes_to_focused_document_and_clears_preedit() {
    let mut app = text_editor::app(text_editor::State::default());

    app.start();

    let window = app.session().windows()[0].id();
    let projected = app.present(window).expect("window should have a view");
    let text_area = text_area_node(projected.root()).expect("text area should be in the view");
    let focus = text_area
        .text_area_model()
        .and_then(view::TextArea::focus)
        .expect("text area should declare a focus target");

    app.handle_input(window, Input::focus(focus))
        .expect("focus input should be handled");
    app.handle_input(
        window,
        Input::text_preedit(text::Preedit::new("世", Some((0, "世".len())))),
    )
    .expect("preedit input should be handled");

    let outcome = app
        .handle_input(window, Input::text_commit("界"))
        .expect("commit input should be handled");

    assert!(outcome.is_handled());
    assert!(outcome.changed_state());
    assert!(outcome.effect().contains_invalidation());
    assert_eq!(app.state().document.text(), "界");
    assert_eq!(app.state().last_status, "edit");
    assert_eq!(app.revision().get(), 1);
    assert!(
        app.session()
            .interaction(window)
            .expect("window should have interaction state")
            .text_input()
            .preedit()
            .is_none()
    );

    let projected = app.present(window).expect("window should have a view");
    let text_area = text_area_node(projected.root()).expect("text area should be in the view");

    assert!(
        text_area
            .text_area_model()
            .expect("node should contain text area")
            .preedit()
            .is_none()
    );
}

#[test]
fn cancel_input_clears_text_preedit_before_clearing_focus() {
    let mut app = text_editor::app(text_editor::State::default());

    app.start();

    let window = app.session().windows()[0].id();
    let projected = app.present(window).expect("window should have a view");
    let text_area = text_area_node(projected.root()).expect("text area should be in the view");
    let focus = text_area
        .text_area_model()
        .and_then(view::TextArea::focus)
        .expect("text area should declare a focus target");

    app.handle_input(window, Input::focus(focus))
        .expect("focus input should be handled");
    app.handle_input(
        window,
        Input::text_preedit(text::Preedit::new("世", Some((0, "世".len())))),
    )
    .expect("preedit input should be handled");

    let canceled = app
        .handle_input(window, Input::cancel())
        .expect("cancel input should clear preedit");

    assert!(canceled.is_handled());
    assert!(!canceled.changed_state());
    assert!(canceled.effect().contains_invalidation());
    assert_eq!(app.session().focused(window), Some(focus));
    assert!(
        app.session()
            .interaction(window)
            .expect("window should have interaction state")
            .text_input()
            .preedit()
            .is_none()
    );

    let canceled = app
        .handle_input(window, Input::cancel())
        .expect("second cancel input should clear focus");

    assert!(canceled.is_handled());
    assert_eq!(app.session().focused(window), None);
}

#[test]
fn text_input_preedit_is_transient_and_clears_on_restore() {
    let mut app = text_editor::app(text_editor::State::default());

    app.start();

    let window = app.session().windows()[0].id();
    let projected = app.present(window).expect("window should have a view");
    let text_area = text_area_node(projected.root()).expect("text area should be in the view");
    let focus = text_area
        .text_area_model()
        .and_then(view::TextArea::focus)
        .expect("text area should declare a focus target");

    app.handle_input(window, Input::focus(focus))
        .expect("focus input should be handled");
    let snapshot = app.snapshot();

    app.handle_input(
        window,
        Input::text_preedit(text::Preedit::new("世", Some((0, "世".len())))),
    )
    .expect("preedit input should be handled");
    assert!(
        app.session()
            .interaction(window)
            .expect("window should have interaction state")
            .text_input()
            .preedit()
            .is_some()
    );

    app.restore(snapshot);

    assert_eq!(app.session().focused(window), Some(focus));
    assert!(
        app.session()
            .interaction(window)
            .expect("restored window should have interaction state")
            .text_input()
            .preedit()
            .is_none()
    );
    let projected = app.present(window).expect("window should have a view");
    let text_area = text_area_node(projected.root()).expect("text area should be in the view");
    assert!(
        text_area
            .text_area_model()
            .expect("node should contain text area")
            .preedit()
            .is_none()
    );
}

#[test]
fn text_area_pointer_click_focuses_and_routes_cursor_edit() {
    let mut app = text_editor::app(text_editor::State {
        document: TextDocument::from_text("hello world"),
        ..text_editor::State::default()
    });

    app.start();

    let window = app.session().windows()[0].id();
    let projected = app.present(window).expect("window should have a view");
    let text_area = text_area_node(projected.root()).expect("text area should be in the view");
    let focus = text_area
        .text_area_model()
        .and_then(view::TextArea::focus)
        .expect("text area should declare a focus target");
    let target = text_area
        .pointer_target()
        .expect("text area should have a pointer target");

    let outcome = app
        .handle_view(
            window,
            text_area
                .text_pointer_down_action(text::buffer::Position::new(5))
                .expect("text area should expose pointer click"),
        )
        .expect("text area pointer click should be handled");

    assert!(outcome.is_handled());
    assert!(outcome.changed_state());
    assert!(outcome.effect().contains_invalidation());
    let actual_focus = app
        .session()
        .focused(window)
        .expect("text area should be focused");
    assert!(actual_focus.same_target(&focus));
    assert_eq!(actual_focus.reason(), session::Reason::Pointer);
    assert_eq!(actual_focus.visibility(), session::Visibility::Hidden);
    let focused = app
        .show_scene(window, geometry::Size::new(480, 180))
        .expect("pointer-focused text area should render");
    let focused_text_area = focused
        .layout()
        .find_role(view::Role::TextArea)
        .into_iter()
        .next()
        .expect("text area should be laid out");

    assert!(
        focused.scene().outlines().iter().any(|outline| {
            outline.rect() == focused_text_area.rect()
                && outline.color() == Theme::default().focus().color
        }),
        "pointer-focused editable text area should retain editor chrome"
    );
    assert_eq!(app.state().document.position().index, 5);
    assert_eq!(app.state().document.selected_text(), None);
    assert_eq!(
        app.session()
            .interaction(window)
            .expect("window should have interaction state")
            .pointer()
            .capture()
            .map(|capture| capture.target()),
        Some(&target)
    );
}

#[test]
fn pointer_left_preserves_captured_text_area_until_release() {
    let mut app = text_editor::app(text_editor::State::default());

    app.start();

    let window = app.session().windows()[0].id();
    let projected = app.present(window).expect("window should have a view");
    let text_area = text_area_node(projected.root()).expect("text area should be in the view");
    let target = text_area
        .pointer_target()
        .expect("text area should have a pointer target");

    app.handle_view(
        window,
        text_area
            .pointer_down_action()
            .expect("text area should expose pointer down"),
    )
    .expect("text area pointer down should be handled");

    let left = app
        .handle_view(window, view::Action::pointer_left())
        .expect("pointer left should be handled");

    assert!(left.is_handled());
    assert!(!left.changed_state());
    assert!(left.effect().contains_invalidation());
    let pointer = app
        .session()
        .interaction(window)
        .expect("window should have interaction state")
        .pointer();

    assert_eq!(pointer.hovered(), None);
    assert_eq!(pointer.pressed(), Some(&target));
    assert_eq!(
        pointer.capture().map(|capture| capture.target()),
        Some(&target)
    );

    let released = app
        .handle_view(window, view::Action::pointer_up_outside())
        .expect("pointer up should be handled");

    assert!(released.is_handled());
    let pointer = app
        .session()
        .interaction(window)
        .expect("window should have interaction state")
        .pointer();
    assert_eq!(pointer.hovered(), None);
    assert_eq!(pointer.pressed(), None);
    assert_eq!(pointer.capture(), None);
}

#[test]
fn pointer_departure_clears_retained_position_and_hover() {
    let mut app = text_editor::app(text_editor::State::default());
    app.start();
    let window = app.session().windows()[0].id();
    let size = geometry::Size::new(480, 180);
    let shown = app
        .show_scene(window, size)
        .expect("text editor should present");
    let menu = shown
        .layout()
        .find_role(view::Role::Menu)
        .into_iter()
        .next()
        .expect("menu should be laid out");
    let point = frame_point(menu);

    app.pointer_move_at(window, size, point)
        .expect("pointer move should be handled");
    let pointer = app.session().interaction(window).unwrap().pointer();
    assert_eq!(
        pointer.location().map(|location| location.point()),
        Some(point)
    );
    assert!(pointer.hovered().is_some());

    app.pointer_left_at(window)
        .expect("pointer departure should be handled");
    let pointer = app.session().interaction(window).unwrap().pointer();
    assert_eq!(pointer.location(), None);
    assert_eq!(pointer.hovered(), None);
}

#[test]
fn cancel_input_clears_pointer_capture_before_clearing_focus() {
    let mut app = text_editor::app(text_editor::State::default());

    app.start();

    let window = app.session().windows()[0].id();
    let projected = app.present(window).expect("window should have a view");
    let text_area = text_area_node(projected.root()).expect("text area should be in the view");
    let focus = text_area
        .text_area_model()
        .and_then(view::TextArea::focus)
        .expect("text area should declare a focus target");
    app.handle_input(window, Input::focus(focus))
        .expect("focus input should be handled");
    app.handle_view(
        window,
        text_area
            .pointer_down_action()
            .expect("text area should expose pointer down"),
    )
    .expect("text area pointer down should be handled");
    assert!(
        app.session()
            .interaction(window)
            .and_then(|interaction| interaction.pointer().capture())
            .is_some()
    );

    let canceled = app
        .handle_input(window, Input::cancel())
        .expect("cancel input should be handled");

    assert!(canceled.is_handled());
    assert!(!canceled.changed_state());
    assert!(canceled.effect().contains_invalidation());
    assert_eq!(app.session().focused(window), Some(focus));
    let pointer = app
        .session()
        .interaction(window)
        .expect("window should have interaction state")
        .pointer();
    assert_eq!(pointer.pressed(), None);
    assert_eq!(pointer.capture(), None);
    assert_eq!(app.revision(), state::Revision::initial());
}

#[test]
fn captured_pointer_drag_routes_text_edit_to_captured_text_area() {
    let mut app = text_editor::app(text_editor::State {
        document: TextDocument::from_text("hello world"),
        ..text_editor::State::default()
    });

    app.start();

    let window = app.session().windows()[0].id();
    let projected = app.present(window).expect("window should have a view");
    let text_area = text_area_node(projected.root()).expect("text area should be in the view");
    let target = text_area
        .pointer_target()
        .expect("text area should have a pointer target");
    app.handle_view(
        window,
        text_area
            .text_pointer_down_action(text::buffer::Position::new(0))
            .expect("text area should expose pointer click"),
    )
    .expect("text area pointer down should be handled");

    let dragged = app
        .handle_view(
            window,
            text_area
                .text_pointer_drag_action(text::buffer::Position::new(5))
                .expect("text area should expose pointer drag"),
        )
        .expect("captured pointer drag should be handled");

    assert!(dragged.is_handled());
    assert!(dragged.changed_state());
    assert_eq!(dragged.effect(), &response::Effect::None);
    assert_eq!(app.state().document.text(), "hello world");
    assert_eq!(
        app.state().document.selected_text().as_deref(),
        Some("hello")
    );
    assert_eq!(
        app.session()
            .interaction(window)
            .expect("window should have interaction state")
            .pointer()
            .capture()
            .map(|capture| capture.target()),
        Some(&target)
    );
}

#[test]
fn pointer_drag_without_matching_capture_does_not_invoke_text_edit() {
    let mut app = text_editor::app(text_editor::State {
        document: TextDocument::from_text("hello world"),
        ..text_editor::State::default()
    });

    app.start();

    let window = app.session().windows()[0].id();
    let projected = app.present(window).expect("window should have a view");
    let text_area = text_area_node(projected.root()).expect("text area should be in the view");

    let dragged = app
        .handle_view(
            window,
            text_area
                .text_pointer_drag_action(text::buffer::Position::new(5))
                .expect("text area should expose pointer drag"),
        )
        .expect("uncaptured pointer drag should still be handled as pointer state");

    assert!(dragged.is_handled());
    assert!(!dragged.changed_state());
    assert!(dragged.effect().contains_invalidation());
    assert_eq!(app.state().document.text(), "hello world");
    assert_eq!(app.state().document.selected_text(), None);
    assert_eq!(app.revision(), state::Revision::initial());
}

#[test]
fn pointer_release_over_pressed_menu_invokes_menu_action() {
    let mut app = text_editor::app(text_editor::State::default());

    app.start();

    let window = app.session().windows()[0].id();
    let projected = app.present(window).expect("window should have a view");
    let file = projected
        .menus()
        .into_iter()
        .find(|menu| menu.label_text() == Some("File"))
        .expect("file menu should be in the view");
    let pointer_down = file
        .pointer_down_action()
        .expect("file menu should expose pointer down");
    let pointer_up = file
        .pointer_up_action()
        .expect("file menu should expose pointer up");

    app.handle_view(window, pointer_down)
        .expect("pointer down should be handled");
    let released = app
        .handle_view(window, pointer_up)
        .expect("pointer up should be handled");

    assert!(released.is_handled());
    assert!(!released.changed_state());
    assert!(released.effect().contains_invalidation());
    let interaction = app
        .session()
        .interaction(window)
        .expect("window should have interaction state");
    assert_eq!(interaction.pointer().pressed(), None);
    assert_eq!(
        interaction.open_menu().map(|menu| menu.label()),
        Some("File")
    );
    assert_eq!(app.revision(), state::Revision::initial());
}

#[test]
fn pointer_release_over_pressed_command_invokes_typed_command_binding() {
    let mut app = text_editor::app(text_editor::State::default());

    app.start();

    let window = app.session().windows()[0].id();
    let size = geometry::Size::new(800, 600);
    let wrap_point = open_view_menu_and_wrap_command_point(&mut app, window, size);

    assert!(app.state().wrap_text);
    app.pointer_down_at(window, size, wrap_point)
        .expect("pointer down should be handled");
    let released = app
        .pointer_up_at(window, size, wrap_point)
        .expect("pointer up should be handled");

    assert!(released.is_handled());
    assert!(released.changed_state());
    assert!(!app.state().wrap_text);
    assert_eq!(app.state().last_status, "wrap text disabled");
    assert_eq!(app.revision().get(), 1);
    assert_eq!(
        app.session()
            .interaction(window)
            .expect("window should have interaction state")
            .pointer()
            .pressed(),
        None
    );
}

#[test]
fn pointer_release_away_from_pressed_command_does_not_invoke() {
    let mut app = text_editor::app(text_editor::State::default());

    app.start();

    let window = app.session().windows()[0].id();
    let size = geometry::Size::new(800, 600);
    let wrap_point = open_view_menu_and_wrap_command_point(&mut app, window, size);

    app.pointer_down_at(window, size, wrap_point)
        .expect("pointer down should be handled");
    let released = app
        .handle_view(window, view::Action::pointer_up_outside())
        .expect("pointer up should be handled");

    assert!(released.is_handled());
    assert!(!released.changed_state());
    assert!(app.state().wrap_text);
    assert_eq!(app.state().last_status, "ready");
    assert_eq!(app.revision(), state::Revision::initial());
    let interaction = app
        .session()
        .interaction(window)
        .expect("window should have interaction state");
    assert_eq!(interaction.pointer().hovered(), None);
    assert_eq!(interaction.pointer().pressed(), None);
}

#[test]
fn text_editor_host_presents_pending_redraws() {
    let mut app = text_editor::app(text_editor::State::default());

    app.start();

    let window = app.session().windows()[0].id();
    assert!(app.session().windows()[0].redraw_requested());

    let presentations = app.present_pending();

    assert_eq!(presentations.len(), 1);
    assert_eq!(presentations[0].window(), window);
    assert_eq!(
        presentations[0].view().text_areas()[0].wrap(),
        view::Wrap::Word
    );
    assert!(!app.session().windows()[0].redraw_requested());
    assert!(app.present_pending().is_empty());

    let wrap_action = presentations[0]
        .view()
        .binding::<text_editor::ToggleWrapText>()
        .expect("wrap command should be in the presented view")
        .action();

    app.handle_view(window, wrap_action)
        .expect("wrap action should be handled");

    assert!(app.session().windows()[0].redraw_requested());

    let presentations = app.present_pending();

    assert_eq!(presentations.len(), 1);
    assert_eq!(presentations[0].window(), window);
    assert_eq!(
        presentations[0].view().text_areas()[0].wrap(),
        view::Wrap::None
    );
    assert!(!app.session().windows()[0].redraw_requested());
}

fn labeled_frame<'a>(
    layout: &'a layout::Layout,
    role: view::Role,
    label: &str,
) -> &'a layout::Frame {
    layout
        .find_role(role)
        .into_iter()
        .find(|frame| frame.label_text() == Some(label))
        .expect("labeled frame should be laid out")
}
