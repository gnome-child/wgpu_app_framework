use std::time::Duration;

use wgpu_l3::{
    Action, Event, Icon, Task, action, app,
    geometry::{Rect, area, point, rect},
    icon, layout, paint, text, ui, window,
};

const RUN_TASK: action::Id = action::Id::new("run_task");
const TOGGLE_PREVIEW: action::Id = action::Id::new("toggle_preview");
const ROOT: ui::Id = ui::Id::new("root");
const STATUS_PANEL: ui::Id = ui::Id::new("status_panel");
const DOCUMENT_PANEL: ui::Id = ui::Id::new("document_panel");
const SELECT_BUTTON: ui::Id = ui::Id::new("select_button");
const COMMAND_SCOPE_PANEL: ui::Id = ui::Id::new("command_scope_panel");
const LOCAL_FIELD: ui::Id = ui::Id::new("local_field");
const LOCAL_SELECT_BUTTON: ui::Id = ui::Id::new("local_select_button");
const CAPTURED_SELECT_BUTTON: ui::Id = ui::Id::new("captured_select_button");
const RUN_BUTTON: ui::Id = ui::Id::new("run_button");
const PREVIEW_BUTTON: ui::Id = ui::Id::new("preview_button");

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

        let window = cx.open_window(window::Options {
            title: "wgpu_l3".to_owned(),
            inner_area: area::physical(512, 560),
            canvas_color: paint::Color::BLACK,
        });

        self.window = Some(window);

        cx.spawn(Task::future(async {
            std::thread::sleep(Duration::from_millis(700));
            AppEvent::WorkspaceReady
        }));
    }

    fn event(&mut self, cx: &mut app::Context<'_, Self::Event>, event: Event<Self::Event>) {
        let Event::App(event) = event else {
            return;
        };

        let Some(window) = self.window else {
            return;
        };

        match event {
            AppEvent::WorkspaceReady => {
                self.workspace_ready = true;
                cx.focus(window, document_path(), ui::focus::Visibility::Visible);
                cx.request_redraw(window);
            }
            AppEvent::RunTaskFinished => {
                self.run_count += 1;
                cx.request_redraw(window);
            }
            AppEvent::TogglePreview => {
                self.preview_enabled = !self.preview_enabled;
                cx.request_redraw(window);
            }
            AppEvent::SelectAll { target, origin } => {
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

        let mut document_panel = ui::control::panel(DOCUMENT_PANEL)
            .with_size(layout::Size::Fill, layout::Size::Fixed(80.0))
            .with_background(subject_background(document_is_subject))
            .with_interactivity(
                ui::Interactivity::NONE
                    .with_hit_test(true)
                    .with_focusable(true),
            )
            .with_responder(action::SELECT_ALL)
            .with_label(label(document_label));
        if document_is_subject {
            document_panel = document_panel.with_stroke(subject_stroke());
        }

        let mut local_field = ui::control::panel(LOCAL_FIELD)
            .with_size(layout::Size::Fill, layout::Size::Fixed(48.0))
            .with_background(subject_background(local_is_subject))
            .with_interactivity(
                ui::Interactivity::NONE
                    .with_hit_test(true)
                    .with_focusable(true),
            )
            .with_responder(action::SELECT_ALL)
            .with_label(label(if local_is_subject {
                "Local responder | current subject"
            } else {
                "Local responder"
            }));
        if local_is_subject {
            local_field = local_field.with_stroke(subject_stroke());
        }

        let preview_icon = if self.preview_enabled {
            Icon::phosphor(icon::Id::new("eye"))
        } else {
            Icon::phosphor(icon::Id::new("eye-slash"))
        };
        let popup_panel = ui::control::panel(COMMAND_SCOPE_PANEL)
            .with_command_scope()
            .with_backdrop(
                ui::Backdrop::glass(paint::Color::rgba(0.12, 0.13, 0.15, 0.22)).with_blur(1.0),
            )
            .with_radius(rect::Radius::splat(0.12))
            .with_stroke(paint::Stroke {
                brush: paint::Brush::Solid(paint::Color::rgba(1.0, 1.0, 1.0, 0.14)),
                width: 1.0,
            })
            .with_shadow(
                paint::Color::rgba(0.0, 0.0, 0.0, 0.36),
                18.0,
                1.0,
                point::logical(0.0, 7.0),
            )
            .with_padding(layout::Insets::splat(10.0))
            .with_child(
                ui::control::panel(ui::Id::new("scope_status"))
                    .with_size(layout::Size::Fill, layout::Size::Fixed(32.0))
                    .with_label(label(scope_label)),
            )
            .with_child(local_field)
            .with_child(
                ui::control::labeled_button(
                    LOCAL_SELECT_BUTTON,
                    action::SELECT_ALL,
                    "Select current subject",
                )
                .with_action_target(ui::ActionTarget::Command)
                .with_size(layout::Size::Fill, layout::Size::Fixed(46.0)),
            )
            .with_child(
                ui::control::labeled_button(
                    CAPTURED_SELECT_BUTTON,
                    action::SELECT_ALL,
                    "Select captured subject",
                )
                .with_action_target(ui::ActionTarget::Captured)
                .with_size(layout::Size::Fill, layout::Size::Fixed(46.0)),
            );
        let root = ui::control::panel(ROOT)
            .with_background(paint::Color::BLACK)
            .with_padding(layout::Insets::splat(16.0))
            .with_child(
                ui::control::panel(STATUS_PANEL)
                    .with_size(layout::Size::Fill, layout::Size::Fixed(64.0))
                    .with_label(label(status)),
            )
            .with_child(document_panel)
            .with_child(
                ui::control::labeled_button(SELECT_BUTTON, action::SELECT_ALL, "Select subject")
                    .with_action_target(ui::ActionTarget::Command)
                    .with_size(layout::Size::Fill, layout::Size::Fixed(48.0)),
            )
            .with_child(
                ui::control::labeled_button(RUN_BUTTON, RUN_TASK, "Run task")
                    .with_size(layout::Size::Fill, layout::Size::Fixed(48.0)),
            )
            .with_child(
                ui::control::panel(ui::Id::new("footer_panel"))
                    .with_size(layout::Size::Fill, layout::Size::Fixed(36.0))
                    .with_label(label(footer)),
            )
            .with_child(
                ui::control::icon_button(PREVIEW_BUTTON, TOGGLE_PREVIEW, preview_icon)
                    .with_size(layout::Size::Fill, layout::Size::Fixed(40.0)),
            );

        tree.set_root(root);
        tree.clear_popups();
        tree.push_popup(ui::Popup::new(
            Rect::new(point::logical(40.0, 294.0), area::logical(432.0, 200.0)),
            popup_panel,
        ));
    }
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

fn subject_background(active: bool) -> paint::Color {
    if active {
        paint::Color::rgb(0.16, 0.23, 0.32)
    } else {
        paint::Color::rgb(0.18, 0.20, 0.24)
    }
}

fn subject_stroke() -> paint::Stroke {
    paint::Stroke {
        brush: paint::Brush::Solid(paint::Color::rgb(0.34, 0.62, 1.0)),
        width: 2.0,
    }
}

fn command_scope_path() -> ui::Path {
    ui::Path::new([ROOT, COMMAND_SCOPE_PANEL])
}

fn document_path() -> ui::Path {
    ui::Path::new([ROOT, DOCUMENT_PANEL])
}

fn local_field_path() -> ui::Path {
    ui::Path::new([ROOT, COMMAND_SCOPE_PANEL, LOCAL_FIELD])
}

fn label(label: impl Into<String>) -> text::Document {
    let mut block = text::Block::new(text::Align::Center);
    block.push_run(text::Run::new(label, text::Style::default()));

    text::Document::from_block(block)
}
