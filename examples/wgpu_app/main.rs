use std::time::Duration;

use wgpu_l3::{
    Action, Event, Icon, Task, Theme, action, app,
    geometry::{Rect, area, point},
    icon, layout, menu, paint, text, ui, widget, window,
};

const RUN_TASK: action::Id = action::Id::new("run_task");
const TOGGLE_PREVIEW: action::Id = action::Id::new("toggle_preview");
const ROOT: ui::Id = ui::Id::new("root");
const MENU_BAR: ui::Id = ui::Id::new("menu_bar");
const STATUS_PANEL: ui::Id = ui::Id::new("status_panel");
const DOCUMENT_PANEL: ui::Id = ui::Id::new("document_panel");
const DOCUMENT_SCROLL: ui::Id = ui::Id::new("document_scroll");
const SELECT_BUTTON: ui::Id = ui::Id::new("select_button");
const COMMAND_SCOPE_PANEL: ui::Id = ui::Id::new("command_scope_panel");
const LOCAL_FIELD: ui::Id = ui::Id::new("local_field");
const LOCAL_SELECT_BUTTON: ui::Id = ui::Id::new("local_select_button");
const CAPTURED_SELECT_BUTTON: ui::Id = ui::Id::new("captured_select_button");
const RUN_BUTTON: ui::Id = ui::Id::new("run_button");
const PREVIEW_BUTTON: ui::Id = ui::Id::new("preview_button");
const FILE_MENU: menu::Id = menu::Id::new("file");
const EDIT_MENU: menu::Id = menu::Id::new("edit");
const VIEW_MENU: menu::Id = menu::Id::new("view");
const PANELS_MENU: menu::Id = menu::Id::new("panels");
const SCROLL_VIEW_HEIGHT: f32 = 86.0;
const SCROLL_PADDING: f32 = 4.0;
const SCROLL_GAP: f32 = 3.0;
const SCROLL_ROW_HEIGHT: f32 = 24.0;
const SCROLL_ROWS: [ui::Id; 10] = [
    ui::Id::new("scroll_row_0"),
    ui::Id::new("scroll_row_1"),
    ui::Id::new("scroll_row_2"),
    ui::Id::new("scroll_row_3"),
    ui::Id::new("scroll_row_4"),
    ui::Id::new("scroll_row_5"),
    ui::Id::new("scroll_row_6"),
    ui::Id::new("scroll_row_7"),
    ui::Id::new("scroll_row_8"),
    ui::Id::new("scroll_row_9"),
];

fn main() -> app::Result<()> {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("debug")).init();

    app::run(App::default())
}

#[derive(Default)]
struct App {
    window: Option<window::Id>,
    workspace_ready: bool,
    preview_enabled: bool,
    run_count: u32,
    select_count: u32,
    last_select_target: Option<String>,
    last_select_origin: Option<String>,
    scope_capture_hint: Option<String>,
    document_scroll: f32,
}

enum AppEvent {
    WorkspaceReady,
    RunTaskFinished,
    TogglePreview,
    SelectAll {
        target: action::Context,
        origin: Option<ui::Path>,
    },
}

impl app::Application for App {
    type Event = AppEvent;

    fn started(&mut self, cx: &mut app::Context<'_, Self::Event>) {
        cx.register_action(Action::new(RUN_TASK, "Run Task").task(|_| {
            Task::future(async {
                std::thread::sleep(Duration::from_millis(700));
                AppEvent::RunTaskFinished
            })
        }));
        cx.register_action(
            Action::new(TOGGLE_PREVIEW, "Toggle Preview").emit(|_| AppEvent::TogglePreview),
        );
        cx.register_action(
            Action::new(action::SELECT_ALL, "Select All")
                .with_shortcut(action::Shortcut::control('a'))
                .emit(|invocation| AppEvent::SelectAll {
                    target: invocation.context().clone(),
                    origin: invocation.origin().cloned(),
                }),
        );

        let theme = Theme::default_dark();
        let window = cx.open_window(window::Options {
            title: "wgpu_l3".to_owned(),
            inner_area: area::physical(512, 560),
            canvas_color: theme.surfaces().canvas(),
        });

        self.window = Some(window);

        cx.spawn(Task::future(async {
            std::thread::sleep(Duration::from_millis(700));
            AppEvent::WorkspaceReady
        }));
    }

    fn event(&mut self, cx: &mut app::Context<'_, Self::Event>, event: Event<Self::Event>) {
        let Some(window) = self.window else {
            return;
        };

        match event {
            Event::Ui {
                window: event_window,
                event: ui::Event::ScrollRequested { target, offset },
            } if event_window == window && target == scroll_path() => {
                self.document_scroll = offset.y();
                cx.request_redraw(window);
            }
            Event::Ui { .. } => {}
            Event::App(AppEvent::WorkspaceReady) => {
                self.workspace_ready = true;
                cx.focus(window, document_path(), ui::focus::Visibility::Visible);
                cx.request_redraw(window);
            }
            Event::App(AppEvent::RunTaskFinished) => {
                self.run_count += 1;
                cx.request_redraw(window);
            }
            Event::App(AppEvent::TogglePreview) => {
                self.preview_enabled = !self.preview_enabled;
                cx.request_redraw(window);
            }
            Event::App(AppEvent::SelectAll { target, origin }) => {
                log::debug!("select all target={target:?} origin={origin:?}");
                self.last_select_target = Some(context_name(&target));
                self.last_select_origin = origin.as_ref().map(path_name);
                self.select_count += 1;
                cx.request_redraw(window);
            }
        }
    }

    fn view(
        &mut self,
        cx: &mut app::Context<'_, Self::Event>,
        window: window::Id,
        tree: &mut ui::Tree,
    ) {
        if Some(window) != self.window {
            return;
        }

        cx.action(window, RUN_TASK)
            .enabled(self.workspace_ready)
            .active(false);
        cx.action(window, TOGGLE_PREVIEW)
            .enabled(true)
            .active(self.preview_enabled);
        cx.action(window, action::SELECT_ALL)
            .enabled(false)
            .active(false);
        cx.set_action_state(
            action::SELECT_ALL,
            action::Context::path(window, document_path()),
            action::State::new(self.workspace_ready, false),
        );
        cx.set_action_state(
            action::SELECT_ALL,
            action::Context::path(window, local_field_path()),
            action::State::new(self.workspace_ready, false),
        );

        let theme = Theme::default_dark();
        let density = theme.density();
        let command_subject = cx.command_target(window);
        let subject_name = context_name(&command_subject);
        let focused_name = cx
            .focused(window)
            .as_ref()
            .map(path_name)
            .unwrap_or_else(|| "none".to_owned());

        if is_outside_command_scope(command_subject.scope()) && subject_name != "window" {
            self.scope_capture_hint = Some(subject_name.clone());
        }

        let capture_name = self
            .scope_capture_hint
            .as_deref()
            .unwrap_or("none")
            .to_owned();
        let last_select = match (&self.last_select_target, &self.last_select_origin) {
            (Some(target), Some(origin)) => format!("last select: {target} from {origin}"),
            (Some(target), None) => format!("last select: {target}"),
            _ => "last select: none".to_owned(),
        };

        let status = if self.workspace_ready {
            format!(
                "Ready | focus: {} | subject: {} | selections: {}",
                focused_name, subject_name, self.select_count,
            )
        } else {
            "Loading workspace...".to_owned()
        };
        let document_label = if self.workspace_ready {
            format!("Document responder | {last_select}")
        } else {
            "Document loading...".to_owned()
        };
        let scope_label = format!("Scope capture: {capture_name}");
        let footer = format!(
            "runs: {} | preview {}",
            self.run_count,
            if self.preview_enabled { "on" } else { "off" }
        );
        let document_is_subject = matches_path(command_subject.scope(), &document_path());
        let local_is_subject = matches_path(command_subject.scope(), &local_field_path());

        let mut document_panel = widget::panel_with_theme(DOCUMENT_PANEL, &theme)
            .with_size(layout::Size::Fill, layout::Size::Fixed(56.0))
            .with_background(subject_background(document_is_subject, &theme))
            .with_interactivity(
                ui::Interactivity::NONE
                    .with_hit_test(true)
                    .with_focusable(true),
            )
            .with_responder(action::SELECT_ALL)
            .with_label(label(document_label, &theme));
        if document_is_subject {
            document_panel = document_panel.with_stroke(subject_stroke(&theme));
        }
        let mut scroll_view = widget::scroll_view(DOCUMENT_SCROLL)
            .with_scroll_offset(point::logical(0.0, self.document_scroll))
            .with_background(theme.surfaces().panel())
            .with_rounding(theme.roundings().panel())
            .with_gap(SCROLL_GAP)
            .with_padding(layout::Insets::splat(SCROLL_PADDING))
            .with_size(layout::Size::Fill, layout::Size::Fixed(SCROLL_VIEW_HEIGHT));
        for (index, id) in SCROLL_ROWS.iter().copied().enumerate() {
            scroll_view.push_child(
                widget::panel_with_theme(id, &theme)
                    .with_size(layout::Size::Fill, layout::Size::Fixed(SCROLL_ROW_HEIGHT))
                    .with_label(label(format!("Scrollable row {}", index + 1), &theme)),
            );
        }

        let mut local_field = widget::panel_with_theme(LOCAL_FIELD, &theme)
            .with_size(
                layout::Size::Fill,
                layout::Size::Fixed(density.control_height()),
            )
            .with_background(subject_background(local_is_subject, &theme))
            .with_interactivity(
                ui::Interactivity::NONE
                    .with_hit_test(true)
                    .with_focusable(true),
            )
            .with_responder(action::SELECT_ALL)
            .with_label(label(
                if local_is_subject {
                    "Local responder | current subject"
                } else {
                    "Local responder"
                },
                &theme,
            ));
        if local_is_subject {
            local_field = local_field.with_stroke(subject_stroke(&theme));
        }

        let preview_icon = if self.preview_enabled {
            Icon::phosphor(icon::Id::new("eye"))
        } else {
            Icon::phosphor(icon::Id::new("eye-slash"))
        };
        let popup_panel = widget::floating_panel_with_theme(COMMAND_SCOPE_PANEL, &theme)
            .with_command_scope()
            .with_child(
                widget::panel_with_theme(ui::Id::new("scope_status"), &theme)
                    .with_size(
                        layout::Size::Fill,
                        layout::Size::Fixed(density.control_height()),
                    )
                    .with_label(label(scope_label, &theme)),
            )
            .with_child(local_field)
            .with_child(
                widget::labeled_button_with_theme(
                    LOCAL_SELECT_BUTTON,
                    action::SELECT_ALL,
                    "Select current subject",
                    &theme,
                )
                .with_action_target(ui::ActionTarget::Command)
                .with_size(
                    layout::Size::Fill,
                    layout::Size::Fixed(density.control_height()),
                ),
            )
            .with_child(
                widget::labeled_button_with_theme(
                    CAPTURED_SELECT_BUTTON,
                    action::SELECT_ALL,
                    "Select captured subject",
                    &theme,
                )
                .with_action_target(ui::ActionTarget::Captured)
                .with_size(
                    layout::Size::Fill,
                    layout::Size::Fixed(density.control_height()),
                ),
            );
        let root = ui::Node::container(ROOT, layout::Axis::Vertical)
            .with_background(theme.surfaces().app())
            .with_padding(layout::Insets::splat(density.app_padding()))
            .with_child(widget::menu_bar_with_theme(MENU_BAR, app_menu(), &theme))
            .with_child(
                widget::panel_with_theme(STATUS_PANEL, &theme)
                    .with_size(layout::Size::Fill, layout::Size::Fixed(44.0))
                    .with_label(label(status, &theme)),
            )
            .with_child(document_panel)
            .with_child(scroll_view)
            .with_child(
                widget::labeled_button_with_theme(
                    SELECT_BUTTON,
                    action::SELECT_ALL,
                    "Select subject",
                    &theme,
                )
                .with_action_target(ui::ActionTarget::Command)
                .with_size(
                    layout::Size::Fill,
                    layout::Size::Fixed(density.control_height()),
                ),
            )
            .with_child(
                widget::labeled_button_with_theme(RUN_BUTTON, RUN_TASK, "Run task", &theme)
                    .with_size(
                        layout::Size::Fill,
                        layout::Size::Fixed(density.control_height()),
                    ),
            )
            .with_child(
                widget::panel_with_theme(ui::Id::new("footer_panel"), &theme)
                    .with_size(
                        layout::Size::Fill,
                        layout::Size::Fixed(density.control_height()),
                    )
                    .with_label(label(footer, &theme)),
            )
            .with_child(
                widget::icon_button_with_theme(
                    PREVIEW_BUTTON,
                    TOGGLE_PREVIEW,
                    preview_icon,
                    &theme,
                )
                .with_size(
                    layout::Size::Fill,
                    layout::Size::Fixed(density.icon_button_height()),
                ),
            );

        tree.set_root(root);
        tree.clear_popups();
        tree.push_popup(widget::Popup::new(
            Rect::new(point::logical(36.0, 256.0), area::logical(440.0, 168.0)),
            popup_panel,
        ));
    }
}

fn app_menu() -> menu::Bar {
    menu::Bar::new()
        .menu(
            menu::Menu::new(FILE_MENU, "File")
                .section(menu::Section::new().item(menu::Item::new(RUN_TASK))),
        )
        .menu(
            menu::Menu::new(EDIT_MENU, "Edit")
                .section(menu::Section::new().item(menu::Item::new(action::SELECT_ALL))),
        )
        .menu(
            menu::Menu::new(VIEW_MENU, "View").section(
                menu::Section::new()
                    .item(menu::Item::new(TOGGLE_PREVIEW))
                    .separator()
                    .submenu(
                        menu::Menu::new(PANELS_MENU, "Panels").section(
                            menu::Section::new()
                                .item(menu::Item::new(TOGGLE_PREVIEW).with_label("Preview Panel")),
                        ),
                    ),
            ),
        )
}

fn is_outside_command_scope(scope: &action::Scope) -> bool {
    match scope {
        action::Scope::Path(path) => !path.is_descendant_of(&command_scope_path()),
        action::Scope::Window => false,
    }
}

fn matches_path(scope: &action::Scope, expected: &ui::Path) -> bool {
    matches!(scope, action::Scope::Path(path) if path == expected)
}

fn context_name(context: &action::Context) -> String {
    match context.scope() {
        action::Scope::Path(path) => path_name(path),
        action::Scope::Window => "window".to_owned(),
    }
}

fn path_name(path: &ui::Path) -> String {
    if path == &document_path() {
        "document".to_owned()
    } else if path == &local_field_path() {
        "local responder".to_owned()
    } else {
        path.leaf()
            .map(ui::Id::as_str)
            .unwrap_or("unknown")
            .to_owned()
    }
}

fn subject_background(active: bool, theme: &Theme) -> paint::Brush {
    if active {
        paint::Brush::linear_gradient(
            paint::Color::rgba(0.020, 0.032, 0.055, 0.98),
            paint::Color::rgba(0.014, 0.023, 0.040, 0.98),
        )
    } else {
        theme.surfaces().panel()
    }
}

fn subject_stroke(theme: &Theme) -> paint::Stroke {
    paint::Stroke {
        brush: paint::Brush::solid(theme.palette().accent()),
        width: 1.0,
    }
}

fn command_scope_path() -> ui::Path {
    ui::Path::new([ROOT, COMMAND_SCOPE_PANEL])
}

fn document_path() -> ui::Path {
    ui::Path::new([ROOT, DOCUMENT_PANEL])
}

fn scroll_path() -> ui::Path {
    ui::Path::new([ROOT, DOCUMENT_SCROLL])
}

fn local_field_path() -> ui::Path {
    ui::Path::new([ROOT, COMMAND_SCOPE_PANEL, LOCAL_FIELD])
}

fn label(label: impl Into<String>, theme: &Theme) -> text::Document {
    let mut block = text::Block::new(text::Align::Center);
    block.push_run(text::Run::new(
        label,
        text::Style::default()
            .with_size(theme.text().body_size())
            .with_color(theme.text().primary()),
    ));

    text::Document::from_block(block)
}
