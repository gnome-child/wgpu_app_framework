use std::time::Duration;

use wgpu_l3::{Action, Event, Task, action, app, geometry::area, layout, paint, text, ui, window};

const RUN_TASK: action::Id = action::Id::new("run_task");
const TOGGLE_PREVIEW: action::Id = action::Id::new("toggle_preview");
const ROOT: ui::Id = ui::Id::new("root");
const STATUS_PANEL: ui::Id = ui::Id::new("status_panel");
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
}

enum AppEvent {
    WorkspaceReady,
    RunTaskFinished,
    TogglePreview,
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

        let window = cx.open_window(window::Options {
            title: "wgpu_l3".to_owned(),
            inner_area: area::physical(512, 512),
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

        let status = if self.workspace_ready {
            format!(
                "Workspace ready - runs: {} - preview {}",
                self.run_count,
                if self.preview_enabled { "on" } else { "off" }
            )
        } else {
            "Loading workspace...".to_owned()
        };
        let root = ui::control::panel(ROOT)
            .with_background(paint::Color::BLACK)
            .with_padding(layout::Insets::splat(16.0))
            .with_child(
                ui::control::panel(STATUS_PANEL)
                    .with_size(layout::Size::Fill, layout::Size::Fixed(96.0))
                    .with_label(label(status)),
            )
            .with_child(ui::control::labeled_button(
                RUN_BUTTON, RUN_TASK, "Run task",
            ))
            .with_child(ui::control::labeled_button(
                PREVIEW_BUTTON,
                TOGGLE_PREVIEW,
                "Preview",
            ));

        tree.set_root(root);
    }
}

fn label(label: impl Into<String>) -> text::Document {
    let mut block = text::Block::new(text::Align::Center);
    block.push_run(text::Run::new(label, text::Style::default()));

    text::Document::from_block(block)
}
